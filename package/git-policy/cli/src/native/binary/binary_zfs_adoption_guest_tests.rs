//! What:
//!  The end-to-end arm of the administrator's parent adoption,
//! in the throwaway guest:
//!  the recorded parent folder is replaced,
//!  the handler's `adopt-worktree-parent` mode records the replacement,
//!  and each refusal leaves the record as it was.
//! Why:
//!  The mode writes a store under `/var/lib` and opens a checkout as root,
//!  so no control on a developer's machine may run it:
//!  this arm runs only where `CLI_GIT_ZFS_ADOPTION_GUEST` names a disposable guest
//!  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`,
//!  on OpenZFS 2.4.4 in an `mvm` Ubuntu guest),
//!  and it prints `SKIPPED` everywhere else,
//! so the gate says what it did not run
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 5,
//!  "Acceptance tests").
//!  The arm drives the built executable,
//! not the module,
//!  because the mode and its grammar are what the install ships.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! run(handler, ['adopt-worktree-parent', '--checkout', checkout, '--device', String(dev), '--inode', String(ino)]);
//! expect(storeParent()).toEqual(liveIdentity());
//! ```

use git_policy_cli::worktree_clone_mount::{
    CloneFolderIdentity, SLOT_IDENTITY_STORE_PATH, STORE_HEADER, SlotIdentityStore, append_slot_identity_under_root_control, record_parent_identity_under_root_control,
};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The variable the guest recipe sets.
///
/// It names the guest this arm may run in,
/// so a start anywhere else prints that it skipped instead of writing
/// `/var/lib` on a machine somebody uses.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_ADOPTION_GUEST";

/// The status the handler answers a refusal with,
/// which the unit controls also hold.
const EXIT_REFUSED: i32 = 2;

/// The folder this arm builds its checkout under,
/// inside the guest.
///
/// The guest recipe runs the arm as root,
/// so root's own home is the one folder every walk below holds without a
/// world-writable parent,
/// and no path on the arm's own machine is ever named here.
const GUEST_SCRATCH_ROOT: &str = "/root";

/// The mode's exit status for one invocation,
/// its whole output,
/// and the store's bytes before and after it.
struct Arm {
    /// The status the program answered.
    status: Option<i32>,
    /// Its standard output and standard error,
    /// joined.
    said: String,
    /// The store's bytes after the run,
    /// which a refusal must leave as they were.
    store_after: Vec<u8>,
}

/// The copy on disk of one folder's identity,
/// which the store records and the mode records again.
fn identity_of(path: &Path) -> CloneFolderIdentity {
    let folder: std::fs::File = std::fs::File::open(path).expect("the folder opens");
    return CloneFolderIdentity::capture(&folder).expect("the folder's identity reads");
}

/// The store's bytes,
/// for the arms that prove a refusal recorded nothing.
fn store_bytes() -> Vec<u8> {
    return std::fs::read(SLOT_IDENTITY_STORE_PATH).unwrap_or_default();
}

/// The recorded parent identity,
/// read the way the program reads it.
fn recorded_parent() -> CloneFolderIdentity {
    let store: SlotIdentityStore = SlotIdentityStore::read_under_root_control(Path::new(SLOT_IDENTITY_STORE_PATH)).expect("the store reads under root control");
    return store.parent().expect("the store records the parent the arm wrote").clone();
}

/// Run the built handler's adoption mode and read the store afterwards.
fn adopt(checkout: &Path, device: u64, inode: u64) -> Arm {
    let binary: &str = env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets");
    let output: Output = Command::new(binary)
        .arg("adopt-worktree-parent")
        .arg("--checkout")
        .arg(checkout)
        .arg("--device")
        .arg(device.to_string())
        .arg("--inode")
        .arg(inode.to_string())
        .output()
        .expect("the handler starts");
    return Arm {
        status: output.status.code(),
        said: format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())),
        store_after: store_bytes(),
    };
}

/// What:
///  The checkout this arm creates,
///  and the numbers `stat` prints for its parent folder.
/// Why:
///   The administrator identifies the replacement independently,
///  so the arm reads the device and inode from the file system
///  and hands the program only those numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkout = mkdtemp(); mkdir(join(checkout, '.claude/worktrees'));
/// const { dev, ino } = statSync(join(checkout, '.claude/worktrees'));
/// ```
struct Fixture {
    /// The checkout the parent hangs below.
    checkout: PathBuf,
}

impl Fixture {
    /// The `.claude/worktrees` folder of the checkout.
    fn parent(&self) -> PathBuf {
        return self.checkout.join(".claude").join("worktrees");
    }
}

/// Create one checkout with a `.claude/worktrees` folder the arm can replace,
/// under a folder of the guest's own.
fn create_fixture() -> Fixture {
    let root: PathBuf = Path::new(GUEST_SCRATCH_ROOT).join(format!("mount-handler-adoption-{}", std::process::id()));
    let _removed: std::io::Result<()> = std::fs::remove_dir_all(root.as_path());
    let checkout: PathBuf = root.join("checkout");
    let parent: PathBuf = checkout.join(".claude").join("worktrees");
    std::fs::create_dir_all(parent.as_path()).expect("the checkout and its worktrees folder are made");
    std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the folder holds the mode the policy records");
    return Fixture { checkout };
}

/// Replace `.claude/worktrees` with a fresh folder,
/// which is what a checkout restore or a controlled recreation leaves,
/// and read its device and inode.
fn replace_parent(fixture: &Fixture) -> (u64, u64) {
    let parent: PathBuf = fixture.parent();
    // The recorded folder is renamed aside rather than removed, so its inode stays allocated and the new folder
    // cannot reuse it: a remove and a create in one step got the same inode on the guest's file system, which made
    // the arm prove nothing.
    let aside: PathBuf = parent.with_file_name("worktrees-before-the-adoption");
    std::fs::rename(parent.as_path(), aside.as_path()).expect("the recorded folder is moved aside");
    std::fs::create_dir(parent.as_path()).expect("the replacement folder is made");
    std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the replacement keeps the recorded mode");
    let metadata: std::fs::Metadata = std::fs::metadata(parent.as_path()).expect("the replacement's facts read");
    return (metadata.dev(), metadata.ino());
}

/// Make the store the setup run would have left:
/// a root-owned file in a root-owned folder,
/// recording the parent folder as it is now.
fn write_store(fixture: &Fixture) {
    let folder: &Path = Path::new(SLOT_IDENTITY_STORE_PATH).parent().expect("the store has a folder");
    std::fs::create_dir_all(folder).expect("the store's folder is made");
    std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o700)).expect("the store's folder is root's");
    let _removed: std::io::Result<()> = std::fs::remove_file(SLOT_IDENTITY_STORE_PATH);
    // The setup run creates the store with its header and then records the parent; a writer replaces a store that is
    // there, so the header is written first and the record is the writer's own.
    std::fs::write(SLOT_IDENTITY_STORE_PATH, format!("{STORE_HEADER}\n")).expect("the store starts with its header");
    std::fs::set_permissions(SLOT_IDENTITY_STORE_PATH, std::fs::Permissions::from_mode(0o644)).expect("the store is root's and readable");
    record_parent_identity_under_root_control(Path::new(SLOT_IDENTITY_STORE_PATH), &identity_of(fixture.parent().as_path()))
        .expect("the setup run's parent record is written");
}

/// The arm,
/// in one test:
/// the replacement is recorded,
/// and each refusal names its cause and leaves the record as it was.
#[test]
fn the_adoption_mode_records_a_replacement_and_refuses_each_mismatch() {
    let Some(guest) = std::env::var_os(GUEST_VARIABLE) else {
        eprintln!("SKIPPED: the adoption arm writes /var/lib and opens a checkout as root, so it runs only in a disposable guest; set {GUEST_VARIABLE} in one");
        return;
    };
    println!("ADOPTION guest={}", guest.to_string_lossy());
    let fixture: Fixture = create_fixture();
    write_store(&fixture);
    let recorded_before: CloneFolderIdentity = recorded_parent();
    let (device, inode) = replace_parent(&fixture);
    let live: CloneFolderIdentity = identity_of(fixture.parent().as_path());
    assert_ne!(recorded_before.inode, live.inode, "the replacement is another folder, so the arm proves a write");

    let adoption: Arm = adopt(fixture.checkout.as_path(), device, inode);
    assert_eq!(adoption.status, Some(0), "the identified replacement is recorded: {}", adoption.said);
    assert!(adoption.said.contains("as the adopted `.claude/worktrees` parent folder"), "{}", adoption.said);
    let recorded_after: CloneFolderIdentity = recorded_parent();
    assert_eq!(recorded_after.differences(&live), Vec::new(), "the store records the replacement's own identity");
    println!("ADOPTION recorded device={device} inode={inode}");

    let after_success: Vec<u8> = adoption.store_after;
    let wrong_device: Arm = adopt(fixture.checkout.as_path(), device + 1, inode);
    assert_eq!(wrong_device.status, Some(EXIT_REFUSED), "another device is refused: {}", wrong_device.said);
    assert!(wrong_device.said.contains("not the one the administrator identified"), "{}", wrong_device.said);
    assert_eq!(wrong_device.store_after, after_success, "a refused adoption records nothing");
    println!("ADOPTION refused=wrong-device");

    let elsewhere: PathBuf = fixture.checkout.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("the link's target is made");
    std::fs::remove_dir(fixture.parent().as_path()).expect("the parent goes");
    std::os::unix::fs::symlink(elsewhere.as_path(), fixture.parent()).expect("the parent is replaced by a link");
    let linked: Arm = adopt(fixture.checkout.as_path(), device, inode);
    assert_eq!(linked.status, Some(EXIT_REFUSED), "a link at the parent is refused: {}", linked.said);
    assert!(linked.said.to_lowercase().contains("link"), "the refusal names the link: {}", linked.said);
    assert_eq!(linked.store_after, after_success, "a refused adoption records nothing");
    println!("ADOPTION refused=link");
    std::fs::remove_file(fixture.parent().as_path()).expect("the link goes");
    std::fs::create_dir(fixture.parent().as_path()).expect("a plain folder comes back");
    std::fs::set_permissions(fixture.parent().as_path(), std::fs::Permissions::from_mode(0o777)).expect("the folder is widened");
    let widened: (u64, u64) = {
        let metadata: std::fs::Metadata = std::fs::metadata(fixture.parent().as_path()).expect("the widened folder's facts read");
        (metadata.dev(), metadata.ino())
    };
    let widened_arm: Arm = adopt(fixture.checkout.as_path(), widened.0, widened.1);
    assert_eq!(widened_arm.status, Some(EXIT_REFUSED), "a folder wider than the recorded one is refused: {}", widened_arm.said);
    assert!(widened_arm.said.contains("does not hold the shape"), "{}", widened_arm.said);
    assert_eq!(widened_arm.store_after, after_success, "a refused adoption records nothing");
    println!("ADOPTION refused=mode-widened");

    std::fs::set_permissions(fixture.parent().as_path(), std::fs::Permissions::from_mode(0o755)).expect("the recorded mode comes back");
    let survivor: PathBuf = fixture.parent().join("survivor");
    std::fs::create_dir(survivor.as_path()).expect("a surviving slot folder is made");
    append_slot_identity_under_root_control(Path::new(SLOT_IDENTITY_STORE_PATH), "survivor", &identity_of(survivor.as_path())).expect("the slot record is written");
    let current: (u64, u64) = {
        let metadata: std::fs::Metadata = std::fs::metadata(fixture.parent().as_path()).expect("the parent's facts read");
        (metadata.dev(), metadata.ino())
    };
    // The parent folder the arm made after the link arm is recorded first, so the slot arm refuses at the slot
    // rather than at the identity: every earlier check passes, which is what makes this arm report the slot.
    let re_recording: Arm = adopt(fixture.checkout.as_path(), current.0, current.1);
    assert_eq!(re_recording.status, Some(0), "the folder the arm stands in is recorded with its agreeing slot: {}", re_recording.said);
    std::fs::remove_dir(survivor.as_path()).expect("the slot folder is replaced");
    std::fs::create_dir(survivor.as_path()).expect("another folder stands at the slot");
    let before_slot_arm: Vec<u8> = store_bytes();
    let slot_arm: Arm = adopt(fixture.checkout.as_path(), current.0, current.1);
    assert_eq!(slot_arm.status, Some(EXIT_REFUSED), "a surviving slot that is not its record stops the adoption: {}", slot_arm.said);
    assert!(slot_arm.said.contains("survivor"), "the refusal names the slot: {}", slot_arm.said);
    assert_eq!(slot_arm.store_after, before_slot_arm, "a refused adoption records nothing");
    println!("ADOPTION refused=slot-differs");

    let scratch: &Path = fixture.checkout.parent().unwrap_or(Path::new("/nonexistent"));
    let _cleanup: std::io::Result<()> = std::fs::remove_dir_all(scratch);
    let _store_cleanup: std::io::Result<()> = std::fs::remove_file(Path::new(SLOT_IDENTITY_STORE_PATH));
    println!("ADOPTION done");
}
