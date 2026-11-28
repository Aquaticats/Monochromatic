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
use crate::diagnostic::{
    Diagnostic,
    Severity,
};
use crate::edits::{
    Fix,
    FixError,
    apply_fixes,
};
use crate::fix_loop::{
    FixStop,
    FixedSource,
    SourceChecker,
    fix_source,
};
use crate::markdown_source::{
    MarkdownSource,
    paragraph_placements,
};
use satteri_ast::mdast::MdastNodeType;

/// Run the rule through its actual native source model.
fn check(source: &str) -> Vec<Diagnostic> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("prose.md"), String::from(source), false)
            .expect("fixture");
    return semantic_line_breaks(&context, Severity::Error);
}

/// The rule as the fix loop's checker, so a test counts the passes a run of it changes.
struct RuleChecker;

/// Reparse each submitted snapshot and answer the rule's findings on it.
impl SourceChecker for RuleChecker {
    fn check(
        &mut self,
        source: &str,
    ) -> Result<Vec<Diagnostic>, FixError> {
        let context: MarkdownSource =
            match MarkdownSource::new(String::from("prose.md"), String::from(source), false) {
                Ok(parsed) => parsed,
                Err(error) => {
                    return Err(FixError {
                        message: error.message,
                    });
                }
            };
        return Ok(semantic_line_breaks(&context, Severity::Error));
    }
}

/// Run the whole bounded fix loop over one source, as one `--fix` run does.
fn fix_loop(source: &str) -> FixedSource {
    let mut checker: RuleChecker = RuleChecker;
    return fix_source(source, &mut checker).expect("fix");
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

/// Each paragraph with the kinds of the containers around it and its words,
///  which is everything about where text sits that a line break must not change.
///  The extraction itself is shared with the whole-invocation property check.
fn placements(source: &str) -> Vec<(Vec<MdastNodeType>, Vec<String>)> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("prose.md"), String::from(source), false)
            .expect("fixture");
    return paragraph_placements(&context).expect("an acyclic parse");
}

/// A break inside a quotation re-emits every `>` marker of the line the paragraph opens on,
///  so the continuation line stays at the same quote depth in the raw text and not only
///  through the laziness CommonMark allows,
///  and the text stays in the containers it came from.
#[test]
fn a_break_in_a_quotation_keeps_the_marker_of_every_depth() {
    for (source, expected) in [
        ("> first, second here.\n", "> first,\n> second here.\n"),
        (
            "> > first, second here. And more\n> > text.\n",
            "> > first,\n> > second here.\n> > And more\n> > text.\n",
        ),
        (
            "- > first, second here.\n  > third, fourth here.\n",
            "- > first,\n  > second here.\n  > third,\n  > fourth here.\n",
        ),
        (
            "> - first, second here.\n> - third, fourth here.\n",
            "> - first,\n>   second here.\n> - third,\n>   fourth here.\n",
        ),
        (
            "1. > first, second here.\n",
            "1. > first,\n   > second here.\n",
        ),
        (
            ">>> foo, bar. Then rest\n> bar\n>>baz\n",
            ">>> foo,\n>>> bar.\n>>> Then rest\n> bar\n>>baz\n",
        ),
    ] {
        let output: String = fixed(source);
        assert_eq!(output, expected, "{source:?}");
        assert_eq!(
            placements(output.as_str()),
            placements(source),
            "{output:?}"
        );
    }
}

/// Only the lines the fix inserts carry the copied prefix:
///  those get the paragraph's own depth,
///  while this fixture's author-written lines keep the bytes they were written with,
///  because the fixture's tail holds three words and so carries no join.
///  The tail holds three words so the join of a fragment the old wrap cut does not also apply here.
#[test]
fn inserted_lines_take_the_paragraph_depth_and_existing_lines_keep_their_bytes() {
    let source: &str = "> > deep, words here now\nlazy, line here.\n";
    let output: String = fixed(source);
    assert_eq!(
        output,
        "> > deep,\n> > words here now\nlazy,\n> > line here.\n"
    );
    assert_eq!(
        placements(output.as_str()),
        placements(source),
        "{output:?}"
    );
}

/// A fragment the old wrap cut off the end of a line joins the line that continues its clause,
///  so the article that ended an old line does not stand alone.
#[test]
fn a_fragment_the_old_wrap_cut_joins_the_clause_that_continues_it() {
    assert_eq!(
        fixed("Owns the record family, the\nmanaged lock's words, the finder.\n"),
        "Owns the record family,\nthe managed lock's words,\nthe finder.\n"
    );
    assert_eq!(
        fixed("It naps. It\nwakes by the door.\n"),
        "It naps.\nIt wakes by the door.\n"
    );
}

/// A fragment inside a list item joins the line that continues the same item,
///  at the column the item's content starts at.
#[test]
fn a_fragment_in_a_list_item_joins_within_its_item() {
    assert_eq!(
        fixed("- first, the\n  managed words here.\n"),
        "- first,\n  the managed words here.\n"
    );
}

/// A line the rule itself wants to end at a break-point character stays alone,
///  because joining it would only have the rule break it again.
#[test]
fn a_fragment_ending_at_a_break_point_stays_and_the_fix_settles() {
    let source: &str = "The counts were\n4,\n5,\nheld,\nfor the pass.\n";
    assert_eq!(fixed(source), source);
    let broken: &str = "The counts were the words, 4,\n5,\nheld,\nfor the pass.\n";
    let output: String = fixed(broken);
    assert_eq!(
        output,
        "The counts were the words,\n4,\n5,\nheld,\nfor the pass.\n"
    );
    assert!(check(output.as_str()).is_empty(), "{output}");
}

/// A break-point character inside a closing emphasis delimiter is the same ending:
///  the rule wants the line to end after the delimiter,
///  so a joined fragment is broken again on the next pass.
#[test]
fn a_fragment_ending_inside_closing_emphasis_stays_and_the_fix_settles() {
    for (source, expected) in [
        (
            "One two, *held,*\nfour five.\n",
            "One two,\n*held,*\nfour five.\n",
        ),
        (
            "One two, **held,**\nfour five.\n",
            "One two,\n**held,**\nfour five.\n",
        ),
        (
            "One two, ~~held,~~\nfour five.\n",
            "One two,\n~~held,~~\nfour five.\n",
        ),
        (
            "One two, *a b,*\nfour five.\n",
            "One two,\n*a b,*\nfour five.\n",
        ),
        (
            "One two, ***held,***\nfour five.\n",
            "One two,\n***held,***\nfour five.\n",
        ),
        (
            "One two, **_held,_**\nfour five.\n",
            "One two,\n**_held,_**\nfour five.\n",
        ),
        (
            "One two, *held;*\nfour five.\n",
            "One two,\n*held;*\nfour five.\n",
        ),
        (
            "One two, *held.*\nfour five.\n",
            "One two,\n*held.*\nfour five.\n",
        ),
        (
            "One two, ~held,~\nfour five.\n",
            "One two,\n~held,~\nfour five.\n",
        ),
    ] {
        // `fixed` applies exactly one pass and then requires no finding to be left.
        assert_eq!(fixed(source), expected, "{source:?}");
    }
}

/// A star, an underscore or a bracket that closes nothing is not a delimiter the rule breaks past,
///  so the fragment joins the line that continues its clause and the run settles in one pass;
///  only the parsed tree tells that star from one that closes an emphasis.
#[test]
fn a_delimiter_that_closes_nothing_joins_in_one_pass() {
    for written in [
        "(*held,*)",
        "\"*held,*\"",
        "held,*",
        "held,__",
        "snake_case,_",
        "x*,*",
        "**held**,**",
    ] {
        let source: String = format!("One two, {written}\nfour five.\n");
        assert_eq!(
            fixed(source.as_str()),
            format!("One two,\n{written} four five.\n"),
            "{written}"
        );
        let result: FixedSource = fix_loop(source.as_str());
        assert_eq!(result.changed_passes, 1, "{written}");
        assert_eq!(result.stop, FixStop::Unchanged, "{written}");
        assert!(result.diagnostics.is_empty(), "{written}");
    }
}

/// Two breaks on one line leave two fragments that end at one line ending,
///  and one line ending takes one join, so the pass settles where the fixes would otherwise compete.
#[test]
fn two_breaks_sharing_a_line_ending_settle_in_one_pass() {
    for (source, expected) in [
        (
            "clause one, x, y\nnext line here.\n",
            "clause one,\nx,\ny next line here.\n",
        ),
        (
            "clause one, x, y, z w\nnext line here.\n",
            "clause one,\nx,\ny,\nz w next line here.\n",
        ),
    ] {
        assert_eq!(fixed(source), expected, "{source:?}");
    }
}

/// What:
///  The passes one `--fix` run changes on inputs that hold both shapes of a join the rule declines late.
/// Why:
///  A join the next pass would undo costs a second pass, and a paragraph of fifteen such lines
///  spends the loop's whole budget and leaves a finding after the first run;
///  counting changed passes is what shows one pass settles.
#[test]
fn both_shapes_change_one_pass_and_no_more() {
    // The emphasis-closed fragment, the two breaks on one line, each alone and both in one document.
    for source in [
        "One two, *held,*\nfour five.\n",
        "clause one, x, y\nnext line here.\n",
        "One two, *held,*\nfour five.\n\nclause one, x, y\nnext line here.\n",
        "One two, **held,**\nfour five.\n\nclause one, x, y, z w\nnext line here.\n",
    ] {
        let result: FixedSource = fix_loop(source);
        assert_eq!(result.changed_passes, 1, "{source:?}");
        assert_eq!(result.stop, FixStop::Unchanged, "{source:?}");
        assert!(result.diagnostics.is_empty(), "{source:?}");
    }
    // Fifteen lines that each end inside emphasis: before this control, the loop spent its ten passes.
    let chain: String = format!(
        "One two, *a,*\n{}",
        [
            "*b,*", "*c,*", "*d,*", "*e,*", "*f,*", "*g,*", "*h,*", "*i,*", "*j,*", "*k,*", "*l,*",
            "*m,*", "*n,*", "*o,*"
        ]
        .map(|line| return format!("{line}\n"))
        .concat()
    );
    let result: FixedSource = fix_loop(chain.as_str());
    assert_eq!(result.changed_passes, 1, "{chain:?}");
    assert_eq!(result.stop, FixStop::Unchanged, "{chain:?}");
    assert!(result.diagnostics.is_empty(), "{chain:?}");
}

/// A line the fix writes carries the indent the paragraph's first continuation line already has,
///  so the lines of one paragraph are written at one depth.
#[test]
fn a_line_the_fix_writes_takes_the_paragraphs_own_indent() {
    let source: &str = "Owns the record family, the managed lock's words and the finder of\n the pass, and the recipe too.\n";
    assert_eq!(
        fixed(source),
        "Owns the record family,\n the managed lock's words and the finder of\n the pass,\n and the recipe too.\n"
    );
}

/// A following line that carries a quote marker is a token a whitespace-only fix may not delete,
///  so the marker and the line stand as they are.
#[test]
fn a_quoted_line_keeps_its_marker_and_its_line() {
    assert_eq!(
        fixed("> first, the\n> managed words here.\n"),
        "> first,\n> the\n> managed words here.\n"
    );
}

/// A fragment that would join a fenced block's opening line stays where it is,
///  because the fence marker is not whitespace.
#[test]
fn a_fence_beside_a_fragment_keeps_its_bytes() {
    let source: &str = "Owns the record family, the\n```js\nconst a = 1;\n```\n";
    let output: String = fixed(source);
    assert_eq!(
        output,
        "Owns the record family,\nthe\n```js\nconst a = 1;\n```\n"
    );
    assert!(output.contains("```js\nconst a = 1;\n```"), "{output}");
}

/// A break inside a partially marked lazy line takes the paragraph's full depth,
///  while the author's own shorter line keeps the bytes it was written with.
#[test]
fn a_partially_marked_lazy_line_gets_the_paragraph_depth_on_the_inserted_line() {
    let source: &str = "> > opening words\n> continued, further words.\n";
    let output: String = fixed(source);
    assert_eq!(
        output,
        "> > opening words\n> continued,\n> > further words.\n"
    );
    assert_eq!(
        placements(output.as_str()),
        placements(source),
        "{output:?}"
    );
}

/// A list item's continuation line keeps its item by starting at the item's content column,
///  because writing a list marker there again would open a new block instead of continuing this one:
///  a paragraph's containers offer only list markers and quote markers,
///  and the quote markers are the family that has to be written again.
#[test]
fn a_list_continuation_line_keeps_its_item_by_indentation() {
    for (source, expected) in [
        ("- first, second here.\n", "- first,\n  second here.\n"),
        ("* first, second here.\n", "* first,\n  second here.\n"),
        ("+ first, second here.\n", "+ first,\n  second here.\n"),
        ("1. first, second here.\n", "1. first,\n   second here.\n"),
        (
            "10. first, second here.\n",
            "10. first,\n    second here.\n",
        ),
        (
            "- [ ] first, second here.\n",
            // The task marker sits before the paragraph, so the blank run aligns to the paragraph's own column.
            "- [ ] first,\n      second here.\n",
        ),
        (
            "- - first, second here.\n",
            "- - first,\n    second here.\n",
        ),
        (
            "-\n  first, second here.\n",
            "-\n  first,\n  second here.\n",
        ),
    ] {
        let output: String = fixed(source);
        assert_eq!(output, expected, "{source:?}");
        assert_eq!(
            placements(output.as_str()),
            placements(source),
            "{output:?}"
        );
    }
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
        // The inserted line takes the indent the paragraph's first continuation line carries,
        // so the paragraph's lines are written at one depth.
        "Old shape,\n with the stray space.\n And more."
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
