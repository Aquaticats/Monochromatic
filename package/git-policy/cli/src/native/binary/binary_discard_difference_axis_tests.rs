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

/// The bytes a tracked file holds.
const TARGET_TEXT: &[u8] = b"committed target\n";

/// The text of the link one row holds, which is the name it points at and not the bytes there:
///  a file of these bytes hashes to the object the link's own text makes.
const LINK_TEXT: &[u8] = b"target.txt";

/// The difference one row makes to the tracked entry.
///  An `enum` is a closed set of named alternatives.
#[derive(Clone, Copy)]
enum Difference {
    /// The content bytes differ from the index's.
    Bytes,
    /// Only the executable bit differs, added on a file the index holds as plain.
    Mode,
    /// Git's own executable bit removed from a file the index holds as executable:
    ///  `chmod 0655` on a `0755` file, where the group and other bits stay set.
    ModeOwnerRemoved,
    /// A regular file the index holds is a link in the worktree.
    Type,
    /// A link the index holds, replaced by a regular file whose bytes are the text of what it pointed at,
    ///  which hashes to the object the link's own text makes.
    TypeFromALinkToItsTargetsText,
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
            Difference::Bytes | Difference::Type => "a.txt",
            Difference::Mode | Difference::ModeOwnerRemoved => "script.sh",
            Difference::TypeFromALinkToItsTargetsText => "lnk",
        };
    }

    /// The name a row is reported by.
    fn name(&self) -> &'static str {
        return match self {
            Difference::Bytes => "the bytes",
            Difference::Mode => "the executable bit added",
            Difference::ModeOwnerRemoved => "the owner's executable bit removed",
            Difference::Type => "a file turned into a link",
            Difference::TypeFromALinkToItsTargetsText => "a link turned into a file of its target's text",
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
        Difference::TypeFromALinkToItsTargetsText => {
            // The person's file held the link's own text; a regular file of that text is still theirs.
            std::fs::read(path.as_path()).is_ok_and(|bytes| return bytes == LINK_TEXT)
        }
        Difference::Mode => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::metadata(path.as_path()).is_ok_and(|metadata| return metadata.permissions().mode() & 0o100 != 0)
            }
            #[cfg(not(unix))]
            {
                false
            }
        }
        Difference::ModeOwnerRemoved => {
            // Git's own bit is the owner's, so the row asks whether the owner's bit is still set.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::metadata(path.as_path()).is_ok_and(|metadata| return metadata.permissions().mode() & 0o100 == 0)
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
    std::fs::write(repo.join("target.txt"), TARGET_TEXT).expect("a tracked target");
    if matches!(difference, Difference::ModeOwnerRemoved) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(repo.join("script.sh"), std::fs::Permissions::from_mode(0o755)).expect("the index's own bit");
        }
    }
    if matches!(difference, Difference::TypeFromALinkToItsTargetsText) {
        std::os::unix::fs::symlink("target.txt", repo.join("lnk")).expect("the index's own link");
    }
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "script.sh", "target.txt"]);
    if matches!(difference, Difference::TypeFromALinkToItsTargetsText) {
        git(fixture, repo.as_path(), &["add", "--", "lnk"]);
    }
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
        Difference::ModeOwnerRemoved => {
            // The owner's bit goes and the group's and other's stay, so the mode reads `0655`.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(linked.join(path), std::fs::Permissions::from_mode(0o655)).expect("the person's mode");
            }
        }
        Difference::TypeFromALinkToItsTargetsText => {
            // The link becomes a regular file holding exactly the text it pointed at, which hashes to the
            //  object the link's own text makes.
            std::fs::remove_file(linked.join(path)).expect("the link goes");
            std::fs::write(linked.join(path), LINK_TEXT).expect("the person's file");
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
        // One name and no `--`: Git reads the name as a path rather than a revision, and with this option
        //  it writes the entry the mark hides (the eleventh review's fifth finding).
        vec!["checkout", "--ignore-skip-worktree-bits", path],
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
    let kinds: [Difference; 5] = [
        Difference::Bytes,
        Difference::Mode,
        Difference::ModeOwnerRemoved,
        Difference::Type,
        Difference::TypeFromALinkToItsTargetsText,
    ];
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
