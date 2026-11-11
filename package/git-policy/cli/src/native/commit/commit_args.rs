//! What:
//!  Argument transforms of native preparation:
//!       the normalised `commit` argument region,
//!       the split of global options from the commit region,
//!       the removal of `-a`/`--all`,
//!       and the arguments of the real `git commit` that run against the complete private index.
//! Why:
//!  Native `git commit` must see the policy-settled index,
//! not worktree bytes again,
//!  so selection controls and pathspecs are removed once the private index already holds them.
//!  The removals match the TypeScript wrapper's token for token
//!  (deleted on 2026-10-13 in `642b6d72cb`),
//! including its use of `lastIndexOf`,
//!  so a `-m` value equal to a pathspec is removed from the same position.
//!  A token is bytes and never text:
//! a message or a file name the person typed may hold a byte that is not UTF-8,
//!  and a transform that spelled such a token would hand Git another message and another file
//!  (`path_bytes.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const invocation = splitCommitInvocation({ args, subcommandIndex });
//! const commitArgs = withoutAllFlag(invocation.commitArgs);
//! ```

/// One argument as the operating system holds it,
/// and the bytes it is made of.
use std::ffi::{OsStr, OsString};
/// Pathspec separator token,
/// shared with the escape hatch module.
/// The pathspec separator that ends option parsing in Git and in the incumbent.
const PATHSPEC_SEPARATOR: &str = "--";
/// One token of a command line as bytes.
use crate::path_bytes::{argument_bytes, os_from_path_bytes};
/// The byte-level readings of one token:
/// an option's name and a cluster's letters and values.
use crate::commit_args_token_bytes::{
    COMMIT_VALUE_OPTIONS, OPTIONAL_VALUE_LETTERS, cluster_letters, is_a_short_cluster, is_a_short_value_letter, is_an_equals_sign, is_one_of, is_optional_value_letter,
    normalise_inline_short,
};
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;

/// Global options that redirect native Git away from the shadow repository,
/// so they are dropped.
const REDIRECTING_VALUE_OPTIONS: [&str; 3] = ["-C", "--git-dir", "--work-tree"];

/// Global options that consume the next token as their value.
const VALUE_TAKING_GLOBAL_OPTIONS: [&str; 7] = [
    "-c",
    "-C",
    "--git-dir",
    "--work-tree",
    "--namespace",
    "--super-prefix",
    "--attr-source",
];

/// What:
///  Whether a commit option token takes the next token as its value.
/// Why:
///   A value can be spelled like the separator (`-m --`),
/// and only a token in option position can end the options.
///  A short cluster takes the next token when its value letter ends it;
/// `-S` and `-u` take their value attached only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takesNextToken('-m'); // true
/// takesNextToken('-qm'); // true
/// takesNextToken('-mhello'); // false
/// ```
pub(crate) fn takes_next_token(token: &OsStr) -> bool {
    let bytes: &[u8] = argument_bytes(token);
    if bytes.starts_with(b"--") {
        return is_one_of(bytes, &COMMIT_VALUE_OPTIONS);
    }
    if !is_a_short_cluster(bytes) {
        return false;
    }
    let letters: &[u8] = cluster_letters(bytes);
    for (index, letter) in letters.iter().enumerate() {
        if OPTIONAL_VALUE_LETTERS.contains(letter) {
            return false;
        }
        if is_a_short_value_letter(*letter) {
            return index + 1 == letters.len();
        }
    }
    return false;
}

/// What:
///  The index of the `--` that ends the options of a commit region,
/// or nothing when the person typed none.
/// Why:
///   Everything after it is a path,
/// so a wrapper option must never follow it;
/// a `--` that is the value of an option such as
///  `-m` is a message and not the separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// separatorIndex(['-m', '--', '--', 'a.txt']); // 2
/// ```
pub fn separator_index(region: &[OsString]) -> Option<usize> {
    let mut index: usize = 0;
    while index < region.len() {
        let token: &[u8] = argument_bytes(region[index].as_os_str());
        if token == PATHSPEC_SEPARATOR.as_bytes() {
            return Some(index);
        }
        if takes_next_token(region[index].as_os_str()) {
            index += 1;
        }
        index += 1;
    }
    return None;
}

/// What:
///  Normalises the post-`commit` argument region,
/// keeping the pathspec tail verbatim.
/// Why:
///   Later transforms recognize separated values only,
/// and the pathspec tail after `--` must not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// normaliseCommitArgs(['-amhello', 'file.ts']); // ['-a', '-m', 'hello', 'file.ts']
/// ```
pub fn normalise_commit_args(args: &[OsString]) -> Vec<OsString> {
    let separator: Option<usize> = separator_index(args);
    let (region, tail): (&[OsString], &[OsString]) = match separator {
        Some(index) => (&args[..index], &args[index..]),
        None => (args, &[]),
    };
    let mut normalised: Vec<OsString> = Vec::new();
    let mut value_next: bool = false;
    for token in region {
        // The value of an option is the person's bytes and is never read as a cluster of options.
        if value_next {
            normalised.push(token.clone());
            value_next = false;
            continue;
        }
        normalised.extend(normalise_inline_short(token.as_os_str()));
        value_next = takes_next_token(token.as_os_str());
    }
    normalised.extend(tail.iter().cloned());
    return normalised;
}

/// What:
///  The name half of a global option,
/// without the value an `=` attached to it.
/// Why:
///   `--git-dir=/somewhere` redirects Git exactly as `--git-dir /somewhere` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = token.startsWith('--') && token.includes('=') ? token.slice(0, token.indexOf('=')) : token
/// ```
fn global_name(token: &[u8]) -> &[u8] {
    if !token.starts_with(b"--") {
        return token;
    }
    let Some(at) = token.iter().position(is_an_equals_sign) else {
        return token;
    };
    return &token[..at];
}

/// What:
///  Splits a transformed argument list into the kept global options and the commit region.
/// Why:
///   Redirecting options (`-C`,
/// `--git-dir`,
/// `--work-tree`,
/// `--bare`) would send native commit elsewhere,
///  so they are dropped with their values,
/// and every other global option is kept in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitCommitInvocation({ args, subcommandIndex });
/// ```
pub fn split_commit_invocation(args: &[OsString], subcommand_index: usize) -> (Vec<OsString>, Vec<OsString>) {
    let globals: &[OsString] = &args[..subcommand_index];
    let mut kept: Vec<OsString> = Vec::new();
    let mut index: usize = 0;
    while index < globals.len() {
        let token: &[u8] = argument_bytes(globals[index].as_os_str());
        let name: &[u8] = global_name(token);
        let separated: bool = is_one_of(token, &VALUE_TAKING_GLOBAL_OPTIONS);
        if is_one_of(name, &REDIRECTING_VALUE_OPTIONS) || token == b"--bare" {
            if separated {
                index += 1;
            }
            index += 1;
            continue;
        }
        kept.push(globals[index].clone());
        if separated {
            if let Some(value) = globals.get(index + 1) {
                kept.push(value.clone());
            }
            index += 1;
        }
        index += 1;
    }
    return (kept, args[subcommand_index + 1..].to_vec());
}

/// What:
///  Removes `-a`/`--all` from the commit region,
/// keeping clustered short options.
/// Why:
///   The private index already holds every tracked change,
/// so native commit must not restage worktree bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutAllFlag(['-am', 'x']); // ['-m', 'x']
/// ```
pub fn without_all_flag(commit_args: &[OsString]) -> Vec<OsString> {
    let normalised: Vec<OsString> = normalise_commit_args(commit_args);
    let mut kept: Vec<OsString> = Vec::new();
    let mut value_next: bool = false;
    let mut after_separator: bool = false;
    for token in normalised {
        let bytes: &[u8] = argument_bytes(token.as_os_str());
        if after_separator || value_next {
            kept.push(token);
            value_next = false;
            continue;
        }
        if bytes == PATHSPEC_SEPARATOR.as_bytes() {
            after_separator = true;
            kept.push(token);
            continue;
        }
        if bytes == b"--all" {
            continue;
        }
        if is_a_short_cluster(bytes) {
            let letters: &[u8] = cluster_letters(bytes);
            let value_start: Option<usize> = letters.iter().position(is_optional_value_letter);
            let flags: &[u8] = match value_start {
                Some(index) => &letters[..index],
                None => letters,
            };
            let mut cluster: Vec<u8> = vec![b'-'];
            for letter in flags {
                if *letter != b'a' {
                    cluster.push(*letter);
                }
            }
            if let Some(index) = value_start {
                cluster.extend_from_slice(&letters[index..]);
            }
            if cluster == b"-" {
                continue;
            }
            value_next = is_one_of(cluster.as_slice(), &COMMIT_VALUE_OPTIONS);
            kept.push(os_from_path_bytes(cluster.as_slice()));
            continue;
        }
        value_next = is_one_of(bytes, &COMMIT_VALUE_OPTIONS);
        kept.push(token);
    }
    return kept;
}

/// What:
///  Removes the pathspec-file controls and the explicit pathspecs,
///  then the `--` separator and `-o`/`--only`,
/// from the commit region.
/// Why:
///   The private index already holds the selected paths,
/// so the real commit names none of them.
///  The pathspec removal runs from the last pathspec backwards with `lastIndexOf`,
/// as the incumbent does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateExplicitCommitArgs({ args, pathspecs });
/// ```
pub fn private_explicit_commit_args(args: &[OsString], pathspecs: &[OsString]) -> Vec<OsString> {
    let mut without_file: Vec<OsString> = Vec::new();
    let mut index: usize = 0;
    while index < args.len() {
        let token: &[u8] = argument_bytes(args[index].as_os_str());
        if token == b"--pathspec-from-file" {
            index += 2;
            continue;
        }
        if token.starts_with(b"--pathspec-from-file=") || token == b"--pathspec-file-nul" {
            index += 1;
            continue;
        }
        without_file.push(args[index].clone());
        index += 1;
    }
    for pathspec in pathspecs.iter().rev() {
        let wanted: &[u8] = argument_bytes(pathspec.as_os_str());
        // The last token equal to the pathspec goes, as `.rposition(...)` found it, and the paths are compared as bytes so
        //  two names that read the same with replacement characters are still two paths.
        let mut last_position: Option<usize> = None;
        for (position, token) in without_file.iter().enumerate() {
            if argument_bytes(token.as_os_str()) == wanted {
                last_position = Some(position);
            }
        }
        if let Some(position) = last_position {
            without_file.remove(position);
        }
    }
    // Only the separator and the `-o` and `--only` the person typed in option position go; the same text as the value of an option
    //  (`-m --`) is the person's and stays.
    let separator: Option<usize> = separator_index(without_file.as_slice());
    let mut kept: Vec<OsString> = Vec::new();
    let mut value_next: bool = false;
    for (position, token) in without_file.into_iter().enumerate() {
        let past_separator: bool = match separator {
            Some(at) => position > at,
            None => false,
        };
        if past_separator {
            kept.push(token);
            continue;
        }
        if separator == Some(position) {
            continue;
        }
        if value_next {
            kept.push(token);
            value_next = false;
            continue;
        }
        let bytes: &[u8] = argument_bytes(token.as_os_str());
        value_next = takes_next_token(token.as_os_str());
        if bytes == b"-o" || bytes == b"--only" {
            continue;
        }
        kept.push(token);
    }
    return kept;
}

/// What:
///  Removes the selection controls once the private index holds the exact user choice.
/// Why:
///   `-i`,
/// `-p`,
/// `--include`,
/// `--interactive` and `--patch` already ran against the private index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateSelectionCommitArgs({ args, pathspecs });
/// ```
pub fn private_selection_commit_args(args: &[OsString], pathspecs: &[OsString]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::new();
    for token in private_explicit_commit_args(args, pathspecs) {
        if !is_one_of(argument_bytes(token.as_os_str()), &["-i", "-p", "--include", "--interactive", "--patch"]) {
            kept.push(token);
        }
    }
    return kept;
}

/// What:
///  Chooses the arguments of the real `git commit` for the commit mode.
/// Why:
///   Explicit-path commits drop their pathspecs;
/// index commits drop selection controls only
///  when selection ran privately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolvePrivateCommitArgs({ args, pathspecs, mode, selectedPrivately });
/// ```
pub fn resolve_private_commit_args(
    args: &[OsString],
    pathspecs: &[OsString],
    mode: TransactionMode,
    selected_privately: bool,
) -> Vec<OsString> {
    if mode == TransactionMode::ExplicitPath {
        return private_explicit_commit_args(args, pathspecs);
    }
    if selected_privately {
        return private_selection_commit_args(args, pathspecs);
    }
    return args.to_vec();
}

/// Argument transforms stay out of the release executable.
#[cfg(test)]
#[path = "commit_args_tests.rs"]
mod tests;
