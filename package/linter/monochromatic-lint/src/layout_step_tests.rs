//! What:
//!  Controls of the layout step through whole runs.
//! Why:
//!  The rules are off unless configured,
//! switch on per package,
//! and report through the usual JSONL channel;
//!  these tests run the command itself over disposable packages.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the rules are off by default', () => expect(run(root, [])).toHaveNoFindings());
//! ```

use super::LineSpans;
use crate::{
    diagnostic::Span,
    layout_test_support::package_with,
    run_output::RunOutput,
    run_test_support::{ALL_RULES, CONFIG, records, run, write},
    test_fs::Fixture,
};

/// Configuration turning both layout rules on for every Rust file.
const LAYOUT_ON: &str = r#"[
  { "files": ["**/*.rs"], "rules": {
    "rust/folder-uses-links": { "severity": "error" },
    "rust/file-order-in-folder": { "severity": "warn" }
  } }
]"#;

/// A package with a folder that uses another and a mod.rs with no list.
fn crooked_package() -> Fixture {
    return package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", "mod one;\nuse crate::b::Thing;\n"),
        ("src/a/one.rs", ""),
        ("src/b/mod.rs", "pub struct Thing;\n"),
    ]);
}

/// The `(code, severity, file, line)` of every finding.
fn summary(output: &RunOutput) -> Vec<String> {
    return records(output.stdout.as_str())
        .iter()
        .map(|record| {
            return format!(
                "{} {} {} {}",
                record["code"].as_str().unwrap_or(""),
                record["severity"].as_str().unwrap_or(""),
                record["filename"].as_str().unwrap_or(""),
                record["labels"][0]["span"]["line"]
            );
        })
        .collect::<Vec<String>>();
}

/// Without a block that selects them,
/// the rules do nothing,
/// and the starter rule set does not select them either.
#[test]
fn the_rules_are_off_unless_configuration_selects_them() {
    let fixture = crooked_package();
    write(&fixture.path, CONFIG, ALL_RULES);
    let output: RunOutput = run(&fixture.path, &[], "");
    assert!(
        !output.stdout.contains("folder-uses-links"),
        "{}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("file-order-in-folder"),
        "{}",
        output.stdout
    );
}

/// Selected rules report in the usual channel with their configured severity and the proving line.
#[test]
fn selected_rules_report_with_their_severity_and_line() {
    let fixture = crooked_package();
    write(&fixture.path, CONFIG, LAYOUT_ON);
    let output: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(output.exit_code, 1, "{}", output.stderr);
    let found: Vec<String> = summary(&output);
    assert!(
        found.contains(&String::from("rust/folder-uses-links error src/a/mod.rs 2")),
        "{found:#?}"
    );
    assert!(
        found.contains(&String::from(
            "rust/file-order-in-folder warn src/a/mod.rs 1"
        )),
        "{found:#?}"
    );
}

/// The rules switch on per package:
/// a package whose files no block selects is not read.
#[test]
fn the_rules_switch_on_per_package() {
    let fixture = Fixture::new();
    for name in ["one", "two"] {
        write(
            &fixture.path,
            format!("{name}/Cargo.toml").as_str(),
            "[package]\nname = \"x\"\n",
        );
        write(
            &fixture.path,
            format!("{name}/src/lib.rs").as_str(),
            "mod a;\nmod b;\n",
        );
        write(
            &fixture.path,
            format!("{name}/src/a/mod.rs").as_str(),
            "use crate::b::Thing;\n",
        );
        write(&fixture.path, format!("{name}/src/b/mod.rs").as_str(), "");
    }
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["one/**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error" } } }]"#,
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    let found: Vec<String> = summary(&output);
    assert_eq!(
        found,
        ["rust/folder-uses-links error one/src/a/mod.rs 1"],
        "{}",
        output.stderr
    );
}

/// A module file that does not exist fails the run with a processing failure and exit status 2,
/// and the package is not checked.
#[test]
fn an_incomplete_tree_is_a_processing_failure() {
    let fixture = package_with(&[("src/lib.rs", "mod missing;\n")]);
    write(&fixture.path, CONFIG, LAYOUT_ON);
    let output: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(output.exit_code, 2, "{}", output.stderr);
    let found: Vec<String> = summary(&output);
    assert_eq!(found, ["core/processing-failure error src/lib.rs 1"]);
    assert!(
        output.stdout.contains("could not complete the module tree"),
        "{}",
        output.stdout
    );
}

/// Files below `uses` links are linted once,
/// not once per link,
/// and the layout rules agree with the links that are there.
#[cfg(unix)]
#[test]
fn files_behind_uses_links_are_linted_once() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod top;\nmod middle;\n"),
        ("src/top/mod.rs", "use crate::middle::X;\n"),
        ("src/middle/mod.rs", "#[test]\nfn stray() {}\n"),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/top/uses/middle", "../../middle");
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "error" }, "rust/folder-uses-links": { "severity": "error" } } }]"#,
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    let found: Vec<String> = summary(&output);
    assert_eq!(
        found,
        ["rust/tests-in-sidecars error src/middle/mod.rs 2"],
        "{}",
        output.stderr
    );
}

/// The span of a finding's line starts at that line,
///  covers it without its line break,
///  and falls back to the file start for line zero,
///  a line the file does not have and a file that cannot be read.
///  One file read answers every line it holds,
///  which is what keeps a list full of dead lines from reading the list once per finding.
#[test]
fn layout_the_span_of_a_line_starts_at_that_line() {
    let text: &str = "first\nsecond line\r\nthird";
    let fixture = package_with(&[("lines.txt", text)]);
    let file = fixture.path.join("lines.txt");
    let second_start: usize = text.find("second").expect("second line");
    let third_start: usize = text.find("third").expect("third line");
    let mut spans: LineSpans = LineSpans::default();
    assert_eq!(
        spans.span(&file, 1),
        Span {
            offset: 0,
            length: "first".len(),
            line: 1,
            column: 1
        }
    );
    assert_eq!(
        spans.span(&file, 2),
        Span {
            offset: second_start,
            length: "second line".len(),
            line: 2,
            column: 1
        }
    );
    assert_eq!(
        spans.span(&file, 3),
        Span {
            offset: third_start,
            length: "third".len(),
            line: 3,
            column: 1
        }
    );
    let at_start: Span = spans.span(&file, 0);
    assert_eq!(spans.span(&file, 4), at_start);
    assert_eq!(spans.span(&fixture.path.join("missing.txt"), 2), at_start);
}

/// A folder that sits in a plain loop of its own and in a loop of its entry is one state a list can satisfy:
/// listing every use leaves no loop finding and no entry that tolerates nothing.
/// The control arm runs the same package with an empty list and shows both loops.
#[cfg(unix)]
#[test]
fn a_list_that_names_every_use_of_a_folder_loop_and_an_entry_loop_passes() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\nmod grp;\n"),
        (
            "src/a/mod.rs",
            "use crate::b::X;\nuse crate::grp::g1::X as Y;\n",
        ),
        ("src/b/mod.rs", "use crate::a::X;\n"),
        ("src/grp/mod.rs", "mod g1;\nmod g2;\n"),
        ("src/grp/g1/mod.rs", ""),
        ("src/grp/g2/mod.rs", "use crate::a::X;\n"),
    ]);
    // Every folder links exactly the folders its code uses, so only the loops are left to the list.
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    crate::layout_test_support::link(&fixture.path, "src/a/uses/g1", "../../grp/g1");
    crate::layout_test_support::link(&fixture.path, "src/b/uses/a", "../../a");
    crate::layout_test_support::link(&fixture.path, "src/grp/g2/uses/a", "../../../a");
    crate::layout_test_support::link(&fixture.path, "src/grp/uses/a", "../../a");
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/folder-uses-links": { "severity": "error", "allow": "folder.txt" } } }]"#,
    );
    write(&fixture.path, "folder.txt", "");
    // The control: both loops exist, and the empty list reports them.
    let open: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(open.exit_code, 1, "{}", open.stderr);
    assert!(
        open.stdout
            .contains("These folders use each other in a loop"),
        "{}",
        open.stdout
    );
    assert!(
        open.stdout
            .contains("These groups and folders use each other in a loop"),
        "{}",
        open.stdout
    );
    // Every use of both loops on the list leaves nothing to report.
    write(
        &fixture.path,
        "folder.txt",
        "uses src/a -> src/b\nuses src/b -> src/a\nuses src/a -> src/grp/g1\nuses src/grp/g2 -> src/a\n",
    );
    let listed: RunOutput = run(&fixture.path, &[], "");
    assert!(
        !listed.stdout.contains("tolerates nothing any more"),
        "{}",
        listed.stdout
    );
    assert_eq!(listed.stdout, "", "{}", listed.stderr);
    assert_eq!(listed.exit_code, 0, "{}", listed.stderr);
}

/// A `no-file-list` entry tolerates the missing list alone,
///  never a file added to that folder afterwards.
#[test]
fn a_no_file_list_entry_does_not_hide_a_new_file() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/a/mod.rs", "//! Words only.\nmod one;\nmod two;\n"),
        ("src/a/one.rs", ""),
        ("src/a/two.rs", ""),
    ]);
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/file-order-in-folder": { "severity": "error", "allow": "file.txt" } } }]"#,
    );
    // The list records the missing list and the file that was there when the line was written.
    write(
        &fixture.path,
        "file.txt",
        "no-file-list src/a/mod.rs\nmissing-entry src/a/mod.rs one.rs\n",
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    assert!(
        output.stdout.contains("does not list the file `two.rs`"),
        "{}",
        output.stdout
    );
    assert_eq!(output.exit_code, 1, "{}", output.stderr);
    assert!(
        !output.stdout.contains("tolerates nothing any more"),
        "{}",
        output.stdout
    );
}

/// Configuration turning one layout rule on with a list beside the fixture's configuration file.
fn allow_config(rule: &str, list: &str) -> String {
    return format!(
        r#"[{{ "files": ["**/*.rs"], "rules": {{ "{rule}": {{ "severity": "error", "allow": "{list}" }} }} }}]"#
    );
}

/// One line tolerates one finding:
///  two glob re-exports of one module are two findings, and one line names the first alone.
///  A line that matched both would let a glob added later slip in unseen.
#[test]
fn one_line_tolerates_one_glob_finding() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "mod native;\npub use native::*;\npub use native::*;\n",
        ),
        ("src/native/mod.rs", "pub mod saving;\n"),
        ("src/native/saving.rs", ""),
    ]);
    write(
        &fixture.path,
        CONFIG,
        allow_config("rust/folder-uses-links", "folder.txt").as_str(),
    );
    write(
        &fixture.path,
        "folder.txt",
        "missing-link src -> src/native\nglob src/lib.rs native\n",
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    let found: Vec<String> = summary(&output);
    assert_eq!(found.len(), 1, "{}", output.stdout);
    // The second glob of the module, on its own line, is the one the list does not name.
    assert_eq!(found[0], "rust/folder-uses-links error src/lib.rs 3", "{}", output.stdout);
    assert!(
        output.stdout.contains("re-exports every item"),
        "{}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("tolerates nothing any more"),
        "{}",
        output.stdout
    );
}

/// An `impl` block and a `use` line each carry their own identity in their key,
///  so one line tolerates one of them and a second one of the same kind is reported.
#[test]
fn one_line_tolerates_one_index_item() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `a.rs`: a.\nmod a;\n\nimpl Holder {\n    /// one\n    pub fn one() {}\n}\n\nimpl Holder {\n    /// two\n    pub fn two() {}\n}\n\nuse std::fmt;\nuse std::io;\n",
        ),
        ("src/a/mod.rs", ""),
    ]);
    write(
        &fixture.path,
        CONFIG,
        allow_config("rust/file-order-in-folder", "file.txt").as_str(),
    );
    write(
        &fixture.path,
        "file.txt",
        "index-item src/lib.rs impl-Holder\nindex-item src/lib.rs private-use-std::fmt\nindex-item src/lib.rs private-use-std::io\n",
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    let found: Vec<String> = summary(&output);
    assert_eq!(found.len(), 1, "{}", output.stdout);
    // The second block impls the same type, so it is the one the list does not name.
    assert_eq!(found[0], "rust/file-order-in-folder error src/lib.rs 12", "{}", output.stdout);
    assert!(
        output.stdout.contains("it holds an `impl` block"),
        "{}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("tolerates nothing any more"),
        "{}",
        output.stdout
    );
}

/// A list that cannot be read ends the run as a processing failure naming it,
///  because a rule checked without its list would report every finding the list tolerates.
#[test]
fn a_list_that_cannot_be_read_is_a_processing_failure() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", "use crate::b::Thing;\n"),
        ("src/b/mod.rs", "pub struct Thing;\n"),
    ]);
    write(
        &fixture.path,
        CONFIG,
        allow_config("rust/folder-uses-links", "folder.txt").as_str(),
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(output.exit_code, 2, "{}", output.stdout);
    assert_eq!(
        summary(&output),
        ["core/processing-failure error folder.txt 1"],
        "{}",
        output.stdout
    );
    assert!(output.stdout.contains("allow list that cannot be read"), "{}", output.stdout);
}

/// A list that is a folder rather than a file is refused,
///  so a path that names a directory cannot stand in for a list.
#[test]
fn a_list_that_is_not_a_regular_file_is_a_processing_failure() {
    let fixture = package_with(&[("src/lib.rs", "")]);
    std::fs::create_dir(fixture.path.join("folder.txt")).expect("a folder where the list belongs");
    write(
        &fixture.path,
        CONFIG,
        allow_config("rust/folder-uses-links", "folder.txt").as_str(),
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(output.exit_code, 2, "{}", output.stdout);
    assert!(output.stdout.contains("not a regular file"), "{}", output.stdout);
}

/// A list over the cap is refused rather than read,
///  so a mistyped path cannot read a disk image into memory.
#[test]
fn a_list_over_the_cap_is_refused() {
    let fixture = package_with(&[("src/lib.rs", "")]);
    let over: String = format!(
        "# {}\n",
        "x".repeat(crate::layout_allow_list::MAX_BYTES as usize + 1)
    );
    write(&fixture.path, "folder.txt", over.as_str());
    write(
        &fixture.path,
        CONFIG,
        allow_config("rust/folder-uses-links", "folder.txt").as_str(),
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(output.exit_code, 2, "{}", output.stdout);
    assert!(output.stdout.contains("bytes this reader accepts"), "{}", output.stdout);
}

/// `--debug` prints the key of every finding a list line could hold,
///  so a reader copies a key instead of guessing its shape.
#[test]
fn debug_prints_the_key_of_a_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", "use crate::b::Thing;\n"),
        ("src/b/mod.rs", "pub struct Thing;\n"),
    ]);
    write(&fixture.path, CONFIG, LAYOUT_ON);
    let output: RunOutput = run(&fixture.path, &["--debug"], "");
    assert!(
        output
            .stderr
            .contains("rust/folder-uses-links: allow-list key: missing-link src/a -> src/b"),
        "{}",
        output.stderr
    );
}

/// The `allow` path is relative to the configuration file's own directory,
///  not to the directory the run started in.
#[test]
fn the_allow_path_is_relative_to_the_configuration() {
    let fixture = package_with(&[
        ("nested/pkg/Cargo.toml", "[package]\nname = \"nested\"\nversion = \"0.0.0\"\n"),
        ("nested/pkg/src/lib.rs", "mod a;\nmod b;\n"),
        ("nested/pkg/src/a/mod.rs", "use crate::b::Thing;\n"),
        ("nested/pkg/src/b/mod.rs", "pub struct Thing;\n"),
    ]);
    write(
        &fixture.path,
        "nested/monochromatic-lint.config.jsonc",
        allow_config("rust/folder-uses-links", "lists/folder.txt").as_str(),
    );
    write(
        &fixture.path,
        "nested/lists/folder.txt",
        "missing-link src/a -> src/b\n",
    );
    let output: RunOutput = run(&fixture.path, &[], "");
    assert!(
        !output.stdout.contains("has no link to it"),
        "{}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("tolerates nothing any more"),
        "{}",
        output.stdout
    );
}
