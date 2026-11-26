//! Controls for the route's gate through its entry point:
//!  a main checkout on this test machine's own filesystem keeps the ordinary add,
//!  a command that creates no worktree with a known plan is never the route's,
//!  a removal's plan that still names the clone refuses the record's first write
//!  in the plan stop's own sentence,
//!  and a checkout whose source the fill's own restore would refuse makes the
//!  runner decline before the claim,
//!  so the add reaches the ladder with nothing of the route's own written,
//!  and it reaches the ladder with the person's own lock words too,
//!  which a rule of the route that is not serving it would otherwise refuse.
//!
//! Why the plan's refusal is driven here:
//!  the plan read is the record writer's first statement,
//!  so nothing is ever asked of the device or the record home a fixture hands it,
//!  and what a person reads is then this control's own subject
//!  (issue 6450: the add printed a Rust `Debug` value here).

use super::fresh_the_record;
use super::run_a_zfs_add;
use super::run_the_route;
use crate::test_support::{REAL_GIT, fixture, git, git_context, remove, repository};
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_add_route::add_preflight::Preflight;
use crate::worktree_clone_add_route::add_route_gate::RouteFacts;
use crate::worktree_clone_add_route::add_route_steps::StepRefusal;
use crate::worktree_clone_add_route::add_zfs::OpenedDevice;
use crate::worktree_clone_binding_record::plan_folder_read::PLAN_HEADER;
use crate::worktree_clone_binding_record::registration_home::{home_folder, plans_folder};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_copy_forward::CopyFacts;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_ladder_test_support::main_identity;
use crate::worktree_snapshot_source::{SnapshotSource, common_source};
use monochromatic_zfs_request_to_openzfs_kernel_device::ZfsDevice;
use std::ffi::OsString;
use std::path::PathBuf;

/// The plan's own file name:
///  the clone's canonical numeric GUID,
///  which is the plan's key and the only name this reader reads.
const THE_PLANS_KEY: &str = "4821193048571903102";

/// A main checkout fixture with an add plan at `.claude/worktrees/topic`.
fn a_fixture(name: &str) -> (PathBuf, CopyFacts) {
    let root: PathBuf = fixture(name);
    std::fs::create_dir_all(root.join(".git")).expect("the common directory");
    std::fs::create_dir_all(root.join(".claude").join("worktrees")).expect("the worktrees folder");
    let destination: PathBuf = root.join(".claude").join("worktrees").join("topic");
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree { common_dir: root.join(".git"), git_dir: root.join(".git"), worktree_root: root.clone() };
    let plan: AddPlan = AddPlan { destination: destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    let facts: CopyFacts = CopyFacts { identity, creates: true, destination: Some(destination), add_plan: Some(plan), remove_plan: None, removal_target: None, quiet: false };
    return (root, facts);
}

#[test]
fn this_machine_s_own_filesystem_keeps_the_ordinary_add() {
    let (root, facts) = a_fixture("route-entry-filesystem");
    let arguments: Vec<OsString> = ["worktree", "add", "--no-checkout", "/tmp/topic", "main"].iter().map(OsString::from).collect();
    assert_eq!(run_a_zfs_add(std::path::Path::new("/usr/bin/git"), arguments.as_slice(), &[], &facts), None, "a checkout that is not the design's dataset takes the ladder");
    remove(root.as_path());
}

#[test]
fn a_command_with_no_plan_is_never_the_route_s() {
    let (root, mut facts) = a_fixture("route-entry-no-plan");
    facts.add_plan = None;
    let arguments: Vec<OsString> = ["worktree", "add", "/tmp/topic"].iter().map(OsString::from).collect();
    assert_eq!(run_a_zfs_add(std::path::Path::new("/usr/bin/git"), arguments.as_slice(), &[], &facts), None);
    remove(root.as_path());
}

/// What:
///  A plan that still names the clone stops the record's first write,
///  and the words the person reads are the stop's own sentence,
///  with the one command that ends the plan and the cancel command beside it.
/// Why:
///  The command the person reads follows from the plan's head word and the clone
///  the pool can find
///  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
///  choice 2C),
///  and this is the add's second of two plan reads,
///  so it must print what the preflight's read prints for the same state.
///  The device is a file this control opens because a host control may not open
///  `/dev/zfs`:
///  every pool read through it refuses,
///  which is the absence no pool could prove,
///  and a `present` head beside it is the settling command's state.
///  The plan holds no attempt line,
///  so it witnesses nothing destroyed,
///  and the cancel command is named beside the removal.
///  No Rust `Debug` rendering of the stop may reach the person (issue 6450).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = freshTheRecord(standingPlan);
/// expect(refusal.message).toContain('settle-worktree-clone-record');
/// expect(refusal.message).toContain('cancel-worktree-removal');
/// ```
#[test]
fn a_standing_plan_stops_the_record_with_the_commands_that_end_and_cancel_it() {
    let root: PathBuf = fixture("route-a2-plan-sentence");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "checkout")).expect("the checkout's path");
    let route: RouteFacts = RouteFacts {
        source_root: repo.clone(),
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        destination: repo.join(".claude").join("worktrees").join("topic"),
        slot: String::from("topic"),
        checkout_dataset: String::from("tank/checkouts/checkout"),
        container_dataset: String::from("tank/clones"),
        pool: String::from("tank"),
    };
    let folder: PathBuf = plans_folder(route.common_dir.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder");
    let plan_text: String = format!("{PLAN_HEADER}\ncloneDataset={}\ncloneGuid={THE_PLANS_KEY}\nhead=present\nattempts=0\n", route.clone_dataset());
    let plan_path: PathBuf = folder.join(THE_PLANS_KEY);
    std::fs::write(plan_path.as_path(), plan_text.as_bytes()).expect("the standing plan");
    let plan: AddPlan = AddPlan { destination: route.destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    let source: SnapshotSource = common_source(&git_context(), &main_identity(route.source_root.as_path())).expect("a clean source");
    let device_file: PathBuf = root.join("a-file-opened-as-the-device");
    std::fs::write(device_file.as_path(), b"every request through this file refuses").expect("the device file");
    let preflight: Preflight = Preflight {
        source,
        device: OpenedDevice { device: ZfsDevice::open_at(device_file.as_path()).expect("the device file opens") },
        checkout_guid: DatasetGuid::parse("4821193048571903102").expect("the checkout's guid"),
        pool_guid: DatasetGuid::parse("4821193048571903103").expect("the pool's guid"),
        warnings: Vec::new(),
    };
    let home_path: PathBuf = root.join("the-adds-record-home");
    std::fs::write(home_path.as_path(), b"").expect("the home file");
    let home: std::fs::File = std::fs::File::open(home_path.as_path()).expect("the home opens");
    let arguments: Vec<OsString> = [String::from("worktree"), String::from("add"), route.destination.display().to_string(), String::from("main")].iter().map(OsString::from).collect();
    let refusal: StepRefusal = match fresh_the_record(&route, &plan, arguments.as_slice(), &preflight, &home, 1) {
        Ok(_) => panic!("the standing plan stops the record before its first write"),
        Err(refusal) => refusal,
    };
    let message: &str = refusal.message.as_str();
    assert!(message.contains(plan_path.display().to_string().as_str()), "the plan's own path is named: {message}");
    assert!(message.contains("settle-worktree-clone-record"), "a `present` head the pool cannot answer for names the settling command: {message}");
    assert!(message.contains("cancel-worktree-removal"), "a plan witnessing nothing destroyed names the cancel command beside it: {message}");
    for leak in ["APlanThatNamesTheClone", "PlanUnreadable", "ClonePresence", "{:?}"] {
        assert!(!message.contains(leak), "no `Debug` rendering of a stop reaches the person (`{leak}`): {message}");
    }
    remove(root.as_path());
}

/// What:
///  A main checkout holding an untracked Git repository of its own makes the
///  route's runner decline,
///  before it takes the claim,
///  so every write of the route's own stays unmade.
/// Why:
///  The preflight serves such a source up to the pool's read on this machine,
///  and the claim is the runner's first write after it,
///  so the runner is where the decline has to happen for the claim,
///  the locked registration,
///  the clone and the origin snapshot to stay unwritten
///  (`SPEC.md`, "What this release does not yet do").
///  On a machine with the pool,
///  a run that did not decline would take the claim and then meet the restore's
///  refusal with nothing to settle it.
///  No control on this machine can reach that far,
///  so what this one proves is the answer and the absence of every write before
///  the pool would have been read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runTheRoute(nestedRepositoryCheckout, addPlan)).toBeNull();
/// expect(existsSync(homeFolder(commonDir))).toBe(false);
/// ```
#[test]
fn an_untracked_repository_makes_the_runner_decline_before_the_claim() {
    let root: PathBuf = fixture("route-nested-repository-declines");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "checkout")).expect("the checkout's path");
    let route: RouteFacts = RouteFacts {
        source_root: repo.clone(),
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        destination: repo.join(".claude").join("worktrees").join("topic"),
        slot: String::from("topic"),
        checkout_dataset: String::from("tank/checkouts/checkout"),
        container_dataset: String::from("tank/clones"),
        pool: String::from("tank"),
    };
    let plan: AddPlan = AddPlan { destination: route.destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(nested.as_path()).expect("the repository's directory");
    git(nested.as_path(), &["init", "--quiet"]);
    std::fs::write(nested.join("inside.txt"), b"inside\n").expect("the repository's own file");
    let context: GitContext = git_context();
    let arguments: Vec<OsString> = [String::from("worktree"), String::from("add"), route.destination.display().to_string(), String::from("main")].iter().map(OsString::from).collect();
    let answer: Result<Option<Vec<String>>, StepRefusal> = run_the_route(&route, &plan, &context, std::path::Path::new(REAL_GIT), arguments.as_slice(), &[]);
    match answer {
        Ok(None) => {}
        Ok(Some(lines)) => panic!("the route served an add from a source its own restore cannot prove and printed {lines:?}"),
        Err(refusal) => panic!("the route fails the add instead of leaving it to the ladder: {}", refusal.message),
    }
    assert!(!home_folder(route.common_dir.as_path()).exists(), "the claim, and the record written after it, stay unwritten");
    assert!(!route.destination.exists(), "the slot stays empty when the route declines");
    remove(root.as_path());
}

/// What:
///  An add naming the person's own lock from a source the route declines reaches
///  the ladder with its words,
///  before the claim.
/// Why:
///  The runner is what the wrapper's add path calls,
///  so this is the answer a person meets:
///  the lock sentence names a rule of the ZFS clone route,
///  and the ladder hands the same words to Git,
///  whose `worktree add` takes `--lock`, `--reason` and `--checkout` by itself
///  (`SPEC.md`, the ladder's forms).
///  Refusing here failed an add another rung makes (issue 6453).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runTheRoute(nestedRepositoryCheckout, addPlan, ['worktree','add','--lock',slot,'main'])).toBeNull();
/// ```
#[test]
fn a_lock_word_from_a_declined_source_leaves_the_add_to_the_ladder() {
    let root: PathBuf = fixture("route-nested-repository-lock-word");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "checkout")).expect("the checkout's path");
    let route: RouteFacts = RouteFacts {
        source_root: repo.clone(),
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        destination: repo.join(".claude").join("worktrees").join("topic"),
        slot: String::from("topic"),
        checkout_dataset: String::from("tank/checkouts/checkout"),
        container_dataset: String::from("tank/clones"),
        pool: String::from("tank"),
    };
    let plan: AddPlan = AddPlan { destination: route.destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(nested.as_path()).expect("the repository's directory");
    git(nested.as_path(), &["init", "--quiet"]);
    std::fs::write(nested.join("inside.txt"), b"inside\n").expect("the repository's own file");
    let context: GitContext = git_context();
    let arguments: Vec<OsString> = [String::from("worktree"), String::from("add"), String::from("--lock"), route.destination.display().to_string(), String::from("main")].iter().map(OsString::from).collect();
    let answer: Result<Option<Vec<String>>, StepRefusal> = run_the_route(&route, &plan, &context, std::path::Path::new(REAL_GIT), arguments.as_slice(), &[]);
    match answer {
        Ok(None) => {}
        Ok(Some(lines)) => panic!("the route served an add from a source its own restore cannot prove and printed {lines:?}"),
        Err(refusal) => panic!("the route fails an add given `--lock` from a source it declines, so the ladder never makes the worktree Git's `--lock` asks for: {}", refusal.message),
    }
    assert!(!home_folder(route.common_dir.as_path()).exists(), "the claim, and the record written after it, stay unwritten");
    assert!(!route.destination.exists(), "the slot stays empty when the route declines");
    remove(root.as_path());
}
