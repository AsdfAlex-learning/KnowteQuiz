//! PDF text-layer extraction.
//!
//! v1 is text-only by design: `pdf-extract` recovers the text layer but not
//! tables, columns or vector figures. That boundary is documented in TECH_SPEC
//! 附录 A, and the caller surfaces the warning to the user rather than
//! pretending the conversion is lossless.

use crate::errors::AppError;
use crate::models::doc_parse::ParsedDocument;
use std::path::Path;

pub(crate) fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| AppError::Internal(format!("Failed to extract PDF text: {e}")))?;

    let text = text.trim().to_string();
    let mut warnings = Vec::new();

    if text.is_empty() {
        warnings.push(
            "This PDF has no text layer (a scanned document?). OCR is not enabled in this version."
                .to_string(),
        );
    } else {
        warnings.push(
            "PDF conversion is text-only: tables, multi-column layout and figures are not recovered."
                .to_string(),
        );
    }

    // A first line that looks like a heading is a good best-effort title.
    let title = text
        .lines()
        .map(str::trim)
        .find(|line| line.len() >= 4 && line.len() <= 200)
        .map(String::from);

    Ok(ParsedDocument {
        title,
        markdown: text,
        images: Vec::new(),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-pdf-tests")
            .join(name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    /// A minimal but structurally valid single-page PDF with a text layer.
    /// Stream length is computed from the content so the xref/trailer agree.
    fn minimal_pdf() -> String {
        let stream = "BT /F1 12 Tf 10 100 Td (Hello KnowteQuiz PDF) Tj ET";
        let objects: Vec<String> = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 144] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".to_string(),
            format!(
                "<< /Length {} >>\nstream\n{}\nendstream",
                stream.len(),
                stream
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        ];
        build_pdf(&objects)
    }

    /// Assemble a PDF with a correct xref table and trailer.
    fn build_pdf(objects: &[String]) -> String {
        let mut body = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for (index, object) in objects.iter().enumerate() {
            offsets.push(body.len());
            body.push_str(&format!("{} 0 obj\n{}\nendobj\n", index + 1, object));
        }
        let xref_start = body.len();
        body.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
        body.push_str("0000000000 65535 f \n");
        for offset in offsets {
            body.push_str(&format!("{:010} 00000 n \n", offset));
        }
        body.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len() + 1,
            xref_start
        ));
        body
    }

    /// A structurally valid PDF whose page has no content stream (no text layer).
    fn empty_page_pdf() -> String {
        let objects: Vec<String> = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 144] >>".to_string(),
        ];
        build_pdf(&objects)
    }

    #[test]
    fn extracts_text_and_warns_about_text_only() {
        let dir = temp_dir("extracts_text_and_warns_about_text_only");
        let path = dir.join("hello.pdf");
        std::fs::write(&path, minimal_pdf()).expect("pdf should be written");

        let doc = parse(&path).expect("parse should succeed");
        assert!(
            doc.markdown.contains("Hello KnowteQuiz PDF"),
            "got: {}",
            doc.markdown
        );
        assert!(doc.warnings.iter().any(|w| w.contains("text-only")));
        assert_eq!(doc.images.len(), 0, "pdf images are an upgrade item");
    }

    #[test]
    fn empty_text_layer_warns_about_ocr() {
        let dir = temp_dir("empty_text_layer_warns_about_ocr");
        let path = dir.join("empty.pdf");
        std::fs::write(&path, empty_page_pdf()).expect("pdf should be written");

        let doc = parse(&path).expect("parse should succeed");
        assert!(
            doc.warnings.iter().any(|w| w.contains("no text layer")),
            "got: {:?}",
            doc.warnings
        );
    }
}
