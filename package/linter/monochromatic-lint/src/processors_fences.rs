//! What:
//!  Native Markdown/MDX Rust-fence discovery with exact physical payloads.
//! Why:
//!  Markdown determines containers;
//!  processors never search prose for pretend fences.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse nodes, pair normalized code lines with authored suffixes, retain prefixes.
//! ```

/// Import the shared Markdown parse,
///  typed native code decoder and mapping helpers.
use crate::markdown_code::fence_marker_end;
/// Borrow the same native arena interface as Markdown rules.
use crate::markdown_source::MarkdownSource;
/// Retain original newline spelling when copying decoded lines.
use crate::processors_lines::{copy_line, physical_lines};
/// Build immutable child snapshots with explicit extraction refusals.
use crate::processors_model::{Guard, Mapping, ProcessorError, ProcessorLanguage};
/// Decode only parser-confirmed code nodes.
use satteri_ast::mdast::{MdastNodeType, decode_code_data};
/// Share exact parent snapshots between sibling fences.
use std::sync::Arc;

/// Extract selected Rust fences;
///  unlabeled fences become Rust only inside authored Rustdoc.
pub(crate) fn fences(parent: &Arc<Mapping>, rustdoc: bool) -> Result<Vec<Mapping>, ProcessorError> {
    // Owned parse inputs isolate the parser from the immutable mapping snapshot.
    let parsed: MarkdownSource = match MarkdownSource::new(
        parent.filename.clone(),
        parent.text.clone(),
        parent.language == ProcessorLanguage::Mdx,
    ) {
        // Result extraction keeps the parser's affected byte rather than replacing it with a host sentinel.
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(parent.error_at(
                error.offset,
                format!("Markdown extraction failed: {}", error.message).as_str(),
            ));
        }
    };
    // Vec owns a runtime-sized result catalog, not references into the arena.
    let mut result: Vec<Mapping> = Vec::new();
    // Native nodes below MDX containers still participate in processor discovery.
    for id in parsed.all_nodes() {
        if parsed.kind(*id) != MdastNodeType::Code {
            continue;
        }
        // Native range starts at the marker, excluding the surrounding Markdown container.
        let Some(marker_end) = fence_marker_end(&parsed, *id) else {
            continue;
        };
        let data = decode_code_data(parsed.data(*id));
        // Borrow the decoded info string, not its escaped source spelling.
        let language: &str = parsed.text(data.lang);
        if !language.starts_with("rust")
            && !language.starts_with("rs")
            && !(rustdoc && language.is_empty())
        {
            continue;
        }
        // Borrow original node coordinates and decoded payload for exact pairing.
        let (marker, node_end): (usize, usize) = parsed.offsets(*id);
        let virtual_name: String = format!("{}/{}.rs", parent.filename, result.len());
        let mut child: Mapping = crate::processors::child(
            parent,
            virtual_name,
            ProcessorLanguage::Rust,
            Guard::Fence { marker },
            marker,
        );
        let written: &str = &parent.text[marker..node_end];
        let lines = physical_lines(written);
        let decoded: &str = parsed.text(data.value);
        let decoded_lines = physical_lines(decoded);
        // The opening fence line is authored but never decoded, so authored lines must outnumber decoded ones.
        if decoded_lines.len() >= lines.len() {
            return Err(parent.error("Native fence payload has no exact physical-line mapping."));
        }
        for (index, expected) in decoded_lines.iter().enumerate() {
            let authored = &lines[index + 1];
            let body_start: usize = marker + authored.start;
            let content_end: usize = marker + authored.content_end;
            let expected_text: &str = &decoded[expected.start..expected.content_end];
            let authored_text: &str = &parent.text[body_start..content_end];
            if !authored_text.ends_with(expected_text) {
                return Err(parent.error_at(body_start, "Native fence normalization cannot be mapped to exact authored bytes (for example a partially expanded tab)."));
            }
            let payload: usize = content_end - expected_text.len();
            // Preserve every removed blockquote/list/indentation byte for future inserted lines.
            let prefix: String = String::from(&parent.text[body_start..payload]);
            copy_line(
                &mut child,
                parent.text.as_str(),
                body_start,
                payload,
                marker + authored.end,
                prefix,
            );
        }
        // Empty fences retain a reporting anchor but refuse unproven inserted-body mappings.
        child.anchor = marker_end;
        result.push(child);
    }
    return Ok(result);
}

/// Re-extract the same fenced container after a projected grouped edit.
pub(crate) fn fence_text(
    parent: &Arc<Mapping>,
    marker: usize,
) -> Result<Option<String>, ProcessorError> {
    // All explicit Rust fences are selected irrespective of their rustdoc attributes.
    for mapping in fences(parent, true)? {
        // Require both the fence variant and its original marker in one let-chain.
        if let Guard::Fence { marker: candidate } = mapping.guard
            && candidate == marker
        {
            return Ok(Some(mapping.text));
        }
    }
    return Ok(None);
}
