//! Controls for the awaited faces of the commit an amend makes:
//! both faces write one object,
//! and the object is written inside a task where the
//!  synchronous face is refused.

use super::{AmendedCommit, BuildSite, amended_fields, amended_fields_awaiting, rebuilt_onto, rebuilt_onto_awaiting, stored_commit_fields};
use crate::command_commit::parse_commit_region;
use crate::commit_transaction::CommitInvocation;
use crate::config_schema::CliGitConfig;
use crate::service_protocol::CommitRequest;
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use crate::transaction_git::GitContext;
use crate::wrapper_controls::no_controls;
use std::path::{Path, PathBuf};

/// The message the amended commit carries,
/// as a commit stores it.
const THE_MESSAGE: &[u8] = b"the subject an amend keeps\n\nand its body\n";

/// The raw commit one amend replaces:
/// an identity,
/// a header line a commit made again keeps,
/// and a message.
fn a_replaced_commit() -> Vec<u8> {
    let mut raw: Vec<u8> = Vec::new();
    raw.extend_from_slice(b"tree 1111111111111111111111111111111111111111\n");
    raw.extend_from_slice(b"parent 2222222222222222222222222222222222222222\n");
    raw.extend_from_slice(b"author A U Thor <a@example.invalid> 1700000000 +0100\n");
    raw.extend_from_slice(b"committer C O Mitter <c@example.invalid> 1700000100 +0100\n");
    raw.extend_from_slice(b"encoding ISO-8859-1\n\n");
    raw.extend_from_slice(THE_MESSAGE);
    return raw;
}

/// The scratch directory one commit made again keeps its message in.
fn the_scratch_of(root: &Path, name: &str) -> PathBuf {
    let scratch: PathBuf = root.join(name);
    std::fs::create_dir(scratch.as_path()).expect("a scratch directory");
    return scratch;
}

/// What:
///  The commit both faces make of one amended commit,
/// over one repository and one tree.
/// Why:
///  A commit's name is its content,
/// so two faces that write the same tree,
/// parent,
/// headers and message give one name,
///  and any field they disagree about gives another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = { sync, awaited };
/// ```
struct Made {
    /// The name the synchronous face answered.
    synchronous: String,
    /// The name the awaited face answered.
    awaited: String,
    /// The message file the synchronous face wrote.
    synchronous_message: Vec<u8>,
    /// The message file the awaited face wrote.
    awaited_message: Vec<u8>,
}

/// Both faces' answers about one amended commit.
fn made_by_both_faces(root: &Path, repo: &Path, fields: &AmendedCommit, tree: &str, parent: &str) -> Made {
    let context: GitContext = git_context();
    let sync_scratch: PathBuf = the_scratch_of(root, "scratch-sync");
    let awaited_scratch: PathBuf = the_scratch_of(root, "scratch-awaited");
    let sync_site: BuildSite<'_> = BuildSite { context: &context, cwd: repo, scratch: sync_scratch.as_path(), signing: None };
    let awaited_site: BuildSite<'_> = BuildSite { context: &context, cwd: repo, scratch: awaited_scratch.as_path(), signing: None };
    let synchronous: String = rebuilt_onto(fields, &sync_site, tree, parent).expect("the synchronous face writes the commit");
    let awaited: String = crate::lazy_runtime::run_on_runtime(rebuilt_onto_awaiting(fields, &awaited_site, tree, parent)).expect("the runtime").expect("the awaited face writes the commit");
    let synchronous_message: Vec<u8> = std::fs::read(sync_scratch.join("amended-message")).expect("the synchronous face wrote its message");
    let awaited_message: Vec<u8> = std::fs::read(awaited_scratch.join("amended-message")).expect("the awaited face wrote its message");
    return Made { synchronous, awaited, synchronous_message, awaited_message };
}

/// What:
///  Both faces write one commit object,
/// with one message,
/// from the amended commit's own fields.
/// Why:
///   The commit a moved branch makes is written whole by one face today and by the awaited one once the commit path is a
///  task,
/// and a field one of them loses (a header line above all) is a commit the person never asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(made.synchronous, made.awaited);
/// ```
#[test]
fn the_awaited_commit_answers_what_the_synchronous_one_answers() {
    let root: PathBuf = fixture("amend-awaiting-agrees");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let fields: AmendedCommit = stored_commit_fields(a_replaced_commit().as_slice()).expect("the fields of the commit one amend replaces");
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let parent: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let made: Made = made_by_both_faces(root.as_path(), repo.as_path(), &fields, tree.as_str(), parent.as_str());
    assert_eq!(made.synchronous, made.awaited, "the two faces write one commit object");
    assert_eq!(made.synchronous_message, THE_MESSAGE, "the synchronous face writes the amended commit's message");
    assert_eq!(made.awaited_message, THE_MESSAGE, "the awaited face writes the amended commit's message");
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-p", made.awaited.as_str()]), git_text(repo.as_path(), &["cat-file", "-p", made.synchronous.as_str()]), "the two names hold one object");
    assert!(git_text(repo.as_path(), &["cat-file", "-p", made.awaited.as_str()]).contains("encoding ISO-8859-1"), "the header line the amended commit carried is kept");
    remove(root.as_path());
}

/// What:
///  The commit is written inside a task,
/// and the synchronous face of the same step is refused there.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// so the commit
///  path that makes a commit again needs this face;
/// the name it answers is the synchronous face's,
/// which is what pins
///  that the child ran and was awaited rather than abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = await runtime.spawn(() => rebuiltOntoAwaiting(...)); const refused = await runtime.spawn(() => rebuiltOnto(...));
/// ```
#[test]
fn the_commit_is_written_inside_a_task_where_the_synchronous_one_is_refused() {
    let root: PathBuf = fixture("amend-awaiting-in-a-task");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let fields: AmendedCommit = stored_commit_fields(a_replaced_commit().as_slice()).expect("the fields of the commit one amend replaces");
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let parent: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let expected: String = {
        let scratch: PathBuf = the_scratch_of(root.as_path(), "scratch-sync");
        let site: BuildSite<'_> = BuildSite { context: &context, cwd: repo.as_path(), scratch: scratch.as_path(), signing: None };
        rebuilt_onto(&fields, &site, tree.as_str(), parent.as_str()).expect("the synchronous face writes the commit")
    };
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let awaited_scratch: PathBuf = the_scratch_of(root.as_path(), "scratch-awaited");
    let answered: Result<String, crate::commit_transaction::TransactionStop> = runtime.block_on(async {
        let site: BuildSite<'_> = BuildSite { context: &context, cwd: repo.as_path(), scratch: awaited_scratch.as_path(), signing: None };
        return rebuilt_onto_awaiting(&fields, &site, tree.as_str(), parent.as_str()).await;
    });
    assert_eq!(answered.expect("the awaited face writes the commit inside a task"), expected, "the awaited face writes the one commit object inside a task");
    let still_scratch: PathBuf = the_scratch_of(root.as_path(), "scratch-still");
    let refused: Result<String, crate::commit_transaction::TransactionStop> = runtime.block_on(async {
        let site: BuildSite<'_> = BuildSite { context: &context, cwd: repo.as_path(), scratch: still_scratch.as_path(), signing: None };
        return rebuilt_onto(&fields, &site, tree.as_str(), parent.as_str());
    });
    let said: String = format!("{:?}", refused.expect_err("the synchronous face cannot start its Git from inside the runtime"));
    assert!(said.contains("from inside itself"), "the refusal says the runtime was asked for from inside itself: {said}");
    remove(root.as_path());
}

/// What:
///  The face whose subject is a prepared commit is shaped as the synchronous one is:
/// each asks the
///  same question through the same builder,
/// and none of them starts Git through the synchronous runner.
/// Why:
///   Its picture of a preparation is the commit path's own fixture,
/// so what is pinned here is the shape a picture
///  cannot reach:
/// a face that started Git synchronously would be refused inside
///  the runtime it is for,
/// and a face that asked another question would make another commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(source.includes('the_amends_tree_awaiting') && source.includes('run_git_checked_awaiting'))
/// ```
#[test]
fn the_awaited_faces_of_a_prepared_commit_ask_what_the_synchronous_ones_ask() {
    let source: &str = include_str!("service_amend_replayed.rs");
    assert!(source.contains("amended_fields_awaiting"), "the face that reads back the commit Git built is there for the commit path to call");
    assert!(source.contains("run_git_checked_awaiting(invocation.context"), "and it awaits the read on the runtime");
}

/// What:
///  The request one amend stands for,
/// naming the commit it prepared and nothing else a read of that commit needs.
/// Why:
///  `amended_fields` reads one commit object and no other field of the request,
/// so the picture of a request names that commit and leaves the rest as an explicit path commit of the
///  worktree's own index leaves them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sent = { amend: true, commit: oid } as CommitRequest;
/// ```
fn the_request_of_one_prepared_commit(repo: &Path, oid: &str) -> CommitRequest {
    let git_dir: PathBuf = repo.join(".git");
    return CommitRequest {
        id: String::from("an-amend-read-back"),
        repository: repo.to_path_buf(),
        common_dir: git_dir.clone(),
        git_dir,
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: oid.to_string(),
        changes: Vec::new(),
        message: String::from("the subject an amend keeps"),
        author: None,
        committer: String::from("C O Mitter <c@example.invalid> 1700000100 +0100"),
        amend: true,
        tree: None,
        commit: Some(oid.to_string()),
        cleanup: None,
        index: repo.join(".git").join("index"),
        scratch: repo.join("scratch"),
        captured: 0,
        holder: String::new(),
    };
}

/// What:
///  The fields both faces read back from one commit Git built.
/// Why:
///   A commit made again keeps the authorship,
/// the committer,
/// the message and the header lines of the commit it replaces,
///  so a face that read another object or dropped a field would make a commit the person never asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(await amendedFieldsAwaiting(invocation, sent, undefined), amendedFields(invocation, sent, undefined));
/// ```
#[test]
fn the_awaited_fields_of_an_amend_answer_what_the_synchronous_ones_answer() {
    let root: PathBuf = fixture("amend-fields-awaiting-agrees");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    // The commit the amend replaces is not the branch's tip: a read of another commit would answer other fields.
    let oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("later.txt"), b"later\n").expect("write later.txt");
    crate::test_support::git(repo.as_path(), &["add", "--", "later.txt"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=a later commit"]);
    let raw: Vec<u8> = crate::test_support::git(repo.as_path(), &["cat-file", "commit", oid.as_str()]).stdout;
    let expected: AmendedCommit = stored_commit_fields(raw.as_slice()).expect("the fields of the commit Git built");
    let environment: Vec<(std::ffi::OsString, std::ffi::OsString)> = Vec::new();
    let arguments: Vec<std::ffi::OsString> = Vec::new();
    let region = parse_commit_region(&[std::ffi::OsString::from("--amend")], &[]).expect("a region Git's table reads");
    let config: CliGitConfig = CliGitConfig::defaults();
    let controls = no_controls();
    let invocation: CommitInvocation<'_> = CommitInvocation {
        context: &context,
        environment: environment.as_slice(),
        cwd: repo.as_path(),
        arguments: arguments.as_slice(),
        subcommand_index: 0,
        region: &region,
        config: &config,
        controls: &controls,
    };
    let sent: CommitRequest = the_request_of_one_prepared_commit(repo.as_path(), oid.as_str());
    let synchronous: AmendedCommit = amended_fields(&invocation, &sent, None).expect("the synchronous face reads the commit back");
    let awaited: AmendedCommit = crate::lazy_runtime::run_on_runtime(amended_fields_awaiting(&invocation, &sent, None)).expect("the runtime").expect("the awaited face reads the commit back");
    assert_eq!(synchronous, expected, "the synchronous face reads the commit's own fields");
    assert_eq!(synchronous, awaited, "the two faces read one commit back into one set of fields");
    remove(root.as_path());
}
