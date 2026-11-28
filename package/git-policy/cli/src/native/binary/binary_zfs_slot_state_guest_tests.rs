//! What:
//!  Plan step 12's arms in the throwaway guest,
//!  on the design's layout:
//!  one real Git registration and one real clone,
//!  the mounted and the unmounted class of the same slot,
//!  a record that disagrees with the mount,
//!  a pending add and a pending removal,
//!  and the wrapper's `git worktree move` refusal beside Git's own answer,
//!  which is `Device or resource busy` while mounted and success while
//!  unmounted,
//!  and the removal the wrapper runs itself where Git's own would have deleted
//!  the registration and the underlay pointer while the clone dataset stood.
//! Why:
//!  The kernel refuses a move only while the clone is mounted,
//!  so the unmounted arm is the one that shows why the wrapper needs a refusal
//!  of its own
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  the measurement's item 6).
//!  The removal's own order takes the bound slot and stops before its first
//!  write,
//!  because this fixture writes no origin-side inventory,
//!  so the arm shows the wrapper answering the removal rather than handing it to
//!  Git without building the pool the whole removal would destroy.
//!  The arm runs only where `CLI_GIT_ZFS_SLOT_STATE_GUEST` names a disposable
//!  guest and prints `SKIP` everywhere else,
//!  so a host run that ran nothing says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mounted = reportOf(readSlot(common, 'one', slot, { mounts, intents })); // mounted managed clone
//! const refusal = refuseMoveOfABoundSlot(common, slot); // bound, mounted or not
//! ```

use super::the_destruction_plan_a_control_plants::{
    ACloneAPlanNames,
    APlanAControlPlants,
    plant,
};
use super::zfs_slot_state_guest_fixture::{
    Place,
    class_of,
    createtxg_of,
    git,
    guid_of,
    intent_word,
    publish,
    readings_of,
};
use super::zfs_sweep_guest::{
    CHECKOUT_VARIABLE,
    NONCE,
    ORIGIN_PREFIX,
    POOL_VARIABLE,
    SBIN_VARIABLE,
    clone_the_checkouts_snapshot,
    command_ok,
    make_the_designs_layout,
    mount_request,
    prepare_the_store,
    required,
    sweep,
    unmount_at,
    write_configuration,
    write_passing_probe_record,
    zfs,
    zfs_ignored,
};
use git_policy_cli::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use git_policy_cli::worktree_clone_binding_record::binding_words::Phase;
use git_policy_cli::worktree_clone_move_refusal::managed_move_refusal::{
    ManagedMoveRefusal,
    ManagedMoveRefusalCause,
    refuse_move_of_a_bound_slot,
};
use git_policy_cli::worktree_clone_slot_state::slot_state_classification::classify_slot;
use git_policy_cli::worktree_clone_slot_state::slot_state_readings::SlotReadings;
use git_policy_cli::worktree_clone_slot_state::slot_state_report::report_of;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Output;

/// The variable the guest driver sets for these arms.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SLOT_STATE_GUEST";

/// The slot this arm registers,
/// clones and moves.
const SLOT: &str = "one";

/// The second slot a pending add's record is filed for.
const SLOT_TWO: &str = "two";

/// Whether this run is inside the guest.
fn in_the_guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP worktree_clone_slot_state::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One checkout path as the argument Git takes.
fn shown(path: &Path) -> &str {
    return path.to_str().expect("the guest's paths are UTF-8");
}

/// The wrapper program the driver built,
/// which the arm reaches through a `git` link so it runs as the person's `git`.
const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// What:
///  Run one Git command through the built wrapper,
///  the way a person's shell starts it.
/// Why:
///  Plan step 12's wiring is the command path itself,
///  so the arm proves the refusal,
///  the restore and the report a person meets rather than the function behind
///  them.
fn wrapper_git(
    checkout: &Path,
    arguments: &[&str],
) -> Output {
    let bin: PathBuf = PathBuf::from("/root/slot-state-bin");
    std::fs::create_dir_all(bin.as_path()).expect("the wrapper's bin folder");
    let home: PathBuf = PathBuf::from("/root/slot-state-home");
    std::fs::create_dir_all(home.as_path()).expect("the wrapper's home folder");
    let runtime: PathBuf = PathBuf::from("/root/slot-state-runtime");
    std::fs::create_dir_all(runtime.as_path()).expect("the wrapper's runtime folder");
    let link: PathBuf = bin.join("git");
    let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
    std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's git link");
    return std::process::Command::new(link.as_path())
        .args(arguments)
        .current_dir(checkout)
        .env_clear()
        .env(
            "PATH",
            format!(
                "{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                bin.display()
            ),
        )
        .env("HOME", home.as_path())
        .env("XDG_RUNTIME_DIR", runtime.as_path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .output()
        .unwrap_or_else(|error| {
            panic!("the wrapper `git {arguments:?}` could not start: {error}")
        });
}

/// One command's whole answer as text.
fn said_of(output: &Output) -> String {
    return format!(
        "{}{}",
        String::from_utf8_lossy(output.stdout.as_slice()),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
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
    let (checkout_dataset, container): (String, String) =
        make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
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
        command_ok(
            "git",
            &[
                "-C",
                shown(checkout.as_path()),
                "worktree",
                "add",
                "--quiet",
                "--no-checkout",
                shown(place.as_path()),
                "HEAD",
            ],
        );
    }
    let slot_identity: FolderIdentity = FolderIdentity::of_folder(slot.as_path())
        .expect("the underlay's identity reads before the mount");
    // The pointer the add keeps for prune is the underlay's own, and the clone's copy replaces it under the mount.
    let pointer: Vec<u8> =
        std::fs::read(slot.join(".git")).expect("the underlay pointer reads before the mount");
    let second_pointer: Vec<u8> =
        std::fs::read(second.join(".git")).expect("the second slot's underlay pointer reads");
    write_passing_probe_record();
    prepare_the_store(checkout.as_path());
    write_configuration(checkout.as_path(), None, None);
    let clone: String = clone_the_checkouts_snapshot(
        sbin.as_str(),
        checkout_dataset.as_str(),
        container.as_str(),
        SLOT,
        NONCE,
        Some(mount_request(NONCE).as_str()),
    );
    let common: PathBuf = checkout.join(".git");
    let place: Place = Place {
        common: common.clone(),
        pool: pool.clone(),
        checkout_dataset: checkout_dataset.clone(),
        clone_dataset: clone.clone(),
        slot: slot.clone(),
        slot_identity,
        pointer,
        registration: common.join("worktrees").join(SLOT),
    };
    let attached = sweep();
    assert_eq!(
        attached.status,
        Some(0),
        "the sweep attached the clone: {}",
        attached.said
    );
    // The add's own exchange, measured as item 1's order: the clone's copy of the checkout's administration is moved
    // aside under the mount and the registration's pointer installed in its place, so the slot Git validates is the
    // clone's root rather than the copied repository.
    std::fs::rename(slot.join(".git"), slot.join("git-administered-copy"))
        .expect("the copied administration moves aside");
    std::fs::write(
        slot.join(".git"),
        format!("gitdir: {}\n", place.registration.display()),
    )
    .expect("the registration pointer is installed under the mount");
    let two: Place = Place {
        common: common.clone(),
        pool: pool.clone(),
        checkout_dataset: checkout_dataset.clone(),
        clone_dataset: format!("{container}/{SLOT_TWO}"),
        slot: second.clone(),
        slot_identity: FolderIdentity::of_folder(second.as_path())
            .expect("the second slot's identity reads"),
        pointer: second_pointer,
        registration: common.join("worktrees").join(SLOT_TWO),
    };

    // The mounted class, the report, and a record that disagrees with the mount.
    publish(&place, Phase::Filled, sbin.as_str(), SLOT);
    let mounted: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    let mounted_report = report_of(&mounted);
    assert_eq!(
        class_of(&classify_slot(&mounted)),
        "mounted managed clone",
        "{:?}",
        mounted_report.lines()
    );
    assert!(
        mounted_report
            .next_safe_action
            .contains("`git worktree move` is refused"),
        "{}",
        mounted_report.next_safe_action
    );
    println!(
        "SLOT-STATE mounted class={} dataset={}",
        mounted_report.state, mounted_report.dataset
    );
    publish(&place, Phase::Cloned, sbin.as_str(), SLOT);
    let behind: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(class_of(&classify_slot(&behind)), "cannot be proved");
    println!(
        "SLOT-STATE record-behind-mount class={}",
        class_of(&classify_slot(&behind))
    );
    publish(&place, Phase::Filled, sbin.as_str(), SLOT);

    // Mounted: the wrapper refuses and Git's own move meets the kernel's refusal.
    let bound: ManagedMoveRefusal = refuse_move_of_a_bound_slot(common.as_path(), slot.as_path())
        .expect("a bound slot refuses");
    assert!(
        matches!(bound.cause, ManagedMoveRefusalCause::Bound { .. }),
        "{:?}",
        bound.cause
    );
    println!("MOVE-REFUSAL mounted cause=bound");
    let moved: PathBuf = worktrees.join("moved");
    let refused: Output = git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    let said: String = format!(
        "{}{}",
        String::from_utf8_lossy(refused.stdout.as_slice()),
        String::from_utf8_lossy(refused.stderr.as_slice())
    );
    assert!(
        !refused.status.success(),
        "a mounted clone's move is refused by the kernel: {said}"
    );
    assert!(
        said.to_lowercase().contains("busy"),
        "the refusal is the busy mount: {said}"
    );
    println!("MOVE-REFUSAL mounted git=refused-busy");
    // The wrapper's own command path refuses the same move before Git runs.
    let by_the_wrapper: Output = wrapper_git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    assert_eq!(
        by_the_wrapper.status.code(),
        Some(1),
        "the wrapper's refusal status: {}",
        said_of(&by_the_wrapper)
    );
    assert!(
        said_of(&by_the_wrapper).contains("`git worktree move` is refused"),
        "{}",
        said_of(&by_the_wrapper)
    );
    assert!(slot.is_dir(), "the wrapper moved nothing");
    println!("MOVE-REFUSAL wrapper-mounted cause=bound status=refused");

    // Unmounted: the kernel no longer refuses and Git's own move succeeds, which is the hole the wrapper closes.
    unmount_at(slot.as_path());
    let unmounted: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(
        class_of(&classify_slot(&unmounted)),
        "known unmounted clone"
    );
    assert_eq!(intent_word(&unmounted.intent), "mount-requested");
    println!(
        "SLOT-STATE unmounted class={} intent={}",
        class_of(&classify_slot(&unmounted)),
        intent_word(&unmounted.intent)
    );
    let unmounted_refusal: ManagedMoveRefusal =
        refuse_move_of_a_bound_slot(common.as_path(), slot.as_path())
            .expect("the unmounted slot refuses too");
    assert!(matches!(
        unmounted_refusal.cause,
        ManagedMoveRefusalCause::Bound { .. }
    ));
    println!("MOVE-REFUSAL unmounted cause=bound");
    let refused_unmounted: Output = wrapper_git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    assert_eq!(
        refused_unmounted.status.code(),
        Some(1),
        "the wrapper refuses the unmounted move: {}",
        said_of(&refused_unmounted)
    );
    assert!(
        said_of(&refused_unmounted).contains("`git worktree move` is refused"),
        "{}",
        said_of(&refused_unmounted)
    );
    assert!(
        slot.is_dir(),
        "the wrapper moved nothing while the clone was unmounted"
    );
    println!("MOVE-REFUSAL wrapper-unmounted cause=bound status=refused");
    let allowed: Output = git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    assert!(
        allowed.status.success(),
        "Git's own move succeeds while the clone is unmounted: {}{}",
        String::from_utf8_lossy(allowed.stdout.as_slice()),
        String::from_utf8_lossy(allowed.stderr.as_slice())
    );
    println!("MOVE-REFUSAL unmounted git=succeeded");
    command_ok(
        "git",
        &[
            "-C",
            shown(checkout.as_path()),
            "worktree",
            "move",
            shown(moved.as_path()),
            shown(slot.as_path()),
        ],
    );

    // The wrapper answers the removal itself and Git never runs: a whole managed clone is bound to the slot, so the
    // removal's own order takes it. This fixture writes no origin-side inventory, so the order stops before its first
    // write and nothing is destroyed. The registration and the underlay pointer are therefore still what Git wrote,
    // which is the whole point of the step: Git's own removal would have deleted both while the clone dataset stood.
    let removal: Output = wrapper_git(
        checkout.as_path(),
        &["worktree", "remove", "--force", shown(slot.as_path())],
    );
    assert_eq!(
        removal.status.code(),
        Some(1),
        "the removal stops before it writes: {}",
        said_of(&removal)
    );
    assert!(
        said_of(&removal).contains("runs here: a whole managed clone is bound to this slot"),
        "the wrapper's own removal runs and Git does not: {}",
        said_of(&removal)
    );
    assert!(
        said_of(&removal).contains("Nothing was destroyed, and the registration stands."),
        "the stop says what it left: {}",
        said_of(&removal)
    );
    assert!(
        said_of(&removal).contains(&format!(
            "the origin-side inventory the add recorded for the slot `{SLOT}` could not be read"
        )),
        "the stop names the record this fixture never wrote: {}",
        said_of(&removal)
    );
    assert!(place.registration.is_dir(), "the registration stands");
    assert!(slot.join(".git").is_file(), "the underlay pointer stands");
    println!(
        "REMOVAL-RUNS wrapper class=known-unmounted-clone stop=origin-inventory status=refused"
    );

    // The same answer behind a shell alias: Git runs the alias's shell at the top of the working tree, and the
    //  operand is a path from there with a `.` step in it, so no registered path ends with it and the run has to
    //  resolve it against the directory Git runs the command in. The host controls hold that rule
    //  (`a_bound_slots_removal_reads_its_operand_where_git_runs_the_alias`); this arm repeats it once on a real pool.
    let by_an_alias: Output = wrapper_git(
        checkout.as_path(),
        &[
            "-c",
            "alias.rmwt=!git worktree remove --force",
            "rmwt",
            format!(".claude/worktrees/./{SLOT}").as_str(),
        ],
    );
    assert_eq!(
        by_an_alias.status.code(),
        Some(1),
        "the removal behind a shell alias stops at the same place: {}",
        said_of(&by_an_alias)
    );
    assert!(
        said_of(&by_an_alias).contains("runs here: a whole managed clone is bound to this slot"),
        "the alias's operand is resolved to the bound slot the step answers: {}",
        said_of(&by_an_alias)
    );
    assert!(
        said_of(&by_an_alias).contains("the origin-side inventory the add recorded for the slot"),
        "the alias's removal stopped at the same record: {}",
        said_of(&by_an_alias)
    );
    assert!(place.registration.is_dir(), "the registration stands");
    assert!(slot.join(".git").is_file(), "the underlay pointer stands");
    println!("REMOVAL-RUNS alias class=known-unmounted-clone stop=origin-inventory status=refused");

    // The pointer the add keeps for prune: real Git's own prune keeps the registration while the pointer stands.
    let pruned_intact: Output = git(checkout.as_path(), &["worktree", "prune", "--verbose"]);
    assert!(
        pruned_intact.status.success(),
        "prune ran: {}",
        said_of(&pruned_intact)
    );
    assert!(
        place.registration.is_dir(),
        "the registration stands after a prune with the pointer intact"
    );
    assert!(
        slot.join(".git").is_file(),
        "the underlay pointer stands after the prune"
    );
    println!("PRUNE-POINTER intact registration=kept");

    // A pointer an account deleted: the wrapper's prune writes the recorded bytes back before Git starts, and the
    // registration it would have dropped stands.
    std::fs::remove_file(slot.join(".git")).expect("the underlay pointer goes");
    let restored: Output = wrapper_git(checkout.as_path(), &["worktree", "prune"]);
    assert!(
        restored.status.success(),
        "the wrapped prune ran: {}",
        said_of(&restored)
    );
    assert!(
        said_of(&restored).contains("restored the pointer"),
        "{}",
        said_of(&restored)
    );
    assert!(
        place.registration.is_dir(),
        "the registration stands after the restore"
    );
    assert_eq!(
        std::fs::read(slot.join(".git")).expect("the restored pointer reads"),
        place.pointer
    );
    println!("PRUNE-POINTER restored registration=kept");

    // A damaged link with the registration surviving: the report says what a repair restores, and Git rebuilds it.
    std::fs::remove_file(slot.join(".git")).expect("the underlay pointer goes");
    let repaired: Output = wrapper_git(
        checkout.as_path(),
        &["worktree", "repair", shown(slot.as_path())],
    );
    assert!(
        said_of(&repaired).contains("link: damaged, and the registration stands"),
        "{}",
        said_of(&repaired)
    );
    assert!(
        said_of(&repaired).contains("next safe command: `git worktree repair"),
        "{}",
        said_of(&repaired)
    );
    assert!(
        slot.join(".git").is_file(),
        "Git rebuilt the pointer from the registration"
    );
    println!("REPAIR damaged registration=stands");

    // A pending add at a slot the add has not mounted, and a pending removal whose plan names this clone.
    publish(&two, Phase::Snapshotted, sbin.as_str(), SLOT_TWO);
    let adding: SlotReadings = readings_of(&two, SLOT_TWO, second.as_path());
    assert_eq!(class_of(&classify_slot(&adding)), "pending add");
    println!(
        "SLOT-STATE pending-add class={}",
        class_of(&classify_slot(&adding))
    );
    // The removal's own plan, published by the removal's own writer: the state its first write leaves, with the
    // head present and every line planned. The arm made this clone and never snapshotted it, so its inventory is empty.
    let plan: APlanAControlPlants = APlanAControlPlants::a_removals_first_write(ACloneAPlanNames {
        clone_createtxg: createtxg_of(sbin.as_str(), clone.as_str()),
        origin_dataset: checkout_dataset.clone(),
        origin_snapshot: format!("{checkout_dataset}@{ORIGIN_PREFIX}{NONCE}"),
        ..ACloneAPlanNames::a_fixture_clone(
            common.as_path(),
            clone.as_str(),
            guid_of(sbin.as_str(), clone.as_str()),
        )
    });
    let plan_path: PathBuf = plant(&plan);
    let removing: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(class_of(&classify_slot(&removing)), "pending removal");
    println!(
        "SLOT-STATE pending-removal class={}",
        class_of(&classify_slot(&removing))
    );

    // The plan goes first, so the prune below reads a slot no removal is in flight for.
    std::fs::remove_file(plan_path.as_path()).expect("the plan is removed");

    // A registration Git already pruned: the report says what is lost, and a pointer restored from the record points at
    // nothing, because Git does not rebuild a registration.
    std::fs::remove_file(slot.join(".git")).expect("the underlay pointer goes");
    let pruned: Output = git(checkout.as_path(), &["worktree", "prune"]);
    assert!(pruned.status.success(), "prune ran: {}", said_of(&pruned));
    assert!(!place.registration.exists(), "the registration is gone");
    let pruned_report: Output = wrapper_git(
        checkout.as_path(),
        &["worktree", "repair", shown(slot.as_path())],
    );
    assert!(
        said_of(&pruned_report).contains("link: pruned, and the registration is gone"),
        "{}",
        said_of(&pruned_report)
    );
    assert!(
        said_of(&pruned_report).contains("cannot rebuild a registration"),
        "{}",
        said_of(&pruned_report)
    );
    println!("REPAIR pruned registration=gone");

    // The fixture's own cleanup: the base dataset and the repository.
    let _unmounted = zfs(
        sbin.as_str(),
        "zfs",
        &["unmount", checkout_dataset.as_str()],
        None,
    );
    zfs_ignored(
        sbin.as_str(),
        &["destroy", "-r", "-f", format!("{pool}/step8").as_str()],
    );
    let _cleared: std::io::Result<()> = std::fs::remove_dir_all(checkout.as_path());
    println!("SLOT-STATE done");
}
