//! What:
//!  The calls the companion's child makes,
//!  the word each is named by,
//!  and the one value a failed call answers.
//! Why:
//!  A skip names the call the host refused,
//!  so the call is a value rather than a sentence,
//!  and the child sends its number on a pipe instead of building text in a
//!  process whose other threads it did not copy
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "The child starts").
//!  A number the parent does not know is the parent's own arm failure rather
//!  than a guess at a call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Failure = { call: Call; errno: number };
//! ```

/// What:
///  One call the child can name in a record.
/// Why:
///  The skip's line and the failed-arm's line both name the call,
///  so the vocabulary is closed and carried as a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Call = 'unshare' | 'mount' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CompanionCall {
    /// `unshare(CLONE_NEWUSER)`.
    UnshareUserNamespace = 0,
    /// The open of `/proc/self/setgroups` for its `deny` write.
    OpenSetgroups = 1,
    /// The `deny` write itself.
    WriteSetgroups = 2,
    /// The open of `/proc/self/uid_map`.
    OpenUidMap = 3,
    /// The first mapping write.
    WriteUidMap = 4,
    /// The open of `/proc/self/gid_map`.
    OpenGidMap = 5,
    /// The second mapping write.
    WriteGidMap = 6,
    /// `unshare(CLONE_NEWNS)`.
    UnshareMountNamespace = 7,
    /// `mount_setattr` making every mount private.
    MountSetattr = 8,
    /// `mount` making every mount private,
    /// the fallback the design names.
    MountPropagation = 9,
    /// `mount` of the scratch tmpfs.
    MountTmpfs = 10,
    /// `mkdirat` inside the scratch tmpfs.
    Mkdir = 11,
    /// `open` of a directory the walk holds.
    OpenDirectory = 12,
    /// `openat2`,
    /// the no-crossing walk.
    Openat2 = 13,
    /// `statx`,
    /// the identity of a folder.
    Statx = 14,
    /// `symlinkat`,
    /// the link control.
    Symlinkat = 15,
    /// `renameat2`,
    /// the changed-identity control.
    Renameat = 16,
    /// `fsopen`.
    FsOpen = 17,
    /// `fsconfig`.
    FsConfig = 18,
    /// `fsmount`.
    FsMount = 19,
    /// `move_mount` onto the held slot descriptor.
    MoveMount = 20,
    /// The read of `/proc/self/fdinfo`,
    /// where the mount's ID comes from.
    ReadFdinfo = 21,
    /// The read of `/proc/self/mountinfo`.
    ReadMountinfo = 22,
    /// `umount2`.
    Umount = 23,
    /// `fork`.
    Fork = 24,
    /// `pipe2`.
    Pipe = 25,
    /// The write of one step's record.
    WriteRecord = 26,
    /// The read-back's own comparison,
    /// which no kernel call answers,
    /// so its failure carries no errno.
    MountFactsCheck = 27,
}

/// Every call,
/// in the numbering a record carries.
pub const CALLS_IN_ORDER: &[CompanionCall] = &[
    CompanionCall::UnshareUserNamespace,
    CompanionCall::OpenSetgroups,
    CompanionCall::WriteSetgroups,
    CompanionCall::OpenUidMap,
    CompanionCall::WriteUidMap,
    CompanionCall::OpenGidMap,
    CompanionCall::WriteGidMap,
    CompanionCall::UnshareMountNamespace,
    CompanionCall::MountSetattr,
    CompanionCall::MountPropagation,
    CompanionCall::MountTmpfs,
    CompanionCall::Mkdir,
    CompanionCall::OpenDirectory,
    CompanionCall::Openat2,
    CompanionCall::Statx,
    CompanionCall::Symlinkat,
    CompanionCall::Renameat,
    CompanionCall::FsOpen,
    CompanionCall::FsConfig,
    CompanionCall::FsMount,
    CompanionCall::MoveMount,
    CompanionCall::ReadFdinfo,
    CompanionCall::ReadMountinfo,
    CompanionCall::Umount,
    CompanionCall::Fork,
    CompanionCall::Pipe,
    CompanionCall::WriteRecord,
    CompanionCall::MountFactsCheck,
];

impl CompanionCall {
    /// The call one number names,
    /// or nothing when the number names no call of this build.
    pub fn from_number(number: u8) -> Option<CompanionCall> {
        return CALLS_IN_ORDER
            .iter()
            .copied()
            .find(|call| return *call as u8 == number);
    }

    /// The name the lines print,
    /// with the call the kernel documents rather than a wrapper's own word.
    pub fn name(self) -> &'static str {
        return match self {
            CompanionCall::UnshareUserNamespace | CompanionCall::UnshareMountNamespace => "unshare",
            CompanionCall::OpenSetgroups => "open /proc/self/setgroups",
            CompanionCall::WriteSetgroups => "write /proc/self/setgroups",
            CompanionCall::OpenUidMap => "open /proc/self/uid_map",
            CompanionCall::WriteUidMap => "write /proc/self/uid_map",
            CompanionCall::OpenGidMap => "open /proc/self/gid_map",
            CompanionCall::WriteGidMap => "write /proc/self/gid_map",
            CompanionCall::MountSetattr => "mount_setattr",
            CompanionCall::MountPropagation => "mount",
            CompanionCall::MountTmpfs => "mount tmpfs",
            CompanionCall::Mkdir => "mkdirat",
            CompanionCall::OpenDirectory => "open",
            CompanionCall::Openat2 => "openat2",
            CompanionCall::Statx => "statx",
            CompanionCall::Symlinkat => "symlinkat",
            CompanionCall::Renameat => "renameat",
            CompanionCall::FsOpen => "fsopen",
            CompanionCall::FsConfig => "fsconfig",
            CompanionCall::FsMount => "fsmount",
            CompanionCall::MoveMount => "move_mount",
            CompanionCall::ReadFdinfo => "read /proc/self/fdinfo",
            CompanionCall::ReadMountinfo => "read /proc/self/mountinfo",
            CompanionCall::Umount => "umount2",
            CompanionCall::Fork => "fork",
            CompanionCall::Pipe => "pipe2",
            CompanionCall::WriteRecord => "write",
            CompanionCall::MountFactsCheck => "the mount facts read back",
        };
    }

    /// The errno's name where this build knows it,
    /// and its number otherwise.
    pub fn errno_word(errno: i32) -> String {
        return match crate::start_probe::probe_errno_names::errno_of(errno) {
            Some(name) => String::from(name),
            None => format!("{errno}"),
        };
    }
}

/// What:
///  One failed call,
///  as a child answers it.
/// Why:
///  A step stops at its first failure,
///  and both lines the design words need the call and the errno.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failure = { call: Call; errno: number };
/// ```
/// The errno field a failure carries when no call answered it,
/// which a comparison's failure is.
pub const NO_ERRNO: i32 = -1;

/// What:
///  One failed step,
///  as the child or the parent answers it.
/// Why:
///  A skipped or refused run's line names the call and the number the kernel
///  gave,
///  and a comparison has no number to give,
///  which is the one place `NO_ERRNO` is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failure = { call: Call; errno: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Failure {
    /// The call that refused.
    pub call: CompanionCall,
    /// The number the kernel answered.
    pub errno: i32,
}

/// What:
///  The failure one refused call answers,
///  from the number the kernel left in this thread.
/// Why:
///  A child reads the thread's own number rather than building an
///  `io::Error`,
/// so nothing here allocates.
pub fn failed(call: CompanionCall) -> Failure {
    // SAFETY: `__errno_location` answers this thread's own error number, which the failed call has just set.
    let number: i32 = unsafe { *libc::__errno_location() };
    return Failure {
        call,
        errno: number,
    };
}
