//! What:
//!  The `--help` and `--version` surface:
//!  both long and short spellings exit
//!  zero and write their text to stdout,
//!  with the program name and the usage block where callers look for them.
//! Why:
//!  Wrappers and package managers pipe these through pagers or parse the
//!  version line,
//!  so a moved channel or a changed exit code breaks them silently.
//!  Neither run needs a fixture directory,
//!  which is why this part needs no temporary paths.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // help-and-version.test.ts: spawn, read stdout, check the exit code
//! ```

use crate::{
    BIN,
    Command,
};

// What:     `#[test] fn help_long_flag_exits_zero_and_lists_usage()`.
//           `--help` must exit 0 (success) and print the usage block --
//           `USAGE:`, the binary name, and at least one flag header --
//           to stdout. Convention matches `cargo`, `rustc`, every
//           POSIX-shaped CLI.
// Why:      A regression that, say, accidentally redirected the help
//           text to stderr or returned exit 2 would break shells and
//           wrappers that pipe `--help` through a pager and assert on
//           the exit code. Pins both the channel and the exit shape.
#[test]
fn help_long_flag_exits_zero_and_lists_usage() {
    let output = Command::configured(BIN)
        .arg("--help")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "--help must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.contains("forbidden-strings"),
        "help missing program name; stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("USAGE:"),
        "help missing USAGE block; stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("--rules"),
        "help missing --rules flag; stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("--all"),
        "help missing --all flag; stdout: {}",
        stdout
    );
}

// What:     `-h` short alias must behave identically to `--help`.
// Why:      Argv parsing in main.rs is a manual while-loop. A typo or
//           dropped branch on `-h` would silently fall through to the
//           "unknown flag" arm and exit 2; this test pins the alias.
#[test]
fn help_short_flag_exits_zero_and_lists_usage() {
    let output = Command::configured(BIN)
        .arg("-h")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "-h must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.contains("USAGE:"),
        "short -h missing USAGE; stdout: {}",
        stdout
    );
}

// What:     `--version` exits 0 and prints `forbidden-strings <semver>`
//           on stdout. The semver baked in via `env!("CARGO_PKG_VERSION")`
//           is the version Cargo.toml declares for this crate.
// Why:      Tools (mise, asdf, package managers) parse `--version`
//           output to verify which build is installed. A regression
//           that swapped the channel or dropped the prefix would break
//           those tools silently.
#[test]
fn version_long_flag_exits_zero_and_prints_version_line() {
    let output = Command::configured(BIN)
        .arg("--version")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "--version must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.starts_with("forbidden-strings "),
        "version output must begin with program name; got: {}",
        stdout,
    );
    // Crude semver shape check: must contain at least one digit (the
    // major version). Avoids hardcoding the current version here, which
    // would force this test to update on every release.
    assert!(
        stdout.chars().any(|c| return c.is_ascii_digit()),
        "version output must contain at least one digit; got: {}",
        stdout,
    );
}

#[test]
fn version_short_flag_exits_zero_and_prints_version_line() {
    let output = Command::configured(BIN)
        .arg("-V")
        .output()
        .expect("spawn binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "-V must exit 0; stdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.starts_with("forbidden-strings "),
        "short -V must print version line; got: {}",
        stdout,
    );
}
