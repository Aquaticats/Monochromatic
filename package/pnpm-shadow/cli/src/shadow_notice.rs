//! What:
//! The one place the shadow says what it is:
//! a line after `pnpm --version` and a section after `pnpm --help`.
//! Why:
//! A shadow that prints only pnpm's text cannot be told apart from pnpm,
//! so a person asking for the version or the help
//! learns there that a wrapper is in the way and where the real pnpm is.
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
//! export function noticeText(request, realPnpm: string): string | undefined { ... }
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
    /// `pnpm --version` or `pnpm -v`.
    Version,
    /// `pnpm --help`,
    /// `pnpm -h` or `pnpm help`.
    Help,
    /// Any other command line,
    /// including a command's own help such as `pnpm install --help`.
    Other,
}

/// The words that ask for the version,
/// as pnpm 12.16.1 answers them (read by running `pnpm -v` and `pnpm --version`).
pub const VERSION_WORDS: [&str; 2] = ["--version", "-v"];

/// The words that ask for the top-level help,
/// as pnpm 12.16.1 answers them (read by running `pnpm help`,
/// `-h`,
/// `--help`).
pub const HELP_WORDS: [&str; 3] = ["--help", "-h", "help"];

/// What:
/// `pub fn top_level_request(arguments: &[OsString]) -> TopLevelRequest` classifies a command line.
/// Why:
/// Only a command line that is exactly one of those words is a top-level request.
/// `pnpm install --help` and
/// `pnpm help install` ask about a command,
/// and `pnpm --dir x --version` has an option in front;
/// each prints exactly what
/// pnpm printed,
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
/// `pub fn version_line(real_pnpm: &Path) -> String` is the line printed after pnpm's version.
/// Why:
/// It names the wrapper,
/// its version and the real pnpm it ran,
/// in the shape `hub` and `mpm-shadow` use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function versionLine(realPnpm: string): string { return `pnpm-shadow ${version}, shadowing pnpm at ${realPnpm}\n`; }
/// ```
pub fn version_line(real_pnpm: &Path) -> String {
    // `env!("CARGO_PKG_VERSION")` is the crate's version from `Cargo.toml`, read at compile time.
    return format!("pnpm-shadow {}, shadowing pnpm at {}\n", env!("CARGO_PKG_VERSION"), real_pnpm.display());
}

/// What:
/// `pub fn help_section(real_pnpm: &Path) -> String` is the section printed after pnpm's help.
/// Why:
/// It says what the wrapper does in two sentences,
/// after a blank line so it reads as separate from pnpm's text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function helpSection(realPnpm: string): string
/// ```
pub fn help_section(real_pnpm: &Path) -> String {
    return format!(
        "\n{}Before a command that adds, installs, updates or removes dependencies it warns on standard error about \
package.json dependency versions that are not `catalog:`, `catalog:<name>` or exactly `workspace:*`, then runs pnpm with \
the same arguments. Every other command goes straight to pnpm. It never blocks pnpm.\n",
        version_line(real_pnpm)
    );
}

/// What:
/// `pub fn notice_text(request: TopLevelRequest, real_pnpm: &Path) -> Option<String>` is the text to print on standard
/// output after a successful pnpm run,
/// or `None` when nothing is to be added.
/// Why:
/// One function decides,
/// so the caller and the tests agree on which requests get a notice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function noticeText(request: TopLevelRequest, realPnpm: string): string | undefined
/// ```
pub fn notice_text(request: TopLevelRequest, real_pnpm: &Path) -> Option<String> {
    match request {
        TopLevelRequest::Version => return Some(version_line(real_pnpm)),
        TopLevelRequest::Help => return Some(help_section(real_pnpm)),
        TopLevelRequest::Other => return None,
    }
}
