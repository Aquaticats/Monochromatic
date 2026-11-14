//! What:
//!  Controls of the commits a removal would leave named by nothing.
//! Why:
//!  Every control here must keep EXACTLY the commits no surviving reference names,
//!  or a removal that keeps every worktree passes all of them but the last.
//!  The detached `HEAD` of the worktree itself and of a worktree inside it is the case the audit of 0.6.1-alpha.14 found
//!  destroyed (F2),
//!  and a person's `refs/worktree/` reference is the second way to be in that state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! gitWorkARemovalWouldStrand(context, common, worktree, admin) // ['its detached HEAD names the commit ...']
//! ```

use super::git_work_a_removal_would_strand;
use crate::test_support::{fixture, git, git_context, git_text, repository};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// A repository and one linked worktree of it,
/// with one tracked file in the first commit.
struct Fixture {
    /// The main checkout.
    main: PathBuf,
    /// The common Git directory.
    common: PathBuf,
    /// The linked worktree,
    /// which is the candidate of every control here.
    worktree: PathBuf,
    /// That worktree's own Git directory.
    admin: PathBuf,
}

impl Fixture {
    /// What a removal of the linked worktree would leave named by nothing.
    fn stranded(&self) -> Result<Vec<String>, String> {
        let context: GitContext = git_context();
        return git_work_a_removal_would_strand(&context, self.common.as_path(), self.worktree.as_path(), self.admin.as_path());
    }

    /// Commit one more file on the worktree's checked-out branch.
    fn commit(&self, name: &str) {
        let file: PathBuf = self.worktree.join(name);
        std::fs::write(file.as_path(), b"work\n").expect("a file in the worktree");
        git(self.worktree.as_path(), &["add", "--", name]);
        git(self.worktree.as_path(), &["commit", "--quiet", "--message=one more"]);
    }

    /// Move the worktree's `HEAD` off its branch and commit there.
    fn detach_and_commit(&self) -> String {
        git(self.worktree.as_path(), &["checkout", "--quiet", "--detach"]);
        self.commit("detached.txt");
        return git_text(self.worktree.as_path(), &["rev-parse", "HEAD"]);
    }
}

/// What:
///  The repository every control here works on.
/// Why:
///  The question reads the registrations of the repository,
///  the worktree's own Git directory and the references of both,
///  so one fixture carries all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRepositoryWithAWorktree('keeps-git-work');
/// ```
fn a_repository_with_a_worktree(name: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    git(main.as_path(), &["add", "--", "README.md"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let added: PathBuf = root.join("topic");
    let named: &str = added.to_str().expect("a UTF-8 fixture path");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", named]);
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("the common Git directory");
    let worktree: PathBuf = std::fs::canonicalize(added.as_path()).expect("the worktree's real path");
    let admin: PathBuf = common.join("worktrees").join("topic");
    return Fixture { main, common, worktree, admin };
}

/// What:
///  A worktree on its branch that commits nothing of its own names nothing,
///  which is what lets a removal go.
/// Why:
///  Every other control here keeps a worktree,
///  and one that always answered "keep" would pass all of them.
#[test]
fn a_worktree_on_its_branch_holds_no_commit_named_by_nothing() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-branch");
    fixture.commit("work.txt");
    assert_eq!(fixture.stranded(), Ok(Vec::new()), "a worktree on its branch holds nothing a removal would strand");
}

/// What:
///  A commit only the detached `HEAD` names is named,
///  with what names it and the reason it would go.
/// Why:
///  This is the case the audit found destroyed after the merge handed the removal over (F6).
#[test]
fn a_commit_only_the_detached_head_names_is_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-detached");
    let commit: String = fixture.detach_and_commit();
    let said: Vec<String> = fixture.stranded().expect("the references can be read");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("its detached HEAD"), "{}", said[0]);
    assert!(said[0].contains(&commit[..12]), "{}", said[0]);
    assert!(said[0].contains("git gc --prune=now"), "{}", said[0]);
}

/// What:
///  A commit a branch of the repository names is not stranded,
///  even with a detached `HEAD` standing on it.
/// Why:
///  The commit outlives the worktree through that branch,
///  and a question that named it would keep worktrees nothing stands in the way of.
#[test]
fn a_commit_a_branch_names_is_not_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-named");
    let commit: String = fixture.detach_and_commit();
    git(fixture.main.as_path(), &["branch", "kept", commit.as_str()]);
    assert_eq!(fixture.stranded(), Ok(Vec::new()), "a branch names the commit, so nothing goes with the worktree");
}

/// What:
///  A commit only a person's `refs/worktree/` reference names is named,
///  with that reference.
/// Why:
///  The reference lives in the worktree's own Git directory,
///  so the removal deletes the last name of a commit that exists nowhere else (a second audit's finding R4-A).
#[test]
fn a_commit_only_a_person_worktree_reference_names_is_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-person-ref");
    // A commit of the person's own that no branch names, exactly as `git commit-tree` makes one.
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = git_text(fixture.worktree.as_path(), &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a worktree reference names"]);
    git(fixture.worktree.as_path(), &["update-ref", "refs/worktree/person-keep", commit.as_str()]);
    let said: Vec<String> = fixture.stranded().expect("the references can be read");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("the reference `refs/worktree/person-keep`"), "{}", said[0]);
    assert!(said[0].contains(&commit[..12]), "{}", said[0]);
}

/// What:
///  A detached worktree inside the candidate is asked too,
///  and the reason names the commit its own `HEAD` holds.
/// Why:
///  A removal takes the nested worktree's files and its own Git directory with the outer one,
///  and a detached registration is no branch,
/// so nothing else names what it holds
///  (a Sonnet audit of 0.6.1-alpha.14,
/// finding F2).
#[test]
fn a_commit_only_a_detached_worktree_inside_holds_is_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-nested");
    let nested: PathBuf = fixture.worktree.join("inner");
    let named: &str = nested.to_str().expect("a UTF-8 fixture path");
    git(fixture.worktree.as_path(), &["worktree", "add", "--quiet", "--detach", named, "HEAD"]);
    std::fs::write(nested.join("inner.txt"), b"only here\n").expect("a file in the nested worktree");
    git(nested.as_path(), &["add", "--", "inner.txt"]);
    git(nested.as_path(), &["commit", "--quiet", "--message=detached work"]);
    let commit: String = git_text(nested.as_path(), &["rev-parse", "HEAD"]);
    let said: Vec<String> = fixture.stranded().expect("the references can be read");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains(&commit[..12]), "{}", said[0]);
    assert!(said[0].contains("its detached HEAD"), "{}", said[0]);
}

/// What:
///  A `HEAD` that names neither a branch nor a commit keeps the removal from running,
///  with the reason.
/// Why:
///  A record a killed writer left torn is exactly the state nothing here can read:
///  answering "not detached" for it would report a commit only that record names as unstranded,
///  and the removal would delete the only thing that names it
///  (a review of this change, finding F2).
/// Why:
///  A `HEAD` this program cannot read may name a commit of its own,
///  and guessing "no commit" would destroy the one thing that names it.
#[test]
fn a_head_that_cannot_be_read_is_the_reason_nothing_is_removed() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-unreadable");
    std::fs::remove_file(fixture.admin.join("HEAD").as_path()).expect("the HEAD record goes");
    let said: Result<Vec<String>, String> = fixture.stranded();
    assert!(said.is_err(), "a worktree whose HEAD cannot be read is no worktree a removal proves clear");
}

/// What:
///  A `HEAD` a killed writer left torn keeps the removal from running,
///  in each shape that is neither a reference nor a full object name.
/// Why:
///  Only two shapes are states Git writes:
///  a line naming a reference, whose commit survives the removal,
///  and a full object name, which is a commit only the worktree's own directory names.
///  Every other shape is a record this program cannot read,
///  and a commit it might hold would be reported unstranded and deleted if the question answered "not detached".
#[test]
fn a_head_that_names_neither_a_branch_nor_a_commit_is_the_reason_nothing_is_removed() {
    for torn in ["0f1af2e0f295", "the record a killed writer left behind", "gggggggggggggggggggggggggggggggggggggggg"] {
        let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-torn-head");
        std::fs::write(fixture.admin.join("HEAD").as_path(), format!("{torn}\n")).expect("a torn HEAD record");
        let said: Result<Vec<String>, String> = fixture.stranded();
        let reason: String = match said {
            Err(reason) => reason,
            Ok(empty) => panic!("a HEAD that names neither a branch nor a commit must keep the removal, but it answered {empty:?} for {torn:?}"),
        };
        assert!(reason.contains("names neither a branch nor a commit"), "{torn:?} answered: {reason}");
    }
}
