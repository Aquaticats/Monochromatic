//! What:
//!  The public update of one pushed reference and the limits on how many queries of a push probe run side by side.
//! Why:
//!  The probe in `manual_push_probe.rs` and the lifecycle that reads the scopes both need these;
//! they do not use the probe,
//! so
//!  they sit in a file of their own and `manual_push_probe.rs` re-exports them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type PushUpdate = { remoteName: string; remoteOid?: string; to: string; ... };
//! const MOST_CONCURRENT_REMOTE_QUERIES = 4;
//! ```


/// What:
///  One confirmed push update,
/// the public fact the policies read.
/// Why:
///   An absent value is `None`,
/// which replaces the incumbent's `ABSENT_GIT_VALUE` sentinel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushUpdate = { localOid: string | typeof ABSENT_GIT_VALUE; remoteOid: string | typeof ABSENT_GIT_VALUE; remoteName; remoteRef };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushUpdate {
    /// Local object name;
    /// `None` for a deletion.
    pub local_oid: Option<String>,
    /// The local side Git reported for the update,
    /// as the push named it:
    /// a branch,
    /// `HEAD`,
    /// or a commit expression,
    /// and empty for a deletion.
    pub from: String,
    /// Destination object name confirmed by `ls-remote`;
    /// `None` when the ref is absent.
    pub remote_oid: Option<String>,
    /// Destination remote name.
    pub remote_name: String,
    /// Destination ref.
    pub remote_ref: String,
}

/// The most per-destination queries that open a connection to a remote and run at once (`git ls-remote` of each destination):
///  the bound `SPEC.md` gives,
/// four.
/// The remote side (how many connections a server tolerates) was not measured,
/// so the value is
///  the one the SPEC already gave;
/// it is the coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// concurrent per-update queries,
/// option B).
pub const MOST_CONCURRENT_REMOTE_QUERIES: usize = 4;

/// The most per-update queries of the local repository that run at once (`git cat-file` and `git rev-list` of each update).
/// Measured
///  on 16 branch tips of this repository,
/// medians of three runs in alternating orders,
/// other sessions running:
/// `git ls-tree -r`
///  took 218,
/// 114,
/// 80,
/// 50 and 46 ms at 1,
/// 2,
/// 4,
/// 8 and 16 at once,
/// and `git rev-list` of a whole branch 845,
/// 413,
/// 231,
/// 190 and 194
///  ms,
/// so eight is where the gain stops.
/// The coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// concurrent per-update queries,
/// option B).
pub const MOST_CONCURRENT_LOCAL_QUERIES: usize = LOCAL_QUERY_GAIN_STOPS_AT;

/// The number of queries at once at which the measured gain of local queries stopped (see `MOST_CONCURRENT_LOCAL_QUERIES`).
pub const LOCAL_QUERY_GAIN_STOPS_AT: usize = 8;

/// The bytes one query holds at its peak when it lists the whole history of this repository:
/// the largest resident size of
///  `git rev-list --all` in three runs (29,164 KiB,
/// 29,032 KiB and 28,848 KiB,
/// so 29,863,936 bytes) plus the 1,413,721 bytes of
///  its output,
/// which the wrapper reads whole (34,481 commits on 2026-11-03).
/// `git ls-tree -r` of the main tree peaked at 26,080
///  KiB with 2,459,743 bytes of output.
pub const MEASURED_WHOLE_HISTORY_QUERY: u64 = 29_863_936 + 1_413_721;
