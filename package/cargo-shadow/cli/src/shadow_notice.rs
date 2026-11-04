//! What:
//! The one place the shadow says what it is:
//! a line after `cargo --version` and a section after `cargo --help`.
//! Why:
//! A shadow that prints only cargo's text cannot be told apart from cargo,
//! so a person asking for the version or the help
//! learns there that a wrapper is in the way and where the real cargo is.
//! Every other run stays unbranded:
//! the manifest
//! warnings name the manifests and the rule,
//! not the wrapper.
//! This is how `hub` and `mpm-shadow` do it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-notice.ts
//! export function topLevelRequest(args: string[]): 'version' | 'help' | 'other' { ... }
//! export function noticeText(request, realCargo: string): string | undefined { ... }
//! ```

/// Imports the owned operating-system text type:
/// arguments may be invalid UTF-8.
use std::ffi::OsString;

/// Imports the borrowed path type.
use std::path::Path;

/// What:
/// `pub enum TopLevelRequest { Version, Help, Other }` is a type with exactly three possible values (a TS union
/// `'version' | 'help' | 'other'`).
/// `#[derive(...)]` asks the compiler to write copying,
/// comparing and printing for it.
/// Why:
/// The caller matches on it,
/// and the compiler checks that every case is handled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type TopLevelRequest = 'version' | 'help' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopLevelRequest {
    /// `cargo --version` or `cargo -V`.
    Version,
    /// `cargo --help`,
    /// `cargo -h` or `cargo help`.
    Help,
    /// Any other command line,
    /// including a command's own help such as `cargo install --help`.
    Other,
}

/// The words that ask for the version,
/// as cargo 1.101.0-nightly answers them (read by running `cargo -V` and `cargo --version`).
pub const VERSION_WORDS: [&str; 2] = ["--version", "-V"];

/// The words that ask for the top-level help,
/// as cargo 1.101.0-nightly answers them (read by running `cargo help`,
/// `-h`,
/// `--help`).
pub const HELP_WORDS: [&str; 3] = ["--help", "-h", "help"];

/// What:
/// `pub fn top_level_request(arguments: &[OsString]) -> TopLevelRequest` classifies a command line.
/// Why:
/// Only a command line that is exactly one of those words is a top-level request.
/// `cargo build --help` and
/// `cargo help build` ask about a command,
/// and `cargo --color never --version` has an option in front;
/// each prints exactly what
/// cargo printed,
/// with no notice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function topLevelRequest(args: string[]): TopLevelRequest {
///   if (args.length !== 1) return 'other';
///   return VERSION_WORDS.includes(args[0]) ? 'version' : HELP_WORDS.includes(args[0]) ? 'help' : 'other';
/// }
/// ```
pub fn top_level_request(arguments: &[OsString]) -> TopLevelRequest {
    if arguments.len() != 1 {
        return TopLevelRequest::Other;
    }
    // `to_string_lossy()` borrows the argument as text, replacing invalid UTF-8 with a placeholder that matches no word.
    let word = arguments[0].to_string_lossy();
    for version_word in VERSION_WORDS {
        if word == version_word {
            return TopLevelRequest::Version;
        }
    }
    for help_word in HELP_WORDS {
        if word == help_word {
            return TopLevelRequest::Help;
        }
    }
    return TopLevelRequest::Other;
}

/// What:
/// `pub fn version_line(real_cargo: &Path) -> String` is the line printed after cargo's version.
/// Why:
/// It names the wrapper,
/// its version and the real cargo it ran,
/// in the shape `hub` and `mpm-shadow` use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function versionLine(realCargo: string): string { return `cargo-shadow ${version}, shadowing cargo at ${realCargo}\n`; }
/// ```
pub fn version_line(real_cargo: &Path) -> String {
    // `env!("CARGO_PKG_VERSION")` is the crate's version from `Cargo.toml`, read at compile time.
    return format!("cargo-shadow {}, shadowing cargo at {}\n", env!("CARGO_PKG_VERSION"), real_cargo.display());
}

/// What:
/// `pub fn help_section(real_cargo: &Path) -> String` is the section printed after cargo's help.
/// Why:
/// It says what the wrapper does,
/// after a blank line so it reads as separate from cargo's text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function helpSection(realCargo: string): string
/// ```
pub fn help_section(real_cargo: &Path) -> String {
    return format!(
        "\n{}Before a command that resolves dependencies it makes the workspace's Cargo.lock satisfy its manifests under the \
rust-catalog.toml trust policy (fails naming the stale lockfiles in CI or with --locked or --frozen), then runs cargo with the \
same arguments. After `cargo add` and `cargo remove` it warns about dependencies that do not use the workspace catalog. Every \
other command goes straight to cargo.\n",
        version_line(real_cargo)
    );
}

/// What:
/// `pub fn notice_text(request: TopLevelRequest, real_cargo: &Path) -> Option<String>` is the text to print on standard
/// output after a successful cargo run,
/// or `None` when nothing is to be added.
/// Why:
/// One function decides,
/// so the caller and the tests agree on which requests get a notice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function noticeText(request: TopLevelRequest, realCargo: string): string | undefined
/// ```
pub fn notice_text(request: TopLevelRequest, real_cargo: &Path) -> Option<String> {
    match request {
        TopLevelRequest::Version => return Some(version_line(real_cargo)),
        TopLevelRequest::Help => return Some(help_section(real_cargo)),
        TopLevelRequest::Other => return None,
    }
}
