//! What:
//!  How one reference lock is judged: too new, held, stale on a proof, or unproven.
//! Why:
//!  See `stale_ref_locks.rs` and `SPEC.md`, "Reference locks left by a Git that was killed".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = judge.verdict(lockFacts);
//! ```

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Why a lock may be removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Proof {
    /// The process the pid file names is gone.
    PidGone(i64),
    /// The process number now belongs to a program that is not Git.
    PidReused {
        /// The number the pid file names.
        pid: i64,
        /// The command name of the process that has it now.
        command: String,
    },
    /// The lock was born before the current boot of the machine.
    BeforeBoot,
}

/// What a judgement concluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// Younger than the margin, or not a regular file: left, and not mentioned.
    TooNew,
    /// A running Git holds it; the text names the process and how that is known.
    Held(String),
    /// Nothing proves the holder gone; the text says why.
    Unproven(String),
    /// Removable, on this proof.
    Stale(Proof),
}

/// The evidence a judgement may use, and the limits of what it can see.
#[derive(Clone, Debug)]
pub struct Judge {
    /// The process table.
    pub proc_root: PathBuf,
    /// How old a lock must be before it is judged at all.
    pub margin: Duration,
    /// Whether the process numbers read here mean the same processes as in the pid files.
    pub pid_view_complete: bool,
    /// When the machine booted, where known.
    pub boot: Option<SystemTime>,
    /// The file system types a lock may sit on.
    pub local_file_systems: Vec<String>,
}

impl Judge {
    /// The judge of this process: the real process table and the platform's view of it.
    pub fn here() -> Self {
        return Self {
            proc_root: PathBuf::from("/proc"),
            margin: Duration::from_secs(2),
            pid_view_complete: false,
            boot: None,
            local_file_systems: ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs"].iter().map(|name| { return String::from(*name); }).collect(),
        };
    }
}
