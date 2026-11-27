//! The records and the words of one adoption:
//! what the policy hands the adopter,
//! what the manager's answer becomes,
//! and the wording a report shows.

/// The limits the scope carries.
use crate::inherited_limits::InheritedLimits;
/// Why a process is kept outside the tunnel,
/// which the scope's description names.
use crate::update_entry_points::MatchReason;

/// Why a move that met another watcher's origin-scope sweep is left for the next scan,
/// in the words the log line carries.
pub const ADOPTION_DEFERRED: &str = "another watcher was stopping empty origin scopes just then, and a stopped scope ends every process in it, so the process stayed where it was and the next scan moves it";

/// One adoption of a process group into the scope of its origin.
pub struct AdoptRequest<'a> {
    /// Processes to move,
    /// the matched process first.
    pub pids: &'a [i32],
    /// User whose manager adopts them.
    pub uid: u32,
    /// Slice that receives the scope.
    pub slice: &'a str,
    /// Limits a new scope carries,
    /// read from the origin cgroup the processes leave.
    pub limits: &'a InheritedLimits,
    /// Name of the scope of this origin,
    /// as `origin_scope_names` resolves a scope that is already running under an earlier spelling.
    pub scope_name: &'a str,
    /// The cgroup the processes leave,
    /// which the scope's description names.
    pub origin: &'a str,
    /// The method that matched them,
    /// which the scope's description names.
    pub reason: MatchReason,
    /// Start time of the first process as it was read when it was matched,
    /// so a pid that another process took over later counts as gone.
    /// `None` skips that check.
    pub first_start_time: Option<u64>,
}

/// Whether a process is established as gone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Presence {
    /// The entry is absent,
    /// the process is a zombie or exiting,
    /// or the pid belongs to another process.
    Gone,
    /// Its state was read and it is running.
    Alive,
    /// Its state could not be read or understood.
    Unknown,
}

/// New limits for a scope that is running.
pub struct LimitRequest<'a> {
    /// User whose manager owns the scope.
    pub uid: u32,
    /// Slice that holds the scope.
    pub slice: &'a str,
    /// Name of the scope.
    pub scope_name: &'a str,
    /// The limits it gets.
    pub limits: &'a InheritedLimits,
}

/// How the processes got into the scope.
#[derive(Debug, Eq, PartialEq)]
pub enum Joined {
    /// The scope did not exist and was created with the origin's limits.
    Created,
    /// The scope existed and the processes were attached to it.
    Attached,
}

/// Why an adoption did not happen.
#[derive(Debug, Eq, PartialEq)]
pub enum AdoptError {
    /// The first process exited before it could be moved,
    /// or was exiting and was left alone,
    /// which needs no report.
    Vanished,
    /// The manager answered "No such process" while the first process was not established as gone
    /// and another process of the group was.
    /// The group is collected again on the next scan,
    /// so a few in a row need no report,
    /// and the policy reports a run of them.
    MemberVanished,
    /// The scope ended before the limits could be applied,
    /// which needs no report.
    ScopeGone,
    /// Another watcher was stopping empty origin scopes just then,
    /// and the stop of a scope ends every process in it,
    /// so the process is left where it was and the next scan moves it.
    Deferred,
    /// The manager refused or the move did not show.
    Failed(
        /// The reason.
        String,
    ),
    /// The client ran past its timeout and was killed.
    /// The manager may be hung.
    TimedOut(
        /// The reason.
        String,
    ),
}

/// Wording for reports.
impl std::fmt::Display for AdoptError {
    /// Writes the plain text of the failure.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            AdoptError::Vanished => {
                write!(formatter, "the process exited before it could be moved")
            }
            AdoptError::MemberVanished => write!(
                formatter,
                "a process of the group exited before the group could be moved"
            ),
            AdoptError::ScopeGone => write!(
                formatter,
                "the scope ended before its limits could be applied"
            ),
            AdoptError::Deferred => write!(formatter, "{ADOPTION_DEFERRED}"),
            AdoptError::Failed(text) | AdoptError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}
