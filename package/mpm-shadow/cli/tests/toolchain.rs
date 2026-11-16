//! End-to-end tests of the toolchain verbs of the built `mpm-shadow` binary,
//! run against a disposable worktree and a
//! disposable home.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.

use std::fs;

use mpm_shadow::cli::{build_command, declared_verbs};
use mpm_shadow::constants::USAGE_EXIT_CODE;
use mpm_shadow::toolchain::cargo_installs::{current_link, version_directory};
use mpm_shadow::toolchain::lock::{parse_lock, render_lock};
use mpm_shadow::toolchain::scope::{EnvMap, EXPOSURE_DIR_RELATIVE, LOCK_FILE_NAME, PLAN_FILE_NAME, Scope, repo_scope};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The plan the tests write:
/// two tools,
/// one with a pinned version so no network is needed.
const PLAN: &str = r#"{
  // Two tools: a pinned archive and a binstall crate.
  "schemaVersion": 1,
  "tools": {
    "bin": { "request": "github:marcosnils/bin@0.29.4", "route": "archive", "bins": ["bin"] },
    "cargo:fd-find": { "request": "cargo:fd-find@latest", "route": "cargo-binstall", "packageId": "fd-find", "bins": ["fd"] },
  },
}
"#;

/// A lock without a digest,
/// as the TypeScript tool wrote it.
const OLD_LOCK: &str = r#"{
  "schemaVersion": 1,
  "generatedFrom": "package/file-enforcer/core/src/toolchain/plan.ts",
  "tools": {
    "bin": {
      "request": "github:marcosnils/bin@0.29.4", "version": "0.29.4", "route": "archive", "bins": ["bin"], "env": {}, "options": {},
      "artifacts": { "linux-x64": { "url": "https://example.invalid/bin.tar.gz", "sha256": "439e9c438acdf6a6ffe47418da709153d66c4bed812e30a473faacb107dd2791" } }
    },
    "cargo:fd-find": {
      "request": "cargo:fd-find@latest", "version": "10.5.0", "route": "cargo-binstall", "packageId": "fd-find", "bins": ["fd"], "env": {}, "options": {}, "artifacts": {}
    }
  }
}
"#;

struct Fixture {
    directory: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        fs::create_dir_all(directory.path().join("home")).expect("make the home");
        fs::create_dir_all(directory.path().join("work").join("sub")).expect("make the worktree");
        fs::write(directory.path().join("work").join(PLAN_FILE_NAME), PLAN).expect("write the plan");
        fs::write(directory.path().join("work").join(LOCK_FILE_NAME), OLD_LOCK).expect("write the lock");
        return Fixture { directory };
    }

    /// Rewrites the lock in `directory` with its digest,
    /// the way `update` would have written it.
    fn sign_lock(&self, directory: &Path) {
        let path = directory.join(LOCK_FILE_NAME);
        let parsed = parse_lock(&fs::read_to_string(&path).expect("read the lock"), "lock").expect("the fixture lock parses");
        fs::write(&path, render_lock(&parsed.lock)).expect("write the signed lock");
    }

    fn work(&self) -> PathBuf {
        return self.directory.path().join("work");
    }

    fn run(&self, arguments: &[&str]) -> Output {
        return self.run_in(&self.work().join("sub"), arguments);
    }

    fn run_in(&self, directory: &Path, arguments: &[&str]) -> Output {
        return Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
            .args(arguments)
            .current_dir(directory)
            .env_clear()
            .env("HOME", self.directory.path().join("home"))
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run mpm-shadow");
    }
}

fn text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).to_string();
}

#[test]
fn lock_check_refuses_a_lock_without_a_digest_and_passes_once_it_is_signed() {
    let fixture = Fixture::new();
    let refused = fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused.stderr));
    assert!(text(&refused.stderr).contains("no digest"), "{}", text(&refused.stderr));

    fixture.sign_lock(&fixture.work());
    let checked = fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(checked.status.code(), Some(0), "{}", text(&checked.stderr));
    assert!(text(&checked.stdout).contains("digest verified"), "{}", text(&checked.stdout));
}

#[test]
fn lock_after_a_plan_edit_keeps_the_resolved_versions_and_passes_the_gate() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let edited = PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]");
    fs::write(fixture.work().join(PLAN_FILE_NAME), edited).expect("edit the plan");
    assert_eq!(fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]).status.code(), Some(1));
    let rewritten = fixture.run(&["install", "--lockfile-only", "--offline"]);
    assert_eq!(rewritten.status.code(), Some(0), "{}", text(&rewritten.stderr));
    let lock: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read")).expect("JSON");
    assert_eq!(lock["tools"]["cargo:fd-find"]["version"], "10.5.0");
    assert_eq!(lock["tools"]["cargo:fd-find"]["bins"][1], "fdfind");
    assert_eq!(fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]).status.code(), Some(0));
}

#[test]
fn a_broken_plan_exits_with_code_two_naming_the_file() {
    let fixture = Fixture::new();
    fs::write(fixture.work().join(PLAN_FILE_NAME), "{ \"tools\": ").expect("break the plan");
    let output = fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE));
    assert!(text(&output.stderr).contains(PLAN_FILE_NAME), "{}", text(&output.stderr));
}

#[test]
fn outside_a_worktree_the_command_names_what_to_do() {
    let fixture = Fixture::new();
    let lonely = fixture.directory.path().join("lonely");
    fs::create_dir_all(&lonely).expect("make the directory");
    let output = fixture.run_in(&lonely, &["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE));
    assert!(text(&output.stderr).contains("MONOCHROMATIC_WORKTREE_ROOT"), "{}", text(&output.stderr));
}

#[test]
fn global_reads_the_per_user_config_directory() {
    let fixture = Fixture::new();
    let output = fixture.run(&["install", "--lockfile-only", "--frozen-lockfile", "--global"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE));
    let expected_plan = format!(".config/monochromatic-dev/toolchain/{PLAN_FILE_NAME}");
    assert!(
        text(&output.stderr).contains(&expected_plan),
        "{}",
        text(&output.stderr)
    );
    let config = fixture.directory.path().join("home").join(".config").join("monochromatic-dev").join("toolchain");
    fs::create_dir_all(&config).expect("make the config directory");
    fs::write(config.join(PLAN_FILE_NAME), PLAN).expect("write the global plan");
    fs::write(config.join(LOCK_FILE_NAME), OLD_LOCK).expect("write the global lock");
    fixture.sign_lock(&config);
    assert_eq!(fixture.run(&["install", "--lockfile-only", "--frozen-lockfile", "--global"]).status.code(), Some(0));
}

#[test]
fn bin_json_prints_the_pinned_shape_for_the_ide_launcher() {
    let fixture = Fixture::new();
    let exposure = fixture.work().join(EXPOSURE_DIR_RELATIVE);
    fs::create_dir_all(&exposure).expect("make the exposure directory");
    fs::write(exposure.join("fd"), "").expect("write a link stand-in");
    let output = fixture.run(&["bin", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let document: serde_json::Value = serde_json::from_str(&text(&output.stdout)).expect("JSON on standard output");
    assert_eq!(document["exposureDir"], EXPOSURE_DIR_RELATIVE);
    assert_eq!(document["bins"]["fd"], exposure.join("fd").to_str().expect("utf-8"));
    assert!(document["bins"].get("bin").is_none(), "a binary without a link is not listed");
    assert_eq!(document["pathDirs"][0], exposure.to_str().expect("utf-8"));
}

#[test]
fn bin_needs_no_lock() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.work().join(LOCK_FILE_NAME)).expect("remove the lock");
    let output = fixture.run(&["bin", "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
}

/// The rendered help line of the subcommand `name`:
/// the first line whose first word is the command name.
fn help_line<'help>(help: &'help str, name: &str) -> &'help str {
    return help
        .lines()
        .find(|line| return line.split_whitespace().next() == Some(name))
        .unwrap_or_else(|| {
            panic!("{name} is missing from --help:\n{help}");
        });
}

/// The declared verb surface must be exactly the surface the grammar registers:
/// a subcommand added without a declaration would escape the rendered-help assertion.
fn assert_grammar_matches_declaration() {
    let grammar = build_command();
    let registered: Vec<&str> = grammar
        .get_subcommands()
        .map(|command| return command.get_name())
        .collect();
    let declared: Vec<&str> = declared_verbs().iter().map(|verb| return verb.name).collect();
    assert_eq!(
        registered, declared,
        "the grammar registers a subcommand the declaration does not name"
    );
    for verb in declared_verbs() {
        let command = grammar
            .find_subcommand(verb.name)
            .unwrap_or_else(|| {
                panic!("{} is declared but not registered", verb.name);
            });
        let aliases: Vec<&str> = command.get_visible_aliases().collect();
        assert_eq!(
            aliases, verb.aliases,
            "{}: the grammar registers other aliases than the declaration",
            verb.name
        );
    }
}

/// The alias bracket `clap` renders beside a subcommand:
/// `[alias: i]` for one alias and `[aliases: i, ls]` for several.
fn alias_bracket(aliases: &[&str]) -> Option<String> {
    return match aliases {
        [] => None,
        [only] => Some(format!("[alias: {only}]")),
        many => Some(format!("[aliases: {}]", many.join(", "))),
    };
}

/// The alias bracket a rendered help line carries,
/// if any.
fn rendered_alias_bracket(line: &str) -> Option<&str> {
    let start: usize = line.find("[alias")?;
    let rest: &str = &line[start..];
    let end: usize = rest.find(']')?;
    return Some(&rest[..=end]);
}

/// What:
/// `update` takes pnpm's own aliases,
/// `up` and `upgrade`.
/// Why:
/// pnpm's page lists them ("Aliases:
/// up,
/// upgrade",
/// <https://pnpm.io/cli/update>,
/// read 2026-11-03),
/// and the commands follow pnpm's,
///           aliases included,
/// so `mpm up` and `mpm upgrade` are the toolchain's update.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(declaredVerbs().find((verb) => verb.name === 'update')?.aliases).toEqual(['up', 'upgrade']);
/// ```
#[test]
fn update_takes_pnpms_aliases_up_and_upgrade() {
    let update = declared_verbs()
        .into_iter()
        .find(|verb| return verb.name == "update")
        .expect("an update verb");
    assert_eq!(update.aliases, ["up", "upgrade"]);
}

#[test]
fn help_lists_every_declared_verb_and_its_visible_aliases() {
    assert_grammar_matches_declaration();
    let fixture = Fixture::new();
    let output = fixture.run(&["--help"]);
    let help = text(&output.stdout);
    for verb in declared_verbs() {
        let line: &str = help_line(&help, verb.name);
        let rendered: Option<&str> = rendered_alias_bracket(line);
        assert_eq!(
            alias_bracket(verb.aliases).as_deref(),
            rendered,
            "{}: --help renders {rendered:?} but the declaration has {:?}:\n{line}",
            verb.name,
            verb.aliases
        );
    }
}

#[test]
fn install_and_list_refuse_when_the_plan_and_lock_disagree_and_name_the_remedy() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let edited = PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]");
    fs::write(fixture.work().join(PLAN_FILE_NAME), edited).expect("edit the plan");
    for arguments in [&["install", "--frozen-lockfile"][..], &["list"][..]] {
        let verb = arguments[0];
        let output = fixture.run(arguments);
        assert_eq!(output.status.code(), Some(1), "{verb}: {}", text(&output.stderr));
        assert!(text(&output.stderr).contains("mpm-shadow install --lockfile-only --offline"), "{verb}: {}", text(&output.stderr));
        assert!(text(&output.stdout).is_empty(), "{verb} printed nothing before refusing: {}", text(&output.stdout));
    }
    assert!(!fixture.directory.path().join("home").join(".local").exists(), "nothing was installed under the home");
}

#[test]
fn update_refuses_a_hand_edited_lock_before_any_network_use() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let path = fixture.work().join(LOCK_FILE_NAME);
    let text_before = fs::read_to_string(&path).expect("read");
    fs::write(&path, text_before.replace("\"version\": \"10.5.0\"", "\"version\": \"1.0.0\"")).expect("hand edit");
    let output = fixture.run(&["outdated"]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert!(text(&output.stderr).contains("edited by hand"), "{}", text(&output.stderr));
}

#[test]
fn verify_archive_accepts_the_recorded_hash_and_refuses_anything_else() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let download = fixture.directory.path().join("bin.tar.gz");
    fs::write(&download, "not the recorded bytes").expect("write the download");
    let refused = fixture.run(&["verify-archive", "bin", download.to_str().expect("utf-8"), "--platform", "linux-x64"]);
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused.stderr));
    assert!(text(&refused.stderr).contains("SHA-256 mismatch"), "{}", text(&refused.stderr));
    assert!(text(&refused.stderr).contains("439e9c438acdf6a6ffe47418da709153d66c4bed812e30a473faacb107dd2791"), "{}", text(&refused.stderr));
    let unknown = fixture.run(&["verify-archive", "bin", download.to_str().expect("utf-8"), "--platform", "plan9-mips"]);
    assert_eq!(unknown.status.code(), Some(USAGE_EXIT_CODE));
}

#[test]
fn the_pre_pnpm_command_names_are_gone() {
    let fixture = Fixture::new();
    for verb in ["sync", "lock", "status", "bin-paths"] {
        let output = fixture.run(&[verb]);
        assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE), "{verb} is no longer a command: {}", text(&output.stderr));
    }
    let output = fixture.run(&["update", "--check"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE), "update has no --check; outdated is the report: {}", text(&output.stderr));
}

#[test]
fn offline_needs_lockfile_only_and_the_aliases_are_pnpms() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let output = fixture.run(&["install", "--offline"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE), "{}", text(&output.stderr));
    assert!(text(&output.stderr).contains("--lockfile-only"), "{}", text(&output.stderr));
    let aliased = fixture.run(&["i", "--lockfile-only", "--offline"]);
    assert_eq!(aliased.status.code(), Some(0), "i is pnpm's alias of install: {}", text(&aliased.stderr));
    let listed = fixture.run(&["ls"]);
    assert_ne!(listed.status.code(), Some(USAGE_EXIT_CODE), "ls is pnpm's alias of list: {}", text(&listed.stderr));
}

#[test]
fn bin_prints_the_exposure_directory_like_pnpm_bin_and_json_prints_the_document() {
    let fixture = Fixture::new();
    let plain = fixture.run(&["bin"]);
    assert_eq!(plain.status.code(), Some(0), "{}", text(&plain.stderr));
    let expected = fixture.work().join(EXPOSURE_DIR_RELATIVE);
    assert_eq!(text(&plain.stdout), format!("{}\n", expected.display()));
    let json = fixture.run(&["bin", "--json"]);
    serde_json::from_str::<serde_json::Value>(&text(&json.stdout)).expect("JSON on standard output");
}

#[test]
fn a_frozen_lock_check_and_a_plain_offline_rewrite_are_the_two_lockfile_only_modes() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let edited = PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]");
    fs::write(fixture.work().join(PLAN_FILE_NAME), edited).expect("edit the plan");
    let before = fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read");
    let checked = fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(checked.status.code(), Some(1), "{}", text(&checked.stderr));
    assert_eq!(fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read"), before, "a check writes nothing");
    let rewritten = fixture.run(&["install", "--lockfile-only", "--offline"]);
    assert_eq!(rewritten.status.code(), Some(0), "{}", text(&rewritten.stderr));
    assert_ne!(fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read"), before);
    assert_eq!(fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]).status.code(), Some(0));
}

#[test]
fn a_non_frozen_install_lockfile_only_writes_an_offline_fixable_edit_into_the_lock() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let edited = PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]");
    fs::write(fixture.work().join(PLAN_FILE_NAME), edited).expect("edit the plan");
    let output = fixture.run(&["install", "--lockfile-only"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert!(text(&output.stdout).contains("lock updated from the plan: cargo:fd-find now differs from the plan in: bins"), "{}", text(&output.stdout));
    assert_eq!(fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]).status.code(), Some(0), "the lock now agrees");
}

#[test]
fn help_says_in_its_first_line_that_it_is_a_shadow_of_mpm_and_lists_the_pnpm_named_commands() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--help"]);
    let help = text(&output.stdout);
    let first = help.lines().next().expect("a first line");
    assert!(first.starts_with("A shadow of mpm"), "{first}");
    assert!(first.contains("invoked as `mpm`"), "{first}");
    for verb in ["install", "update", "outdated", "list", "bin"] {
        assert!(first.contains(verb), "{verb} is missing from the first line: {first}");
    }
    let version = fixture.run(&["--version"]);
    assert_eq!(text(&version.stdout), format!("mpm-shadow {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn the_toolchain_prune_says_it_covers_release_archives_and_words_an_empty_result_for_both() {
    let fixture = Fixture::new();
    let help = fixture.run(&["prune", "--help"]);
    assert_eq!(help.status.code(), Some(0), "{}", text(&help.stderr));
    assert!(text(&help.stdout).contains("release-archive versions"), "{}", text(&help.stdout));

    // A dry run in a disposable home with nothing installed: nothing to list, nothing deleted.
    let empty = fixture.run(&["prune", "--toolchain", "--global", "--dry-run"]);
    assert_eq!(empty.status.code(), Some(0), "{}", text(&empty.stderr));
    let stdout = text(&empty.stdout);
    assert!(stdout.starts_with("nothing to delete under "), "{stdout}");
    assert!(stdout.contains("release archive"), "the empty result names the release archives as well: {stdout}");
    assert!(!stdout.contains("no unnamed cargo version"), "{stdout}");
}

/// The machine-global plan and lock under a fixture's disposable home.
fn global_config(fixture: &Fixture) -> PathBuf {
    return fixture.directory.path().join("home").join(".config").join("monochromatic-dev").join("toolchain");
}

#[test]
fn install_help_shows_the_tool_argument_and_the_prefixes_it_can_add() {
    let fixture = Fixture::new();
    let output = fixture.run(&["install", "--help"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let help = text(&output.stdout);
    assert!(help.contains("[TOOL]..."), "the usage line shows the argument:\n{help}");
    for prefix in ["cargo:<crate>", "npm:<package>", "go:<module>"] {
        assert!(help.contains(prefix), "{prefix} is missing from the argument's help:\n{help}");
    }
    assert!(help.contains("the newest when none is given"), "{help}");
}

#[test]
fn install_adds_nothing_for_a_bare_name_that_is_neither_an_id_nor_a_request() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let plan_before = fs::read_to_string(fixture.work().join(PLAN_FILE_NAME)).expect("read the plan");
    let lock_before = fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read the lock");
    let output = fixture.run(&["install", "fastmod"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE), "{}", text(&output.stderr));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("\"fastmod\""), "{stderr}");
    assert!(stderr.contains(PLAN_FILE_NAME), "the plan is named: {stderr}");
    assert!(stderr.contains("README.md"), "the entry shapes are named: {stderr}");
    assert_eq!(fs::read_to_string(fixture.work().join(PLAN_FILE_NAME)).expect("read the plan"), plan_before);
    assert_eq!(fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read the lock"), lock_before);
    assert!(!fixture.directory.path().join("home").join(".local").exists(), "nothing was installed under the home");
}

#[test]
fn install_refuses_a_route_it_cannot_add_and_names_the_readme() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let plan_before = fs::read_to_string(fixture.work().join(PLAN_FILE_NAME)).expect("read the plan");
    let output = fixture.run(&["install", "github:FiloSottile/age"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE), "{}", text(&output.stderr));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("github:"), "{stderr}");
    assert!(stderr.contains("README.md"), "{stderr}");
    assert_eq!(fs::read_to_string(fixture.work().join(PLAN_FILE_NAME)).expect("read the plan"), plan_before);
}

#[test]
fn a_frozen_lockfile_check_accepts_an_existing_id_as_an_argument_and_writes_nothing() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let plan_before = fs::read_to_string(fixture.work().join(PLAN_FILE_NAME)).expect("read the plan");
    let lock_before = fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read the lock");
    let output = fixture.run(&["i", "--lockfile-only", "--frozen-lockfile", "cargo:fd-find"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(fs::read_to_string(fixture.work().join(PLAN_FILE_NAME)).expect("read the plan"), plan_before);
    assert_eq!(fs::read_to_string(fixture.work().join(LOCK_FILE_NAME)).expect("read the lock"), lock_before);
}

#[test]
fn a_global_install_refuses_to_add_a_tool_offline_and_leaves_the_machine_files_as_they_were() {
    let fixture = Fixture::new();
    let config = global_config(&fixture);
    fs::create_dir_all(&config).expect("make the config directory");
    fs::write(config.join(PLAN_FILE_NAME), PLAN).expect("write the global plan");
    fs::write(config.join(LOCK_FILE_NAME), OLD_LOCK).expect("write the global lock");
    fixture.sign_lock(&config);
    let plan_before = fs::read_to_string(config.join(PLAN_FILE_NAME)).expect("read the global plan");
    let lock_before = fs::read_to_string(config.join(LOCK_FILE_NAME)).expect("read the global lock");
    let output = fixture.run(&["i", "-g", "--lockfile-only", "--offline", "cargo:fastmod"]);
    assert_eq!(output.status.code(), Some(USAGE_EXIT_CODE), "{}", text(&output.stderr));
    let stderr = text(&output.stderr);
    assert!(stderr.contains("cargo:fastmod"), "{stderr}");
    assert!(stderr.contains("--offline"), "the switch that refused is named: {stderr}");
    assert_eq!(fs::read_to_string(config.join(PLAN_FILE_NAME)).expect("read the global plan"), plan_before);
    assert_eq!(fs::read_to_string(config.join(LOCK_FILE_NAME)).expect("read the global lock"), lock_before);
}

//region An install with nothing to do opens the launcher folder once, however many bins the plan declares

// What:
// The bound this control checks is the plan's own: a plan declares one launcher folder for its one scope, so an
// install with nothing to do opens that folder a number of times that does not grow with the bins the plan declares.
// Why a fixture of two sizes rather than one measured number:
// the number of opens an install makes of its records legitimately grows with the bins, so a bound per bin would need
// a written-down allowance; the relation between two plans of the same shape needs none, and a per-bin open of the
// folder's listing breaks it (measured 2026-11-16 on a plan declaring 174 bins: 175 opens of the launcher folder
// against 1 before the cleanup of killed writes learned to look once per folder).

/// The crate the control's plan declares:
/// one of this repository's own,
/// so every declared bin is exposed through a launcher whose record the install writes.
const CONTROL_CRATE: &str = "monochromatic-mpm-shadow-control";

/// The version of the control's crate the fixture leaves installed under the disposable home.
const CONTROL_VERSION: &str = "0.6.0";

/// The bins the control's smaller plan declares.
const FEW_BINS: usize = 2;

/// The bins the control's larger plan declares.
const MANY_BINS: usize = 40;

/// The bin names of a plan declaring `bins` bins: the crate's own program first, then one per further bin.
fn declared_bins(bins: usize) -> Vec<String> {
    let mut names: Vec<String> = vec![CONTROL_CRATE.to_string()];
    while names.len() < bins {
        names.push(format!("{CONTROL_CRATE}-bin-{}", names.len() - 1));
    }
    return names;
}

/// What:
/// A worktree whose plan declares `bins` bins of one repository-published crate,
/// with that crate already installed under a disposable home,
/// so that an `install` after the first has nothing to do.
/// Why:
/// The cost at stake is what an install does when it changes nothing,
/// which it reaches at every start of work in a checkout;
/// this fixture is the smallest state that gets there.
struct NothingToDo {
    directory: tempfile::TempDir,
}

impl NothingToDo {
    fn new(bins: usize) -> NothingToDo {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        let fixture = NothingToDo { directory };
        fs::create_dir_all(fixture.work()).expect("make the worktree");
        fs::create_dir_all(fixture.home()).expect("make the home");
        let names: Vec<String> = declared_bins(bins);
        fs::write(fixture.work().join(PLAN_FILE_NAME), plan_declaring(&names)).expect("write the plan");
        fs::write(fixture.work().join(LOCK_FILE_NAME), lock_declaring(&names)).expect("write the lock");
        fixture.sign_lock();
        fixture.install_the_crate(&names);
        // The first install makes the launchers and their records; every install after it has nothing to do.
        let first = fixture.run(&["install", "--frozen-lockfile"]);
        assert_eq!(first.status.code(), Some(0), "{}", text(&first.stderr));
        return fixture;
    }

    fn home(&self) -> PathBuf {
        return self.directory.path().join("home");
    }

    fn work(&self) -> PathBuf {
        return self.directory.path().join("work");
    }

    /// The scope the binary resolves for this fixture,
    /// so the test names the same launcher folder the install writes into.
    fn scope(&self) -> Scope {
        let home: String = self.home().to_string_lossy().to_string();
        return repo_scope(&self.work(), &EnvMap::from_pairs(&[("HOME", home.as_str())])).expect("resolve the scope");
    }

    /// Rewrites the lock with the digest the binary's own gate computes,
    /// the way `update` would have written it.
    fn sign_lock(&self) {
        let path: PathBuf = self.work().join(LOCK_FILE_NAME);
        let parsed = parse_lock(&fs::read_to_string(&path).expect("read the lock"), "lock").expect("the fixture lock parses");
        fs::write(&path, render_lock(&parsed.lock)).expect("write the signed lock");
    }

    /// Installs the control's crate into the fixture's home the way `cargo binstall` leaves it:
    /// one version directory holding every declared program and cargo's own record of them,
    /// and the `current` link the bin search follows.
    fn install_the_crate(&self, names: &[String]) {
        let scope: Scope = self.scope();
        let version: PathBuf = version_directory(&scope.toolchain_root, CONTROL_CRATE, CONTROL_VERSION);
        let programs: PathBuf = version.join("bin");
        fs::create_dir_all(&programs).expect("make the crate's bin directory");
        for name in names {
            let program: PathBuf = programs.join(name);
            fs::write(&program, "#!/bin/sh\nexit 0\n").expect("write the program");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make the program executable");
            }
        }
        let quoted: Vec<String> = names.iter().map(|name| return format!("\"{name}\"")).collect();
        fs::write(
            version.join(".crates.toml"),
            format!(
                "[v1]\n\"{CONTROL_CRATE} {CONTROL_VERSION} (registry+https://github.com/rust-lang/crates.io-index)\" = [{}]\n",
                quoted.join(", ")
            ),
        )
        .expect("write cargo's record");
        std::os::unix::fs::symlink(CONTROL_VERSION, current_link(&scope.toolchain_root, CONTROL_CRATE)).expect("link current");
    }

    /// Runs the binary with only `HOME` and `PATH` in its environment,
    /// so nothing of this shell (a session bus address above all) reaches it.
    fn command(&self) -> Command {
        let mut command: Command = Command::new(env!("CARGO_BIN_EXE_mpm-shadow"));
        command.current_dir(self.work().join("sub"));
        command.env_clear().env("HOME", self.home()).env("PATH", "/usr/bin:/bin");
        return command;
    }

    fn run(&self, arguments: &[&str]) -> Output {
        fs::create_dir_all(self.work().join("sub")).expect("make the directory the command runs in");
        return self.command().args(arguments).output().expect("run mpm-shadow");
    }

    /// The `openat` lines of one install run under `strace`,
    /// which is how a machine counts the files a program opens.
    fn traced_install(&self) -> Vec<String> {
        let trace: PathBuf = self.directory.path().join("openat-of-one-install.txt");
        let mut traced: Command = Command::new("strace");
        traced.current_dir(self.work().join("sub"));
        traced.env_clear().env("HOME", self.home()).env("PATH", "/usr/bin:/bin");
        traced
            .args(["--quiet", "--follow-forks", "--trace=openat", "--output"])
            .arg(&trace)
            .args([env!("CARGO_BIN_EXE_mpm-shadow"), "install", "--frozen-lockfile"]);
        let output: Output = match traced.output() {
            Ok(output) => output,
            Err(error) => {
                panic!(
                    "this control counts the files an install opens, so it runs the binary under `strace`, and starting it failed: {error}. Install strace on this machine and run the task again."
                );
            }
        };
        let stdout: String = text(&output.stdout);
        assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
        assert!(stdout.contains("0 installed"), "the fixture must have nothing to do: {stdout}");
        assert!(stdout.contains("0 failed"), "the fixture must have nothing to do: {stdout}");
        return fs::read_to_string(&trace).expect("read the trace").lines().map(str::to_string).collect();
    }
}

/// The plan text declaring `names` as the bins of the control's crate.
fn plan_declaring(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| return format!("        \"{name}\",\n")).collect();
    return format!(
        "{{\n  \"schemaVersion\": 1,\n  \"tools\": {{\n    \"cargo:{CONTROL_CRATE}\": {{\n      \"request\": \"cargo:{CONTROL_CRATE}@{CONTROL_VERSION}\",\n      \"route\": \"cargo-binstall\",\n      \"packageId\": \"{CONTROL_CRATE}\",\n      \"bins\": [\n{}      ]\n    }}\n  }},\n}}\n",
        quoted.join("")
    );
}

/// The lock text declaring `names` as the bins of the control's crate,
/// without a digest: `sign_lock` adds the one the binary's gate computes.
fn lock_declaring(names: &[String]) -> String {
    let mut document: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    document.insert("schemaVersion".to_string(), serde_json::Value::from(1));
    document.insert("generatedFrom".to_string(), serde_json::Value::String(PLAN_FILE_NAME.to_string()));
    let mut tool: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    tool.insert("request".to_string(), serde_json::Value::String(format!("cargo:{CONTROL_CRATE}@{CONTROL_VERSION}")));
    tool.insert("version".to_string(), serde_json::Value::String(CONTROL_VERSION.to_string()));
    tool.insert("route".to_string(), serde_json::Value::String("cargo-binstall".to_string()));
    tool.insert("packageId".to_string(), serde_json::Value::String(CONTROL_CRATE.to_string()));
    tool.insert(
        "bins".to_string(),
        serde_json::Value::Array(names.iter().map(|name| return serde_json::Value::String(name.clone())).collect()),
    );
    tool.insert("env".to_string(), serde_json::Value::Object(serde_json::Map::new()));
    tool.insert("options".to_string(), serde_json::Value::Object(serde_json::Map::new()));
    tool.insert("artifacts".to_string(), serde_json::Value::Object(serde_json::Map::new()));
    let mut tools: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    tools.insert(format!("cargo:{CONTROL_CRATE}"), serde_json::Value::Object(tool));
    document.insert("tools".to_string(), serde_json::Value::Object(tools));
    return format!("\n{}\n", serde_json::to_string_pretty(&serde_json::Value::Object(document)).expect("render the lock"));
}

/// The system calls in `trace` that opened exactly `path`, not a file inside it.
fn opens_of(trace: &[String], path: &Path) -> usize {
    let opened: String = format!("openat(AT_FDCWD, \"{}\"", path.display());
    return trace.iter().filter(|line| return line.contains(&opened)).count();
}

/// What:
/// An install with nothing to do opens the launcher folder the same number of times whatever number of bins the plan
/// declares.
/// Why:
/// The plan gives the scope one launcher folder,
/// so its opens cannot be a function of the bins;
/// a listing of that folder per launch record is what made an install with nothing to do cost more than an install
/// that predates the cleanup of what a killed write leaves
/// (175 listings across a plan of 174 bins),
/// and this relation is what fails if such a per-bin listing comes back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('opens the launcher folder once, however many bins the plan declares', () => {
///   expect(opensOfTheLauncherFolder(FEW_BINS)).toBe(opensOfTheLauncherFolder(MANY_BINS));
/// });
/// ```
#[test]
#[cfg(target_os = "linux")]
fn an_install_with_nothing_to_do_opens_the_launcher_folder_no_more_often_for_many_bins_than_for_few() {
    let few = NothingToDo::new(FEW_BINS);
    let few_opens: usize = opens_of(&few.traced_install(), &few.scope().launcher_dir);
    let many = NothingToDo::new(MANY_BINS);
    let many_opens: usize = opens_of(&many.traced_install(), &many.scope().launcher_dir);
    assert_eq!(
        few_opens, many_opens,
        "an install with nothing to do opened the launcher folder {few_opens} times for a plan declaring {FEW_BINS} bins and {many_opens} times for one declaring {MANY_BINS}: the plan declares one launcher folder, so its opens must not grow with the bins"
    );
}

//endregion An install with nothing to do opens the launcher folder once, however many bins the plan declares
