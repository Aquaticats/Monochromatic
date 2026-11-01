//! What:
//!  Deliberately damaged native arenas exercising the source adapter's validation guards.
//! Why:
//!  Parser-generated happy paths cannot prove rejection of corrupt child graphs and byte ranges.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Prove a valid arena is accepted, mutate one invariant, then require the exact processing failure.
//! ```

/// Import the real guard boundary and installed arena construction API.
use super::{MarkdownError, MarkdownSource, traversal};
/// Import the finding model and every rule that walks ancestors or descendants,
///  to observe how a walk failure is reported.
use crate::diagnostic::{Diagnostic, Severity};
use crate::markdown_duplicate_headings::no_duplicate_heading;
use crate::markdown_headings::no_emphasis_as_heading;
use crate::markdown_punctuation::no_trailing_punctuation;
use crate::markdown_semantic_breaks::semantic_line_breaks;
use satteri_arena::{Arena, Mdast};
use satteri_ast::mdast::MdastNodeType;

/// Construct an actual arena with a root and one reachable child.
fn valid_arena() -> Arena<Mdast> {
    // Owned UTF-8 text contains one astral character so midpoint byte offsets are distinguishable.
    let mut arena: Arena<Mdast> = Arena::<Mdast>::new(String::from("🚀x"));
    let root: u32 = arena.alloc_node(MdastNodeType::Root as u8);
    let child: u32 = arena.alloc_node(MdastNodeType::Paragraph as u8);
    arena.set_position(root, 0, 5, 1, 1, 1, 4);
    arena.set_position(child, 0, 5, 1, 1, 1, 4);
    arena.set_children(root, &[child]);
    // The positive control checks the same path and exact bytes later corruption tests mutate.
    assert!(traversal(&arena, "🚀x", 0).is_ok());
    return arena;
}

/// Extract the operational failure without requiring Debug on the successful traversal indexes.
fn rejected(arena: &Arena<Mdast>) -> MarkdownError {
    match traversal(arena, "🚀x", 0) {
        Ok(_) => panic!("corrupted native arena must be rejected"),
        Err(error) => return error,
    }
}

/// Repeated valid ids and individually invalid ids are independent rejection paths.
#[test]
fn child_graph_rejects_duplicate_invalid_and_cyclic_ids() {
    let mut duplicate: Arena<Mdast> = valid_arena();
    duplicate.set_children(0, &[1, 1]);
    assert!(
        rejected(&duplicate)
            .message
            .contains("invalid or repeated child")
    );
    let mut invalid: Arena<Mdast> = valid_arena();
    invalid.children[0] = 99;
    assert!(
        rejected(&invalid)
            .message
            .contains("invalid or repeated child")
    );
    let mut cyclic: Arena<Mdast> = valid_arena();
    cyclic.children[0] = 0;
    assert!(
        rejected(&cyclic)
            .message
            .contains("invalid or repeated child")
    );
}

/// Every source-range predicate fails independently,
///  not only when several defects happen together.
#[test]
fn source_ranges_reject_reversal_out_of_bounds_and_each_utf8_midpoint() {
    for (start, end) in [(5, 4), (0, 6), (1, 4), (0, 1)] {
        let mut arena: Arena<Mdast> = valid_arena();
        arena.nodes[1].start_offset = start;
        arena.nodes[1].end_offset = end;
        let error: MarkdownError = rejected(&arena);
        assert!(error.message.contains("outside UTF-8 boundaries"));
        assert!(error.offset <= 5);
    }
}

/// Missing roots,
///  unknown kinds and a valid-but-wrong entry kind remain distinct diagnostics.
#[test]
fn root_and_kind_validation_does_not_assume_a_valid_parser_result() {
    let empty: Arena<Mdast> = Arena::<Mdast>::new(String::from("🚀x"));
    assert!(rejected(&empty).message.contains("no root node"));
    let mut unknown: Arena<Mdast> = valid_arena();
    unknown.nodes[1].node_type = u8::MAX;
    assert!(rejected(&unknown).message.contains("unknown node kind"));
    let mut wrong_root: Arena<Mdast> = valid_arena();
    wrong_root.nodes[0].node_type = MdastNodeType::Paragraph as u8;
    assert!(
        rejected(&wrong_root)
            .message
            .contains("non-root entry node")
    );
}

/// An invalid flat child-list range is rejected before the arena's unchecked accessor is called.
#[test]
fn child_list_bounds_are_checked_before_borrowing() {
    let mut arena: Arena<Mdast> = valid_arena();
    arena.nodes[0].children_start = 99;
    assert!(rejected(&arena).message.contains("out-of-range child list"));
}

/// Parse one fixture through the real adapter;
///  these fixtures always parse.
fn parsed(source: &str) -> MarkdownSource {
    // `.expect` unwraps `Ok(document)` or fails the test with this message.
    return MarkdownSource::new(String::from("cycle.md"), String::from(source), false)
        .expect("fixture parses");
}

/// Find the only node of one kind,
///  so corruption below targets real parser output rather than guessed ids.
fn only(document: &MarkdownSource, kind: MdastNodeType) -> u32 {
    // `Option<u32>` holds the match once found; a second match means the fixture is not what the test assumes.
    let mut found: Option<u32> = None;
    for id in document.all_nodes() {
        if document.kind(*id) == kind {
            assert_eq!(found, None, "fixture has one node of this kind");
            found = Some(*id);
        }
    }
    return found.expect("fixture has this node kind");
}

/// The bounded walk admits the deepest chain a document can hold:
///  every node on one path below the root.
#[test]
fn ancestor_walks_reach_the_root_from_the_deepest_possible_node() {
    let document: MarkdownSource = parsed("*__a__*\n");
    let text: u32 = only(&document, MdastNodeType::Text);
    let chain: Vec<u32> = document.ancestors(text).expect("an acyclic parse");
    assert_eq!(
        chain,
        [
            only(&document, MdastNodeType::Strong),
            only(&document, MdastNodeType::Emphasis),
            only(&document, MdastNodeType::Paragraph),
            0,
        ]
    );
    // Every node of the arena lies on this path, so the walk needs exactly as many passes as there are nodes.
    assert_eq!(document.parents.len(), chain.len() + 1);
    assert_eq!(document.ancestors(0), Ok(Vec::<u32>::new()));
}

/// `traversal` rejects cyclic child graphs before any parent index exists
/// (`child_graph_rejects_duplicate_invalid_and_cyclic_ids`),
///  so a parent cycle can only appear in the derived
/// index itself:
///  through a later change to `traversal`,
///  or a mutated `parent` accessor.
/// This plants one there,
///  then requires the typed error and one processing failure from each ancestry rule.
#[test]
fn a_parent_index_cycle_is_a_typed_error_and_a_processing_failure() {
    let mut document: MarkdownSource = parsed("*a*\n");
    let paragraph: u32 = only(&document, MdastNodeType::Paragraph);
    let emphasis: u32 = only(&document, MdastNodeType::Emphasis);
    let text: u32 = only(&document, MdastNodeType::Text);
    // Positive control: the intact index climbs text, emphasis, paragraph, root.
    assert_eq!(document.ancestors(text), Ok(vec![emphasis, paragraph, 0]));
    assert_eq!(
        document.has_ancestor(text, MdastNodeType::Paragraph),
        Ok(true)
    );
    // Corrupt the derived index: the paragraph's parent becomes the emphasis inside it.
    // `as usize` widens the u32 id to the index type a Vec takes.
    document.parents[paragraph as usize] = emphasis;
    let nodes: usize = document.parents.len();
    let cycle: String = format!(
        "has more ancestors than the {nodes} nodes of its document, so the parser's parent index has a cycle and the document's structure cannot be trusted. This is a defect in the linter, not in the file: report it with this file."
    );
    // The walk from the text node fails at the text node's own offset, after the `*`.
    let error: MarkdownError = document.ancestors(text).expect_err("a parent cycle");
    assert_eq!(error.message, format!("Markdown node {text} {cycle}"));
    assert_eq!(error.offset, 1);
    assert_eq!(
        document.has_ancestor(text, MdastNodeType::ListItem),
        Err(error.clone())
    );
    // Each rule reports exactly one processing failure that names it, at the node whose walk failed.
    for (findings, rule, node, offset) in [
        (
            semantic_line_breaks(&document, Severity::Warn),
            "markdown/semantic-line-breaks",
            text,
            1,
        ),
        (
            no_emphasis_as_heading(&document, Severity::Warn),
            "markdown/no-emphasis-as-heading",
            paragraph,
            0,
        ),
    ] {
        assert_eq!(findings.len(), 1, "{rule}");
        let failure: &Diagnostic = &findings[0];
        assert_eq!(failure.code, "core/processing-failure", "{rule}");
        assert!(failure.processing_failure, "{rule}");
        assert_eq!(failure.severity, Severity::Error, "{rule}");
        assert_eq!(failure.fix, None, "{rule}");
        assert_eq!(
            failure.message,
            format!("{rule} could not check this file: Markdown node {node} {cycle}")
        );
        assert_eq!(failure.labels[0].span.offset, offset, "{rule}");
        assert_eq!(failure.filename, "cycle.md", "{rule}");
    }
}

/// Point a node's first child slot at another node,
///  the way a corrupt child index would.
fn replace_first_child(document: &mut MarkdownSource, node: u32, child: u32) {
    // `as usize` widens ids and offsets to the index type a Vec takes.
    let slot: usize = document.arena.nodes[node as usize].children_start as usize;
    document.arena.children[slot] = child;
}

/// The bounded descendant walk admits the largest subtree a document can hold:
///  the whole document.
#[test]
fn descendant_walks_cover_the_whole_document_from_the_root() {
    let document: MarkdownSource = parsed("# T\n\n*a*\n");
    // The root's subtree is every node, in the same source order the validated traversal recorded.
    assert_eq!(document.subtree(0), Ok(document.all_nodes().to_vec()));
    assert_eq!(document.all_nodes().len(), document.parents.len());
    let emphasis: u32 = only(&document, MdastNodeType::Emphasis);
    let texts: Vec<u32> = document.text_nodes(0).expect("an acyclic parse");
    assert_eq!(texts.len(), 2);
    assert_eq!(document.text_content(0), Ok(String::from("Ta")));
    // A subtree starts with its own node and holds only what is below it.
    assert_eq!(document.subtree(emphasis), Ok(vec![emphasis, texts[1]]));
    assert_eq!(document.text_nodes(emphasis), Ok(vec![texts[1]]));
    assert_eq!(document.text_content(emphasis), Ok(String::from("a")));
}

/// `traversal` rejects a cyclic child graph when a document is built
/// (`child_graph_rejects_duplicate_invalid_and_cyclic_ids`),
///  so a child cycle can only appear afterwards:
/// through a later change to `traversal`,
///  or a mutated `children` accessor.
/// This plants two in a built document,
///  then requires the typed error and one processing failure
/// from each rule that reads text below a node.
#[test]
fn a_child_index_cycle_is_a_typed_error_and_a_processing_failure() {
    let mut document: MarkdownSource = parsed("# T\n\n*a*\n");
    let heading: u32 = only(&document, MdastNodeType::Heading);
    let emphasis: u32 = only(&document, MdastNodeType::Emphasis);
    // Positive control: the intact index yields each node's own text.
    assert_eq!(document.text_content(heading), Ok(String::from("T")));
    assert_eq!(document.text_content(emphasis), Ok(String::from("a")));
    // Corrupt the child index: the heading and the emphasis each become their own first child.
    replace_first_child(&mut document, heading, heading);
    replace_first_child(&mut document, emphasis, emphasis);
    let nodes: usize = document.parents.len();
    let cycle: String = format!(
        "has more descendants than the {nodes} nodes of its document, so the parser's child index has a cycle and the document's structure cannot be trusted. This is a defect in the linter, not in the file: report it with this file."
    );
    // Each walk fails at its own starting node: the heading at byte 0, the emphasis at byte 5.
    let error: MarkdownError = document.subtree(heading).expect_err("a child cycle");
    assert_eq!(error.message, format!("Markdown node {heading} {cycle}"));
    assert_eq!(error.offset, 0);
    assert_eq!(document.text_content(heading), Err(error.clone()));
    assert_eq!(document.text_nodes(heading), Err(error.clone()));
    assert_eq!(
        document.text_content(emphasis),
        Err(MarkdownError {
            message: format!("Markdown node {emphasis} {cycle}"),
            offset: 5,
        })
    );
    // Each rule reports exactly one processing failure that names it, at the node whose walk failed.
    for (findings, rule, node, offset) in [
        (
            no_duplicate_heading(&document, Severity::Warn),
            "markdown/no-duplicate-heading",
            heading,
            0,
        ),
        (
            no_trailing_punctuation(&document, Severity::Warn),
            "markdown/no-trailing-punctuation",
            heading,
            0,
        ),
        (
            no_emphasis_as_heading(&document, Severity::Warn),
            "markdown/no-emphasis-as-heading",
            emphasis,
            5,
        ),
    ] {
        assert_eq!(findings.len(), 1, "{rule}");
        let failure: &Diagnostic = &findings[0];
        assert_eq!(failure.code, "core/processing-failure", "{rule}");
        assert!(failure.processing_failure, "{rule}");
        assert_eq!(failure.severity, Severity::Error, "{rule}");
        assert_eq!(failure.fix, None, "{rule}");
        assert_eq!(
            failure.message,
            format!("{rule} could not check this file: Markdown node {node} {cycle}")
        );
        assert_eq!(failure.labels[0].span.offset, offset, "{rule}");
        assert_eq!(failure.filename, "cycle.md", "{rule}");
    }
}
