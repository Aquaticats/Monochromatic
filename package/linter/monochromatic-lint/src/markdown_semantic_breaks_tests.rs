//! What:
//!  Semantic break integration through actual Markdown/MDX parses.
//! Why:
//!  Lexically plausible insertions must also preserve inline delimiters,
//!  container prefixes and positions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Apply advertised edits once, reparse, and require no remaining semantic-break findings.
//! ```

/// Import production checking,
///  parsing and grouped edits.
use super::semantic_line_breaks;
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::{Fix, apply_fixes};
use crate::markdown_source::MarkdownSource;

/// Run the rule through its actual native source model.
fn check(source: &str) -> Vec<Diagnostic> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("prose.md"), String::from(source), false)
            .expect("fixture");
    return semantic_line_breaks(&context, Severity::Error);
}

/// Require one-pass convergence while keeping every original byte except the separating spaces the breaks replace.
fn fixed(source: &str) -> String {
    let mut fixes: Vec<Fix> = Vec::<Fix>::new();
    for diagnostic in check(source) {
        fixes.push(diagnostic.fix.expect("fix"));
    }
    let output: String = apply_fixes(source, fixes.as_slice()).expect("apply").source;
    assert!(check(output.as_str()).is_empty(), "{output}");
    return output;
}

/// Breaks at inline tails land outside closing delimiters,
///  including nested wrappers.
#[test]
fn breaks_preserve_inline_delimiters_and_paragraph_tails() {
    assert_eq!(
        fixed("**First. Second.** Tail text here.\n"),
        "**First.\nSecond.**\nTail text here.\n"
    );
    assert_eq!(
        fixed("- **_Term._** Explanation continues.\n"),
        "- **_Term._**\n  Explanation continues.\n"
    );
    for source in [
        "**the end.**\n",
        "a lead and **the end.**\n",
        "- **Term.**\n  Explanation continues.\n",
    ] {
        assert!(check(source).is_empty(), "{source}");
    }
    for punctuation in [',', ';', ':', '?', '!'] {
        let source: String = format!("- **Term{punctuation}** rest here.\n");
        assert_eq!(check(source.as_str()).len(), 1);
    }
}

/// Each inline delimiter family and a nested quote/list keeps its own closing and continuation syntax.
#[test]
fn all_delimiter_families_and_nested_prefixes_are_preserved() {
    for source in [
        "~~Term.~~ More here.\n",
        "*Term.* More here.\n",
        "_Term._ More here.\n",
    ] {
        assert_eq!(check(source).len(), 1);
        let output: String = fixed(source);
        assert!(output.contains("\nMore here."), "{output}");
    }
    assert_eq!(
        fixed("- > first, second here.\n"),
        "- > first,\n  > second here.\n"
    );
}

/// Point diagnostics and inserted newlines preserve source units and the incumbent's newline choice.
#[test]
fn diagnostics_and_line_endings_use_original_source_positions() {
    let findings: Vec<Diagnostic> = check("one\ntwo\nthree\nfour, five\n");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].labels[0].span.line, 4);
    assert_eq!(findings[0].labels[0].span.column, 6);
    assert_eq!(findings[0].labels[0].span.length, 0);
    let rendered: String = crate::diagnostic::render(&findings).expect("JSONL point output");
    let record: serde_json::Value =
        serde_json::from_str::<serde_json::Value>(rendered.trim_end()).expect("one JSONL record");
    assert_eq!(record["labels"][0]["span"]["length"], 0);
    assert_eq!(record["labels"][0]["span"]["line"], 4);
    assert_eq!(
        fixed("First. Second sentence here.\r\n"),
        "First.\r\nSecond sentence here.\r\n"
    );
    assert_eq!(
        fixed("\u{feff}🚀 First. Second here.\n"),
        "\u{feff}🚀 First.\nSecond here.\n"
    );
    assert_eq!(
        fixed("> first, second word.\n"),
        "> first,\n> second word.\n"
    );
    assert_eq!(
        fixed("Sentence.  `code` follows here.\n"),
        "Sentence.\n `code` follows here.\n"
    );
}

/// Non-prose content and ambiguous tokens do not acquire breaks.
#[test]
fn non_prose_and_existing_breaks_are_excluded() {
    for source in [
        "# Title, here\n",
        "use `a, b` here.\n",
        "see [a, b](/p,q) here.\n",
        "```js\nconst a = 1, b = 2;\n```\n",
        "this is e.g. important here.\n",
        "pi is 3.14 and 1,000 here.\n",
        "wait... here.\n",
        "Node.js checker.TupleType here.\n",
        "First.\r\nSecond here.\r\n",
        "| A, B |\n| --- |\n| C, D |\n",
    ] {
        assert!(check(source).is_empty(), "{source}");
    }
    let spaced: String = format!("First.{}\nSecond here.\n", " ".repeat(300));
    assert!(check(spaced.as_str()).is_empty());
}

/// Following prose must not become a heading,
///  nested list,
///  quote,
///  code fence or raw HTML block.
#[test]
fn inserted_breaks_cannot_change_block_structure() {
    for suffix in [
        "# h", "###### h", "- item", "+ item", "* item", "> quote", "1) item", "```js", "~~~",
        "<div>", "---",
    ] {
        let source: String = format!("Intro sentence. {suffix}\n");
        assert!(check(source.as_str()).is_empty(), "{source}");
    }
    assert_eq!(check("Intro sentence. #nothashheading here.\n").len(), 1);
    assert_eq!(
        fixed("one, two. three word.\n"),
        "one,\ntwo.\nthree word.\n"
    );
}

/// A break replaces the space it breaks at,
///  so a continuation line is the block's prefix and nothing more.
#[test]
fn a_break_replaces_the_space_and_leaves_no_stray_indent() {
    assert_eq!(fixed("It naps. It wakes."), "It naps.\nIt wakes.");
    assert_eq!(
        fixed("> She said the cat naps. It wakes.\n"),
        "> She said the cat naps.\n> It wakes.\n"
    );
    assert_eq!(
        fixed("- The cat naps. It wakes.\n- Another, with a clause."),
        "- The cat naps.\n  It wakes.\n- Another,\n  with a clause."
    );
    assert_eq!(
        fixed("1. The cat naps. It wakes.\n"),
        "1. The cat naps.\n   It wakes.\n"
    );
    // A break at the end of a line has no space to replace and nothing else is removed.
    assert_eq!(
        fixed("Left,\nright, word here.\n"),
        "Left,\nright,\nword here.\n"
    );
    // Only the one separating space is consumed from a longer run.
    assert_eq!(
        fixed("Sentence.   Next here.\n"),
        "Sentence.\n  Next here.\n"
    );
    // A tab is not a space and is kept.
    assert_eq!(
        fixed("Sentence.\tNext here.\n"),
        "Sentence.\n\tNext here.\n"
    );
}

/// A continuation settled before breaks consumed their space has no break left to insert,
///  so a second pass leaves it exactly as it is.
#[test]
fn a_continuation_carrying_the_older_stray_space_stays_as_it_is() {
    for settled in [
        "> Personalitywise,\n>  she was clingy.\n",
        "- A list item,\n   with a comma.\n",
        "Plain prose,\n with a comma.\n",
    ] {
        assert!(check(settled).is_empty(), "{settled}");
        assert_eq!(fixed(settled), settled);
    }
    assert_eq!(
        fixed("Old shape,\n with the stray space. And more."),
        "Old shape,\n with the stray space.\nAnd more."
    );
}

/// A paragraph that is one tag has no prose to break,
///  whichever grammar failed to claim it as HTML.
#[test]
fn a_tag_like_paragraph_stays_unbroken() {
    for source in [
        "<PhotoScroll photos={[ '${path}/photos/sill.webp',]} />  \n",
        "<PhotoScroll photos={[\n    '${path}/photos/sill.webp', '${path}/photos/nap.webp',\n]} />\n",
        "\u{feff}<Tag a={[ 'x', 'y' ]} />\n",
    ] {
        assert!(check(source).is_empty(), "{source}");
        assert_eq!(fixed(source), source);
    }
    // Prose beside a tag is still prose.
    assert_eq!(check("See <b>this, here</b> now.\n").len(), 1);
    // A paragraph that only opens or only closes like a tag is prose.
    assert_eq!(check("<b>bold</b> then, more words here.\n").len(), 1);
    assert_eq!(check("Words first, then <b>tag</b>\n").len(), 1);
}
