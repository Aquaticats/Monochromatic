//! What:
//!  Controls of the call that names one mount's source by its unique number,
//!  and of the answer a kernel without it gives.
//! Why:
//!  The rename gap this closes is a dataset swap that raises no mount event,
//!  so the control mounts one tmpfs inside a namespace of its own and reads
//!  the source the kernel reports for it,
//!  which the kernel prints from the same `show_devname` the mount table uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(kernelMountSource().sourceOf(uniqueNumberOf(target))).toEqual({ kind: 'source', source: 'tmpfs' });
//! ```

use super::{KernelMountSource, MountSourceAnswer, MountSourceByNumber, STATMOUNT_HEADER_BYTES};
use std::os::unix::ffi::OsStrExt as _;

/// The variable the inner half of the control sets.
const INSIDE: &str = "CLI_GIT_STANDING_MOUNT_SOURCE_ROOT";

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
///  The source the kernel reports for one mount by its unique number is the
///  source that mount was made with.
/// Why:
///  This is the call the rename gap is closed with,
///  so the answer has to be read here before a sweep can rest on it.
#[test]
fn the_kernel_names_the_source_of_one_mount_by_its_unique_number() {
    if !crate::testing::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_standing_mount_source::tests::the_kernel_names_the_source_of_one_mount_by_its_unique_number",
    ) {
        return;
    }
    let root: std::path::PathBuf = crate::test_support::fixture("standing-mount-source");
    let folder: std::path::PathBuf = root.join("slot");
    std::fs::create_dir_all(folder.as_path()).expect("the folder is made");
    mount_tmpfs("a-source-this-control-names", folder.as_path());
    let _guard: MountedFolder = MountedFolder { path: folder.clone() };
    let unique: u64 = unique_number_of(folder.as_path());
    let answer: MountSourceAnswer = KernelMountSource.source_of(Some(unique));
    match answer {
        MountSourceAnswer::Source(source) => assert_eq!(source, "a-source-this-control-names", "the kernel named the mount's own source"),
        MountSourceAnswer::NoAnswer(why) => {
            // A kernel before 6.13 reports no source at all, and the sweep then reads the live table; the string test below
            // is what proves the layout on a kernel that does answer.
            assert!(why.contains("6.13") || why.contains("statmount"), "the answer says why it could not name the source: {why}");
        }
    }
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
    if !crate::testing::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_standing_mount_source::tests::the_answer_reads_its_strings_at_the_offsets_it_names",
    ) {
        return;
    }
    let root: std::path::PathBuf = crate::test_support::fixture("standing-mount-source-offsets");
    let folder: std::path::PathBuf = root.join("slot");
    std::fs::create_dir_all(folder.as_path()).expect("the folder is made");
    mount_tmpfs("offset-source", folder.as_path());
    let _guard: MountedFolder = MountedFolder { path: folder.clone() };
    let unique: u64 = unique_number_of(folder.as_path());
    let mut buffer: Vec<u8> = vec![0_u8; STATMOUNT_HEADER_BYTES + 1_024];
    let request: super::MountNumberRequest = super::MountNumberRequest {
        size: super::MOUNT_NUMBER_REQUEST_BYTES,
        flags_or_descriptor: 0,
        mount_number: unique,
        fields: super::STATMOUNT_MOUNT_ROOT | super::STATMOUNT_SOURCE,
    };
    let number: libc::c_long = super::STATMOUNT_NUMBER.expect("this architecture declares the call");
    // SAFETY: the request and the buffer are this call's own and outlive it, the buffer is writable for its length, and
    // the flags argument is the zero the call takes.
    let answered: libc::c_long = unsafe { libc::syscall(number, &request as *const super::MountNumberRequest, buffer.as_mut_ptr(), buffer.len(), 0_u32) };
    assert!(answered >= 0, "the call answers on this kernel: {}", std::io::Error::last_os_error());
    let mask: u64 = super::number_at(&buffer, super::MASK_AT);
    if mask & super::STATMOUNT_SOURCE == 0 {
        eprintln!("this kernel reports no mount source, so only the call itself is proved here");
        return;
    }
    assert_eq!(super::string_at(&buffer, super::field_at(&buffer, super::MOUNT_ROOT_AT) as usize), Some("/"), "the root string sits where this build reads it");
    assert_eq!(super::string_at(&buffer, super::field_at(&buffer, super::SOURCE_AT) as usize), Some("offset-source"), "the source string sits where this build reads it");
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
