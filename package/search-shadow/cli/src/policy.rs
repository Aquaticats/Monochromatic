//! What:     The rules that refuse two ripgrep invocations. First, the short flag `-r` anywhere (including in clusters
//!           like `-rl`): in ripgrep it means `--replace`, not grep's "recursive". Second, a repository-wide
//!           unrestricted walk: a flag that includes hidden or ignored files (`--hidden`, `--no-ignore*`, `-u`, `-.`),
//!           no search path, run from a repository root.
//! Why:      `rg -rl pattern` does not list files: it replaces every match with the text `l` in the output, silently.
//!           And a wide walk reads hundreds of thousands of files (`.git`, `.cache`, `.claude/worktrees`,
//!           `node_modules`, `target`) and stalls the NVMe for every session on the machine. Naming a path makes the
//!           walk bounded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // policy.ts
//! export function refusalFor(input: Invocation): string | undefined { ... }
//! ```

/// Imports `PathBuf`, the owned path type, for the working directory and home of an invocation.
use std::path::PathBuf;

use crate::constants::ALLOW_WIDE_VARIABLE;
use crate::policy_find::find_refusal;
use crate::policy_grep::grep_refusal;

/// What:     `const WIDE_LONG_FLAGS: &[&str] = &[...]` is a private constant list of ripgrep's long flags that widen a
///           walk to hidden or ignored files. `&[&str]` is a borrowed read-only list of string slices (siblings:
///           `Vec<&str>`, `[&str; 9]`).
/// Why:      The parser checks every `--flag` against it; a constant slice and not a `Vec` because it never changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WIDE_LONG_FLAGS: readonly string[] = ['--hidden', '--no-ignore', /* ... */];
/// ```
const WIDE_LONG_FLAGS: &[&str] = &[
    "--hidden",
    "--no-ignore",
    "--no-ignore-vcs",
    "--no-ignore-parent",
    "--no-ignore-global",
    "--no-ignore-dot",
    "--no-ignore-exclude",
    "--no-ignore-files",
    "--unrestricted",
];

/// The long flags that only print information (help, version, lists), so there is no walk to refuse.
const INFORMATIONAL_LONG_FLAGS: &[&str] = &[
    "--help",
    "--version",
    "--type-list",
    "--pcre2-version",
    "--generate",
];

/// The long flags that take a value in the next argument (`--glob '*.ts'`); with `=` the value is inline instead.
const VALUE_LONG_FLAGS: &[&str] = &[
    "--after-context",
    "--before-context",
    "--context",
    "--regexp",
    "--file",
    "--glob",
    "--iglob",
    "--type",
    "--type-not",
    "--type-add",
    "--type-clear",
    "--max-count",
    "--max-depth",
    "--maxdepth",
    "--max-filesize",
    "--max-columns",
    "--threads",
    "--replace",
    "--encoding",
    "--sort",
    "--sortr",
    "--color",
    "--colors",
    "--engine",
    "--field-context-separator",
    "--field-match-separator",
    "--path-separator",
    "--pre",
    "--pre-glob",
    "--context-separator",
    "--dfa-size-limit",
    "--regex-size-limit",
    "--ignore-file",
    "--hyperlink-format",
];

/// What:     `const VALUE_SHORT_LETTERS: &str = "..."` is a constant text slice holding the one-letter flags that take a
///           value (`-g '*.ts'`, `-A3`, `-e pattern`).
/// Why:      Inside a cluster such as `-uA3`, the first value letter ends the flags and the rest is its value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const VALUE_SHORT_LETTERS = 'ABCEMTdefgjmrt';
/// ```
const VALUE_SHORT_LETTERS: &str = "ABCEMTdefgjmrt";

/// The one-letter flags that widen a walk: `-u` (unrestricted, repeatable) and `-.` (hidden).
const WIDE_SHORT_LETTERS: &str = "u.";

/// What:     `pub struct WalkDescription { ... }` declares a record type with four public fields (like a TS `type`).
///           `usize` is the unsigned integer wide enough to count items on this machine (siblings: `u32`, `u64`, `i32`).
///           `#[derive(Debug, PartialEq)]` is an attribute that asks the compiler to generate printing and `==`
///           comparison for the type.
/// Why:      The tests compare whole descriptions with `==`. `usize` and not `u32` because a count of arguments is
///           what `Vec::len()` returns, so no casts are needed.
/// Gotcha:   The derive attribute is generated code; it is the one place this file uses a macro-like feature.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkDescription = { widened: boolean; informational: boolean; pathCount: number };
/// ```
#[derive(Debug, PartialEq)]
pub struct WalkDescription {
    /// Whether a flag asked for hidden or ignored files.
    pub widened: bool,
    /// Whether the invocation only prints information (help, version, lists).
    pub informational: bool,
    /// How many search paths the invocation names.
    pub path_count: usize,
    /// Whether the short flag `-r` (ripgrep's `--replace`) appears as a flag, alone or inside a cluster.
    pub short_replace: bool,
}

/// What:     `fn long_flag_name(token: &str) -> &str` declares a private function that takes a borrowed text slice and
///           returns a borrowed slice of it (the same text, cut at the first `=`).
/// Why:      `--glob=*.ts` and `--glob` must both be recognized as `--glob`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function longFlagName(token: string): string { return token.split('=')[0]; }
/// ```
fn long_flag_name(token: &str) -> &str {
    // What:     `if let Some((before, _)) = token.split_once('=')` runs the block only when the text contains `=`.
    //           `split_once` returns an `Option` of a pair; `Some((before, _))` unwraps it, naming the left piece
    //           `before` and discarding the right one with `_`.
    // Why:      Keep only the flag name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = token.indexOf('=');
    // if (index !== -1) return token.slice(0, index);
    // ```
    if let Some((before, _)) = token.split_once('=') {
        return before;
    }
    return token;
}

/// What:     `pub fn describe_walk(arguments: &[String]) -> WalkDescription` declares a public function. `&[String]` is a
///           borrowed read-only view of a list of owned texts (siblings: `Vec<String>` owns the list, `&[&str]` views
///           borrowed texts).
/// Why:      It reads rg's argument grammar just enough to know whether a flag widens the walk and how many search
///           paths are named. `&[String]` and not `Vec<String>` because it only reads and the caller keeps the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function describeWalk(args: readonly string[]): WalkDescription { ... }
/// ```
pub fn describe_walk(arguments: &[String]) -> WalkDescription {
    // What:     `let mut widened = false;` creates a variable that CAN be reassigned (`mut`); plain `let` is like
    //           `const`. The same applies to the six variables that follow.
    // Why:      The loop below records what it saw.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let widened = false;
    // ```
    let mut widened: bool = false;
    let mut informational: bool = false;
    let mut pattern_from_flag: bool = false;
    let mut files_mode: bool = false;
    let mut short_replace: bool = false;
    let mut positional_count: usize = 0;
    let mut after_double_dash: bool = false;
    let mut index: usize = 0;

    // What:     `while index < arguments.len()` loops while the position is inside the list.
    // Why:      A manual index lets the loop skip the value that follows a flag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (index < args.length) {
    // ```
    while index < arguments.len() {
        // What:     `let token: &String = &arguments[index];` borrows (`&`) the element at `index` without copying it.
        // Why:      Look at the argument without taking it from the list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const token = args[index];
        // ```
        let token: &String = &arguments[index];
        index += 1;

        // After `--` everything is a positional argument.
        if after_double_dash {
            positional_count += 1;
            continue;
        }
        if token == "--" {
            after_double_dash = true;
            continue;
        }

        if token.starts_with("--") {
            // `long_flag_name(token)` borrows `token` as text and returns the flag name without any `=value`.
            let name: &str = long_flag_name(token);
            let has_inline_value: bool = token.contains('=');

            // What:     `WIDE_LONG_FLAGS.contains(&name)` asks whether the list holds `name`; `&name` lends the text
            //           because `contains` compares by reference.
            // Why:      Record that this flag widens the walk.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (WIDE_LONG_FLAGS.includes(name)) widened = true;
            // ```
            if WIDE_LONG_FLAGS.contains(&name) {
                widened = true;
            }
            if INFORMATIONAL_LONG_FLAGS.contains(&name) {
                informational = true;
            }
            if name == "--regexp" || name == "--file" {
                pattern_from_flag = true;
            }
            if name == "--files" {
                files_mode = true;
            }
            // A flag that takes a value without `=` consumes the next argument too.
            if VALUE_LONG_FLAGS.contains(&name) && !has_inline_value {
                index += 1;
            }
            continue;
        }

        if token.starts_with('-') && token.len() > 1 {
            // What:     `token.chars().skip(1).collect()` walks the characters, skips the leading `-`, and gathers
            //           the rest into a `Vec<char>` (an owned growable list of characters). The `: Vec<char>`
            //           annotation names the result type; `collect()` needs it to know what to build.
            // Why:      A short cluster like `-uA3` is a run of one-letter flags.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const letters: string[] = [...token].slice(1);
            // ```
            let letters: Vec<char> = token.chars().skip(1).collect();
            let mut position: usize = 0;
            while position < letters.len() {
                let letter: char = letters[position];
                if WIDE_SHORT_LETTERS.contains(letter) {
                    widened = true;
                }
                if letter == 'h' || letter == 'V' {
                    informational = true;
                }
                if letter == 'e' || letter == 'f' {
                    pattern_from_flag = true;
                }
                // `-r` takes the rest of its cluster as its value, so `-rl` replaces matches with `l`.
                if letter == 'r' {
                    short_replace = true;
                }
                if VALUE_SHORT_LETTERS.contains(letter) {
                    // The value is the rest of this token, or the next argument when this letter ends the token.
                    if position == letters.len() - 1 {
                        index += 1;
                    }
                    break;
                }
                position += 1;
            }
            continue;
        }

        positional_count += 1;
    }

    // With `--files`, or a pattern given by `-e` or `-f`, every positional is a path; otherwise the first is the pattern.
    let mut path_count: usize = positional_count;
    if !files_mode && !pattern_from_flag && positional_count > 0 {
        path_count = positional_count - 1;
    }

    // What:     `WalkDescription { widened, informational, path_count }` builds the record from the variables of the
    //           same names (shorthand for `widened: widened`).
    // Why:      Hand the result to the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { widened, informational, pathCount: pathCount };
    // ```
    return WalkDescription {
        widened,
        informational,
        path_count,
        short_replace,
    };
}

/// What:     `pub fn rg_refusal(...) -> Option<String>` declares a public function returning `Some(message)` when the
///           invocation must be refused and `None` when it may run. `String` is the owned text type (sibling `&str` is
///           the borrowed view).
/// Why:      `refusal_for` calls it for `rg`. `String` and not `&str` because the message is built here and outlives
///           this function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rgRefusal(name, args, allowWide, inRepositoryRoot, stdinHasData, workingDirectory): string | undefined
/// ```
pub fn rg_refusal(
    name: &str,
    arguments: &[String],
    allow_wide: bool,
    in_repository_root: bool,
    stdin_has_data: bool,
    working_directory: &str,
) -> Option<String> {
    // Only ripgrep is policed for now; grep and find are only slowed down.
    if name != "rg" {
        return None;
    }

    let walk: WalkDescription = describe_walk(arguments);

    // The `-r` ban applies everywhere and has no escape hatch: the long `--replace` stays available for real replacement.
    if walk.short_replace {
        let replace_message: String = format!(
            "search-shadow: refused `rg {}`. In ripgrep `-r` is `--replace`, not grep's \"recursive\" flag: it rewrites each match in the output (`rg -rl PATTERN` prints the text `l` in place of every match, and lists no files). ripgrep already searches directories recursively, so drop `-r`; use `-l` to list file names, or write `--replace TEXT` if you really mean replacement.",
            arguments.join(" "),
        );
        return Some(replace_message);
    }

    // The escape hatch for one deliberate full walk.
    if allow_wide {
        return None;
    }
    // Outside a repository root a walk is bounded by the directory itself.
    if !in_repository_root {
        return None;
    }
    // With piped input and no path, ripgrep reads the input and walks nothing.
    if stdin_has_data {
        return None;
    }

    if walk.informational || !walk.widened || walk.path_count > 0 {
        return None;
    }

    // What:     `format!("...{}...", value)` builds an owned `String` by filling each `{}` with the next value (a
    //           macro, which is why it ends in `!`).
    // Why:      The message names the affected command and directory, then every way to proceed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const message = `search-shadow: refused ...${workingDirectory}...`;
    // ```
    let message: String = format!(
        "search-shadow: refused `rg {}`. With hidden or ignored files included and no search path, it would walk all of {} (.git, .cache, .claude/worktrees, node_modules and target hold hundreds of thousands of files), which stalls this machine's disk for every session. Name a narrower path, for example `rg --hidden PATTERN .github`, or set {}=1 for one deliberate full walk.",
        arguments.join(" "),
        working_directory,
        ALLOW_WIDE_VARIABLE,
    );

    // What:     `Some(message)` wraps the text in the "present" case of `Option`.
    // Why:      Tell the shadow to refuse and print this message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return message;
    // ```
    return Some(message);
}

/// What:     `pub struct Invocation { ... }` declares a record holding everything a policy may look at. Every field is OWNED
///           (`String`, `Vec<String>`, `PathBuf`) and not borrowed (`&str`, `&[String]`, `&Path`).
/// Why:      One value to pass instead of a long list of arguments. Owned fields mean no lifetime annotations, which have no
///           TypeScript analogue; copying a few short texts costs nothing next to the search that follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Invocation = { name: string; args: string[]; allowWide: boolean; stdinHasData: boolean; workingDirectory: string; home: string };
/// ```
pub struct Invocation {
    /// The tool name the binary was started under (`rg`, `grep`, `find`, ...).
    pub name: String,
    /// The tool's arguments as lossy text.
    pub arguments: Vec<String>,
    /// Whether the escape-hatch variable is set to `1`.
    pub allow_wide: bool,
    /// Whether standard input is a pipe, file or socket (so a search may read it and walk nothing).
    pub stdin_has_data: bool,
    /// The directory the tool will run in.
    pub working_directory: PathBuf,
    /// The user's home directory.
    pub home: PathBuf,
}

/// What:     `pub fn refusal_for(invocation: &Invocation) -> Option<String>` declares the public entry the shim calls.
///           `&Invocation` lends the record without giving it away.
/// Why:      It picks the policy by tool name: `rg`, the `grep` family, or `find` and its clone `bfs`. Every other shadowed
///           tool is only slowed down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function refusalFor(invocation: Invocation): string | undefined { ... }
/// ```
pub fn refusal_for(invocation: &Invocation) -> Option<String> {
    let name: &str = invocation.name.as_str();

    if name == "rg" {
        let in_repository_root: bool = invocation.working_directory.join(".git").exists();
        let directory_text: String = invocation.working_directory.to_string_lossy().to_string();
        return rg_refusal(
            name,
            &invocation.arguments,
            invocation.allow_wide,
            in_repository_root,
            invocation.stdin_has_data,
            &directory_text,
        );
    }
    if name == "grep" || name == "egrep" || name == "fgrep" {
        return grep_refusal(
            name,
            &invocation.arguments,
            invocation.allow_wide,
            &invocation.working_directory,
            &invocation.home,
        );
    }
    if name == "find" || name == "bfs" {
        return find_refusal(
            name,
            &invocation.arguments,
            invocation.allow_wide,
            &invocation.working_directory,
            &invocation.home,
        );
    }
    return None;
}
