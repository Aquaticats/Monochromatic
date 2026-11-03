//! What:
//! End-to-end tests of the built `mpm-shadow` binary:
//! it is started through a symlink named `mpm`,
//! with a
//!           fake "real" `mpm` script on PATH and a disposable cargo home,
//! exactly as the toolchain will run it.
//! Why:
//! The unit tests prove the pieces;
//! only a real process shows that the arguments arrive untouched,
//! the exit
//!           status passes through,
//! and the stale copies are gone afterwards.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.integration.test.ts
//! const result = spawnSync(path.join(shadowDirectory, 'mpm'), ['list'], { env: { PATH, CARGO_HOME } });
//! ```

/// Imports the file-system module (writes the fake mpm,
/// makes links and fixtures).
use std::fs;

/// Imports the trait that adds `from_mode` to permissions (marks the fake mpm executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed path type.
use std::path::Path;

/// Imports the not-found exit code the shadow reports when no real mpm is on PATH.
use mpm_shadow::constants::{EXEC_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE};
use mpm_shadow::nesting::MAX_NESTING as NESTING_LIMIT;

/// What:
/// `use mpm_shadow::cli::declared_verbs;` imports the function that lists every verb the shadow declares (its
///           pnpm-named toolchain verbs and `prune`), each with its name and aliases.
/// Why:
/// The pnpm-verb tests walk that one table,
/// so a verb added to it is tested as an `mpm` word with no list kept here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { declaredVerbs } from '../src/cli.ts';
/// ```
use mpm_shadow::cli::declared_verbs;

/// Imports `Command` (start a program) and `Output` (its result).
use std::process::{Command, Output};

/// What:
/// `fn fixture_root() -> tempfile::TempDir` declares a helper returning a disposable directory that is
///           removed when the returned guard drops.
/// Why:
/// `env!("CARGO_TARGET_TMPDIR")` is a compile-time macro (the `!`) Cargo sets for integration tests:
/// a
///           directory in the build output,
/// so no test writes under the system temporary directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = fs.mkdtempSync(path.join(TARGET_TMPDIR, 'mpm-shadow-'));
/// ```
fn fixture_root() -> tempfile::TempDir {
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
}

/// What:
/// `fn write_fake_mpm(directory: &Path, script: &str)` declares a helper writing an executable `mpm` script.
/// Why:
/// The shadow must find and run a program named `mpm` that is not itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeFakeMpm(directory: string, script: string): void
/// ```
fn write_fake_mpm(directory: &Path, script: &str) {
    fs::create_dir_all(directory).expect("make the fake mpm directory");
    fs::write(directory.join("mpm"), script).expect("write the fake mpm");
    fs::set_permissions(directory.join("mpm"), fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// What:
/// `fn start_shadow(root: &Path, arguments: &[&str]) -> Output` declares a helper that links the built binary
///           as `mpm` and runs it with a PATH holding the link directory then the fake mpm's directory.
/// Why:
/// This is the exact shape of the real setup:
/// the link's directory first on PATH.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startShadow(root: string, args: string[]): SpawnSyncReturns
/// ```
fn start_shadow(root: &Path, arguments: &[&str]) -> Output {
    return start_shadow_with(root, arguments, &[]);
}

/// Like `start_shadow`, with extra environment variables for the shadow.
fn start_shadow_with(root: &Path, arguments: &[&str], extra_env: &[(&str, &str)]) -> Output {
    let links: std::path::PathBuf = root.join("links");
    fs::create_dir_all(&links).expect("make the link directory");
    // `env!("CARGO_BIN_EXE_mpm-shadow")` is a compile-time macro Cargo sets to the built binary's path.
    let _ = fs::remove_file(links.join("mpm"));
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_mpm-shadow"), links.join("mpm")).expect("link the shadow as mpm");
    let path_variable: String = format!("{}:{}:/usr/bin:/bin", links.display(), root.join("real").display());
    return Command::new(links.join("mpm"))
        .args(arguments)
        .env("PATH", path_variable)
        .env("CARGO_HOME", root.join("cargo"))
        // The shadow finds the toolchain roots through these two, so they point into the fixture:
        // the machine's own install records must never decide what a test prunes.
        .env("HOME", root.join("home"))
        .env("XDG_DATA_HOME", root.join("data"))
        .envs(extra_env.iter().copied())
        .output()
        .expect("run the shadow");
}

/// What:
/// `fn write_stale_copies(root: &Path)` declares a helper creating a cargo home with an install record and
///           two cached versions of one of our crates.
/// Why:
/// The record says 0.4.0 is installed,
/// so the 0.1.0 archive and source must disappear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeStaleCopies(root: string): void
/// ```
fn write_stale_copies(root: &Path) {
    let cargo: std::path::PathBuf = root.join("cargo");
    let cache: std::path::PathBuf = cargo.join("registry/cache/index.crates.io-fixture");
    let source: std::path::PathBuf = cargo.join("registry/src/index.crates.io-fixture");
    fs::create_dir_all(&cache).expect("make the cache");
    fs::create_dir_all(source.join("monochromatic-lint-0.1.0")).expect("make the old source");
    fs::create_dir_all(source.join("monochromatic-lint-0.4.0")).expect("make the current source");
    fs::write(cache.join("monochromatic-lint-0.1.0.crate"), "").expect("write the old archive");
    fs::write(cache.join("monochromatic-lint-0.4.0.crate"), "").expect("write the current archive");
    fs::write(
        cargo.join(".crates.toml"),
        "[v1]\n\"monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"monochromatic-lint\"]\n",
    )
    .expect("write the install record");
}

/// The arguments arrive untouched,
/// the child's exit code passes through,
/// and the stale copies are gone afterwards.
#[test]
fn passes_arguments_and_exit_code_through_then_prunes() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nfor a in \"$@\"; do echo \"[$a]\"; done\nexit 3\n");
    write_stale_copies(root.path());

    let output: Output = start_shadow(root.path(), &["--output-format", "json", "a b"]);

    // `String::from_utf8_lossy(&bytes)` reads captured output bytes as text, replacing invalid UTF-8.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "[--output-format]\n[json]\n[a b]\n");
    assert_eq!(output.status.code(), Some(3));
    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains("post-run cleanup"), "{stderr}");
    assert!(!stderr.contains("deleted stale"), "the per-path list is for MPM_SHADOW_VERBOSE: {stderr}");
    let cargo: std::path::PathBuf = root.path().join("cargo");
    assert!(!cargo.join("registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate").exists());
    assert!(!cargo.join("registry/src/index.crates.io-fixture/monochromatic-lint-0.1.0").exists());
    assert!(cargo.join("registry/cache/index.crates.io-fixture/monochromatic-lint-0.4.0.crate").exists());
}

/// Standard output stays exactly mpm's when there is nothing to prune.
#[test]
fn adds_nothing_to_standard_output_or_error_when_nothing_is_stale() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\necho hello\n");

    let output: Output = start_shadow(root.path(), &[]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
}

/// With no real mpm on PATH the shadow reports it and exits 127.
#[test]
fn exits_127_when_no_real_mpm_is_on_path() {
    let root = fixture_root();

    let output: Output = start_shadow(root.path(), &[]);

    assert_eq!(output.status.code(), Some(NOT_FOUND_EXIT_CODE));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no real mpm found on PATH"));
}

/// Started as `mpm-shadow`,
/// the `prune` subcommand deletes the stale copies and says which.
#[test]
fn the_prune_subcommand_deletes_and_reports() {
    let root = fixture_root();
    write_stale_copies(root.path());

    let output: Output = Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
        .arg("prune")
        .env("CARGO_HOME", root.path().join("cargo"))
        .output()
        .expect("run mpm-shadow prune");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("deleted stale"));
    assert!(!root.path().join("cargo/registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate").exists());
}

/// The fake mpm of the notice tests:
/// it answers the way mpm 8.2.0 does for a top-level `--version` and `--help`,
/// for a subcommand's own help,
/// and for another output format.
const FAKE_MPM_WITH_VERSION_AND_HELP: &str = "#!/bin/sh\n\
case \"$1\" in\n\
  --version) printf 'mpm, version 8.2.0\\nPython 3.13\\n' ;;\n\
  --help|-h) printf 'Usage: mpm [OPTIONS] COMMAND [ARGS]...\\n\\nOther commands:\\n  help\\n' ;;\n\
  installed) printf 'Usage: mpm installed [OPTIONS]\\n' ;;\n\
  --help-format) printf '{\"help\": true}\\n' ;;\n\
  *) printf 'other\\n' ;;\n\
esac\n\
exit 0\n";

/// The real mpm's text is byte for byte first,
/// then exactly one added line.
#[test]
fn a_top_level_version_gets_one_added_line_after_mpms_own_output() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), FAKE_MPM_WITH_VERSION_AND_HELP);
    let real: String = root.path().join("real").join("mpm").display().to_string();

    let output: Output = start_shadow(root.path(), &["--version"]);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("mpm, version 8.2.0\nPython 3.13\nmpm-shadow {}, shadowing mpm at {real}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(output.status.code(), Some(0));
}

/// Options before the subcommand are skipped the way mpm's parser skips them.
#[test]
fn a_version_after_other_top_level_options_still_gets_the_line() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nprintf 'mpm, version 8.2.0\\n'\n");

    let output: Output = start_shadow(root.path(), &["--verbosity", "DEBUG", "--no-color", "--version"]);

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(text.starts_with("mpm, version 8.2.0\nmpm-shadow "), "{text}");
    assert_eq!(text.lines().count(), 2, "{text}");
}

/// After mpm's help comes a blank line and a short section naming the shadow.
#[test]
fn a_top_level_help_gets_a_section_after_mpms_own_help() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), FAKE_MPM_WITH_VERSION_AND_HELP);
    let real: String = root.path().join("real").join("mpm").display().to_string();

    for flag in ["--help", "-h"] {
        let output: Output = start_shadow(root.path(), &[flag]);
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        let own = "Usage: mpm [OPTIONS] COMMAND [ARGS]...\n\nOther commands:\n  help\n";
        assert!(text.starts_with(own), "{flag}: mpm's help comes first and unchanged:\n{text}");
        let added = &text[own.len()..];
        assert!(added.starts_with(&format!("\nmpm-shadow {}:\n", env!("CARGO_PKG_VERSION"))), "{flag}: {added}");
        assert!(added.contains(&real) && added.contains("monochromatic-") && added.contains("deleted stale"), "{flag}: {added}");
        assert_eq!(output.status.code(), Some(0));
    }
}

/// The help of a subcommand only the real mpm has (`installed`),
/// the `help` subcommand,
/// a line that starts with mpm's own option,
/// and another output format print exactly what mpm printed.
/// (`mpm install --help` is the shadow's own `install` help:
/// `install` is pnpm's verb.)
#[test]
fn non_top_level_help_and_other_outputs_pass_through_untouched() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), FAKE_MPM_WITH_VERSION_AND_HELP);

    let cases: [(&[&str], &str); 4] = [
        (&["installed", "--help"], "Usage: mpm installed [OPTIONS]\n"),
        (&["--verbose", "install", "--help"], "other\n"),
        (&["help", "install"], "other\n"),
        (&["--help-format", "json", "--help"], "{\"help\": true}\n"),
    ];
    for (arguments, expected) in cases {
        let output: Output = start_shadow(root.path(), arguments);
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "{arguments:?}");
    }
}

/// A failing `--version` keeps mpm's exit code and gets no added line.
#[test]
fn the_exit_code_of_mpm_passes_through_and_a_failed_version_gets_no_line() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nprintf 'broken\\n'\nexit 7\n");

    let output: Output = start_shadow(root.path(), &["--version"]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "broken\n");
    assert_eq!(output.status.code(), Some(7));
}

/// A version no real machine has installed,
/// so a test that sees it can only have read the disposable data directory it planted it in.
const PLANTED_NEWER_VERSION: &str = "9.9.9";

/// A newer version of the stale crate recorded the way `install` records it
/// (a version directory with its own `.crates.toml` under the toolchain root of the disposable data home),
/// and its archive cached.
fn write_toolchain_record(root: &Path) {
    let version_dir: std::path::PathBuf =
        root.join("data/monochromatic-dev/toolchain/cargo/monochromatic-lint").join(PLANTED_NEWER_VERSION);
    fs::create_dir_all(&version_dir).expect("make the version directory");
    fs::write(
        version_dir.join(".crates.toml"),
        format!(
            "[v1]\n\"monochromatic-lint {PLANTED_NEWER_VERSION} (registry+https://github.com/rust-lang/crates.io-index)\" = [\"monochromatic-lint\"]\n"
        ),
    )
    .expect("write the version-keyed record");
    fs::write(
        root.join("cargo/registry/cache/index.crates.io-fixture").join(format!("monochromatic-lint-{PLANTED_NEWER_VERSION}.crate")),
        "",
    )
    .expect("write the newer archive");
}

/// The shadow reads the toolchain roots under the data home it is given,
/// never the machine's own,
/// so a newer recorded version there is the one kept and the older caches go.
#[test]
fn reads_the_toolchain_roots_of_the_data_home_it_is_given_and_never_the_machines_own() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_stale_copies(root.path());
    write_toolchain_record(root.path());

    let output: Output = start_shadow(root.path(), &["installed"]);

    assert_eq!(output.status.code(), Some(0));
    let cache: std::path::PathBuf = root.path().join("cargo/registry/cache/index.crates.io-fixture");
    assert!(cache.join(format!("monochromatic-lint-{PLANTED_NEWER_VERSION}.crate")).exists());
    assert!(!cache.join("monochromatic-lint-0.4.0.crate").exists());
    assert!(!cache.join("monochromatic-lint-0.1.0.crate").exists());
}

/// How many stale copies the many-paths tests plant.
const MANY_STALE: usize = 18;

/// Many stale archives of one crate next to the one the install record names.
fn write_many_stale_copies(root: &Path) {
    let cargo: std::path::PathBuf = root.join("cargo");
    let cache: std::path::PathBuf = cargo.join("registry/cache/index.crates.io-fixture");
    fs::create_dir_all(&cache).expect("make the cache");
    fs::write(cache.join("monochromatic-lint-0.4.0.crate"), "").expect("write the current archive");
    for index in 0..MANY_STALE {
        fs::write(cache.join(format!("monochromatic-lint-0.1.{index}.crate")), "").expect("write a stale archive");
    }
    fs::write(
        cargo.join(".crates.toml"),
        "[v1]\n\"monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"monochromatic-lint\"]\n",
    )
    .expect("write the install record");
}

/// Many removals are one line by default, and the line says it is cleanup after the mpm command.
#[test]
fn many_stale_copies_are_one_post_run_cleanup_line_by_default() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_many_stale_copies(root.path());

    let output: Output = start_shadow(root.path(), &["installed"]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("post-run cleanup") && stderr.contains(&format!("{MANY_STALE} ")), "{stderr}");
    assert!(stderr.contains("MPM_SHADOW_VERBOSE"), "{stderr}");
}

/// The variable named in that line lists every path.
#[test]
fn the_verbose_variable_lists_every_removed_path_after_the_summary() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_many_stale_copies(root.path());

    let output: Output = start_shadow_with(root.path(), &["installed"], &[("MPM_SHADOW_VERBOSE", "1")]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(stderr.lines().count(), 1 + MANY_STALE, "{stderr}");
    assert_eq!(stderr.matches("deleted stale").count(), MANY_STALE, "{stderr}");
}

/// Nothing stale, nothing printed.
#[test]
fn a_run_with_nothing_stale_prints_no_cleanup_line_even_when_verbose() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");

    let output: Output = start_shadow_with(root.path(), &["installed"], &[("MPM_SHADOW_VERBOSE", "1")]);

    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

/// A real mpm that cannot be started (its interpreter is missing) gets the full explanation,
/// the exit status 126, and no cleanup.
#[test]
fn a_real_mpm_that_cannot_start_is_explained_with_command_program_stage_cause_and_status() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/nonexistent/interpreter\n");
    write_stale_copies(root.path());
    let program: String = root.path().join("real").join("mpm").display().to_string();

    let output: Output = start_shadow(root.path(), &["installed"]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(EXEC_FAILED_EXIT_CODE), "{stderr}");
    for expected in ["mpm installed", &program, "starting the real `mpm`", "126", "could not be started", "post-run cleanup did not run"] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
    assert!(!stderr.contains("deleted stale"), "{stderr}");
    assert!(root.path().join("cargo/registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate").exists(), "no cleanup ran");
}

/// The program the shadow starts learns how deep it is.
#[test]
fn the_program_it_starts_gets_a_nesting_depth_one_deeper_than_its_own() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nprintf '%s' \"$MPM_SHADOW_NESTING\"\n");

    let top: Output = start_shadow(root.path(), &[]);
    assert_eq!(String::from_utf8_lossy(&top.stdout), "1");

    let deeper: Output = start_shadow_with(root.path(), &[], &[("MPM_SHADOW_NESTING", "2")]);
    assert_eq!(String::from_utf8_lossy(&deeper.stdout), "3");
}

/// At the bound the shadow starts nothing and says why.
#[test]
fn a_shadow_nested_to_the_limit_refuses_to_start_another_program() {
    let root = fixture_root();
    let marker: std::path::PathBuf = root.path().join("ran");
    write_fake_mpm(&root.path().join("real"), &format!("#!/bin/sh\ntouch '{}'\n", marker.display()));

    let output: Output = start_shadow_with(root.path(), &["installed"], &[("MPM_SHADOW_NESTING", &NESTING_LIMIT.to_string())]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(EXEC_FAILED_EXIT_CODE), "{stderr}");
    assert!(!marker.exists(), "the program must not have started");
    for expected in ["mpm installed", "started by another mpm-shadow", "itself", "126"] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
}

/// The failure of 2026-11-03 (issue 6259):
/// the link `install` makes resolves to a per-tree launcher, which starts the real shadow binary under the name `mpm`;
/// that shadow must not take the launcher for the real mpm and start it again.
#[test]
fn the_per_tree_launcher_of_the_running_shadow_is_not_taken_for_the_real_mpm() {
    use mpm_shadow::toolchain::launcher::{LaunchRecord, record_path, render_record};
    use mpm_shadow::toolchain::scope::ScopeKind;
    let root = fixture_root();
    let runs: std::path::PathBuf = root.path().join("runs.log");
    write_fake_mpm(&root.path().join("real"), &format!("#!/bin/sh\necho run >> '{}'\nexit 5\n", runs.display()));
    let binary: &str = env!("CARGO_BIN_EXE_mpm-shadow");
    let installed: std::path::PathBuf = root.path().join("toolchain/mpm-shadow/bin");
    fs::create_dir_all(&installed).expect("make the install directory");
    fs::copy(binary, installed.join("mpm-shadow")).expect("install the real shadow binary");
    let launchers: std::path::PathBuf = root.path().join("launchers");
    fs::create_dir_all(&launchers).expect("make the launcher directory");
    // A copy, not a hard link: the build directory and the fixture can be on different file systems.
    fs::copy(binary, launchers.join("mpm-shadow")).expect("make the launcher");
    let record = LaunchRecord {
        tool_id: "mpm-shadow".to_string(),
        lock_file: root.path().join("toolchain.lock.json"),
        real: installed.join("mpm-shadow"),
        scope: ScopeKind::Repo,
    };
    fs::write(record_path(&launchers.join("mpm-shadow")), render_record(&record)).expect("write the launch record");
    let links: std::path::PathBuf = root.path().join("links");
    fs::create_dir_all(&links).expect("make the link directory");
    std::os::unix::fs::symlink(launchers.join("mpm-shadow"), links.join("mpm")).expect("link the launcher as mpm");
    let path_variable: String = format!("{}:{}:/usr/bin:/bin", links.display(), root.path().join("real").display());

    let output: Output = Command::new(links.join("mpm"))
        .arg("installed")
        .env("PATH", path_variable)
        .env("CARGO_HOME", root.path().join("cargo"))
        .env("HOME", root.path().join("home"))
        .env("XDG_DATA_HOME", root.path().join("data"))
        .output()
        .expect("run the launcher");

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(5), "the real mpm's own status: {stderr}");
    assert_eq!(fs::read_to_string(&runs).expect("the real mpm ran").lines().count(), 1, "{stderr}");
}

/// What:
/// A fake real mpm that leaves a marker file when it runs,
/// so a test can tell whether the shadow started it.
/// Why:
/// The pnpm-verb tests must prove the real mpm never ran,
/// not only that the output looks right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeMarkingMpm(root: string): string // returns the marker path
/// ```
fn write_marking_mpm(root: &Path) -> std::path::PathBuf {
    let marker: std::path::PathBuf = root.join("real-mpm-ran");
    write_fake_mpm(&root.join("real"), &format!("#!/bin/sh\necho \"$@\" >> '{}'\nexit 7\n", marker.display()));
    return marker;
}

/// What:
/// Every verb and alias the shadow declares,
/// typed to `mpm` as its first word,
/// runs the shadow's own command:
///           `mpm <word> --help` prints that command's help and exits 0,
/// and the real mpm never starts.
/// Why:
/// The owner wants pnpm's verbs on `mpm` itself:
/// "I want pnpm style verbs.
/// We talked about this!" (2026-11-03,
///           after `mpm i -g` reached mpm 8.2.0 and failed with "No such command 'i'").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const verb of declaredVerbs()) for (const word of [verb.name, ...verb.aliases]) {
///   const result = startShadow(root, [word, '--help']);
///   expect(result.status).toBe(0); expect(realMpmRan).toBe(false);
/// }
/// ```
#[test]
fn every_pnpm_verb_and_alias_typed_to_mpm_runs_the_shadows_own_command_and_never_the_real_mpm() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());
    for verb in declared_verbs() {
        // The verb's name first, then each alias the table gives it.
        let mut words: Vec<&str> = vec![verb.name];
        words.extend(verb.aliases.iter().copied());
        for word in words {
            let output: Output = start_shadow(root.path(), &[word, "--help"]);
            let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
            assert_eq!(output.status.code(), Some(0), "mpm {word} --help: {stdout}{stderr}");
            assert!(stdout.contains(&format!("mpm {}", verb.name)), "mpm {word} --help prints the {} help: {stdout}", verb.name);
            assert!(!marker.exists(), "mpm {word} started the real mpm");
        }
    }
}

/// What:
/// The command of issue 6259,
/// `mpm i -g`,
/// is the toolchain's global install:
/// with no machine-global plan in the fixture
///           it is refused by the shadow,
/// and the real mpm is never started.
/// Why:
/// It is the exact command the owner typed;
/// the config home points into the fixture so the machine's own global plan is never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = startShadow(root, ['i', '-g'], { XDG_CONFIG_HOME: join(root, 'config') });
/// expect(realMpmRan).toBe(false); expect(result.stderr).not.toContain("No such command");
/// ```
#[test]
fn mpm_i_g_is_the_toolchains_global_install_and_never_reaches_the_real_mpm() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());
    let config: String = root.path().join("config").display().to_string();

    let output: Output = start_shadow_with(root.path(), &["i", "-g"], &[("XDG_CONFIG_HOME", &config)]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(!marker.exists(), "mpm i -g started the real mpm: {stderr}");
    assert!(!stderr.contains("No such command"), "{stderr}");
    assert!(stderr.contains("toolchain"), "the shadow's own install answered: {stderr}");
}

/// What:
/// A word only the real mpm has (`installed`) still reaches it,
/// with every argument untouched and its exit status passed through.
/// Why:
/// Only pnpm's verbs moved to the shadow;
/// mpm's own commands keep working through `mpm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = startShadow(root, ['installed', '--manager', 'pip']);
/// expect(readMarker()).toBe('installed --manager pip\n'); expect(result.status).toBe(7);
/// ```
#[test]
fn a_word_only_the_real_mpm_knows_still_reaches_it_with_its_arguments_and_status() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());

    let output: Output = start_shadow(root.path(), &["installed", "--manager", "pip"]);

    assert_eq!(output.status.code(), Some(7), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read_to_string(&marker).expect("the real mpm ran"), "installed --manager pip\n");
}

/// What:
/// A command line that starts with one of mpm's own options stays mpm's,
/// even when a pnpm verb follows:
///           `mpm --verbosity DEBUG outdated` runs the real mpm's `outdated`.
/// Why:
/// pnpm verbs are typed first (`mpm i -g`);
/// mpm's top-level options belong to the real mpm,
/// so such a line is the real mpm's command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startShadow(root, ['--verbosity', 'DEBUG', 'outdated']); expect(realMpmRan).toBe(true);
/// ```
#[test]
fn a_command_line_that_starts_with_an_mpm_option_stays_the_real_mpms_even_before_a_pnpm_verb() {
    let root = fixture_root();
    let marker: std::path::PathBuf = write_marking_mpm(root.path());

    let output: Output = start_shadow(root.path(), &["--verbosity", "DEBUG", "outdated"]);

    assert_eq!(output.status.code(), Some(7), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read_to_string(&marker).expect("the real mpm ran"), "--verbosity DEBUG outdated\n");
}
