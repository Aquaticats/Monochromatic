//! What:
//!  The reads one sweep makes of the pool:
//!  one dataset's identity and properties,
//!  and the immediate children of the pinned parent.
//! Why:
//!  The sweep enumerates the parent's immediate children itself and verifies
//!  each one
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering"),
//!  and both reads go through `/dev/zfs` rather than through a started `zfs`
//!  program.
//!  The trait is the seam a control drives with a scripted pool,
//!  because no control on a person's machine may open the device;
//!  the release implementation is the binding's own handle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reader = zfsReader(); const children = childrenOf(reader, 'zpcachyos/…/clones');
//! ```

use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DeviceError, ListedObject, ZfsDevice};

/// What:
///  How many children one sweep enumerates before it stops.
/// Why:
///  The listing is the only unbounded loop the sweep takes from the pool,
///  and a parent with a runaway number of children would make one ZFS event
///  scan for ever.
///  The bound is a choice of this build and not a measurement:
///  the design names no limit,
///  and four thousand children is far more than the number of worktrees a
///  checkout holds.
///  A listing longer than it refuses rather than truncating,
///  so a parent nobody meant to configure is reported instead of half swept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let cursor = 0, count = 0; count < 4096; count += 1) { ... }
/// ```
pub const MAXIMUM_CHILDREN: usize = 4096;

/// What:
///  One step of the parent's child listing.
/// Why:
///  The kernel writes the next child's name and the cursor to ask after in one
///  reply,
///  so a caller keeps the two together and never reads a half-updated cursor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListedChild = { name: string; cursor: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedChild {
    /// The child's full dataset name.
    pub name: String,
    /// The cursor the next step is asked after.
    pub cursor: u64,
}

/// What:
///  The pool reads one sweep makes.
/// Why:
///  The sweep needs one dataset's identity and one step of the parent's listing,
///  and nothing else,
///  so the seam is exactly those two calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface DatasetReader { identity(dataset): DatasetIdentity; nextChild(parent, cursor): ListedChild | undefined }
/// ```
pub trait DatasetReader {
    /// One dataset's identity and property reply.
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, String>;

    /// The child of `parent` after `cursor`,
    /// or nothing at the end of the listing.
    fn next_child(&self, parent: &str, cursor: u64) -> Result<Option<ListedChild>, String>;
}

/// What:
///  The release reader:
///  the binding's open `/dev/zfs` handle.
/// Why:
///  The route reads the pool through the typed binding rather than by starting
///  `zfs`,
///  so the handler loads no code from a checkout and needs no program on the
///  path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = new ZfsDatasetReader();
/// ```
pub struct ZfsDatasetReader {
    /// The open device.
    device: ZfsDevice,
}

impl ZfsDatasetReader {
    /// Open the device one sweep reads and writes through.
    ///
    /// # Errors
    /// The device is absent or refuses the open,
    /// which is what a machine without OpenZFS answers.
    pub fn open() -> Result<Self, DeviceError> {
        return Ok(ZfsDatasetReader { device: ZfsDevice::open()? });
    }
}

impl DatasetReader for ZfsDatasetReader {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, String> {
        return self.device.objset_stats(dataset).map_err(|error| return error.to_string());
    }

    fn next_child(&self, parent: &str, cursor: u64) -> Result<Option<ListedChild>, String> {
        let listed: Option<ListedObject> = self.device.dataset_list_next(parent, cursor).map_err(|error| return error.to_string())?;
        return Ok(listed.map(|object| return ListedChild { name: object.name, cursor: object.cursor }));
    }
}

/// What:
///  Why the parent's children could not be enumerated.
/// Why:
///  A read that refused and a parent with more children than one sweep takes are
///  two different things for a person to look at,
///  and the second is this build's own bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildrenRefusal = { kind: 'read' | 'too-many'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChildrenRefusal {
    /// One step of the listing refused.
    Read {
        /// The parent the listing walked.
        parent: String,
        /// The cursor the step was asked after.
        cursor: u64,
        /// What the read answered.
        reason: String,
    },
    /// The parent holds more children than one sweep enumerates.
    TooMany {
        /// The parent the listing walked.
        parent: String,
        /// The bound one sweep takes.
        bound: usize,
    },
}

impl ChildrenRefusal {
    /// The refusal in words,
    /// naming the parent and the way forward.
    pub fn message(&self) -> String {
        return match self {
            ChildrenRefusal::Read { parent, cursor, reason } => format!(
                "the children of the clone parent `{parent}` could not be listed after cursor {cursor} ({reason}), so this sweep stopped there: the clones it already attached keep their mounts, no further clone was attached. Ways forward: check that the parent dataset is imported and mounted as the configuration names it, then run the sweep again from the handler's journal entry."
            ),
            ChildrenRefusal::TooMany { parent, bound } => format!(
                "the clone parent `{parent}` lists more than the {bound} children one sweep takes, so this sweep stopped: the clones it already attached keep their mounts and no further clone was attached. Ways forward: `zfs list -r -d 1 -H -o name {parent}` shows the children, a child that is no clone is answered by the checks rather than by this bound, and this build's bound is written in `sweep_dataset_reader.rs`."
            ),
        };
    }
}

/// What:
///  What one listing reached:
///  how many children it gave,
///  and why it stopped before the end when it did.
/// Why:
///  A sweep must keep the clones it already attached when a later step of the
///  listing refuses,
///  so the count comes back with the stop rather than instead of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Listing = { seen: number; refusal?: ChildrenRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Listing {
    /// How many children the listing gave before it stopped.
    pub seen: usize,
    /// Why it stopped before the end,
    /// or nothing when it reached the end.
    pub refusal: Option<ChildrenRefusal>,
}

/// What:
///  Walk the pinned parent's immediate children,
///  handing each to `visit` as the kernel lists it.
/// Why:
///  The sweep enumerates the parent itself and lists no store folder,
///  which is what the accepted per-slot store design chose
///  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
///  "The sweep plan step 8").
///  A child is visited as it is listed,
///  so a step that refuses later costs the clones already reached nothing:
///  their mounts stand and the listing says where it stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listing = eachChild(reader, parent, (name) => turn(name));
/// ```
pub fn each_child(reader: &dyn DatasetReader, parent: &str, visit: &mut impl FnMut(&str)) -> Listing {
    let mut cursor: u64 = 0;
    let mut seen: usize = 0;
    loop {
        let step: Option<ListedChild> = match reader.next_child(parent, cursor) {
            Ok(step) => step,
            Err(reason) => return Listing { seen, refusal: Some(ChildrenRefusal::Read { parent: String::from(parent), cursor, reason }) },
        };
        // The end is asked for even at the bound, so a parent holding exactly the bound is listed whole and only one more
        // child refuses.
        let Some(child) = step else {
            return Listing { seen, refusal: None };
        };
        if seen == MAXIMUM_CHILDREN {
            return Listing { seen, refusal: Some(ChildrenRefusal::TooMany { parent: String::from(parent), bound: MAXIMUM_CHILDREN }) };
        }
        seen += 1;
        visit(child.name.as_str());
        cursor = child.cursor;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_dataset_reader_tests.rs"]
mod tests;
