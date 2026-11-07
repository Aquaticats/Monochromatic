//! What:
//!  Whole-invocation controls for the two fix classes:
//!  plain `--fix` applies only fixes that change whitespace,
//!  and `--fix-content` also applies the fixes that rewrite content.
//! Why:
//!  `--fix` runs in the commit path and in agents' hands;
//!  a fix that rewrites content must never run there,
//!  which is how the first whitespace defects reached the repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('fix classes', () => { /* per content rule: plain --fix leaves it, --fix-content applies it */ });
//! ```

/// Import the shared invocation fixtures.
use crate::run_output::RunOutput;
use crate::run_test_support::{CONFIG, codes, read, records, run, write};
use crate::test_fs::Fixture;

/// Measured with coreutils `sha256sum` for the bytes `image bytes`.
const IMAGE_OID: &str = "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec";

/// One rule whose fix rewrites content,
///  with a source it fires on and the text its fix produces.
struct ContentCase {
    /// The rule identifier without the `markdown/` prefix.
    rule: &'static str,
    /// The source the rule reports and fixes.
    source: &'static str,
    /// The source after the rule's own fix.
    fixed: &'static str,
}

/// Every Markdown rule other than the line-break rule that attaches a fix,
///  measured against the installed 0.5.2-alpha.3 executable.
const CONTENT_CASES: [ContentCase; 6] = [
    ContentCase {
        rule: "commands-show-output",
        source: "```sh\n$ ls\n```\n",
        fixed: "```sh\nls\n```\n",
    },
    ContentCase {
        rule: "no-trailing-punctuation",
        source: "# Title.\n",
        fixed: "# Title\n",
    },
    ContentCase {
        rule: "no-bare-urls",
        source: "See https://x.example/a now.\n",
        fixed: "See <https://x.example/a> now.\n",
    },
    ContentCase {
        rule: "fenced-code-language",
        source: "```\ncode\n```\n",
        fixed: "```text\ncode\n```\n",
    },
    ContentCase {
        rule: "link-image-reference-definitions",
        source: "Text.\n\n[unused]: https://x.example/\n",
        fixed: "Text.\n\n",
    },
    ContentCase {
        rule: "link-image-style",
        source: "See [label] now.\n\n[label]: https://x.example/\n",
        fixed: "See [label][] now.\n\n[label]: https://x.example/\n",
    },
];

/// A configuration that selects exactly one Markdown rule.
fn only(rule: &str) -> String {
    return format!(
        r#"[{{ "files": ["**/*.md"], "rules": {{ "markdown/{rule}": {{ "severity": "error" }} }} }}]"#
    );
}

/// Plain `--fix` leaves the file of every content rule as written and still reports the finding.
#[test]
fn plain_fix_leaves_every_content_rule_alone() {
    for case in &CONTENT_CASES {
        let fixture: Fixture = Fixture::new();
        write(&fixture.path, CONFIG, only(case.rule).as_str());
        write(&fixture.path, "a.md", case.source);
        let output: RunOutput = run(&fixture.path, &["--fix"], "");
        assert_eq!(read(&fixture.path, "a.md"), case.source, "{}", case.rule);
        assert_eq!(output.exit_code, 1, "{}", case.rule);
        assert_eq!(
            codes(output.stdout.as_str()),
            [format!("markdown/{}", case.rule)],
            "{}",
            case.rule
        );
    }
}

/// `--fix-content` alone fixes,
///  applies every content rule's fix and leaves nothing of that rule to report.
#[test]
fn fix_content_applies_every_content_rule() {
    for case in &CONTENT_CASES {
        let fixture: Fixture = Fixture::new();
        write(&fixture.path, CONFIG, only(case.rule).as_str());
        write(&fixture.path, "a.md", case.source);
        let output: RunOutput = run(&fixture.path, &["--fix-content"], "");
        assert_eq!(read(&fixture.path, "a.md"), case.fixed, "{}", case.rule);
        assert_eq!(
            (output.exit_code, output.stdout.as_str()),
            (0, ""),
            "{}",
            case.rule
        );
    }
}

/// On standard input the same split holds:
///  plain `--fix` returns the source unchanged and `--fix-content` returns the rewritten source.
#[test]
fn standard_input_follows_the_same_split() {
    for case in &CONTENT_CASES {
        let fixture: Fixture = Fixture::new();
        write(&fixture.path, CONFIG, only(case.rule).as_str());
        let plain: RunOutput = run(
            &fixture.path,
            &["--stdin", "--stdin-filename", "a.md", "--fix"],
            case.source,
        );
        assert_eq!(plain.stdout, case.source, "{}", case.rule);
        let content: RunOutput = run(
            &fixture.path,
            &["--stdin", "--stdin-filename", "a.md", "--fix-content"],
            case.source,
        );
        assert_eq!(content.stdout, case.fixed, "{}", case.rule);
    }
}

/// The line-break rule is the whitespace class:
///  plain `--fix` and `--fix-content` both apply it.
#[test]
fn both_flags_apply_the_whitespace_rule() {
    let source: &str = "First sentence. Second sentence.\n";
    let broken: &str = "First sentence.\nSecond sentence.\n";
    for flag in ["--fix", "--fix-content"] {
        let fixture: Fixture = Fixture::new();
        write(&fixture.path, CONFIG, only("semantic-line-breaks").as_str());
        write(&fixture.path, "a.md", source);
        let output: RunOutput = run(&fixture.path, &[flag], "");
        assert_eq!(read(&fixture.path, "a.md"), broken, "{flag}");
        assert_eq!(output.exit_code, 0, "{flag}");
    }
}

/// With every rule enabled,
///  plain `--fix` applies only the line breaks and `--fix-content` also applies the content fixes.
#[test]
fn with_every_rule_plain_fix_applies_only_line_breaks() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, crate::run_test_support::ALL_RULES);
    let source: &str = "# Title.\n\nFirst sentence. See https://x.example/a now.\n";
    write(&fixture.path, "a.md", source);
    let plain: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!(
        read(&fixture.path, "a.md"),
        "# Title.\n\nFirst sentence.\nSee https://x.example/a now.\n"
    );
    assert_eq!(
        codes(plain.stdout.as_str()),
        ["markdown/no-trailing-punctuation", "markdown/no-bare-urls"]
    );
    let content: RunOutput = run(&fixture.path, &["--fix-content"], "");
    assert_eq!(
        read(&fixture.path, "a.md"),
        "# Title\n\nFirst sentence.\nSee <https://x.example/a> now.\n"
    );
    assert_eq!((content.exit_code, content.stdout.as_str()), (0, ""));
}

/// The LFS rule rewrites an image URL,
///  so it is a content fix:
///  plain `--fix` reports it and `--fix-content` rewrites it.
#[test]
fn the_lfs_rewrite_needs_the_content_flag() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        ".lfsconfig",
        "[lfs]\n\turl = https://lfs:token@lfs.example\n",
    );
    write(
        &fixture.path,
        ".gitattributes",
        "*.png filter=lfs diff=lfs merge=lfs -text\n",
    );
    write(&fixture.path, "pkg/asset/shot.png", "image bytes");
    write(&fixture.path, CONFIG, only("lfs-image-url").as_str());
    let source: &str = "![shot](asset/shot.png)\n";
    let url: String = format!("https://lfs.example/{IMAGE_OID}/pkg/asset/shot.png");
    let plain: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "pkg/README.md", "--fix"],
        source,
    );
    assert_eq!(plain.stdout, source);
    let content: RunOutput = run(
        &fixture.path,
        &[
            "--stdin",
            "--stdin-filename",
            "pkg/README.md",
            "--fix-content",
        ],
        source,
    );
    assert_eq!(content.stdout, format!("![shot]({url})\n"));
}

/// `--rules` names each rule's fix class:
///  the line-break rule is `whitespace`,
///  every other fixable rule is `content`,
///  and a rule without a fix has none.
#[test]
fn the_rule_list_names_each_fix_class() {
    let fixture: Fixture = Fixture::new();
    let output: RunOutput = run(&fixture.path, &["--rules"], "");
    let mut whitespace: Vec<String> = Vec::<String>::new();
    for record in records(output.stdout.as_str()) {
        let id: String = String::from(record["id"].as_str().expect("rule id"));
        let fixable: bool = record["fixable"].as_bool().expect("fixable flag");
        let class: Option<&str> = record["fixClass"].as_str();
        assert_eq!(
            class.is_some(),
            fixable,
            "{id} has a class exactly when it has a fix"
        );
        if class == Some("whitespace") {
            whitespace.push(id);
        } else if fixable {
            assert_eq!(class, Some("content"), "{id}");
        }
    }
    assert_eq!(whitespace, ["markdown/semantic-line-breaks"]);
}
