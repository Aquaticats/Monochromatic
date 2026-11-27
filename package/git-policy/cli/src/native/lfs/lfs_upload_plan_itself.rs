//! What:
//!  What the Git LFS upload before a push sends:
//!  the destinations `git lfs push` is run for,
//!  the refs whose objects it uploads,
//!  and the argument lists one run of it takes.
//! Why:
//!  Both faces of the plan build this one value,
//!  the face that asks Git as it goes (`lfs_upload_plan.rs`)
//!  and the one that awaits every question (`lfs_upload_plan_awaiting.rs`),
//!  so the value lives beside them rather than in either,
//!  and neither face uses the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type UploadPlan = { remotes: string[]; refs: string[]; everything: boolean; submodules: boolean; fullScan: boolean; confirmedTrackingRefs: string[]; objects?: string[]; lfsServer?: string };
//! ```

/// `OsStr` is a borrowed piece of operating-system text,
///  and `OsString` the owned one
///  (the siblings `&str` and `String` must be UTF-8,
///  which a refspec or a path need not be).
use std::ffi::{
    OsStr,
    OsString,
};

/// What an upload sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UploadPlan = { remotes: string[]; refs: string[]; everything: boolean; submodules: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadPlan {
    /// The destinations `git lfs push` is run for,
    /// one run each:
    /// the remote name,
    /// or one URL per push URL when the
    ///  remote has several (each may keep its objects in its own place).
    pub remotes: Vec<OsString>,
    /// The refs whose objects are uploaded;
    /// empty with `everything`.
    pub refs: Vec<OsString>,
    /// Upload everything reachable from every local ref.
    pub everything: bool,
    /// The push also pushes submodules,
    /// whose objects this upload does not cover.
    pub submodules: bool,
    /// Scan every object reachable from the refs (`--all`) instead of the ones the remote's tracking refs do not cover.
    ///  Set for every destination that is no ordinary configured remote,
    ///  for a mirror push,
    /// and when the destination does not confirm every one of this clone's tracking refs
    ///  (its own skip list matches a cached ref by branch name alone).
    pub full_scan: bool,
    /// The full names of this clone's tracking refs the destination reports at the commit held here.
    ///  The commits an upload may stop at,
    ///  and only those:
    /// an empty list reads the whole pushed history.
    /// Empty for a destination that was not asked,
    /// for one that confirmed none of them,
    /// and for a
    ///  mirror push.
    pub confirmed_tracking_refs: Vec<String>,
    /// The exact objects to upload (`git-lfs push --object-id`),
    /// when the wrapper found them itself;
    /// `None` leaves the
    ///  scan to `git-lfs` with the refs.
    pub objects: Option<Vec<String>>,
    /// The server the upload is connected to,
    /// as its host and port with any userinfo left out,
    /// when the configuration names one:
    /// what a stop that held the connection names,
    /// so its reader knows which server did not answer.
    pub lfs_server: Option<String>,
}

/// How many object names one `git-lfs push --object-id` run takes,
/// so the argument list stays far under the system limit.
pub const OBJECTS_PER_RUN: usize = 500;

impl UploadPlan {
    /// The argument lists of the runs for one destination:
    /// one run that scans the refs,
    /// or,
    /// with named objects,
    /// one
    ///  `push --object-id <destination> <oid>...` run per [`OBJECTS_PER_RUN`] objects.
    pub fn runs(
        &self,
        remote: &OsStr,
    ) -> Vec<Vec<OsString>> {
        let Some(objects) = self.objects.as_ref() else {
            return vec![self.arguments(remote)];
        };
        let mut runs: Vec<Vec<OsString>> = Vec::new();
        for chunk in objects.chunks(OBJECTS_PER_RUN) {
            let mut arguments: Vec<OsString> = vec![
                OsString::from("push"),
                OsString::from("--object-id"),
                remote.to_owned(),
            ];
            arguments.extend(chunk.iter().map(OsString::from));
            runs.push(arguments);
        }
        return runs;
    }

    /// The arguments of one `git-lfs` run:
    /// `push`,
    /// with `--all` for a full scan,
    /// the destination,
    /// then the refs.
    ///  `--all` with refs uploads every object reachable from them,
    /// so objects that remote-tracking refs of this clone
    ///  mention are not skipped;
    /// without it `git-lfs` skips them,
    /// and a push that adds no object sends no batch request.
    pub fn arguments(
        &self,
        remote: &OsStr,
    ) -> Vec<OsString> {
        let mut arguments: Vec<OsString> = vec![OsString::from("push")];
        if self.full_scan {
            arguments.push(OsString::from("--all"));
        }
        arguments.push(remote.to_owned());
        if !self.everything {
            arguments.extend(self.refs.iter().cloned());
        }
        return arguments;
    }
}
