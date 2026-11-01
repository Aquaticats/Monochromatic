//! What:
//! End-to-end tests of the built `search-shadow` binary:
//! it is started through a symlink named `rg`,
//! with a fake "real" `rg` script on PATH,
//! exactly as agents will meet it.
//! Why:
//! The unit tests prove the pieces.
//! Only a real process shows that the priority is applied and the exit codes are right
//! and the arguments arrive untouched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.integration.test.ts
//! const result = spawnSync(path.join(shadowDirectory, 'rg'), ['foo'], { env: { PATH } });
//! ```

/// What:
/// `use std::fs;` imports the file-system module (write scripts / make links).
/// Why:
/// The fixtures are files in temporary directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fs from 'node:fs';
/// ```
use std::fs;

/// What:
/// `use std::ffi::OsString;` imports the owned operating-system text,
/// and `use std::os::unix::ffi::OsStringExt;` imports a trait that adds `OsString::from_vec`
/// (which builds one from raw bytes).
/// Why:
/// One test sends an argument that is not valid UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = Buffer.from([0xff]);
/// ```
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

/// Imports the trait that adds `from_mode` to permissions (marks a script executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed path type.
use std::path::Path;

/// Imports `Command` (start a program) and `Output` (its result)
/// and `Stdio` (what its standard input is attached to).
use std::process::{Command, Output, Stdio};

/// What:
/// `const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_search-shadow");` pastes in at compile time
/// the path of the freshly built binary.
/// `env!` is a macro (the `!`) that reads a build-time variable Cargo sets for integration tests.
/// Why:
/// The tests must run the real binary and not a stand-in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHADOW_EXECUTABLE = path.resolve('target/release/search-shadow');
/// ```
const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_search-shadow");

/// What:
/// `fn write_script(directory: &Path, name: &str, body: &str)` declares a helper that writes an executable shell
/// script into `directory`.
/// Why:
/// The fake "real" tool is a script that prints its arguments and its own disk priority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeScript(directory: string, name: string, body: string): void { ... }
/// ```
fn write_script(directory: &Path, name: &str, body: &str) {
    // `.expect("...")` unwraps a `Result` and stops the test with that message on error.
    fs::write(directory.join(name), format!("#!/bin/sh\n{body}\n")).expect("write the script");
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o755)).expect("make the script executable");
}

/// A fake real `rg` that prints its arguments and the disk priority it runs with.
const REAL_RG_BODY: &str = "printf 'REAL %s\\n' \"$*\"\nionice -p $$";

/// What:
/// `fn make_world(...)` declares a helper that builds the two PATH directories:
/// one holding the `rg` shadow link and one holding the fake real `rg`.
/// It returns the temporary directory (kept alive by the caller so it is not deleted early)
/// and the PATH text.
/// Why:
/// Every test starts from the same arrangement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function makeWorld(): { root: string; pathVariable: string } { ... }
/// ```
fn make_world() -> (tempfile::TempDir, String) {
    let root = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
    let shadow_directory = root.path().join("shadow");
    let real_directory = root.path().join("real");
    fs::create_dir(&shadow_directory).expect("make the shadow directory");
    fs::create_dir(&real_directory).expect("make the real directory");
    // Shadow links and fake real tools for the three tools the tests drive; grep and find print which tool answered.
    for name in ["rg", "grep", "find"] {
        std::os::unix::fs::symlink(SHADOW_EXECUTABLE, shadow_directory.join(name)).expect("make the shadow link");
    }
    write_script(&real_directory, "rg", REAL_RG_BODY);
    write_script(&real_directory, "grep", "printf 'REAL grep %s\\n' \"$*\"");
    write_script(&real_directory, "find", "printf 'REAL find %s\\n' \"$*\"");
    let path_variable = format!("{}:{}:/usr/bin:/bin", shadow_directory.display(), real_directory.display());
    return (root, path_variable);
}

/// What:
/// `fn run_rg(...) -> Output` declares a helper that runs the shadow link
/// with the given arguments,
/// working directory,
/// PATH and extra environment,
/// with standard input attached to nothing.
/// Why:
/// Standard input must not be a pipe,
/// or the policy would take the run for a piped search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runRg(root, args, cwd, pathVariable, env): SpawnSyncReturns<Buffer> { ... }
/// ```
fn run_rg(root: &Path, arguments: &[OsString], working_directory: &Path, path_variable: &str, extra_env: &[(&str, &str)]) -> Output {
    return run_tool(root, "rg", arguments, working_directory, path_variable, extra_env);
}

/// What:
/// `fn run_tool(...) -> Output` declares the general helper:
/// it runs the shadow link with the given tool name.
/// Why:
/// The `grep` and `find` tests use the same arrangement as `rg` under a different name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTool(root, name, args, cwd, pathVariable, env): SpawnSyncReturns<Buffer> { ... }
/// ```
fn run_tool(root: &Path, name: &str, arguments: &[OsString], working_directory: &Path, path_variable: &str, extra_env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(root.join("shadow").join(name));
    command.args(arguments);
    command.current_dir(working_directory);
    command.env_clear();
    command.env("PATH", path_variable);
    for (key, value) in extra_env {
        command.env(key, value);
    }
    command.stdin(Stdio::null());
    return command.output().expect("run the shadow");
}

/// What:
/// `fn texts(items: &[&str]) -> Vec<OsString>` declares a helper turning borrowed text slices into owned
/// operating-system texts.
/// Why:
/// `run_rg` takes arguments in the shape the process API wants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = (items: string[]) => items;
/// ```
fn texts(items: &[&str]) -> Vec<OsString> {
    let mut owned: Vec<OsString> = Vec::new();
    for item in items {
        owned.push(OsString::from(*item));
    }
    return owned;
}

/// What:
/// `#[test]` marks the next function as a test case.
/// Why:
/// The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('runs the real tool with the arguments and at the lowest best-effort priority', () => { ... });
/// ```
#[test]
fn runs_the_real_tool_with_the_arguments_at_the_lowest_best_effort_priority() {
    let (root, path_variable) = make_world();
    let output = run_rg(root.path(), &texts(&["foo", "bar"]), root.path(), &path_variable, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // `assert!(condition, "message")` stops the test with the message when the condition is false (a macro).
    assert!(output.status.success(), "the shadow failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("REAL foo bar"), "arguments did not arrive: {stdout}");
    // Only assert the class where `ionice` exists, since the shim runs the tool at normal priority without it.
    if Path::new("/usr/bin/ionice").exists() {
        assert!(stdout.contains("best-effort: prio 7"), "the tool did not run at best-effort level 7: {stdout}");
    }
}

/// Positive control for the test above:
/// the same script run directly does not report level 7.
#[test]
fn the_fake_tool_alone_does_not_run_at_level_seven() {
    if !Path::new("/usr/bin/ionice").exists() {
        return;
    }
    let (root, _path_variable) = make_world();
    let direct = Command::new(root.path().join("real").join("rg")).env("PATH", "/usr/bin:/bin").output().expect("run the script");
    assert!(!String::from_utf8_lossy(&direct.stdout).contains("prio 7"));
}

/// The policy refusal:
/// a wide walk with no path at a repository root exits 2 with the reason on stderr.
#[test]
fn refuses_a_wide_walk_with_no_path_at_a_repository_root() {
    let (root, path_variable) = make_world();
    let repository = root.path().join("repository");
    fs::create_dir_all(repository.join(".git")).expect("make a repository root");

    let output = run_rg(root.path(), &texts(&["--hidden", "foo"]), &repository, &path_variable, &[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("refused"));
    assert!(output.stdout.is_empty(), "the real tool must not have run");
}

/// The `-r` ban:
/// ripgrep's `-r` is `--replace`,
/// so the shadow refuses it anywhere with no repository needed.
#[test]
fn refuses_the_short_r_flag_with_an_explanation() {
    let (root, path_variable) = make_world();

    let output = run_rg(root.path(), &texts(&["-rl", "foo"]), root.path(), &path_variable, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("--replace"), "{stderr}");
    assert!(output.stdout.is_empty(), "the real tool must not have run");

    // The long form is explicit and must still run.
    let long_form = run_rg(root.path(), &texts(&["--replace", "X", "foo"]), root.path(), &path_variable, &[]);
    assert!(String::from_utf8_lossy(&long_form.stdout).contains("REAL --replace X foo"));
}

/// A named path bounds the walk,
/// so it runs.
#[test]
fn runs_a_wide_search_when_a_path_is_named() {
    let (root, path_variable) = make_world();
    let repository = root.path().join("repository");
    fs::create_dir_all(repository.join(".git")).expect("make a repository root");

    let output = run_rg(root.path(), &texts(&["--hidden", "foo", ".github"]), &repository, &path_variable, &[]);

    assert!(String::from_utf8_lossy(&output.stdout).contains("REAL --hidden foo .github"));
}

/// The escape hatch lets one deliberate full walk through.
#[test]
fn the_escape_variable_lets_a_full_walk_through() {
    let (root, path_variable) = make_world();
    let repository = root.path().join("repository");
    fs::create_dir_all(repository.join(".git")).expect("make a repository root");

    let output = run_rg(root.path(), &texts(&["--hidden", "foo"]), &repository, &path_variable, &[("SEARCH_SHADOW_ALLOW_WIDE", "1")]);

    assert!(String::from_utf8_lossy(&output.stdout).contains("REAL --hidden foo"));
}

/// With no real tool anywhere on PATH,
/// the shadow says so and exits 127.
#[test]
fn exits_127_when_the_real_tool_is_missing() {
    let (root, _path_variable) = make_world();
    let only_shadow = format!("{}", root.path().join("shadow").display());

    let output = run_rg(root.path(), &texts(&["foo"]), root.path(), &only_shadow, &[]);

    assert_eq!(output.status.code(), Some(127));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no real `rg`"));
}

/// An argument that is not valid UTF-8 must reach the real tool byte for byte.
#[test]
fn passes_a_non_utf8_argument_through_unchanged() {
    let (root, path_variable) = make_world();
    // A script that prints the bytes of its first argument as hex.
    write_script(&root.path().join("real"), "rg", "printf '%s' \"$1\" | od -An -tx1");
    let arguments: Vec<OsString> = vec![OsString::from_vec(vec![0xff, 0x61])];

    let output = run_rg(root.path(), &arguments, root.path(), &path_variable, &[]);

    assert!(String::from_utf8_lossy(&output.stdout).contains("ff 61"));
}

/// `search-shadow link DIRECTORY` links only the tools that exist on PATH and reports the rest,
/// and `--help` describes the command.
#[test]
fn the_link_subcommand_links_only_present_tools_and_help_mentions_it() {
    let root = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
    let links_directory = root.path().join("bin");
    let (world, _path_variable) = make_world();
    // PATH holds the fake real tools of the world (rg, grep, find) and nothing else.
    let only_fakes = world.path().join("real").to_string_lossy().to_string();

    let linked = Command::new(SHADOW_EXECUTABLE).arg("link").arg(&links_directory).env_clear().env("PATH", &only_fakes).output().expect("run link");
    let stdout = String::from_utf8_lossy(&linked.stdout).to_string();

    assert!(linked.status.success(), "{}", String::from_utf8_lossy(&linked.stderr));
    for name in ["rg", "grep", "find"] {
        assert!(links_directory.join(name).symlink_metadata().is_ok(), "{name} link missing");
    }
    // Tools without a real binary get no link, so `command -v` stays honest.
    for name in ["ag", "ack", "ugrep", "fdfind", "bfs", "ncdu", "cloc"] {
        assert!(links_directory.join(name).symlink_metadata().is_err(), "{name} must not be linked");
    }
    assert!(stdout.contains("linked"), "{stdout}");
    assert!(stdout.contains("skipped, no real tool on PATH"), "{stdout}");

    let help = Command::new(SHADOW_EXECUTABLE).arg("--help").output().expect("run help");
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("link"));
}

/// A recursive grep from a repository root is refused through the real process,
/// and a named subdirectory runs.
#[test]
fn refuses_a_recursive_grep_from_a_repository_root_but_runs_a_bounded_one() {
    let (root, path_variable) = make_world();
    let repository = root.path().join("repository");
    fs::create_dir_all(repository.join(".git")).expect("make a repository root");
    fs::create_dir_all(repository.join("package")).expect("make a subdirectory");

    let refused = run_tool(root.path(), "grep", &texts(&["-rn", "foo"]), &repository, &path_variable, &[("HOME", "/nonexistent")]);
    let allowed = run_tool(root.path(), "grep", &texts(&["-rn", "foo", "package"]), &repository, &path_variable, &[("HOME", "/nonexistent")]);

    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("a whole repository"));
    assert!(refused.stdout.is_empty(), "the real tool must not have run");
    assert!(String::from_utf8_lossy(&allowed.stdout).contains("REAL grep -rn foo package"));
}

/// An unbounded find from a repository root is refused,
/// and the same find with a small depth limit runs.
#[test]
fn refuses_an_unbounded_find_from_a_repository_root_but_runs_a_bounded_one() {
    let (root, path_variable) = make_world();
    let repository = root.path().join("repository");
    fs::create_dir_all(repository.join(".git")).expect("make a repository root");

    let refused = run_tool(root.path(), "find", &texts(&[".", "-name", "x"]), &repository, &path_variable, &[("HOME", "/nonexistent")]);
    let allowed = run_tool(root.path(), "find", &texts(&[".", "-maxdepth", "2", "-name", "x"]), &repository, &path_variable, &[("HOME", "/nonexistent")]);
    let escaped = run_tool(root.path(), "find", &texts(&[".", "-name", "x"]), &repository, &path_variable, &[("HOME", "/nonexistent"), ("SEARCH_SHADOW_ALLOW_WIDE", "1")]);

    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("-maxdepth 3"));
    assert!(String::from_utf8_lossy(&allowed.stdout).contains("REAL find . -maxdepth 2 -name x"));
    assert!(String::from_utf8_lossy(&escaped.stdout).contains("REAL find . -name x"));
}
