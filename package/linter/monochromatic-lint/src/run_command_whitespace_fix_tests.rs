//! What:
//!  Reproductions of the fixer rewriting content,
//!  and the property check that a whitespace-only fix leaves every whitespace-separated token in order.
//! Why:
//!  `--fix` runs in the commit path and in agents' hands;
//!  a fix that moves a token into or out of code,
//!  a table or a quotation changes what a reader sees,
//!  so each defect has a whole-invocation fixture and the property is checked over fixtures and a real tree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('fix changes only whitespace', () => { /* one fixture per defect, then the token property */ });
//! ```

/// Import the shared invocation fixtures.
use crate::markdown_source::{
    MarkdownSource, block_containers, paragraph_placements, verbatim_blocks,
};
use crate::run_output::RunOutput;
use crate::run_test_support::{codes, read, run, write};
use crate::test_fs::Fixture;
use satteri_ast::mdast::MdastNodeType;
use serde::Deserialize;
use std::path::Path;

/// The configuration file name every fixture tree uses.
const CONFIG_FILE: &str = "monochromatic-lint.config.jsonc";

/// Every fixable Markdown rule,
///  for Markdown files and for the Markdown inside rustdoc comments:
///  plain `--fix` must still change no token with all of them enabled,
///  because only the rule that breaks lines may fix without a content flag.
const EVERY_FIXABLE_RULE: &str = r#"[
  { "files": ["**/*.md", "**/*.mdx", "**/*.rs/*.md", "**/*.md/*.rs/*.md"], "rules": {
    "markdown/semantic-line-breaks": { "severity": "error" },
    "markdown/commands-show-output": { "severity": "error" },
    "markdown/no-trailing-punctuation": { "severity": "error" },
    "markdown/no-bare-urls": { "severity": "error" },
    "markdown/fenced-code-language": { "severity": "error" },
    "markdown/link-image-reference-definitions": { "severity": "error" },
    "markdown/link-image-style": { "severity": "error" },
    "markdown/no-pipe-tables": { "severity": "error" }
  } }
]"#;

/// The whitespace rule plus the two rules whose fixes the reported defects came from.
const REPORTED_RULES: &str = r#"[
  { "files": ["**/*.md", "**/*.mdx", "**/*.rs/*.md", "**/*.md/*.rs/*.md"], "rules": {
    "markdown/semantic-line-breaks": { "severity": "error" },
    "markdown/no-bare-urls": { "severity": "error" },
    "markdown/no-pipe-tables": { "severity": "error" }
  } }
]"#;

/// One fixture of the property check,
///  read from `fixtures/whitespace-fix-cases.json`.
#[derive(Deserialize)]
struct Case {
    /// What the source exercises.
    name: String,
    /// The logical file name,
    ///  which picks the language and the configuration blocks.
    filename: String,
    /// The exact bytes the fixer receives.
    source: String,
}

/// The comment marker that opens a line of Rust comment text,
///  which a broken doc comment line gains on its continuation line.
///  Only that one leading marker is set aside:
///  a marker later in the line is quoted source and stays a token.
fn without_leading_comment_marker(line: &str) -> &str {
    let trimmed: &str = line.trim_start();
    for marker in ["///", "//!", "//"] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return rest;
        }
    }
    return trimmed;
}

/// Drop the quote markers that open a line,
///  which a broken line inside a quotation gains at the depth it continues.
///  A marker is container structure,
/// like a list marker or a Rust comment marker,
///  and not a token a reader sees;
///  the tests that require a particular depth compare exact bytes and parsed containers instead.
fn without_leading_quote_markers(line: &str) -> &str {
    let mut rest: &str = line.trim_start();
    while let Some(after_marker) = rest.strip_prefix('>') {
        rest = after_marker.trim_start();
    }
    return rest;
}

/// The tokens a language compares:
///  Rust source without the doc comment marker each line starts with,
///  and every language without the quote markers that place a line in a quotation.
fn comparable_tokens<'text>(filename: &str, text: &'text str) -> Vec<&'text str> {
    let rust: bool = filename.ends_with(".rs");
    let mut found: Vec<&'text str> = Vec::<&'text str>::new();
    for line in text.lines() {
        let without_comment: &'text str = if rust {
            without_leading_comment_marker(line)
        } else {
            line
        };
        found.extend(without_leading_quote_markers(without_comment).split_whitespace());
    }
    return found;
}

/// Fix one source on standard input under a configuration and return the fixed source.
fn fix_with(root: &Path, filename: &str, source: &str, configuration: &str) -> String {
    write(root, CONFIG_FILE, configuration);
    let output: RunOutput = run(
        root,
        &["--stdin", "--stdin-filename", filename, "--fix"],
        source,
    );
    assert_ne!(output.exit_code, 2, "{}", output.stderr);
    return output.stdout;
}

/// Run `--fix` over one file of a disposable tree and return its new text and the codes still reported.
fn fix_file(filename: &str, source: &str, configuration: &str) -> (String, Vec<String>) {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG_FILE, configuration);
    write(&fixture.path, filename, source);
    let output: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_ne!(output.exit_code, 2, "{}", output.stderr);
    return (read(&fixture.path, filename), codes(output.stdout.as_str()));
}

/// Parse text the way the rule does,
///  for questions about which block holds which text.
fn parse(source: &str) -> MarkdownSource {
    return MarkdownSource::new(String::from("quote.md"), String::from(source), false)
        .expect("fixture parses");
}

/// A bare URL that the parser read together with a code span's closing backtick is not wrapped:
///  wrapping `https://x.example/` plus the backtick would move the backtick into the link.
#[test]
fn a_url_that_swallowed_a_backtick_is_not_wrapped() {
    let source: &str = "- `curl 'a/\n- b'`, see `<script>` tags.\n  All except `https://x.example/` (DNS\n  failure).\n";
    let (fixed, remaining): (String, Vec<String>) = fix_file("a.md", source, REPORTED_RULES);
    assert_eq!(fixed, source);
    assert!(!fixed.contains("`<https"), "{fixed}");
    assert_eq!(remaining, ["markdown/no-bare-urls"]);
}

/// A pipe table is reported and left as it is:
///  the HTML table broke backticks in cells and the repository forbids tables of either kind.
#[test]
fn a_pipe_table_is_left_as_it_is() {
    let source: &str = "| Name | Code |\n| - | - |\n| one | `a\\|b` |\n";
    let (fixed, remaining): (String, Vec<String>) = fix_file("a.md", source, REPORTED_RULES);
    assert_eq!(fixed, source);
    assert_eq!(remaining, ["markdown/no-pipe-tables"]);
}

/// Doc comments inside a fenced Rust listing in Markdown are quoted source:
///  breaking their lines adds comment markers to the quotation.
#[test]
fn doc_comments_in_a_fenced_listing_in_markdown_are_left_as_written() {
    let source: &str =
        "Quoted source:\n\n```rust\n/// Adds one. Then returns it.\nfn f() {}\n```\n";
    let (fixed, remaining): (String, Vec<String>) = fix_file("a.md", source, EVERY_FIXABLE_RULE);
    assert_eq!(fixed, source);
    assert_eq!(remaining, ["markdown/semantic-line-breaks"]);
}

/// A doc example inside a Rust file's doc comment is a listing too,
///  while the file's own doc comments are still broken at their markers.
#[test]
fn a_doc_example_in_a_rust_file_is_left_as_written_beside_its_broken_doc_comment() {
    let source: &str = "/// Adds one. Then returns it.\n///\n/// ```rust\n/// /// Inner doc. More words.\n/// fn g() {}\n/// ```\nfn f() {}\n";
    let (fixed, _): (String, Vec<String>) = fix_file("a.rs", source, EVERY_FIXABLE_RULE);
    assert_eq!(
        fixed,
        "/// Adds one.\n/// Then returns it.\n///\n/// ```rust\n/// /// Inner doc. More words.\n/// fn g() {}\n/// ```\nfn f() {}\n"
    );
}

/// A line break inside a blockquote re-emits the marker at the depth the paragraph sits at,
///  so the raw text keeps the quotation and a later edit cannot move the broken line out of it.
#[test]
fn a_blockquote_break_keeps_the_marker_at_the_same_depth() {
    let source: &str = "> I think we can make it much better. Because we control the code.\n> Second line, here.\n";
    let (fixed, remaining): (String, Vec<String>) = fix_file("a.md", source, EVERY_FIXABLE_RULE);
    assert_eq!(
        fixed,
        "> I think we can make it much better.\n> Because we control the code.\n> Second line,\n> here.\n"
    );
    assert_eq!(remaining, Vec::<String>::new());
    assert_eq!(
        comparable_tokens("a.md", fixed.as_str()),
        comparable_tokens("a.md", source)
    );
    // Every written line carries its marker, so the quotation holds in the raw text too.
    /// Whether every line of one quotation starts with its marker and a space.
    fn every_line_quoted(text: &str) -> bool {
        return text.lines().all(|line| return line.starts_with("> "));
    }
    assert!(every_line_quoted(fixed.as_str()), "{fixed}");
    let context: MarkdownSource = parse(fixed.as_str());
    let mut root_children: Vec<MdastNodeType> = Vec::<MdastNodeType>::new();
    for id in context.all_nodes() {
        if context.kind(*id) != MdastNodeType::Root {
            continue;
        }
        for child in context.children(*id) {
            root_children.push(context.kind(*child));
        }
    }
    assert_eq!(root_children, [MdastNodeType::Blockquote]);
}

/// Every fixture keeps its token sequence under plain `--fix` with every fixable Markdown rule enabled,
///  and a second run changes nothing.
#[test]
fn whitespace_fix_keeps_every_token_of_every_fixture() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("../fixtures/whitespace-fix-cases.json"))
            .expect("fixture file is a list of cases");
    assert!(!cases.is_empty());
    let fixture: Fixture = Fixture::new();
    for case in &cases {
        let fixed: String = fix_with(
            &fixture.path,
            case.filename.as_str(),
            case.source.as_str(),
            EVERY_FIXABLE_RULE,
        );
        assert_eq!(
            comparable_tokens(case.filename.as_str(), fixed.as_str()),
            comparable_tokens(case.filename.as_str(), case.source.as_str()),
            "{}: {fixed}",
            case.name
        );
        if case.filename.ends_with(".md") || case.filename.ends_with(".mdx") {
            let mdx: bool = case.filename.ends_with(".mdx");
            let before = MarkdownSource::new(case.filename.clone(), case.source.clone(), mdx)
                .expect("fixture parses");
            let after = MarkdownSource::new(case.filename.clone(), fixed.clone(), mdx)
                .expect("fixed fixture parses");
            assert_eq!(
                block_containers(&before).expect("an acyclic parse"),
                block_containers(&after).expect("an acyclic parse"),
                "{} changed a block boundary",
                case.name
            );
            assert_eq!(
                verbatim_blocks(&before).expect("an acyclic parse"),
                verbatim_blocks(&after).expect("an acyclic parse"),
                "{} changed quoted source",
                case.name
            );
        }
        let again: String = fix_with(
            &fixture.path,
            case.filename.as_str(),
            fixed.as_str(),
            EVERY_FIXABLE_RULE,
        );
        assert_eq!(again, fixed, "{} is not stable", case.name);
    }
}

/// The property's comparisons can each fail:
///  a paragraph moved into another quotation,
///  one changed word,
///  one quotation split in two,
///  and one character changed inside a fence are all detected,
///  so a green run is not a comparison that never looks.
#[test]
fn the_property_comparisons_detect_a_moved_word_a_split_quotation_and_a_changed_fence() {
    let plain: MarkdownSource = parse("> quoted words\n\nplain text\n");
    let quoted: MarkdownSource = parse("> quoted words\n\n> plain text\n");
    assert_ne!(
        paragraph_placements(&plain).expect("an acyclic parse"),
        paragraph_placements(&quoted).expect("an acyclic parse")
    );
    assert_ne!(
        comparable_tokens("a.md", "one two\n"),
        comparable_tokens("a.md", "one three\n")
    );
    // One quotation with two paragraphs is not the same block structure as two quotations,
    // which the paragraph comparison above cannot tell apart.
    let joined: MarkdownSource = parse("> first\n>\n> second\n");
    let split: MarkdownSource = parse("> first\n\n> second\n");
    assert_eq!(
        paragraph_placements(&joined).expect("an acyclic parse"),
        paragraph_placements(&split).expect("an acyclic parse")
    );
    assert_ne!(
        block_containers(&joined).expect("an acyclic parse"),
        block_containers(&split).expect("an acyclic parse")
    );
    // A marker the token comparison sets aside inside a fence is still compared as quoted source.
    let kept: MarkdownSource = parse("```text\n> keep\n```\n");
    let lost: MarkdownSource = parse("```text\nkeep\n```\n");
    assert_eq!(
        comparable_tokens("a.md", "```text\n> keep\n```\n"),
        comparable_tokens("a.md", "```text\nkeep\n```\n")
    );
    assert_ne!(
        verbatim_blocks(&kept).expect("an acyclic parse"),
        verbatim_blocks(&lost).expect("an acyclic parse")
    );
}

/// The measurement on a real tree,
///  run through the package task:
///  `MONOCHROMATIC_LINT_PROPERTY_ROOT=<directory> MONOCHROMATIC_LINT_PROPERTY_STEP=<n> mise run //package/linter/monochromatic-lint:test:property`.
///  Every `n`th tracked Markdown,
/// MDX and Rust file under the directory is fixed on standard input.
///  Tokens are compared for every file;
///  every Markdown host is parsed before and after so its paragraphs,
///  its block boundaries and its quoted source are compared as well.
///  Nothing is skipped:
/// a host the linter cannot process,
///  a file that cannot be read and a host that stops parsing are failures,
///  so the measurement cannot pass by measuring less.
#[test]
#[ignore = "measurement over a real tree named by MONOCHROMATIC_LINT_PROPERTY_ROOT, not a unit control"]
fn real_tree_keeps_every_token_and_block_structure_under_the_whitespace_rule() {
    let root: String = std::env::var("MONOCHROMATIC_LINT_PROPERTY_ROOT")
        .expect("MONOCHROMATIC_LINT_PROPERTY_ROOT names the tree to measure");
    let step: usize = match std::env::var("MONOCHROMATIC_LINT_PROPERTY_STEP") {
        Ok(text) => text.parse::<usize>().expect("the step is a count"),
        Err(_) => 1,
    };
    let listing: std::process::Output = std::process::Command::new("git")
        .args([
            "-C",
            root.as_str(),
            "ls-files",
            "-z",
            "*.md",
            "*.mdx",
            "*.rs",
        ])
        .output()
        .expect("git runs");
    assert!(
        listing.status.success(),
        "git lists {root}: {}",
        String::from_utf8_lossy(listing.stderr.as_slice())
    );
    let names: Vec<String> = String::from_utf8(listing.stdout)
        .expect("tracked names are text")
        .split('\0')
        .filter(|name| return !name.is_empty())
        .map(String::from)
        .collect::<Vec<String>>();
    assert!(!names.is_empty(), "{root} lists no Markdown or Rust file");
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG_FILE, EVERY_FIXABLE_RULE);
    let mut measured: usize = 0;
    let mut changed: usize = 0;
    let mut tokens: usize = 0;
    let mut structured: usize = 0;
    let mut quoted: usize = 0;
    let mut broken_tokens: Vec<String> = Vec::<String>::new();
    let mut moved: Vec<String> = Vec::<String>::new();
    let mut reshaped: Vec<String> = Vec::<String>::new();
    let mut misquoted: Vec<String> = Vec::<String>::new();
    for name in names.iter().step_by(step) {
        let source: String = std::fs::read_to_string(Path::new(root.as_str()).join(name))
            .unwrap_or_else(|error| panic!("{name} is readable text: {error}"));
        let output: RunOutput = run(
            &fixture.path,
            &["--stdin", "--stdin-filename", name.as_str(), "--fix"],
            source.as_str(),
        );
        assert_ne!(
            output.exit_code, 2,
            "{name} is processed: {}",
            output.stderr
        );
        measured += 1;
        let fixed: String = output.stdout;
        if fixed != source {
            changed += 1;
        }
        let written: Vec<&str> = comparable_tokens(name.as_str(), source.as_str());
        tokens += written.len();
        if written != comparable_tokens(name.as_str(), fixed.as_str()) {
            broken_tokens.push(name.clone());
        }
        if !(name.ends_with(".md") || name.ends_with(".mdx")) {
            continue;
        }
        let mdx: bool = name.ends_with(".mdx");
        let before: MarkdownSource = MarkdownSource::new(name.clone(), source, mdx)
            .unwrap_or_else(|error| panic!("{name} parses: {error}"));
        let after: MarkdownSource = MarkdownSource::new(name.clone(), fixed, mdx)
            .unwrap_or_else(|error| panic!("{name} still parses after the fix: {error}"));
        structured += 1;
        let placement_before = paragraph_placements(&before).expect("an acyclic parse");
        let placement_after = paragraph_placements(&after).expect("an acyclic parse");
        if placement_before != placement_after {
            moved.push(name.clone());
        }
        let shape_before = block_containers(&before).expect("an acyclic parse");
        let shape_after = block_containers(&after).expect("an acyclic parse");
        if shape_before != shape_after {
            reshaped.push(name.clone());
        }
        let quoted_before = verbatim_blocks(&before).expect("an acyclic parse");
        let quoted_after = verbatim_blocks(&after).expect("an acyclic parse");
        quoted += quoted_before.len();
        if quoted_before != quoted_after {
            misquoted.push(name.clone());
        }
    }
    eprintln!(
        "measured {measured} files ({changed} changed by the fix, {tokens} tokens, {structured} Markdown hosts parsed, {quoted} quoted blocks)"
    );
    assert!(
        broken_tokens.is_empty(),
        "token sequence changed in: {broken_tokens:#?}"
    );
    assert!(
        moved.is_empty(),
        "paragraph containers or words changed in: {moved:#?}"
    );
    assert!(
        reshaped.is_empty(),
        "block boundaries changed in: {reshaped:#?}"
    );
    assert!(
        misquoted.is_empty(),
        "quoted source changed in: {misquoted:#?}"
    );
}
