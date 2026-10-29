//! What:
//!  Grouped source-fix projection with container re-extraction proofs.
//! Why:
//!  A syntactically encodable prefix is insufficient if the edit closes a fence or Rustdoc block.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Encode edited newlines, project whole groups, re-extract each container, compare intended bytes.
//! ```

/// Import existing grouped edits,
///  exact lines and immutable mapping facts.
use crate::edits::{Edit, Fix};
/// Preserve each affected physical line's original newline spelling.
use crate::processors_lines::{newline, physical_lines};
/// Identify synthetic text and the extraction operation to recheck.
use crate::processors_model::{Guard, MappedLine, Mapping, ProcessorError};
/// Validate atomic edits without rejecting an empty virtual snippet.
use crate::processors_rewrite::rewrite;
/// Share original snapshots while checking a disposable changed parent.
use std::sync::Arc;

/// Locate an authored edit endpoint,
///  preferring the next line at a shared boundary.
fn endpoint(mapping: &Mapping, offset: usize) -> Result<&MappedLine, ProcessorError> {
    for line in &mapping.lines {
        if line.start <= offset && offset < line.end {
            return Ok(line);
        }
    }
    // A let-chain accepts the optional last authored line only at its exact endpoint.
    if let Some(last) = mapping.lines.last()
        && last.end == offset
    {
        return Ok(last);
    }
    return Err(mapping.error("Processor fix touches synthetic text or an empty container with no proven insertion mapping."));
}

/// Restore a replacement's newline and direct-parent prefix at its final interpolation.
fn encoded(replacement: &str, ending: &str, prefix: &str) -> String {
    let mut result: String = String::new();
    for line in physical_lines(replacement) {
        result.push_str(&replacement[line.start..line.content_end]);
        if line.content_end != line.end {
            result.push_str(ending);
            result.push_str(prefix);
        }
    }
    return result;
}

/// Project one validated edit,
///  preserving whole-line deletion and detecting synthetic gaps.
fn edit_to_parent(
    mapping: &Mapping,
    parent: &Mapping,
    edit: &Edit,
) -> Result<(Edit, Edit), ProcessorError> {
    let first: &MappedLine = endpoint(mapping, edit.start)?;
    let last: &MappedLine = endpoint(mapping, edit.end)?;
    let mut coverage: usize = edit.start;
    for line in &mapping.lines {
        if line.end <= edit.start || line.start >= edit.end {
            continue;
        }
        if line.start > coverage {
            return Err(mapping.error(
                "Processor fix crosses synthetic main text; the atomic group cannot be projected.",
            ));
        }
        coverage = line.end.min(edit.end);
    }
    if coverage < edit.end {
        return Err(mapping.error(
            "Processor fix includes synthetic main text; the atomic group cannot be projected.",
        ));
    }
    let ending: &str = newline(parent.text.as_str(), first.parent_start);
    let normalized: String = encoded(edit.replacement.as_str(), ending, "");
    let mut replacement: String = encoded(edit.replacement.as_str(), ending, first.prefix.as_str());
    // Successful endpoint lookup proves at least one authored line exists.
    let final_line: &MappedLine = mapping.lines.last().expect("authored endpoint exists");
    if normalized.ends_with(['\r', '\n']) && final_line.end == edit.end {
        // No next payload remains at virtual EOF; do not create an orphan container prefix.
        replacement.truncate(replacement.len() - first.prefix.len());
    }
    let whole_lines: bool = edit.replacement.is_empty()
        && edit.start == first.start
        && (edit.end == last.start || edit.end == last.end);
    let start: usize = if whole_lines {
        first.envelope
    } else {
        first.parent_start + edit.start - first.start
    };
    let end: usize = if whole_lines && edit.end == last.start {
        last.envelope
    } else {
        last.parent_start + edit.end - last.start
    };
    return Ok((
        Edit {
            start,
            end,
            replacement,
        },
        Edit {
            start: edit.start,
            end: edit.end,
            replacement: normalized,
        },
    ));
}

/// Prove the same native container yields exactly the intended virtual rewrite.
fn verify(mapping: &Mapping, parent: &Arc<Mapping>, expected: &str) -> Result<(), ProcessorError> {
    let extracted: Result<Option<String>, ProcessorError> = match mapping.guard {
        Guard::Fence { marker } => crate::processors_fences::fence_text(parent, marker),
        Guard::Docs { anchor } => crate::processors_docs::doc_text(parent, anchor),
        Guard::Prepared => Ok(Some(crate::processors_prepare::prepared_text(parent))),
        Guard::Step => {
            return Err(mapping
                .error("A workflow step unexpectedly requested a fix; its rules carry none."));
        }
        Guard::Root => {
            return Err(
                mapping.error("A physical host unexpectedly requested container verification.")
            );
        }
    };
    // A rejected candidate's byte addresses belong to proposed bytes, never the original host.
    // Rebase that failure to this immutable authored container while retaining its native explanation.
    let actual: Option<String> = match extracted {
        Ok(text) => text,
        Err(error) => {
            return Err(mapping.error(
                format!("Cannot verify the projected container: {}", error.message).as_str(),
            ));
        }
    };
    if actual.as_deref() == Some(expected) || actual.is_none() && expected.is_empty() {
        return Ok(());
    }
    return Err(mapping.error("Projected fix changes its Markdown/Rust container or hidden-line preparation. No edits in this atomic group can be applied safely."));
}

/// Project one entire fix to the original host,
///  validating every intermediate container.
pub(crate) fn project(mapping: &Mapping, fix: &Fix) -> Result<Fix, ProcessorError> {
    rewrite(mapping, fix)?;
    let Some(parent) = &mapping.parent else {
        return Ok(fix.clone());
    };
    let mut projected: Vec<Edit> = Vec::new();
    let mut normalized: Vec<Edit> = Vec::new();
    for edit in &fix.edits {
        // No-op replacements do not expand or normalize untouched authored bytes.
        if mapping.text[edit.start..edit.end] == edit.replacement {
            continue;
        }
        let (host_edit, virtual_edit): (Edit, Edit) = edit_to_parent(mapping, parent, edit)?;
        projected.push(host_edit);
        normalized.push(virtual_edit);
    }
    let group: Fix = Fix { edits: projected };
    let expected: String = rewrite(mapping, &Fix { edits: normalized })?;
    let changed: String = rewrite(parent, &group)?;
    // Clone map facts only for re-extraction; further projection uses original parent offsets.
    let mut candidate: Mapping = parent.as_ref().clone();
    candidate.text = changed;
    verify(mapping, &Arc::new(candidate), expected.as_str())?;
    return project(parent, &group);
}
