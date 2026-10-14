//! OOXML (`.docx` / `.pptx`) → Markdown, via `zip` + `quick-xml`.
//!
//! An OOXML file is a zip of XML parts. We read the main document part and its
//! relationships, walk paragraphs/headings/tables, and recover the embedded
//! images from the archive. Layout fidelity is best-effort — merged cells,
//! text boxes and SmartArt are flattened or skipped (TECH_SPEC 附录 A).

use crate::errors::AppError;
use crate::models::doc_parse::{DocumentFormat, ParsedDocument, ParsedImage};
use crate::services::doc_parser::IMAGE_REF_TOKEN;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Read, Seek};
use zip::ZipArchive;

pub(crate) fn parse<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    format: DocumentFormat,
) -> Result<ParsedDocument, AppError> {
    match format {
        DocumentFormat::Docx => parse_docx(archive),
        DocumentFormat::Pptx => parse_pptx(archive),
        _ => Err(AppError::InvalidInput(
            "ooxml parser only handles DOCX and PPTX".to_string(),
        )),
    }
}

// ── archive helpers ───────────────────────────────────────────────────────

fn read_part<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Option<String> {
    let mut file = archive.by_name(name).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    Some(buf)
}

/// `.rels` part → `rId → target path inside the archive`.
fn parse_relationships<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    rels_name: &str,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(xml) = read_part(archive, rels_name) else {
        return map;
    };
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().local_name().as_ref() == b"Relationship" => {
                let mut id = None;
                let mut target = None;
                for attr in e.attributes().with_checks(false).flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    if key.ends_with("Id") {
                        id = Some(value);
                    } else if key.ends_with("Target") {
                        target = Some(value);
                    }
                }
                if let (Some(id), Some(target)) = (id, target) {
                    map.insert(id, target);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    map
}

/// Collect `word/media/*` or `ppt/media/*` entries keyed by basename.
fn collect_media<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    dir: &str,
) -> BTreeMap<String, Vec<u8>> {
    let names: Vec<String> = archive
        .file_names()
        .filter(|n| n.starts_with(dir) && !n.ends_with('/'))
        .map(|n| n.to_string())
        .collect();
    let mut out = BTreeMap::new();
    for name in names {
        let mut file = match archive.by_name(&name) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_ok() {
            let base = name.rsplit('/').next().unwrap_or("").to_string();
            out.insert(base, bytes);
        }
    }
    out
}

// ── shared walk state ─────────────────────────────────────────────────────

struct Walk {
    rels: HashMap<String, String>,
    media: BTreeMap<String, Vec<u8>>,
    markdown: String,
    images: Vec<ParsedImage>,
    warnings: Vec<String>,
    in_table: bool,
    used_media: HashSet<String>,
    title_hint: Option<String>,
    kind: &'static str,
}

impl Walk {
    fn new(
        rels: HashMap<String, String>,
        media: BTreeMap<String, Vec<u8>>,
        kind: &'static str,
    ) -> Self {
        Walk {
            rels,
            media,
            markdown: String::new(),
            images: Vec::new(),
            warnings: Vec::new(),
            in_table: false,
            used_media: HashSet::new(),
            title_hint: None,
            kind,
        }
    }

    fn flush_paragraph(&mut self) {
        if self.in_table {
            return;
        }
        while self.markdown.ends_with('\n') {
            self.markdown.pop();
        }
        if !self.markdown.is_empty() {
            self.markdown.push_str("\n\n");
        }
    }

    /// Emit `![](TOKEN/<basename>)` for an image relationship, staging the
    /// bytes for the converter to write next to the note.
    fn push_image_ref(&mut self, rel_id: &str) {
        let Some(target) = self.rels.get(rel_id).cloned() else {
            self.warnings
                .push(format!("Unresolved image relationship {rel_id}"));
            return;
        };
        let base = target.rsplit(['/', '\\']).next().unwrap_or("").to_string();
        if base.is_empty() {
            return;
        }
        if !self.used_media.contains(&base) {
            match self.media.get(&base) {
                Some(bytes) => {
                    self.images.push(ParsedImage {
                        ref_name: base.clone(),
                        bytes: bytes.clone(),
                    });
                    self.used_media.insert(base.clone());
                }
                None => {
                    self.warnings
                        .push(format!("Image part {base} missing from the archive"));
                }
            }
        }
        self.markdown
            .push_str(&format!("![{}]({}{})\n\n", base, IMAGE_REF_TOKEN, base));
    }

    fn finish(mut self) -> ParsedDocument {
        self.flush_paragraph();
        if self.media.is_empty() {
            self.warnings
                .push(format!("No embedded images found in the {}", self.kind));
        }
        while self.markdown.ends_with('\n') {
            self.markdown.pop();
        }
        ParsedDocument {
            title: self.title_hint,
            markdown: self.markdown,
            images: self.images,
            warnings: self.warnings,
        }
    }
}

// ── DOCX ──────────────────────────────────────────────────────────────────

fn parse_docx<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<ParsedDocument, AppError> {
    let document = read_part(archive, "word/document.xml").ok_or_else(|| {
        AppError::Internal("Not a valid DOCX: missing word/document.xml".to_string())
    })?;
    let rels = parse_relationships(archive, "word/_rels/document.xml.rels");
    let media = collect_media(archive, "word/media/");

    let mut walk = Walk::new(rels, media, "DOCX");
    walk_word(&document, &mut walk);
    Ok(walk.finish())
}

fn walk_word(xml: &str, walk: &mut Walk) {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut in_ppr = false;
    let mut heading_level: Option<usize> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "w:pPr" => in_ppr = true,
                    "w:tbl" => walk.in_table = true,
                    "w:tr" => walk.markdown.push_str("\n|"),
                    "w:tc" => walk.markdown.push(' '),
                    "a:blip" | "v:imagedata" => grab_rid(&e, walk),
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "w:pStyle" => {
                        for attr in e.attributes().with_checks(false).flatten() {
                            if attr.key.local_name().as_ref() == b"val" {
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                heading_level = style_to_heading(&value);
                            }
                        }
                    }
                    "w:tbl" => walk.in_table = true,
                    "a:blip" | "v:imagedata" => grab_rid(&e, walk),
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "w:pPr" => in_ppr = false,
                    "w:p" => {
                        if let Some(level) = heading_level.take() {
                            let text = prefix_heading(&mut walk.markdown, level);
                            if walk.title_hint.is_none() && level == 1 {
                                walk.title_hint = Some(text);
                            }
                        }
                        walk.flush_paragraph();
                    }
                    "w:tbl" => {
                        walk.in_table = false;
                        walk.flush_paragraph();
                    }
                    "w:tr" => walk.markdown.push('|'),
                    "w:tc" => walk.markdown.push('|'),
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if in_ppr {
                    continue;
                }
                let text = String::from_utf8_lossy(&t.into_inner()).to_string();
                if text.is_empty() {
                    continue;
                }
                if walk.in_table {
                    walk.markdown.push_str(&text.replace('|', "\\|"));
                    walk.markdown.push(' ');
                } else {
                    walk.markdown.push_str(&text);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
}

/// Pull the `r:id`/`embed`/`id` attribute off an image element and resolve it.
fn grab_rid(e: &quick_xml::events::BytesStart<'_>, walk: &mut Walk) {
    for attr in e.attributes().with_checks(false).flatten() {
        let local = attr.key.local_name();
        if local.as_ref() == b"embed" || local.as_ref() == b"id" {
            let rid = String::from_utf8_lossy(&attr.value).to_string();
            walk.push_image_ref(&rid);
        }
    }
}

/// `Heading1`..`Heading6` / `Title` → level; otherwise `None`.
fn style_to_heading(style: &str) -> Option<usize> {
    if let Some(rest) = style.strip_prefix("Heading") {
        if let Ok(n) = rest.parse::<usize>() {
            return Some(n.clamp(1, 6));
        }
    }
    (style == "Title").then_some(1)
}

/// Retroactively turn the buffered paragraph into a heading, returning the
/// heading text.
fn prefix_heading(markdown: &mut String, level: usize) -> String {
    let trimmed = markdown.trim_end();
    let start = trimmed.rfind("\n\n").map(|i| i + 2).unwrap_or(0);
    let paragraph = trimmed[start..].trim().to_string();
    let heading = format!("{} {}\n", "#".repeat(level.clamp(1, 6)), paragraph);
    markdown.truncate(trimmed.len() - paragraph.len());
    while markdown.ends_with('\n') {
        markdown.pop();
    }
    markdown.push_str(&heading);
    markdown.push('\n');
    paragraph
}

// ── PPTX ──────────────────────────────────────────────────────────────────

fn parse_pptx<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<ParsedDocument, AppError> {
    let mut slides: Vec<String> = archive
        .file_names()
        .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
        .map(|n| n.to_string())
        .collect();
    if slides.is_empty() {
        return Err(AppError::Internal(
            "Not a valid PPTX: no slides found".to_string(),
        ));
    }
    slides.sort();

    let media = collect_media(archive, "ppt/media/");
    let mut walk = Walk::new(HashMap::new(), media, "PPTX");

    for (index, slide) in slides.iter().enumerate() {
        let Some(xml) = read_part(archive, slide) else {
            walk.warnings
                .push(format!("Could not read slide part {slide}"));
            continue;
        };
        let rels_name = slide
            .replace("ppt/slides/slide", "ppt/slides/_rels/slide")
            .replace(".xml", ".xml.rels");
        walk.rels = parse_relationships(archive, &rels_name);

        walk.flush_paragraph();
        walk.markdown
            .push_str(&format!("## Slide {}\n\n", index + 1));
        let before = walk.markdown.len();
        walk_slide(&xml, &mut walk);
        if walk.markdown.len() == before {
            walk.markdown.push_str("_No text content._\n\n");
        }
        if walk.title_hint.is_none() {
            walk.title_hint = first_slide_title(&walk.markdown);
        }
    }

    Ok(walk.finish())
}

fn first_slide_title(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .skip_while(|l| !l.starts_with("## Slide"))
        .skip(1)
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().chars().take(120).collect())
}

fn walk_slide(xml: &str, walk: &mut Walk) {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut paragraph_open = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "a:p" => paragraph_open = true,
                    "a:blip" => grab_rid(&e, walk),
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "a:p" {
                    paragraph_open = false;
                    walk.flush_paragraph();
                }
            }
            Ok(Event::Text(t)) => {
                if !paragraph_open {
                    continue;
                }
                let text = String::from_utf8_lossy(&t.into_inner()).to_string();
                if !text.trim().is_empty() {
                    walk.markdown.push_str(text.trim());
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    /// Build a minimal but structurally valid DOCX in memory.
    fn minimal_docx() -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut cursor);
            let options: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
            writer
                .start_file("[Content_Types].xml", options)
                .expect("content types");
            writer
                .write_all(br#"<?xml version="1.0"?><Types/>"#)
                .unwrap();
            writer
                .start_file("word/document.xml", options)
                .expect("document.xml");
            writer
                .write_all(
                    br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Design Review</w:t></w:r></w:p>
    <w:p><w:r><w:t>Ownership is a compile-time guarantee.</w:t></w:r></w:p>
    <w:tbl>
      <w:tr><w:tc><w:r><w:t>Term</w:t></w:r></w:tc><w:tc><w:r><w:t>Meaning</w:t></w:r></w:tc></w:tr>
      <w:tr><w:tc><w:r><w:t>Borrow</w:t></w:r></w:tc><w:tc><w:r><w:t>A reference</w:t></w:r></w:tc></w:tr>
    </w:tbl>
  </w:body>
</w:document>"#,
                )
                .unwrap();
            writer.finish().expect("zip");
        }
        cursor.into_inner()
    }

    #[test]
    fn docx_extracts_headings_paragraphs_and_tables() {
        let cursor = Cursor::new(minimal_docx());
        let mut archive = ZipArchive::new(cursor).expect("zip opens");
        let doc = parse_docx(&mut archive).expect("parse");

        assert_eq!(doc.title.as_deref(), Some("Design Review"));
        assert!(
            doc.markdown.contains("# Design Review"),
            "got: {}",
            doc.markdown
        );
        assert!(doc
            .markdown
            .contains("Ownership is a compile-time guarantee."));
        assert!(doc.markdown.contains("Term"), "table cell expected");
        assert!(doc.markdown.contains('|'), "table pipes expected");
        assert!(
            !doc.markdown.contains("w:") && !doc.markdown.contains("<?xml"),
            "markup must not leak: {}",
            doc.markdown
        );
    }

    #[test]
    fn ooxml_parser_rejects_non_ooxml_format() {
        let cursor = Cursor::new(minimal_docx());
        let mut archive = ZipArchive::new(cursor).expect("zip opens");
        let err = parse(&mut archive, DocumentFormat::Pdf).expect_err("pdf is not ooxml");
        assert!(err.to_string().contains("only handles DOCX and PPTX"));
    }
}
