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

use super::{NAMING_RECORD_FILES, NAMING_RECORDS, detached_head, git_work_a_removal_would_strand};
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
///  A `HEAD` a killed writer left torn is a record this program cannot read,
///  and the question that keeps a worktree for such a record answers so of the question itself and of the whole
///  removal.
/// Why:
///  Only two shapes are states Git writes:
///  a line naming a reference, whose commit survives the removal,
///  and a full object name, which is a commit only the worktree's own directory names.
///  Every other shape may name a commit this program cannot see,
///  and answering "not detached" there would report that commit unstranded and delete the only thing that names it
///  (a review of this change, finding F2).
///  Both halves are asked:
///  the reader itself, and the whole question on a worktree whose own Git directory holds no per-worktree reference,
///  because a reference there would make the question ask Git instead, which fails on a torn `HEAD` and so hides the
///  divergence.
#[test]
fn a_head_that_names_neither_a_branch_nor_a_commit_is_the_reason_nothing_is_removed() {
    for torn in ["0f1af2e0f295", "the record a killed writer left behind", "gggggggggggggggggggggggggggggggggggggggg"] {
        let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-torn-head");
        std::fs::write(fixture.admin.join("HEAD").as_path(), format!("{torn}\n")).expect("a torn HEAD record");
        let read: Result<Option<String>, String> = detached_head(fixture.admin.as_path());
        match read {
            Err(reason) => assert!(reason.contains("names neither a branch nor a commit"), "{torn:?} answered: {reason}"),
            Ok(named) => panic!("a HEAD that names neither a branch nor a commit must be a record this program cannot read, but it answered {named:?} for {torn:?}"),
        }
        // The whole question, with no per-worktree reference to make it ask Git instead.
        let _removed = std::fs::remove_dir_all(fixture.admin.join("refs").as_path());
        let said: Result<Vec<String>, String> = fixture.stranded();
        match said {
            Err(reason) => assert!(reason.contains("names neither a branch nor a commit"), "{torn:?} answered: {reason}"),
            Ok(empty) => panic!("a removal of a worktree whose HEAD cannot be read must keep it, but it answered {empty:?} for {torn:?}"),
        }
    }
}

/// What:
///  A commit made on a detached `HEAD` and then reset away is stranded,
///  named by the worktree's `HEAD` reflog.
/// Why:
///  The reflog is the only thing left that names it once `HEAD` names a commit that survives,
///  and the removal deletes that reflog with the worktree's own Git directory,
///  so `git gc --prune=now` takes the commit afterwards
///  (a review of `e615fb599b`, finding 1, reproduced through the built wrapper as well).
#[test]
fn a_commit_only_the_head_reflog_names_is_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-reflog");
    let commit: String = fixture.detach_and_commit();
    git(fixture.worktree.as_path(), &["reset", "--hard", "main"]);
    let said: Vec<String> = fixture.stranded().expect("the logs and references can be read");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("its HEAD reflog"), "{}", said[0]);
    assert!(said[0].contains(&commit[..12]), "{}", said[0]);
    assert!(said[0].contains("git gc --prune=now"), "{}", said[0]);
}

/// What:
///  A commit an amend took off the worktree's branch is not stranded:
///  the branch's own reflog, in the common Git directory, still names it.
/// Why:
///  That reflog outlives the worktree,
///  so nothing dies that names the commit,
///  and a question that asked only the references would keep a worktree nothing stands in the way of
///  (a review of `e615fb599b`, finding 1, states this as the ordinary case of work on a branch).
#[test]
fn a_commit_a_surviving_reflog_names_is_not_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-reflog-amend");
    fixture.commit("work.txt");
    let amended_away: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(fixture.worktree.join("more.txt"), b"more\n").expect("a file in the worktree");
    git(fixture.worktree.as_path(), &["add", "--", "more.txt"]);
    git(fixture.worktree.as_path(), &["commit", "--quiet", "--amend", "--no-edit"]);
    assert_ne!(git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]), amended_away, "the amend made another commit");
    assert_eq!(fixture.stranded(), Ok(Vec::new()), "the branch reflog names what the amend took off it");
}

/// What:
///  Every record of an operation that stopped is read for the commit it holds,
///  and the sentence names that record.
/// Why:
///  A rebase, an `am`, a bisect, a cherry-pick and a merge each leave a record in the worktree's own Git
///  directory,
///  and the removal deletes them with it:
///  a commit named only there would go with them
///  (a review of `e615fb599b`, finding 1, for the records beside the reflog).
#[test]
fn a_record_of_an_operation_that_stopped_names_the_commit_it_holds() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-records");
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    let mut records: Vec<(String, String)> = Vec::new();
    for name in NAMING_RECORDS {
        records.push((name.to_owned(), name.to_owned()));
    }
    for (directory, name) in NAMING_RECORD_FILES {
        records.push((format!("{directory}/{name}"), format!("{directory}/{name}")));
    }
    for (where_it_stands, holder) in records {
        let commit: String = git_text(
            fixture.worktree.as_path(),
            &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a record names"],
        );
        let path: PathBuf = fixture.admin.join(where_it_stands.as_str());
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the record's directory");
        std::fs::write(path.as_path(), format!("{commit}\n")).expect("the record");
        let said: Vec<String> = fixture.stranded().expect("the records can be read");
        assert!(
            said.iter().any(|one| return one.contains(&commit[..12]) && one.contains(holder.as_str())),
            "the record `{where_it_stands}` names {commit}, but the question said {said:?}"
        );
        std::fs::remove_file(path.as_path()).expect("the record goes");
    }
}

/// What:
///  A commit only `FETCH_HEAD` names is not stranded,
///  and the module's own words say why.
/// Why:
///  `FETCH_HEAD` names an object Git fetched from somewhere else,
///  which is still there where it came from and comes back with the same fetch,
///  so it is no commit a person made and no work a removal can lose.
///  Measured on this repository, 2026-11-14: 4 of its 179 linked worktrees hold such a commit, and refusing them
///  would keep four worktrees standing over an object the far side still has.
#[test]
fn a_commit_only_fetch_head_names_is_not_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-fetch-head");
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = git_text(
        fixture.worktree.as_path(),
        &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a fetch names"],
    );
    std::fs::write(fixture.admin.join("FETCH_HEAD").as_path(), format!("{commit}\t\tbranch 'main' of somewhere\n")).expect("a fetch record");
    assert_eq!(fixture.stranded(), Ok(Vec::new()), "a fetched object is not work a removal can lose");
}

/// What:
///  A line of a log this program cannot read is the reason nothing is removed.
/// Why:
///  Such a line may name a commit this program cannot see,
///  and answering "no commit" there would delete the only thing that names it
///  (the same rule the `HEAD` record already follows).
#[test]
fn a_log_line_this_program_cannot_read_is_the_reason_nothing_is_removed() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-torn-log");
    std::fs::write(fixture.admin.join("logs/HEAD").as_path(), b"a line a killed writer left behind\n").expect("a torn log");
    let said: Result<Vec<String>, String> = fixture.stranded();
    assert!(said.is_err(), "a log this program cannot read is no log that proves a worktree clear");
    let reason: String = said.expect_err("a reason");
    assert!(reason.contains("its HEAD reflog"), "the reason names the log: {reason}");
    assert!(reason.contains("a line a killed writer left behind"), "and quotes the line: {reason}");
}

/// What:
///  A reference of the worktree that names an annotated tag names the commit the tag reaches,
///  a tag of a tag reaches the same commit,
///  and a tag of a tree is kept as it stands.
/// Why:
///  The query that lists this worktree's references asked for the object a reference names,
///  which for an annotated tag is the tag object:
///  the commit its tag was made for was dropped as no commit,
///  the removal went ahead, and `git gc --prune=now` took the commit
///  (a review of `5a2bd4bb6c`, finding 1).
///  A tag of a tree names no commit at all,
///  and nothing here proves such an object outlives the removal, so it is kept as well.
#[test]
fn a_reference_that_names_a_tag_names_what_the_tag_reaches() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-tag");
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = git_text(
        fixture.worktree.as_path(),
        &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a tag names"],
    );
    let tree: String = git_text(fixture.worktree.as_path(), &["rev-parse", &format!("{head}^{{tree}}")]);
    for (reference, kind, reached) in [("plain-tag", "commit", commit.clone()), ("nested-tag", "commit", commit.clone()), ("tag-of-tree", "tree", tree.clone())] {
        // The tag a person made, and for the nested case the tag of that tag.
        let inner: String = match reference {
            "tag-of-tree" => {
                git(fixture.worktree.as_path(), &["tag", "--annotate", "--message=of a tree", "construction-inner", tree.as_str()]);
                git_text(fixture.worktree.as_path(), &["rev-parse", "refs/tags/construction-inner"])
            }
            _ => {
                git(fixture.worktree.as_path(), &["tag", "--annotate", "--message=inner", "construction-inner", commit.as_str()]);
                git_text(fixture.worktree.as_path(), &["rev-parse", "refs/tags/construction-inner"])
            }
        };
        let pointed: String = match reference {
            "nested-tag" => {
                git(fixture.worktree.as_path(), &["tag", "--annotate", "--message=outer", "construction-outer", inner.as_str()]);
                git_text(fixture.worktree.as_path(), &["rev-parse", "refs/tags/construction-outer"])
            }
            _ => inner,
        };
        git(fixture.worktree.as_path(), &["update-ref", "--no-deref", &format!("refs/worktree/{reference}"), pointed.as_str()]);
        // The shared tag names go, as a person's do once the worktree's own reference is the last name.
        for name in ["construction-inner", "construction-outer"] {
            git(fixture.worktree.as_path(), &["update-ref", "-d", &format!("refs/tags/{name}")]);
        }
        let said: Vec<String> = fixture.stranded().expect("the references can be read");
        assert_eq!(said.len(), 1, "{reference} names one object: {said:?}");
        assert!(said[0].contains(&format!("the reference `refs/worktree/{reference}`")), "{reference}: {}", said[0]);
        assert!(said[0].contains(&reached[..12]), "{reference} reaches {reached}: {}", said[0]);
        assert!(said[0].contains(&format!("names the {kind}")), "{reference} is kept as a {kind}: {}", said[0]);
        git(fixture.worktree.as_path(), &["update-ref", "-d", &format!("refs/worktree/{reference}")]);
    }
}

/// What:
///  A record of an operation that is there and cannot be read is the reason nothing is removed,
///  in the worktree's own Git directory and in the directory of an operation that stopped.
/// Why:
///  Such a record may hold the only surviving name of a commit,
///  and reading it as no record deleted that name with the worktree
///  (a review of `5a2bd4bb6c`, finding 2).
#[test]
fn a_record_this_program_cannot_read_is_the_reason_nothing_is_removed() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-unreadable-record");
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = git_text(
        fixture.worktree.as_path(),
        &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a record names"],
    );
    for (where_it_stands, holder) in [("ORIG_HEAD", "the record `ORIG_HEAD`"), ("rebase-merge/onto", "the record `rebase-merge/onto`")] {
        let path: PathBuf = fixture.admin.join(where_it_stands);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the record's directory");
        std::fs::write(path.as_path(), format!("{commit}\n")).expect("the record");
        let said: Vec<String> = fixture.stranded().expect("a readable record can be read");
        assert!(
            said.iter().any(|one| return one.contains(holder) && one.contains(&commit[..12])),
            "a readable record names its commit: {said:?}"
        );
        // The same record, there and unreadable.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a record nothing may read");
        let refused: Result<Vec<String>, String> = fixture.stranded();
        let reason: String = refused.expect_err("a record this program cannot read is the reason nothing is removed");
        assert!(reason.contains(holder), "the reason names the record: {reason}");
        assert!(reason.contains("cannot be read"), "and says it could not be read: {reason}");
        std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o644)).expect("the record is readable again");
        std::fs::remove_file(path.as_path()).expect("the record goes");
    }
}


/// What:
///  A registration this program cannot read is the reason nothing is removed,
///  because it may name a worktree inside the one being removed.
/// Why:
///  A removal deletes the registrations inside it with their checkouts and their own Git directories,
///  and the list it asks was built by skipping every `gitdir` that could not be read:
///  the nested worktree's commit and its uncommitted bytes went with the outer one
///  (a review of `5a2bd4bb6c`, finding 3).
#[test]
fn an_unreadable_registration_is_the_reason_nothing_is_removed() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-unreadable-registration");
    let nested: PathBuf = fixture.worktree.join("inner");
    let named: &str = nested.to_str().expect("a UTF-8 fixture path");
    git(fixture.worktree.as_path(), &["worktree", "add", "--quiet", "--detach", named, "HEAD"]);
    let nested_admin: PathBuf = fixture.common.join("worktrees").join("inner");
    let registration: PathBuf = nested_admin.join("gitdir");
    // The control: a registration this program can read asks the nested worktree, which holds nothing of its own beyond
    // the commit it is checked out at, and that commit survives the removal.
    assert_eq!(fixture.stranded(), Ok(Vec::new()), "a readable registration of a nested worktree with nothing of its own");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(registration.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a registration nothing may read");
    let refused: Result<Vec<String>, String> = fixture.stranded();
    let reason: String = refused.expect_err("a registration this program cannot read is the reason nothing is removed");
    assert!(reason.contains("the registration in"), "the reason names the registration: {reason}");
    assert!(reason.contains("may name a worktree inside the one being removed"), "and why it stops the removal: {reason}");
    std::fs::set_permissions(registration.as_path(), std::fs::Permissions::from_mode(0o644)).expect("the registration is readable again");
}
