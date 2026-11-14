//! What:
//!  Controls of the words for what a removal deletes besides the files Git tracks.
//! Why:
//!  A removal may delete an ignored directory and a whole Git repository,
//!  and the owner's decision lets that happen,
//!  so the line that reports the removal is the only thing that tells a person what went.
//!  A question that named nothing would pass every control but these.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! whatTheRemovalTakes(context, worktree, admin) // ['the ignored directories `ignored`', 'the Git repository at `ignored/clone`, whose branches are ...']
//! ```

use super::what_the_removal_takes;
use crate::test_support::{fixture, git, git_context, git_text, repository};
use std::path::PathBuf;

/// A repository and one linked worktree of it,
/// with the ignore rules a control writes in the first commit.
struct Fixture {
    /// The linked worktree a control fills.
    worktree: PathBuf,
    /// That worktree's own Git directory.
    admin: PathBuf,
}

impl Fixture {
    /// What a removal of the linked worktree would take.
    fn takes(&self) -> Vec<String> {
        return what_the_removal_takes(&git_context(), self.worktree.as_path(), self.admin.as_path());
    }

    /// Write a file, making its directories.
    fn write(&self, relative: &str, bytes: &[u8]) {
        let path: PathBuf = self.worktree.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(path.as_path(), bytes).expect("a file");
    }
}

/// What:
///  The repository every control here works on.
/// Why:
///  The line is read from the paths Git ignores,
///  so the fixture carries ignore rules in its first commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithAWorktree('removal-takes', 'ignored/\n*.log\n');
/// ```
fn a_repository_with_a_worktree(name: &str, rules: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    std::fs::write(main.join(".gitignore"), rules.as_bytes()).expect("the ignore rules");
    std::fs::write(main.join("base.txt"), b"base\n").expect("a tracked file");
    git(main.as_path(), &["add", "--", ".gitignore", "base.txt"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let added: PathBuf = root.join("topic");
    let named: &str = added.to_str().expect("a UTF-8 fixture path");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", named]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("the common Git directory");
    let worktree: PathBuf = std::fs::canonicalize(added.as_path()).expect("the worktree's real path");
    let admin: PathBuf = common.join("worktrees").join("topic");
    return Fixture { worktree, admin };
}

/// What:
///  A worktree holding nothing Git ignores says nothing went.
/// Why:
///  The line is printed for every removal,
///  and a sentence about nothing would read as loss where there is none.
#[test]
fn a_worktree_with_nothing_ignored_says_nothing_went() {
    let fixture: Fixture = a_repository_with_a_worktree("removal-takes-clean", "ignored/\n");
    assert_eq!(fixture.takes(), Vec::<String>::new(), "nothing below an ignored name, so nothing to name");
}

/// What:
///  An ignored directory the removal deletes is named.
/// Why:
///  A person who kept something below an ignored name reads which name went.
#[test]
fn an_ignored_directory_that_goes_is_named() {
    let fixture: Fixture = a_repository_with_a_worktree("removal-takes-directory", "ignored/\n");
    fixture.write("ignored/output.bin", b"build output\n");
    let said: Vec<String> = fixture.takes();
    assert_eq!(said.len(), 1, "{said:?}");
    assert_eq!(said[0], "the ignored directories `ignored`", "{said:?}");
}

/// What:
///  A Git repository inside an ignored directory is named,
///  with the tip of the branch it holds.
/// Why:
///  The repository's commits can exist nowhere else,
///  and the branch tip is the last thing a person can look for once the directory is gone
///  (a Sonnet audit of 0.6.1-alpha.14, finding F1).
#[test]
fn a_git_repository_inside_an_ignored_directory_is_named_with_its_branch_tips() {
    let fixture: Fixture = a_repository_with_a_worktree("removal-takes-repository", "ignored/\n");
    let clone: PathBuf = fixture.worktree.join("ignored/clone");
    std::fs::create_dir_all(clone.as_path()).expect("an ignored directory");
    git(clone.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(clone.join("readme"), b"only here\n").expect("a file in the repository");
    git(clone.as_path(), &["add", "--", "readme"]);
    git(clone.as_path(), &["commit", "--quiet", "--message=nested work"]);
    let tip: String = git_text(clone.as_path(), &["rev-parse", "HEAD"]);
    let said: Vec<String> = fixture.takes();
    let repository_line: &String = said.iter().find(|line| return line.contains("the Git repository at `ignored/clone`")).unwrap_or_else(|| panic!("a line about the repository: {said:?}"));
    assert!(repository_line.contains("whose branches are `main` at "), "{repository_line}");
    assert!(repository_line.contains(&tip[..12]), "{repository_line}");
}

/// What:
///  A repository the removal deletes that holds no branch says so,
///  rather than printing an empty list.
/// Why:
///  A person reading about a repository that held no branch learns that its commits are named by nothing now,
///  which is the one thing they have to act on before the directory goes.
#[test]
fn a_repository_that_holds_no_branch_says_so() {
    let fixture: Fixture = a_repository_with_a_worktree("removal-takes-no-branch", "ignored/\n");
    let clone: PathBuf = fixture.worktree.join("ignored/clone");
    std::fs::create_dir_all(clone.as_path()).expect("an ignored directory");
    git(clone.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let said: Vec<String> = fixture.takes();
    let repository_line: &String = said.iter().find(|line| return line.contains("the Git repository at `ignored/clone`")).unwrap_or_else(|| panic!("a line about the repository: {said:?}"));
    assert!(repository_line.contains("which held no branch"), "{repository_line}");
}
