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
///  How many children one sweep visits before it stops.
/// Why:
///  The listing is the only unbounded loop the sweep takes from the pool,
///  and a parent with a runaway number of children would make one ZFS event
///  scan for ever.
///  The design names no limit,
///  so the project adopted this number with a measurement in the guest and its
///  veto open
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "Adopted with the project's veto open"):
///  one sweep over one more child than this visits the first this many,
///  answers each from the checks,
///  and stops at the next one with a report that says where it stopped,
///  so a parent nobody meant to configure is reported and the clones past the
///  bound are left unattached rather than half swept.
///  The measurements beside the number are the mounted shape a parent of
///  clones holds,
///  measured again 2026-11-23T19:50-05:00 after the standing answer began asking
///  the kernel for one mount's source by its unique number,
///  in the throwaway guest `zfs-step-8-sweep-timing-2026-11-23`
///  (Ubuntu 24.04,
///  kernel 6.14.0-37-generic,
///  OpenZFS 2.4.4,
///  four vCPUs and 7,941 MB),
///  interleaved,
///  so no arm carries the machine's mood:
///  this branch's handler,
///  main's handler,
///  and this branch again as an identical-binary control,
///  each round rotated,
///  and a one-child control beside every many-child sweep.
///  Every figure is a median of one arm's sweeps in that run,
///  and the raw lines are
///  `perf/sweep-cost-2026-11-23-statmount-interleaved.json`.
///  Steady sweeps over standing mounts were 0.34 of main's time at 1,024
///  children (0.19 to 0.42,
///  identical-binary control 0.65 to 1.65),
///  which the other run kept beside it measured as 0.36 (0.30 to 0.58,
///  control 0.71 to 1.10);
///  at 64 children the ratio was 0.89 (0.68 to 1.00) and 1.07 (0.92 to 1.30),
///  and at 8 children 1.14 in both runs,
///  each inside its control's spread,
///  because a sweep that small is mostly the run's fixed cost either way.
///  Cold sweeps that attach every clone were 0.97, 1.03 and 1.02 of main across
///  three interleaved passes at 1,024 children (identical-binary control 0.92,
///  0.90 and 1.04),
///  and each arm opened the mount table 3,073 and 3,074 times there,
///  so the attach path is not the standing answer's.
///  A sweep whose mount table changed before every answer was 1.05 to 1.47 of
///  main,
///  inside the identical-binary spread measured beside it.
///  The added call costs 3.1 to 4.4 microseconds on that guest,
///  so 1,024 standing answers pay 3 to 5 ms of a sweep that takes 3 to 4 s.
///  No 2,048 or 4,096 figure is cited here because no raw line for that shape
///  is kept for this build;
///  the earlier 4,097-child figure counted children the origin check refuses,
///  which is not a parent of mounted clones,
///  and its raw lines are not kept either.
///  The bound stays the project's to change,
///  and these are the measurements it is carried with now.
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
///  One failed read of the pool,
///  in the binding's own words and with the kernel's error number where it
///  kept one.
/// Why:
///  A sweep decides a pool is gone from `ENOENT` rather than from a sentence the
///  binding happens to print,
///  so the number travels with the words and one rule reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadFailure = { said: string; errno?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadFailure {
    /// What the read answered.
    pub said: String,
    /// The kernel's error number,
    /// where the read kept one.
    pub errno: Option<i32>,
}

impl ReadFailure {
    /// A failure this route built itself,
    /// with no kernel answer behind it.
    pub fn at(said: String) -> Self {
        return ReadFailure { said, errno: None };
    }

    /// The failure in words.
    pub fn message(&self) -> String {
        return self.said.clone();
    }
}

impl From<String> for ReadFailure {
    fn from(said: String) -> Self {
        return ReadFailure { said, errno: None };
    }
}

impl std::fmt::Display for ReadFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.said.as_str());
    }
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
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure>;

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
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        return self.device.objset_stats(dataset).map_err(|error| return ReadFailure { said: error.to_string(), errno: error.errno() });
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
