//! What:
//!     End-to-end tests of the version and help announcements:
//! the built `search-shadow` binary is started through a
//!           symlink named after each shadowed tool,
//! with a fixture executable standing in for the real tool.
//! Why:
//!      The announcement runs the real tool as a child and appends text,
//! so only a real process shows that the real
//!           tool's output is byte-identical,
//! the added text is exactly one line or section,
//! the exit status passes
//!           through,
//! other argument lists are untouched and the refusal policies still come first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // announce.integration.test.ts
//! const result = spawnSync(path.join(shadowDirectory, 'rg'), ['--version'], { env: { PATH } });
//! ```

/// Imports the file-system module (write scripts,
/// make links).
use std::fs;

/// Imports the owned operating-system text and the trait that builds one from raw bytes.
use std::ffi::OsString;

/// Imports the trait that adds `from_mode` to permissions (marks a script executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed path type.
use std::path::Path;

/// Imports `Command` (start a program),
/// `Output` (its result) and `Stdio` (what its standard streams attach to).
use std::process::{Command, Output, Stdio};

/// The path of the freshly built binary,
/// pasted in at compile time by Cargo for integration tests.
const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_search-shadow");

/// The version the shadow reports,
/// pasted in at compile time.
const SHADOW_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Every shadowed name with the version flags and help flags that must be announced,
/// written out independently of the
/// crate's own table (read off each real tool's CLI).
const TOOLS: &[(&str, &[&str], &[&str])] = &[
    ("rg", &["--version", "-V"], &["--help", "-h"]),
    ("grep", &["--version", "-V"], &["--help"]),
    ("egrep", &["--version", "-V"], &["--help"]),
    ("fgrep", &["--version", "-V"], &["--help"]),
    ("ugrep", &["--version", "-V"], &["--help", "-?"]),
    ("ug", &["--version", "-V"], &["--help", "-?"]),
    ("ag", &["--version", "-V"], &["--help", "-h"]),
    ("ack", &["--version"], &["--help"]),
    ("find", &["--version", "-version"], &["--help", "-help"]),
    ("fd", &["--version", "-V"], &["--help", "-h"]),
    ("fdfind", &["--version", "-V"], &["--help", "-h"]),
    ("bfs", &["--version", "-version"], &["--help", "-help"]),
    ("tree", &["--version"], &["--help"]),
    ("du", &["--version"], &["--help"]),
    ("ncdu", &["--version", "-V"], &["--help", "-h", "-?"]),
    ("tokei", &["--version", "-V"], &["--help", "-h"]),
    ("cloc", &["--version"], &["--help"]),
];

/// What:
///     `fn write_script(directory: &Path, name: &str, body: &str)` writes an executable shell script.
/// Why:
///      The fixture "real" tools are scripts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeScript(directory: string, name: string, body: string): void { ... }
/// ```
fn write_script(directory: &Path, name: &str, body: &str) {
    fs::write(directory.join(name), format!("#!/bin/sh\n{body}\n")).expect("write the script");
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o755)).expect("make the script executable");
}

/// What:
///     `fn real_body(name: &str) -> String` returns the fixture's script body:
/// two fixed stdout lines that include
///           the arguments it got,
/// one stderr line,
/// and exit status 3.
/// Why:
///      Status 3 proves the exit code passes through;
/// the stderr line proves stderr is untouched;
/// the argument echo
///           proves the arguments arrive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const realBody = (name: string) => `...`;
/// ```
fn real_body(name: &str) -> String {
    return format!("printf 'REAL {name} args=[%s]\\nsecond line\\n' \"$*\"\nprintf 'fixture stderr\\n' >&2\nexit 3");
}

/// What:
///     `fn make_world() -> tempfile::TempDir` builds a scratch directory holding `shadow/` (one symlink per shadowed
///           name) and `real/` (one fixture script per name),
/// and returns it.
/// Why:
///      Every test starts from the same arrangement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function makeWorld(): string { ... }
/// ```
fn make_world() -> tempfile::TempDir {
    let root = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
    let shadow_directory = root.path().join("shadow");
    let real_directory = root.path().join("real");
    fs::create_dir(&shadow_directory).expect("make the shadow directory");
    fs::create_dir(&real_directory).expect("make the real directory");
    for (name, _version, _help) in TOOLS {
        std::os::unix::fs::symlink(SHADOW_EXECUTABLE, shadow_directory.join(name)).expect("make the shadow link");
        write_script(&real_directory, name, &real_body(name));
    }
    return root;
}

/// What:
///     `fn path_variable(root: &Path) -> String` returns the PATH text:
/// the shadow directory,
/// then the real one.
/// Why:
///      The shadow must skip its own links and find the fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathVariable = (root: string) => `${root}/shadow:${root}/real:/usr/bin:/bin`;
/// ```
fn path_variable(root: &Path) -> String {
    return format!("{}:{}:/usr/bin:/bin", root.join("shadow").display(), root.join("real").display());
}

/// What:
///     `fn run_tool(...) -> Output` runs the shadow link of one tool with the given arguments,
/// standard input
///           attached to nothing and the working directory given.
/// Why:
///      Standard input must not be a pipe,
/// or the `rg` policy would take the run for a piped search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTool(root, name, args, cwd, env?): SpawnSyncReturns<Buffer> { ... }
/// ```
fn run_tool(root: &Path, name: &str, arguments: &[&str], working_directory: &Path, path_text: &str) -> Output {
    let mut command = Command::new(root.join("shadow").join(name));
    for argument in arguments {
        command.arg(OsString::from(*argument));
    }
    command.current_dir(working_directory);
    command.env_clear();
    command.env("PATH", path_text);
    command.stdin(Stdio::null());
    return command.output().expect("run the shadow");
}

/// What:
///     `fn real_stdout(name: &str, arguments: &[&str]) -> String` returns exactly what the fixture prints on
///           standard output for those arguments.
/// Why:
///      The expected standard output is the real tool's,
/// byte for byte,
/// plus the added text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const realStdout = (name: string, args: string[]) => `REAL ${name} args=[${args.join(' ')}]\nsecond line\n`;
/// ```
fn real_stdout(name: &str, arguments: &[&str]) -> String {
    return format!("REAL {name} args=[{}]\nsecond line\n", arguments.join(" "));
}

/// The header line the shadow adds:
/// the version line without its newline.
fn header(root: &Path, name: &str) -> String {
    return format!("search-shadow {SHADOW_VERSION}, shadowing {name} at {}", root.join("real").join(name).display());
}

/// What:
///     `#[test]` marks the next function as a test case.
/// Why:
///      The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('adds one version line after the real version output', () => { ... });
/// ```
#[test]
fn every_tool_prints_the_real_version_output_then_exactly_one_line() {
    let root = make_world();
    let path_text = path_variable(root.path());
    for (name, version_flags, _help) in TOOLS {
        for flag in *version_flags {
            let output = run_tool(root.path(), name, &[flag], root.path(), &path_text);
            let expected = format!("{}{}\n", real_stdout(name, &[flag]), header(root.path(), name));
            assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "{name} {flag}");
            assert_eq!(output.status.code(), Some(3), "{name} {flag} must keep the real tool's exit code");
            assert_eq!(String::from_utf8_lossy(&output.stderr), "fixture stderr\n", "{name} {flag} stderr must be untouched");
        }
    }
}

/// The help section,
/// written out in full for `rg`.
#[test]
fn rg_help_is_the_real_help_then_the_full_section() {
    let root = make_world();
    let path_text = path_variable(root.path());
    for flag in ["--help", "-h"] {
        let output = run_tool(root.path(), "rg", &[flag], root.path(), &path_text);
        let expected = format!(
            "{real}\n{header}\n\
             The help above is the real rg's own. search-shadow starts it at best-effort disk priority 7 and:\n\
             - refuses the short flag -r anywhere (in ripgrep it is --replace, not grep's recursive flag)\n\
             - refuses --hidden, --no-ignore*, -u or -. with no search path and no piped input from a git repository root, unless SEARCH_SHADOW_ALLOW_WIDE=1 is set for one deliberate walk\n\
             Run `search-shadow --help` for the shadow itself.\n",
            real = real_stdout("rg", &[flag]),
            header = header(root.path(), "rg"),
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "rg {flag}");
        assert_eq!(output.status.code(), Some(3));
    }
}

/// Every tool's help is the real output followed by a section that starts with a blank line and the header line.
#[test]
fn every_tool_prints_the_real_help_output_then_one_section_naming_the_shadow() {
    let root = make_world();
    let path_text = path_variable(root.path());
    for (name, _version, help_flags) in TOOLS {
        for flag in *help_flags {
            let output = run_tool(root.path(), name, &[flag], root.path(), &path_text);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let prefix = format!("{}\n{}\n", real_stdout(name, &[flag]), header(root.path(), name));
            assert!(stdout.starts_with(&prefix), "{name} {flag}: {stdout}");
            let rest = &stdout[prefix.len()..];
            assert!(rest.starts_with(&format!("The help above is the real {name}'s own.")), "{name} {flag}: {rest}");
            assert!(rest.ends_with("Run `search-shadow --help` for the shadow itself.\n"), "{name} {flag}: {rest}");
            assert_eq!(output.status.code(), Some(3), "{name} {flag} must keep the real tool's exit code");
            assert_eq!(String::from_utf8_lossy(&output.stderr), "fixture stderr\n", "{name} {flag}");
        }
    }
}

/// Anything but a lone version or help flag runs untouched:
/// the output is the real tool's alone.
#[test]
fn other_argument_lists_run_untouched() {
    let root = make_world();
    let path_text = path_variable(root.path());
    let cases: &[(&str, &[&str])] = &[
        ("rg", &["--version", "foo"]),
        ("rg", &["foo", "--version"]),
        ("rg", &["--no-config", "--help"]),
        ("rg", &["foo"]),
        ("rg", &[]),
        ("grep", &["-h"]),
        ("grep", &["-V", "x"]),
        ("du", &["-h"]),
        ("du", &["-V"]),
        ("tree", &["-h"]),
        ("tree", &["-V"]),
        ("find", &["-V"]),
        ("find", &[".", "--version"]),
        ("ack", &["-h"]),
        ("ugrep", &["-h"]),
        ("cloc", &["-h"]),
        ("fd", &["--versions"]),
    ];
    for (name, arguments) in cases {
        let output = run_tool(root.path(), name, arguments, root.path(), &path_text);
        assert_eq!(String::from_utf8_lossy(&output.stdout), real_stdout(name, arguments), "{name} {arguments:?}");
        assert_eq!(output.status.code(), Some(3));
    }
}

/// The refusal policies still fire before any of this:
/// `rg -rV` is refused and the real tool never runs.
#[test]
fn a_refused_run_is_refused_before_any_announcement() {
    let root = make_world();
    let path_text = path_variable(root.path());
    let output = run_tool(root.path(), "rg", &["-rV"], root.path(), &path_text);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "the real tool must not have run: {}", String::from_utf8_lossy(&output.stdout));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--replace"));
}

/// With no real tool on PATH the run still ends with 127 and prints nothing on standard output.
#[test]
fn a_missing_real_tool_still_exits_127_without_an_announcement() {
    let root = make_world();
    let only_shadow = root.path().join("shadow").display().to_string();
    let output = run_tool(root.path(), "rg", &["--version"], root.path(), &only_shadow);
    assert_eq!(output.status.code(), Some(127));
    assert!(output.stdout.is_empty());
}

/// A real tool that cannot be started ends with 126 and no announcement.
#[test]
fn a_real_tool_that_cannot_start_exits_126_without_an_announcement() {
    let root = make_world();
    let path_text = path_variable(root.path());
    fs::write(root.path().join("real").join("rg"), "#!/nonexistent/interpreter\n").expect("write a script with a bad interpreter");
    let output = run_tool(root.path(), "rg", &["--version"], root.path(), &path_text);
    assert_eq!(output.status.code(), Some(126));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("could not start"));
}

/// A real tool ended by a signal gives the shell's `128 + signal` exit code,
/// and the announcement is still added.
#[test]
fn a_signal_exit_is_reported_as_128_plus_the_signal() {
    let root = make_world();
    let path_text = path_variable(root.path());
    write_script(&root.path().join("real"), "rg", "printf 'before\\n'\nkill -TERM $$");
    let output = run_tool(root.path(), "rg", &["--version"], root.path(), &path_text);
    assert_eq!(output.status.code(), Some(143));
    assert_eq!(String::from_utf8_lossy(&output.stdout), format!("before\n{}\n", header(root.path(), "rg")));
}

/// A standard output that cannot be written to (`/dev/full`) must not turn the run into a panic or a different status.
#[test]
fn an_unwritable_standard_output_keeps_the_real_exit_code() {
    let root = make_world();
    let path_text = path_variable(root.path());
    let full = fs::OpenOptions::new().write(true).open("/dev/full").expect("open /dev/full");
    let status = Command::new(root.path().join("shadow").join("rg"))
        .arg("--version")
        .current_dir(root.path())
        .env_clear()
        .env("PATH", &path_text)
        .stdin(Stdio::null())
        .stdout(Stdio::from(full))
        .stderr(Stdio::null())
        .status()
        .expect("run the shadow");
    assert_eq!(status.code(), Some(3));
}

/// `search-shadow --version` and `--help` say plainly that it is a shadow,
/// and of which tools.
#[test]
fn the_search_shadow_command_says_it_is_a_shadow_of_the_listed_tools() {
    let version = Command::new(SHADOW_EXECUTABLE).arg("--version").env_clear().output().expect("run --version");
    let version_text = String::from_utf8_lossy(&version.stdout).to_string();
    assert!(version.status.success());
    assert!(version_text.starts_with(&format!("search-shadow {SHADOW_VERSION}, a shadow of rg, grep, ")), "{version_text}");
    for (name, _version, _help) in TOOLS {
        assert!(version_text.contains(name), "--version must name {name}: {version_text}");
    }

    let help = Command::new(SHADOW_EXECUTABLE).arg("--help").env_clear().output().expect("run --help");
    let help_text = String::from_utf8_lossy(&help.stdout).to_string();
    assert!(help.status.success());
    assert!(help_text.contains("search-shadow is a shadow"), "{help_text}");
    for (name, _version, _help) in TOOLS {
        assert!(help_text.contains(name), "--help must name {name}: {help_text}");
    }
}
