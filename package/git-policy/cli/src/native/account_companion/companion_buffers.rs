//! What:
//!  Every name and every line the companion's child writes,
//!  built in the parent before the fork.
//! Why:
//!  The child's process holds only the thread that forked it,
//!  so a call it makes that allocates can take a lock another thread held at
//!  the moment of the fork and never give it back
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "The child starts").
//!  The strings and the mapping lines are therefore made here,
//!  and the child only reads them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const buffers = buildBuffers({ scratch, euid, egid }); // every path and line the child writes
//! ```

use super::companion_steps::CompanionStep;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt as _;
use std::path::Path;

/// The name of the slot inside the scratch tmpfs.
pub const SLOT_NAME: &str = "slot";
/// The name of the folder the mount is moved to before its unmount.
pub const MOVE_TARGET_NAME: &str = "move-target";
/// The name of the refusal tree inside the scratch tmpfs.
pub const REFUSAL_NAME: &str = "refusal";
/// The name of the folder the bind mount is planted on.
pub const BOUND_NAME: &str = "bound";
/// The name of the symbolic link the walk must refuse.
pub const LINK_NAME: &str = "link";
/// The name of the folder whose identity is recorded and then changed.
pub const REPLACED_NAME: &str = "replaced";
/// The name the replaced folder is moved to.
pub const REPLACED_ELSEWHERE_NAME: &str = "replaced-elsewhere";
/// The refusal tree's bound folder,
/// as a path below the scratch mount point.
pub const BOUND_PATH: &str = "refusal/bound";
/// The slot the bind mount is planted over.
pub const BOUND_SLOT_PATH: &str = "refusal/bound/slot";
/// The folder whose identity is changed under its recorded one.
pub const REPLACED_PATH: &str = "refusal/replaced";
/// The relative path the walk must refuse to leave the scratch root with.
pub const ESCAPE_NAME: &str = "../escape";
/// The scratch tmpfs's size and inode count,
/// large enough for the slot,
/// the move target,
/// the refusal tree and one link.
pub const SCRATCH_TMPFS_OPTIONS: &str = "size=64k,nr_inodes=32,mode=0700";
/// The filesystem the scratch and the attached mount are.
pub const TMPFS: &str = "tmpfs";
/// The root the propagation is made on.
pub const ROOT: &str = "/";
/// Where the first mapping is written.
pub const UID_MAP: &str = "/proc/self/uid_map";
/// Where the second mapping is written.
pub const GID_MAP: &str = "/proc/self/gid_map";
/// Where the group list is denied before the second mapping.
pub const SETGROUPS: &str = "/proc/self/setgroups";
/// The mount table the attach's facts are read from.
pub const MOUNTINFO: &str = "/proc/self/mountinfo";
/// The word written to `/proc/self/setgroups`.
pub const SETGROUPS_DENY: &[u8] = b"deny";

/// What:
///  One test-only fixture the parent parsed before the fork:
///  the step whose call answers an errno the harness chose,
///  and that errno.
/// Why:
///  M11's `fsmount` and second-mapping-write fixtures need a seam the kernel's
///  syscall filters cannot give,
///  because a filter names a system call and cannot single out one `write` of
///  three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fixture = { step: Step; errno: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fixture {
    /// The step whose first call answers this errno.
    pub step: CompanionStep,
    /// The errno it answers.
    pub errno: i32,
}

/// What:
///  Everything the child reads,
///  as C strings and lines made before the fork.
/// Why:
///  The child names no path of its own and allocates nothing,
///  so every name it uses is a borrow of this value,
///  and a path with a NUL byte is refused here rather than in the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Buffers = { scratch: Buffer; slot: Buffer; ... };
/// ```
pub struct CompanionBuffers {
    /// The scratch mount point under the run's own directory.
    pub scratch: CString,
    /// The slot inside the scratch tmpfs.
    pub slot: CString,
    /// The folder the mount is moved to.
    pub move_target: CString,
    /// The refusal tree.
    pub refusal: CString,
    /// The folder the bind mount is planted on.
    pub bound: CString,
    /// The slot the bind mount is planted over,
    /// inside the refusal tree.
    pub bound_slot: CString,
    /// The folder whose identity is recorded and then changed.
    pub replaced: CString,
    /// The name of the slot,
    /// relative to the scratch root.
    pub slot_name: CString,
    /// The refusal tree's name,
    /// relative to the scratch root.
    pub refusal_name: CString,
    /// The bound folder's name,
    /// relative to the refusal tree.
    pub bound_name: CString,
    /// The link's name,
    /// relative to the refusal tree.
    pub link_name: CString,
    /// The replaced folder's name,
    /// relative to the refusal tree.
    pub replaced_name: CString,
    /// The name the replaced folder is moved to.
    pub replaced_elsewhere_name: CString,
    /// The relative path that must leave the scratch root.
    pub escape_name: CString,
    /// The line written to `uid_map`.
    pub uid_line: CString,
    /// The line written to `gid_map`.
    pub gid_line: CString,
    /// The tmpfs options.
    pub tmpfs_options: CString,
    /// The filesystem type the two mounts are made as.
    pub tmpfs: CString,
    /// The root the propagation is made on.
    pub root: CString,
    /// Where the group list is denied.
    pub setgroups: CString,
    /// Where the first mapping is written.
    pub uid_map: CString,
    /// Where the second mapping is written.
    pub gid_map: CString,
    /// The mount table the facts are read from.
    pub mountinfo: CString,
    /// The fixture,
    /// where the harness set one.
    pub fixture: Option<Fixture>,
    /// The step the child sleeps after,
    /// which only the deadline arm sets.
    pub hold: Option<CompanionStep>,
}

/// What:
///  Why the buffers could not be built:
///  the path the run would use holds a NUL byte or no name.
/// Why:
///  A path that cannot cross into C cannot be given to a call,
///  and the run must say so rather than fork a child that would do nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BufferRefusal = { path: string; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BufferRefusal {
    /// The path that could not be used.
    pub path: String,
    /// Why it could not.
    pub why: &'static str,
}

/// A NUL-terminated copy of one path or name,
/// or the refusal that says why it has none.
fn c_string(bytes: &[u8], path: &Path) -> Result<CString, BufferRefusal> {
    return CString::new(bytes).map_err(|_zero| return BufferRefusal { path: path.display().to_string(), why: "it holds a NUL byte" });
}

/// A NUL-terminated copy of one fixed word.
fn word(text: &str) -> CString {
    return CString::new(text).expect("a fixed word holds no NUL byte");
}

/// What:
///  Every name and line the child needs,
///  made before the fork.
/// Why:
///  The mapping lines name this process's effective ids,
///  which are read before the `unshare` because the ids read after it are the
///  overflow ids the kernel refuses.
///
/// # Errors
/// The scratch path holds a NUL byte,
///  so no call could name it.
pub fn build(scratch: &Path, user: u32, group: u32, fixture: Option<Fixture>, hold: Option<CompanionStep>) -> Result<CompanionBuffers, BufferRefusal> {
    let bytes: &[u8] = scratch.as_os_str().as_bytes();
    let joined = |name: &str| -> Vec<u8> {
        let mut path: Vec<u8> = bytes.to_vec();
        path.push(b'/');
        path.extend_from_slice(name.as_bytes());
        return path;
    };
    return Ok(CompanionBuffers {
        scratch: c_string(bytes, scratch)?,
        slot: c_string(joined(SLOT_NAME).as_slice(), scratch)?,
        move_target: c_string(joined(MOVE_TARGET_NAME).as_slice(), scratch)?,
        refusal: c_string(joined(REFUSAL_NAME).as_slice(), scratch)?,
        bound: c_string(joined(BOUND_PATH).as_slice(), scratch)?,
        bound_slot: c_string(joined(BOUND_SLOT_PATH).as_slice(), scratch)?,
        replaced: c_string(joined(REPLACED_PATH).as_slice(), scratch)?,
        slot_name: word(SLOT_NAME),
        refusal_name: word(REFUSAL_NAME),
        bound_name: word(BOUND_NAME),
        link_name: word(LINK_NAME),
        replaced_name: word(REPLACED_NAME),
        replaced_elsewhere_name: word(REPLACED_ELSEWHERE_NAME),
        escape_name: word(ESCAPE_NAME),
        uid_line: word(format!("0 {user} 1\n").as_str()),
        gid_line: word(format!("0 {group} 1\n").as_str()),
        tmpfs_options: word(SCRATCH_TMPFS_OPTIONS),
        tmpfs: word(TMPFS),
        root: word(ROOT),
        setgroups: word(SETGROUPS),
        uid_map: word(UID_MAP),
        gid_map: word(GID_MAP),
        mountinfo: word(MOUNTINFO),
        fixture,
        hold,
    });
}
