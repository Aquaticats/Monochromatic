//! Controls for the route's gate through its entry point:
//!  a main checkout on this test machine's own filesystem keeps the ordinary add,
//!  a command that creates no worktree with a known plan is never the route's,
//!  and a removal's plan that still names the clone refuses the record's first write
//!  in the plan stop's own sentence.
//!
//! Why the plan's refusal is driven here:
//!  the plan read is the record writer's first statement,
//!  so nothing is ever asked of the device or the record home a fixture hands it,
//!  and what a person reads is then this control's own subject
//!  (issue 6450: the add printed a Rust `Debug` value here).

use super::fresh_the_record;
use super::run_a_zfs_add;
use crate::test_support::{fixture, git_context, remove, repository};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_add_route::add_preflight::Preflight;
use crate::worktree_clone_add_route::add_route_gate::RouteFacts;
use crate::worktree_clone_add_route::add_route_steps::StepRefusal;
use crate::worktree_clone_add_route::add_zfs::OpenedDevice;
use crate::worktree_clone_binding_record::plan_folder_read::{PLAN_HEADER, PlanStop, refuse_while_a_plan_names_the_clone};
use crate::worktree_clone_binding_record::registration_home::plans_folder;
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
    let facts: CopyFacts = CopyFacts { identity, creates: true, destination: Some(destination), add_plan: Some(plan), remove_plan: None, quiet: false };
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
///  and the words the person reads are that stop's own sentence,
///  never a Rust `Debug` rendering of it.
/// Why:
///   The refusal is the whole of what the person gets,
///  so a `Debug` value there names types instead of the removal that ends the plan
///  (issue 6450).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(freshTheRecord(standingPlan).message).toBe(stop.sentence());
/// ```
#[test]
fn a_standing_plan_refuses_the_record_in_the_plans_own_sentence() {
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
    let plan_text: String = format!("{PLAN_HEADER}\ncloneDataset={}\n", route.clone_dataset());
    std::fs::write(folder.join(THE_PLANS_KEY), plan_text.as_bytes()).expect("the standing plan");
    let plan: AddPlan = AddPlan { destination: route.destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    let stop: PlanStop = refuse_while_a_plan_names_the_clone(route.common_dir.as_path(), None, route.clone_dataset().as_str()).expect_err("the standing plan stops the read");
    let sentence: &str = stop.sentence();
    let source: SnapshotSource = common_source(&git_context(), &main_identity(route.source_root.as_path())).expect("a clean source");
    // The record's first statement reads the plans, so the device and the home are never asked anything:
    // any file this process can open stands in for both.
    let device_file: PathBuf = root.join("a-file-opened-as-the-device");
    std::fs::write(device_file.as_path(), b"no request reaches this file").expect("the device file");
    let preflight: Preflight = Preflight {
        source,
        device: OpenedDevice { device: ZfsDevice::open_at(device_file.as_path()).expect("the device file opens") },
        checkout_guid: DatasetGuid::parse("4821193048571903102").expect("the checkout's guid"),
        pool_guid: DatasetGuid::parse("4821193048571903103").expect("the pool's guid"),
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
    assert_eq!(message, sentence, "the add prints the plan stop's own sentence: {message}");
    assert!(!message.contains("NamesTheClone") && !message.contains("PlanNamesTheClone"), "no `Debug` rendering of the stop reaches the person: {message}");
    remove(root.as_path());
}
