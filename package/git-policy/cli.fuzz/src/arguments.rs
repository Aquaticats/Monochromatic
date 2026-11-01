//! What:
//!  Argument vectors built from fuzz bytes,
//!  and the invariants of argument classification.
//! Why:
//!  Git arguments are arbitrary bytes.
//!  Classification must never change them,
//!  must
//!      agree with itself,
//!  and must never let a mutating `branch` or `tag` form skip
//!      policy configuration.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkGlobalLayout(argumentsFromBytes(data)); checkConfigLoading(generatedArguments(data));
//! ```

/// Import the token tables kept in the sibling module.
use crate::argument_tables::{BRANCH_MUTATIONS, INSPECTION_COMMANDS, TAG_MUTATIONS, TOKENS};
/// Import the classification functions under test.
use git_policy_cli::config_loading::{ConfigLoading, classify_config_loading};
use git_policy_cli::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
/// The production conversion of raw bytes to an argument, so the generators feed the code under test the same way.
use git_policy_cli::path_bytes::os_from_path_bytes as argument;
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Fuzz input is turned into arguments without any decoding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  Largest generated argument vector.
///       `usize` is the index and length type of every Rust list (siblings `u32`,
///  `u64`).
/// Why:
///   A bound keeps each execution fast;
///  real command lines are far shorter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_ARGUMENTS = 48;
/// ```
pub const MAX_ARGUMENTS: usize = 48;

/// What:
///  Split fuzz bytes at NUL into at most `MAX_ARGUMENTS` arguments.
///       `Vec<OsString>` is an owned list of owned arguments.
/// Why:
///   NUL is the only byte an argument cannot contain,
///  so every other byte
///       sequence,
///  UTF-8 or not,
///  becomes argument content exactly as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function argumentsFromBytes(data: Uint8Array): string[];
/// ```
pub fn arguments_from_bytes(data: &[u8]) -> Vec<OsString> {
    // `Vec::<OsString>::new()` is an empty owned list; `mut` allows pushing.
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    if data.is_empty() {
        return result;
    }
    // `.split(...)` with a named predicate cuts the bytes at each NUL.
    for piece in data.split(is_nul) {
        if result.len() == MAX_ARGUMENTS {
            break;
        }
        result.push(argument(piece));
    }
    return result;
}

/// Named predicate for splitting at NUL;
///  `&u8` borrows one byte.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Map each fuzz byte to one real Git token,
///  up to `MAX_ARGUMENTS`.
/// Why:
///   This reaches value options,
///  queries,
///  separators and `branch`/`tag` forms that
///       raw bytes almost never spell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedArguments(data: Uint8Array): string[];
/// ```
pub fn generated_arguments(data: &[u8]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    // `for byte in data` borrows each byte; `*byte` reads it.
    for byte in data {
        if result.len() == MAX_ARGUMENTS {
            break;
        }
        // `usize::from` widens the byte to an index; `%` wraps it into the table.
        result.push(argument(TOKENS[usize::from(*byte) % TOKENS.len()]));
    }
    return result;
}

/// What:
///  Assert the invariants of global-argument layout for one argument vector.
///       `&[OsString]` borrows the arguments.
/// Why:
///   The layout decides where the subcommand is.
///  It must leave the arguments
///       untouched,
///  be repeatable,
///  point at a boundary of the right kind,
///  and not be
///       changed by anything after a decided boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGlobalLayout(args: string[]): void;
/// ```
pub fn check_global_layout(arguments: &[OsString]) {
    let snapshot: Vec<OsString> = arguments.to_vec();
    let layout: GlobalLayout = global_layout(arguments);
    assert_eq!(arguments, snapshot.as_slice(), "layout changed its input");
    assert_eq!(global_layout(arguments), layout, "layout is not repeatable");
    assert!(layout.prefix_len <= arguments.len(), "prefix exceeds input");
    if layout.outcome == GlobalOutcome::NoCommand {
        assert_eq!(
            layout.prefix_len,
            arguments.len(),
            "no command must consume everything"
        );
    } else {
        assert!(
            layout.prefix_len < arguments.len(),
            "a boundary token must exist"
        );
        // `&[u8]` borrows the boundary token's raw bytes.
        let boundary: &[u8] = arguments[layout.prefix_len].as_encoded_bytes();
        if layout.outcome == GlobalOutcome::Command {
            assert!(
                !boundary.starts_with(b"-"),
                "a command never starts with a dash"
            );
        } else {
            assert!(
                boundary.starts_with(b"-"),
                "queries and errors are dash-led"
            );
        }
        if layout.outcome == GlobalOutcome::MissingValue {
            assert_eq!(
                layout.prefix_len + 1,
                arguments.len(),
                "a missing value is only possible at the end"
            );
        }
    }
    // The global prefix alone is complete and names no command.
    assert_eq!(
        global_layout(&arguments[..layout.prefix_len]),
        GlobalLayout {
            prefix_len: layout.prefix_len,
            outcome: GlobalOutcome::NoCommand,
        },
        "the prefix must stand alone"
    );
    // A decided boundary is not changed by replacing or extending what follows it.
    if layout.outcome == GlobalOutcome::Command
        || layout.outcome == GlobalOutcome::Query
        || layout.outcome == GlobalOutcome::InvalidOption
    {
        // `..=n` is the inclusive range up to and including index `n`.
        let mut decided: Vec<OsString> = arguments[..=layout.prefix_len].to_vec();
        assert_eq!(
            global_layout(decided.as_slice()),
            layout,
            "suffix removal changed layout"
        );
        decided.push(argument(b"-C"));
        decided.push(argument(b"--help"));
        decided.push(argument(b"commit"));
        assert_eq!(
            global_layout(decided.as_slice()),
            layout,
            "suffix change changed layout"
        );
    }
}

/// What:
///  Assert the invariants of configuration-loading classification for one vector.
/// Why:
///   Skipping configuration skips policy.
///  Only native queries,
///  option errors and
///       known inspection commands may skip;
///  any `branch` or `tag` invocation gains a
///       mutating flag in first position must require configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkConfigLoading(args: string[]): void;
/// ```
pub fn check_config_loading(arguments: &[OsString]) {
    let snapshot: Vec<OsString> = arguments.to_vec();
    let decision: ConfigLoading = classify_config_loading(arguments);
    assert_eq!(
        arguments,
        snapshot.as_slice(),
        "classification changed its input"
    );
    assert_eq!(
        classify_config_loading(arguments),
        decision,
        "classification is not repeatable"
    );
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome == GlobalOutcome::NoCommand {
        assert_eq!(
            decision,
            ConfigLoading::Required,
            "no command must not skip"
        );
        return;
    }
    if layout.outcome != GlobalOutcome::Command {
        assert_eq!(
            decision,
            ConfigLoading::Skip,
            "Git handles queries and option errors itself"
        );
        return;
    }
    let command: &[u8] = arguments[layout.prefix_len].as_encoded_bytes();
    if INSPECTION_COMMANDS.contains(&command) {
        assert_eq!(decision, ConfigLoading::Skip, "inspection commands skip");
        return;
    }
    let branch: bool = command == b"branch";
    if !branch && command != b"tag" {
        assert_eq!(
            decision,
            ConfigLoading::Required,
            "unknown commands must not skip"
        );
        return;
    }
    // The bare listing form skips.
    assert_eq!(
        classify_config_loading(&arguments[..=layout.prefix_len]),
        ConfigLoading::Skip,
        "a bare branch or tag lists"
    );
    let mutations: &[&[u8]] = if branch {
        BRANCH_MUTATIONS
    } else {
        TAG_MUTATIONS
    };
    // `for flag in mutations` borrows each mutating spelling in turn.
    for flag in mutations {
        let mut mutated: Vec<OsString> = arguments[..=layout.prefix_len].to_vec();
        mutated.push(argument(flag));
        mutated.extend_from_slice(&arguments[layout.prefix_len + 1..]);
        assert_eq!(
            classify_config_loading(mutated.as_slice()),
            ConfigLoading::Required,
            "a leading mutating flag must require configuration: {mutated:?}"
        );
    }
}

/// Generator and invariant controls stay out of the fuzz binaries.
#[cfg(test)]
#[path = "arguments_tests.rs"]
mod tests;
