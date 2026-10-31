//! What:
//! The rule that refuses a recursive `grep` (and `egrep` and `fgrep`)
//! that would start from a repository root or the home directory or the filesystem root:
//! `-r` / `-R` / `--recursive` / `--dereference-recursive` / `-d recurse`
//! with no file operand (which searches the current directory)
//! or an operand that resolves to one of those places.
//! Why:
//! Such a walk reads hundreds of thousands of files and stalls the NVMe for every session.
//! A narrower directory or `rg PATTERN PATH` does the same job bounded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // policy-grep.ts
//! export function grepRefusal(name, args, allowWide, workingDirectory, home): string | undefined { ... }
//! ```

/// Imports `Path`,
/// the borrowed read-only path type.
use std::path::Path;

/// Imports the escape-hatch variable name for the refusal message.
use crate::constants::ALLOW_WIDE_VARIABLE;

/// Imports the walk-start classifier and the words the refusal message uses for each case.
use crate::walk_target::{WalkTarget, classify, phrase};

/// What:
/// `const VALUE_SHORT_LETTERS: &str = "..."` is a constant text slice with grep's one-letter flags that take a
/// value (`-A3` / `-e pattern` / `-d recurse` / `-m 5`).
/// Why:
/// A value is not a pattern or a file operand,
/// so the parser must skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const VALUE_SHORT_LETTERS = 'ABCDdefm';
/// ```
const VALUE_SHORT_LETTERS: &str = "ABCDdefm";

/// The long flags that take a value in the next argument (or inline after `=`).
const VALUE_LONG_FLAGS: &[&str] = &[
    "--after-context",
    "--before-context",
    "--context",
    "--binary-files",
    "--devices",
    "--directories",
    "--exclude",
    "--exclude-from",
    "--exclude-dir",
    "--file",
    "--group-separator",
    "--include",
    "--label",
    "--max-count",
    "--regexp",
];

/// What:
/// `pub struct GrepDescription { ... }` declares a record with whether the call recurses and the file operands it
/// names.
/// `Vec<String>` is an owned growable list of owned texts (the sibling `&[String]` is a borrowed view).
/// `#[derive(Debug, PartialEq)]` generates printing and `==` for the tests.
/// Why:
/// The refusal needs both facts.
/// It is `Vec` and not a borrow because the list is built by the parser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GrepDescription = { recursive: boolean; operands: string[] };
/// ```
#[derive(Debug, PartialEq)]
pub struct GrepDescription {
    /// Whether any flag makes grep descend into directories.
    pub recursive: bool,
    /// The file operands (the pattern is not one of them).
    pub operands: Vec<String>,
}

/// What:
/// `pub fn describe_grep(arguments: &[String]) -> GrepDescription` declares a public function that reads grep's
/// argument grammar just enough to know whether it recurses and which operands it names.
/// Why:
/// The refusal rule is stated in those two terms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function describeGrep(args: readonly string[]): GrepDescription { ... }
/// ```
pub fn describe_grep(arguments: &[String]) -> GrepDescription {
    let mut recursive: bool = false;
    let mut pattern_from_flag: bool = false;
    let mut positionals: Vec<String> = Vec::new();
    let mut after_double_dash: bool = false;
    let mut index: usize = 0;

    while index < arguments.len() {
        let token: &String = &arguments[index];
        index += 1;

        if after_double_dash {
            positionals.push(token.clone());
            continue;
        }
        if token == "--" {
            after_double_dash = true;
            continue;
        }

        if token.starts_with("--") {
            // `split_once('=')` cuts the flag at the first `=`; an `=` means the value is inline.
            let mut name: &str = token.as_str();
            let mut inline_value: &str = "";
            let has_inline_value: bool = token.contains('=');
            if let Some((before, after)) = token.split_once('=') {
                name = before;
                inline_value = after;
            }
            if name == "--recursive" || name == "--dereference-recursive" {
                recursive = true;
            }
            if name == "--regexp" || name == "--file" {
                pattern_from_flag = true;
            }
            if name == "--directories" {
                // The value is inline (`--directories=recurse`) or the next argument.
                let mut value: &str = inline_value;
                if !has_inline_value && index < arguments.len() {
                    value = arguments[index].as_str();
                }
                if value == "recurse" {
                    recursive = true;
                }
            }
            if VALUE_LONG_FLAGS.contains(&name) && !has_inline_value {
                index += 1;
            }
            continue;
        }

        if token.starts_with('-') && token.len() > 1 {
            let letters: Vec<char> = token.chars().skip(1).collect();
            let mut position: usize = 0;
            while position < letters.len() {
                let letter: char = letters[position];
                if letter == 'r' || letter == 'R' {
                    recursive = true;
                }
                if letter == 'e' || letter == 'f' {
                    pattern_from_flag = true;
                }
                if VALUE_SHORT_LETTERS.contains(letter) {
                    // The value is the rest of this token, or the next argument when the letter ends the token.
                    let mut value: String = letters[position + 1..].iter().collect();
                    if value.is_empty() && index < arguments.len() {
                        value = arguments[index].clone();
                    }
                    if letter == 'd' && value == "recurse" {
                        recursive = true;
                    }
                    if position == letters.len() - 1 {
                        index += 1;
                    }
                    break;
                }
                position += 1;
            }
            continue;
        }

        positionals.push(token.clone());
    }

    // The first positional is the pattern unless a flag supplied the pattern; the rest are file operands.
    let mut operands: Vec<String> = positionals.clone();
    if !pattern_from_flag && !operands.is_empty() {
        operands.remove(0);
    }
    return GrepDescription { recursive, operands };
}

/// What:
/// `pub fn grep_refusal(...) -> Option<String>` declares a public function returning `Some(message)` when the call
/// must be refused and `None` when it may run.
/// Why:
/// The shim calls it for `grep` and `egrep` and `fgrep` before launching anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function grepRefusal(name, args, allowWide, workingDirectory, home): string | undefined { ... }
/// ```
pub fn grep_refusal(
    name: &str,
    arguments: &[String],
    allow_wide: bool,
    working_directory: &Path,
    home: &Path,
) -> Option<String> {
    if allow_wide {
        return None;
    }
    let description: GrepDescription = describe_grep(arguments);
    if !description.recursive {
        return None;
    }

    // With no file operand a recursive grep searches the current directory.
    let mut starts: Vec<String> = description.operands.clone();
    if starts.is_empty() {
        starts.push(".".to_string());
    }

    for start in &starts {
        let target: WalkTarget = classify(start, working_directory, home);
        if target == WalkTarget::Bounded {
            continue;
        }
        let message: String = format!(
            "search-shadow: refused `{} {}`. A recursive search of {} (from `{}`) reads hundreds of thousands of files (.git, .cache, .claude/worktrees, node_modules and target among them), which stalls this machine's disk for every session. Name a narrower directory, for example `{} -r PATTERN package/NAME`, search with `rg PATTERN PATH`, or set {}=1 for one deliberate full walk.",
            name,
            arguments.join(" "),
            phrase(&target),
            start,
            name,
            ALLOW_WIDE_VARIABLE,
        );
        return Some(message);
    }
    return None;
}
