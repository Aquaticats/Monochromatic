//! What:
//! The rule that refuses a `find` (and its clone `bfs`) that starts from a repository root,
//! the home directory or the filesystem root,
//! with no `-maxdepth` of 3 or less.
//! No start point means the current directory.
//! Why:
//! An unbounded walk of those places reads hundreds of thousands of files and stalls the NVMe for every session.
//! A `-maxdepth` of 3,
//! a narrower start directory or `rg --files PATH` does the same job bounded.
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
/// `const VALUE_FLAGS: &[&str]` is a private constant list of the `find` and `bfs` options that take the argument
/// after them as a value (a pattern, a number, a path, a name), never as an option of their own.
/// `&[&str]` is a borrowed read-only list of string slices (the siblings are `Vec<&str>` / `[&str; 36]`).
/// Why:
/// The next argument of `find . -name --help` is the pattern spelled `--help`,
/// and a raw scan that counts it as `find`'s own help flag would let an unbounded walk of `/` start.
/// It is a constant slice and not a `Vec` because it never changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const VALUE_FLAGS: readonly string[] = ['-name', '-type', /* ... */];
/// ```
const VALUE_FLAGS: &[&str] = &[
    "-D",
    "-amin",
    "-anewer",
    "-atime",
    "-cmin",
    "-cnewer",
    "-context",
    "-ctime",
    "-files0-from",
    "-fls",
    "-fprint",
    "-fprint0",
    "-fprintf",
    "-fstype",
    "-gid",
    "-group",
    "-ilname",
    "-iname",
    "-inum",
    "-ipath",
    "-iregex",
    "-iwholename",
    "-links",
    "-lname",
    "-maxdepth",
    "-mindepth",
    "-mmin",
    "-mtime",
    "-name",
    "-newer",
    "-path",
    "-perm",
    "-printf",
    "-regex",
    "-regextype",
    "-samefile",
    "-size",
    "-type",
    "-uid",
    "-used",
    "-user",
    "-wholename",
    "-xtype",
];

/// What:
/// `fn flag_takes_a_value(token: &str) -> bool` declares a private function that answers whether the option in
/// `token` consumes the argument after it.
/// Why:
/// `-newerXY` is a family of options (`-neweram` / `-newerct` and so on) that no single list member spells,
/// so the family is recognized by its start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flagTakesAValue = (token: string): boolean => token.startsWith('-newer') || VALUE_FLAGS.includes(token);
/// ```
fn flag_takes_a_value(token: &str) -> bool {
    if token.starts_with("-newer") {
        return true;
    }
    // `VALUE_FLAGS.contains(&token)` asks whether the list holds this text; `&token` lends it because `contains`
    // compares by reference.
    return VALUE_FLAGS.contains(&token);
}

/// What:
/// `fn value_positions(arguments: &[String]) -> Vec<bool>` declares a private function returning one answer per
/// argument: `true` when that argument is the value of the option before it.
/// Why:
/// Both the information-flag exemption and the `-maxdepth` reading need the same answer,
/// and a value must never be read as an option.
/// It takes `&[String]` and not `Vec<String>` because it only reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const valuePositions = (args: readonly string[]): boolean[] => { ... };
/// ```
fn value_positions(arguments: &[String]) -> Vec<bool> {
    // `vec![false; arguments.len()]` makes an owned growable list holding one `false` per argument; the siblings
    // are `[bool; N]` (fixed length) and `&[bool]` (borrowed).
    let mut positions: Vec<bool> = vec![false; arguments.len()];
    let mut index: usize = 0;
    while index + 1 < arguments.len() {
        if flag_takes_a_value(&arguments[index]) {
            // The value is the next argument, and the pair is consumed together, so the value cannot serve as an
            // option of its own either.
            positions[index + 1] = true;
            index += 2;
            continue;
        }
        index += 1;
    }
    // `return positions;` leaves the function with the answers.
    return positions;
}

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
    let values: Vec<bool> = value_positions(arguments);
    let mut smallest: Option<u64> = None;
    let mut index: usize = 0;
    while index + 1 < arguments.len() {
        // A `-maxdepth` that is itself the value of an earlier option (`-name -maxdepth 2`) is a pattern, not the
        // depth option, so it does not bound the walk.
        if arguments[index] == "-maxdepth" && !values[index] {
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
    // otherwise look like a walk of the current directory). A value that happens to spell one of them (`-name --help`)
    // is find's own pattern and does not lift the refusal.
    let values: Vec<bool> = value_positions(arguments);
    let mut index: usize = 0;
    while index < arguments.len() {
        if !values[index] && INFORMATION_FLAGS.contains(&arguments[index].as_str()) {
            return None;
        }
        index += 1;
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
