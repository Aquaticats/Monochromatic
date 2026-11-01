//! What:
//!  Native Markdown/MDX parser-interface controls.
//! Why:
//!  Rule ports must receive exact source slices and must not inspect MDX code as prose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('native Markdown source', () => { /* Unicode, options and subtree controls */ });
//! ```

/// Import the production adapter and typed node decoders.
use super::MarkdownSource;
use satteri_ast::mdast::{MdastNodeType, decode_heading_data};

/// Find a concrete parsed node while failing if the fixture did not exercise that syntax.
fn node(document: &MarkdownSource, kind: MdastNodeType) -> u32 {
    for id in document.all_nodes() {
        if document.kind(*id) == kind {
            return *id;
        }
    }
    panic!("fixture did not create requested node kind");
}

/// Original offsets remain byte-based after BOM stripping and astral characters.
#[test]
fn bom_and_astral_source_slices_are_exact() {
    let document = MarkdownSource::new(
        String::from("source.md"),
        String::from("\u{feff}🚀\n\n# Title\n"),
        false,
    )
    .expect("Markdown parse");
    let heading = node(&document, MdastNodeType::Heading);
    assert_eq!(document.slice(heading), "# Title");
    assert_eq!(document.node_span(heading).offset, 9);
    assert_eq!(document.node_span(heading).length, 7);
    assert_eq!(document.node_span(heading).line, 3);
    assert_eq!(document.node_span(heading).column, 1);
    assert_eq!(decode_heading_data(document.data(heading)).depth, 1);
    assert_eq!(document.text_content(heading), Ok(String::from("Title")));
    assert_eq!(document.filename, "source.md");
    assert!(!document.mdx);
}

/// Explicit flags enable both frontmatter formats without enabling math.
#[test]
fn frontmatter_and_math_options_match_the_selected_contract() {
    let yaml = MarkdownSource::new(
        String::from("a.md"),
        String::from("---\na: b\n---\ntext\n"),
        false,
    )
    .expect("YAML parse");
    node(&yaml, MdastNodeType::Yaml);
    let toml = MarkdownSource::new(
        String::from("b.md"),
        String::from("+++\na = 1\n+++\ntext\n"),
        false,
    )
    .expect("TOML parse");
    node(&toml, MdastNodeType::Toml);
    let math = MarkdownSource::new(String::from("c.md"), String::from("$x$\n"), false)
        .expect("plain dollar text");
    assert!(
        !math
            .all_nodes()
            .iter()
            .any(|id| return math.kind(*id) == MdastNodeType::InlineMath)
    );
}

/// MDX code nodes and their children are hidden from prose-rule traversal.
#[test]
fn mdx_subtrees_are_not_prose_rule_inputs() {
    let document = MarkdownSource::new(
        String::from("a.mdx"),
        String::from("export const value = 1\n\n<div>inside</div>\n\nOutside\n"),
        true,
    )
    .expect("MDX parse");
    let esm = node(&document, MdastNodeType::MdxjsEsm);
    let jsx = node(&document, MdastNodeType::MdxJsxFlowElement);
    assert!(!document.visible_nodes().contains(&esm));
    assert!(!document.visible_nodes().contains(&jsx));
    for child in document.children(jsx) {
        assert!(!document.visible_nodes().contains(child));
    }
    assert!(
        document
            .visible_nodes()
            .iter()
            .any(|id| return document.kind(*id) == MdastNodeType::Paragraph)
    );
}

/// The root has no parent and every other reachable node names the container that lists it.
#[test]
fn parents_mirror_child_edges_and_stop_at_the_root() {
    let document: MarkdownSource = MarkdownSource::new(
        String::from("nested.md"),
        String::from("> - **deep** text\n\nplain\n"),
        false,
    )
    .expect("nested parse");
    // The root is checked first and without any ancestor walk: a wrong answer fails here instead of spinning.
    assert_eq!(document.parent(0), None);
    // Count the edges so an empty traversal cannot pass silently.
    let mut edges: usize = 0;
    for id in document.all_nodes() {
        for child in document.children(*id) {
            edges += 1;
            assert_eq!(document.parent(*child), Some(*id));
        }
    }
    // Every node except the root is exactly one container's child.
    assert_eq!(edges, document.all_nodes().len() - 1);
    // Two fixed upward steps prove distinct containers without an unbounded climb.
    let strong: u32 = node(&document, MdastNodeType::Strong);
    let paragraph: u32 = document
        .parent(strong)
        .expect("strong text sits in a paragraph");
    assert_eq!(document.kind(paragraph), MdastNodeType::Paragraph);
    let item: u32 = document
        .parent(paragraph)
        .expect("the paragraph sits in a list item");
    assert_eq!(document.kind(item), MdastNodeType::ListItem);
}

/// Empty source still has a usable root and no invalid range.
#[test]
fn empty_source_has_a_valid_root() {
    let document =
        MarkdownSource::new(String::from("empty.md"), String::new(), false).expect("empty parse");
    assert_eq!(document.kind(0), MdastNodeType::Root);
    assert_eq!(document.node_span(0).length, 0);
    assert_eq!(document.visible_nodes(), [0]);
}

/// Parser-reported MDX errors reject the partial tree and keep original byte offsets.
#[test]
fn reported_mdx_errors_are_processing_failures() {
    for source in ["<A>\n</B>\n", "\u{feff}<A>\n</B>\n"] {
        // The native error vector is a positive control: this fixture must actually exercise rejection.
        let native_errors: Vec<(usize, String)> =
            satteri_pulldown_cmark::parse(source, satteri_pulldown_cmark::Options::ENABLE_MDX).1;
        assert!(
            !native_errors.is_empty(),
            "native parser must report the mismatched closing tag"
        );
        let result: Result<MarkdownSource, super::MarkdownError> =
            MarkdownSource::new(String::from("broken.mdx"), String::from(source), true);
        let error: super::MarkdownError = match result {
            Ok(_) => panic!("a parser-reported MDX error must reject the tree"),
            Err(failure) => failure,
        };
        assert!(error.message.starts_with("MDX parsing failed:"));
        assert!(error.message.contains("closing tag"));
        assert_eq!(
            error.offset,
            source.find("</B>").expect("closing tag marker")
        );
        assert_eq!(error.to_string(), error.message);
    }
}

/// ESM node construction is not a complete JavaScript validator in the selected parser.
#[test]
fn esm_node_acceptance_does_not_claim_javascript_validity() {
    let document: MarkdownSource = MarkdownSource::new(
        String::from("esm.mdx"),
        String::from("export const =\n"),
        true,
    )
    .expect("native ESM construction does not report this JavaScript error");
    let esm: u32 = node(&document, MdastNodeType::MdxjsEsm);
    // Node ranges include the authored line ending even though the ESM value trims it.
    assert_eq!(document.slice(esm), "export const =\n");
    assert!(!document.visible_nodes().contains(&esm));
}
