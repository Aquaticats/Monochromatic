//! What:
//!  Step 4's guest arms of the descriptor attach,
//!  run inside the throwaway `mvm` guest the recipe in
//!  `package/zfs/request-to-openzfs-kernel-device/e2e/` makes:
//!  the clone mounted onto the slot the walk held,
//!  the setuid control on that mount's enforced flags,
//!  item 2's refusals with an account-owned parent and slot,
//!  the attach that lands beside its slot after a rename and is rolled back,
//!  a mount made beside the held slot that the design's first proof refuses,
//!  and the decoy control of H1 gate 1.
//! Why:
//!  The real `fsmount`,
//! the `move_mount` onto a held descriptor and the mount table an attach's flag is read back from
//!  need a kernel that lets this process mount,
//!  which the host's tests do not have
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 4,
//! "VM measurements";
//!  `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
//!  items 2 and 7).
//!  Until step 5's `mount-git-worktree-clone-datasets` binary exists,
//!  these arms are a root-run integration test of this module rather than part of the account-side suite,
//!  and the three mount calls are the test's own:
//!  the production attach that step 5 ships is the code the arms will drive instead.
//!  The mounts land in the guest's boot mount namespace,
//!  each is taken down by the guard that made it however the control ended,
//!  and the recipe destroys the guest.
//!  Item 2's measured arms map here as follows:
//!  a slot that already holds the clone's mount is the `already-mounted` refusal,
//!  a real clone mount at the slot rather than a bind,
//!  and a path through `/proc/self/fd` is refused before any walk,
//!  as a checkout that is not its own canonical path
//!  (`clone_slot_walk_tests::a_checkout_named_through_a_link_is_refused`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = holdCloneSlotUnderRootControl({ checkout, slot, store });
//! const attached = attachTheClone({ clone, target: held.target, flags: ATTACH_MOUNT_FLAGS }); // fsmount + move_mount
//! const outcome = postAttachCheck({ ...held, attachedMountId: attached }, lookAgainAtTheSlot({ checkout, slot }));
//! ```

use super::attach_post_check::{AttachOutcome, AttachPostCheck, MisplacedReason, post_attach_check};
use super::attached_mount_identity::{ATTACH_MOUNT_FLAGS, MountIdentity, mount_facts_of, mount_identity_of};
use super::clone_folder_identity::CloneFolderIdentity;
use super::clone_slot_walk::{
    FreshSlotLookup, HeldCloneSlot, HoldCloneSlotRequest, LookAgainAtSlotRequest, hold_clone_slot_under_root_control, look_again_at_the_slot, mount_id_at_the_held_target,
};
use super::clone_slot_walk_refusals::{CloneSlotRefusal, WalkRefusalReason};
use super::misplaced_attach_rollback::{RollbackPlan, plan_rollback};
use super::slot_identity_store::{append_slot_identity_under_root_control, record_parent_identity_under_root_control};
use super::slot_identity_store_records::STORE_HEADER;
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::fs_mount_options::MountFacts;
use std::ffi::CString;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Output;

//region The guest and its fixture

/// The variable that names the guest this recipe made,
/// which every arm compares with the running machine's host name.
const GUEST_VARIABLE: &str = "MONOCHROMATIC_ZFS_ATTACH_ARM_GUEST";

/// The variable that names the folder the recipe's pool is mounted at.
const FIXTURE_ROOT_VARIABLE: &str = "MONOCHROMATIC_ZFS_ATTACH_ARM_ROOT";

/// The variable that names the guest's account,
/// which owns the checkout,
/// the worktrees parent and every slot the arms judge.
const ACCOUNT_VARIABLE: &str = "MONOCHROMATIC_ZFS_ATTACH_ARM_ACCOUNT";

/// The variable that names the ZFS dataset the arms mount at a slot.
const CLONE_VARIABLE: &str = "MONOCHROMATIC_ZFS_ATTACH_ARM_CLONE";

/// The variable that names the second clone,
/// which stays mounted at another slot while item 2's refusals run.
const BESIDE_CLONE_VARIABLE: &str = "MONOCHROMATIC_ZFS_ATTACH_ARM_CLONE_BESIDE";

/// The slot every arm creates unless it names another.
const THE_SLOT: &str = "one";

/// The slot that holds a mount while the refusals run,
/// which is what "leaves existing mounts alone" is read against.
const THE_BESIDE_SLOT: &str = "beside";

/// The slots item 2's refusals are planted on,
/// one per refusal,
/// so one plant never sees what another plant left.
const REFUSAL_SLOTS: [&str; 9] = ["link-slot", "planted-link", "new-inode", "wrong-owner", "widened-mode", "access-acl", "default-acl", "boundary", "already-mounted"];

/// How many refusals the arm asserts:
/// one per refusal slot,
/// a link at each of the two parent folders,
/// and the parent escape the walk refuses as a name.
const REFUSAL_ARMS: usize = REFUSAL_SLOTS.len() + 3;

/// What:
///  Whether this run is inside the guest the recipe made,
/// as the guest's root,
///  and the variables every arm reads.
/// Why:
///   These arms mount file systems,
///  so a run anywhere else must refuse rather than mount on a machine nobody offered;
///  with the variable unset the test prints a `SKIP` line and returns,
///  which the recipe refuses to count as a pass.
fn in_the_guest(test: &str) -> Option<Guest> {
    let Some(named) = std::env::var_os(GUEST_VARIABLE) else {
        println!("SKIP {test}: {GUEST_VARIABLE} is unset, so this is not the guest the recipe made");
        return None;
    };
    // SAFETY: `gethostname` writes at most the bytes of the buffer it is given, whose length it is passed with.
    let mut name: [libc::c_char; 256] = [0; 256];
    let written: libc::c_int = unsafe { libc::gethostname(name.as_mut_ptr(), name.len()) };
    assert_eq!(written, 0, "the host name of this machine could not be read");
    // SAFETY: `name` is the NUL-terminated buffer `gethostname` wrote into.
    let machine: String = unsafe { std::ffi::CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned();
    let expected: String = named.to_string_lossy().into_owned();
    assert_eq!(machine, expected, "these arms run in the guest {expected} and not on {machine}");
    // SAFETY: `geteuid` reads this process's own credentials and touches no memory.
    let euid: libc::uid_t = unsafe { libc::geteuid() };
    assert_eq!(euid, 0, "these arms record a root-owned store and mount a clone, so they run as root and not as {euid}");
    let fixture_root: PathBuf = PathBuf::from(std::env::var_os(FIXTURE_ROOT_VARIABLE).expect("the fixture root is named"));
    let account: String = std::env::var(ACCOUNT_VARIABLE).expect("the account is named");
    let clone: String = std::env::var(CLONE_VARIABLE).expect("the clone is named");
    let beside: String = std::env::var(BESIDE_CLONE_VARIABLE).expect("the second clone is named");
    let uid: u32 = said(&as_the_account(account.as_str(), &["id", "--user"])).parse().expect("the account's user number");
    return Some(Guest { fixture_root, account, uid, clone, beside });
}

/// What:
///  The guest's fixture root,
///  its account and that account's user number,
///  and the two datasets the arms mount.
/// Why:
///   Every arm reads the same recipe's fixture,
///  so one value carries what the arms share and the account is asked for its number once.
struct Guest {
    /// The folder the pool's `checkout` dataset is mounted at.
    fixture_root: PathBuf,
    /// The guest's account,
    /// which owns the checkout the walk starts at.
    account: String,
    /// The account's user number,
    /// which its folders are owned by.
    uid: u32,
    /// The dataset an arm mounts at a slot.
    clone: String,
    /// The dataset that stays mounted at another slot while the refusals run.
    beside: String,
}

/// Run one program as the guest's account,
/// its output kept for the assertion that reads it.
fn as_the_account(account: &str, program: &[&str]) -> Output {
    let mut command: std::process::Command = std::process::Command::new("runuser");
    command.args(["--user", account, "--"]).args(program);
    return command.output().expect("the program runs");
}

/// The words of a program's standard output,
/// which every arm reads an answer from.
fn said(output: &Output) -> String {
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
}

/// What:
///  One arm's fixture:
///  an account-owned checkout with `.claude/worktrees` and the slots the arm names,
///  and the store root recorded both identities in.
/// Why:
///   The walk compares a folder against a record root wrote,
///  so the identities are recorded through the route's own writers as the guest's root,
///  which is the one place that path runs without a user namespace.
struct AttachArm {
    /// The arm's own folder below the fixture root,
    /// owned by root,
    /// because the store and the lock beside it must be root's.
    root: PathBuf,
    /// The checkout the walk starts at,
    /// owned by the account.
    checkout: PathBuf,
    /// `.claude/worktrees`,
    /// owned by the account.
    parent: PathBuf,
    /// The store root recorded the two identities in.
    store: PathBuf,
}

impl AttachArm {
    /// What:
    ///  Build the fixture:
    /// root makes the arm's folder and the checkout,
    ///  the account makes the worktrees folder and the slots below it,
    ///  and root writes the store and records both identities.
    /// Why:
    ///   Item 2 judges folders the account created and still owns,
    ///  and the store is a file only root may write,
    ///  so the fixture is built by both accounts,
    /// as the route's is.
    fn build(guest: &Guest, name: &str, slots: &[&str]) -> AttachArm {
        let root: PathBuf = guest.fixture_root.join("arms").join(name);
        // A run of this step in the same guest builds the same arms again, so a folder an earlier run left is cleared
        // first; nothing outside the arm's own folder is touched.
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("the earlier run's arm folder is removed");
        }
        std::fs::create_dir_all(&root).expect("the arm's folder");
        let checkout: PathBuf = root.join("checkout");
        std::fs::create_dir(&checkout).expect("the arm's checkout");
        std::os::unix::fs::chown(&checkout, Some(guest.uid), Some(guest.uid)).expect("the checkout is the account's");
        let parent: PathBuf = checkout.join(".claude/worktrees");
        let mut paths: Vec<String> = Vec::new();
        for slot in slots {
            paths.push(parent.join(slot).display().to_string());
        }
        let mut arguments: Vec<&str> = vec!["mkdir", "--parents"];
        arguments.extend(paths.iter().map(|path| return path.as_str()));
        let made: Output = as_the_account(guest.account.as_str(), arguments.as_slice());
        assert!(made.status.success(), "the account makes the slots: {}", String::from_utf8_lossy(made.stderr.as_slice()));
        let narrowed: Output = as_the_account(guest.account.as_str(), &["chmod", "0755", parent.display().to_string().as_str()]);
        assert!(narrowed.status.success(), "the account gives the worktrees folder its mode: {}", String::from_utf8_lossy(narrowed.stderr.as_slice()));
        let store: PathBuf = root.join("slot-identities");
        std::fs::write(&store, format!("{STORE_HEADER}\n")).expect("the store's header");
        let built: AttachArm = AttachArm { root, checkout, parent, store };
        let recorded_parent: CloneFolderIdentity = built.identity_of(built.parent.as_path());
        record_parent_identity_under_root_control(built.store.as_path(), &recorded_parent).expect("root records the parent folder");
        for slot in slots {
            let slot_path: PathBuf = built.slot(slot);
            let recorded_slot: CloneFolderIdentity = built.identity_of(slot_path.as_path());
            append_slot_identity_under_root_control(built.store.as_path(), slot, &recorded_slot).expect("root records the slot");
        }
        return built;
    }

    /// The identity of the folder at `path`,
    /// read through a descriptor root opens.
    fn identity_of(&self, path: &Path) -> CloneFolderIdentity {
        let folder: std::fs::File = std::fs::File::open(path).expect("the folder opens");
        return CloneFolderIdentity::capture(&folder).expect("the folder's identity");
    }

    /// The path of one slot below `.claude/worktrees`.
    fn slot(&self, slot: &str) -> PathBuf {
        return self.parent.join(slot);
    }

    /// What:
    ///  Hold one slot through the route's walk,
    ///  which reads the store as root's own and compares both records.
    /// Why:
    ///   Every attach starts here,
    ///  and a refusal is asserted against this call rather than against a path.
    fn hold(&self, slot: &str) -> Result<HeldCloneSlot, CloneSlotRefusal> {
        return hold_clone_slot_under_root_control(&HoldCloneSlotRequest { checkout: self.checkout.as_path(), slot, store: self.store.as_path() });
    }

    /// Remove the arm's own folder,
    /// and nothing outside it.
    fn remove(self) {
        std::fs::remove_dir_all(&self.root).expect("the arm's folder is removed");
    }
}

//endregion The guest and its fixture

//region The mount the test makes

/// The `fsopen` flag that keeps the file-system context out of a started child.
const FSOPEN_CLOEXEC: libc::c_uint = 0x0000_0001;

/// The `fsconfig` command that sets one option of the context.
const FSCONFIG_SET_STRING: libc::c_uint = 1;

/// The `fsconfig` command that creates the mount from the context.
const FSCONFIG_CMD_CREATE: libc::c_uint = 6;

/// The `fsmount` flag that keeps the mount out of a started child.
const FSMOUNT_CLOEXEC: libc::c_uint = 0x0000_0001;

/// The `move_mount` flag that reads the file-system descriptor rather than a path.
const MOVE_MOUNT_F_EMPTY_PATH: libc::c_uint = 0x0000_0004;

/// The `move_mount` flag that mounts onto the target descriptor rather than a path,
/// which is the design's attach.
const MOVE_MOUNT_T_EMPTY_PATH: libc::c_uint = 0x0000_0040;

/// The file-system type a ZFS dataset is mounted as.
const ZFS_FILE_SYSTEM_TYPE: &str = "zfs";

/// The `fsconfig` key that names the dataset to mount.
const SOURCE_OPTION: &str = "source";

/// The file system's own mount call,
/// which the decoy control uses for the path mount the design forbids.
const BIND_MOUNT: libc::c_ulong = libc::MS_BIND;

/// What:
///  A clone mounted at one folder,
///  taken down when this guard ends however the control ended.
/// Why:
///   Every arm that mounts must leave the folder as it found it even when an assertion panics,
///  or the next arm's mount of the same dataset is refused as busy;
///  the guard takes down only the mount whose number is the one it recorded,
///  so a cleanup can never take down another arm's attach.
///  The number is read through the path,
///  because the guard and the rollback arm name where the mount landed after a rename,
///  while the post-attach check reads the mount at the held target's name beneath the held parent.
struct MountedClone {
    /// Where the mount landed.
    path: PathBuf,
    /// The kernel's number for the mount this guard made.
    mount_id: u64,
}

impl Drop for MountedClone {
    fn drop(&mut self) {
        if mount_id_at(self.path.as_path()) != Some(self.mount_id) {
            eprintln!("the mount this guard made is gone and another mount stands at {}, so it is left alone", self.path.display());
            return;
        }
        if let Err(error) = unmount_at(self.path.as_path()) {
            eprintln!("the mount this guard made could not be taken down: {error}");
        }
    }
}

/// The mount number a path resolves to now,
/// or nothing when the path cannot be opened.
fn mount_id_at(path: &Path) -> Option<u64> {
    let opened: std::fs::File = std::fs::File::open(path).ok()?;
    return mount_id_of(&opened).ok();
}

/// Take the mount at `path` under a guard that removes it however the control ends.
fn guard_the_mount(path: &Path) -> MountedClone {
    let mount_id: u64 = mount_id_at(path).expect("the mount the control made is there");
    return MountedClone { path: path.to_path_buf(), mount_id };
}

/// What:
///  The file-system context of `clone`,
///  created and made into a mount with `flags`.
/// Why:
///   `attach_the_clone` and `attach_the_clone_by_path` differ only in where the mount is moved to,
///  and the design's attach and the control's path mount must start from the same context
///  for the decoy to prove what the difference is.
fn a_mount_of(clone: &str, flags: libc::c_ulong) -> Result<libc::c_int, String> {
    let file_system_type: CString = CString::new(ZFS_FILE_SYSTEM_TYPE).expect("no NUL in the file-system type");
    let source_name: CString = CString::new(SOURCE_OPTION).expect("no NUL in the option's name");
    let source_value: CString = CString::new(clone).expect("no NUL in the dataset's name");
    // SAFETY: every argument is a NUL-terminated string or a flag the call reads, and the returned descriptor is
    // owned by this function from the moment it is checked.
    let context: libc::c_int = unsafe { libc::syscall(libc::SYS_fsopen, file_system_type.as_ptr(), FSOPEN_CLOEXEC) as libc::c_int };
    if context < 0 {
        return Err(format!("fsopen {ZFS_FILE_SYSTEM_TYPE}: {}", std::io::Error::last_os_error()));
    }
    // SAFETY: the descriptor and both strings outlive each call, which reads them and returns a number.
    let set: libc::c_int = unsafe { libc::syscall(libc::SYS_fsconfig, context, FSCONFIG_SET_STRING, source_name.as_ptr(), source_value.as_ptr(), 0) as libc::c_int };
    if set < 0 {
        let said: std::io::Error = std::io::Error::last_os_error();
        // SAFETY: the context descriptor is this function's own and is closed exactly once.
        unsafe { libc::close(context) };
        return Err(format!("fsconfig {SOURCE_OPTION}={clone}: {said}"));
    }
    // SAFETY: the context descriptor is this function's own and `fsconfig` reads it and no other pointer.
    let created: libc::c_int = unsafe { libc::syscall(libc::SYS_fsconfig, context, FSCONFIG_CMD_CREATE, std::ptr::null::<libc::c_char>(), std::ptr::null::<libc::c_char>(), 0) as libc::c_int };
    if created < 0 {
        let said: std::io::Error = std::io::Error::last_os_error();
        // SAFETY: the context descriptor is this function's own and is closed exactly once.
        unsafe { libc::close(context) };
        return Err(format!("fsconfig FSCONFIG_CMD_CREATE {clone}: {said}"));
    }
    // SAFETY: the context descriptor is this function's own, which is the descriptor `fsmount` makes the mount from;
    // the error is read before the descriptor is closed so the number is the call's own.
    let mount: libc::c_int = unsafe { libc::syscall(libc::SYS_fsmount, context, FSMOUNT_CLOEXEC, u32::try_from(flags).expect("the flags fit a word")) as libc::c_int };
    let mount_error: std::io::Error = std::io::Error::last_os_error();
    // SAFETY: the context descriptor is closed exactly once, after the mount call read it.
    unsafe { libc::close(context) };
    if mount < 0 {
        return Err(format!("fsmount {clone}: {mount_error}"));
    }
    return Ok(mount);
}

/// What:
///  Mount the dataset `clone` at the folder `target` through the descriptor the walk held,
///  with `flags`.
/// Why:
///   The host's tests cannot mount,
///  so these arms run the primitive itself and then judge it with the module's own reads;
///  the production copy of these calls is step 5's binary.
fn attach_the_clone(clone: &str, target: &std::fs::File, flags: libc::c_ulong) -> Result<(), String> {
    let mount: libc::c_int = a_mount_of(clone, flags)?;
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    // SAFETY: the mount descriptor and the target descriptor are open, and the two empty paths are the
    // NUL-terminated strings the flags tell the call to read from those descriptors.
    let moved: libc::c_int = unsafe {
        libc::syscall(libc::SYS_move_mount, mount, empty.as_ptr(), target.as_raw_fd(), empty.as_ptr(), MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH) as libc::c_int
    };
    // SAFETY: the mount descriptor is this function's own and the move consumed what it named.
    unsafe { libc::close(mount) };
    if moved < 0 {
        return Err(format!("move_mount {clone} onto a descriptor: {}", std::io::Error::last_os_error()));
    }
    return Ok(());
}

/// What:
///  Mount the dataset `clone` at the path `target`,
///  which is the check-then-path behaviour the design forbids and the decoy control reproduces.
/// Why:
///   A mount moved onto a path can land wherever that path names,
///  while the walk that passed judged another folder;
///  the control needs exactly that difference from `attach_the_clone`.
fn attach_the_clone_by_path(clone: &str, target: &Path, flags: libc::c_ulong) -> Result<(), String> {
    let mount: libc::c_int = a_mount_of(clone, flags)?;
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    let named: CString = CString::new(target.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: the mount descriptor and the named path outlive the call, and the empty source path is the
    // NUL-terminated string the flag tells the call to read from that descriptor.
    let moved: libc::c_int = unsafe { libc::syscall(libc::SYS_move_mount, mount, empty.as_ptr(), libc::AT_FDCWD, named.as_ptr(), MOVE_MOUNT_F_EMPTY_PATH) as libc::c_int };
    // SAFETY: the mount descriptor is this function's own and the move consumed what it named.
    unsafe { libc::close(mount) };
    if moved < 0 {
        return Err(format!("move_mount {clone} onto a path: {}", std::io::Error::last_os_error()));
    }
    return Ok(());
}

/// What:
///  Take the mount at the folder `target` down through the descriptor rather than through a path.
/// Why:
///   The design's rollback acts on the descriptor the attach verified,
///  so a path resolved again could name another folder by then;
///  the kernel resolves the descriptor's own entry,
/// which follows the folder even after a rename.
fn unmount_through(target: &std::fs::File) -> Result<(), std::io::Error> {
    let path: CString = CString::new(format!("/proc/self/fd/{}", target.as_raw_fd())).expect("no NUL in a descriptor path");
    // SAFETY: `path` is a NUL-terminated string that outlives the call, and no flag is passed.
    let taken: libc::c_int = unsafe { libc::umount2(path.as_ptr(), 0) };
    return match taken {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    };
}

/// The mount table's facts for the mount at `target`,
/// read through the module's own reader.
fn facts_at(target: &std::fs::File) -> MountFacts {
    let mount_id: u64 = mount_id_of(target).expect("the folder's mount number");
    return mount_facts_of(mount_id).expect("the mount is listed");
}

/// Make the bind mount `source` over `target`,
/// which is the control's own mount rather than the module's.
fn bind_over(source: &Path, target: &Path) -> Result<(), String> {
    let from: CString = CString::new(source.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    let onto: CString = CString::new(target.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: both paths are NUL-terminated strings that outlive the call, no file-system type or data is passed, and
    // the flags ask for a bind of one existing folder onto another.
    let bound: libc::c_int = unsafe { libc::mount(from.as_ptr(), onto.as_ptr(), std::ptr::null(), BIND_MOUNT, std::ptr::null()) };
    return match bound {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error().to_string()),
    };
}

/// Take down the mount a control made at `target` by its own path,
/// which is a control's cleanup and never the route's unmount.
fn unmount_at(target: &Path) -> Result<(), String> {
    let named: CString = CString::new(target.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: the path is a NUL-terminated string that outlives the call, and no flag is passed.
    let taken: libc::c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
    return match taken {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error().to_string()),
    };
}

/// What:
///  Assert the clone mounted at the beside slot is still the mount the control made,
///  after `refusal` answered.
/// Why:
///   "Leaves existing mounts alone" is a claim about other mounts,
///  so it is read after every refusal rather than once at the end.
fn assert_the_beside_mount_is_alone(slot: &Path, mount: &MountedClone, refusal: &CloneSlotRefusal) {
    assert_eq!(mount_id_at(slot), Some(mount.mount_id), "the beside mount is untouched by: {}", refusal.message());
    let opened: std::fs::File = std::fs::File::open(slot).expect("the beside slot opens");
    let facts: MountFacts = facts_at(&opened);
    assert_eq!(facts.mount_id, mount.mount_id, "the mount table still lists the beside mount after: {}", refusal.message());
}

//endregion The mount the test makes

//region The arms

/// What:
///  The clone the walk held lands at the slot it held,
///  the kernel lists the enforced flags,
///  and the post-attach check finds that mount at the canonical slot.
/// Why:
///  This is the positive control of every other arm:
///  without a mount that lands where the walk intended,
///  a refusal proves nothing about the attach;
///  the flag read-back is H1 gate 5's recheck on the built code,
///  read from the mount table rather than from the call that asked for it.
#[test]
#[ignore = "mounts a real ZFS clone: the throwaway `mvm` guest of package/zfs/request-to-openzfs-kernel-device/e2e runs it, and any other machine answers SKIP"]
fn attach_arm_the_clone_lands_at_the_held_slot_with_the_enforced_flags() {
    let Some(guest) = in_the_guest("worktree_clone_mount::attach_arm_tests::attach_arm_the_clone_lands_at_the_held_slot_with_the_enforced_flags") else { return };
    let arm: AttachArm = AttachArm::build(&guest, "lands", &[THE_SLOT]);
    let held: HeldCloneSlot = arm.hold(THE_SLOT).expect("the slot is the recorded folder");
    attach_the_clone(guest.clone.as_str(), &held.target, ATTACH_MOUNT_FLAGS).expect("the clone mounts at the held slot");
    let mounted: MountedClone = guard_the_mount(held.canonical_slot.as_path());
    let landed: std::fs::File = std::fs::File::open(held.canonical_slot.as_path()).expect("the slot opens after the attach");
    let facts: MountFacts = facts_at(&landed);
    assert_eq!(facts.mount_point, held.canonical_slot, "the kernel lists the mount at the canonical slot");
    assert_eq!(facts.source, guest.clone, "the mount's source is the clone's dataset");
    assert!(facts.mount_options.iter().any(|option| return option == "nosuid"), "the mount carries nosuid: {:?}", facts.mount_options);
    assert!(facts.mount_options.iter().any(|option| return option == "nodev"), "the mount carries nodev: {:?}", facts.mount_options);
    let attached: MountIdentity = mount_identity_of(&landed).expect("the landed folder's identity");
    assert_eq!(attached.mount_id, mounted.mount_id, "a descriptor opened after the attach is on the mount the attach made");
    let parent_mount: u64 = mount_id_of(&held.parent).expect("the parent's mount");
    // The design's first proof reads the mount at the held target's name beneath the held parent.
    // The held target's own descriptor is not that reader: it keeps naming the mount it was opened on, and the guest
    // measured the parent's mount from it after this very attach.
    let through_the_held_target: u64 = mount_id_of(&held.target).expect("the held target's own mount");
    assert_eq!(through_the_held_target, parent_mount, "a descriptor opened before the attach keeps naming the parent's mount");
    let at_the_held_target: u64 = mount_id_at_the_held_target(&held.parent, THE_SLOT).expect("the mount at the held target's name");
    assert_eq!(at_the_held_target, mounted.mount_id, "a fresh openat2 of the slot beneath the held parent answers the mount the attach made");
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id: parent_mount,
        held_target_mount_id: at_the_held_target,
        attached_mount_id: mounted.mount_id,
        canonical_slot: held.canonical_slot.as_path(),
    };
    let fresh = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: arm.checkout.as_path(), slot: THE_SLOT });
    let outcome: AttachOutcome = post_attach_check(&check, fresh).expect("the mount table reads");
    let AttachOutcome::Attached { mount, .. } = outcome else { panic!("the clone the walk held is not reported at its slot: {outcome:?}") };
    assert_eq!(mount.mount_id, mounted.mount_id, "the check records the mount the attach made");
    assert_eq!(mount.source, guest.clone, "the check records the clone's own source");
    println!(
        "MEASUREMENT attach-arm lands mount={} slot={} options={}",
        mounted.mount_id,
        held.canonical_slot.display(),
        facts.mount_options.join(",")
    );
    // An open descriptor inside the mount keeps it busy, so it is closed before the guard takes the mount down.
    drop(landed);
    drop(mounted);
    let bare: MountFacts = facts_at(&std::fs::File::open(held.canonical_slot.as_path()).expect("the slot opens after the unmount"));
    assert_eq!(bare.filesystem_type, ZFS_FILE_SYSTEM_TYPE, "the slot is on its checkout dataset again");
    assert_ne!(bare.source, guest.clone, "the clone's dataset is no longer what the slot holds: {}", bare.source);
    arm.remove();
}

/// What:
///  Item 7 repeated with the built code:
///  a root-owned setuid executable on the clone gains effective UID 0 on a mount made without the enforced flags,
///  and keeps the account's own effective UID on one made with them.
/// Why:
///   The measured control showed the elevation and its absence,
///  and the flag the module declares is what the arm reads back from the mount table;
///  the unrestricted arm is the positive control,
///  because a setuid program that never elevates would make the restricted arm prove nothing.
#[test]
#[ignore = "mounts a real ZFS clone: the throwaway `mvm` guest of package/zfs/request-to-openzfs-kernel-device/e2e runs it, and any other machine answers SKIP"]
fn attach_arm_the_setuid_probe_keeps_the_account_under_the_enforced_flags() {
    let Some(guest) = in_the_guest("worktree_clone_mount::attach_arm_tests::attach_arm_the_setuid_probe_keeps_the_account_under_the_enforced_flags") else { return };
    let arm: AttachArm = AttachArm::build(&guest, "setuid", &[THE_SLOT]);
    let held: HeldCloneSlot = arm.hold(THE_SLOT).expect("the slot is the recorded folder");
    let probe: String = held.canonical_slot.join("setuid-probe").display().to_string();
    attach_the_clone(guest.clone.as_str(), &held.target, 0).expect("the clone mounts at the held slot without a flag");
    let unrestricted: MountedClone = guard_the_mount(held.canonical_slot.as_path());
    std::fs::copy("/usr/bin/id", &probe).expect("the probe is copied onto the clone");
    std::os::unix::fs::chown(Path::new(&probe), Some(0), Some(0)).expect("the probe is root's");
    std::fs::set_permissions(Path::new(&probe), std::fs::Permissions::from_mode(0o4755)).expect("the probe is setuid and executable");
    let elevated: Output = as_the_account(guest.account.as_str(), &[probe.as_str(), "--user"]);
    assert!(elevated.status.success(), "the probe runs: {}", String::from_utf8_lossy(elevated.stderr.as_slice()));
    assert_eq!(said(&elevated), "0", "positive control: a setuid program on an unrestricted mount of the clone gains effective UID 0, and answered {}", said(&elevated));
    let real: Output = as_the_account(guest.account.as_str(), &[probe.as_str(), "--real", "--user"]);
    assert_eq!(said(&real), guest.uid.to_string(), "the probe's real user is the account");
    drop(unrestricted);
    attach_the_clone(guest.clone.as_str(), &held.target, ATTACH_MOUNT_FLAGS).expect("the clone mounts at the held slot again");
    let restricted: MountedClone = guard_the_mount(held.canonical_slot.as_path());
    let facts: MountFacts = facts_at(&std::fs::File::open(held.canonical_slot.as_path()).expect("the slot opens after the attach"));
    assert!(facts.mount_options.iter().any(|option| return option == "nosuid"), "the restricted mount carries nosuid: {:?}", facts.mount_options);
    assert!(facts.mount_options.iter().any(|option| return option == "nodev"), "the restricted mount carries nodev: {:?}", facts.mount_options);
    let kept: Output = as_the_account(guest.account.as_str(), &[probe.as_str(), "--user"]);
    assert!(kept.status.success(), "the probe runs on the restricted mount too: {}", String::from_utf8_lossy(kept.stderr.as_slice()));
    assert_eq!(said(&kept), guest.uid.to_string(), "the restricted mount keeps the account's effective user, and answered {}", said(&kept));
    println!(
        "MEASUREMENT attach-arm setuid unrestricted-euid=0 restricted-euid={} mount={} options={}",
        said(&kept),
        restricted.mount_id,
        facts.mount_options.join(",")
    );
    std::fs::remove_file(Path::new(&probe)).expect("the probe leaves the clone");
    drop(restricted);
    arm.remove();
}

/// What:
///  Item 2 repeated with the built code:
///  every refusal names the component or the field it found,
///  and a clone mounted at another slot stays exactly where it was through all of them.
/// Why:
///  The account creates links,
/// replaces a slot,
/// changes its mode and its ACLs,
///  and renames a link into the slot rather than creating it there,
/// which is the arm the plan records as unrun;
///  "leaves existing mounts alone" needs a real mount of its own to be seen against,
///  so a second clone is attached before the refusals and its number is read after each one.
#[test]
#[ignore = "mounts a real ZFS clone: the throwaway `mvm` guest of package/zfs/request-to-openzfs-kernel-device/e2e runs it, and any other machine answers SKIP"]
fn attach_arm_each_refusal_names_its_component_and_leaves_the_beside_mount_alone() {
    let Some(guest) = in_the_guest("worktree_clone_mount::attach_arm_tests::attach_arm_each_refusal_names_its_component_and_leaves_the_beside_mount_alone") else { return };
    let mut names: Vec<&str> = vec![THE_BESIDE_SLOT, THE_SLOT];
    names.extend(REFUSAL_SLOTS);
    let arm: AttachArm = AttachArm::build(&guest, "refusals", names.as_slice());
    let beside: HeldCloneSlot = arm.hold(THE_BESIDE_SLOT).expect("the beside slot is recorded");
    attach_the_clone(guest.beside.as_str(), &beside.target, ATTACH_MOUNT_FLAGS).expect("the second clone mounts at its slot");
    let beside_mount: MountedClone = guard_the_mount(arm.slot(THE_BESIDE_SLOT).as_path());
    let mut refused: usize = 0;

    // A link at `.claude`, at `.claude/worktrees` and at a slot, each made by the account.
    let links: [(&str, &str, &str); 3] = [("claude", ".claude", THE_SLOT), ("worktrees", ".claude/worktrees", THE_SLOT), ("slot", ".claude/worktrees/link-slot", "link-slot")];
    for (what, relative, slot) in links {
        let held_path: PathBuf = arm.checkout.join(relative);
        let aside: PathBuf = arm.checkout.join(format!(".{what}-aside"));
        let moved: Output = as_the_account(guest.account.as_str(), &["mv", held_path.display().to_string().as_str(), aside.display().to_string().as_str()]);
        assert!(moved.status.success(), "the account moves {relative} aside: {}", String::from_utf8_lossy(moved.stderr.as_slice()));
        let planted: Output = as_the_account(guest.account.as_str(), &["ln", "--symbolic", aside.display().to_string().as_str(), held_path.display().to_string().as_str()]);
        assert!(planted.status.success(), "the account plants a link at {relative}: {}", String::from_utf8_lossy(planted.stderr.as_slice()));
        let refusal: CloneSlotRefusal = arm.hold(slot).expect_err("a link in the way is refused");
        assert!(matches!(refusal, CloneSlotRefusal::Walk(ref walk) if walk.reason == WalkRefusalReason::LinkInTheWay), "{refusal:?}");
        assert!(refusal.message().contains(relative), "the component is named: {}", refusal.message());
        assert!(refusal.message().contains("never follows a link"), "the cause is named: {}", refusal.message());
        assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &refusal);
        refused += 1;
        let removed: Output = as_the_account(guest.account.as_str(), &["rm", "--force", held_path.display().to_string().as_str()]);
        assert!(removed.status.success(), "the link is removed: {}", String::from_utf8_lossy(removed.stderr.as_slice()));
        let restored: Output = as_the_account(guest.account.as_str(), &["mv", aside.display().to_string().as_str(), held_path.display().to_string().as_str()]);
        assert!(restored.status.success(), "{relative} is restored: {}", String::from_utf8_lossy(restored.stderr.as_slice()));
    }

    // The unrun alternative planting case: a link made elsewhere and renamed into the slot, never created there.
    let decoy_folder: PathBuf = arm.root.join("decoy-folder");
    std::fs::create_dir(&decoy_folder).expect("the decoy folder");
    let planted_path: PathBuf = arm.checkout.join("planted-elsewhere");
    let planted: Output = as_the_account(guest.account.as_str(), &["ln", "--symbolic", decoy_folder.display().to_string().as_str(), planted_path.display().to_string().as_str()]);
    assert!(planted.status.success(), "the account makes the link elsewhere: {}", String::from_utf8_lossy(planted.stderr.as_slice()));
    std::fs::remove_dir(arm.slot("planted-link")).expect("the recorded slot folder makes way for the link");
    let renamed: Output = as_the_account(guest.account.as_str(), &["mv", planted_path.display().to_string().as_str(), arm.slot("planted-link").display().to_string().as_str()]);
    assert!(renamed.status.success(), "the account renames the link into the slot: {}", String::from_utf8_lossy(renamed.stderr.as_slice()));
    let planted_refusal: CloneSlotRefusal = arm.hold("planted-link").expect_err("a link renamed into the slot is refused");
    assert!(matches!(planted_refusal, CloneSlotRefusal::Walk(ref walk) if walk.reason == WalkRefusalReason::LinkInTheWay), "{planted_refusal:?}");
    assert!(planted_refusal.message().contains(".claude/worktrees/planted-link"), "the slot is named: {}", planted_refusal.message());
    assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &planted_refusal);
    refused += 1;
    std::fs::remove_file(arm.slot("planted-link")).expect("the planted link is removed");
    let empty: Output = as_the_account(guest.account.as_str(), &["mkdir", "--parents", arm.slot("planted-link").display().to_string().as_str()]);
    assert!(empty.status.success(), "the slot is a folder again: {}", String::from_utf8_lossy(empty.stderr.as_slice()));

    // A mount at the slot itself: the walk refuses to cross it.
    let boundary: PathBuf = arm.slot("boundary");
    bind_over(decoy_folder.as_path(), boundary.as_path()).expect("the control binds a folder over the slot");
    let boundary_refusal: CloneSlotRefusal = arm.hold("boundary").expect_err("a mount at the slot is not crossed");
    assert!(matches!(boundary_refusal, CloneSlotRefusal::Walk(ref walk) if walk.reason == WalkRefusalReason::BoundaryCrossed), "{boundary_refusal:?}");
    assert!(boundary_refusal.message().contains("boundary"), "the slot is named: {}", boundary_refusal.message());
    assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &boundary_refusal);
    refused += 1;
    unmount_at(boundary.as_path()).expect("the control takes its own bind mount down");

    // Item 2's other arm: the slot already holds the clone's own mount, which is a boundary the walk never crosses.
    let already_target: HeldCloneSlot = arm.hold("already-mounted").expect("the slot is recorded before it is mounted");
    attach_the_clone(guest.beside.as_str(), &already_target.target, ATTACH_MOUNT_FLAGS).expect("the clone mounts at the slot");
    let already_mount: MountedClone = guard_the_mount(arm.slot("already-mounted").as_path());
    drop(already_target);
    let already_refusal: CloneSlotRefusal = arm.hold("already-mounted").expect_err("a mount at the slot is not crossed");
    assert!(matches!(already_refusal, CloneSlotRefusal::Walk(ref walk) if walk.reason == WalkRefusalReason::BoundaryCrossed), "{already_refusal:?}");
    assert!(already_refusal.message().contains("already-mounted"), "the slot is named: {}", already_refusal.message());
    assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &already_refusal);
    refused += 1;
    drop(already_mount);

    // A replacement slot with the same owner and mode: the inode field is what moved.
    let new_inode: PathBuf = arm.slot("new-inode");
    // The replacement is made while the recorded folder still exists, so its inode is one the record cannot hold:
    // a folder made after the removal may be handed the number the removal freed.
    let replacement: PathBuf = arm.checkout.join("new-inode-replacement");
    let prepared: Output = as_the_account(guest.account.as_str(), &["mkdir", replacement.display().to_string().as_str()]);
    assert!(prepared.status.success(), "the account makes the replacement folder: {}", String::from_utf8_lossy(prepared.stderr.as_slice()));
    std::fs::remove_dir(&new_inode).expect("the recorded slot is removed");
    let replaced: Output = as_the_account(guest.account.as_str(), &["mv", replacement.display().to_string().as_str(), new_inode.display().to_string().as_str()]);
    assert!(replaced.status.success(), "the account makes a replacement folder at the slot: {}", String::from_utf8_lossy(replaced.stderr.as_slice()));
    let inode_refusal: CloneSlotRefusal = arm.hold("new-inode").expect_err("a replacement folder is not the recorded one");
    assert!(matches!(inode_refusal, CloneSlotRefusal::SlotIdentity { .. }), "{inode_refusal:?}");
    assert!(inode_refusal.message().contains("inode"), "the field is named: {}", inode_refusal.message());
    assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &inode_refusal);
    refused += 1;

    // The owner, the mode, the access ACL and the default ACL, each changed on its own.
    let wrong_owner: PathBuf = arm.slot("wrong-owner");
    std::os::unix::fs::chown(&wrong_owner, Some(guest.uid + 1), None).expect("another owner stands on the slot");
    let owner_refusal: CloneSlotRefusal = arm.hold("wrong-owner").expect_err("another owner is not the recorded one");
    assert!(owner_refusal.message().contains("owner"), "the field is named: {}", owner_refusal.message());
    assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &owner_refusal);
    refused += 1;
    let widened: PathBuf = arm.slot("widened-mode");
    std::fs::set_permissions(&widened, std::fs::Permissions::from_mode(0o775)).expect("a wider mode stands on the slot");
    let mode_refusal: CloneSlotRefusal = arm.hold("widened-mode").expect_err("a wider mode is not the recorded one");
    assert!(mode_refusal.message().contains("mode"), "the field is named: {}", mode_refusal.message());
    assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &mode_refusal);
    refused += 1;
    // The access arm changes the access ACL; the default arm changes the default one, which setfacl wants both words for.
    let acls: [(&str, &str, &[&str]); 2] = [("access-acl", "access ACL", &["--modify"]), ("default-acl", "default ACL", &["--default", "--modify"])];
    let acl_text: &str = "u:nobody:r-x";
    for (slot, field, acl_options) in acls {
        let path: PathBuf = arm.slot(slot);
        let slot_path: String = path.display().to_string();
        let mut arguments: Vec<&str> = vec!["setfacl"];
        arguments.extend_from_slice(acl_options);
        arguments.push(acl_text);
        arguments.push(slot_path.as_str());
        let set: Output = as_the_account(guest.account.as_str(), arguments.as_slice());
        assert!(set.status.success(), "the account sets the {field} on the slot: {}", String::from_utf8_lossy(set.stderr.as_slice()));
        let acl_refusal: CloneSlotRefusal = arm.hold(slot).expect_err("a changed ACL is not the recorded one");
        assert!(matches!(acl_refusal, CloneSlotRefusal::SlotIdentity { .. }), "{acl_refusal:?}");
        assert!(acl_refusal.message().contains(field), "the field is named: {}", acl_refusal.message());
        assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &acl_refusal);
        refused += 1;
    }

    // A name that is no plain component, which is what a parent escape is refused as.
    let name_refusal: CloneSlotRefusal = arm.hold("..").expect_err("a parent escape is no slot name");
    assert!(matches!(name_refusal, CloneSlotRefusal::Walk(ref walk) if walk.reason == WalkRefusalReason::NotAPlainName), "{name_refusal:?}");
    assert!(name_refusal.message().contains(".."), "the input is named: {}", name_refusal.message());
    assert_the_beside_mount_is_alone(arm.slot(THE_BESIDE_SLOT).as_path(), &beside_mount, &name_refusal);
    refused += 1;

    assert_eq!(refused, REFUSAL_ARMS, "every refusal arm ran");
    println!("MEASUREMENT attach-arm refusals={refused} beside-mount={} untouched=yes", beside_mount.mount_id);
    drop(beside_mount);
    arm.remove();
}

/// What:
///  Item 2's rename trial repeated with the built code:
///  the account renames `.claude` between the walk and the move,
///  the mount lands at the held folder,
///  the fresh canonical lookup refuses,
///  the post-attach check reports a misplaced attach,
///  and the rollback's plan names that exact mount and takes it down through the descriptor.
/// Why:
///  The measured trials proved a held descriptor moves with its ancestor,
///  so only the fresh walk catches this case;
///  a refusal that left the mount where it landed would leave the clone attached outside any slot.
///  The plan's report is produced by the call step 5 ships,
///  so this arm asserts the plan's mount and path and that the descriptor unmount takes exactly that mount down.
#[test]
#[ignore = "mounts a real ZFS clone: the throwaway `mvm` guest of package/zfs/request-to-openzfs-kernel-device/e2e runs it, and any other machine answers SKIP"]
fn attach_arm_a_rename_after_the_walk_is_reported_misplaced_and_rolled_back() {
    let Some(guest) = in_the_guest("worktree_clone_mount::attach_arm_tests::attach_arm_a_rename_after_the_walk_is_reported_misplaced_and_rolled_back") else { return };
    let arm: AttachArm = AttachArm::build(&guest, "rename", &[THE_SLOT]);
    let claude: PathBuf = arm.checkout.join(".claude");
    let trials: [&str; 3] = ["first", "second", "third"];
    for trial in trials {
        let held: HeldCloneSlot = arm.hold(THE_SLOT).expect("the slot is the recorded folder");
        let elsewhere: PathBuf = arm.checkout.join(format!(".claude-{trial}"));
        let renamed: Output = as_the_account(guest.account.as_str(), &["mv", claude.display().to_string().as_str(), elsewhere.display().to_string().as_str()]);
        assert!(renamed.status.success(), "the account renames `.claude` after the walk: {}", String::from_utf8_lossy(renamed.stderr.as_slice()));
        let landed: PathBuf = elsewhere.join("worktrees").join(THE_SLOT);
        attach_the_clone(guest.clone.as_str(), &held.target, ATTACH_MOUNT_FLAGS).expect("the clone mounts at the folder the walk held");
        let mounted: MountedClone = guard_the_mount(landed.as_path());
        let parent_mount: u64 = mount_id_of(&held.parent).expect("the parent's mount");
        let at_the_held_target: u64 = mount_id_at_the_held_target(&held.parent, THE_SLOT).expect("the mount at the held target's name");
        assert_eq!(at_the_held_target, mounted.mount_id, "the rename moved the held folder and not the mount the attach made there");
        let check: AttachPostCheck<'_> = AttachPostCheck {
            held_parent_identity: &held.parent_identity,
            held_parent_mount_id: parent_mount,
            held_target_mount_id: at_the_held_target,
            attached_mount_id: mounted.mount_id,
            canonical_slot: held.canonical_slot.as_path(),
        };
        let fresh = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: arm.checkout.as_path(), slot: THE_SLOT });
        let outcome: AttachOutcome = post_attach_check(&check, fresh).expect("the mount table reads");
        let AttachOutcome::Misplaced(misplaced) = outcome else { panic!("the {trial} rename trial is reported at its slot: {outcome:?}") };
        assert!(misplaced.reason.message().contains(".claude"), "the reason names the folder the walk stopped at: {}", misplaced.reason.message());
        let plan: RollbackPlan = plan_rollback(&misplaced, held.target.try_clone().expect("the target is duplicated"), parent_mount).expect("the mount is one the attach made");
        assert_eq!(plan.attached_mount_id, mounted.mount_id, "the plan carries the mount the attach recorded");
        assert!(plan.mounted_at.ends_with(format!(".claude-{trial}/worktrees/{THE_SLOT}")), "the plan names where the mount landed: {}", plan.mounted_at.display());
        unmount_through(&plan.target).expect("the rollback takes the mount down through the descriptor");
        assert_eq!(mount_id_at(plan.mounted_at.as_path()), Some(parent_mount), "the folder the mount landed on is on its own mount again");
        drop(mounted);
        let restored: Output = as_the_account(guest.account.as_str(), &["mv", elsewhere.display().to_string().as_str(), claude.display().to_string().as_str()]);
        assert!(restored.status.success(), "the account puts `.claude` back: {}", String::from_utf8_lossy(restored.stderr.as_slice()));
        println!("MEASUREMENT attach-arm rename-trial={trial} misplaced=yes rolled-back=mount-{}", plan.attached_mount_id);
    }
    arm.remove();
}

/// What:
///  A mount the attach made somewhere other than the held target's name is refused by the design's first proof,
///  naming the mount at the held target's name and the mount the attach recorded.
/// Why:
///  This is the branch that proof exists for:
///  the number is read at the held target's name beneath the held parent,
///  so a mount that landed beside the slot cannot pass for the attach the walk judged.
///  The arm is also the comparison's control:
///  deleting the comparison in `post_attach_check` leaves this arm answering the fresh walk's cause instead,
///  which is what the integration pass showed the other arms could not see.
#[test]
#[ignore = "mounts a real ZFS clone: the throwaway `mvm` guest of package/zfs/request-to-openzfs-kernel-device/e2e runs it, and any other machine answers SKIP"]
fn attach_arm_a_mount_beside_the_held_slot_is_refused_by_the_held_target_proof() {
    let Some(guest) = in_the_guest("worktree_clone_mount::attach_arm_tests::attach_arm_a_mount_beside_the_held_slot_is_refused_by_the_held_target_proof") else { return };
    let arm: AttachArm = AttachArm::build(&guest, "beside-proof", &[THE_SLOT, THE_BESIDE_SLOT]);
    let held: HeldCloneSlot = arm.hold(THE_SLOT).expect("the slot is the recorded folder");
    let beside: HeldCloneSlot = arm.hold(THE_BESIDE_SLOT).expect("the beside slot is recorded");
    attach_the_clone(guest.beside.as_str(), &beside.target, ATTACH_MOUNT_FLAGS).expect("the clone mounts at the beside slot");
    let mounted: MountedClone = guard_the_mount(arm.slot(THE_BESIDE_SLOT).as_path());
    let parent_mount: u64 = mount_id_of(&held.parent).expect("the parent's mount");
    let at_the_held_target: u64 = mount_id_at_the_held_target(&held.parent, THE_SLOT).expect("the mount at the held target's name");
    assert_eq!(at_the_held_target, parent_mount, "the attach did not act through the held target's name");
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id: parent_mount,
        held_target_mount_id: at_the_held_target,
        attached_mount_id: mounted.mount_id,
        canonical_slot: held.canonical_slot.as_path(),
    };
    let fresh = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: arm.checkout.as_path(), slot: THE_SLOT });
    let outcome: AttachOutcome = post_attach_check(&check, fresh).expect("the mount table reads");
    let AttachOutcome::Misplaced(misplaced) = outcome else { panic!("a mount beside the held slot is reported at its slot: {outcome:?}") };
    assert!(
        matches!(misplaced.reason, MisplacedReason::HeldTargetElsewhere { held: found, attached } if found == parent_mount && attached == mounted.mount_id),
        "the held target's proof refuses a mount made beside it: {misplaced:?}"
    );
    println!(
        "MEASUREMENT attach-arm beside-proof held-target-mount={at_the_held_target} attached-mount={} reason=HeldTargetElsewhere",
        mounted.mount_id
    );
    drop(mounted);
    arm.remove();
}

/// What:
///  H1 gate 1's decoy control:
///  the check-then-path behaviour reproduces a mount outside the slot the check passed,
///  and this harness sees the attachment there before any no-mount assertion is trusted.
/// Why:
///  A path mount reached a decoy in the research,
///  which did not test a substitution after the check;
///  the control proves the observation code reports a mount that is not at the slot,
///  so a later "no mount at the slot" answer counts as evidence rather than as a blind read.
///  The decoy is a slot of this guest's own fixture and nothing outside it is touched.
#[test]
#[ignore = "mounts a real ZFS clone: the throwaway `mvm` guest of package/zfs/request-to-openzfs-kernel-device/e2e runs it, and any other machine answers SKIP"]
fn attach_arm_a_path_mount_onto_a_decoy_is_seen_away_from_the_checked_slot() {
    let Some(guest) = in_the_guest("worktree_clone_mount::attach_arm_tests::attach_arm_a_path_mount_onto_a_decoy_is_seen_away_from_the_checked_slot") else { return };
    let arm: AttachArm = AttachArm::build(&guest, "decoy", &[THE_SLOT, "decoy"]);
    // The check passes on the slot, and the mount is then made by a path onto another slot.
    let checked: HeldCloneSlot = arm.hold(THE_SLOT).expect("the checked slot is the recorded folder");
    let checked_mount: u64 = mount_id_of(&checked.target).expect("the checked slot's mount");
    let decoy: PathBuf = arm.slot("decoy");
    attach_the_clone_by_path(guest.beside.as_str(), decoy.as_path(), ATTACH_MOUNT_FLAGS).expect("the clone mounts at the decoy's path");
    let decoy_mount: u64 = mount_id_at(decoy.as_path()).expect("the decoy's mount");
    let decoy_guard: MountedClone = guard_the_mount(decoy.as_path());
    let decoy_file: std::fs::File = std::fs::File::open(&decoy).expect("the decoy opens");
    let seen: MountFacts = facts_at(&decoy_file);
    assert_ne!(decoy_mount, checked_mount, "the harness reads another mount at the decoy than at the checked slot");
    assert_eq!(seen.mount_point, decoy, "the harness names the decoy as where the attachment landed");
    assert_eq!(seen.source, guest.beside, "the harness names the dataset that is attached there");
    let again = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: arm.checkout.as_path(), slot: THE_SLOT });
    let at_the_slot: FreshSlotLookup = again.expect("the checked slot's folder is still there");
    assert_eq!(at_the_slot.mount_id, checked_mount, "the checked slot holds no new mount, so a no-mount answer there counts");
    println!(
        "MEASUREMENT attach-arm decoy mount={decoy_mount} at={} checked-slot-mount={checked_mount}",
        seen.mount_point.display()
    );
    drop(decoy_file);
    unmount_at(decoy.as_path()).expect("the control takes its own path mount down");
    drop(decoy_guard);
    arm.remove();
}

//endregion The arms
