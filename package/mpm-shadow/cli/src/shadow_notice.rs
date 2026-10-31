//! What:
//! Makes the shadow visible when it runs as `mpm`:
//! which top-level request of mpm's command line is a `--version` or a `--help`,
//! the one line added after mpm's version,
//! and the section added after mpm's help.
//! Why:
//! A shadow that prints only mpm's own text cannot be told from the real thing.
//! `hub` is the precedent:
//! `hub --version` runs `git version`,
//! then prints `hub version <X>`
//! (<https://github.com/mislav/hub/blob/master/commands/version.go>),
//! and `hub help` appends the commands hub adds to git's help.
//! mpm 8.2.0 spells these `--version` and `-h` / `--help`
//! (`mpm --help`,
//! sections "Other options" and "Introspection options");
//! it has no `-V`.
//!
//! Only options before the subcommand count,
//! the way mpm's own parser (click) reads them:
//! `mpm install --help` and `mpm help install` belong to the subcommand and pass through untouched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-notice.ts
//! export function topLevelRequest(args: string[]): 'version' | 'help' | 'other'
//! export function versionLine(realMpm: string): string
//! export function helpSection(realMpm: string): string
//! ```

/// Imports `OsString`.
use std::ffi::OsString;
/// Imports `Path`.
use std::path::Path;

/// The top-level options of mpm 8.2.0 that take a separate value
/// (read from `mpm --help`).
/// `--color` is not here:
/// its value is optional ("A bare --color is the same as --color=always").
const VALUE_OPTIONS: [&str; 12] = [
    "--sudo-command",
    "--timeout",
    "--cooldown",
    "--jobs",
    "--sort-by",
    "--config",
    "--validate-config",
    "--export-config",
    "--theme",
    "--table-format",
    "--verbosity",
    "--help-format",
];

/// The short spellings of the options above that take a value:
/// `-t`,
/// `-j`
/// and `-s`.
const SHORT_VALUE_OPTIONS: [char; 3] = ['t', 'j', 's'];

/// Top-level options that print something else and exit
/// (a different help format,
/// the man page,
/// the tree,
/// the parameters,
/// a configuration):
/// the text that follows mpm's output must not be added to them.
const OTHER_OUTPUT_OPTIONS: [&str; 7] = [
    "--help-format",
    "--man",
    "--tree",
    "--params",
    "--export-config",
    "--validate-config",
    "--bar-plugin-path",
];

/// The values `--color` accepts.
const COLOR_VALUES: [&str; 3] = ["auto", "always", "never"];

/// What the top-level options of one mpm command line ask for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopLevelRequest {
    /// mpm prints its version and exits.
    Version,
    /// mpm prints its own help and exits.
    Help,
    /// Anything else:
    /// a subcommand,
    /// another output,
    /// or a command line the scan does not understand.
    Other,
}

/// What:
/// Reads the options before the subcommand in order and returns the first of `--version`,
/// `-h` / `--help`,
/// or an option that prints something else;
/// click runs these eager options in the order they were typed.
/// Why:
/// The shadow adds its text only where mpm printed its plain version or its plain help.
/// An option it does not know is taken as a flag,
/// so a new mpm option that takes a value must be added to `VALUE_OPTIONS`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function topLevelRequest(args: string[]): 'version' | 'help' | 'other'
/// ```
pub fn top_level_request(arguments: &[OsString]) -> TopLevelRequest {
    let mut index: usize = 0;
    while index < arguments.len() {
        let Some(argument) = arguments[index].to_str() else {
            return TopLevelRequest::Other;
        };
        index += 1;
        if argument == "--version" {
            return TopLevelRequest::Version;
        }
        if argument == "--help" || argument == "-h" {
            return TopLevelRequest::Help;
        }
        if let Some(long) = argument.strip_prefix("--") {
            if long.is_empty() {
                return TopLevelRequest::Other;
            }
            let name: &str = long.split('=').next().unwrap_or(long);
            if OTHER_OUTPUT_OPTIONS.contains(&format!("--{name}").as_str()) {
                return TopLevelRequest::Other;
            }
            if long.contains('=') {
                continue;
            }
            if VALUE_OPTIONS.contains(&argument) {
                // The next word is the value, whatever it looks like.
                index += 1;
                continue;
            }
            if argument == "--color"
                && let Some(next) = arguments.get(index).and_then(|value| return value.to_str())
                && COLOR_VALUES.contains(&next)
            {
                index += 1;
            }
            continue;
        }
        if argument.starts_with('-') && argument.len() > 1 {
            // A cluster of short options such as `-vvt 5`: a value option takes the rest of the word or the next word.
            let letters: Vec<char> = argument.chars().skip(1).collect();
            for (position, letter) in letters.iter().enumerate() {
                if *letter == 'h' {
                    return TopLevelRequest::Help;
                }
                if SHORT_VALUE_OPTIONS.contains(letter) {
                    if position + 1 == letters.len() {
                        index += 1;
                    }
                    break;
                }
                if !"adxp0vq".contains(*letter) {
                    return TopLevelRequest::Other;
                }
            }
            continue;
        }
        // The first word that is not an option is the subcommand.
        return TopLevelRequest::Other;
    }
    return TopLevelRequest::Other;
}

/// The version of this shadow.
fn shadow_version() -> &'static str {
    return env!("CARGO_PKG_VERSION");
}

/// What:
/// The single line printed on standard output after mpm's `--version`.
/// Why:
/// Like `hub version <X>` after git's version line:
/// it names the shadow,
/// its version,
/// and the real mpm it ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function versionLine(realMpm: string): string
/// ```
pub fn version_line(real_mpm: &Path) -> String {
    return format!("mpm-shadow {}, shadowing mpm at {}\n", shadow_version(), real_mpm.display());
}

/// What:
/// The section printed on standard output after mpm's `--help`:
/// a blank line,
/// then what the shadow is,
/// its version,
/// the real mpm's path,
/// and what it does after every run.
/// Why:
/// Like the commands `hub help` lists after git's help,
/// a reader of `mpm --help` learns that this `mpm` is a shadow and what it adds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function helpSection(realMpm: string): string
/// ```
pub fn help_section(real_mpm: &Path) -> String {
    return format!(
        "\nmpm-shadow {version}:\n  \
         This `mpm` is a shadow of mpm, run by mpm-shadow (shadowing the real mpm at {path}).\n  \
         It passes your arguments, the standard streams and mpm's exit status through unchanged.\n  \
         After every run it deletes the superseded cached versions of this repository's own `monochromatic-` crates\n  \
         from cargo's registry, and reports each on standard error as `mpm-shadow: deleted stale <path>`.\n  \
         The toolchain commands (install, update, outdated, list, bin) belong to `mpm-shadow --help`.\n",
        version = shadow_version(),
        path = real_mpm.display()
    );
}

/// The unit tests of the scan and the added text.
#[cfg(test)]
#[path = "shadow_notice_tests.rs"]
mod tests;
