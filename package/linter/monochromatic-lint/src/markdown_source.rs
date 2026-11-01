//! What:
//!  The owned native Markdown/MDX parse and byte-safe source interface.
//! Why:
//!  Rules share one arena,
//!  retain original source offsets,
//!  and never execute MDX code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse once and expose node ids, decoded data, source slices and diagnostic positions.
//! ```

/// Import the byte span type that source positions are reported in.
use crate::diagnostic::Span;
/// Import the position map that translates parser offsets back to source offsets.
use crate::markdown_positions::MarkdownPositions;
/// Import the validated structural walk.
use crate::markdown_traversal::traversal;
/// Import the approved parser and arena types.
use satteri_arena::{Arena, ArenaNode, Mdast};
/// Import the node kind enum and string decoder used by the parsed syntax tree.
use satteri_ast::mdast::{MdastNodeType, decode_string_ref_data};
/// Import the CommonMark parser and its options that build the syntax tree.
use satteri_pulldown_cmark::{Options, parse};
/// Import panic containment and the common diagnostic span.
use std::panic::catch_unwind;

/// What:
///  A parser or native-tree failure with an original-source byte position.
/// Why:
///  The engine reports one processing failure instead of attempting fixes on an invalid parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MarkdownError extends Error { readonly offset: number }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownError {
    /// Explanation supplied to the per-file processing finding.
    pub message: String,
    /// Original-source byte location,
    ///  including any leading BOM.
    pub offset: usize,
}

/// Render parser failures through ordinary application error handling.
impl std::fmt::Display for MarkdownError {
    /// Emit only the stored explanation;
    ///  the caller owns filename and position rendering.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// Mark parser failures as standard errors.
impl std::error::Error for MarkdownError {}

/// What:
///  Exact source plus an arena and validated traversal indexes.
/// Why:
///  Arena ids and byte offsets avoid the TypeScript wrapper's extra Unicode-offset conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MarkdownSource { source: string; arena: Arena; visibleNodes: number[] }
/// ```
pub struct MarkdownSource {
    /// Filename or virtual name until processor mapping assigns its host.
    pub filename: String,
    /// Exact original source,
    ///  including a leading BOM if present.
    pub source: String,
    /// Whether MDX syntax was enabled for this input.
    pub mdx: bool,
    /// Parsed tree;
    ///  string and type-data reads stay behind this module's methods.
    arena: Arena<Mdast>,
    /// Source bytes stripped by the parser before producing offsets.
    bom: usize,
    /// Original-source line and UTF-16 position index.
    positions: MarkdownPositions,
    /// Reachable nodes in source-tree order,
    ///  including MDX subtrees.
    all_nodes: Vec<u32>,
    /// Rule-visible nodes with MDX nodes and their subtrees excluded.
    visible_nodes: Vec<u32>,
    /// Parents derived from reachable child edges,
    ///  rather than trusting separate parser metadata.
    parents: Vec<u32>,
}

/// What:
///  Parse with the exact accepted feature set and expose immutable views.
/// Why:
///  Defaults in the dependency enable unwanted math and omit TOML frontmatter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Explicit GFM, footnote, YAML and TOML options; MDX is conditional on file language.
/// ```
impl MarkdownSource {
    /// Parse source without evaluating MDX and retain parser failures as typed outcomes.
    pub fn new(
        filename: String,
        source: String,
        mdx: bool,
    ) -> Result<MarkdownSource, MarkdownError> {
        let bom = if source.starts_with('\u{feff}') { 3 } else { 0 };
        let mut options = Options::ENABLE_GFM
            | Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
        if mdx {
            options.insert(Options::ENABLE_MDX);
        }
        let result = catch_unwind(
            #[cfg_attr(
                dylint_lib = "monochromatic_lint_closure",
                expect(
                    avoidable_closure,
                    reason = "catch_unwind accepts only a callable without parameters, so the borrowed source and the parser options must be captured by a closure"
                )
            )]
            || return parse(source.as_str(), options),
        );
        let (arena, errors) = match result {
            Ok(parsed) => parsed,
            Err(payload) => {
                let detail = if let Some(message) = payload.downcast_ref::<String>() {
                    message.clone()
                } else if let Some(message) = payload.downcast_ref::<&str>() {
                    String::from(*message)
                } else {
                    format!("non-text panic payload {:?}", payload.as_ref().type_id())
                };
                return Err(MarkdownError {
                    message: format!("Markdown parser panicked: {detail}"),
                    offset: bom,
                });
            }
        };
        if let Some((offset, message)) = errors.first() {
            return Err(MarkdownError {
                message: format!("MDX parsing failed: {message}"),
                offset: offset.saturating_add(bom).min(source.len()),
            });
        }
        let traversal = traversal(&arena, source.as_str(), bom)?;
        let positions = MarkdownPositions::new(source.as_str());
        return Ok(MarkdownSource {
            filename,
            source,
            mdx,
            arena,
            bom,
            positions,
            all_nodes: traversal.all,
            visible_nodes: traversal.visible,
            parents: traversal.parents,
        });
    }

    /// Borrow the validated rule-visible traversal.
    pub fn visible_nodes(&self) -> &[u32] {
        return self.visible_nodes.as_slice();
    }

    /// Borrow every reachable node for processor discovery,
    ///  including nodes below JSX.
    pub fn all_nodes(&self) -> &[u32] {
        return self.all_nodes.as_slice();
    }

    /// Read one parser-owned node by a validated id.
    pub fn node(&self, id: u32) -> &ArenaNode {
        return self.arena.get_node(id);
    }

    /// Decode a kind already checked during construction.
    pub fn kind(&self, id: u32) -> MdastNodeType {
        return MdastNodeType::from_u8(self.node(id).node_type)
            .expect("construction validates node kinds");
    }

    /// Borrow direct children without flattening structural relationships.
    pub fn children(&self, id: u32) -> &[u32] {
        return self.arena.get_children(id);
    }

    /// Borrow type-specific data for the node's matching typed decoder.
    pub fn data(&self, id: u32) -> &[u8] {
        return self.arena.get_type_data(id);
    }

    /// Read decoded parser text by its validated arena reference.
    pub fn text(&self, reference: satteri_arena::StringRef) -> &str {
        return self.arena.get_str(reference);
    }

    /// Return the original-source half-open byte range.
    pub fn offsets(&self, id: u32) -> (usize, usize) {
        let node = self.node(id);
        return (
            node.start_offset as usize + self.bom,
            node.end_offset as usize + self.bom,
        );
    }

    /// Borrow exact authored spelling rather than the parser's normalized text.
    pub fn slice(&self, id: u32) -> &str {
        let (start, end) = self.offsets(id);
        return &self.source[start..end];
    }

    /// Resolve original-source byte positions with Markdown's UTF-16 columns.
    pub fn span(&self, offset: usize, length: usize) -> Span {
        return self.positions.span(offset, length);
    }

    /// Return a full-node diagnostic range with original-source addressing.
    pub fn node_span(&self, id: u32) -> Span {
        let (start, end) = self.offsets(id);
        return self.span(start, end - start);
    }

    /// Read the reachable parent,
    ///  with root absence represented explicitly.
    pub fn parent(&self, id: u32) -> Option<u32> {
        let parent = self.parents[id as usize];
        if parent == u32::MAX {
            return None;
        }
        return Some(parent);
    }

    /// What:
    ///  The ancestors of a node,
    ///  nearest first and ending at the root;
    ///  the root itself has none.
    /// Why:
    ///  Every ancestor walk goes through this one bounded loop.
    ///  In a tree each node has at most
    /// one ancestor per other node,
    ///  so a walk that has not reached the root within the document's node
    /// count has found a cycle in the parent index,
    ///  and it reports one typed error instead of looping forever.
    /// `traversal` derives that index from a validated,
    ///  cycle-free child graph,
    ///  so the error marks a
    /// defect in this crate,
    ///  never a property of the Markdown input.
    /// `Result<Vec<u32>, MarkdownError>` is `Ok(list)` on success or `Err(error)` in place of a throw;
    /// `Vec<u32>` is an owned growable list,
    ///  unlike a borrowed `&[u32]` view or a fixed-size `[u32; N]`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ancestors(id: number): number[] // throws MarkdownError when the parent index has a cycle
    /// ```
    pub fn ancestors(&self, id: u32) -> Result<Vec<u32>, MarkdownError> {
        // Owned list the caller receives; it grows by one id per step up the tree.
        let mut chain: Vec<u32> = Vec::new();
        // `Option<u32>` is `Some(id)` for a parent or `None` above the root, like `number | undefined`.
        let mut current: Option<u32> = self.parent(id);
        // A fixed range bounds the walk whatever the body does: a node at depth d needs d + 1 passes,
        // and d is at most one less than the node count, so a tree always finishes inside this range.
        for _ in 0..self.parents.len() {
            // `let Some(parent) = current else { ... }` unwraps the parent or, above the root, returns the chain.
            let Some(parent) = current else {
                // `Ok(chain)` is the success variant; the root was reached.
                return Ok(chain);
            };
            chain.push(parent);
            current = self.parent(parent);
        }
        // `Err(...)` is the failure variant; the offset places the processing failure at the starting node.
        return Err(MarkdownError {
            message: format!(
                "Markdown node {id} has more ancestors than the {} nodes of its document, so the parser's parent index has a cycle and the document's structure cannot be trusted. This is a defect in the linter, not in the file: report it with this file.",
                self.parents.len()
            ),
            offset: self.offsets(id).0,
        });
    }

    /// What:
    ///  Whether any ancestor of `id` has the given kind;
    ///  `Err` reports a parent-index cycle.
    /// Why:
    ///  Ancestry is answered from the bounded walk,
    ///  never from parser-internal parent bookkeeping.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hasAncestor(id: number, kind: MdastNodeType): boolean // throws MarkdownError on a cycle
    /// ```
    pub fn has_ancestor(&self, id: u32, kind: MdastNodeType) -> Result<bool, MarkdownError> {
        // The `?` returns the walk's error to the caller at once, like rethrowing; otherwise it unwraps the list.
        for parent in self.ancestors(id)? {
            if self.kind(parent) == kind {
                return Ok(true);
            }
        }
        return Ok(false);
    }

    /// What:
    ///  A node followed by all its descendants,
    ///  in source order.
    /// Why:
    ///  Every descendant walk goes through this one bounded loop,
    ///  the downward twin of `ancestors`.
    /// A tree holds each node once,
    ///  so a walk that has visited more nodes than the document has
    /// has found a cycle in the child index,
    ///  and it reports one typed error instead of
    /// looping until memory runs out.
    ///  `traversal` rejects such an index when the document is built,
    /// so the error marks a defect in this crate,
    ///  never a property of the Markdown input.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// subtree(id: number): number[] // throws MarkdownError when the child index has a cycle
    /// ```
    pub fn subtree(&self, id: u32) -> Result<Vec<u32>, MarkdownError> {
        // Owned list the caller receives; it grows by one id per visited node.
        let mut visited: Vec<u32> = Vec::new();
        // A work stack of nodes still to visit, used in place of recursion.
        let mut pending: Vec<u32> = vec![id];
        // A fixed range bounds the walk whatever the body does. Each pass visits one node, and `0..=len`
        // is inclusive: one pass per node of the document, plus the pass that finds nothing waiting.
        for _ in 0..=self.parents.len() {
            // `pending.pop()` is `Some(id)` while nodes wait and `None` once the whole subtree was visited.
            let Some(current) = pending.pop() else {
                return Ok(visited);
            };
            visited.push(current);
            // Push children last to first, so the first child is visited next.
            for child in self.children(current).iter().rev() {
                pending.push(*child);
            }
        }
        return Err(MarkdownError {
            message: format!(
                "Markdown node {id} has more descendants than the {} nodes of its document, so the parser's child index has a cycle and the document's structure cannot be trusted. This is a defect in the linter, not in the file: report it with this file.",
                self.parents.len()
            ),
            offset: self.offsets(id).0,
        });
    }

    /// What:
    ///  The text and inline-code values below a node,
    ///  joined in source order;
    ///  `Err` reports a child-index cycle.
    /// Why:
    ///  This matches the incumbent's collectText helper.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// textContent(id: number): string // throws MarkdownError on a cycle
    /// ```
    pub fn text_content(&self, id: u32) -> Result<String, MarkdownError> {
        let mut output: String = String::new();
        for current in self.subtree(id)? {
            let kind: MdastNodeType = self.kind(current);
            if kind == MdastNodeType::Text || kind == MdastNodeType::InlineCode {
                let reference = decode_string_ref_data(self.data(current));
                output.push_str(self.text(reference));
            }
        }
        return Ok(output);
    }

    /// What:
    ///  The text nodes below a node,
    ///  in source order,
    ///  for localized text edits;
    ///  `Err` reports a child-index cycle.
    /// Why:
    ///  A rule edits the last text node of a heading and needs its id,
    ///  not only its text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// textNodes(id: number): number[] // throws MarkdownError on a cycle
    /// ```
    pub fn text_nodes(&self, id: u32) -> Result<Vec<u32>, MarkdownError> {
        let mut output: Vec<u32> = Vec::new();
        for current in self.subtree(id)? {
            if self.kind(current) == MdastNodeType::Text {
                output.push(current);
            }
        }
        return Ok(output);
    }
}

/// Direct native-arena guard controls,
///  independent of what malformed text the parser happens to emit.
#[cfg(test)]
#[path = "markdown_traversal_tests.rs"]
mod traversal_tests;

/// Keep native-parser controls outside release artifacts.
#[cfg(test)]
#[path = "markdown_source_tests.rs"]
mod tests;
