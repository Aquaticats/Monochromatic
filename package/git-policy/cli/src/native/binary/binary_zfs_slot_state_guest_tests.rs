//! What:
//!  Plan step 12's arms in the throwaway guest,
//!  on the design's layout:
//!  one real Git registration and one real clone,
//!  the mounted and the unmounted class of the same slot,
//!  a record that disagrees with the mount,
//!  a pending add and a pending removal,
//!  and the wrapper's `git worktree move` refusal beside Git's own answer,
//!  which is `Device or resource busy` while mounted and success while
//!  unmounted.
//! Why:
//!  The kernel refuses a move only while the clone is mounted,
//!  so the unmounted arm is the one that shows why the wrapper needs a refusal
//!  of its own
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  the measurement's item 6).
//!  The arm runs only where `CLI_GIT_ZFS_SLOT_STATE_GUEST` names a disposable
//!  guest and prints `SKIP` everywhere else,
//!  so a host run that ran nothing says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mounted = reportOf(readSlot(common, 'one', slot, { mounts, intents })); // mounted managed clone
//! const refusal = refuseMoveOfABoundSlot(common, slot); // bound, mounted or not
//! ```

use super::zfs_slot_state_guest_fixture::{Place, class_of, git, guid_of, intent_word, publish, readings_of};
use super::zfs_sweep_guest::{
    CHECKOUT_VARIABLE, NONCE, POOL_VARIABLE, SBIN_VARIABLE, clone_the_checkouts_snapshot, command_ok, make_the_designs_layout, mount_request, prepare_the_store, required, sweep,
    unmount_at, write_configuration, write_passing_probe_record, zfs, zfs_ignored,
};
use git_policy_cli::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use git_policy_cli::worktree_clone_binding_record::binding_words::Phase;
use git_policy_cli::worktree_clone_binding_record::registration_home::plans_folder;
use git_policy_cli::worktree_clone_move_refusal::managed_move_refusal::{ManagedMoveRefusal, ManagedMoveRefusalCause, refuse_move_of_a_bound_slot};
use git_policy_cli::worktree_clone_slot_state::slot_state_classification::classify_slot;
use git_policy_cli::worktree_clone_slot_state::slot_state_readings::SlotReadings;
use git_policy_cli::worktree_clone_slot_state::slot_state_report::report_of;
use std::path::{Path, PathBuf};
use std::process::Output;

/// The variable the guest driver sets for these arms.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SLOT_STATE_GUEST";

/// The slot this arm registers, clones and moves.
const SLOT: &str = "one";

/// The second slot a pending add's record is filed for.
const SLOT_TWO: &str = "two";

/// Whether this run is inside the guest.
fn in_the_guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP worktree_clone_slot_state::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One checkout path as the argument Git takes.
fn shown(path: &Path) -> &str {
    return path.to_str().expect("the guest's paths are UTF-8");
}

/// What:
///  Plan step 12's arms in the guest.
/// Why:
///  Each class the host controls prove on scripted readings is proved once on a
///  real pool,
///  and the refusal is shown beside Git's own answer in both arms.
#[test]
fn the_slots_class_and_the_move_refusal_on_a_real_pool() {
    if !in_the_guest() {
        return;
    }
    let pool: String = required(POOL_VARIABLE);
    let sbin: String = required(SBIN_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    let (checkout_dataset, container): (String, String) = make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    let worktrees: PathBuf = checkout.join(".claude").join("worktrees");
    std::fs::create_dir_all(worktrees.as_path()).expect("the slot parent is made");
    // A real repository and real registrations: the move this arm refuses is Git's own, not a fixture's.
    command_ok("git", &["init", "--quiet", shown(checkout.as_path())]);
    command_ok(
        "git",
        &[
            "-c",
            "user.name=the arm",
            "-c",
            "user.email=arm@example.invalid",
            "-C",
            shown(checkout.as_path()),
            "commit",
            "--quiet",
            "--allow-empty",
            "--message",
            "the arm's base",
        ],
    );
    let slot: PathBuf = worktrees.join(SLOT);
    let second: PathBuf = worktrees.join(SLOT_TWO);
    for place in [&slot, &second] {
        command_ok("git", &["-C", shown(checkout.as_path()), "worktree", "add", "--quiet", "--no-checkout", shown(place.as_path()), "HEAD"]);
    }
    let slot_identity: FolderIdentity = FolderIdentity::of_folder(slot.as_path()).expect("the underlay's identity reads before the mount");
    write_passing_probe_record();
    prepare_the_store(checkout.as_path());
    write_configuration(checkout.as_path(), None, None);
    let clone: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), SLOT, NONCE, Some(mount_request(NONCE).as_str()));
    let attached = sweep();
    assert_eq!(attached.status, Some(0), "the sweep attached the clone: {}", attached.said);
    let common: PathBuf = checkout.join(".git");
    let place: Place = Place {
        common: common.clone(),
        pool: pool.clone(),
        checkout_dataset: checkout_dataset.clone(),
        clone_dataset: clone.clone(),
        slot: slot.clone(),
        slot_identity,
        registration: common.join("worktrees").join(SLOT),
    };
    let two: Place = Place {
        common: common.clone(),
        pool: pool.clone(),
        checkout_dataset: checkout_dataset.clone(),
        clone_dataset: format!("{container}/{SLOT_TWO}"),
        slot: second.clone(),
        slot_identity: FolderIdentity::of_folder(second.as_path()).expect("the second slot's identity reads"),
        registration: common.join("worktrees").join(SLOT_TWO),
    };

    // The mounted class, the report, and a record that disagrees with the mount.
    publish(&place, Phase::Filled, sbin.as_str(), SLOT);
    let mounted: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    let mounted_report = report_of(&mounted);
    assert_eq!(class_of(&classify_slot(&mounted)), "mounted managed clone", "{:?}", mounted_report.lines());
    assert!(mounted_report.next_safe_action.contains("`git worktree move` is refused"), "{}", mounted_report.next_safe_action);
    println!("SLOT-STATE mounted class={} dataset={}", mounted_report.state, mounted_report.dataset);
    publish(&place, Phase::Cloned, sbin.as_str(), SLOT);
    let behind: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(class_of(&classify_slot(&behind)), "cannot be proved");
    println!("SLOT-STATE record-behind-mount class={}", class_of(&classify_slot(&behind)));
    publish(&place, Phase::Filled, sbin.as_str(), SLOT);

    // Mounted: the wrapper refuses and Git's own move meets the kernel's refusal.
    let bound: ManagedMoveRefusal = refuse_move_of_a_bound_slot(common.as_path(), slot.as_path()).expect("a bound slot refuses");
    assert!(matches!(bound.cause, ManagedMoveRefusalCause::Bound { .. }), "{:?}", bound.cause);
    println!("MOVE-REFUSAL mounted cause=bound");
    let moved: PathBuf = worktrees.join("moved");
    let refused: Output = git(checkout.as_path(), &["worktree", "move", shown(slot.as_path()), shown(moved.as_path())]);
    let said: String = format!("{}{}", String::from_utf8_lossy(refused.stdout.as_slice()), String::from_utf8_lossy(refused.stderr.as_slice()));
    assert!(!refused.status.success(), "a mounted clone's move is refused by the kernel: {said}");
    assert!(said.to_lowercase().contains("busy"), "the refusal is the busy mount: {said}");
    println!("MOVE-REFUSAL mounted git=refused-busy");

    // Unmounted: the kernel no longer refuses and Git's own move succeeds, which is the hole the wrapper closes.
    unmount_at(slot.as_path());
    let unmounted: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(class_of(&classify_slot(&unmounted)), "known unmounted clone");
    assert_eq!(intent_word(&unmounted.intent), "mount-requested");
    println!("SLOT-STATE unmounted class={} intent={}", class_of(&classify_slot(&unmounted)), intent_word(&unmounted.intent));
    let unmounted_refusal: ManagedMoveRefusal = refuse_move_of_a_bound_slot(common.as_path(), slot.as_path()).expect("the unmounted slot refuses too");
    assert!(matches!(unmounted_refusal.cause, ManagedMoveRefusalCause::Bound { .. }));
    println!("MOVE-REFUSAL unmounted cause=bound");
    let allowed: Output = git(checkout.as_path(), &["worktree", "move", shown(slot.as_path()), shown(moved.as_path())]);
    assert!(
        allowed.status.success(),
        "Git's own move succeeds while the clone is unmounted: {}{}",
        String::from_utf8_lossy(allowed.stdout.as_slice()),
        String::from_utf8_lossy(allowed.stderr.as_slice())
    );
    println!("MOVE-REFUSAL unmounted git=succeeded");
    command_ok("git", &["-C", shown(checkout.as_path()), "worktree", "move", shown(moved.as_path()), shown(slot.as_path())]);

    // A pending add at a slot the add has not mounted, and a pending removal whose plan names this clone.
    publish(&two, Phase::Snapshotted, sbin.as_str(), SLOT_TWO);
    let adding: SlotReadings = readings_of(&two, SLOT_TWO, second.as_path());
    assert_eq!(class_of(&classify_slot(&adding)), "pending add");
    println!("SLOT-STATE pending-add class={}", class_of(&classify_slot(&adding)));
    let plans: PathBuf = plans_folder(common.as_path());
    std::fs::create_dir_all(plans.as_path()).expect("the plan folder is made");
    let clone_guid = guid_of(sbin.as_str(), clone.as_str());
    std::fs::write(
        plans.join(clone_guid.file_name()),
        format!(
            "{}\ncloneGuid={}\ncloneDataset={clone}\n",
            git_policy_cli::worktree_clone_binding_record::plan_folder_read::PLAN_HEADER,
            clone_guid.file_name()
        ),
    )
    .expect("the plan writes");
    let removing: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(class_of(&classify_slot(&removing)), "pending removal");
    println!("SLOT-STATE pending-removal class={}", class_of(&classify_slot(&removing)));

    // The fixture's own cleanup: the plan, the base dataset and the repository.
    std::fs::remove_file(plans.join(clone_guid.file_name())).expect("the plan is removed");
    let _unmounted = zfs(sbin.as_str(), "zfs", &["unmount", checkout_dataset.as_str()], None);
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", format!("{pool}/step8").as_str()]);
    let _cleared: std::io::Result<()> = std::fs::remove_dir_all(checkout.as_path());
    println!("SLOT-STATE done");
}
