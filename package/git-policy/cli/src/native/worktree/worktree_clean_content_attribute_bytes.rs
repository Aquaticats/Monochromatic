//! What:
//!  The bytes of an attributes file,
//! up to the room the caller's bound leaves,
//! taken with the read a control sets or the operating system's own,
//! each piece charged as it arrives,
//! and accepted only when the file did not move under the read.
//! Why:
//!   The bound is on the reading,
//! not on the size an earlier look reported,
//! so a file that grows after that look may not hand over a byte the caller never counted.
//! A read the system interrupts is tried again,
//! and a source that interrupts every try is refused rather than waited on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function attributeBytes(file, before, room) { /* read in chunks and charge each one */ }
//! ```

use crate::worktree_clean_content::ContentJudge;
use crate::worktree_clean_content::same_state;
use crate::worktree_path_failure_message::subject_failure;
use std::os::unix::fs::MetadataExt;

/// The bytes one attribute read asks for at most.
const ATTRIBUTES_CHUNK: usize = 64 * 1024;

/// How many times one attributes read is tried again after the system interrupts it.
/// Why:
///  An interruption is not a fact about the file,
/// so one is retried;
/// a file system that interrupts every attempt for ever would otherwise hold the landing
///  without a bound,
/// which the owner's rule forbids ("nothing should ever wait forever or indefinitely").
/// Three tries is the coordinator's pick:
/// a single signal is the interruption the retry exists for,
/// and a source that interrupts three reads in a row is one this reader refuses rather than waits on.
pub(super) const INTERRUPTED_TRIES: usize = 3;

impl<'a> ContentJudge<'a> {
    /// What:
    ///  Take attributes files with `read`,
    /// for a control that hands over what a file grown under the read would.
    /// Why:
    ///   The overshoot the review found needs a file that grows between the size check and the read,
    /// and a control cannot make that race happen without a seam.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.readAttributesWith(read);
    /// ```
    #[cfg(test)]
    pub fn read_attributes_with(
        &mut self,
        read: &'a dyn Fn(&std::fs::File, &mut [u8]) -> std::io::Result<usize>,
    ) {
        self.attribute_read = Some(read);
    }

    /// What:
    ///  The bytes of an attributes file,
    /// up to `room`,
    /// taken with the read a control sets or the operating system's own,
    /// each piece charged as it arrives,
    /// and accepted only when the file did not move under the read.
    /// Why:
    ///   The bound is on the reading,
    /// not on the size an earlier look reported:
    /// a file that grows after that look may not hand over a byte the caller
    ///  never counted,
    /// so the read is capped at the room.
    /// The proof that the bytes are one version of the file is the state `before` compared with the state after:
    /// on this host's file systems (ext4 and btrfs) a write to the same inode advances the change and modification
    ///  times,
    /// both to the nanosecond,
    /// and a user cannot set the change time back,
    /// so an unchanged state is an unchanged file;
    /// a size alone is not enough,
    /// because a rewrite whose pieces add up to the new length
    ///  passes it (the fourth review found exactly that).
    /// Where the nanoseconds are absent,
    /// the file system keeps whole seconds and a rewrite within one
    ///  of them would leave the same recorded times,
    /// so such a file is refused rather than trusted.
    /// A read the system interrupts is tried again,
    /// as the recorded-content reader already does:
    ///  an interruption is not a fact about the file,
    /// and it may not cost every remaining path of the landing its stat data.
    /// The tries are counted (`INTERRUPTED_TRIES`):
    /// a source that interrupts every one of them ends in the same refusal a hard read error does,
    /// so nothing waits for an interruption to stop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function attributeBytes(file, before, room) {
    ///   const data = []; while (data.length < room) { let taken; try { taken = read(file, new Uint8Array(Math.min(64 << 10, room - data.length))); }
    ///     catch (error) { if (error.kind === 'Interrupted') continue; throw error; } if (taken === 0) break; data.push(...taken); spent += taken; }
    ///   const after = stat(file); return after.ctimeNsec !== 0 && sameState(before, after) ? data : undefined;
    /// }
    /// ```
    pub(super) fn attribute_bytes(
        &self,
        file: &std::fs::File,
        before: &std::fs::Metadata,
        room: u64,
        label: &dyn std::fmt::Display,
    ) -> Result<Option<Vec<u8>>, String> {
        // A file whose change time has no nanoseconds is on a file system that cannot tell two rewrites in one second
        // apart, so nothing it hands over can be proven to be one version.
        if before.ctime_nsec() == 0 {
            return Ok(None);
        }
        let mut data: Vec<u8> = Vec::new();
        let mut interrupted: usize = 0;
        let mut buffer: Vec<u8> =
            vec![0; usize::try_from(room.min(ATTRIBUTES_CHUNK as u64)).unwrap_or(ATTRIBUTES_CHUNK)];
        while (data.len() as u64) < room {
            let want: usize = usize::try_from(room - data.len() as u64)
                .unwrap_or(usize::MAX)
                .min(ATTRIBUTES_CHUNK);
            let read: std::io::Result<usize> = match self.attribute_read {
                Some(read) => read(file, &mut buffer[..want]),
                None => {
                    use std::io::Read as _;
                    (&*file).take(want as u64).read(&mut buffer[..want])
                }
            };
            let taken: usize = match read {
                Ok(taken) => taken,
                // An interruption is not a fact about the file, so it is tried again;
                // a source that interrupts every try is refused rather than waited on.
                Err(error)
                    if error.kind() == std::io::ErrorKind::Interrupted
                        && interrupted < INTERRUPTED_TRIES =>
                {
                    interrupted += 1;
                    continue;
                }
                Err(error) => return Err(format!("{label} cannot be read ({error})")),
            };
            if taken == 0 {
                break;
            }
            let took: usize = taken.min(want);
            data.extend_from_slice(&buffer[..took]);
            self.charge(took as u64);
        }
        let after: std::fs::Metadata =
            subject_failure(file.metadata(), label, "cannot be examined")?;
        if !same_state(before, &after) {
            return Ok(None);
        }
        return Ok(Some(data));
    }

    /// Add `bytes` to the caller's bound,
    /// when it set one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function charge(bytes) { if (budget) budget.bytes += bytes; }
    /// ```
    pub(super) fn charge(
        &self,
        bytes: u64,
    ) {
        if let Some(spent) = self.budget {
            spent.set(spent.get() + bytes);
        }
    }
}
