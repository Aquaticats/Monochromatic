//! What:
//! The rule that refuses a `find` (and its clone `bfs`) that starts from a repository root
//! or the home directory or the filesystem root,
//! with no `-maxdepth` of 3 or less.
//! No start point means the current directory.
//! Why:
//! An unbounded walk of those places reads hundreds of thousands of files and stalls the NVMe for every session.
//! A `-maxdepth` of 3 or a narrower start directory or `rg --files PATH` does the same job bounded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // policy-find.ts
//! export function findRefusal(name, args, allowWide, workingDirectory, home): string | undefined { ... }
//! ```

/// Imports `Path`,
/// the borrowed read-only path type.
use std::path::Path;

/// Imports the escape-hatch variable name for the refusal message.
use crate::constants::ALLOW_WIDE_VARIABLE;

/// Imports the walk-start classifier and the words the refusal message uses for each case.
use crate::walk_target::{WalkTarget, classify, phrase};

/// What:
/// `const MAX_BOUNDED_DEPTH: u64 = 3;` is a constant unsigned 64-bit integer
/// (the siblings are `u32` / `usize` / `i64`).
/// Why:
/// A `-maxdepth` up to this value keeps a walk of a repository root small enough to allow.
/// It is `u64` because the value is parsed from text with `parse::<u64>()` and needs no cast.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_BOUNDED_DEPTH = 3;
/// ```
const MAX_BOUNDED_DEPTH: u64 = 3;

/// The options that make `find` and `bfs` print their version or usage text instead of walking (`-version` and `-help` are
/// the single-dash spellings GNU find accepts).
const INFORMATION_FLAGS: &[&str] = &["--version", "--help", "-version", "-help"];

/// The leading one-token options of `find` and `bfs` that come before the start points
/// (`-H` / `-L` / `-P` / `-E` / `-X` and so on).
const LEADING_FLAGS: &[&str] = &["-H", "-L", "-P", "-E", "-X", "-d", "-s", "-x"];

/// What:
/// `pub fn find_start_points(arguments: &[String]) -> Vec<String>` declares a public function returning the start
/// directories named before the first expression (`-name ...` / `(` / `!` / `,`).
/// Why:
/// The refusal needs to know where the walk begins,
/// and `find` with none starts at `.`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findStartPoints(args: readonly string[]): string[] { ... }
/// ```
pub fn find_start_points(arguments: &[String]) -> Vec<String> {
    let mut index: usize = 0;

    // Skip the leading global options; `-D` and `-O` carry a value or a level.
    while index < arguments.len() {
        let token: &str = arguments[index].as_str();
        if LEADING_FLAGS.contains(&token) {
            index += 1;
            continue;
        }
        if token == "-D" {
            index += 2;
            continue;
        }
        if token.starts_with("-O") {
            index += 1;
            continue;
        }
        break;
    }

    let mut starts: Vec<String> = Vec::new();
    while index < arguments.len() {
        let token: &str = arguments[index].as_str();
        if token.starts_with('-') || token == "(" || token == "!" || token == "," {
            break;
        }
        starts.push(token.to_string());
        index += 1;
    }
    if starts.is_empty() {
        starts.push(".".to_string());
    }
    return starts;
}

/// What:
/// `pub fn find_max_depth(arguments: &[String]) -> Option<u64>` declares a public function returning the smallest
/// `-maxdepth` value given or `None` when there is none.
/// `Option<u64>` is `Some(number)` or `None` (Rust's replacement for `number | undefined`).
/// Why:
/// A small `-maxdepth` bounds the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findMaxDepth(args: readonly string[]): number | undefined { ... }
/// ```
pub fn find_max_depth(arguments: &[String]) -> Option<u64> {
    let mut smallest: Option<u64> = None;
    let mut index: usize = 0;
    while index + 1 < arguments.len() {
        if arguments[index] == "-maxdepth" {
            // `parse::<u64>()` turns text into a number as a `Result`; `.is_ok()` asks whether it worked.
            let parsed = arguments[index + 1].parse::<u64>();
            if parsed.is_ok() {
                let depth: u64 = parsed.unwrap_or_default();
                if smallest.is_none() || depth < smallest.unwrap_or_default() {
                    smallest = Some(depth);
                }
            }
        }
        index += 1;
    }
    return smallest;
}

/// What:
/// `pub fn find_refusal(...) -> Option<String>` declares a public function returning `Some(message)` when the call
/// must be refused and `None` when it may run.
/// Why:
/// The shim calls it for `find` and `bfs` before launching anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findRefusal(name, args, allowWide, workingDirectory, home): string | undefined { ... }
/// ```
pub fn find_refusal(
    name: &str,
    arguments: &[String],
    allow_wide: bool,
    working_directory: &Path,
    home: &Path,
) -> Option<String> {
    if allow_wide {
        return None;
    }
    // `--version` and `--help` print text and never walk, so they are never refused (with no start point they would
    // otherwise look like a walk of the current directory).
    for argument in arguments {
        if INFORMATION_FLAGS.contains(&argument.as_str()) {
            return None;
        }
    }
    let depth: Option<u64> = find_max_depth(arguments);
    if depth.is_some() && depth.unwrap_or_default() <= MAX_BOUNDED_DEPTH {
        return None;
    }

    for start in find_start_points(arguments) {
        let target: WalkTarget = classify(&start, working_directory, home);
        if target == WalkTarget::Bounded {
            continue;
        }
        let message: String = format!(
            "search-shadow: refused `{} {}`. Walking {} (from `{}`) without `-maxdepth {}` or less reads hundreds of thousands of files (.git, .cache, .claude/worktrees, node_modules and target among them), which stalls this machine's disk for every session. Add `-maxdepth {}`, start from a narrower directory, list files with `rg --files PATH`, or set {}=1 for one deliberate full walk.",
            name,
            arguments.join(" "),
            phrase(&target),
            start,
            MAX_BOUNDED_DEPTH,
            MAX_BOUNDED_DEPTH,
            ALLOW_WIDE_VARIABLE,
        );
        return Some(message);
    }
    return None;
}
