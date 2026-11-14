//! End-to-end tests of the toolchain verbs of the built `mpm-shadow` binary,
//! run against a disposable worktree and a
//! disposable home.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.

use std::fs;

use mpm_shadow::cli::{build_command, declared_verbs};
use mpm_shadow::constants::USAGE_EXIT_CODE;
use mpm_shadow::toolchain::archive_staging;
use mpm_shadow::toolchain::lock::{parse_lock, render_lock};
use mpm_shadow::toolchain::scope::{EXPOSURE_DIR_RELATIVE, LOCK_FILE_NAME, PLAN_FILE_NAME};
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

//region a second command changing the store under the cleanup

/// The crate the cleanup controls keep in a store of their own.
const CLEANUP_CRATE: &str = "mpm-shadow-cleanup-fixture";

/// The version the cleanup judges removable in the controls.
const CLEANUP_OLD: &str = "1.0.0";

/// The version a lock names, which the cleanup keeps.
const CLEANUP_NAMED: &str = "2.0.0";

/// A worktree, a home and a store the cleanup controls own,
/// with a plan that declares no tool and the lock the binary itself wrote for it.
struct Store {
    /// The fixture directory, removed when the fixture drops.
    directory: tempfile::TempDir,
}

impl Store {
    /// A store whose worktree is a git repository, because the cleanup lists worktrees through git.
    fn new() -> Store {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        fs::create_dir_all(directory.path().join("home")).expect("make the home");
        fs::create_dir_all(directory.path().join("work")).expect("make the worktree");
        fs::write(directory.path().join("work").join(PLAN_FILE_NAME), "{\"schemaVersion\": 1, \"tools\": {}}\n").expect("write the plan");
        let repository = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(directory.path().join("work"))
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run git init");
        assert!(repository.status.success(), "{}", text(&repository.stderr));
        let fixture = Store { directory };
        // A plan with no tool resolves to nothing, so this writes the lock with no network read.
        let written = fixture.run(&["install", "--no-frozen-lockfile"]);
        assert_eq!(written.status.code(), Some(0), "{}", text(&written.stderr));
        return fixture;
    }

    /// The worktree.
    fn work(&self) -> PathBuf {
        return self.directory.path().join("work");
    }

    /// The home every command of these controls reads.
    fn home(&self) -> PathBuf {
        return self.directory.path().join("home");
    }

    /// The repository scope's toolchain root.
    fn root(&self) -> PathBuf {
        return self.home().join(".local").join("share").join("monochromatic-dev").join("toolchain");
    }

    /// The directory of the fixture crate in that root.
    fn crate_dir(&self) -> PathBuf {
        return self.root().join("cargo").join(CLEANUP_CRATE);
    }

    /// The directory the archives of one version were downloaded into.
    fn downloads(&self, version: &str) -> PathBuf {
        return self.root().join("downloads").join("cargo").join(CLEANUP_CRATE).join(version);
    }

    /// Writes one version directory of the fixture crate.
    fn write_version(&self, version: &str, files: &[&str]) {
        let directory = self.crate_dir().join(version);
        fs::create_dir_all(&directory).expect("make the version");
        for file in files {
            fs::write(directory.join(file), b"#!/bin/sh\n").expect("write a file of the version");
        }
    }

    /// Writes the machine scope's lock, naming the fixture crate at `version`.
    /// The machine scope keeps its lock in the config directory, not beside the store.
    fn name_in_the_machine_lock(&self, version: &str) {
        let directory = self.home().join(".config").join("monochromatic-dev").join("toolchain");
        fs::create_dir_all(&directory).expect("make the machine scope");
        let text = format!(
            "{{\"schemaVersion\": 1, \"tools\": {{\"cargo:{CLEANUP_CRATE}\": {{\"request\": \"cargo:{CLEANUP_CRATE}@latest\", \
             \"version\": \"{version}\", \"route\": \"cargo-binstall\", \"packageId\": \"{CLEANUP_CRATE}\"}}}}}}\n"
        );
        fs::write(directory.join(LOCK_FILE_NAME), text).expect("write the machine lock");
    }

    /// A command in the worktree with a given `PATH`,
    /// and the cleanup's bounds set so that the store the fixture wrote is what it judges.
    fn command(&self, arguments: &[&str], path: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mpm-shadow"));
        command
            .args(arguments)
            .current_dir(self.work())
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", path)
            // Every version the fixture wrote is over the size bound and older than the floor:
            // the controls are about the removal, not about the rule that asks for one.
            // The work limit is raised because a control that holds the install's renames spends the shipped two
            // seconds before the cleanup reaches its first removal.
            .env("MPM_SHADOW_STORE_SIZE_BOUND", "0")
            .env("MPM_SHADOW_STORE_AGE_FLOOR", "0")
            .env("MPM_SHADOW_STORE_WORK_LIMIT", "60s")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        return command;
    }

    /// Runs one command and returns what it printed and how it ended.
    fn run(&self, arguments: &[&str]) -> Output {
        return self.command(arguments, "/usr/bin:/bin").output().expect("run mpm-shadow");
    }
}

/// Waits for `child` to end,
/// giving up after `deadline` so a command that waits for ever cannot hold the test.
fn wait_until_ended(child: &mut std::process::Child, deadline: std::time::Duration) -> Option<std::process::ExitStatus> {
    let until = std::time::Instant::now() + deadline;
    loop {
        match child.try_wait().expect("ask about the command") {
            Some(status) => return Some(status),
            None if std::time::Instant::now() >= until => return None,
            None => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
}

/// Waits for `path` to appear,
/// giving up after `deadline`.
fn wait_until_present(path: &Path, deadline: std::time::Duration) -> bool {
    let until = std::time::Instant::now() + deadline;
    while std::time::Instant::now() < until {
        if path.exists() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    return path.exists();
}

/// What:
/// A version another command publishes at the name the cleanup is removing is never removed:
/// the cleanup keeps it and says why.
/// Why:
/// The review of 2026-11-14 found by reading that `store_trim`'s removal rechecks the version directory's write time
/// and then renames whatever stands at that name to the removal marker;
/// a `publish_staged` of another command that lands between the two was renamed away and deleted,
/// which is a freshly installed version, possibly one a lock now names.
/// The window is a few system calls, so this control holds the install's `rename` calls with `strace` and publishes
/// the fresh copy while the cleanup is inside the window.
#[test]
fn a_version_another_run_publishes_while_the_cleanup_judges_it_is_never_removed() {
    let store = Store::new();
    store.name_in_the_machine_lock(CLEANUP_NAMED);
    store.write_version(CLEANUP_OLD, &["program"]);
    store.write_version(CLEANUP_NAMED, &["program"]);
    fs::create_dir_all(store.downloads(CLEANUP_OLD)).expect("make the downloads");
    fs::write(store.downloads(CLEANUP_OLD).join("archive.tar.gz"), b"x").expect("write an archive");
    // A removal an earlier run left unfinished: the pass runs even though this install installs nothing.
    fs::write(store.root().join(".mpm-shadow-removal-unfinished"), b"").expect("leave the marker the pass runs for");
    // The fresh copy the other command publishes at the version's name inside the window,
    // staged the way an archive install stages it.
    let fresh = store.directory.path().join("fresh");
    fs::create_dir_all(&fresh).expect("make the fresh copy");
    fs::write(fresh.join("program"), b"#!/bin/sh\n").expect("write the fresh program");
    fs::write(fresh.join(archive_staging::COMPLETE_MARKER), b"mpm-shadow-cleanup-fixture 1.0.0\n").expect("mark the fresh copy finished");

    // Every rename of the install is held at its entry, so the cleanup takes seconds between its recheck and its move.
    let mut held = Command::new("strace");
    held.args(["-f", "-o", "/dev/null", "-e", "trace=rename", "-e", "inject=rename:delay_enter=1000000"]);
    held.arg(env!("CARGO_BIN_EXE_mpm-shadow")).arg("install").arg("--frozen-lockfile");
    held.current_dir(store.work())
        .env_clear()
        .env("HOME", store.home())
        .env("PATH", "/usr/bin:/bin")
        .env("MPM_SHADOW_STORE_SIZE_BOUND", "0")
        .env("MPM_SHADOW_STORE_AGE_FLOOR", "0")
        .env("MPM_SHADOW_STORE_WORK_LIMIT", "60s")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child: std::process::Child = match held.spawn() {
        Ok(started) => started,
        Err(error) => panic!("this control holds the cleanup's renames with strace, which must start: {error}"),
    };
    // The cleanup moves the downloads out of their name first, which is where the window opens.
    let downloads_marker: PathBuf = store.downloads(CLEANUP_OLD).with_file_name(".mpm-shadow-removing.1.0.0");
    if !wait_until_present(&downloads_marker, std::time::Duration::from_secs(120)) {
        let _killed = child.kill();
        let _reaped = child.wait();
        panic!("the cleanup never reached its first rename, so this control proved nothing");
    }
    // The publish of another command, through the code that does it: the copy there is not finished,
    // so it is moved aside, the fresh one takes the name, and the copy that was there is discarded.
    archive_staging::publish_staged(&fresh, &store.crate_dir().join(CLEANUP_OLD), "program").expect("publish the fresh copy");
    let ended = wait_until_ended(&mut child, std::time::Duration::from_secs(180));
    let Some(status) = ended else {
        let _killed = child.kill();
        let _reaped = child.wait();
        panic!("the install was still running 180 s after it started");
    };
    let output: Output = child.wait_with_output().expect("read the install's output");
    let report: String = format!("{}{}", text(&output.stdout), text(&output.stderr));

    assert!(status.success(), "an install whose cleanup kept a published version ends well: {report}");
    assert!(
        store.crate_dir().join(CLEANUP_OLD).join(archive_staging::COMPLETE_MARKER).is_file(),
        "the version another command published was removed: {report}"
    );
    assert!(
        report.contains("another run wrote it while the cleanup ran"),
        "the cleanup says why it kept the published version: {report}"
    );
    assert!(store.crate_dir().join(CLEANUP_NAMED).join("program").is_file(), "the version the lock names stays");
}

/// What:
/// A `git` that never answers costs the install its worktree list within a stated bound, and nothing else:
/// the cleanup keeps every version and says which read ran out of time.
/// Why:
/// The review of 2026-11-14 found that the install path runs `git worktree list` through a runner with no timeout,
/// so a git blocked on a listed worktree's hung file system made `install` wait for ever.
#[test]
fn a_git_that_never_answers_costs_the_install_its_worktree_list_within_a_stated_bound() {
    let store = Store::new();
    store.name_in_the_machine_lock(CLEANUP_NAMED);
    store.write_version(CLEANUP_OLD, &["program"]);
    store.write_version(CLEANUP_NAMED, &["program"]);
    fs::write(store.root().join(".mpm-shadow-removal-unfinished"), b"").expect("leave the marker the pass runs for");
    // A `git` that answers nothing: the stand-in waits on its own sleep, which ends long before the test's deadline
    // only if something stops it.
    let standin = store.directory.path().join("standin");
    fs::create_dir_all(&standin).expect("make the stand-in directory");
    fs::write(standin.join("git"), "#!/bin/sh\nsleep 300\n").expect("write the stand-in");
    #[cfg(unix)]
    {
        /// Imports `PermissionsExt` from `std::os::unix::fs`.
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(standin.join("git"), fs::Permissions::from_mode(0o700)).expect("make the stand-in executable");
    }

    let mut child: std::process::Child = store
        .command(&["install", "--frozen-lockfile"], &format!("{}:/usr/bin:/bin", standin.display()))
        .spawn()
        .expect("start the install");
    let ended = wait_until_ended(&mut child, std::time::Duration::from_secs(60));
    let Some(status) = ended else {
        let _killed = child.kill();
        let _reaped = child.wait();
        panic!("the install was still waiting on git 60 s after it started, so the read has no bound");
    };
    let output: Output = child.wait_with_output().expect("read the install's output");
    let report: String = format!("{}{}", text(&output.stdout), text(&output.stderr));

    assert!(status.success(), "a read that ran out of time is said, and the install still ends well: {report}");
    assert!(report.contains("worktree list --porcelain"), "the sentence names the read: {report}");
    assert!(report.contains("was still running after its bound of 10 s"), "the sentence names the bound: {report}");
    assert!(report.contains("every version is kept until the locks can be read"), "the sentence says what it did: {report}");
    assert!(store.crate_dir().join(CLEANUP_OLD).join("program").is_file(), "nothing was removed");
}

//endregion
