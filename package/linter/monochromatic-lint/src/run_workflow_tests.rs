//! What:
//!  End-to-end controls for the shell rules through discovery,
//!  the workflow processor,
//!  configuration and the command.
//! Why:
//!  The pieces are unit-tested apart;
//!  these controls show that a real `.github` tree is found,
//!  its steps are selected by the virtual file name,
//!  findings land on the workflow's own lines,
//!  and the ratchet blocks behave as the design says.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('workflow rules end to end', () => { /* discovery, findings, ratchet, refusals */ });
//! ```

/// Import the command fixtures and the disposable directory.
use crate::run_output::RunOutput;
use crate::run_test_support::{CONFIG, codes, records, run, write};
use crate::test_fs::Fixture;

/// Every `shell/` rule at error over every dialect of a workflow or composite action,
///  and a ratchet block for one old workflow.
const RULES: &str = r#"[
  { "files": [
      ".github/workflows/*.yml/*.sh", ".github/workflows/*.yml/*.ps1",
      ".github/workflows/*.yml/*.cmd", ".github/workflows/*.yml/*.fish",
      ".github/actions/**/action.yml/*.sh"
    ], "rules": {
    "shell/max-lines": { "severity": "error" },
    "shell/no-pipe": { "severity": "error" },
    "shell/no-and-chain": { "severity": "error" },
    "shell/no-or-chain": { "severity": "error" },
    "shell/no-pipe-stderr": { "severity": "error" },
    "shell/no-semicolon": { "severity": "error" },
    "shell/no-dollar-paren": { "severity": "error" },
    "shell/no-backtick": { "severity": "error" },
    "shell/no-ampersand": { "severity": "error" }
  } },
  { "name": "ratchet", "files": [".github/workflows/old.yml/*.sh"], "rules": {
    "shell/max-lines": { "severity": "error", "max": 4 },
    "shell/no-pipe": { "severity": "error", "max": 1 }
  } }
]"#;

/// A workflow with one clean step.
const CLEAN: &str = "name: ci\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - run: mise run lint\n      - uses: actions/checkout@v4\n";

/// Lint a tree and return the output.
fn lint(root: &std::path::Path) -> RunOutput {
    return run(root, &["."], "");
}

/// A clean workflow passes with no output.
#[test]
fn a_clean_workflow_passes() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    write(&fixture.path, ".github/workflows/ci.yml", CLEAN);
    let output: RunOutput = lint(&fixture.path);
    assert_eq!(output.stdout, "");
    assert_eq!(output.exit_code, 0);
}

/// A long block with pipes in a hidden directory is found and reported on the workflow's lines.
#[test]
fn findings_land_on_the_workflow_lines() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let workflow: &str = "jobs:\n  a:\n    steps:\n      - run: |\n          one\n          two | x\n          three && y\n";
    write(&fixture.path, ".github/workflows/ci.yml", workflow);
    let output: RunOutput = lint(&fixture.path);
    assert_eq!(output.exit_code, 1, "{}", output.stdout);
    let found: Vec<serde_json::Value> = records(&output.stdout);
    let mut located: Vec<String> = Vec::new();
    for record in &found {
        assert_eq!(record["filename"], ".github/workflows/ci.yml");
        located.push(format!(
            "{}@{}:{}",
            record["code"].as_str().expect("code"),
            record["labels"][0]["span"]["line"],
            record["labels"][0]["span"]["column"]
        ));
    }
    located.sort();
    assert_eq!(
        located,
        [
            "shell/max-lines@7:11",
            "shell/no-and-chain@7:17",
            "shell/no-pipe@6:15"
        ]
    );
}

/// A ratchet block lets a file keep its current worst step and no worse.
#[test]
fn the_ratchet_holds_a_file_at_its_current_worst_step() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let held: &str = "jobs:\n  a:\n    steps:\n      - run: |\n          1\n          2 | x\n          3\n          4\n";
    write(&fixture.path, ".github/workflows/old.yml", held);
    assert_eq!(lint(&fixture.path).exit_code, 0);
    let worse: &str = "jobs:\n  a:\n    steps:\n      - run: |\n          1\n          2 | x\n          3 | y\n          4\n          5\n";
    write(&fixture.path, ".github/workflows/old.yml", worse);
    let output: RunOutput = lint(&fixture.path);
    assert_eq!(output.exit_code, 1);
    let mut found: Vec<String> = codes(&output.stdout);
    found.sort();
    assert_eq!(found, ["shell/max-lines", "shell/no-pipe"]);
}

/// A step under a non-shell interpreter is not checked.
#[test]
fn node_steps_are_not_checked() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let workflow: &str = "jobs:\n  a:\n    steps:\n      - run: |\n          const a = 1 | 2;\n          console.log(a); console.log(a);\n          console.log(a);\n        shell: node {0}\n";
    write(&fixture.path, ".github/workflows/ci.yml", workflow);
    assert_eq!(lint(&fixture.path).exit_code, 0);
}

/// A form the walk cannot read fails the run with a processing failure,
///  never a pass.
#[test]
fn unreadable_forms_fail_the_run() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let workflow: &str = "jobs:\n  a:\n    steps:\n      - run: \"echo a | b\"\n";
    write(&fixture.path, ".github/workflows/ci.yml", workflow);
    let output: RunOutput = lint(&fixture.path);
    assert_eq!(output.exit_code, 2, "{}", output.stdout);
    assert_eq!(codes(&output.stdout), ["core/processing-failure"]);
    let found: Vec<serde_json::Value> = records(&output.stdout);
    assert_eq!(found[0]["labels"][0]["span"]["line"], 4);
}

/// YAML outside `.github/workflows` and `.github/actions` is never read.
#[test]
fn other_yaml_is_never_read() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    write(&fixture.path, ".github/workflows/ci.yml", CLEAN);
    write(
        &fixture.path,
        "config/other.yml",
        "jobs:\n  a:\n    steps:\n      - run: \"a | b\"\n",
    );
    write(
        &fixture.path,
        ".github/dependabot.yml",
        "version: 2\nrun: \"a | b\"\n",
    );
    write(
        &fixture.path,
        ".github/workflows/nested/deep.yml",
        "jobs:\n  a:\n    steps:\n      - run: \"a | b\"\n",
    );
    let output: RunOutput = lint(&fixture.path);
    assert_eq!(output.stdout, "");
    assert_eq!(output.exit_code, 0);
}

/// A composite action's steps are checked,
///  and an inline directive comment suppresses nothing.
#[test]
fn composite_actions_are_checked_and_directives_do_nothing() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let action: &str = "runs:\n  using: composite\n  steps:\n    # monochromatic-lint-disable-next-line shell/no-pipe -- not allowed\n    - run: a | b\n      shell: bash\n";
    write(&fixture.path, ".github/actions/setup/action.yml", action);
    let output: RunOutput = lint(&fixture.path);
    assert_eq!(output.exit_code, 1, "{}", output.stdout);
    assert_eq!(codes(&output.stdout), ["shell/no-pipe"]);
}

/// A comment inside a shell body is script text,
///  not a directive.
#[test]
fn a_directive_inside_a_script_suppresses_nothing() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let workflow: &str = "jobs:\n  a:\n    steps:\n      - run: |\n          # monochromatic-lint-disable-next-line shell/no-pipe -- not allowed\n          a | b\n";
    write(&fixture.path, ".github/workflows/ci.yml", workflow);
    let output: RunOutput = lint(&fixture.path);
    assert_eq!(codes(&output.stdout), ["shell/no-pipe"]);
}

/// The rule listing names all nine rules as unsuppressible and takes `max`.
#[test]
fn the_listing_declares_the_rules_unsuppressible() {
    let fixture: Fixture = Fixture::new();
    let output: RunOutput = run(&fixture.path, &["--rules"], "");
    let mut shell: Vec<serde_json::Value> = Vec::new();
    for record in records(&output.stdout) {
        let id: &str = record["id"].as_str().unwrap_or("");
        if id.starts_with("shell/") {
            shell.push(record);
        }
    }
    assert_eq!(shell.len(), 9);
    for record in &shell {
        assert_eq!(record["suppressible"], false, "{record}");
        let options = record["options"].as_array().expect("options list");
        assert_eq!(options.len(), 1, "{record}");
        assert_eq!(options[0], "max", "{record}");
    }
}

/// An unknown `shell/` rule in configuration is a setup error.
#[test]
fn a_misspelled_rule_is_a_setup_error() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.yml/*.sh"], "rules": { "shell/no-pipes": { "severity": "error" } } }]"#,
    );
    write(&fixture.path, ".github/workflows/ci.yml", CLEAN);
    let output: RunOutput = lint(&fixture.path);
    assert_ne!(output.exit_code, 0);
}
