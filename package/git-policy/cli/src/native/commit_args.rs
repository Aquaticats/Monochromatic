//! What:
//!  Argument transforms of native preparation:
//!       the normalised `commit` argument region,
//!       the split of global options from the commit region,
//!       the removal of `-a`/`--all`,
//!       and the arguments of the real `git commit` that run against the complete private index.
//! Why:
//!  Native `git commit` must see the policy-settled index, not worktree bytes again,
//!  so selection controls and pathspecs are removed once the private index already holds them
//!  (`src/policy-engine/commit-preparation-native.ts`,
//!  `commit-transaction-selection.ts`,
//!  `src/parser/commit-normalise.ts`).
//!  The removals match the incumbent token for token, including its use of `lastIndexOf`,
//!  so a `-m` value equal to a pathspec is removed from the same position.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const invocation = splitCommitInvocation({ args, subcommandIndex });
//! const commitArgs = withoutAllFlag(invocation.commitArgs);
//! ```

/// Pathspec separator token, shared with the escape hatch module.
/// The pathspec separator that ends option parsing in Git and in the incumbent.
const PATHSPEC_SEPARATOR: &str = "--";
/// Commit selection mode of the transaction.
use super::transaction_journal::TransactionMode;

/// Global options that redirect native Git away from the shadow repository, so they are dropped.
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

/// Commit options whose separated value never names a flag.
const COMMIT_VALUE_OPTIONS: [&str; 20] = [
    "-m",
    "-F",
    "-C",
    "-c",
    "-t",
    "-U",
    "--message",
    "--file",
    "--reuse-message",
    "--reedit-message",
    "--squash",
    "--fixup",
    "--author",
    "--date",
    "--cleanup",
    "--trailer",
    "--template",
    "--unified",
    "--inter-hunk-context",
    "--pathspec-from-file",
];

/// Short-value options whose clustered form can carry the value in the same token.
const SHORT_VALUE_OPTIONS: [&str; 6] = ["-m", "-F", "-C", "-c", "-t", "-U"];

/// Short commit options with an optional attached value (`-S[<keyid>]`, `-u[<mode>]`).
const OPTIONAL_VALUE_LETTERS: [char; 2] = ['S', 'u'];

/// What:
///  Finds the first value-taking short option letter inside a cluster token.
/// Why:
///   A cluster such as `-am` carries `m`, whose value is the following token or the rest of the cluster.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// findValueOptionIndex(token)
/// ```
fn find_value_option_index(letters: &[char]) -> Option<usize> {
    for (index, letter) in letters.iter().enumerate().skip(1) {
        let option: String = format!("-{letter}");
        if SHORT_VALUE_OPTIONS.contains(&option.as_str()) {
            return Some(index);
        }
    }
    return None;
}

/// What:
///  Splits one short cluster that carries its value inline into the separated form.
/// Why:
///   `-amhello` becomes `-a`, `-m`, `hello`, so the value is a token of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// normaliseInlineShort('-amhello') // ['-a', '-m', 'hello']
/// ```
fn normalise_inline_short(token: &str) -> Vec<String> {
    if !token.starts_with('-') || token.starts_with("--") || token.len() <= 2 {
        return vec![String::from(token)];
    }
    let letters: Vec<char> = token.chars().collect();
    let Some(value_index) = find_value_option_index(letters.as_slice()) else {
        return vec![String::from(token)];
    };
    let mut tokens: Vec<String> = letters[1..value_index]
        .iter()
        .map(|letter| format!("-{letter}"))
        .collect();
    tokens.push(format!("-{}", letters[value_index]));
    let inline: String = letters[value_index + 1..].iter().collect();
    if !inline.is_empty() {
        tokens.push(inline);
    }
    return tokens;
}

/// What:
///  Normalises the post-`commit` argument region, keeping the pathspec tail verbatim.
/// Why:
///   Later transforms recognize separated values only, and the pathspec tail after `--` must not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// normaliseCommitArgs(['-amhello', 'file.ts']); // ['-a', '-m', 'hello', 'file.ts']
/// ```
pub fn normalise_commit_args(args: &[String]) -> Vec<String> {
    let separator: Option<usize> = args.iter().position(|token| { return token == PATHSPEC_SEPARATOR; });
    let (region, tail): (&[String], &[String]) = match separator {
        Some(index) => (&args[..index], &args[index..]),
        None => (args, &[]),
    };
    let mut normalised: Vec<String> = region
        .iter()
        .flat_map(|token| { return normalise_inline_short(token); })
        .collect();
    normalised.extend(tail.iter().cloned());
    return normalised;
}

/// What:
///  Splits a transformed argument list into the kept global options and the commit region.
/// Why:
///   Redirecting options (`-C`, `--git-dir`, `--work-tree`, `--bare`) would send native commit elsewhere,
///  so they are dropped with their values, and every other global option is kept in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitCommitInvocation({ args, subcommandIndex });
/// ```
pub fn split_commit_invocation(args: &[String], subcommand_index: usize) -> (Vec<String>, Vec<String>) {
    let globals: &[String] = &args[..subcommand_index];
    let mut kept: Vec<String> = Vec::new();
    let mut index: usize = 0;
    while index < globals.len() {
        let token: &str = globals[index].as_str();
        let name: &str = if token.starts_with("--") && token.contains('=') {
            &token[..token.find('=').unwrap_or(token.len())]
        } else {
            token
        };
        let separated: bool = VALUE_TAKING_GLOBAL_OPTIONS.contains(&token);
        if REDIRECTING_VALUE_OPTIONS.contains(&name) || token == "--bare" {
            if separated {
                index += 1;
            }
            index += 1;
            continue;
        }
        kept.push(String::from(token));
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
///  Removes `-a`/`--all` from the commit region, keeping clustered short options.
/// Why:
///   The private index already holds every tracked change, so native commit must not restage worktree bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutAllFlag(['-am', 'x']); // ['-m', 'x']
/// ```
pub fn without_all_flag(commit_args: &[String]) -> Vec<String> {
    let normalised: Vec<String> = normalise_commit_args(commit_args);
    let mut kept: Vec<String> = Vec::new();
    let mut value_next: bool = false;
    let mut after_separator: bool = false;
    for token in normalised {
        if after_separator || value_next {
            kept.push(token);
            value_next = false;
            continue;
        }
        if token == PATHSPEC_SEPARATOR {
            after_separator = true;
            kept.push(token);
            continue;
        }
        if token == "--all" {
            continue;
        }
        if token.starts_with('-') && !token.starts_with("--") && token.len() > 1 {
            let letters: Vec<char> = token[1..].chars().collect();
            let value_start: Option<usize> = letters
                .iter()
                .position(|letter| { return OPTIONAL_VALUE_LETTERS.contains(letter); });
            let flags: &[char] = match value_start {
                Some(index) => &letters[..index],
                None => &letters[..],
            };
            let mut cluster: String = String::from("-");
            for letter in flags.iter().filter(|letter| { return **letter != 'a'; }) {
                cluster.push(*letter);
            }
            if let Some(index) = value_start {
                for letter in &letters[index..] {
                    cluster.push(*letter);
                }
            }
            if cluster == "-" {
                continue;
            }
            value_next = COMMIT_VALUE_OPTIONS.contains(&cluster.as_str());
            kept.push(cluster);
            continue;
        }
        value_next = COMMIT_VALUE_OPTIONS.contains(&token.as_str());
        kept.push(token);
    }
    return kept;
}

/// What:
///  Removes the pathspec-file controls and the explicit pathspecs,
///  then the `--` separator and `-o`/`--only`, from the commit region.
/// Why:
///   The private index already holds the selected paths, so the real commit names none of them.
///  The pathspec removal runs from the last pathspec backwards with `lastIndexOf`, as the incumbent does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateExplicitCommitArgs({ args, pathspecs });
/// ```
pub fn private_explicit_commit_args(args: &[String], pathspecs: &[String]) -> Vec<String> {
    let mut without_file: Vec<String> = Vec::new();
    let mut index: usize = 0;
    while index < args.len() {
        let token: &str = args[index].as_str();
        if token == "--pathspec-from-file" {
            index += 2;
            continue;
        }
        if token.starts_with("--pathspec-from-file=") || token == "--pathspec-file-nul" {
            index += 1;
            continue;
        }
        without_file.push(String::from(token));
        index += 1;
    }
    for pathspec in pathspecs.iter().rev() {
        if let Some(position) = without_file.iter().rposition(|token| { return token == pathspec; }) {
            without_file.remove(position);
        }
    }
    return without_file
        .into_iter()
        .filter(|token| { return token != "--" && token != "-o" && token != "--only"; })
        .collect();
}

/// What:
///  Removes the selection controls once the private index holds the exact user choice.
/// Why:
///   `-i`, `-p`, `--include`, `--interactive` and `--patch` already ran against the private index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateSelectionCommitArgs({ args, pathspecs });
/// ```
pub fn private_selection_commit_args(args: &[String], pathspecs: &[String]) -> Vec<String> {
    return private_explicit_commit_args(args, pathspecs)
        .into_iter()
        .filter(|token| { return !["-i", "-p", "--include", "--interactive", "--patch"].contains(&token.as_str()); })
        .collect();
}

/// What:
///  Chooses the arguments of the real `git commit` for the commit mode.
/// Why:
///   Explicit-path commits drop their pathspecs; index commits drop selection controls only
///  when selection ran privately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolvePrivateCommitArgs({ args, pathspecs, mode, selectedPrivately });
/// ```
pub fn resolve_private_commit_args(
    args: &[String],
    pathspecs: &[String],
    mode: TransactionMode,
    selected_privately: bool,
) -> Vec<String> {
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
