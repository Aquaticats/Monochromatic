//! What:
//!  The bounds one native journal entry is held to:
//!  the most characters of a message,
//!  the most characters of a field value,
//!  and the longest field name.
//! Why:
//!  The protocol counts characters,
//!  not encoded bytes,
//!  and bound no total,
//!  so an entry made of very many or very wide fields can still exceed what a socket takes;
//!  the send then fails with `EMSGSIZE` and the entry goes to standard error.
//!  The two caps are exempt from a citation under `LQM`,
//!  its class is a display cap:
//!  they limit how much text a log entry names,
//!  and no command result or duration depends on them
//!  (`doc/planning/cli-git-shipped-limits-audit.md`,
//!  "Exempt limits").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const MESSAGE_LIMIT = 16 * 1024; const FIELD_LIMIT = 1024; const FIELD_NAME_LIMIT = 64;
//! ```
/// The most characters of a message an entry carries;
/// longer text is cut.
/// The limits count characters,
/// not encoded bytes,
/// and bound no total,
/// so an entry made of very many or very wide fields can still exceed what a socket takes;
/// the send then
/// fails with `EMSGSIZE` and the entry goes to standard error,
/// as every entry that cannot be sent does.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const MESSAGE_LIMIT: usize = 16 * 1024;

/// The most characters of a field value an entry carries.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const FIELD_LIMIT: usize = 1024;

/// The longest field name journald takes,
/// in characters.
pub const FIELD_NAME_LIMIT: usize = 64;
