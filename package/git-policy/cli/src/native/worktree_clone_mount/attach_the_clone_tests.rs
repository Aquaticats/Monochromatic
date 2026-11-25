//! What:
//!  Controls of the attach's refusals and its flags on this host.
//! Why:
//!  The real `fsmount` of a ZFS dataset and the move onto a real slot need the throwaway guest,
//!  so what this host can prove is the refusal path a machine without the module takes,
//!  the two spellings of the enforced flags,
//!  and the words a report names the failing call with.
//!  The positive arm,
//!  a real clone landing at the held slot with both flags in `mountinfo`,
//!  is step 4's guest arm and the sweep's own arm
//!  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(attachWith('no-such-filesystem', 'pool/clone', folder).call).toBe('fsopen');
//! ```

use super::{ATTACH_FSMOUNT_ATTRIBUTES, AttachCall, Placement, attach_with, unmount_through_descriptor};
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::fs_mount_options::parse_mount_table;
use crate::worktree_clone_mount::ATTACH_MOUNT_FLAGS;

/// A filesystem type this kernel does not have,
/// which is what a machine with no loaded `zfs` module answers to `fsopen`.
const ABSENT_FILE_SYSTEM: &str = "monochromatic-no-such-filesystem";

/// The two errnos `fsopen` can answer here:
/// the account's lack of mount authority,
/// and a type no module registers.
const FSOPEN_ERRNOS: [i32; 2] = [libc::EPERM, libc::ENODEV];

/// One fixture folder of this control's own,
/// removed however the control ends.
struct Fixture {
    /// The folder's path.
    path: std::path::PathBuf,
}

impl Fixture {
    /// A fresh folder of this control's own,
    /// named so two controls of one process never share it.
    fn new(name: &str) -> Self {
        return Fixture { path: crate::test_support::fixture(format!("attach-the-clone-{name}").as_str()) };
    }

    /// The folder,
    /// opened as the descriptor an attach or an unmount acts on.
    fn open(&self) -> std::fs::File {
        return std::fs::File::open(self.path.as_path()).expect("the fixture folder opens");
    }
}

/// What:
///  A filesystem type this kernel does not have refuses at `fsopen`,
///  with that call's own errno,
///  and nothing is left mounted.
/// Why:
///  This is the arm a machine with no loaded module takes,
///  and it is the one attach refusal this host can make;
///  the message has to send a person to the call,
///  not to a summary.
#[test]
fn a_filesystem_type_the_kernel_does_not_have_is_refused_at_fsopen() {
    let fixture: Fixture = Fixture::new("fsopen");
    let target: std::fs::File = fixture.open();
    let refusal = attach_with(ABSENT_FILE_SYSTEM, "pool/clone", &target, ATTACH_FSMOUNT_ATTRIBUTES, &mount_id_of).expect_err("a type no module registers cannot be opened");
    assert_eq!(refusal.call, AttachCall::FileSystemContext, "the refusal names fsopen: {refusal:?}");
    assert_eq!(refusal.call.word(), "fsopen");
    assert!(FSOPEN_ERRNOS.contains(&refusal.errno), "the errno is the call's own (EPERM without mount authority, ENODEV for a type no module registers): {refusal:?}");
    assert_eq!(refusal.source, "pool/clone");
    let said: String = refusal.message();
    assert!(said.contains("fsopen"), "the message names the call: {said}");
    assert!(said.contains("pool/clone"), "the message names the dataset: {said}");
    assert!(said.contains(&format!("errno {}", refusal.errno)), "the message carries the errno: {said}");
    for later in ["fsconfig", "fsmount", "move_mount"] {
        assert!(!said.contains(later), "the message names no later call than the one that refused, and it names {later}: {said}");
    }
    assert!(said.contains("Nothing was mounted"), "a refused fsopen mounted nothing, and the message says so: {said}");
}

/// What:
///  A mount-number read that fails after the move takes the placed mount down
///  and says it did.
/// Why:
///  A mount is placed at the slot before the number is read,
///  so a failure there that left the mount standing would report "nothing was
///  mounted" over a mount nobody named;
///  this control attaches a tmpfs through the same calls in a namespace of its
///  own and fails the read on purpose.
#[test]
fn a_mount_number_read_that_fails_after_the_move_takes_the_placed_mount_down() {
    if !crate::testing::root_namespace_test_support::in_a_root_namespace(
        "CLI_GIT_ATTACH_TAKE_DOWN_ROOT",
        "worktree_clone_mount::attach_the_clone::tests::a_mount_number_read_that_fails_after_the_move_takes_the_placed_mount_down",
    ) {
        return;
    }
    let root: std::path::PathBuf = crate::test_support::fixture("attach-take-down");
    let target_path: std::path::PathBuf = root.join("target");
    std::fs::create_dir_all(target_path.as_path()).expect("the target folder is made");
    let target: std::fs::File = std::fs::File::open(target_path.as_path()).expect("the target opens");
    let mount_id_at = |path: &std::path::Path| -> u64 {
        let opened: std::fs::File = std::fs::File::open(path).expect("the folder opens");
        return mount_id_of(&opened).expect("the folder's mount number reads");
    };
    let parent_before: u64 = mount_id_at(root.as_path());
    assert_eq!(mount_id_at(target_path.as_path()), parent_before, "the target starts on its parent's own mount");
    let refusal = attach_with(
        "tmpfs",
        "monochromatic-attach-take-down-control",
        &target,
        ATTACH_FSMOUNT_ATTRIBUTES,
        &|_held: &std::fs::File| -> std::io::Result<u64> { return Err(std::io::Error::from_raw_os_error(libc::EIO)); },
    )
    .expect_err("the control fails the mount-number read on purpose");
    assert_eq!(refusal.call, AttachCall::MountNumber, "the refusal names the read that failed: {refusal:?}");
    assert_eq!(refusal.placement, Placement::PlacedThenTakenDown, "the placed mount was taken down again: {refusal:?}");
    assert_eq!(mount_id_at(target_path.as_path()), parent_before, "the target is back on its parent's own mount");
    let table = parse_mount_table(std::fs::read("/proc/self/mountinfo").expect("this namespace's mount table reads").as_slice());
    let canonical: std::path::PathBuf = std::fs::canonicalize(target_path.as_path()).expect("the target resolves");
    assert!(!table.iter().any(|facts| return facts.mount_point == canonical), "no mount stands at the target after the take-down");
    let said: String = refusal.message();
    assert!(said.contains("took it down again"), "the message says the mount was taken down: {said}");
    assert!(!said.contains("could not be taken down again"), "the message does not claim the mount was retained: {said}");
}

/// What:
///  The flags the mount descriptor is made with are the two the route reports,
///  bit for bit.
/// Why:
///  `ATTACH_MOUNT_FLAGS` is the `MS_` spelling a report and the guest's read-back use,
///  and `fsmount` takes the `MOUNT_ATTR_` spelling;
///  the two must not drift,
///  because only the read-back proves the mount really carries them.
#[test]
fn the_two_flag_spellings_are_the_same_two_bits() {
    assert_eq!(ATTACH_FSMOUNT_ATTRIBUTES, u64::from(libc::MOUNT_ATTR_NOSUID | libc::MOUNT_ATTR_NODEV));
    assert_eq!(ATTACH_FSMOUNT_ATTRIBUTES, u64::try_from(ATTACH_MOUNT_FLAGS).expect("the report's flags fit a word"));
    assert_eq!(ATTACH_FSMOUNT_ATTRIBUTES & u64::from(libc::MOUNT_ATTR_NOSUID), u64::from(libc::MOUNT_ATTR_NOSUID));
    assert_eq!(ATTACH_FSMOUNT_ATTRIBUTES & u64::from(libc::MOUNT_ATTR_NODEV), u64::from(libc::MOUNT_ATTR_NODEV));
}

/// What:
///  A folder that is no mount is refused by the descriptor-bound unmount,
///  and the refusal names that call.
/// Why:
///  The rollback and step 9 both act on a descriptor,
///  and a call that took a path would report a path's errno instead;
///  the two errnos this host can answer are the account's lack of mount authority
///  and the kernel's answer that nothing is mounted there.
#[test]
fn a_folder_that_is_no_mount_is_refused_by_the_descriptor_unmount() {
    let fixture: Fixture = Fixture::new("unmount");
    let target: std::fs::File = fixture.open();
    let refusal = unmount_through_descriptor(&target).expect_err("a plain folder is no mount");
    assert_eq!(refusal.call, AttachCall::Unmount);
    assert_eq!(refusal.call.word(), "unmount");
    assert!(
        refusal.errno == libc::EPERM || refusal.errno == libc::EINVAL,
        "the errno is the call's own (EPERM without mount authority, EINVAL for a folder that is no mount): {refusal:?}"
    );
    let said: String = refusal.message();
    assert!(said.contains("umount2 through the held descriptor"), "the message names the call: {said}");
    assert!(said.contains(fixture.path.display().to_string().as_str()), "the message names the folder the descriptor holds: {said}");
}

/// What:
///  Every step's word is the one a receipt may carry,
///  and no two steps share one.
/// Why:
///  A receipt names the failed step,
///  and a word two steps share would send an operator to the wrong call.
#[test]
fn every_step_has_its_own_word() {
    let steps: [AttachCall; 7] = [
        AttachCall::FileSystemContext,
        AttachCall::SourceOption,
        AttachCall::Create,
        AttachCall::MountDescriptor,
        AttachCall::Place,
        AttachCall::MountNumber,
        AttachCall::Unmount,
    ];
    let mut words: Vec<&'static str> = Vec::new();
    for step in steps {
        assert!(!words.contains(&step.word()), "{} is carried by two steps", step.word());
        assert!(step.word().len() <= crate::worktree_clone_mount_protocol::clone_mount_receipt::MAXIMUM_WORD_BYTES, "{} fits a receipt's word", step.word());
        words.push(step.word());
    }
    assert_eq!(words.len(), steps.len());
}
