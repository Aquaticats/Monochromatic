//! What:
//!  One tmpfs mounted at a fixture slot,
//!  and a standing-mount reader whose table shapes that mount into a clone's own
//!  ZFS mount.
//! Why:
//!  A control on a person's machine cannot mount a ZFS dataset,
//!  and the proof that a slot holds this clone's own mount reads a mount's
//!  filesystem type,
//!  its root and the two attributes the attach enforces before it asks the
//!  kernel for that mount's source;
//!  a tmpfs carries the two attributes as any mount does,
//!  so a table that names it `zfs`,
//!  with root `/` and the clone's dataset as its source,
//!  is the one mount shape a fixture can give the proof.
//!  The reader takes the kernel's own table and its own poll,
//!  so a control proves the release predicate against a real mount rather than
//!  against a table written by hand.
//!  A control mounts only inside the user and mount namespace
//!  `root_namespace_test_support` re-executes it in,
//!  where a mount stays inside that namespace.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = MountedSlot.at(slotPath);          // rw,nosuid,nodev tmpfs
//! const standing = theSlotsMountReads('pool/clones/one', '/');
//! expect(standing.standingMountOf(request, 'pool/clones/one')).toBeDefined();
//! ```

use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount_handler::sweep_one_clone::{LiveMountTableSource, LiveStandingMounts, MountSourceAnswer, MountSourceByNumber, MountTableSource};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// The source name a control's tmpfs is mounted with,
/// which the shaper knows its own row by.
pub(crate) const SHAPED_SOURCE: &str = "monochromatic-standing-slot";

/// What:
///  One tmpfs a control mounted at a fixture slot,
///  unmounted when it goes however the control ends.
/// Why:
///  A control that fails mid-way must not leave a mount in the fixture tree it
///  removes next,
///  and a mount the kernel refuses to take down plainly is detached,
///  so the fixture folder can go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using slot = MountedSlot.at(path); // unmounted at scope exit
/// ```
pub(crate) struct MountedSlot {
    /// The folder the tmpfs is mounted on.
    at: PathBuf,
    /// The mount's own number,
    /// read after it was made,
    /// so a control can prove which mount a crossed handle is on.
    id: u64,
}

impl MountedSlot {
    /// What:
    ///  Mount one tmpfs at `path` with the two attributes the attach enforces.
    /// Why:
    ///  The proof reads those two attributes from the mount's own options,
    ///  so a mount without them is refused before the dataset is asked about at
    ///  all,
    ///  and a fixture that wants the accept must carry them.
    ///  The kernel call is used rather than `/usr/bin/mount`:
    ///  util-linux keeps a mounted-filesystem table in `/run`,
    ///  which this namespace may not write,
    ///  and a mount made without that entry is one its `umount` refuses to take
    ///  down here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const slot = MountedSlot.at(path); // mount('tmpfs', path, 'tmpfs', MS_NOSUID|MS_NODEV)
    /// ```
    pub(crate) fn at(path: &Path) -> Self {
        return Self::with_flags(path, libc::MS_NOSUID | libc::MS_NODEV);
    }

    /// Mount one tmpfs at `path` with `flags`.
    pub(crate) fn with_flags(path: &Path, flags: libc::c_ulong) -> Self {
        let source: std::ffi::CString = std::ffi::CString::new(SHAPED_SOURCE).expect("no NUL in the source");
        let target: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in the path");
        let kind: std::ffi::CString = std::ffi::CString::new("tmpfs").expect("no NUL in the type");
        // SAFETY: all three strings outlive the call, the type is a NUL-terminated name, and no data pointer is read.
        let mounted: libc::c_int = unsafe { libc::mount(source.as_ptr(), target.as_ptr(), kind.as_ptr(), flags, std::ptr::null()) };
        assert_eq!(mounted, 0, "the control's tmpfs mounts at {}: {}", path.display(), std::io::Error::last_os_error());
        let folder: std::fs::File = std::fs::File::open(path).expect("the mounted folder opens");
        let id: u64 = crate::fs_beneath_mount_facts::mount_id_of(&folder).expect("the mounted folder's own mount reads");
        return Self { at: path.to_path_buf(), id };
    }

    /// What:
    ///  Change one mount's flags in place.
    /// Why:
    ///  A remount keeps the mount's number,
    ///  so a control can prove that a change no mount event announces is still
    ///  seen after the namespace's poll.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// slot.remount(0); // the two attributes off
    /// ```
    pub(crate) fn remount(&self, flags: libc::c_ulong) {
        let target: std::ffi::CString = std::ffi::CString::new(self.at.as_os_str().as_bytes()).expect("no NUL in the path");
        // SAFETY: the target is a NUL-terminated path that outlives the call, and `MS_REMOUNT|MS_BIND` with no type or
        // data changes the per-mount flags in place.
        let changed: libc::c_int = unsafe { libc::mount(std::ptr::null(), target.as_ptr(), std::ptr::null(), libc::MS_REMOUNT | libc::MS_BIND | flags, std::ptr::null()) };
        assert_eq!(changed, 0, "the control's tmpfs remounts in place: {}", std::io::Error::last_os_error());
    }

    /// The folder the tmpfs is mounted on.
    pub(crate) fn path(&self) -> &Path {
        return self.at.as_path();
    }

    /// The mount's own number,
    /// which a control reads back from a handle it crossed into the mount.
    pub(crate) fn id(&self) -> u64 {
        return self.id;
    }
}

impl Drop for MountedSlot {
    fn drop(&mut self) {
        let target: std::ffi::CString = std::ffi::CString::new(self.at.as_os_str().as_bytes()).expect("no NUL in the path");
        // SAFETY: `target` is a NUL-terminated path that outlives both calls.
        let plain: libc::c_int = unsafe { libc::umount2(target.as_ptr(), 0) };
        if plain != 0 {
            // SAFETY: the same string, with the flag that detaches a busy mount.
            let _detached: libc::c_int = unsafe { libc::umount2(target.as_ptr(), libc::MNT_DETACH) };
        }
    }
}

/// What:
///  The kernel's mount table as one clone's own mount,
///  with the control's tmpfs row shaped as that clone's dataset mount.
/// Why:
///  The source,
///  the filesystem type and the root are the three fields a fixture cannot make
///  a real ZFS mount carry,
///  so the table names them and every other field,
///  including the two attributes,
///  stays the kernel's own answer;
///  a flag change therefore still refuses,
///  which is what makes the shaper evidence rather than a script.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = new CloneShapedTable(liveSource, clone, root);
/// ```
struct CloneShapedTable {
    /// The kernel's own table and its poll.
    live: LiveMountTableSource,
    /// The dataset name the slot's mount is given.
    clone: String,
    /// The root the slot's mount is given,
    /// which is `/` for a dataset's own mount and another path for a bind.
    root: PathBuf,
}

impl MountTableSource for CloneShapedTable {
    fn changed(&self) -> bool {
        return self.live.changed();
    }

    fn read(&self) -> Result<Vec<MountFacts>, String> {
        let mut facts: Vec<MountFacts> = self.live.read()?;
        for mount in &mut facts {
            if mount.source == SHAPED_SOURCE && mount.has_option("nosuid") && mount.has_option("nodev") {
                mount.filesystem_type = String::from("zfs");
                mount.source = self.clone.clone();
                mount.root = self.root.clone();
            }
        }
        return Ok(facts);
    }
}

/// What:
///  The kernel's answer about the source of one mount,
///  as the clone's dataset for every mount this control asks about.
/// Why:
///  The source question is asked by the mount's unique number on a kernel that
///  answers it,
///  and a control's tmpfs has no dataset name of its own to answer with;
///  what the answer is about is decided by the table the other half reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = new SourceSaysTheClone(clone);
/// ```
struct SourceSaysTheClone {
    /// The dataset name the answer gives.
    clone: String,
}

impl MountSourceByNumber for SourceSaysTheClone {
    fn source_of(&self, _unique_mount_number: Option<u64>) -> MountSourceAnswer {
        return MountSourceAnswer::Source(self.clone.clone());
    }
}

/// What:
///  A standing-mount reader whose table names the mounted slot's mount as
///  `clone`'s own mount at `root`.
/// Why:
///  A control proves the release proof,
///  not a table written by hand:
///  the reader takes the kernel's own table and poll,
///  and only the three fields no fixture mount can carry are the control's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = theSlotsMountReads('pool/clones/one', '/');
/// ```
pub(crate) fn the_slots_mount_reads(clone: &str, root: &str) -> LiveStandingMounts {
    let table: CloneShapedTable = CloneShapedTable { live: LiveMountTableSource::open(), clone: String::from(clone), root: PathBuf::from(root) };
    return LiveStandingMounts::from_sources(Box::new(table), Box::new(SourceSaysTheClone { clone: String::from(clone) }));
}
