//! What:
//!  The judgement of one staging file that has no lock beside it:
//!  `packed-refs.new`,
//!  too new,
//!  on a file system another machine may share,
//!  open in a process,
//!  or removable on the proof that Git writes that file only while it holds the lock beside it.
//! Why:
//!  A staging file is the one thing a killed Git leaves behind that may be removed on a proof of its own,
//!  and that proof is a different one from the proof about a lock:
//!  the lock's absence is the caller's to have checked,
//!  and this file says whether the half-written write is safe to clear.
//!  It is judged by the same judge and the same records as a lock,
//!  so the two kinds of file are told apart by the proof and never by the words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const examined = judge.examineStaging(stagingPath);
//! ```

/// The process table's report of one file's users.
use crate::process_use::UseScan;
/// What the pid file beside the lock says,
/// which a staging file never has.
use crate::stale_ref_lock_found::PidFile;
/// What a judgement concluded and why a lock may be removed.
use crate::stale_ref_lock_found::{
    Proof,
    Verdict,
};
/// The file's identity,
/// and the reader that takes it.
use crate::stale_ref_lock_identity::{
    Identity,
    identity_of,
};
/// The judge,
/// what one judgement found,
/// and the results it is built from.
use crate::stale_ref_lock_judge::{
    Examined,
    Judge,
    verdict_with,
};
/// The later of the two times a file carries.
use crate::stale_ref_lock_judge_environment::newest_time;
/// `Path` is a borrowed filesystem path,
/// and not the owned `PathBuf`.
use std::path::Path;
/// `SystemTime` is a wall-clock instant,
/// and not the monotonic `Instant`.
use std::time::SystemTime;

impl Judge {
    /// What:
    ///  Judge the staging file `packed-refs.new` at `staging`,
    ///  which has no lock beside it:
    ///  old enough,
    ///  on a local disk,
    ///  and
    ///   open in no process.
    /// Why:
    ///  The caller has checked that the lock is absent,
    ///  and this is what is left to prove before the half-written write may be removed:
    ///  a file a live Git is still filling is held open,
    ///  and one on a shared file system may belong to a Git on another machine.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.examineStaging(staging): Examined | undefined
    /// ```
    pub fn examine_staging(
        &self,
        staging: &Path,
    ) -> Option<Examined> {
        let identity: Identity = identity_of(staging)?;
        let born: Option<SystemTime> = std::fs::symlink_metadata(staging).ok()?.created().ok();
        let newest: SystemTime = newest_time(born, &identity);
        if SystemTime::now().duration_since(newest).unwrap_or_default() < self.margin {
            return verdict_with(Verdict::TooNew, PidFile::Absent, identity, born);
        }
        if let Err(reason) = self.local_check(staging) {
            return verdict_with(Verdict::Unproven(reason), PidFile::Absent, identity, born);
        }
        let scan: UseScan = self.users_of(&identity);
        if let Some(reason) = scan.unavailable.as_ref() {
            return verdict_with(
                Verdict::Unproven(format!("the process table could not be listed: {reason}")),
                PidFile::Absent,
                identity,
                born,
            );
        }
        if let Some(user) = scan.users.first() {
            return verdict_with(
                Verdict::Held(format!(
                    "process {} ({}) has the file open",
                    user.pid,
                    user.command.clone().unwrap_or_default()
                )),
                PidFile::Absent,
                identity,
                born,
            );
        }
        if let Some(reason) = scan.unresolved.first() {
            return verdict_with(
                Verdict::Unproven(format!(
                    "a descriptor whose record names this file could not be copied out of the process that holds it ({reason}), so it may hold the file"
                )),
                PidFile::Absent,
                identity,
                born,
            );
        }
        return verdict_with(
            Verdict::Stale(Proof::OrphanStaging),
            PidFile::Absent,
            identity,
            born,
        );
    }
}
