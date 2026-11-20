//! What:
//!  Typed `/dev/zfs` requests for the Git wrapper's ZFS worktree route:
//! the
//!  OpenZFS 2.4.4 command struct,
//! checked names,
//! a native name-value list codec
//!  and bounded replies.
//! Why:
//!  A Git worktree that is a ZFS clone needs snapshots,
//! clones,
//! property reads
//!  and exact snapshot destruction,
//! all through the kernel's one device file.
//!  The design chose a small binding of this repository's own rather than
//!  linking OpenZFS's CDDL library or starting the `zfs` program,
//! so a program
//!  that links this crate still starts on a machine with no OpenZFS.
//!  A request is built without a device,
//! and `kernel_device` carries it to
//!  `/dev/zfs` and reads the answer;
//! the start-time kernel compatibility
//!  gate is step 6,
//! and it builds on the layout comparison the guest tests
//!  make.
//!
//! The design and its evidence live in
//! `doc/planning/cli-git-zfs-worktree-design-proposal.md`;
//! the codec and the
//! layout mirror `Ukko-Ylijumala/zfsdev-rs` at `4559ffc711`,
//! whose provenance
//! `PROVENANCE.md` records.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { snapshot, destroySnaps, nvlist } from '@monochromatic-dev/zfs-request-to-openzfs-kernel-device';
//! ```

/// The start-time compatibility gate's layout half.
pub mod compatibility_check;
/// The open `/dev/zfs` handle and the live calls it carries.
pub mod kernel_device;
/// The ABI bytes:
/// the command struct,
/// its field bounds and the buffer.
pub mod layout;
/// The checked names a request writes into the fixed fields.
pub mod names;
/// The native name-value list codec.
pub mod nvlist;
/// The typed requests,
/// their replies and their failure type.
pub mod requests;

/// The gate's entry point,
/// its refusal and what a pass read.
pub use compatibility_check::{
    Compatible, LayoutRequest, Refusal as CompatibilityRefusal, Step as CompatibilityStep,
    check_module_layout,
};
/// The open device,
/// why a live call failed,
/// the identity read and one step of a listing.
pub use kernel_device::{DEVICE_PATH, DatasetIdentity, DeviceError, ListedObject, ZfsDevice};
/// The buffer and struct a request writes into.
pub use layout::{CommandBuffer, DmuObjsetStats, ZfsCmd};
/// The checked name types and their field bounds.
pub use names::{CheckedName, NameError, NameField};
/// The codec list,
/// values and refusal type.
pub use nvlist::{NvData, NvError, NvList};
/// The request constructors,
/// their value,
/// operation and failure type.
pub use requests::{
    ChannelProgramRequest, Error, Operation, ReplyBuffer, Request, channel_program, clone,
    dataset_list_next, destroy, destroy_snaps, get_bookmarks, get_fsacl, objset_stats, promote,
    set_prop, snapshot, snapshot_list_next,
};
