//! What:
//!  Whole-invocation controls for the classes of fix,
//!  named and grouped as Oxlint's are:
//!  a safe fix,
//!  a suggestion,
//!  and the dangerous forms of both,
//!  applied by `--fix`,
//! `--fix-suggestions` and `--fix-dangerously`.
//! Why:
//!  The owner settled on 2026-11-15 that this command follows Oxlint's `--fix`
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "Owner decisions in force"),
//!  and Oxlint's classes are the ones the command must offer:
//!  its `--help` names the three flags,
//!  and `crates/oxc_linter/src/fixer/fix.rs` at tag `oxlint_v1.93.0`
//!  defines `Fix`,
//! `Suggestion` and `Dangerous`
//!  with `can_apply` a set containment.
//!  Measured with Oxlint 1.93.0:
//!  `--fix` applies a `Fix` and withholds a suggestion and a dangerous fix,
//!  `--fix-suggestions` applies a suggestion alone,
//!  and `--fix-dangerously` applies all three.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('fix classes', () => { /* per rule: which flag applies it, which withholds it */ });
//! ```

/// Import the shared invocation fixtures.
use crate::markdown_source::MarkdownSource;
use crate::run_output::RunOutput;
use crate::run_test_support::{CONFIG, codes, read, records, run, write};
use crate::test_fs::Fixture;
/// Import the node kinds and payload decoders the link comparison reads.
use satteri_ast::mdast::{
    MdastNodeType, decode_definition_data, decode_link_data, decode_reference_data,
};

/// Measured with coreutils `sha256sum` for the bytes `image bytes`.
const IMAGE_OID: &str = "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec";

/// What:
///  One rule whose fix belongs to one class,
///  with a source it fires on and the text its own fix produces.
/// Why:
///  Every row pins the class the rule listing must print
///  and the flag that must apply it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClassCase = { rule: string; class: 'fix' | 'suggestion' | 'dangerous_fix'; source: string; fixed: string };
/// ```
struct ClassCase {
    /// The rule identifier without its `markdown/` prefix.
    rule: &'static str,
    /// The `fixClass` the rule listing prints for this rule.
    class: &'static str,
    /// The source the rule reports and fixes.
    source: &'static str,
    /// The source after the rule's own fix.
    fixed: &'static str,
}

/// Every Markdown rule with a fix,
///  its class and one firing source,
///  exercised with the current command.
const CLASSES: [ClassCase; 7] = [
    // A line break between two sentences renders the same paragraph.
    ClassCase {
        rule: "semantic-line-breaks",
        class: "fix",
        source: "First sentence. Second sentence.\n",
        fixed: "First sentence.\nSecond sentence.\n",
    },
    // `<url>` is an autolink with the same text and the same destination.
    ClassCase {
        rule: "no-bare-urls",
        class: "fix",
        source: "See https://x.example/a now.\n",
        fixed: "See <https://x.example/a> now.\n",
    },
    // `[label][]` is a collapsed reference that resolves to the same definition.
    ClassCase {
        rule: "link-image-style",
        class: "fix",
        source: "See [label] now.\n\n[label]: https://x.example/\n",
        fixed: "See [label][] now.\n\n[label]: https://x.example/\n",
    },
    // The reader's code listing loses the prompt it showed.
    ClassCase {
        rule: "commands-show-output",
        class: "suggestion",
        source: "```sh\n$ ls\n```\n",
        fixed: "```sh\nls\n```\n",
    },
    // The heading the reader sees loses its punctuation.
    ClassCase {
        rule: "no-trailing-punctuation",
        class: "suggestion",
        source: "# Title.\n",
        fixed: "# Title\n",
    },
    // The listing gains a language label a highlighter acts on.
    ClassCase {
        rule: "fenced-code-language",
        class: "suggestion",
        source: "```\ncode\n```\n",
        fixed: "```text\ncode\n```\n",
    },
    // The fix deletes the definition lines it judges unused or repeated.
    ClassCase {
        rule: "link-image-reference-definitions",
        class: "dangerous_fix",
        source: "Text.\n\n[unused]: https://x.example/\n",
        fixed: "Text.\n\n",
    },
];

/// A configuration that selects exactly one Markdown rule.
fn only(rule: &str) -> String {
    return format!(
        r#"[{{ "files": ["**/*.md", "**/*.mdx"], "rules": {{ "markdown/{rule}": {{ "severity": "error" }} }} }}]"#
    );
}

/// Fix one file of a disposable tree under one rule's configuration and report the result.
fn fix_one(case: &ClassCase, arguments: &[&str]) -> (String, Vec<String>, RunOutput) {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, only(case.rule).as_str());
    write(&fixture.path, "a.md", case.source);
    let output: RunOutput = run(&fixture.path, arguments, "");
    return (
        read(&fixture.path, "a.md"),
        codes(output.stdout.as_str()),
        output,
    );
}

/// What:
///  Every link,
/// image and definition a reader's browser resolves,
///  as one comparable line each.
/// Why:
///  A safe fix may rewrite the source of a link,
///  and must leave what the reader follows and reads exactly as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkFacts(filename, text): string[]
/// ```
fn link_facts(filename: &str, text: &str) -> Vec<String> {
    let mdx: bool = filename.ends_with(".mdx");
    let source: MarkdownSource =
        MarkdownSource::new(String::from(filename), String::from(text), mdx)
            .expect("the fixture parses");
    let mut facts: Vec<String> = Vec::<String>::new();
    for id in source.visible_nodes() {
        match source.kind(*id) {
            MdastNodeType::Link | MdastNodeType::Image => {
                let data = decode_link_data(source.data(*id));
                let content: String = source.text_content(*id).expect("an acyclic subtree");
                facts.push(format!(
                    "{} {content} -> {}",
                    if source.kind(*id) == MdastNodeType::Link {
                        "link"
                    } else {
                        "image"
                    },
                    source.text(data.url),
                ));
            }
            MdastNodeType::LinkReference | MdastNodeType::ImageReference => {
                let data = decode_reference_data(source.data(*id));
                let content: String = source.text_content(*id).expect("an acyclic subtree");
                facts.push(format!(
                    "reference {content} -> [{}]",
                    source.text(data.identifier),
                ));
            }
            MdastNodeType::Definition => {
                let data = decode_definition_data(source.data(*id));
                facts.push(format!(
                    "definition [{}] -> {}",
                    source.text(data.identifier),
                    source.text(data.url),
                ));
            }
            _ => {}
        }
    }
    return facts;
}

/// The command offers Oxlint's three fix flags,
///  and every rule with a fix names the class it belongs to.
#[test]
fn the_rule_list_names_every_fix_class() {
    let fixture: Fixture = Fixture::new();
    let output: RunOutput = run(&fixture.path, &["--rules"], "");
    let mut classes: Vec<String> = Vec::<String>::new();
    for record in records(output.stdout.as_str()) {
        let id: String = String::from(record["id"].as_str().expect("rule id"));
        let fixable: bool = record["fixable"].as_bool().expect("fixable flag");
        let class: Option<&str> = record["fixClass"].as_str();
        assert_eq!(
            class.is_some(),
            fixable,
            "{id} has a class exactly when it has a fix"
        );
        if let Some(name) = class {
            assert!(
                ["fix", "suggestion", "dangerous_fix", "dangerous_suggestion"].contains(&name),
                "{id} names a class Oxlint has: {name}"
            );
            classes.push(format!("{id} {name}"));
        }
    }
    let mut expected: Vec<String> = Vec::<String>::new();
    for case in &CLASSES {
        expected.push(format!("markdown/{} {}", case.rule, case.class));
    }
    expected.push(String::from("markdown/lfs-image-url suggestion"));
    expected.sort();
    classes.sort();
    assert_eq!(classes, expected);
}

/// Plain `--fix` applies the safe class
///  and withholds a suggestion and a dangerous fix,
///  whose findings stay reported.
#[test]
fn plain_fix_applies_the_safe_class_alone() {
    for case in &CLASSES {
        let (text, reported, output) = fix_one(case, &["--fix"]);
        if case.class == "fix" {
            assert_eq!(text, case.fixed, "{}", case.rule);
            assert_eq!((output.exit_code, reported.len()), (0, 0), "{}", case.rule);
        } else {
            assert_eq!(text, case.source, "{} is not safe", case.rule);
            assert_eq!(output.exit_code, 1, "{}", case.rule);
            assert_eq!(
                reported,
                [format!("markdown/{}", case.rule)],
                "{}",
                case.rule
            );
        }
    }
}

/// `--fix-suggestions` applies the suggestion class alone,
///  as Oxlint 1.93.0 does:
/// a safe fix waits for `--fix`.
#[test]
fn fix_suggestions_applies_the_suggestion_class_alone() {
    for case in &CLASSES {
        let (text, reported, output) = fix_one(case, &["--fix-suggestions"]);
        if case.class == "suggestion" {
            assert_eq!(text, case.fixed, "{}", case.rule);
            assert_eq!((output.exit_code, reported.len()), (0, 0), "{}", case.rule);
        } else {
            assert_eq!(text, case.source, "{} is not a suggestion", case.rule);
            assert_eq!(output.exit_code, 1, "{}", case.rule);
            assert_eq!(
                reported,
                [format!("markdown/{}", case.rule)],
                "{}",
                case.rule
            );
        }
    }
}

/// `--fix` with `--fix-suggestions` applies both classes and still withholds a dangerous fix.
#[test]
fn fix_and_suggestions_together_apply_both_classes() {
    for case in &CLASSES {
        let (text, reported, output) = fix_one(case, &["--fix", "--fix-suggestions"]);
        if case.class == "dangerous_fix" {
            assert_eq!(text, case.source, "{} is dangerous", case.rule);
            assert_eq!(output.exit_code, 1, "{}", case.rule);
            assert_eq!(
                reported,
                [format!("markdown/{}", case.rule)],
                "{}",
                case.rule
            );
        } else {
            assert_eq!(text, case.fixed, "{}", case.rule);
            assert_eq!((output.exit_code, reported.len()), (0, 0), "{}", case.rule);
        }
    }
}

/// `--fix-dangerously` alone applies every class,
///  as Oxlint 1.93.0 does:
/// it carries the fixes and the suggestions too.
#[test]
fn fix_dangerously_alone_applies_every_class() {
    for case in &CLASSES {
        let (text, reported, output) = fix_one(case, &["--fix-dangerously"]);
        assert_eq!(text, case.fixed, "{}", case.rule);
        assert_eq!((output.exit_code, reported.len()), (0, 0), "{}", case.rule);
    }
}

/// On standard input the same grouping holds.
#[test]
fn standard_input_follows_the_same_grouping() {
    for case in &CLASSES {
        let fixture: Fixture = Fixture::new();
        write(&fixture.path, CONFIG, only(case.rule).as_str());
        let plain: RunOutput = run(
            &fixture.path,
            &["--stdin", "--stdin-filename", "a.md", "--fix"],
            case.source,
        );
        let everything: RunOutput = run(
            &fixture.path,
            &["--stdin", "--stdin-filename", "a.md", "--fix-dangerously"],
            case.source,
        );
        assert_eq!(everything.stdout, case.fixed, "{}", case.rule);
        assert_eq!(
            plain.stdout,
            if case.class == "fix" {
                case.fixed
            } else {
                case.source
            },
            "{}",
            case.rule
        );
    }
}

/// The LFS rule rewrites an image destination,
///  so it is a suggestion:
///  plain `--fix` reports it and `--fix-suggestions` rewrites it.
#[test]
fn the_lfs_rewrite_is_a_suggestion() {
    let source: &str = "![shot](asset/shot.png)\n";
    let url: String = format!("https://lfs.example/{IMAGE_OID}/pkg/asset/shot.png");
    for (arguments, expected) in [
        (vec!["--fix"], String::from(source)),
        (vec!["--fix-suggestions"], format!("![shot]({url})\n")),
        (vec!["--fix-dangerously"], format!("![shot]({url})\n")),
    ] {
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
        let mut command: Vec<&str> = vec!["--stdin", "--stdin-filename", "pkg/README.md"];
        command.extend(arguments.iter().copied());
        let output: RunOutput = run(&fixture.path, command.as_slice(), source);
        assert_eq!(output.stdout, expected, "{arguments:?}");
    }
}

/// The safe class keeps every link text and destination a reader resolves,
///  in Markdown and in MDX,
///  while a later run changes nothing.
#[test]
fn the_safe_class_keeps_every_link_and_destination() {
    for case in &CLASSES {
        if case.class != "fix" {
            continue;
        }
        for (filename, source, fixed) in [
            ("a.md", case.source, case.fixed),
            // MDX spells a bare URL as an inline link, so the same rule has a second edit.
            (
                "a.mdx",
                if case.rule == "no-bare-urls" {
                    "See https://x.example/a now.\n"
                } else {
                    case.source
                },
                if case.rule == "no-bare-urls" {
                    "See [https://x.example/a](<https://x.example/a>) now.\n"
                } else {
                    case.fixed
                },
            ),
        ] {
            let facts_before: Vec<String> = link_facts(filename, source);
            let facts_after: Vec<String> = link_facts(filename, fixed);
            assert_eq!(facts_after, facts_before, "{} in {filename}", case.rule);
            let fixture: Fixture = Fixture::new();
            write(&fixture.path, CONFIG, only(case.rule).as_str());
            let output: RunOutput = run(
                &fixture.path,
                &["--stdin", "--stdin-filename", filename, "--fix"],
                source,
            );
            assert_eq!(output.stdout, fixed, "{} in {filename}", case.rule);
            assert_eq!(output.exit_code, 0, "{} in {filename}", case.rule);
        }
    }
}
