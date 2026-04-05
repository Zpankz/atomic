//! Section-level operations for incremental wiki updates.
//!
//! Rather than asking the LLM to fully rewrite an article on every update, we
//! ask it to emit a list of structured operations against the existing article.
//! The applier merges them in. Untouched sections stay byte-identical, which
//! makes the review diff localized to what actually changed and preserves the
//! existing citation graph.

use serde::{Deserialize, Serialize};

/// A single operation against an existing wiki article.
///
/// Headings in `AppendToSection` / `ReplaceSection` must exactly match one of
/// the existing `##` or `###` headings (trimmed, case-sensitive). A missing
/// heading is treated as a hallucination and causes the whole proposal to be
/// discarded — we do not fuzzy-match.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "op")]
pub enum WikiSectionOp {
    /// No change — the new sources don't warrant updating the article.
    NoChange,
    /// Append `content` to the body of an existing section.
    AppendToSection { heading: String, content: String },
    /// Replace the body of an existing section (heading line preserved).
    ReplaceSection { heading: String, content: String },
    /// Insert a brand-new section. If `after_heading` is `None`, the section
    /// is appended to the end of the document.
    InsertSection {
        after_heading: Option<String>,
        heading: String,
        content: String,
    },
}

/// Internal representation of a parsed section.
#[derive(Debug, Clone)]
struct Section {
    /// Markdown level (2 for `##`, 3 for `###`).
    level: u8,
    /// Heading text, with `##`/`###` prefix and leading whitespace stripped.
    heading: String,
    /// Body text *without* the heading line, but including any trailing blank
    /// lines so round-tripping stays stable for untouched sections.
    body: String,
}

/// Apply a list of section operations to an existing article body.
///
/// Returns the merged markdown. Errors if any op references a heading that
/// doesn't exist in the article — the caller should log both the missing
/// heading and the list of actual headings, discard the proposal, and return
/// an error to the user.
pub fn apply_section_ops(existing: &str, ops: &[WikiSectionOp]) -> Result<String, String> {
    let (preamble, mut sections) = parse_sections(existing);

    for op in ops {
        match op {
            WikiSectionOp::NoChange => {
                // Tolerate — callers should short-circuit on this, but if a
                // list mixes NoChange with other ops, just skip it.
                continue;
            }
            WikiSectionOp::AppendToSection { heading, content } => {
                let idx = find_section_idx(&sections, heading).ok_or_else(|| {
                    format!(
                        "AppendToSection: heading '{}' not found. Existing headings: [{}]",
                        heading,
                        list_headings(&sections)
                    )
                })?;
                append_to_body(&mut sections[idx].body, content);
            }
            WikiSectionOp::ReplaceSection { heading, content } => {
                let idx = find_section_idx(&sections, heading).ok_or_else(|| {
                    format!(
                        "ReplaceSection: heading '{}' not found. Existing headings: [{}]",
                        heading,
                        list_headings(&sections)
                    )
                })?;
                sections[idx].body = ensure_trailing_blank(content);
            }
            WikiSectionOp::InsertSection {
                after_heading,
                heading,
                content,
            } => {
                let new_section = Section {
                    level: 2,
                    heading: heading.clone(),
                    body: ensure_trailing_blank(content),
                };
                match after_heading {
                    Some(h) => {
                        let idx = find_section_idx(&sections, h).ok_or_else(|| {
                            format!(
                                "InsertSection: after_heading '{}' not found. Existing headings: [{}]",
                                h,
                                list_headings(&sections)
                            )
                        })?;
                        sections.insert(idx + 1, new_section);
                    }
                    None => {
                        sections.push(new_section);
                    }
                }
            }
        }
    }

    Ok(serialize_sections(&preamble, &sections))
}

/// Parse the article into (preamble, sections). The preamble is any content
/// before the first `##` heading. Only `##` (level 2) headings begin new
/// sections; `###` and deeper stay embedded in their parent section's body.
fn parse_sections(content: &str) -> (String, Vec<Section>) {
    let mut preamble = String::new();
    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;

    for line in content.split_inclusive('\n') {
        if let Some((level, heading)) = parse_heading(line) {
            if level == 2 {
                if let Some(sec) = current.take() {
                    sections.push(sec);
                }
                current = Some(Section {
                    level,
                    heading: heading.to_string(),
                    body: String::new(),
                });
                continue;
            }
        }

        match current.as_mut() {
            Some(sec) => sec.body.push_str(line),
            None => preamble.push_str(line),
        }
    }

    if let Some(sec) = current.take() {
        sections.push(sec);
    }

    // Normalize each section's body: strip leading blank lines (the blank
    // between heading and body will be re-emitted during serialization) and
    // ensure a trailing blank-line terminator.
    for sec in &mut sections {
        while sec.body.starts_with('\n') || sec.body.starts_with("\r\n") {
            if sec.body.starts_with("\r\n") {
                sec.body.drain(..2);
            } else {
                sec.body.drain(..1);
            }
        }
        sec.body = ensure_trailing_blank(&sec.body);
    }

    (preamble, sections)
}

/// Parse a line as a markdown heading. Returns (level, heading_text) if the
/// line starts with `## ` or `### ` (etc). Ignores `#` (level 1).
fn parse_heading(line: &str) -> Option<(u8, &str)> {
    let trimmed = line.trim_end_matches(|c| c == '\n' || c == '\r');
    let stripped = trimmed.trim_start();
    let bytes = stripped.as_bytes();
    let mut hashes = 0;
    while hashes < bytes.len() && bytes[hashes] == b'#' {
        hashes += 1;
    }
    if hashes < 2 || hashes > 6 {
        return None;
    }
    if hashes >= bytes.len() || bytes[hashes] != b' ' {
        return None;
    }
    let text = stripped[hashes + 1..].trim();
    Some((hashes as u8, text))
}

fn find_section_idx(sections: &[Section], heading: &str) -> Option<usize> {
    let target = heading.trim();
    sections.iter().position(|s| s.heading.trim() == target)
}

fn list_headings(sections: &[Section]) -> String {
    sections
        .iter()
        .map(|s| format!("'{}'", s.heading))
        .collect::<Vec<_>>()
        .join(", ")
}

fn append_to_body(body: &mut String, content: &str) {
    // Ensure there's a blank line between the existing body and the new content.
    if !body.is_empty() && !body.ends_with("\n\n") {
        if body.ends_with('\n') {
            body.push('\n');
        } else {
            body.push_str("\n\n");
        }
    }
    body.push_str(content.trim_end());
    body.push_str("\n\n");
}

fn ensure_trailing_blank(content: &str) -> String {
    let mut s = content.trim_end().to_string();
    s.push_str("\n\n");
    s
}

fn serialize_sections(preamble: &str, sections: &[Section]) -> String {
    let mut out = String::new();
    out.push_str(preamble);
    // Ensure a blank line between preamble and first section if preamble is non-empty.
    if !preamble.is_empty() && !preamble.ends_with("\n\n") {
        if preamble.ends_with('\n') {
            out.push('\n');
        } else {
            out.push_str("\n\n");
        }
    }
    for sec in sections {
        let hashes = "#".repeat(sec.level as usize);
        // Heading line + mandatory blank line between heading and body.
        out.push_str(&format!("{} {}\n\n", hashes, sec.heading));
        out.push_str(&sec.body);
        // Guarantee separation between sections.
        if !out.ends_with("\n\n") {
            if out.ends_with('\n') {
                out.push('\n');
            } else {
                out.push_str("\n\n");
            }
        }
    }
    // Trim any excess trailing blank lines down to a single trailing newline.
    while out.ends_with("\n\n\n") {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# My Article

Preamble text.

## Overview

Overview body with [1] citation.

## Details

Details body.

### Subsection

Subsection text.

## Status

Status body.
";

    #[test]
    fn no_change_preserves_content() {
        let out = apply_section_ops(SAMPLE, &[WikiSectionOp::NoChange]).unwrap();
        assert_eq!(out.trim(), SAMPLE.trim());
    }

    #[test]
    fn empty_ops_preserves_content() {
        let out = apply_section_ops(SAMPLE, &[]).unwrap();
        assert_eq!(out.trim(), SAMPLE.trim());
    }

    #[test]
    fn append_adds_to_existing_section() {
        let ops = vec![WikiSectionOp::AppendToSection {
            heading: "Details".to_string(),
            content: "New detail [3].".to_string(),
        }];
        let out = apply_section_ops(SAMPLE, &ops).unwrap();
        assert!(out.contains("Details body."));
        assert!(out.contains("### Subsection"));
        assert!(out.contains("New detail [3]."));
        // Overview is untouched and precedes the modified section.
        let overview_pos = out.find("Overview body").unwrap();
        let new_detail_pos = out.find("New detail").unwrap();
        assert!(overview_pos < new_detail_pos);
    }

    #[test]
    fn append_preserves_untouched_sections_byte_for_byte() {
        let ops = vec![WikiSectionOp::AppendToSection {
            heading: "Status".to_string(),
            content: "New status line [3].".to_string(),
        }];
        let out = apply_section_ops(SAMPLE, &ops).unwrap();
        // Overview section must appear exactly as it did in the source.
        assert!(out.contains("## Overview\n\nOverview body with [1] citation."));
    }

    #[test]
    fn replace_swaps_body_but_keeps_heading() {
        let ops = vec![WikiSectionOp::ReplaceSection {
            heading: "Status".to_string(),
            content: "Totally new status [3].".to_string(),
        }];
        let out = apply_section_ops(SAMPLE, &ops).unwrap();
        assert!(out.contains("## Status\n\nTotally new status [3]."));
        assert!(!out.contains("Status body."));
    }

    #[test]
    fn insert_after_specific_heading() {
        let ops = vec![WikiSectionOp::InsertSection {
            after_heading: Some("Overview".to_string()),
            heading: "Background".to_string(),
            content: "Background content [3].".to_string(),
        }];
        let out = apply_section_ops(SAMPLE, &ops).unwrap();
        let overview_pos = out.find("## Overview").unwrap();
        let background_pos = out.find("## Background").unwrap();
        let details_pos = out.find("## Details").unwrap();
        assert!(overview_pos < background_pos);
        assert!(background_pos < details_pos);
        assert!(out.contains("Background content [3]."));
    }

    #[test]
    fn insert_with_none_appends_to_end() {
        let ops = vec![WikiSectionOp::InsertSection {
            after_heading: None,
            heading: "Appendix".to_string(),
            content: "Appendix content [3].".to_string(),
        }];
        let out = apply_section_ops(SAMPLE, &ops).unwrap();
        let status_pos = out.find("## Status").unwrap();
        let appendix_pos = out.find("## Appendix").unwrap();
        assert!(status_pos < appendix_pos);
    }

    #[test]
    fn hallucinated_heading_returns_error() {
        let ops = vec![WikiSectionOp::AppendToSection {
            heading: "Nonexistent".to_string(),
            content: "whatever".to_string(),
        }];
        let err = apply_section_ops(SAMPLE, &ops).unwrap_err();
        assert!(err.contains("Nonexistent"));
        assert!(err.contains("Overview"));
        assert!(err.contains("Details"));
    }

    #[test]
    fn subsection_does_not_split_parent() {
        // Details has a ### Subsection — parsing must keep it inside Details.
        let (_, sections) = parse_sections(SAMPLE);
        let headings: Vec<&str> = sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Overview", "Details", "Status"]);
        let details = sections.iter().find(|s| s.heading == "Details").unwrap();
        assert!(details.body.contains("### Subsection"));
    }

    #[test]
    fn multi_op_sequence_applied_in_order() {
        let ops = vec![
            WikiSectionOp::AppendToSection {
                heading: "Overview".to_string(),
                content: "Added to overview [3].".to_string(),
            },
            WikiSectionOp::InsertSection {
                after_heading: Some("Details".to_string()),
                heading: "Notes".to_string(),
                content: "Notes content [4].".to_string(),
            },
            WikiSectionOp::ReplaceSection {
                heading: "Status".to_string(),
                content: "Replaced status [5].".to_string(),
            },
        ];
        let out = apply_section_ops(SAMPLE, &ops).unwrap();
        assert!(out.contains("Added to overview [3]."));
        assert!(out.contains("## Notes\n\nNotes content [4]."));
        assert!(out.contains("## Status\n\nReplaced status [5]."));
        assert!(!out.contains("Status body."));

        // Verify order: Overview, Details, Notes, Status
        let overview_pos = out.find("## Overview").unwrap();
        let details_pos = out.find("## Details").unwrap();
        let notes_pos = out.find("## Notes").unwrap();
        let status_pos = out.find("## Status").unwrap();
        assert!(overview_pos < details_pos);
        assert!(details_pos < notes_pos);
        assert!(notes_pos < status_pos);
    }

    #[test]
    fn serde_roundtrip_tagged_enum() {
        let ops = vec![
            WikiSectionOp::NoChange,
            WikiSectionOp::AppendToSection {
                heading: "X".into(),
                content: "y".into(),
            },
            WikiSectionOp::ReplaceSection {
                heading: "X".into(),
                content: "y".into(),
            },
            WikiSectionOp::InsertSection {
                after_heading: Some("X".into()),
                heading: "Y".into(),
                content: "z".into(),
            },
            WikiSectionOp::InsertSection {
                after_heading: None,
                heading: "Y".into(),
                content: "z".into(),
            },
        ];
        let json = serde_json::to_string(&ops).unwrap();
        let roundtrip: Vec<WikiSectionOp> = serde_json::from_str(&json).unwrap();
        assert_eq!(ops, roundtrip);
    }
}
