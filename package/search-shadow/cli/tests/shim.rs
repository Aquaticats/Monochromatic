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
/// `use search_shadow::constants::SHADOWED_NAMES;` imports the names the `link` subcommand shadows,
/// the same declaration that subcommand iterates.
/// Why:
/// The link assertions must cover every shadowed name,
/// so a name added to the source list cannot silently escape the test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SHADOWED_NAMES } from 'search_shadow/constants';
/// ```
use search_shadow::constants::SHADOWED_NAMES;
/// Imports the output cap, the exit code of a run it stopped, and the exit code of a run whose output could not be passed on.
use search_shadow::constants::{CAPPED_EXIT_CODE, MAX_OUTPUT_BYTES, OUTPUT_FAILED_EXIT_CODE};
/// Imports the exit code for a missing real tool and the priority level the shim uses.
use search_shadow::constants::NOT_FOUND_EXIT_CODE;
use search_shadow::priority::LOWEST_BEST_EFFORT_LEVEL;

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
/// `fn script_printing(bytes: u64) -> String` returns a fixture body that writes that many `a` bytes on standard output.
/// Why:
/// The cap turns on an exact byte count,
/// so a case must choose the count rather than a size class,
/// and `a` bytes survive any terminal between the tool and the test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scriptPrinting = (bytes: number) => `head --bytes=${bytes} /dev/zero | tr '\\0' a`;
/// ```
fn script_printing(bytes: u64) -> String {
    return format!("head --bytes={bytes} /dev/zero | tr '\\0' a");
}

/// What:
/// `fn host_path() -> String` returns this test process's own PATH.
/// Why:
/// The case that needs a terminal asks `script` (util-linux) for one,
/// and `script` is looked up on the child's PATH,
/// so that PATH holds the shadow directories first and this machine's own PATH after them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hostPath = () => process.env.PATH ?? '';
/// ```
fn host_path() -> String {
    return std::env::var("PATH").unwrap_or_default();
}

/// What:
/// `fn run_rg_with_piped_stdin(...) -> Output` declares a helper that runs the `rg` shadow link with standard input
/// attached to a pipe.
/// Why:
/// The lost-input rule turns on a pipe exactly,
/// so the end-to-end case needs a real one.
/// `Command::output` closes the write end at once,
/// so the child sees a pipe at its end without any writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runRgWithPipedStdin(root, args, cwd, pathVariable): SpawnSyncReturns<Buffer> { ... }
/// ```
fn run_rg_with_piped_stdin(root: &Path, arguments: &[OsString], working_directory: &Path, path_variable: &str) -> Output {
    let mut command = Command::new(root.join("shadow").join("rg"));
    command.args(arguments);
    command.current_dir(working_directory);
    command.env_clear();
    command.env("PATH", path_variable);
    command.stdin(Stdio::piped());
    return command.output().expect("run the shadow");
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
        let expected = format!("best-effort: prio {LOWEST_BEST_EFFORT_LEVEL}");
        assert!(stdout.contains(&expected), "the tool did not run at {expected}: {stdout}");
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
    let seventh = format!("prio {LOWEST_BEST_EFFORT_LEVEL}");
    assert!(!String::from_utf8_lossy(&direct.stdout).contains(&seventh));
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

    assert_eq!(output.status.code(), Some(NOT_FOUND_EXIT_CODE));
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
    let real_directory = world.path().join("real");
    let only_fakes = real_directory.to_string_lossy().to_string();

    let linked = Command::new(SHADOW_EXECUTABLE).arg("link").arg(&links_directory).env_clear().env("PATH", &only_fakes).output().expect("run link");
    let stdout = String::from_utf8_lossy(&linked.stdout).to_string();

    assert!(linked.status.success(), "{}", String::from_utf8_lossy(&linked.stderr));
    // A tool without a real binary gets no link, so `command -v` stays honest. Both sets come from
    // the source list and the fake world's real tools, so every shadowed name gets one assertion.
    for name in SHADOWED_NAMES {
        let link = links_directory.join(name);
        if real_directory.join(name).exists() {
            assert!(link.symlink_metadata().is_ok(), "{name} link missing although the fake world has its real tool");
        } else {
            assert!(link.symlink_metadata().is_err(), "{name} must not be linked when no real tool is on PATH");
        }
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

/// The incident:
/// a shell pipeline into a search that names a path.
/// ripgrep never reads the pipe there,
/// so the shadow refuses through the real process and the pipe's lines are not silently dropped.
#[test]
fn refuses_a_shell_pipeline_into_a_search_that_names_a_path() {
    let (root, path_variable) = make_world();
    let shadow = root.path().join("shadow").join("rg");
    // The pipeline shape of 2026-11-01 (`find ... | rg --invert-match X .`), reduced to its decision. The shadow's path
    // arrives as a positional (`$1`) and is quoted where it is used, so a directory with a space or a metacharacter
    // cannot change the command.
    let script = "printf 'a\\nb\\n' | \"$1\" --invert-match X .";

    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg("pipeline")
        .arg(&shadow)
        .current_dir(root.path())
        .env_clear()
        .env("PATH", &path_variable)
        .output()
        .expect("run the pipeline");

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("refused"), "{stderr}");
    assert!(stderr.contains("standard input is a pipe"), "{stderr}");
    assert!(output.stdout.is_empty(), "the real tool must not have run");
}

/// The same pipe with no search path runs:
/// there the pipe is the input.
#[test]
fn runs_a_piped_search_that_names_no_path() {
    let (root, path_variable) = make_world();

    let output = run_rg_with_piped_stdin(root.path(), &texts(&["X"]), root.path(), &path_variable);

    assert!(output.status.success(), "the shadow failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("REAL X"), "the real tool did not run");
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

/// The cap, on the exact byte count:
/// an output of exactly the cap is passed on whole,
/// and one byte more stops the real tool there and says so.
#[test]
fn passes_an_output_of_the_cap_whole_and_stops_one_byte_past_it() {
    let (root, path_variable) = make_world();
    let real_rg = root.path().join("real").join("rg");

    fs::write(&real_rg, format!("#!/bin/sh\n{}\n", script_printing(MAX_OUTPUT_BYTES))).expect("write the fixture");
    let whole = run_rg(root.path(), &texts(&["X", "."]), root.path(), &path_variable, &[]);
    assert_eq!(whole.stdout.len() as u64, MAX_OUTPUT_BYTES, "an output of exactly the cap is not past it");
    assert_eq!(whole.status.code(), Some(0), "{}", String::from_utf8_lossy(&whole.stderr));
    assert!(whole.stderr.is_empty(), "a complete output needs no line: {}", String::from_utf8_lossy(&whole.stderr));

    fs::write(&real_rg, format!("#!/bin/sh\n{}\n", script_printing(MAX_OUTPUT_BYTES + 1))).expect("write the fixture");
    let stopped = run_rg(root.path(), &texts(&["X", "."]), root.path(), &path_variable, &[]);
    let stderr = String::from_utf8_lossy(&stopped.stderr).to_string();
    assert_eq!(stopped.stdout.len() as u64, MAX_OUTPUT_BYTES, "the shadow passes the head and no more");
    assert_eq!(stopped.status.code(), Some(CAPPED_EXIT_CODE), "{stderr}");
    assert!(stderr.contains("search-shadow: stopped"), "{stderr}");
    assert!(stderr.contains(&MAX_OUTPUT_BYTES.to_string()), "the line must name the cap: {stderr}");
    assert!(stderr.contains("X ."), "the line must name the command: {stderr}");
    assert!(stderr.contains(&real_rg.display().to_string()), "the line must name the real tool to read it all: {stderr}");
}

/// The cap belongs to the shadow and not to ripgrep:
/// any shadowed tool that prints past it is stopped the same way.
#[test]
fn stops_any_shadowed_tool_that_prints_past_the_cap() {
    let (root, path_variable) = make_world();
    write_script(&root.path().join("real"), "find", &script_printing(MAX_OUTPUT_BYTES * 2));

    let output = run_tool(root.path(), "find", &texts(&[".", "-maxdepth", "1", "-name", "x"]), root.path(), &path_variable, &[]);

    assert_eq!(output.stdout.len() as u64, MAX_OUTPUT_BYTES, "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.status.code(), Some(CAPPED_EXIT_CODE));
    assert!(String::from_utf8_lossy(&output.stderr).contains("search-shadow: stopped"));
}

/// A terminal shows the whole output:
/// the cap is for an output nothing is watching,
/// so with a terminal on standard output the shadow still replaces itself and prints everything.
#[test]
fn does_not_cap_an_output_a_terminal_shows() {
    // `script` (util-linux) is what gives the shadow a terminal; without it this case cannot be shown at all.
    if Command::new("script").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_err() {
        return;
    }
    let (root, path_variable) = make_world();
    let past_the_cap: u64 = MAX_OUTPUT_BYTES + 4096;
    write_script(&root.path().join("real"), "rg", &script_printing(past_the_cap));
    let mut command = Command::new("script");
    command.arg("--quiet").arg("--return").arg("--command").arg("exec \"$SHADOW_RG\" X .").arg("/dev/null");
    command.current_dir(root.path());
    command.env_clear();
    // The shadow's own directories first, then this machine's PATH so `script` itself is found.
    command.env("PATH", format!("{path_variable}:{}", host_path()));
    command.env("SHADOW_RG", root.path().join("shadow").join("rg"));
    command.stdin(Stdio::null());

    let output = command.output().expect("run the shadow behind a terminal");

    assert_eq!(output.stdout.len() as u64, past_the_cap, "a terminal's output is not capped: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "no line is added to a complete output: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.status.code(), Some(0), "the real tool's own status is kept");
}

/// An output that cannot be passed on (a full disk) must not pass for a complete one:
/// the run fails and says why.
#[test]
fn fails_and_says_why_when_the_output_cannot_be_passed_on() {
    let (root, path_variable) = make_world();
    let full = fs::OpenOptions::new().write(true).open("/dev/full").expect("open /dev/full");
    let mut command = Command::new(root.path().join("shadow").join("rg"));
    command.args(texts(&["X", "."]));
    command.current_dir(root.path());
    command.env_clear();
    command.env("PATH", &path_variable);
    command.stdin(Stdio::null());
    command.stdout(Stdio::from(full));
    command.stderr(Stdio::piped());

    let child = command.spawn().expect("run the shadow");
    let output = child.wait_with_output().expect("wait for the shadow");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert_eq!(output.status.code(), Some(OUTPUT_FAILED_EXIT_CODE), "{stderr}");
    assert!(stderr.contains("search-shadow:"), "{stderr}");
    assert!(stderr.contains("No space left on device"), "the line must name what the output failed on: {stderr}");
}

/// The lost-pipe refusal has no override:
/// naming the wide-walk variable does not lift it through the real process either.
#[test]
fn the_wide_walk_variable_does_not_lift_the_lost_pipe_refusal() {
    let (root, path_variable) = make_world();
    let shadow = root.path().join("shadow").join("rg");
    let script = "printf 'a\\nb\\n' | \"$1\" --invert-match X .";

    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg("pipeline")
        .arg(&shadow)
        .current_dir(root.path())
        .env_clear()
        .env("PATH", &path_variable)
        .env("SEARCH_SHADOW_ALLOW_WIDE", "1")
        .output()
        .expect("run the pipeline");

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("standard input is a pipe"), "{stderr}");
    assert!(output.stdout.is_empty(), "the real tool must not have run");
}
