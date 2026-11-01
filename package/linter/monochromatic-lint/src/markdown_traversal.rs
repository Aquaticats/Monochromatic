//! What:
//!  Validate a parsed arena and build the iterative traversal lists every rule walks.
//! Why:
//!  Invalid native ranges or repeated child references must not become unchecked slices or an endless walk,
//!  and one structural walk produces the preorder and parent facts together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Validate reachable tree nodes and produce iterative traversal lists.
//! ```

/// Import the arena container the walker indexes.
use satteri_arena::{Arena, Mdast};
/// Import the node-kind enum the walker classifies.
use satteri_ast::mdast::MdastNodeType;

/// Import the typed failure the walker returns for a malformed arena.
use crate::markdown_source::MarkdownError;

/// Validated traversal facts built together in one structural walk.
pub(crate) struct Traversal {
    /// Every reachable node in preorder.
    pub(crate) all: Vec<u32>,
    /// Prose-rule-visible nodes.
    pub(crate) visible: Vec<u32>,
    /// Reachable parent per node;
    ///  u32::MAX is the root's absence sentinel.
    pub(crate) parents: Vec<u32>,
}

/// What:
///  The existing prose walker excludes MDX code and its descendants.
/// Why:
///  JSX,
///  ESM and expression bodies are not Markdown prose-rule inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isMdx(kind: MdastNodeType): boolean;
/// ```
fn is_mdx(kind: MdastNodeType) -> bool {
    return matches!(
        kind,
        MdastNodeType::MdxJsxFlowElement
            | MdastNodeType::MdxJsxTextElement
            | MdastNodeType::MdxFlowExpression
            | MdastNodeType::MdxTextExpression
            | MdastNodeType::MdxjsEsm
    );
}

/// What:
///  Validate reachable tree nodes and produce iterative traversal lists.
/// Why:
///  Invalid native ranges or repeated child references must not become unchecked slices or an endless walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function traversal(arena: Arena, source: string, bom: number): [number[], number[]];
/// ```
pub(crate) fn traversal(
    arena: &Arena<Mdast>,
    source: &str,
    bom: usize,
) -> Result<Traversal, MarkdownError> {
    if arena.is_empty() {
        return Err(MarkdownError {
            message: String::from("Markdown parser returned no root node."),
            offset: bom,
        });
    }
    let mut visited = vec![false; arena.len()];
    let mut parents = vec![u32::MAX; arena.len()];
    let mut pending = vec![(0_u32, false, u32::MAX)];
    let mut all = Vec::new();
    let mut visible = Vec::new();
    while let Some((id, hidden_parent, parent)) = pending.pop() {
        let index = id as usize;
        if index >= arena.len() || visited[index] {
            return Err(MarkdownError {
                message: String::from(
                    "Markdown parser returned an invalid or repeated child reference.",
                ),
                offset: bom,
            });
        }
        visited[index] = true;
        parents[index] = parent;
        let node = arena.get_node(id);
        let Some(kind) = MdastNodeType::from_u8(node.node_type) else {
            return Err(MarkdownError {
                message: String::from("Markdown parser returned an unknown node kind."),
                offset: bom,
            });
        };
        if id == 0 && kind != MdastNodeType::Root {
            return Err(MarkdownError {
                message: String::from("Markdown parser returned a non-root entry node."),
                offset: bom,
            });
        }
        let start = node.start_offset as usize + bom;
        let end = node.end_offset as usize + bom;
        if start > end
            || end > source.len()
            || !source.is_char_boundary(start)
            || !source.is_char_boundary(end)
        {
            return Err(MarkdownError {
                message: String::from(
                    "Markdown parser returned a source range outside UTF-8 boundaries.",
                ),
                offset: start.min(source.len()),
            });
        }
        let hidden = hidden_parent || is_mdx(kind);
        all.push(id);
        if !hidden {
            visible.push(id);
        }
        let children_start = node.children_start as usize;
        let Some(children_end) = children_start.checked_add(node.children_count as usize) else {
            return Err(MarkdownError {
                message: String::from("Markdown parser returned an overflowing child range."),
                offset: start,
            });
        };
        let Some(children) = arena.children.get(children_start..children_end) else {
            return Err(MarkdownError {
                message: String::from("Markdown parser returned an out-of-range child list."),
                offset: start,
            });
        };
        for child in children.iter().rev() {
            pending.push((*child, hidden, id));
        }
    }
    return Ok(Traversal {
        all,
        visible,
        parents,
    });
}
