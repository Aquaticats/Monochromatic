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
use crate::worktree_removal_worktrees_it_takes::the_worktrees_it_takes;
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
        // The list of what this removal takes is made once, here as in the guard: a registration it cannot read is the
        // reason nothing is removed, before any question is asked.
        let taken = the_worktrees_it_takes(self.common.as_path(), self.worktree.as_path(), self.admin.as_path())?;
        return git_work_a_removal_would_strand(&context, self.common.as_path(), taken.as_slice());
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
///  A reference of the worktree that names an annotated tag names the tag object,
///  a tag of a tag names the outer tag object,
///  and a tag of a tree names the tag object that wraps the tree.
/// Why:
///  The question the guard asks is of objects, of every kind:
///  a reference's own object is the name a person sees, and the walk from it reaches what it tags.
///  A guard that kept only the peeled object lost the tag object itself, and the collection took it with its message
///  (a review of `177112aac1`, finding 5);
///  the commit a tag names is reached from the tag and needs no sentence of its own.
#[test]
fn a_reference_that_names_a_tag_names_the_tag_object() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-tag");
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = git_text(
        fixture.worktree.as_path(),
        &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a tag names"],
    );
    let tree: String = git_text(fixture.worktree.as_path(), &["rev-parse", &format!("{head}^{{tree}}")]);
    for (reference, of) in [("plain-tag", commit.clone()), ("nested-tag", commit.clone()), ("tag-of-tree", tree.clone())] {
        // The tag a person made, and for the nested case the tag of that tag.
        git(fixture.worktree.as_path(), &["tag", "--annotate", "--message=inner", "construction-inner", of.as_str()]);
        let inner: String = git_text(fixture.worktree.as_path(), &["rev-parse", "refs/tags/construction-inner"]);
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
        assert!(said[0].contains(&pointed[..12]), "{reference} names the tag {pointed}: {}", said[0]);
        assert!(said[0].contains("names the tag"), "{reference} is kept as a tag object: {}", said[0]);
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

/// What:
///  A detached `HEAD` written in capital letters names the same commit as the small one.
/// Why:
///  Git reads a name in either case as the same object and prints small letters,
///  and a text comparison dropped the commit the capital name held: the removal went ahead and a collection took it
///  (a review of `8858f843ee`, finding 3).
#[test]
fn a_detached_head_in_capitals_names_the_commit_git_prints() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-uppercase-head");
    let commit: String = fixture.detach_and_commit();
    assert!(commit.chars().any(|letter| return letter.is_ascii_alphabetic()), "the commit has letters to capitalise");
    let head: PathBuf = fixture.admin.join("HEAD");
    std::fs::write(head.as_path(), format!("{}\n", commit.to_uppercase())).expect("a capital HEAD");
    let said: Vec<String> = fixture.stranded().expect("the record can be read");
    assert_eq!(said.len(), 1, "the capital name holds one commit: {said:?}");
    assert!(said[0].contains(&commit[..12]), "and the sentence names it as Git prints it: {}", said[0]);
}

/// What:
///  A commit only the worktree's own `FETCH_HEAD` names is stranded,
///  and the sentence says a fetch brought it.
/// Why:
///  `FETCH_HEAD` was left out on the reasoning that what it names came from elsewhere,
///  and a commit made here and fetched by its object name is named by nothing else:
///  the removal ended 0 and a collection took it
///  (a review of `dfd6fab13e`, finding 3).
#[test]
fn a_commit_only_fetch_head_names_is_stranded() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-fetch-head");
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    let commit: String = git_text(
        fixture.worktree.as_path(),
        &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "a commit only a fetch names"],
    );
    std::fs::write(
        fixture.admin.join("FETCH_HEAD").as_path(),
        format!("{commit}\t\tbranch 'main' of somewhere\n"),
    )
    .expect("a fetch record");
    let said: Vec<String> = fixture.stranded().expect("the records can be read");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("the record `FETCH_HEAD`"), "the sentence names the record: {}", said[0]);
    assert!(said[0].contains(&commit[..12]), "and the commit: {}", said[0]);
    assert!(said[0].contains("what a fetch brought here"), "and says a fetch brought it: {}", said[0]);
}

/// What:
///  Every kind of name that dies with a worktree,
///  holding every kind of object it can hold.
/// Why:
///  The question the guard asks is of objects (a review of `177112aac1`, findings 5 and 6):
///  a name holds a commit, an annotated tag, a tree or a blob,
///  and a guard that reasoned about commits alone lost the tag object, its message with it, and the commit a reflog's
///  tag named.
///  So this walks each kind of dying name through each kind of object:
///  a detached `HEAD`, `refs/worktree/`, `refs/bisect/`, the reflog of a per-worktree reference, the `HEAD` reflog and a
///  record of a stopped operation,
///  and every one must keep the worktree naming the object it holds.
///  The last arm of each object names the same object on a reference that survives, and the question is then silent:
///  a walk that answered "everything is stranded" would pass the rows and fail that.
#[test]
fn every_dying_name_holding_every_kind_of_object_keeps_the_worktree() {
    let fixture: Fixture = a_repository_with_a_worktree("keeps-git-work-object-kinds");
    for (kind, object) in [
        ("commit", a_dangling_commit(&fixture)),
        ("tag", a_dangling_tag(&fixture)),
        ("tree", a_dangling_tree(&fixture)),
        ("blob", a_dangling_blob(&fixture)),
    ] {
        for place in ["detached-HEAD", "refs/worktree/only", "refs/bisect/only", "reflog-of-a-reference", "logs/HEAD", "ORIG_HEAD"] {
            // Git writes a commit into a detached `HEAD` and into a record, and refuses any other object there;
            // a reference and a reflog name whatever object a person put in them.
            if place == "detached-HEAD" && kind != "commit" {
                continue;
            }
            place_dying_name(&fixture, place, object.as_str());
            let said: Vec<String> = fixture.stranded().expect("the names can be read");
            assert_eq!(said.len(), 1, "{place} holding a {kind} is one stranded object: {said:?}");
            assert!(said[0].contains(&object[..12]), "{place} holding a {kind} names it: {}", said[0]);
            assert!(said[0].contains(&format!("names the {kind}")), "{place} says what kind it is: {}", said[0]);
            clear_dying_name(&fixture, place);
            let cleared: Vec<String> = fixture.stranded().expect("the names can be read");
            assert!(cleared.is_empty(), "{place} cleared says nothing is stranded: {cleared:?}");
            // The same object named by a reference that survives is nothing a removal would strand.
            git(fixture.worktree.as_path(), &["update-ref", "--no-deref", "refs/keep/the-object", object.as_str()]);
            place_dying_name(&fixture, place, object.as_str());
            let surviving: Vec<String> = fixture.stranded().expect("the names can be read");
            assert!(surviving.is_empty(), "{place} holding a {kind} a surviving reference names: {surviving:?}");
            clear_dying_name(&fixture, place);
            git(fixture.worktree.as_path(), &["update-ref", "--no-deref", "-d", "refs/keep/the-object"]);
        }
    }
}

/// A commit nothing else names: a child of the worktree's `HEAD` that no reference holds.
fn a_dangling_commit(fixture: &Fixture) -> String {
    let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
    return git_text(
        fixture.worktree.as_path(),
        &["commit-tree", format!("{head}^{{tree}}").as_str(), "-p", head.as_str(), "-m", "an object only a dying name holds"],
    );
}

/// An annotated tag nothing else names: made over a dangling commit, its own name deleted.
fn a_dangling_tag(fixture: &Fixture) -> String {
    let commit: String = a_dangling_commit(fixture);
    git(fixture.worktree.as_path(), &["tag", "--annotate", "--message=tag data a person wrote", "only-tag", commit.as_str()]);
    let tag: String = git_text(fixture.worktree.as_path(), &["rev-parse", "refs/tags/only-tag"]);
    git(fixture.worktree.as_path(), &["update-ref", "-d", "refs/tags/only-tag"]);
    return tag;
}

/// A tree nothing else names: one holding a blob nothing else names.
fn a_dangling_tree(fixture: &Fixture) -> String {
    let blob: String = a_dangling_blob(fixture);
    return git_with_input(fixture.worktree.as_path(), &["mktree"], format!("100644 blob {blob}\tonly.txt\n").as_bytes());
}

/// What:
///  One Git start that takes its input on standard input,
///  which the fixture's own helpers do not carry.
/// Why:
///  `git mktree` is the one way to make a tree holding a known blob,
///  and the fixture helpers pass no input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = gitWithInput(worktree, ['mktree'], '100644 blob <sha>\tonly.txt\n');
/// ```
fn git_with_input(directory: &std::path::Path, arguments: &[&str], input: &[u8]) -> String {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let mut command: Command = Command::new(crate::test_support::REAL_GIT);
    command.current_dir(directory).args(arguments).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    for (name, value) in crate::test_support::isolated_overlay() {
        command.env(name, value);
    }
    let mut child: std::process::Child = command.spawn().expect("Git starts");
    child.stdin.take().expect("a pipe").write_all(input).expect("the input");
    let output: std::process::Output = child.wait_with_output().expect("Git ends");
    assert!(output.status.success(), "git {arguments:?} failed: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

/// A blob nothing else names.
fn a_dangling_blob(fixture: &Fixture) -> String {
    std::fs::write(fixture.worktree.join("..only-content.txt"), b"bytes nothing else names\n").expect("content");
    return git_text(fixture.worktree.as_path(), &["hash-object", "-w", "--", "..only-content.txt"]);
}

/// What:
///  Put `object` where one kind of name that dies with the worktree holds it.
/// Why:
///  Git writes each of these itself:
///  a detached `HEAD` and the records name commits,
///  and a reference or a reflog names whatever a person put there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placeDyingName(fixture, place, object)
/// ```
fn place_dying_name(fixture: &Fixture, place: &str, object: &str) {
    match place {
        // Git's own `update-ref` writes whatever object the person names, here where a detached `HEAD` stands.
        "detached-HEAD" => {
            git(fixture.worktree.as_path(), &["update-ref", "--no-deref", "HEAD", object]);
        }
        "refs/worktree/only" => {
            git(fixture.worktree.as_path(), &["update-ref", "--no-deref", "refs/worktree/only", object]);
        }
        "refs/bisect/only" => {
            git(fixture.worktree.as_path(), &["update-ref", "--no-deref", "refs/bisect/only", object]);
        }
        // The reflog holds the object while the reference itself names the worktree's `HEAD`, which survives.
        "reflog-of-a-reference" => {
            git(fixture.worktree.as_path(), &["update-ref", "--no-deref", "--create-reflog", "refs/worktree/only", object]);
            let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
            git(fixture.worktree.as_path(), &["update-ref", "--no-deref", "refs/worktree/only", head.as_str()]);
        }
        // The `HEAD` reflog is a file of lines: the two names Git writes on a line, then who and when.
        "logs/HEAD" => {
            let line: String = format!("{object} {object} a person <person@example.invalid> 1700000000 +0000\tthe object a reflog holds\n");
            let log: PathBuf = fixture.admin.join("logs/HEAD");
            let mut text: String = std::fs::read_to_string(log.as_path()).unwrap_or_default();
            text.push_str(line.as_str());
            std::fs::write(log.as_path(), text.as_bytes()).expect("the reflog line");
        }
        "ORIG_HEAD" => {
            std::fs::write(fixture.admin.join("ORIG_HEAD"), format!("{object}\n")).expect("the record");
        }
        other => panic!("no kind of name called {other}"),
    }
}

/// What:
///  Take that name away again, leaving the worktree as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearDyingName(fixture, place)
/// ```
fn clear_dying_name(fixture: &Fixture, place: &str) {
    match place {
        "detached-HEAD" => {
            // Back on its branch, whose tip is a name that survives: the movement wrote reflog entries of its own, so the
            // log is left as one line naming that tip.
            git(fixture.worktree.as_path(), &["checkout", "--quiet", "feature"]);
            let head: String = git_text(fixture.worktree.as_path(), &["rev-parse", "HEAD"]);
            let line: String = format!("{head} {head} a person <person@example.invalid> 1700000000 +0000\ton a branch\n");
            std::fs::write(fixture.admin.join("logs/HEAD"), line.as_bytes()).expect("one reflog line");
        }
        "refs/worktree/only" | "refs/bisect/only" | "reflog-of-a-reference" => {
            git(fixture.worktree.as_path(), &["update-ref", "-d", "refs/worktree/only"]);
            git(fixture.worktree.as_path(), &["update-ref", "-d", "refs/bisect/only"]);
        }
        "logs/HEAD" => {
            let log: PathBuf = fixture.admin.join("logs/HEAD");
            let text: String = std::fs::read_to_string(log.as_path()).unwrap_or_default();
            let kept: String = text.lines().filter(|line| return !line.contains("the object a reflog holds")).map(|line| return format!("{line}\n")).collect();
            std::fs::write(log.as_path(), kept.as_bytes()).expect("the reflog without the line");
        }
        "ORIG_HEAD" => {
            let _ = std::fs::remove_file(fixture.admin.join("ORIG_HEAD"));
        }
        other => panic!("no kind of name called {other}"),
    }
}
