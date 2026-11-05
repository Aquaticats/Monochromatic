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

use crate::{
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
