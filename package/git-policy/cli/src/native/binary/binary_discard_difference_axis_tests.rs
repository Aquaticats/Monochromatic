//! What:
//!  Every kind of difference Git reports for a path, hidden by each mark and not hidden at all,
//!  against every guarded form that writes the path:
//!  bytes,
//!  the executable bit,
//!  and the type of the entry (a regular file the person turned into a link),
//!  each with no mark, an `assume-unchanged` mark and a `skip-worktree` mark.
//! Why:
//!  The guard refuses a form that would replace an edit the status hides,
//!  and it read that edit by comparing the content object alone:
//!  a file with the branch's bytes and the person's executable bit compared equal, so the guard let
//!  `git reset --hard` and `git restore` take the bit
//!  (the tenth review's third finding).
//!  A change of mode is a person's data like a change of bytes, and Git reports three kinds of change
//!  for a path, so the three are crossed with the two marks and with no mark, and each row is real Git
//!  beside the wrapper on the same fixture.
//!  A row where Git itself refuses the form, or writes the entry without changing it, is counted
//!  rather than trusted: the loss has to exist for the row to say anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const kind of ['bytes','mode','type']) for (const mark of ['none','assume','skip'])
//!   for (const form of forms) expect(wrappedKeeps(kind, mark, form)).toBe(true);
//! ```

use super::support::{Fixture, Observed, direct, fixture, git, observe, remove, repository, stderr_of, wrapped};
use std::path::{Path, PathBuf};

/// The bytes the person wrote over the branch's own, so a row finds a loss by content.
const PRECIOUS_BYTES: &[u8] = b"PRECIOUS BYTES\n";

/// The difference one row makes to the tracked entry.
///  An `enum` is a closed set of named alternatives.
#[derive(Clone, Copy)]
enum Difference {
    /// The content bytes differ from the index's.
    Bytes,
    /// Only the executable bit differs.
    Mode,
    /// A regular file the index holds is a link in the worktree.
    Type,
}

/// How the index hides that difference.
#[derive(Clone, Copy)]
enum Mark {
    /// No mark at all: Git's own status names the difference.
    Visible,
    /// `git update-index --assume-unchanged`.
    AssumeUnchanged,
    /// `git update-index --skip-worktree`.
    SkipWorktree,
}

impl Mark {
    /// The words that set the mark, or none for the visible case.
    fn words(&self) -> Option<&'static [&'static str]> {
        return match self {
            Mark::Visible => None,
            Mark::AssumeUnchanged => Some(&["update-index", "--assume-unchanged", "--"]),
            Mark::SkipWorktree => Some(&["update-index", "--skip-worktree", "--"]),
        };
    }

    /// The name a row is reported by.
    fn name(&self) -> &'static str {
        return match self {
            Mark::Visible => "no mark",
            Mark::AssumeUnchanged => "assume-unchanged",
            Mark::SkipWorktree => "skip-worktree",
        };
    }
}

impl Difference {
    /// The tracked path one row holds.
    fn path(&self) -> &'static str {
        return match self {
            Difference::Bytes => "a.txt",
            Difference::Mode => "script.sh",
            Difference::Type => "a.txt",
        };
    }

    /// The name a row is reported by.
    fn name(&self) -> &'static str {
        return match self {
            Difference::Bytes => "the bytes",
            Difference::Mode => "the executable bit",
            Difference::Type => "the type",
        };
    }
}

/// Whether the wrapper itself rejected the command.
fn refused(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// Whether the person's entry is still the one they made.
fn holds_the_persons_entry(directory: &Path, difference: Difference) -> bool {
    let path: PathBuf = directory.join(difference.path());
    return match difference {
        Difference::Bytes => std::fs::read(path.as_path()).is_ok_and(|bytes| return bytes == PRECIOUS_BYTES),
        Difference::Mode => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::metadata(path.as_path()).is_ok_and(|metadata| return metadata.permissions().mode() & 0o111 != 0)
            }
            #[cfg(not(unix))]
            {
                false
            }
        }
        Difference::Type => std::fs::symlink_metadata(path.as_path()).is_ok_and(|metadata| return metadata.file_type().is_symlink()),
    };
}

/// What:
///  One linked worktree whose tracked entry carries the row's difference, under the row's mark.
///  `&Fixture` borrows the disposable fixture, `&str` names it.
/// Why:
///  A linked worktree is where the guard asks these questions, and the entry has to be the person's
///  before the form runs, so each arm gets a fixture of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linked = prepared(fixture, 'repo', Difference.Bytes, Mark.AssumeUnchanged);
/// ```
fn prepared(fixture: &Fixture, name: &str, difference: Difference, mark: Mark) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new(name));
    let linked: PathBuf = fixture.root.join(format!("{name}-linked"));
    std::fs::write(repo.join("a.txt"), b"committed a\n").expect("a tracked file");
    std::fs::write(repo.join("script.sh"), b"#!/bin/sh\necho committed\n").expect("a tracked script");
    std::fs::write(repo.join("target.txt"), b"committed target\n").expect("a tracked target");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "script.sh", "target.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked_text: &str = linked.to_str().expect("a fixture path is text");
    git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked_text]);
    let path: &str = difference.path();
    match difference {
        Difference::Bytes => std::fs::write(linked.join(path), PRECIOUS_BYTES).expect("the person's bytes"),
        Difference::Mode => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(linked.join(path), std::fs::Permissions::from_mode(0o755)).expect("the person's bit");
            }
        }
        Difference::Type => {
            std::fs::remove_file(linked.join(path)).expect("the regular file goes");
            std::os::unix::fs::symlink("target.txt", linked.join(path)).expect("the person's link");
        }
    }
    if let Some(words) = mark.words() {
        let mut arguments: Vec<&str> = words.to_vec();
        arguments.push(path);
        git(fixture, linked.as_path(), arguments.as_slice());
    }
    return linked;
}

/// The forms that write one tracked path, as the words after the subcommand,
///  with the path appended where the form takes one.
/// Why:
///  These are the guarded forms whose written paths reach the entry:
///  the index's own write, a restore from `HEAD`, a hard reset, a forced checkout of `HEAD`'s path and
///  a `read-tree --reset -u`.
///  A whole-tree form names no path and writes the path from the tree all the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forms('a.txt') === [['checkout-index','-f','--','a.txt'], /* ... */];
/// ```
fn forms(path: &'static str) -> Vec<Vec<&'static str>> {
    return vec![
        vec!["checkout-index", "-f", "--", path],
        vec!["restore", "--source=HEAD", "--worktree", "--", path],
        vec!["reset", "--hard", "HEAD"],
        vec!["checkout", "-f", "HEAD", "--", path],
        vec!["read-tree", "--reset", "-u", "HEAD"],
    ];
}

/// What:
///  One row run in one arm: what the command exited with, whether the wrapper refused,
///  and whether the person's entry is still theirs.
/// Why:
///  The entry is the observable, and the exit status separates a form Git refused from one it ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arm = (wrappedArm) => ({ code, refused, kept: holdsThePersonsEntry(...) });
/// ```
fn run(label: &str, difference: Difference, mark: Mark, words: &[&str], wrapped_arm: bool) -> (Option<i32>, bool, bool) {
    let fixture: Fixture = fixture(label);
    let directory: PathBuf = prepared(&fixture, "repo", difference, mark);
    let observed: Observed = if wrapped_arm {
        let mut command = wrapped(&fixture);
        command.current_dir(directory.as_path()).args(words);
        observe(&mut command, b"")
    } else {
        let mut command = direct(&fixture);
        command.current_dir(directory.as_path()).args(words);
        observe(&mut command, b"")
    };
    let kept: bool = holds_the_persons_entry(directory.as_path(), difference);
    let answer: (Option<i32>, bool, bool) = (observed.code, refused(&observed), kept);
    remove(&fixture);
    return answer;
}

/// What:
///  Every kind of difference, every mark and every form:
///  the person's entry is never gone,
///  and every row where Git takes it is one the wrapper refused.
/// Why:
///  The rule is that a change of mode is the person's data like a change of bytes,
///  so the three kinds of difference Git reports are asked the same question,
///  and a row where Git itself writes nothing is counted rather than trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const kind of kinds) for (const mark of marks) for (const form of forms) expect(lost).toEqual([]);
/// ```
#[test]
fn every_kind_of_difference_git_reports_is_kept_or_refused() {
    let kinds: [Difference; 3] = [Difference::Bytes, Difference::Mode, Difference::Type];
    let marks: [Mark; 3] = [Mark::Visible, Mark::AssumeUnchanged, Mark::SkipWorktree];
    let mut rows: usize = 0;
    let mut refused_rows: usize = 0;
    let mut nothing_to_lose: usize = 0;
    let mut failures: Vec<String> = Vec::<String>::new();
    for difference in kinds {
        for mark in marks {
            for words in forms(difference.path()) {
                let name: String = format!("{} / {} / {}", difference.name(), mark.name(), words.join(" "));
                let wrapped_arm: (Option<i32>, bool, bool) = run("difference-axis-wrapped", difference, mark, words.as_slice(), true);
                let native_arm: (Option<i32>, bool, bool) = run("difference-axis-native", difference, mark, words.as_slice(), false);
                rows += 1;
                if native_arm.2 {
                    nothing_to_lose += 1;
                } else if wrapped_arm.1 {
                    refused_rows += 1;
                } else if !wrapped_arm.2 {
                    failures.push(format!(
                        "{name}: Git took the person's entry (exit {:?}) and the wrapper ran the form (exit {:?})",
                        native_arm.0, wrapped_arm.0
                    ));
                }
                if !wrapped_arm.2 {
                    failures.push(format!("{name}: the person's entry is gone"));
                }
            }
        }
    }
    println!(
        "rows: {rows}; rows where Git takes the entry and the wrapper refuses: {refused_rows}; rows where Git changed nothing: {nothing_to_lose}; rows that lost the entry: {}",
        failures.len()
    );
    assert!(failures.is_empty(), "{} rows:\n{}", rows, failures.join("\n"));
}
