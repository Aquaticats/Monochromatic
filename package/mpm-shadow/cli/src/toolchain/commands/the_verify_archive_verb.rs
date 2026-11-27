//! What:
//! `mpm-shadow verify-archive <tool> <file> [--platform <id>]`:
//! the file is hashed and compared with the SHA-256 the lock records for that tool and platform.
//! Why:
//! A fresh machine needs one prebuilt `mpm-shadow` archive whose SHA-256 is in the lock,
//! and this verb is the check that runs before that archive is run.
//! The platform defaults to the host's,
//! so the common case names only the tool and the file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function verifyArchiveVerb(matches: ParsedArgs, scope: Scope): number
//! ```

/// Imports the `clap` type for reading a parsed command line.
use clap::ArgMatches;

/// Imports `Host` from `super::super::context`.
use super::super::context::Host;
/// Imports `Scope` from `super::super::scope`.
use super::super::scope::Scope;
/// Imports `run_verify_archive` from `super::super::verify_archive`.
use super::super::verify_archive::run_verify_archive;
/// Imports what this module uses from `super::what_every_verb_prints`.
use super::what_every_verb_prints::{
    print_line,
    report,
};

/// What:
/// Runs `verify-archive` with the arguments `matches` in `scope` and returns the exit code.
/// Why:
/// A missing `--platform` is answered with the host's,
/// which is the platform whose archive a fresh machine downloads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function verifyArchiveVerb(matches: ParsedArgs, scope: Scope): number
/// ```
pub(super) fn verify_archive_verb(
    matches: &ArgMatches,
    scope: &Scope,
) -> i32 {
    let tool: &str = matches
        .get_one::<String>("tool")
        .map(String::as_str)
        .unwrap_or("");
    let file: &str = matches
        .get_one::<String>("file")
        .map(String::as_str)
        .unwrap_or("");
    let platform: String = match matches.get_one::<String>("platform") {
        Some(given) => given.clone(),
        None => Host::current().platform,
    };
    return match run_verify_archive(scope, &platform, tool, std::path::Path::new(file)) {
        Ok(line) => {
            print_line(&line);
            0
        }
        Err(error) => report(&error),
    };
}
