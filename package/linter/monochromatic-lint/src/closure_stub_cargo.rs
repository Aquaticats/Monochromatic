//! What:
//!  A stand-in for `cargo` that the tests of the closure check point `MONOCHROMATIC_LINT_CARGO` at.
//! Why:
//!  The linter's own tests run in a container without network,
//! the nightly toolchain or `cargo-dylint`,
//!  so the orchestration is tested against canned answers.
//!  The program is compiled by the tests (`closure_stub_support.rs`) and never part of the linter.
//!  The scenario directory is the directory that holds the program itself,
//! a link per test.
//!  For the subcommand `metadata`,
//! `dylint`,
//! `dylint-version` (`dylint --version`) or `ndk-env`,
//! the files
//!  `<subcommand>.<package>.stdout`,
//! `.stderr` and `.status` answer for the package whose directory name follows
//!  `--manifest-path`,
//! and `<subcommand>.stdout`,
//! `.stderr` and `.status` answer for every other package.
//!  Every call appends one line to `calls.log`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // fake-cargo <subcommand> ...: print the scenario's stdout and stderr files and exit with its status file.
//! ```

/// Import the environment,
/// files and process exit.
use std::{
    fs::{OpenOptions, read_to_string},
    io::Write,
    path::{Path, PathBuf},
};

/// What:
///  The text of a scenario file for a subcommand and package,
///  falling back to the file that is not specific to a package.
/// Why:
///  A scenario that needs different answers per package names them;
/// the rest share one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function answer(directory: Path, subcommand: string, package: string, part: string): string | undefined;
/// ```
fn answer(directory: &Path, subcommand: &str, package: &str, part: &str) -> Option<String> {
    let specific: PathBuf = directory.join(format!("{subcommand}.{package}.{part}"));
    if let Ok(text) = read_to_string(&specific) {
        return Some(text);
    }
    return read_to_string(directory.join(format!("{subcommand}.{part}"))).ok();
}

/// What:
///  The directory name of the package named by `--manifest-path`.
/// Why:
///  It selects the package-specific scenario files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packageName(args: string[]): string;
/// ```
fn package_name(arguments: &[String]) -> String {
    let mut found: String = String::new();
    for (index, argument) in arguments.iter().enumerate() {
        if argument == "--manifest-path"
            && let Some(manifest) = arguments.get(index + 1)
            && let Some(name) = Path::new(manifest).parent().and_then(Path::file_name)
        {
            found = name.to_string_lossy().into_owned();
        }
    }
    return found;
}

/// Answer one cargo invocation from the scenario directory.
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect::<Vec<String>>();
    let program: PathBuf = std::env::current_exe().expect("the program path is known");
    let directory: PathBuf = program.parent().expect("the program has a directory").to_path_buf();
    let subcommand: String = match (arguments.first(), arguments.get(1)) {
        (Some(first), Some(second)) if first == "dylint" && second == "--version" => {
            String::from("dylint-version")
        }
        (Some(first), _) => first.clone(),
        (None, _) => String::new(),
    };
    let package: String = package_name(arguments.as_slice());
    let mut environment: String = String::new();
    // Only the real run is asked about its environment: a probe inherits the test runner's variables (the gate
    // container exports CARGO_NET_OFFLINE for every process), which says nothing about the code under test.
    let observed: &[&str] = if subcommand == "dylint" {
        &[
            "DYLINT_RUSTFLAGS",
            "CARGO_NET_OFFLINE",
            "CARGO_BUILD_BUILD_DIR",
            "QMAKE",
            "STUB_MARKER",
        ]
    } else {
        &[]
    };
    for name in observed {
        if let Ok(value) = std::env::var(name) {
            environment.push_str(format!(" {name}={value}").as_str());
        }
    }
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("calls.log"))
        .expect("the call log opens");
    writeln!(log, "{subcommand}|{}|{environment}", arguments.join(" ")).expect("the call log is written");
    let default_stdout: &str = if subcommand == "metadata" { "{}" } else { "" };
    let stdout: String =
        answer(&directory, &subcommand, &package, "stdout").unwrap_or(String::from(default_stdout));
    let stderr: String = answer(&directory, &subcommand, &package, "stderr").unwrap_or_default();
    let status: i32 = match answer(&directory, &subcommand, &package, "status") {
        Some(text) => text.trim().parse::<i32>().expect("the status file holds a number"),
        None => 0,
    };
    print!("{stdout}");
    eprint!("{stderr}");
    std::process::exit(status);
}
