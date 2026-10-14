//! Document → Markdown conversion.
//!
//! Pure Rust, no external runtime. Each format has its own parser; the public
//! API writes the converted note and its derived image directory next to the
//! source file, using the deterministic names agreed in TECH_SPEC 附录 A:
//!
//! ```text
//! design.docx            →  design.docx.md
//!                         →  design.docx.images/image001.png
//! ```
//!
//! The mapping `X.ext → X.ext.md` is a pure function, so "is this converted?"
//! and "re-convert" need no bookkeeping table.

mod html;
mod ooxml;
mod pdf;

use crate::errors::AppError;
use crate::models::doc_parse::{ConvertOutcome, DocumentFormat, DocumentStatus, ParsedDocument};
use std::path::{Path, PathBuf};

/// Prefix used by parsers for image references inside Markdown. Replaced with
/// the derived image directory before writing the note. It cannot occur in real
/// document text, so the substitution is unambiguous.
pub(crate) const IMAGE_REF_TOKEN: &str = "__DOC_IMAGE__/";

/// `<parent>/<stem>.<ext>.md` — the converted note for `source`.
pub fn converted_output_path(source: &Path) -> PathBuf {
    let mut name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".to_string());
    name.push_str(".md");
    source.with_file_name(name)
}

/// `<parent>/<stem>.<ext>.images` — the derived directory holding the
/// document's images. Sits inside the note root, so the existing Markdown
/// asset pipeline (root-bounded web asset route, Tauri `convertFileSrc`)
/// resolves it with no changes.
pub fn derived_image_dir(source: &Path) -> PathBuf {
    let mut name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".to_string());
    name.push_str(".images");
    source.with_file_name(name)
}

/// Image-ref prefix used inside the produced Markdown, relative to the note.
fn image_ref_prefix(source: &Path) -> String {
    format!(
        "{}.images/",
        source
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    )
}

pub fn document_status(source: &Path) -> DocumentStatus {
    let output = converted_output_path(source);
    let stale = match (std::fs::metadata(source), std::fs::metadata(&output)) {
        (Ok(src), Ok(out)) => match (src.modified(), out.modified()) {
            (Ok(a), Ok(b)) => a > b,
            _ => false,
        },
        _ => false,
    };
    DocumentStatus {
        source_path: source.to_string_lossy().to_string(),
        converted: output.exists(),
        stale,
        output_path: output
            .exists()
            .then(|| output.to_string_lossy().to_string()),
    }
}

/// Convert a document into a Markdown note next to its source, writing any
/// extracted images into the derived `.images/` directory.
///
/// The parse itself runs in `spawn_blocking` — zip/XML/PDF decoding is CPU-bound
/// synchronous work that would otherwise stall the async runtime.
pub async fn convert_document(source_path: &str) -> Result<ConvertOutcome, AppError> {
    let owned = source_path.to_string();
    tokio::task::spawn_blocking(move || convert_document_blocking(&owned))
        .await
        .map_err(|e| AppError::Internal(format!("Document conversion task failed: {e}")))?
}

fn convert_document_blocking(source_path: &str) -> Result<ConvertOutcome, AppError> {
    let source = Path::new(source_path);
    let format = DocumentFormat::from_extension(
        source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default(),
    )
    .ok_or_else(|| {
        AppError::InvalidInput(format!(
            "Unsupported document type: {} (expected PDF, DOCX, PPTX or HTML)",
            source
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("(none)")
        ))
    })?;

    if !source.exists() {
        return Err(AppError::NotFound(format!(
            "Document does not exist: {source_path}"
        )));
    }

    let parsed = parse_source(source, format)?;

    let output = converted_output_path(source);
    let image_dir = derived_image_dir(source);
    let ref_prefix = image_ref_prefix(source);

    // Rewrite the parser's image tokens to their final, note-relative paths.
    let markdown = parsed.markdown.replace(IMAGE_REF_TOKEN, &ref_prefix);

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Internal(format!("Failed to create {}: {e}", parent.display()))
        })?;
    }
    std::fs::write(&output, &markdown)
        .map_err(|e| AppError::Internal(format!("Failed to write {}: {e}", output.display())))?;

    if !parsed.images.is_empty() {
        std::fs::create_dir_all(&image_dir).map_err(|e| {
            AppError::Internal(format!("Failed to create {}: {e}", image_dir.display()))
        })?;
        for image in &parsed.images {
            // Reject path separators smuggled in by a hostile archive entry.
            if image.ref_name.contains('/')
                || image.ref_name.contains('\\')
                || image.ref_name.contains("..")
            {
                continue;
            }
            std::fs::write(image_dir.join(&image.ref_name), &image.bytes).map_err(|e| {
                AppError::Internal(format!("Failed to write image {}: {e}", image.ref_name))
            })?;
        }
    }

    Ok(ConvertOutcome {
        source_path: source_path.to_string(),
        output_path: output.to_string_lossy().to_string(),
        image_dir: parsed
            .images
            .is_empty()
            .then(|| image_dir.to_string_lossy().to_string()),
        title: parsed.title,
        image_count: parsed.images.len() as u32,
        warnings: parsed.warnings,
    })
}

/// Dispatch to the format parser.
fn parse_source(source: &Path, format: DocumentFormat) -> Result<ParsedDocument, AppError> {
    match format {
        DocumentFormat::Docx | DocumentFormat::Pptx => {
            let file = std::fs::File::open(source).map_err(|e| {
                AppError::Internal(format!("Failed to open {}: {e}", source.display()))
            })?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| AppError::Internal(format!("Not a valid OOXML package: {e}")))?;
            ooxml::parse(&mut archive, format)
        }
        DocumentFormat::Pdf => pdf::parse(source),
        DocumentFormat::Html => html::parse(source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-doc-parser-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn converted_output_path_keeps_the_source_extension() {
        let source = Path::new("/vault/design.docx");
        assert_eq!(
            converted_output_path(source),
            Path::new("/vault/design.docx.md")
        );
        assert_eq!(
            derived_image_dir(source),
            Path::new("/vault/design.docx.images")
        );
    }

    #[test]
    fn document_format_maps_extensions() {
        assert_eq!(
            DocumentFormat::from_extension("docx"),
            Some(DocumentFormat::Docx)
        );
        assert_eq!(
            DocumentFormat::from_extension("DOCX"),
            Some(DocumentFormat::Docx)
        );
        assert_eq!(
            DocumentFormat::from_extension("pptx"),
            Some(DocumentFormat::Pptx)
        );
        assert_eq!(
            DocumentFormat::from_extension("pdf"),
            Some(DocumentFormat::Pdf)
        );
        assert_eq!(
            DocumentFormat::from_extension("htm"),
            Some(DocumentFormat::Html)
        );
        assert_eq!(DocumentFormat::from_extension("png"), None);
        assert_eq!(DocumentFormat::from_extension("md"), None);
    }

    #[test]
    fn document_status_is_false_until_converted_and_goes_stale() {
        let dir = temp_dir("document_status_is_false_until_converted_and_goes_stale");
        let source = dir.join("note.docx");
        std::fs::write(&source, "PK").expect("source should be written");

        let before = document_status(&source);
        assert!(!before.converted, "not converted yet");
        assert!(!before.stale, "nothing to be stale against");

        let output = converted_output_path(&source);
        std::fs::write(&output, "# Converted").expect("output should be written");

        let after = document_status(&source);
        assert!(after.converted, "converted flag must flip");

        // Make the source newer than the note -> stale.
        set_modified(&source, now_plus(120));
        let stale = document_status(&source);
        assert!(stale.stale, "source newer than note must be stale");
    }

    fn now_plus(secs: u64) -> std::time::SystemTime {
        std::time::SystemTime::now() + std::time::Duration::from_secs(secs)
    }

    fn set_modified(path: &Path, when: std::time::SystemTime) {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("file should open");
        file.set_modified(when).expect("mtime should be set");
    }

    #[tokio::test]
    async fn convert_rejects_unknown_extension_and_missing_file() {
        let dir = temp_dir("convert_rejects_unknown_extension_and_missing_file");
        let png = dir.join("image.png");
        std::fs::write(&png, "PNG").expect("png should be written");

        let err = convert_document(&png.to_string_lossy())
            .await
            .expect_err("png must be rejected");
        assert!(
            err.to_string().contains("Unsupported document type"),
            "{err}"
        );

        let err = convert_document(&dir.join("missing.pdf").to_string_lossy())
            .await
            .expect_err("missing file must be rejected");
        assert!(err.to_string().contains("does not exist"), "{err}");
    }
}
