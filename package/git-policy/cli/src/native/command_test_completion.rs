//! What:
//!  A table oracle built on `git <command> --git-completion-helper-all`.
//! Why:
//!  That request makes the real Git 2.56.0 print every long option of a command,
//!  an `=`
//!      after each required value and every accepted `--no-` form,
//!  so a copied table is
//!      compared with the binary instead of with a second reading of the C source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(renderCompletion(TABLE)).toBe((await git([...command, '--git-completion-helper-all'])).trim());
//! ```

/// Table row types and the real-Git fixture helper.
use super::command_options::{Arity, OptionSpec};
use super::command_test_support::git;
use std::path::Path;
use std::process::Output;

/// Render a table the way Git's `show_gitcomp` prints it (parse-options.c:795-892):
///  every
/// long option,
///  `=` after a required value,
///  then the positive form of each `no-` row,
///  then
/// the `--no-` form of every other negatable row.
pub(crate) fn render_completion(table: &[OptionSpec]) -> String {
    let mut words: Vec<String> = Vec::<String>::new();
    let mut negative_named: usize = 0;
    for spec in table {
        let long: &str = if let Some(spelling) = spec.long {
            spelling
        } else {
            continue;
        };
        let suffix: &str = if spec.arity == Arity::Required {
            "="
        } else {
            ""
        };
        if long.starts_with("no-") {
            negative_named += 1;
        }
        words.push(format!("--{long}{suffix}"));
    }
    for spec in table {
        let long: &str = if let Some(spelling) = spec.long {
            spelling
        } else {
            continue;
        };
        if !spec.negatable {
            continue;
        }
        if let Some(positive) = long.strip_prefix("no-") {
            words.push(format!("--{positive}"));
        }
    }
    let mut separator_printed: bool = false;
    for spec in table {
        let long: &str = if let Some(spelling) = spec.long {
            spelling
        } else {
            continue;
        };
        if !spec.negatable || long.starts_with("no-") {
            continue;
        }
        if negative_named > 0 && !separator_printed {
            words.push(String::from("--"));
            separator_printed = true;
        }
        words.push(format!("--no-{long}"));
        negative_named += 1;
    }
    return words.join(" ");
}

/// Ask the real binary for one command's complete long-option table.
pub(crate) fn git_completion(directory: &Path, command: &[&str]) -> String {
    let mut arguments: Vec<&str> = command.to_vec();
    arguments.push("--git-completion-helper-all");
    let output: Output = git(directory, arguments.as_slice());
    return String::from_utf8(output.stdout)
        .expect("ASCII completion output")
        .trim()
        .to_owned();
}
