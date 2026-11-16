//! What:
//!  Consumer-level controls for the commit a reference lock holds in a linked worktree,
//!  through the built executable:
//!  a removal or a prune keeps that commit where it cannot reach it,
//!  and a removal stops,
//! naming what is still held,
//! when no reference can hold it.
//! Why:
//!  The wrapper hands the removal to Git and cannot look afterwards,
//!  so only a command that starts the executable over a disposable repository
//!  shows that the person's commit survives the command that deletes its last reference
//!  (the owner's rule "never discard user data").
//!  A merge that lands a worktree's branch hands the removal of that worktree to the commit service and returns,
//!  so the worktree's own end is waited for,
//! bounded,
//! instead of being read at the moment the merge returns;
//!  the states a person can leave that service in are the sibling module's controls
//!  (`binary_worktree_removal_held_commit_hand_over_tests.rs`).
//!  The states a person really leaves are the ones these controls build:
//!  a checkout deleted before `git worktree prune`,
//!  and a lock file standing on the reference the commit should be kept on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = runWrapped(fixture, repo, ['worktree', 'prune']); // the commit is still there after `git gc --prune=now`
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::commit::wrapped_commit;
use super::support::{Fixture, Observed, fixture, git, observe, remove, run_direct, run_wrapped, stderr_of};
use git_policy_cli::held_commit_record::{Held, write};
use git_policy_cli::worktree_added_by_the_wrapper::record;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The branch the linked worktree has checked out.
pub(super) const BRANCH: &str = "feature";

/// The file the held commit changes.
const EDITED: &str = "edited.txt";

/// The batch number of the held commit.
pub(super) const SEQ: u64 = 7;

/// The file the landed branch adds,
/// which is the commit the merge brings in.
const LANDED: &str = "feature.txt";

/// The name of the linked worktree inside the repository's own registrations.
pub(super) const LINKED_REGISTRATION: &str = "worktrees/linked";

/// Real Git's answer to a question,
/// trimmed,
/// as text.
/// Why:
///  A control compares the reference name real Git reports,
/// and the binary harness has no reader of a direct command's standard output.
pub(super) fn git_said(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> String {
    let output: std::process::Output = git(fixture, directory, arguments);
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
}

/// What one fixture holds:
///  the repository the commands run in,
/// and the linked worktree whose commit is held.
pub(super) struct HeldScene {
    /// The fixture whose wrapper link every command starts.
    pub(super) fixture: Fixture,
    /// The main checkout every command runs in.
    pub(super) repo: PathBuf,
    /// The linked worktree the held commit belongs to.
    pub(super) linked: PathBuf,
    /// The common Git directory of the repository.
    pub(super) common: PathBuf,
    /// The commit no branch names.
    pub(super) held: String,
}

impl HeldScene {
    /// The name the commit is kept on.
    pub(super) fn kept_name(&self) -> String {
        return format!("refs/cli-git/kept/{SEQ}");
    }

    /// The name beside it,
    /// written when the first cannot be.
    fn rescue_name(&self) -> String {
        return format!("refs/cli-git/kept/{SEQ}-rescue");
    }

    /// Plant the lock file an update of `name` refused,
    /// as a killed or a concurrent Git leaves it.
    fn plant_lock(&self, name: &str) {
        let lock: PathBuf = self.common.join(format!("{name}.lock"));
        std::fs::create_dir_all(lock.parent().expect("the directory of the lock")).expect("the lock's directory");
        std::fs::write(lock.as_path(), b"").expect("the lock file");
    }

    /// What real Git's own maintenance says about the commit:
    /// its type,
    /// or nothing when no object holds it any more.
    pub(super) fn object_type_after_maintenance(&self) -> String {
        git(&self.fixture, self.repo.as_path(), &["gc", "--prune=now"]);
        let asked: Observed = run_direct(&self.fixture, self.repo.as_path(), &["cat-file", "-t", self.held.as_str()]);
        return String::from_utf8_lossy(asked.stdout.as_slice()).trim().to_string();
    }

    /// Every reference of the common Git directory that names the held commit.
    fn holders(&self) -> String {
        return git_said(&self.fixture, &self.repo, &["for-each-ref", "--contains", self.held.as_str(), "--format=%(refname)"]).trim().to_string();
    }
}

/// A repository,
/// a linked worktree of its own branch with the file the held commit changed,
/// and the record of the hold the wrapper's own hold writes:
/// the state a commit that met a reference lock leaves in that worktree.
fn a_held_commit(label: &str) -> HeldScene {
    let fixture: Fixture = fixture(label);
    let repo: PathBuf = fixture.root.join("repo");
    let linked: PathBuf = fixture.root.join("linked");
    std::fs::create_dir(repo.as_path()).expect("the repository directory");
    git(&fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(&fixture, repo.as_path(), &["add", "--", "base.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked_text: String = linked.display().to_string();
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", linked_text.as_str(), "-b", BRANCH]);
    std::fs::write(linked.join(EDITED), b"the edit\n").expect("the edit");
    git(&fixture, linked.as_path(), &["add", "--", EDITED]);
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--message=the held edit"]);
    let held: String = git_said(&fixture, &linked, &["rev-parse", "HEAD"]).trim().to_string();
    // The reference lock refused the branch write, so the branch still names the commit before it, and no reflog names the
    //  commit either: it was never on the branch, so the worktree's own reference is the only thing holding it.
    git(&fixture, linked.as_path(), &["reset", "--hard", "--quiet", "HEAD~1"]);
    git(&fixture, repo.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
    git(&fixture, linked.as_path(), &["update-ref", "--no-deref", format!("refs/worktree/cli-git-service/{SEQ}").as_str(), held.as_str()]);
    let common: PathBuf = repo.join(".git");
    let worktree_git_dir: PathBuf = common.join("worktrees/linked");
    let record: Held = Held {
        seq: SEQ,
        journal: fixture.root.join("journal"),
        repository: linked.clone(),
        common_dir: common.clone(),
        git_dir: worktree_git_dir.clone(),
        reference: git_policy_cli::ref_name::RefName::from_text(format!("refs/heads/{BRANCH}").as_str()),
        old: git_said(&fixture, &linked, &["rev-parse", "HEAD"]).trim().to_string(),
        new: held.clone(),
        commits: vec![held.clone()],
        index: worktree_git_dir.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![common.join(format!("refs/heads/{BRANCH}.lock"))],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write(worktree_git_dir.as_path(), &record).expect("the record of the hold");
    return HeldScene { fixture, repo, linked, common, held };
}

/// What:
///  The same held state in a repository where the merge lands the linked worktree's branch,
///  which is what hands that worktree to the automatic removal after the merge.
/// Why:
///   The held commit lives in the worktree's own Git directory and on its per-worktree reference,
///  and the removal of a landed worktree deletes both,
///  so only a merge that lands the branch and a maintenance run after it show whether the commit survives
///  (the owner's rule "never discard user data").
///  The branch holds a commit of its own,
///  because a branch that has never moved is not one a merge landed,
///  and main holds one too,
///  so the merge is a merge and not a fast-forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = aHeldCommitAMergeLands('binary-removal-held-landed-sweep');
/// const ran = await wrapped(['merge', '--no-edit', BRANCH]);
/// ```
pub(super) fn a_held_commit_a_merge_lands(label: &str) -> HeldScene {
    let fixture: Fixture = fixture(label);
    let repo: PathBuf = fixture.root.join("repo");
    let linked: PathBuf = fixture.root.join("linked");
    std::fs::create_dir(repo.as_path()).expect("the repository directory");
    git(&fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(&fixture, repo.as_path(), &["add", "--", "base.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked_text: String = linked.display().to_string();
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", linked_text.as_str(), "-b", BRANCH]);
    // The record a `git worktree add` of this program leaves, which is what lets the sweep remove it by itself.
    record(repo.join(".git").as_path(), linked.as_path()).expect("the record of a worktree this program made");
    std::fs::write(linked.join(LANDED), b"the landed work\n").expect("the landed edit");
    git(&fixture, linked.as_path(), &["add", "--", LANDED]);
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--message=the landed work"]);
    // The held commit: a child of the branch's own commit that no branch names, held only by the per-worktree
    //  reference and by the record beside it, exactly as a commit that met a reference lock leaves it.
    let tree: String = git_said(&fixture, &linked, &["rev-parse", "HEAD^{tree}"]);
    let held: String = git_said(&fixture, &linked, &["commit-tree", tree.as_str(), "-p", "HEAD", "-m", "the held edit"]);
    git(&fixture, linked.as_path(), &["update-ref", "--no-deref", format!("refs/worktree/cli-git-service/{SEQ}").as_str(), held.as_str()]);
    git(&fixture, repo.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
    let common: PathBuf = repo.join(".git");
    let worktree_git_dir: PathBuf = common.join(LINKED_REGISTRATION);
    let record_of_the_hold: Held = Held {
        seq: SEQ,
        journal: fixture.root.join("journal"),
        repository: linked.clone(),
        common_dir: common.clone(),
        git_dir: worktree_git_dir.clone(),
        reference: git_policy_cli::ref_name::RefName::from_text(format!("refs/heads/{BRANCH}").as_str()),
        old: git_said(&fixture, &linked, &["rev-parse", "HEAD"]).trim().to_string(),
        new: held.clone(),
        commits: vec![held.clone()],
        index: worktree_git_dir.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![common.join(format!("refs/heads/{BRANCH}.lock"))],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write(worktree_git_dir.as_path(), &record_of_the_hold).expect("the record of the hold");
    // Meanwhile main moves on its own, so the merge is a merge and the commit it lands is the linked worktree's.
    std::fs::write(repo.join("main.txt"), b"the main work\n").expect("the main work");
    git(&fixture, repo.as_path(), &["add", "--", "main.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=the main work"]);
    return HeldScene { fixture, repo, linked, common, held };
}

/// What:
///  `git worktree prune` after a person deleted the checkout keeps the held commit,
///  which the registration and the per-worktree reference were the only holders of.
/// Why:
///  The question about the kept reference and its write ran Git in the deleted checkout,
///  which Git cannot start in,
///  so the wrapper warned and handed the prune to Git:
///  the record,
/// the per-worktree reference and,
/// with real Git's own maintenance,
/// the commit went
///  (a review of `ee545033ad`,
///  finding 1).
#[test]
fn a_prune_keeps_the_held_commit_of_a_checkout_that_is_gone() {
    let scene: HeldScene = a_held_commit("binary-removal-held-prune-checkout-gone");
    std::fs::remove_dir_all(scene.linked.as_path()).expect("the checkout a person deleted");
    let ran: Observed = run_wrapped(&scene.fixture, scene.repo.as_path(), &["worktree", "prune"]);
    assert_eq!(ran.code, Some(0), "the prune ran: {}", stderr_of(&ran));
    let said: String = stderr_of(&ran);
    assert!(said.contains(scene.kept_name().as_str()), "the person is told where the commit is kept: {said}");
    assert!(said.contains(BRANCH), "the person is told which branch stays where it is: {said}");
    assert_eq!(git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(), scene.held, "the kept reference names the held commit");
    assert!(!scene.common.join("worktrees/linked").exists(), "the prune took the registration of the deleted checkout");
    assert_eq!(scene.object_type_after_maintenance(), "commit", "the commit is not lost with the registration");
    remove(&scene.fixture);
}

/// What:
///  A lock standing on the batch's own kept reference keeps the commit on the reference beside it,
///  and the removal the person asked for still finishes.
/// Why:
///  The wrapper reported the failed write and handed the removal to Git,
///  which deleted the worktree's reference and,
/// with real Git's own maintenance,
/// the commit
///  (a review of `ee545033ad`,
///  finding 2).
#[test]
fn a_forced_removal_keeps_the_held_commit_beside_a_locked_kept_reference() {
    let scene: HeldScene = a_held_commit("binary-removal-held-kept-locked");
    scene.plant_lock(scene.kept_name().as_str());
    let linked_text: String = scene.linked.display().to_string();
    let ran: Observed = run_wrapped(&scene.fixture, scene.repo.as_path(), &["worktree", "remove", "--force", linked_text.as_str()]);
    assert_eq!(ran.code, Some(0), "the removal the person asked for ran: {}", stderr_of(&ran));
    let said: String = stderr_of(&ran);
    assert!(said.contains(scene.rescue_name().as_str()), "the person is told the reference the commit is kept at: {said}");
    assert!(!scene.linked.exists(), "Git removed the worktree the person asked to remove");
    assert_eq!(
        git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.rescue_name().as_str()]).trim(),
        scene.held,
        "the reference beside the locked one names the held commit"
    );
    assert_eq!(scene.object_type_after_maintenance(), "commit", "the commit is not lost with the worktree");
    assert_eq!(scene.holders(), scene.rescue_name(), "the reference beside the locked one is the one holder left");
    remove(&scene.fixture);
}

/// What:
///  When neither of the two names can be written,
///  the removal does not run:
///  it exits 2,
/// names the commit and the references it tried,
///  says nothing was removed,
///  and gives the lock that stands in the way with the command that removes it.
/// Why:
///  The wrapper reported the failed write and handed the removal to Git anyway,
///  which deleted the only reference that named the commit
///  (a review of `ee545033ad`,
///  finding 2);
///  a command that must stop names what it kept and where.
#[test]
fn a_forced_removal_stops_when_no_reference_can_hold_the_held_commit() {
    let scene: HeldScene = a_held_commit("binary-removal-held-nothing-keeps");
    scene.plant_lock(scene.kept_name().as_str());
    scene.plant_lock(scene.rescue_name().as_str());
    let linked_text: String = scene.linked.display().to_string();
    let ran: Observed = run_wrapped(&scene.fixture, scene.repo.as_path(), &["worktree", "remove", "--force", linked_text.as_str()]);
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(2), "the command that must stop exits 2: {said}");
    assert!(said.contains(scene.held.as_str()), "the answer names the commit: {said}");
    assert!(said.contains(scene.kept_name().as_str()), "the answer names the reference tried first: {said}");
    assert!(said.contains(scene.rescue_name().as_str()), "the answer names the reference tried beside it: {said}");
    assert!(said.contains("nothing was removed"), "the answer says nothing changed: {said}");
    assert!(said.contains(format!("{}.lock", scene.kept_name()).as_str()), "the answer names the lock that stands there: {said}");
    assert!(said.contains("rm -- "), "the answer gives the command that removes a lock nothing holds: {said}");
    assert!(scene.linked.is_dir(), "Git never ran, so the worktree is where it was");
    assert!(scene.common.join("worktrees/linked").is_dir(), "the registration the prune would have taken is where it was");
    assert_eq!(
        git_said(&scene.fixture, &scene.linked, &["for-each-ref", "--contains", scene.held.as_str(), "--format=%(refname)"]).trim(),
        format!("refs/worktree/cli-git-service/{SEQ}"),
        "the worktree's own reference still names the commit the answer kept"
    );
    // The person removes the two locks nothing holds and runs the same command again, which is what the answer asks for.
    std::fs::remove_file(scene.common.join(format!("{}.lock", scene.kept_name()))).expect("the lock on the batch's kept name");
    std::fs::remove_file(scene.common.join(format!("{}.lock", scene.rescue_name()))).expect("the lock on the name beside it");
    let again: Observed = run_wrapped(&scene.fixture, scene.repo.as_path(), &["worktree", "remove", "--force", linked_text.as_str()]);
    assert_eq!(again.code, Some(0), "the same command keeps the commit once the locks are gone: {}", stderr_of(&again));
    let said_again: String = stderr_of(&again);
    assert!(said_again.contains(scene.kept_name().as_str()), "the second answer names the reference the commit is kept at: {said_again}");
    assert!(!scene.linked.exists(), "the removal the person asked for ran the second time");
    assert_eq!(git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(), scene.held, "the kept reference names the held commit");
    assert_eq!(scene.object_type_after_maintenance(), "commit", "the commit survives the maintenance the answer made room for");
    remove(&scene.fixture);
}

/// How long a control waits for the removal the merge handed to the commit service,
/// which the merge does not wait for.
/// Why a minute:
///  the removal of a worktree this size takes tens of milliseconds once a service is at work on it,
///  the fixture's own wait for the service's work is the same minute (`SERVICE_WORK_PATIENCE` in
///  `src/native/binary/binary_support_observation.rs`),
///  and nextest ends a test after five,
///  so a removal that never runs is named by this bound instead of holding the suite.
pub(super) const REMOVAL_PATIENCE: Duration = Duration::from_secs(60);

/// What:
///  Every state of the removal's queue and every kept-worktree notice of the repository,
///  as one text a failure carries.
/// Why:
///   The merge records the removal and returns,
/// so the only place the outcome exists afterwards is the record it named
///  and,
/// for a worktree the service kept,
/// the notice the next command shows:
///  a failure that says neither leaves a reader guessing between a removal that has not run yet and one that was
///  refused.
pub(super) fn what_became_of_the_removal(scene: &HeldScene) -> String {
    let work: PathBuf = scene.common.join("monochromatic-git-policy-cli/work");
    let mut said: String = format!("the queue of the removal is {}", super::support::work_listing(work.as_path()));
    let notices: PathBuf = scene.common.join("monochromatic-git-policy-cli/kept-worktree-notices");
    let Ok(entries) = std::fs::read_dir(notices.as_path()) else { return said };
    for entry in entries.flatten() {
        let text: String = std::fs::read_to_string(entry.path()).unwrap_or_else(|unreadable| { return format!("(unreadable: {unreadable})") });
        said.push_str(format!("\nthe notice {} says: {text}", entry.file_name().to_string_lossy()).as_str());
    }
    return said;
}

/// What:
///  Wait,
/// bounded,
/// until the worktree the merge handed over and its registration are gone.
/// Why:
///  Removing a worktree Git removes file by file reads every tracked file of it and is a step that may not be run twice,
///  so the merge records it for the commit service and returns before it runs
///  (`src/native/worktree/worktree_landed_removal.rs`):
///  the worktree's own end is the service's step,
///  and a control that read the directory at the moment the merge returned measured which of two processes finished
///  first
///  (the release gate on `0f07c4f0ee` failed on exactly that,
/// once in thirty-five gates).
pub(super) fn wait_until_the_removal_ran(scene: &HeldScene, said: &str) {
    let began: Instant = Instant::now();
    while scene.linked.exists() || scene.common.join(LINKED_REGISTRATION).exists() {
        assert!(
            began.elapsed() < REMOVAL_PATIENCE,
            "the removal the merge handed to the commit service did not run within {REMOVAL_PATIENCE:?}: {}; the merge said: {said}",
            what_became_of_the_removal(scene)
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// What:
///  A merge that lands the branch of a worktree holding a commit behind a reference lock keeps that commit
///  on a reference of this repository before the automatic removal,
///  and the removal the merge hands to the commit service runs,
///  the person being told where the commit is kept before the worktree goes.
/// Why:
///  The sweep the merge wakes removed the landed worktree by one of its own removal routes,
///  which skip what a person's `git worktree remove` does first:
///  the worktree's own record of the hold went,
///  its per-worktree reference went,
///  and so did the commit,
///  which real Git's own maintenance then pruned
///  (Kimi K3.1 Pro's audit of `97aeffe75b`,
///  "Landed sweep drops the only reference to an unmerged held commit").
///  The commit a reference lock holds is the one state where the wrapper's own words say it kept something,
///  so the removal that follows may not take the only reference that names it.
///  That removal is the commit service's whenever the mount forbids the fast destroy,
///  which is every container the gate runs in,
///  so the worktree's end is read by a bounded wait and never by a look at the directory the merge returned from:
///  the kept reference is read on both sides of that wait,
///  because it is the one thing no order may change.
#[test]
fn a_merge_that_lands_a_branch_keeps_the_held_commit_of_the_worktree_it_removes() {
    let scene: HeldScene = a_held_commit_a_merge_lands("binary-removal-held-landed-sweep");
    let branch_tip: String = git_said(&scene.fixture, &scene.repo, &["rev-parse", format!("refs/heads/{BRANCH}").as_str()]).trim().to_string();
    assert_eq!(
        git_said(&scene.fixture, &scene.repo, &["rev-parse", format!("{}^", scene.held).as_str()]).trim(),
        branch_tip,
        "the held commit is the child of the commit the branch kept, so the branch did not land it"
    );
    assert_eq!(
        git_said(&scene.fixture, &scene.linked, &["for-each-ref", "--contains", scene.held.as_str(), "--format=%(refname)"]).trim(),
        format!("refs/worktree/cli-git-service/{SEQ}"),
        "the worktree's own reference is the only thing that names the commit the removal would take"
    );
    let ran: Observed = observe(
        &mut wrapped_commit(&scene.fixture, scene.repo.as_path(), &["merge", "--no-edit", "--no-ff", "--message=land the feature", BRANCH]),
        b"",
    );
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(0), "the merge ran: {said}");
    assert!(said.contains(scene.kept_name().as_str()), "the person is told where the commit is kept: {said}");
    assert_eq!(
        git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(),
        scene.held,
        "the kept reference names the held commit before the removal has run, and after it: {said}"
    );
    assert_eq!(
        git_said(&scene.fixture, &scene.repo, &["rev-parse", format!("refs/heads/{BRANCH}").as_str()]).trim(),
        branch_tip,
        "the branch the person has to move by hand stays where the lock left it"
    );
    wait_until_the_removal_ran(&scene, said.as_str());
    assert!(!scene.common.join(LINKED_REGISTRATION).exists(), "the removal took the worktree's registration: {said}");
    assert_eq!(git_said(&scene.fixture, &scene.repo, &["rev-parse", scene.kept_name().as_str()]).trim(), scene.held, "the kept reference still names the held commit after the removal");
    assert_eq!(scene.object_type_after_maintenance(), "commit", "the commit is not lost with the worktree the merge removed");
    remove(&scene.fixture);
}
