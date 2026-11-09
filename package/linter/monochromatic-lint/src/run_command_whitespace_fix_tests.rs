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
use crate::markdown_source::MarkdownSource;
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
///  A marker is container structure, like a list marker or a Rust comment marker,
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
    assert!(fixed.lines().all(|line| line.starts_with("> ")), "{fixed}");
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
        let again: String = fix_with(
            &fixture.path,
            case.filename.as_str(),
            fixed.as_str(),
            EVERY_FIXABLE_RULE,
        );
        assert_eq!(again, fixed, "{} is not stable", case.name);
    }
}

/// The measurement on a real tree,
///  run once by hand:
///  `MONOCHROMATIC_LINT_PROPERTY_ROOT=<directory> MONOCHROMATIC_LINT_PROPERTY_STEP=<n> cargo nextest run --run-ignored only real_tree`.
///  Every `n`th Markdown and Rust file under the directory is fixed on standard input,
///  and the files whose tokens changed are listed in the failure.
#[test]
#[ignore = "measurement over a real tree named by MONOCHROMATIC_LINT_PROPERTY_ROOT, not a unit control"]
fn real_tree_keeps_every_token_under_the_whitespace_rule() {
    let root: String = std::env::var("MONOCHROMATIC_LINT_PROPERTY_ROOT")
        .expect("MONOCHROMATIC_LINT_PROPERTY_ROOT names the tree to measure");
    let step: usize = match std::env::var("MONOCHROMATIC_LINT_PROPERTY_STEP") {
        Ok(text) => text.parse::<usize>().expect("the step is a count"),
        Err(_) => 1,
    };
    let listing: std::process::Output = std::process::Command::new("git")
        .args(["-C", root.as_str(), "ls-files", "*.md", "*.mdx", "*.rs"])
        .output()
        .expect("git lists the tracked files");
    let names: Vec<String> = String::from_utf8(listing.stdout)
        .expect("tracked names are text")
        .lines()
        .map(String::from)
        .collect::<Vec<String>>();
    let fixture: Fixture = Fixture::new();
    let (mut measured, mut changed, mut broken): (usize, usize, Vec<String>) = (0, 0, Vec::new());
    for name in names.iter().step_by(step) {
        let Ok(source): Result<String, std::io::Error> =
            std::fs::read_to_string(Path::new(root.as_str()).join(name))
        else {
            continue;
        };
        measured += 1;
        let fixed: String = fix_with(
            &fixture.path,
            name.as_str(),
            source.as_str(),
            EVERY_FIXABLE_RULE,
        );
        if fixed != source {
            changed += 1;
        }
        if comparable_tokens(name.as_str(), fixed.as_str())
            != comparable_tokens(name.as_str(), source.as_str())
        {
            broken.push(name.clone());
        }
    }
    eprintln!(
        "measured {measured} files, {changed} changed by the fix, {} changed a token",
        broken.len()
    );
    assert!(broken.is_empty(), "token sequence changed in: {broken:#?}");
}
