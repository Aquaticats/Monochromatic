//! What:
//!  Policy-configuration loading decisions before expensive policy initialization.
//! Why:
//!  Known inspection forms keep the fast path;
//!  mutations,
//!  aliases and ambiguous options require configuration.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Classify native argv without executing or weakening any configured policy.
//! ```

/// Import the exact current-Git global boundary and native arguments.
use super::global_arguments::{GlobalOutcome, global_layout};
use std::ffi::OsString;

/// Whether this invocation needs the shipped policy configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigLoading {
    /// Known inspection/query/native-option-error form does not initialize the policy engines.
    Skip,
    /// A mutation or unresolved command shape requires configuration.
    Required,
}

/// Commands whose existing wrapper contract does not load repository policy configuration.
const INSPECTION_COMMANDS: &[&[u8]] = &[
    b"annotate",
    b"blame",
    b"cat-file",
    b"count-objects",
    b"describe",
    b"diff",
    b"diff-files",
    b"diff-index",
    b"diff-tree",
    b"for-each-ref",
    b"grep",
    b"help",
    b"log",
    b"ls-files",
    b"ls-remote",
    b"ls-tree",
    b"merge-base",
    b"name-rev",
    b"rev-list",
    b"rev-parse",
    b"shortlog",
    b"show",
    b"show-branch",
    b"show-ref",
    b"status",
    b"version",
    b"whatchanged",
];

/// Mutation-bearing long forms from the selected native branch/tag implementations.
fn mutating_long(name: &[u8], branch: bool) -> bool {
    if branch {
        return [
            b"--copy".as_slice(),
            b"--delete",
            b"--delete-merged",
            b"--edit-description",
            b"--force",
            b"--move",
            b"--set-upstream",
            b"--set-upstream-to",
            b"--unset-upstream",
            b"--create-reflog",
        ]
        .contains(&name);
    }
    return [
        b"--annotate".as_slice(),
        b"--delete",
        b"--edit",
        b"--force",
        b"--sign",
        b"--local-user",
        b"--message",
        b"--file",
        b"--trailer",
        b"--create-reflog",
    ]
    .contains(&name);
}

/// Classify branch/tag forms conservatively while distinguishing inline-optional and separated values.
fn mixed_command(arguments: &[OsString], branch: bool) -> ConfigLoading {
    let mut listing: bool = false;
    let mut positional: bool = false;
    let mut after_separator: bool = false;
    // True while the token being visited is the separated value of the filter before it.
    let mut is_value: bool = false;
    // Visiting each argument once with its position cannot loop forever, unlike a hand-stepped index.
    for (index, argument) in arguments.iter().enumerate() {
        if is_value {
            is_value = false;
            continue;
        }
        let token: &[u8] = argument.as_encoded_bytes();
        if after_separator {
            positional = true;
            continue;
        }
        if token == b"--" {
            after_separator = true;
            continue;
        }
        if token.starts_with(b"--") {
            // Only split the option name; attached values remain opaque native bytes.
            let assignment: Option<usize> = token.iter().position(is_equals);
            let name: &[u8] = if let Some(at) = assignment {
                &token[..at]
            } else {
                token
            };
            if mutating_long(name, branch) {
                return ConfigLoading::Required;
            }
            if name == b"--list"
                || (branch && name == b"--show-current")
                || (!branch && name == b"--verify")
            {
                listing = true;
                continue;
            }
            // These native optional arguments consume only an attached '=value', never the next positional.
            if [b"--color".as_slice(), b"--column", b"--abbrev"].contains(&name) {
                continue;
            }
            if [
                b"--contains".as_slice(),
                b"--no-contains",
                b"--with",
                b"--without",
                b"--merged",
                b"--no-merged",
                b"--points-at",
                b"--format",
                b"--sort",
            ]
            .contains(&name)
            {
                if assignment.is_none() {
                    if index + 1 == arguments.len() {
                        // Last-argument defaults are inspection filters; missing required format/sort is left to Git.
                        return if name == b"--format" || name == b"--sort" {
                            ConfigLoading::Required
                        } else {
                            ConfigLoading::Skip
                        };
                    }
                    is_value = true;
                }
                if ![b"--format".as_slice(), b"--sort"].contains(&name) {
                    listing = true;
                }
                continue;
            }
            if [
                b"--verbose".as_slice(),
                b"--quiet",
                b"--remotes",
                b"--all",
                b"--ignore-case",
                b"--omit-empty",
                b"--no-color",
                b"--no-column",
            ]
            .contains(&name)
            {
                continue;
            }
            // Unknown and abbreviated forms can be valid native mutations; do not guess them into the fast path.
            return ConfigLoading::Required;
        }
        if token.starts_with(b"-") && token.len() > 1 {
            let mut numeric_lines: bool = false;
            for letter in &token[1..] {
                if numeric_lines {
                    if letter.is_ascii_digit() {
                        continue;
                    }
                    return ConfigLoading::Required;
                }
                let mutations: &[u8] = if branch { b"cCdDfmMut" } else { b"adefsumF" };
                if mutations.contains(letter) {
                    return ConfigLoading::Required;
                }
                if *letter == b'l' || (!branch && *letter == b'v') {
                    listing = true;
                } else if !branch && *letter == b'n' {
                    listing = true;
                    numeric_lines = true;
                } else if (branch && b"vqrai".contains(letter)) || (!branch && *letter == b'i') {
                    continue;
                } else {
                    return ConfigLoading::Required;
                }
            }
            continue;
        }
        positional = true;
    }
    if !positional || listing {
        return ConfigLoading::Skip;
    }
    return ConfigLoading::Required;
}

/// Named ASCII predicate avoids capturing or reinterpreting native option values.
fn is_equals(byte: &u8) -> bool {
    return *byte == b'=';
}

/// Decide configuration ownership without changing argv,
///  resolving aliases,
///  or reading executable configuration.
pub fn classify_config_loading(arguments: &[OsString]) -> ConfigLoading {
    let layout = global_layout(arguments);
    if layout.outcome == GlobalOutcome::Query
        || layout.outcome == GlobalOutcome::InvalidOption
        || layout.outcome == GlobalOutcome::MissingValue
    {
        return ConfigLoading::Skip;
    }
    if layout.outcome == GlobalOutcome::NoCommand {
        return ConfigLoading::Required;
    }
    let command: &[u8] = arguments[layout.prefix_len].as_encoded_bytes();
    if INSPECTION_COMMANDS.contains(&command) {
        return ConfigLoading::Skip;
    }
    if command == b"branch" || command == b"tag" {
        return mixed_command(&arguments[layout.prefix_len + 1..], command == b"branch");
    }
    return ConfigLoading::Required;
}

/// Native-value and mixed-command controls remain outside release builds.
#[cfg(test)]
#[path = "config_loading_tests.rs"]
mod tests;
