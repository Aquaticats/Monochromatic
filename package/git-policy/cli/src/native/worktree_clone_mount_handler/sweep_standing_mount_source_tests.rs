//! What:
//!  Controls of the call that names one mount's source by its unique number,
//!  and of the answer a kernel without it gives.
//! Why:
//!  The rename gap this closes is a dataset swap that raises no mount event,
//!  so the control mounts one tmpfs inside a namespace of its own and reads
//!  the source the kernel reports for it,
//!  which the kernel prints from the same `show_devname` the mount table uses.
//!  A kernel before 6.13 reports no source at all and the sweep then reads the
//!  live table,
//!  and a container whose seccomp profile refuses the call takes that same
//!  fallback;
//!  the run says which of the two it is by setting
//!  `CLI_GIT_STATMOUNT_ALLOWED_IN_CONTAINERS`,
//!  so a gate that passes a profile and a podman that never applied it is red
//!  rather than quiet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(kernelMountSource().sourceOf(uniqueNumberOf(target))).toEqual({ kind: 'source', source: 'tmpfs' });
//! ```

use super::{KernelMountSource, MOUNT_NUMBER_REQUEST_BYTES, MountNumberRequest, MountSourceAnswer, MountSourceByNumber, STATMOUNT_HEADER_BYTES};
use crate::worktree_clone_mount_handler::standing_mount_source_call_refusal::{self, the_call_must_answer};
use std::os::unix::ffi::OsStrExt as _;

/// The variable the inner half of the control sets.
const INSIDE: &str = "CLI_GIT_STANDING_MOUNT_SOURCE_ROOT";

/// The source the control that reads a mount's source through the release call
/// mounts its tmpfs with.
const THE_SOURCE_THE_RELEASE_CONTROL_MOUNTS: &str = "a-source-this-control-names";

/// The source the control that reads the answer's layout mounts its tmpfs
/// with.
const THE_SOURCE_THE_LAYOUT_CONTROL_MOUNTS: &str = "offset-source";

/// The most bytes one source string may take here,
/// which is the room the release call's own buffer gives it.
const ROOM_FOR_THE_ANSWER_BYTES: usize = 1_024;

/// One tmpfs mount a control makes,
/// taken down however the control ends.
struct MountedFolder {
    /// The mount point.
    path: std::path::PathBuf,
}

impl Drop for MountedFolder {
    fn drop(&mut self) {
        let target: std::ffi::CString = std::ffi::CString::new(self.path.as_os_str().as_bytes()).expect("no NUL in the path");
        // SAFETY: `target` is a NUL-terminated path that outlives the call, and no flag is passed.
        let _taken: libc::c_int = unsafe { libc::umount2(target.as_ptr(), 0) };
    }
}

/// Mount one tmpfs named `source` at `path`.
fn mount_tmpfs(source: &str, path: &std::path::Path) {
    let source_name: std::ffi::CString = std::ffi::CString::new(source).expect("no NUL in the source");
    let target: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in the path");
    let kind: std::ffi::CString = std::ffi::CString::new("tmpfs").expect("no NUL in the type");
    // SAFETY: both strings outlive the call, the type is a NUL-terminated name, and no data pointer is read.
    let mounted: libc::c_int = unsafe { libc::mount(source_name.as_ptr(), target.as_ptr(), kind.as_ptr(), 0, std::ptr::null()) };
    assert_eq!(mounted, 0, "the control's tmpfs mounts: {}", std::io::Error::last_os_error());
}

/// The unique mount number of the mount a folder is on.
fn unique_number_of(path: &std::path::Path) -> u64 {
    let opened: std::fs::File = std::fs::File::open(path).expect("the folder opens");
    return crate::fs_beneath_mount_facts::unique_mount_number_of(&opened).expect("this kernel reports a unique mount number");
}

/// What:
///  The kernel's own answer to the request this build sends about one mount,
///  or the error the call answered with.
/// Why:
///  A control that needs the source has to tell three cases apart:
///  the call was refused or is absent,
///  the call answered on a kernel that reports no source (6.8 to 6.12),
///  and the call answered with the layout this build reads.
///  Asking the call itself is the evidence for that,
///  so no control infers it from a kernel release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theAnswerFor(unique: number): Buffer | Error
/// ```
fn the_answer_for(unique: u64) -> Result<Vec<u8>, std::io::Error> {
    let request: MountNumberRequest = MountNumberRequest {
        size: MOUNT_NUMBER_REQUEST_BYTES,
        flags_or_descriptor: 0,
        mount_number: unique,
        fields: super::STATMOUNT_MOUNT_ROOT | super::STATMOUNT_SOURCE,
    };
    let mut buffer: Vec<u8> = vec![0_u8; STATMOUNT_HEADER_BYTES + ROOM_FOR_THE_ANSWER_BYTES];
    let number: libc::c_long = super::STATMOUNT_NUMBER.expect("this architecture declares the call");
    // SAFETY: the request and the buffer are this call's own and outlive it, the buffer is writable for its length, and
    // the flags argument is the zero the call takes.
    let answered: libc::c_long = unsafe { libc::syscall(number, &request as *const MountNumberRequest, buffer.as_mut_ptr(), buffer.len(), 0_u32) };
    if answered < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(buffer);
}

/// What:
///  What a control here does when the call answered nothing.
/// Why:
///  The run's own statement decides it:
///  a run that states its containers allow the call must see it answer,
///  and every other run prints one line naming the control that did not run
///  instead of passing as if it had measured the kernel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCallAnsweredNothing(control: string, error: Error): void; // throws where the run states the allowance
/// ```
fn the_call_answered_nothing(control: &str, error: &std::io::Error) {
    if the_call_must_answer(control, format!("the call answered {}", error).as_str()) {
        panic!(
            "this run states that its containers allow statmount(2) through their seccomp profile \
             ({}), and the call answered {error}; podman may not have applied the profile the gate passes",
            standing_mount_source_call_refusal::ALLOWED_IN_CONTAINERS_VARIABLE
        );
    }
}

/// What:
///  The source the kernel reports for one mount by its unique number is the
///  source that mount was made with.
/// Why:
///  This is the call the rename gap is closed with,
///  so the answer has to be read here before a sweep can rest on it.
#[test]
fn the_kernel_names_the_source_of_one_mount_by_its_unique_number() {
    let control: &str = "worktree_clone_mount_handler::sweep_standing_mount_source::tests::the_kernel_names_the_source_of_one_mount_by_its_unique_number";
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(INSIDE, control) {
        return;
    }
    let root: std::path::PathBuf = crate::test_support::fixture("standing-mount-source");
    let folder: std::path::PathBuf = root.join("slot");
    std::fs::create_dir_all(folder.as_path()).expect("the folder is made");
    mount_tmpfs(THE_SOURCE_THE_RELEASE_CONTROL_MOUNTS, folder.as_path());
    let _guard: MountedFolder = MountedFolder { path: folder.clone() };
    let unique: u64 = unique_number_of(folder.as_path());
    let answer: MountSourceAnswer = KernelMountSource.source_of(Some(unique));
    let why: String = match answer {
        MountSourceAnswer::Source(source) => {
            assert_eq!(source, THE_SOURCE_THE_RELEASE_CONTROL_MOUNTS, "the kernel named the mount's own source");
            return;
        }
        MountSourceAnswer::NoAnswer(why) => why,
    };
    let buffer: Vec<u8> = match the_answer_for(unique) {
        Err(error) => {
            the_call_answered_nothing(control, &error);
            return;
        }
        Ok(buffer) => buffer,
    };
    assert_eq!(
        super::number_at(buffer.as_slice(), super::MASK_AT) & super::STATMOUNT_SOURCE,
        0,
        "the call answered with a source field, so the release call reading no source from it is a defect: {why}"
    );
    eprintln!(
        "STATMOUNT SOURCE CONTROL NOT RUN: {control} (this kernel answers statmount(2) but reports no sb_source, which \
         arrived in Linux 6.13, so the sweep reads the live table here: {why})"
    );
}

/// What:
///  An answer that carries a source carries its two strings at the offsets the
///  fixed part names,
///  behind a header of the size this build assumes.
/// Why:
///  The header size and the two string offsets are constants of the kernel's
///  uapi,
///  and a kernel that laid them out differently would put another string where
///  the source is expected;
///  the mount's root reading `/` is what the call proves instead of assuming.
#[test]
fn the_answer_reads_its_strings_at_the_offsets_it_names() {
    let control: &str = "worktree_clone_mount_handler::sweep_standing_mount_source::tests::the_answer_reads_its_strings_at_the_offsets_it_names";
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(INSIDE, control) {
        return;
    }
    let root: std::path::PathBuf = crate::test_support::fixture("standing-mount-source-offsets");
    let folder: std::path::PathBuf = root.join("slot");
    std::fs::create_dir_all(folder.as_path()).expect("the folder is made");
    mount_tmpfs(THE_SOURCE_THE_LAYOUT_CONTROL_MOUNTS, folder.as_path());
    let _guard: MountedFolder = MountedFolder { path: folder.clone() };
    let unique: u64 = unique_number_of(folder.as_path());
    let buffer: Vec<u8> = match the_answer_for(unique) {
        Ok(buffer) => buffer,
        Err(error) => {
            the_call_answered_nothing(control, &error);
            return;
        }
    };
    let mask: u64 = super::number_at(buffer.as_slice(), super::MASK_AT);
    if mask & super::STATMOUNT_SOURCE == 0 {
        eprintln!("STATMOUNT SOURCE CONTROL NOT RUN: {control} (this kernel reports no mount source, so only the call itself is proved here)");
        return;
    }
    assert_eq!(
        super::string_at(buffer.as_slice(), super::field_at(buffer.as_slice(), super::MOUNT_ROOT_AT) as usize),
        Some("/"),
        "the root string sits where this build reads it"
    );
    assert_eq!(
        super::string_at(buffer.as_slice(), super::field_at(buffer.as_slice(), super::SOURCE_AT) as usize),
        Some(THE_SOURCE_THE_LAYOUT_CONTROL_MOUNTS),
        "the source string sits where this build reads it"
    );
}

/// What:
///  A kernel with no unique mount number for the slot,
///  no call for this architecture and a mount number no mount holds
///  all answer "no answer" rather than a source the caller could trust.
/// Why:
///  The sweep falls back to the live table on that answer,
///  and a source it made up would be worse than no answer.
#[test]
fn an_answer_that_names_no_mount_is_no_answer() {
    let absent: MountSourceAnswer = KernelMountSource.source_of(Some(u64::MAX - 1));
    assert!(matches!(absent, MountSourceAnswer::NoAnswer(_)), "{absent:?}");
    let unnumbered: MountSourceAnswer = KernelMountSource.source_of(None);
    assert!(matches!(unnumbered, MountSourceAnswer::NoAnswer(_)), "{unnumbered:?}");
    let said: String = match unnumbered {
        MountSourceAnswer::NoAnswer(why) => why,
        MountSourceAnswer::Source(source) => panic!("no unique number is no answer, not the source {source}"),
    };
    assert!(said.contains("no unique mount number"), "the answer says what was missing: {said}");
}
