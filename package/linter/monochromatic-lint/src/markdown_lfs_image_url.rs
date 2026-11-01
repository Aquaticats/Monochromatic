//! What:
//!  `markdown/lfs-image-url`,
//!  object URLs for Markdown images whose target is LFS-tracked.
//! Why:
//!  GitHub renders a relative link to an LFS-tracked image from its pointer,
//!  as a broken image.
//! The rule rewrites such destinations to `<objectBase>/<oid>/<repo path>`,
//!  refreshes a stale oid,
//! and returns an object URL to a relative link when its target leaves LFS.
//! Edits replace only the written destination;
//!  byte offsets are used directly,
//!  with no
//! UTF-16 or astral-character correction.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (image or image-used definition) classify url, compare with resolved target, report with a localized fix
//! ```

/// Import the shared finding model and a single localized replacement.
/// Import the common node-anchored finding builder and the parsed document.
/// Import the shared ECMAScript trim predicate for whitespace after a destination marker.
use crate::{
    diagnostic::{Diagnostic, Severity},
    edits::Edit,
    markdown_finding::finding,
    markdown_lfs_context::{LfsImageContext, LfsImageTarget},
    markdown_lfs_target::{
        ObjectUrlParts, apply_segments, object_url_parts, relative_link, relative_target_path,
    },
    markdown_source::MarkdownSource,
    markdown_table_text::trim_space,
};
/// Import typed payload decoders;
///  identifiers are already normalized by the parser.
use satteri_ast::mdast::{
    DefinitionData, MdastNodeType, decode_definition_data, decode_image_data, decode_reference_data,
};
/// Import an ordered set of owned identifiers.
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

/// What:
///  This rule's identifier in configuration and JSONL `code`.
/// Why:
///  One spelling is shared by every finding the rule emits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ID = 'markdown/lfs-image-url';
/// ```
const ID: &str = "markdown/lfs-image-url";

/// What:
///  Locate the written destination inside a node's source,
///  as absolute byte offsets.
/// Why:
///  The fix replaces only those bytes.
///  The destination follows `](` in an image (searched from
/// the end,
///  because alt text may contain `](`) or `]:` in a definition,
///  bare or in angle brackets.
/// `None` means the written form differs from the parsed URL (an escaped or entity-encoded
/// destination),
///  and the finding is then reported without a fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function urlSpanOf({ node, source }): { start: number; end: number } | undefined;
/// ```
fn url_span(context: &MarkdownSource, id: u32, image: bool, url: &str) -> Option<(usize, usize)> {
    let (start, _): (usize, usize) = context.offsets(id);
    let written: &str = context.slice(id);
    let marker_at: usize = if image {
        written.rfind("](")?
    } else {
        written.find("]:")?
    };
    let after_marker: &str = &written[marker_at + 2..];
    let padding: usize = after_marker.len() - after_marker.trim_start_matches(trim_space).len();
    let url_start: usize = marker_at + 2 + padding;
    let rest: &str = &written[url_start..];
    if rest.starts_with(url) {
        return Some((start + url_start, start + url_start + url.len()));
    }
    if let Some(inner) = rest.strip_prefix('<')
        && let Some(after_url) = inner.strip_prefix(url)
        && after_url.starts_with('>')
    {
        return Some((start + url_start + 1, start + url_start + 1 + url.len()));
    }
    return None;
}

/// What:
///  Build a finding whose fix replaces the written destination,
///  or a report-only finding when
/// the destination cannot be located.
/// Why:
///  A destination this rule cannot find byte-exactly is left for a person to edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rewriteDiagnostic({ node, source, message, replacement }): Diagnostic;
/// ```
fn rewrite_finding(
    context: &MarkdownSource,
    id: u32,
    image: bool,
    url: &str,
    severity: Severity,
    message: String,
    replacement: String,
) -> Diagnostic {
    let Some((start, end)): Option<(usize, usize)> = url_span(context, id, image, url) else {
        return finding(
            context,
            id,
            ID,
            severity,
            format!("{message} (written URL not located; edit by hand)"),
            None,
        );
    };
    return finding(
        context,
        id,
        ID,
        severity,
        message,
        Some(Edit {
            start,
            end,
            replacement,
        }),
    );
}

/// What:
///  Check a destination already under the object base.
/// Why:
///  The embedded oid must match the file's current oid,
///  the path must still exist,
///  and a
/// target that stopped being LFS-tracked returns to a relative link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkObjectUrl({ node, source, lfs }): Diagnostic[];
/// ```
fn check_object_url(
    context: &MarkdownSource,
    id: u32,
    image: bool,
    url: &str,
    severity: Severity,
    lfs: &LfsImageContext,
) -> Option<Diagnostic> {
    let parts: ObjectUrlParts = object_url_parts(url, lfs.object_base.as_str())?;
    let path: &str = parts.repo_relative_path.as_str();
    match lfs.resolve_target(path) {
        LfsImageTarget::Missing => {
            return Some(finding(
                context,
                id,
                ID,
                severity,
                format!("Object URL names {path}, which no longer exists in the repository."),
                None,
            ));
        }
        LfsImageTarget::Plain => {
            let directory: &Path = lfs.file_path.parent()?;
            let target: PathBuf = apply_segments(&lfs.repo_root, path);
            return Some(rewrite_finding(
                context,
                id,
                image,
                url,
                severity,
                format!(
                    "Object URL names {path}, which is no longer LFS-tracked; link it relatively."
                ),
                relative_link(directory, &target),
            ));
        }
        LfsImageTarget::Lfs { oid } => {
            if oid == parts.oid {
                return None;
            }
            return Some(rewrite_finding(
                context,
                id,
                image,
                url,
                severity,
                format!("Object URL oid is stale for {path}; update it to the file's current oid."),
                format!("{}/{oid}/{path}", lfs.object_base),
            ));
        }
    }
}

/// What:
///  Check a relative destination.
/// Why:
///  An LFS-tracked target gets an object URL;
///  anything else is left to other rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRelativeUrl({ node, source, lfs }): Diagnostic[];
/// ```
fn check_relative_url(
    context: &MarkdownSource,
    id: u32,
    image: bool,
    url: &str,
    severity: Severity,
    lfs: &LfsImageContext,
) -> Option<Diagnostic> {
    let path: String = relative_target_path(url, &lfs.file_path, &lfs.repo_root)?;
    let LfsImageTarget::Lfs { oid } = lfs.resolve_target(path.as_str()) else {
        return None;
    };
    return Some(rewrite_finding(
        context,
        id,
        image,
        url,
        severity,
        String::from(
            "Image targets an LFS-tracked file, which GitHub renders from the pointer as broken; use the object URL.",
        ),
        format!("{}/{oid}/{path}", lfs.object_base),
    ));
}

/// What:
///  Report every image and image-used definition whose destination needs a change.
/// Why:
///  A definition used only by links keeps its target;
///  only definitions some image reference
/// resolves to are rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsImageUrl(context, severity, lfs): Diagnostic[];
/// ```
pub fn lfs_image_url(
    context: &MarkdownSource,
    severity: Severity,
    lfs: &LfsImageContext,
) -> Vec<Diagnostic> {
    let mut image_identifiers: BTreeSet<String> = BTreeSet::<String>::new();
    for id in context.visible_nodes() {
        if context.kind(*id) == MdastNodeType::ImageReference {
            let identifier: &str =
                context.text(decode_reference_data(context.data(*id)).identifier);
            image_identifiers.insert(String::from(identifier));
        }
    }
    let object_prefix: String = format!("{}/", lfs.object_base);
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for id in context.visible_nodes() {
        let kind: MdastNodeType = context.kind(*id);
        let image: bool = kind == MdastNodeType::Image;
        let url: &str = if image {
            context.text(decode_image_data(context.data(*id)).url)
        } else if kind == MdastNodeType::Definition {
            let data: DefinitionData = decode_definition_data(context.data(*id));
            if !image_identifiers.contains(context.text(data.identifier)) {
                continue;
            }
            context.text(data.url)
        } else {
            continue;
        };
        let checked: Option<Diagnostic> = if url.starts_with(object_prefix.as_str()) {
            check_object_url(context, *id, image, url, severity, lfs)
        } else {
            check_relative_url(context, *id, image, url, severity, lfs)
        };
        if let Some(diagnostic) = checked {
            findings.push(diagnostic);
        }
    }
    return findings;
}

/// Ported incumbent rule cases plus BOM and astral-offset controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_image_url_tests.rs"]
mod tests;
