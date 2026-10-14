//! Minimal HTML → Markdown text extraction.
//!
//! Deliberately dependency-free and conservative: it recovers the readable
//! text (headings, paragraphs, list items) and drops everything else, rather
//! than attempting a full HTML→MD transform. Rich fidelity is a separate
//! upgrade item (TECH_SPEC 附录 A).

use crate::errors::AppError;
use crate::models::doc_parse::ParsedDocument;
use std::path::Path;

const SKIP_ELEMENTS: [&str; 4] = ["script", "style", "textarea", "head"];

pub(crate) fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| AppError::Internal(format!("Failed to read {}: {e}", path.display())))?;

    let title = extract_title(&raw);
    let mut warnings = Vec::new();
    if title.is_none() {
        warnings.push("No <title> found; the file name is used as the note title".to_string());
    }

    Ok(ParsedDocument {
        title,
        markdown: html_to_markdown(&raw),
        images: Vec::new(),
        warnings,
    })
}

fn extract_title(raw: &str) -> Option<String> {
    let lower = raw.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let open = raw[start..].find('>')? + start + 1;
    let end = lower[open..].find("</title>")? + open;
    let text = decode_entities(&raw[open..end]);
    let text = text.trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// Convert the HTML body to Markdown-ish text: `#` headings, blank-line
/// separated paragraphs, `- ` list items. Everything else is plain text.
fn html_to_markdown(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    let body = match lower.find("<body") {
        Some(body_start) => {
            let open_end = raw[body_start..]
                .find('>')
                .map(|i| i + body_start + 1)
                .unwrap_or(body_start);
            let body_end = lower
                .rfind("</body")
                .map(|i| body_start + i)
                .unwrap_or(raw.len());
            let start = floor_char_boundary(raw, open_end.min(body_end));
            let end = floor_char_boundary(raw, body_end);
            &raw[start..end]
        }
        None => raw,
    };
    document_to_lines(body)
}

/// Clamp a byte index down to a `char` boundary so slicing never panics on
/// multi-byte input (CJK text is the common case here).
fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Walk the markup, emitting text and recording block boundaries.
fn document_to_lines(raw: &str) -> String {
    let mut out = String::new();
    let mut skip_until: Option<String> = None;
    let bytes = raw.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if let Some(name) = &skip_until {
            let rest = &raw[i..];
            let close = format!("</{name}");
            match rest.to_ascii_lowercase().find(&close) {
                Some(pos) => {
                    i += pos;
                    skip_until = None;
                    if let Some(gt) = raw[i..].find('>') {
                        i += gt + 1;
                    }
                    continue;
                }
                None => return finish(out),
            }
        }

        if bytes[i] == b'<' {
            let Some(gt) = raw[i..].find('>') else { break };
            let tag = raw[i + 1..i + gt].trim();
            i += gt + 1;

            if let Some(name) = tag.strip_prefix('/') {
                let name = name.trim().to_ascii_lowercase();
                if matches!(
                    name.as_str(),
                    "p" | "div" | "section" | "article" | "li" | "tr" | "blockquote" | "table"
                ) {
                    break_paragraph(&mut out);
                }
                continue;
            }

            let lower = tag.to_ascii_lowercase();
            let element = lower
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_end_matches('/');

            if SKIP_ELEMENTS.contains(&element) && !element.is_empty() {
                skip_until = Some(element.to_string());
                continue;
            }
            if let Some(level) = heading_level(element) {
                out.push_str(&"#".repeat(level));
                out.push(' ');
                continue;
            }
            if element == "li" {
                break_paragraph(&mut out);
                out.push_str("- ");
                continue;
            }
            if element == "br" {
                out.push('\n');
                continue;
            }
            continue;
        }

        let next = raw[i..].find('<').unwrap_or(raw.len() - i);
        push_text(&mut out, &raw[i..i + next]);
        i += next;
    }

    finish(out)
}

fn finish(out: String) -> String {
    let mut out = out;
    while out.ends_with('\n') || out.ends_with(' ') {
        out.pop();
    }
    out
}

/// `h1`..`h6` → 1..6; anything else (`hgroup`, `hr`, …) → None.
fn heading_level(element: &str) -> Option<usize> {
    if element.len() != 2 || !element.starts_with('h') {
        return None;
    }
    element[1..2]
        .parse::<usize>()
        .ok()
        .filter(|n| (1..=6).contains(n))
}

fn break_paragraph(out: &mut String) {
    while out.ends_with('\n') {
        out.pop();
    }
    if !out.is_empty() {
        out.push_str("\n\n");
    }
}

fn push_text(out: &mut String, raw: &str) {
    let text = decode_entities(raw);
    let text = text.trim_matches(char::is_whitespace);
    if text.is_empty() {
        return;
    }
    if !out.is_empty() && !out.ends_with(['\n', ' ', '-']) {
        out.push(' ');
    }
    out.push_str(text);
}

/// Decode the handful of entities that dominate real documents; anything else
/// is left verbatim (still readable text, and no data loss).
fn decode_entities(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let mut entity = String::new();
        let mut closed = false;
        while let Some(&n) = chars.peek() {
            if n == ';' && !entity.is_empty() && entity.len() <= 10 {
                chars.next();
                closed = true;
                break;
            }
            if !(n.is_ascii_alphanumeric() || n == '#' || n == 'x') {
                break;
            }
            entity.push(n);
            chars.next();
        }
        match (closed, entity.as_str()) {
            (_, "amp") => out.push('&'),
            (_, "lt") => out.push('<'),
            (_, "gt") => out.push('>'),
            (_, "quot") => out.push('"'),
            (_, "apos") => out.push('\''),
            (_, "nbsp") => out.push(' '),
            (_, rest) if rest.starts_with('#') => {
                let code = rest
                    .strip_prefix('#')
                    .and_then(|r| match r.strip_prefix('x') {
                        Some(hex) => u32::from_str_radix(&hex.to_ascii_lowercase(), 16).ok(),
                        None => r.parse().ok(),
                    });
                out.push(char::from_u32(code.unwrap_or(0xfffd)).unwrap_or('\u{fffd}'));
            }
            _ => {
                out.push('&');
                out.push_str(&entity);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-html-tests")
            .join(name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn parse_string(name: &str, html: &str) -> ParsedDocument {
        let dir = temp_dir(name);
        let path = dir.join("page.html");
        std::fs::write(&path, html).expect("write");
        parse(&path).expect("parse")
    }

    #[test]
    fn extracts_title_headings_and_paragraphs() {
        let doc = parse_string(
            "extracts_title_headings_and_paragraphs",
            r#"<html><head><title>Rust 课程</title></head>
            <body><h1>所有权</h1><p>Rust 没有垃圾回收。</p><h2>借用</h2><p>借用不同时可变。</p></body></html>"#,
        );

        assert_eq!(doc.title.as_deref(), Some("Rust 课程"));
        assert!(doc.markdown.contains("# 所有权"), "got: {}", doc.markdown);
        assert!(doc.markdown.contains("Rust 没有垃圾回收。"));
        assert!(doc.markdown.contains("## 借用"), "got: {}", doc.markdown);
    }

    #[test]
    fn skips_script_style_and_head() {
        let doc = parse_string(
            "skips_script_style_and_head",
            r#"<html><head><title>t</title><style>body{color:red}</style><script>var x=1;</script></head>
            <body><script>alert("x")</script><p>visible</p></body></html>"#,
        );

        assert!(!doc.markdown.contains("color:red"), "style leaked");
        assert!(!doc.markdown.contains("alert"), "script leaked");
        assert!(!doc.markdown.contains("var x"), "head leaked");
        assert!(doc.markdown.contains("visible"), "got: {}", doc.markdown);
    }

    #[test]
    fn decodes_entities() {
        let doc = parse_string(
            "decodes_entities",
            r#"<html><body><p>a &amp; b &lt;c&gt; &quot;d&quot; &#20320;&#x597D;</p></body></html>"#,
        );
        assert!(
            doc.markdown.contains("a & b <c> \"d\" 你好"),
            "got: {}",
            doc.markdown
        );
    }

    #[test]
    fn handles_missing_body_and_title() {
        let doc = parse_string("handles_missing_body_and_title", "<p>only a fragment</p>");
        assert!(doc.title.is_none());
        assert!(
            doc.markdown.contains("only a fragment"),
            "got: {}",
            doc.markdown
        );
        assert!(!doc.warnings.is_empty(), "missing title should warn");
    }

    #[test]
    fn does_not_treat_hgroup_or_hr_as_heading() {
        let doc = parse_string(
            "does_not_treat_hgroup_or_hr_as_heading",
            r#"<html><body><hr><hgroup><p>group</p></hgroup></body></html>"#,
        );
        assert!(!doc.markdown.starts_with('#'), "got: {}", doc.markdown);
    }
}
