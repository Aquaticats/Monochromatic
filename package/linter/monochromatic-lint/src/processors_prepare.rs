//! What:
//!  Rustdoc hidden-line preparation with explicitly unmapped synthetic main text.
//! Why:
//!  Fragments parse as function bodies without reporting or counting generated scaffolding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Strip hidden markers, keep leading inner docs, wrap only fragments without a real main.
//! ```

/// Import exact line-copy helpers and immutable map models.
use crate::processors_lines::{copy_line, physical_lines};
/// Keep synthetic scaffolding absent from mapped-line records.
use crate::processors_model::{Guard, Mapping, ProcessorLanguage};
/// Detect a real top-level main declaration through native Rust syntax.
use crate::rust_source::RustSource;
/// Native node kinds avoid matches in strings,
///  comments or nested functions.
use ra_ap_syntax::SyntaxKind;
/// Reuse immutable source snapshots.
use std::sync::Arc;

/// Detect top-level authored main,
///  not a main-shaped substring.
fn has_main(source: &Mapping) -> bool {
    let parsed: RustSource = RustSource::new(source.filename.clone(), source.text.clone());
    for node in parsed.syntax().children() {
        if node.kind() != SyntaxKind::FN {
            continue;
        }
        for child in node.children() {
            // Rust's raw identifier spelling r#main declares the same entry-point name.
            if child.kind() == SyntaxKind::NAME
                && (child.text() == "main" || child.text() == "r#main")
            {
                return true;
            }
        }
    }
    return false;
}

/// Strip hidden markers while retaining the exact source spelling after each marker.
pub(crate) fn hidden(parent: &Arc<Mapping>) -> Mapping {
    let mut result: Mapping = crate::processors::child(
        parent,
        parent.filename.clone(),
        ProcessorLanguage::Rust,
        Guard::Prepared,
        0,
    );
    for line in physical_lines(parent.text.as_str()) {
        let written: &str = &parent.text[line.start..line.content_end];
        let trimmed: &str = written.trim_start_matches([' ', '\t']);
        let indent: usize = written.len() - trimmed.len();
        let mut payload: usize = line.start;
        if trimmed.starts_with("# ") || trimmed.starts_with("##") {
            payload += indent + 1;
        }
        if trimmed == "#" {
            payload += indent + 1;
        }
        if trimmed.starts_with("# ") {
            payload += 1;
        }
        // The removed marker stays attached to newly inserted hidden lines.
        let prefix: String = String::from(&parent.text[line.start..payload]);
        copy_line(
            &mut result,
            parent.text.as_str(),
            line.start,
            payload,
            line.end,
            prefix,
        );
    }
    return result;
}

/// Wrap stripped fragments but leave leading //!
///  comments outside the generated function.
pub(crate) fn wrap(parent: &Arc<Mapping>) -> Mapping {
    let mut result: Mapping = crate::processors::child(
        parent,
        parent.filename.clone(),
        ProcessorLanguage::Rust,
        Guard::Prepared,
        0,
    );
    let main: bool = has_main(parent);
    let mut wrapping: bool = false;
    for line in physical_lines(parent.text.as_str()) {
        let written: &str = &parent.text[line.start..line.content_end];
        if !main
            && !wrapping
            && !written.trim_start().starts_with("//!")
            && !written.trim().is_empty()
        {
            // Generated text has no mapped lines, so no diagnostic or count can address it.
            result.text.push_str("fn main() {\n");
            wrapping = true;
        }
        copy_line(
            &mut result,
            parent.text.as_str(),
            line.start,
            line.start,
            line.end,
            String::new(),
        );
    }
    if !main && !wrapping {
        // Even an empty or doc-only fragment receives an entirely unmapped synthetic main.
        if !result.text.is_empty() && !result.text.ends_with(['\r', '\n']) {
            result.text.push('\n');
        }
        result.text.push_str("fn main() {\n");
        wrapping = true;
    }
    if wrapping {
        // A missing authored final newline needs an unmapped separator before the generated brace.
        if !result.text.ends_with(['\r', '\n']) {
            result.text.push('\n');
        }
        result.text.push_str("}\n");
    }
    return result;
}

/// A preparation layer's exact re-extraction behavior,
///  chosen from its immutable parent relationship.
pub(crate) fn prepared_text(parent: &Arc<Mapping>) -> String {
    // A wrapping layer has a hidden-stripping layer as its direct parent.
    if matches!(parent.guard, Guard::Prepared) {
        return wrap(parent).text;
    }
    // Hidden stripping precedes wrapping and never invents code.
    return hidden(parent).text;
}
