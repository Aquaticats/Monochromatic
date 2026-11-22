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

use super::{ATTACH_FSMOUNT_ATTRIBUTES, AttachCall, attach_with, unmount_through_descriptor};
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
    /// A fresh folder under this process's own temporary root.
    fn new() -> Self {
        let path: std::path::PathBuf = std::env::temp_dir().join(format!("attach-the-clone-{}", std::process::id()));
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(path.as_path());
        std::fs::create_dir_all(path.as_path()).expect("the fixture folder is made");
        return Fixture { path };
    }

    /// The folder,
    /// opened as the descriptor an attach or an unmount acts on.
    fn open(&self) -> std::fs::File {
        return std::fs::File::open(self.path.as_path()).expect("the fixture folder opens");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(self.path.as_path());
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
    let fixture: Fixture = Fixture::new();
    let target: std::fs::File = fixture.open();
    let refusal = attach_with(ABSENT_FILE_SYSTEM, "pool/clone", &target).expect_err("a type no module registers cannot be opened");
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
    let fixture: Fixture = Fixture::new();
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
