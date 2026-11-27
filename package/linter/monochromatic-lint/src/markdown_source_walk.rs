//! What:
//!  The bounded walks one parsed Markdown document answers:
//!  the parent chain above a node,
//!  the subtree below it,
//!  and the text either one holds.
//! Why:
//!  Every ancestor and descendant walk of every rule goes through these count-bounded loops,
//!  so a cycle in the parser's parent or child index is one typed error
//!  instead of a walk that runs until memory ends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parent(id): number | undefined; ancestors(id): number[]; subtree(id): number[];
//! // textContent(id): string; textNodes(id): number[]
//! ```

/// Import the parse these walks read and the failure they report.
use crate::markdown_source::{
    MarkdownError,
    MarkdownSource,
};
/// Import the node kinds a walk compares and the string decoder its text walks read.
use satteri_ast::mdast::{
    MdastNodeType,
    decode_string_ref_data,
};

/// What:
///  Walk one parsed document up to its root and down through its children,
///  without trusting a parent or child index the parse did not validate.
/// Why:
///  A rule asks what encloses a node and what a node holds;
///  both answers come from these loops and their fixed bounds,
///  so a malformed index is a reported processing failure and never an endless walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // The methods of MarkdownSource that walk the tree.
/// ```
impl MarkdownSource {
    /// Read the reachable parent,
    ///  with root absence represented explicitly.
    pub fn parent(
        &self,
        id: u32,
    ) -> Option<u32> {
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
    pub fn ancestors(
        &self,
        id: u32,
    ) -> Result<Vec<u32>, MarkdownError> {
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
    pub fn has_ancestor(
        &self,
        id: u32,
        kind: MdastNodeType,
    ) -> Result<bool, MarkdownError> {
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
    pub fn subtree(
        &self,
        id: u32,
    ) -> Result<Vec<u32>, MarkdownError> {
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
    pub fn text_content(
        &self,
        id: u32,
    ) -> Result<String, MarkdownError> {
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
    pub fn text_nodes(
        &self,
        id: u32,
    ) -> Result<Vec<u32>, MarkdownError> {
        let mut output: Vec<u32> = Vec::new();
        for current in self.subtree(id)? {
            if self.kind(current) == MdastNodeType::Text {
                output.push(current);
            }
        }
        return Ok(output);
    }
}
