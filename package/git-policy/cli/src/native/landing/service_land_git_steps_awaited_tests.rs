//! Controls for the awaited Git steps of a landing:
//! a quick step is answered inside a task where the synchronous face is refused,
//! and both faces keep one side reference and say where the commit is.

use super::*;
use crate::service_land_git_steps_and_import_stream::{keep_unlanded, run};
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The longest a control lets a step run before it calls it a hang.
const THE_PATIENCE: Duration = Duration::from_secs(60);

/// A runtime of two workers:
/// one runs nothing of the control's,
/// so a step that held the worker it was polled on is still seen holding it.
fn two_workers() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("a runtime of two workers");
}

/// A repository of a fixture and the context a landing reads it with.
fn a_repository(name: &str) -> (PathBuf, GitContext, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let context: GitContext = GitContext { common_dir: Some(repo.join(".git")), ..git_context() };
    return (root, context, head);
}

/// What:
///  A quick Git step is answered inside a task,
///  and the synchronous face of the same step is refused there.
/// Why:
///   A step that may be ended at its ceiling has its pipes registered with the runtime,
/// which the synchronous runner does by entering the runtime
/// (`step_output_read_whole_through_pipes.rs`,
/// `attach_pipes`):
/// a task may not enter it,
/// so a landing that is a task asks the awaited face.
/// The answer is what proves the child ran and was awaited,
/// and the refusal is what proves the two faces are not
/// interchangeable inside a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = await runAwaiting(git, repo, ['rev-parse', '--verify', '--quiet', 'HEAD']); expect(answered.stdout).toEqual(head); const refused = await runtime.spawn(() => run(...)); expect(refused).toEqual(Err('unavailable'))
/// ```
#[test]
fn a_quick_git_step_is_answered_inside_a_task() {
    let (root, context, head) = a_repository("git-steps-awaiting-quick");
    let runtime: tokio::runtime::Runtime = two_workers();
    let began: Instant = Instant::now();
    let answered: Result<GitOutput, FailureCode> =
        runtime.block_on(run_awaiting(&context, root.join("repo").as_path(), &["rev-parse", "--verify", "--quiet", "HEAD"], None));
    assert!(began.elapsed() < THE_PATIENCE, "the awaited step ended: {:?}", began.elapsed());
    let output: GitOutput = answered.expect("the awaited face asks Git for the head this repository has");
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()).trim(), head, "the awaited face answers the commit the branch names");
    let refusing: GitContext = context.clone();
    let refusing_repository: PathBuf = root.join("repo");
    let refused: Result<GitOutput, FailureCode> = runtime.block_on(async move {
        return tokio::spawn(async move { return run(&refusing, refusing_repository.as_path(), &["rev-parse", "--verify", "--quiet", "HEAD"], None); })
            .await
            .expect("the task that asks ends");
    });
    assert_eq!(refused.err(), Some(FailureCode::Unavailable), "the synchronous face cannot register its pipes from inside the runtime, which is why a task asks the awaited one");
    remove(root.as_path());
}

/// What:
///  Both faces keep the same side reference for the same batch,
///  and both tell the person where the commit is in the same words.
/// Why:
///   One place names the reference a withdrawn landing's commit is kept on,
/// so a difference between the faces is a commit the person is told about but cannot find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepUnlandedAwaiting(git, repo, 5, head)).toEqual(keepUnlanded(git2, repo2, 5, head2)); expect(both kept refs).toEqual(heads)
/// ```
#[test]
fn both_faces_keep_one_side_reference_and_say_one_line() {
    let (awaited_root, awaited_context, awaited_head) = a_repository("git-steps-awaiting-keeps");
    let (synchronous_root, synchronous_context, synchronous_head) = a_repository("git-steps-awaiting-keeps-synchronous");
    let runtime: tokio::runtime::Runtime = two_workers();
    let awaited: String = runtime.block_on(keep_unlanded_awaiting(&awaited_context, awaited_root.join("repo").as_path(), 5, awaited_head.as_str()));
    let synchronous: String = keep_unlanded(&synchronous_context, synchronous_root.join("repo").as_path(), 5, synchronous_head.as_str());
    let name: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(5);
    assert!(awaited.contains(name.as_str()), "the awaited face names the reference the commit is kept on: {awaited}");
    assert!(synchronous.contains(name.as_str()), "the synchronous face names the same reference: {synchronous}");
    assert_eq!(awaited, synchronous, "both faces say where the commit is in one sentence");
    let awaited_kept: String = git_text(awaited_root.join("repo").as_path(), &["rev-parse", "--verify", "--quiet", name.as_str()]);
    let synchronous_kept: String = git_text(synchronous_root.join("repo").as_path(), &["rev-parse", "--verify", "--quiet", name.as_str()]);
    assert_eq!(awaited_kept, awaited_head, "the awaited face keeps the commit it was given");
    assert_eq!(synchronous_kept, synchronous_head, "the synchronous face keeps the commit of its own repository");
    remove(awaited_root.as_path());
    remove(synchronous_root.as_path());
}
