//! What:
//!  Every option row of every guarded subcommand's own table,
//!  given to a form that would write over an edit a `skip-worktree` mark hides,
//!  each row against real Git:
//!  an option either changes nothing about what the form writes,
//!  or the guard refuses the form,
//!  and the person's edit is never gone.
//! Why:
//!  A form writes the paths its words name,
//!  and Git's options are part of those words:
//!  `--ignore-skip-worktree-bits` makes three forms write the entry the mark hides,
//!  and the guard read the mark as untouched (the ninth review's first finding).
//!  The rows are generated from the tables the guard parses,
//!  so an option a newer Git adds to a table gets its row the moment it is copied,
//!  with `UNREAD` as its identity until a fact reads it,
//!  and the row is what says whether it needed reading.
//!  A row where Git writes the marked edit and the wrapper lets it through is the defect.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const form of forms) for (const row of form.table)
//! //   expect(wrappedLost(`${form} ${row}`)).toEqual([]);
//! ```

use super::support::{Fixture, Observed, direct, fixture, git, observe, remove, repository, stderr_of, wrapped};
use git_policy_cli::command_checkout_index_table::CHECKOUT_INDEX_TABLE;
use git_policy_cli::command_checkout_table::{CHECKOUT_TABLE, SWITCH_TABLE};
use git_policy_cli::command_discard::READ_TREE_TABLE;
use git_policy_cli::command_mv_table::MV_TABLE;
use git_policy_cli::command_options::{Arity, OptionSpec};
use git_policy_cli::command_reset::RESET_TABLE;
use git_policy_cli::command_restore_table::RESTORE_TABLE;
use git_policy_cli::command_rm_table::RM_TABLE;
use std::path::{Path, PathBuf};

/// The text the person's edit begins with, so a row finds a loss by its content.
const USER_DATA: &str = "USER DATA";

/// One guarded subcommand:
///  the words of a form that would write over the marked edit,
///  and the table the guard reads that form by.
///  A `struct` is a record with named fields.
/// Why:
///  The row's option is inserted right after the subcommand word,
///  where Git reads it as the subcommand's own option,
///  and the form's other words are the shortest command that reaches the edit at `a.txt`.
struct Form {
    /// The subcommand's word, as a row is named by it.
    subcommand: &'static str,
    /// The words after the subcommand, before the row's option is put among them.
    words: &'static [&'static str],
    /// The table the guard reads that subcommand's region by.
    table: &'static [OptionSpec],
}

/// Every guarded subcommand whose form writes a worktree entry, with the form that reaches the edit.
/// Why:
///  `clean` is not here: the owner decided on 2026-11-15 that a clean deletes what its own dry run names
///  and the record names each nested repository it takes,
///  so an option that widens a clean needs no refusal and its observable is the line.
///  `worktree remove` is not here either: the entry it takes is a worktree the person named,
///  and its own controls ask that question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FORMS = [{ subcommand: 'checkout', words: ['-f', 'other'], table: CHECKOUT_TABLE }, ...];
/// ```
const FORMS: &[Form] = &[
    Form { subcommand: "checkout", words: &["-f", "other"], table: CHECKOUT_TABLE },
    // The ninth review's own spelling: a path form whose source is the index.
    Form { subcommand: "checkout", words: &["--", "a.txt"], table: CHECKOUT_TABLE },
    Form { subcommand: "switch", words: &["-f", "other"], table: SWITCH_TABLE },
    Form { subcommand: "restore", words: &["--source=other", "--", "a.txt"], table: RESTORE_TABLE },
    Form { subcommand: "read-tree", words: &["--reset", "-u", "other"], table: READ_TREE_TABLE },
    Form { subcommand: "mv", words: &["-f", "b.txt", "a.txt"], table: MV_TABLE },
    Form { subcommand: "checkout-index", words: &["-f", "--", "a.txt"], table: CHECKOUT_INDEX_TABLE },
    Form { subcommand: "reset", words: &["--hard", "other"], table: RESET_TABLE },
    Form { subcommand: "rm", words: &["-f", "a.txt"], table: RM_TABLE },
];

/// A value that makes one option run in the fixture's own names.
///  `&str` is the option's long or short name.
/// Why:
///  An option with a value is only exercised when the value means something:
///  `--prefix=out/` writes below the worktree's own directory,
///  `--source=other` reads the other tree,
///  and `--pathspec-from-file=list.txt` reads the file the fixture wrote.
///  A value that means nothing is Git's own refusal, which the row reports as such.
fn value_for(name: &str) -> &'static str {
    return match name {
        "prefix" => "out/",
        "source" | "from" | "s" => "other",
        "pathspec-from-file" => "list.txt",
        "exclude" | "e" => "*.log",
        "conflict" => "merge",
        "unified" | "U" | "inter-hunk-context" => "3",
        "orphan" | "create" | "c" | "force-create" | "C" => "another-branch",
        "depth" => "1",
        "git-dir" => ".git",
        _ => "x",
    };
}

/// The words one table row contributes to the form.
fn row_words(spec: &OptionSpec) -> Vec<String> {
    let long: Option<String> = spec.long.map(|name| return format!("--{name}"));
    let short: Option<String> = spec.short.map(|letter| return format!("-{}", char::from(letter)));
    let name: String = long.clone().or_else(|| return short.clone()).unwrap_or_else(|| return String::from("-"));
    let spelled: &str = spec.long.unwrap_or("x");
    return match spec.arity {
        Arity::None => vec![name],
        // A short option's value is the token after it, which Git reads the same way as the attached form.
        Arity::Required if long.is_some() => vec![format!("{name}={}", value_for(spelled))],
        Arity::Required => vec![name, String::from(value_for(spelled))],
        // An optional value is left out, so the option means what it means alone,
        //  and so is a last-argument default, which is valid without a value only when it is last.
        Arity::Optional | Arity::LastArgDefault => vec![name],
    };
}

/// Whether the wrapper itself rejected the command.
fn refused(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// The person's edit, as it stood before the command.
fn the_edit(directory: &Path) -> Option<Vec<u8>> {
    let bytes: Vec<u8> = std::fs::read(directory.join("a.txt")).ok()?;
    return if String::from_utf8_lossy(bytes.as_slice()).starts_with(USER_DATA) { Some(bytes) } else { None };
}

/// One repository whose worktree holds an edit a `skip-worktree` mark hides,
///  a branch `other` whose tree holds a different `a.txt`,
///  and a pathspec file naming it.
fn prepared(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new(name));
    let linked: PathBuf = fixture.root.join(format!("{name}-linked"));
    std::fs::write(repo.join("a.txt"), b"main a\n").expect("a tracked file");
    std::fs::write(repo.join("b.txt"), b"a tracked source\n").expect("a tracked source");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("ignore rules");
    std::fs::write(repo.join("list.txt"), b"a.txt\n").expect("a pathspec file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt", ".gitignore", "list.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=main"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("a.txt"), b"other a\n").expect("the other tree's a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked_text: &str = linked.to_str().expect("a fixture path is text");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked_text]);
    git(fixture, linked.as_path(), &["update-index", "--skip-worktree", "a.txt"]);
    std::fs::write(linked.join("a.txt"), format!("{USER_DATA} the edit\n")).expect("the edit");
    return linked;
}

/// What one arm did: whether the wrapper refused, what Git exited with, and whether the edit is gone.
struct Arm {
    /// What the command exited with.
    code: Option<i32>,
    /// Whether a policy refusal is what the caller saw.
    refused: bool,
    /// Whether the edit the mark hides is gone.
    lost: bool,
}

/// Run one row's words in one arm.
fn run(form: &Form, words: &[String], wrapped_arm: bool, label: &str) -> Arm {
    let fixture: Fixture = fixture(label);
    let directory: PathBuf = prepared(&fixture, "repo");
    let mut borrowed: Vec<String> = vec![String::from(form.subcommand)];
    borrowed.extend(words.iter().cloned());
    borrowed.extend(form.words.iter().map(|word| return String::from(*word)));
    let borrowed_text: Vec<&str> = borrowed.iter().map(|word| return word.as_str()).collect();
    let observed: Observed = if wrapped_arm {
        let mut command = wrapped(&fixture);
        command.current_dir(directory.as_path()).args(borrowed_text.as_slice());
        observe(&mut command, b"")
    } else {
        let mut command = direct(&fixture);
        command.current_dir(directory.as_path()).args(borrowed_text.as_slice());
        observe(&mut command, b"")
    };
    let arm: Arm = Arm {
        code: observed.code,
        refused: refused(&observed),
        lost: the_edit(directory.as_path()).is_none(),
    };
    remove(&fixture);
    return arm;
}

/// What:
///  Every option of every guarded table, in a form that would write over a `skip-worktree` edit:
///  no row leaves the person's edit gone,
///  and every row where Git takes the edit is one the guard refused.
/// Why:
///  A table row is a word Git may read to write more than the form without it,
///  so the row is the proof that the guard reads it, refuses it, or loses nothing by it.
///  The native arm says whether the situation exists at all,
///  so an option Git does nothing with is counted rather than trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const form of FORMS) for (const row of form.table) expect(run(form, row, true).lost).toBe(false);
/// ```
#[test]
fn every_option_of_every_guarded_table_leaves_the_hidden_edit_alone_or_is_refused() {
    let mut rows: usize = 0;
    let mut untouched: usize = 0;
    let mut refused_rows: usize = 0;
    let mut failures: Vec<String> = Vec::<String>::new();
    for form in FORMS {
        for spec in form.table {
            let words: Vec<String> = row_words(spec);
            let name: String = words.join(" ");
            let wrapped_arm: Arm = run(form, words.as_slice(), true, "option-axis-wrapped");
            let native_arm: Arm = run(form, words.as_slice(), false, "option-axis-native");
            rows += 1;
            if native_arm.lost {
                if wrapped_arm.refused {
                    refused_rows += 1;
                } else if wrapped_arm.lost {
                    failures.push(format!("{} {}: Git took the marked edit and the wrapper let it through", form.subcommand, name));
                }
            } else {
                untouched += 1;
            }
            if wrapped_arm.lost {
                failures.push(format!(
                    "{} {}: the person's edit is gone (Git exited {:?}, the wrapper {})",
                    form.subcommand,
                    name,
                    native_arm.code,
                    if wrapped_arm.refused { "refused" } else { "allowed it" }
                ));
            }
        }
    }
    println!(
        "rows: {rows}; rows where Git takes the marked edit and the guard refuses: {refused_rows}; rows where the option changes nothing about what is written: {untouched}; rows that lost the edit: {}",
        failures.len()
    );
    assert!(failures.is_empty(), "{} rows:\n{}", rows, failures.join("\n"));
}
