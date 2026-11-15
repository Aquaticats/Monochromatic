//! What:
//!  The generated comparison of every guarded subcommand's option table with real Git 2.56.0:
//!  the long options and negations the binary prints for itself,
//!  and whether each row takes the token after it as its value.
//! Why:
//!  The tables are copied from Git's C source by hand,
//!  and a copy is only trustworthy when the binary itself is the oracle:
//!  a row whose name,
//!  arity or negation the guard got wrong is a word read differently from Git,
//!  which is exactly the defect the seventh review found five times
//!  (`restore -s -h`, a pathspec file named `--no-force`, `--no-sparse` and `--staged`,
//!  and the clean's force count).
//!  A row this control would catch is one the guard reads as a value where Git reads an option,
//!  or the other way round.
//!  The comparison is generated from the tables,
//!  so an option added to a table is probed the moment it is added.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for each guarded subcommand: renderCompletion(TABLE) === await git([...words, '--git-completion-helper-all'])
//! // for each row: didGitConsume(sentinel) === didTheTableConsume(sentinel)
//! ```

/// The tables of every guarded subcommand.
use crate::command_checkout_index_table::CHECKOUT_INDEX_TABLE;
use crate::command_checkout_table::{CHECKOUT_TABLE, SWITCH_TABLE};
use crate::command_clean::CLEAN_TABLE;
use crate::command_discard::READ_TREE_TABLE;
use crate::command_mv_table::MV_TABLE;
use crate::command_options::{DEFAULT_MODE, OptionSpec, parse_options};
use crate::command_options_query::positional_tokens;
use crate::command_reset::RESET_TABLE;
use crate::command_restore_table::RESTORE_TABLE;
use crate::command_rm_table::RM_TABLE;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{assert_table_invariants, git_status, os_arguments, remove, repository_with_tracked_file};
use crate::command_worktree_remove_table::WORKTREE_REMOVE_TABLE;
use std::path::PathBuf;

/// One guarded subcommand: the words real Git is started with, and the table the guard reads it by.
///  A `struct` is a record with named fields.
/// Why:
///  Both controls walk this list,
///  so a table added for a new guarded word is compared as soon as the word is named here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Guarded = { words: ['clean']; table: CLEAN_TABLE };
/// ```
struct Guarded {
    /// The subcommand words, as Git is started with them.
    words: &'static [&'static str],
    /// The table the guard reads that subcommand's region by.
    table: &'static [OptionSpec],
}

/// Every guarded subcommand whose region the guard reads from a table of its own.
/// Why:
///  These are the commands of "Discarding commands" that take options the guard must read;
///  the abort and skip readings ask only whether a word is there,
///  and no table takes part in that question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GUARDED = [{ words: ['clean'], table: CLEAN_TABLE }, /* ... */];
/// ```
const GUARDED: &[Guarded] = &[
    Guarded { words: &["clean"], table: CLEAN_TABLE },
    Guarded { words: &["read-tree"], table: READ_TREE_TABLE },
    Guarded { words: &["reset"], table: RESET_TABLE },
    Guarded { words: &["restore"], table: RESTORE_TABLE },
    Guarded { words: &["rm"], table: RM_TABLE },
    Guarded { words: &["mv"], table: MV_TABLE },
    Guarded { words: &["checkout-index"], table: CHECKOUT_INDEX_TABLE },
    Guarded { words: &["checkout"], table: CHECKOUT_TABLE },
    Guarded { words: &["switch"], table: SWITCH_TABLE },
    Guarded { words: &["worktree", "remove"], table: WORKTREE_REMOVE_TABLE },
];

/// The spellings one row is probed by: its short letter and its long name.
/// Why:
///  Git reads the two through different functions (`parse_short_opt` and `parse_long_opt`),
///  so a row is only compared whole when both spellings are asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spellings = (row) => [row.short && `-${row.short}`, row.long && `--${row.long}`].filter(Boolean);
/// ```
fn spellings(spec: &OptionSpec) -> Vec<String> {
    let mut written: Vec<String> = Vec::<String>::new();
    if let Some(short) = spec.short {
        written.push(format!("-{}", char::from(short)));
    }
    if let Some(long) = spec.long {
        written.push(format!("--{long}"));
    }
    return written;
}

/// What:
///  The list real Git prints for a guarded subcommand is exactly the table's own list.
/// Why:
///  `--git-completion-helper-all` makes Git print its table,
///  with `=` after each option that needs a value and the `--no-` form of every option that
///  accepts one,
///  so a name the guard's copy lacks,
///  a value it thinks is needed where Git does not,
///  and a negation it invented are all differences here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(renderCompletion(RM_TABLE)).toBe(await git(['rm', '--git-completion-helper-all']));
/// ```
#[test]
fn every_guarded_table_is_the_list_git_prints() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("discard-tables-completion");
    for guarded in GUARDED {
        assert_table_invariants(guarded.table);
        assert_eq!(
            render_completion(guarded.table),
            git_completion(root.as_path(), guarded.words),
            "git {:?}",
            guarded.words
        );
    }
    remove(directory.as_path());
}

/// What:
///  Each row of each guarded table takes the token after it as its value exactly when real Git does.
/// Why:
///  This is the question every reading of the guard turns on:
///  a word that is a value to Git and an option to the guard turns a force off,
///  a help request on,
///  or a written path away.
///  The token is `--guard-probe`, which real Git answers as an unknown option when the row before
///  it takes no value,
///  and consumes as the value when it does,
///  so one run of each spelling says which one Git did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const row of table) for (const spelling of spellings(row))
///   expect(theTableTakesTheNextToken(row)).toBe(!stderr.includes('unknown option'));
/// ```
#[test]
fn every_row_consumes_a_value_exactly_when_git_does() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("discard-tables-values");
    let probe: &str = "--guard-probe";
    for guarded in GUARDED {
        for spec in guarded.table {
            for spelling in spellings(spec) {
                let mut command: Vec<&str> = guarded.words.to_vec();
                command.push(spelling.as_str());
                command.push(probe);
                let observed: std::process::Output = git_status(root.as_path(), command.as_slice());
                let stderr: String = String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
                let git_took_it: bool = !stderr.contains("unknown option") && !stderr.contains("unknown switch");
                let region: Vec<std::ffi::OsString> = os_arguments(&[spelling.as_str(), probe]);
                let ours_took_it: bool = match parse_options(region.as_slice(), guarded.table, DEFAULT_MODE, &[]) {
                    Ok(parsed) => parsed
                        .occurrences
                        .iter()
                        .any(|occurrence| matches!(occurrence.value, Some(crate::command_options::OptionValue::Detached { token: 1 }))),
                    // Git's own refusal of the probe is this table's answer that the row took no value.
                    Err(error) if error.token == 1 => false,
                    // A row whose own spelling the table cannot read is a table defect, not an answer.
                    Err(error) => panic!("{command:?}: the table refuses its own row: {error}"),
                };
                assert_eq!(
                    ours_took_it, git_took_it,
                    "{command:?}: the table says the next token is a value: {ours_took_it}, and Git says {git_took_it} ({stderr})"
                );
                // A word the table does not hold would be read as a positional here; the row's own spelling is not one.
                if let Ok(parsed) = parse_options(region.as_slice(), guarded.table, DEFAULT_MODE, &[]) {
                    assert!(
                        !positional_tokens(&parsed, region.len()).contains(&0),
                        "{command:?}: the spelling is no option of this table"
                    );
                }
            }
        }
    }
    remove(directory.as_path());
}
