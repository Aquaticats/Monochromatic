//! What:
//!  Existing bare-URL and shortcut-reference normalization.
//! Why:
//!  Markdown and MDX require different safe link spellings,
//!  while reference identity stays unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // MD034 and MD054 over parser-owned links and references.
//! ```

/// Import the diagnostic and severity types that broken-link findings are reported with.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the edit,
/// fix and class types that a link repair is written with.
use crate::edits::{Edit, Fix, FixClass};
/// Import the finding constructor that wraps a link problem as a located diagnostic.
use crate::markdown_finding::finding;
/// Import the parsed Markdown source whose links are inspected.
use crate::markdown_source::MarkdownSource;
/// Import typed link/reference data and the common result models.
use satteri_ast::mdast::{MdastNodeType, decode_link_data, decode_reference_data};

/// Escape only the punctuation significant to the destination Markdown context.
fn escape(text: &str, reserved: &[char]) -> String {
    let mut output = String::new();
    for character in text.chars() {
        if reserved.contains(&character) {
            output.push('\\');
        }
        output.push(character);
    }
    return output;
}

/// What:
///  Select the target syntax and its explanation together.
/// Why:
///  Each branch returns a complete replacement instead of leaving late-initialized values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bareLinkReplacement(mdx: boolean, written: string, url: string): [string, string];
/// ```
fn bare_link_replacement(mdx: bool, written: &str, url: &str) -> (String, &'static str) {
    if !mdx {
        return (
            format!("<{written}>"),
            "Bare URL; wrap it in angle brackets.",
        );
    }
    // Borrow context-specific punctuation lists; each output string owns its escaped content.
    let label: String = escape(written, &['\\', '[', ']']);
    let destination: String = escape(url, &['\\', '<', '>']);
    return (
        format!("[{label}](<{destination}>)"),
        "Bare URL; use an inline link.",
    );
}

/// Wrap bare supported URLs/emails without turning MDX text into JSX.
pub fn no_bare_urls(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Link {
            continue;
        }
        let written = context.slice(*id);
        if written.starts_with('<') || written.starts_with('[') {
            continue;
        }
        let data = decode_link_data(context.data(*id));
        let url = context.text(data.url);
        let mut wrappable = url.starts_with("mailto:");
        for scheme in ["http://", "https://", "ftp://", "ftps://"] {
            if written.starts_with(scheme) {
                wrappable = true;
            }
        }
        if !wrappable {
            continue;
        }
        let (replacement, message): (String, &str) =
            bare_link_replacement(context.mdx, written, url);
        let (start, end) = context.offsets(*id);
        // A backtick inside the link text is a code span delimiter the autolink swallowed,
        //  so the author wrote code that an earlier unpaired backtick displaced;
        //  wrapping would move the delimiter into the link and the finding stays for a hand fix.
        //  `<url>` is an autolink with the same text and the same destination,
        //  so the fix leaves what a reader reads and follows unchanged.
        let fix: Option<Fix> = if written.contains('`') {
            None
        } else {
            Some(Fix {
                class: FixClass::Fix,
                edits: vec![Edit {
                    start,
                    end,
                    replacement,
                }],
            })
        };
        findings.push(finding(
            context,
            *id,
            "markdown/no-bare-urls",
            severity,
            String::from(message),
            fix,
        ));
    }
    return findings;
}

/// Convert only shortcut references to collapsed references by appending brackets.
pub fn link_image_style(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    for id in context.visible_nodes() {
        let kind = context.kind(*id);
        if kind != MdastNodeType::LinkReference && kind != MdastNodeType::ImageReference {
            continue;
        }
        let data = decode_reference_data(context.data(*id));
        // Sätteri's public codec defines 0 as Shortcut, 1 as Collapsed and 2 as Full.
        if data.reference_kind != 0 {
            continue;
        }
        let (_, end) = context.offsets(*id);
        findings.push(finding(
            context,
            *id,
            "markdown/link-image-style",
            severity,
            String::from("Shortcut reference style; use the collapsed `[label][]` style."),
            // `[label][]` resolves to the same definition and renders the same link,
            //  so the fix is safe.
            Some(Fix {
                class: FixClass::Fix,
                edits: vec![Edit {
                    start: end,
                    end,
                    replacement: String::from("[]"),
                }],
            }),
        ));
    }
    return findings;
}
