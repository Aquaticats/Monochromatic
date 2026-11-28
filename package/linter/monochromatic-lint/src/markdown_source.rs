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
use satteri_arena::{
    Arena,
    ArenaNode,
    Mdast,
};
/// Import the node kind enum used by the parsed syntax tree.
use satteri_ast::mdast::MdastNodeType;
/// Import the CommonMark parser and its options that build the syntax tree.
use satteri_pulldown_cmark::{
    Options,
    parse,
};
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
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
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
    /// What:
    ///  The characters the host file writes before each line of this source,
    ///  when this source is a projection of that host.
    /// Why:
    ///  A rule that bounds the length of a line it writes counts the line the reader meets,
    ///  which is the host's: a Rust doc comment's marker and indent stand before the text,
    ///  and a physical Markdown file writes nothing before it.
    ///  The bound is the widest prefix of the projection, so no written line passes the limit at any depth.
    pub host_prefix_chars: usize,
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
    ///  Its walks read the field directly from their own module.
    pub(crate) parents: Vec<u32>,
}

/// What:
///  Parse with the exact accepted feature set and expose immutable views of the result.
/// Why:
///  Defaults in the dependency enable unwanted math and omit TOML frontmatter;
///  the rules read every node through these accessors,
///  and the walks that need more than one node are in `markdown_source_walk`.
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
            host_prefix_chars: 0,
            arena,
            bom,
            positions,
            all_nodes: traversal.all,
            visible_nodes: traversal.visible,
            parents: traversal.parents,
        });
    }

    /// What:
    ///  Record how many characters the host writes before a line of this projection.
    /// Why:
    ///  Only the caller that built the projection knows its host's prefix,
    ///  and a rule bounding a written line must count that prefix.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// source.withHostPrefixChars(chars);
    /// ```
    pub fn with_host_prefix_chars(
        mut self,
        host_prefix_chars: usize,
    ) -> MarkdownSource {
        self.host_prefix_chars = host_prefix_chars;
        return self;
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
    pub fn node(
        &self,
        id: u32,
    ) -> &ArenaNode {
        return self.arena.get_node(id);
    }

    /// Decode a kind already checked during construction.
    pub fn kind(
        &self,
        id: u32,
    ) -> MdastNodeType {
        return MdastNodeType::from_u8(self.node(id).node_type)
            .expect("construction validates node kinds");
    }

    /// Borrow direct children without flattening structural relationships.
    pub fn children(
        &self,
        id: u32,
    ) -> &[u32] {
        return self.arena.get_children(id);
    }

    /// Borrow type-specific data for the node's matching typed decoder.
    pub fn data(
        &self,
        id: u32,
    ) -> &[u8] {
        return self.arena.get_type_data(id);
    }

    /// Read decoded parser text by its validated arena reference.
    pub fn text(
        &self,
        reference: satteri_arena::StringRef,
    ) -> &str {
        return self.arena.get_str(reference);
    }

    /// Return the original-source half-open byte range.
    pub fn offsets(
        &self,
        id: u32,
    ) -> (usize, usize) {
        let node = self.node(id);
        return (
            node.start_offset as usize + self.bom,
            node.end_offset as usize + self.bom,
        );
    }

    /// Borrow exact authored spelling rather than the parser's normalized text.
    pub fn slice(
        &self,
        id: u32,
    ) -> &str {
        let (start, end) = self.offsets(id);
        return &self.source[start..end];
    }

    /// Resolve original-source byte positions with Markdown's UTF-16 columns.
    pub fn span(
        &self,
        offset: usize,
        length: usize,
    ) -> Span {
        return self.positions.span(offset, length);
    }

    /// Return a full-node diagnostic range with original-source addressing.
    pub fn node_span(
        &self,
        id: u32,
    ) -> Span {
        let (start, end) = self.offsets(id);
        return self.span(start, end - start);
    }
}

/// What:
///  Every paragraph's container kinds and its words,
///  which together say where each piece of text sits.
/// Why:
///  A whitespace-only fix may move line breaks but not text:
///  the property check over a real tree compares two parses and needs one definition
///  of what a document's block structure and content are,
///  not a copy per test module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function paragraphPlacements(context: MarkdownSource): [MdastNodeType[], string[]][];
/// ```
#[cfg(test)]
pub(crate) fn paragraph_placements(
    context: &MarkdownSource
) -> Result<Vec<(Vec<MdastNodeType>, Vec<String>)>, MarkdownError> {
    let mut found: Vec<(Vec<MdastNodeType>, Vec<String>)> = Vec::new();
    for id in context.all_nodes() {
        if context.kind(*id) != MdastNodeType::Paragraph {
            continue;
        }
        let mut containers: Vec<MdastNodeType> = Vec::<MdastNodeType>::new();
        for ancestor in context.ancestors(*id)? {
            containers.push(context.kind(ancestor));
        }
        let words: Vec<String> = context
            .text_content(*id)?
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<String>>();
        found.push((containers, words));
    }
    return Ok(found);
}

/// What:
///  Every block node in preorder with its depth,
///  which records where block boundaries fall and which container holds them.
/// Why:
///  Paragraph ancestors alone cannot tell two adjacent quotations from one quotation with two paragraphs;
///  a whitespace-only fix must not change either boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blockContainers(context: MarkdownSource): [number, MdastNodeType][];
/// ```
#[cfg(test)]
pub(crate) fn block_containers(
    context: &MarkdownSource
) -> Result<Vec<(usize, MdastNodeType)>, MarkdownError> {
    let mut found: Vec<(usize, MdastNodeType)> = Vec::new();
    for id in context.all_nodes() {
        let kind: MdastNodeType = context.kind(*id);
        if is_inline(kind) {
            continue;
        }
        found.push((context.ancestors(*id)?.len(), kind));
    }
    return Ok(found);
}

/// Whether a node kind is inline content rather than a block boundary.
#[cfg(test)]
fn is_inline(kind: MdastNodeType) -> bool {
    return matches!(
        kind,
        MdastNodeType::Text
            | MdastNodeType::Emphasis
            | MdastNodeType::Strong
            | MdastNodeType::InlineCode
            | MdastNodeType::Break
            | MdastNodeType::Link
            | MdastNodeType::Image
            | MdastNodeType::LinkReference
            | MdastNodeType::ImageReference
            | MdastNodeType::FootnoteReference
            | MdastNodeType::Delete
            | MdastNodeType::InlineMath
            | MdastNodeType::LeafDirective
            | MdastNodeType::TextDirective
            | MdastNodeType::Superscript
            | MdastNodeType::Subscript
            | MdastNodeType::MdxJsxTextElement
            | MdastNodeType::MdxTextExpression
            | MdastNodeType::Custom
    );
}

/// What:
///  The exact source of every code and raw HTML block,
/// in document order.
/// Why:
///  The token comparison of a whole file sets line-leading markers aside everywhere,
///  including inside a fence,
///  so quoted source needs its own byte-for-byte comparison.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verbatimBlocks(context: MarkdownSource): string[];
/// ```
#[cfg(test)]
pub(crate) fn verbatim_blocks(context: &MarkdownSource) -> Result<Vec<String>, MarkdownError> {
    let mut found: Vec<String> = Vec::new();
    for id in context.all_nodes() {
        let kind: MdastNodeType = context.kind(*id);
        if kind != MdastNodeType::Code && kind != MdastNodeType::Html {
            continue;
        }
        found.push(String::from(context.slice(*id)));
    }
    return Ok(found);
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
