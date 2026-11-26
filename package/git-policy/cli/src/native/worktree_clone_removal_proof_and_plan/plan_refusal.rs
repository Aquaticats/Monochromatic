//! What:
//!  Why some bytes are not a destruction plan this build reads,
//!  and the sentence a person reads about the file.
//! Why:
//!  A reader's answer has to say what it read and that it changed nothing,
//!  because the project decided an unreadable record is refused,
//!  named,
//!  and left for a person
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "A record that cannot be read"),
//!  and the design requires each refusal to name the path,
//!  the field or line,
//!  and what was read
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  section 5).
//!  No refusal moves a record aside,
//!  truncates one,
//!  or writes anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class PlanRefusal extends Error { cause; field?; found; }
//! ```

use std::path::Path;

/// What:
///  The kind of thing that made bytes unreadable as a plan.
/// Why:
///  A person's next action depends on which of these it is:
///  a version this release does not know waits for a later release,
///  a torn line waits for the removal to be run again,
///  and a foreign file is not this removal's record at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cause = 'not-this-record' | 'version' | 'ends-early' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanRefusalCause {
    /// The first line is not a destruction plan's.
    NotThisRecord,
    /// The first line is a destruction plan's,
    /// with a version this build does not know.
    UnknownVersion,
    /// The file stops before the fields it declares.
    EndsEarly,
    /// The file stops before the attempt lines it declares.
    MissingAttemptLines,
    /// A key the grammar does not hold.
    UnknownKey,
    /// A key outside the fixed order.
    OutOfOrderKey,
    /// A key written twice.
    RepeatedKey,
    /// A key the grammar requires and the file lacks.
    MissingKey,
    /// A number,
    /// GUID,
    /// nonce or hexadecimal field that is not its canonical spelling.
    NotCanonical,
    /// A declared count that disagrees with the lines the file holds.
    CountDisagrees,
    /// An attempt line of another kind than `snapshot`.
    WrongAttemptKind,
    /// A word outside the five snapshot outcomes.
    UnknownOutcome,
    /// A word outside the five head states.
    UnknownHead,
    /// Two lines carrying one snapshot name and one attempt number.
    DuplicateAttempt,
    /// One snapshot's lines out of attempt order.
    AttemptsOutOfOrder,
    /// A line written for a snapshot whose history a `destroyed` line closed.
    DestroyedHistoryReopened,
    /// One line wider than the grammar's line bound.
    LineTooWide,
    /// A file longer than the declared count allows,
    /// or wider than the header bound.
    Oversized,
    /// A name that is not a plan's own:
    /// not a canonical clone GUID.
    ForeignName,
    /// A record filed under a name its `cloneGuid` does not carry.
    NameDisagreesWithTheRecord,
    /// A plan file that is not a regular file.
    NotARegularFile,
    /// A plan file that could not be read.
    Unreadable,
    /// A folder whose mode is not `0700`.
    FolderMode,
    /// A folder or record owned by another account than the common directory.
    WrongOwner,
    /// A `refused` line that carries no errno.
    MissingErrno,
    /// An errno on a line whose outcome is not a refusal.
    ErrnoOnAnotherLine,
    /// A field whose value is wider than its own bound,
    /// refused by the writer before it publishes.
    FieldTooWide,
    /// A plan the removal would write whose own reader would refuse it.
    WriterWouldNotReadItBack,
}

impl PlanRefusalCause {
    /// The clause a message prints before the field and the value.
    fn clause(self) -> &'static str {
        return match self {
            PlanRefusalCause::NotThisRecord => {
                "does not begin with this route's destruction-plan header"
            }
            PlanRefusalCause::UnknownVersion => {
                "carries a destruction-plan version this build does not read"
            }
            PlanRefusalCause::EndsEarly => "stops before the header fields it declares",
            PlanRefusalCause::MissingAttemptLines => {
                "stops before the attempt lines its header declares"
            }
            PlanRefusalCause::UnknownKey => "holds a key the plan grammar does not have",
            PlanRefusalCause::OutOfOrderKey => {
                "writes its keys out of the one order the grammar allows"
            }
            PlanRefusalCause::RepeatedKey => "writes one key twice",
            PlanRefusalCause::MissingKey => "lacks a key the grammar requires",
            PlanRefusalCause::NotCanonical => "holds a field that is not its canonical spelling",
            PlanRefusalCause::CountDisagrees => {
                "declares an attempt count that disagrees with the lines it holds"
            }
            PlanRefusalCause::WrongAttemptKind => {
                "holds an attempt line whose kind is not `snapshot`"
            }
            PlanRefusalCause::UnknownOutcome => {
                "holds an outcome word outside the five the grammar has"
            }
            PlanRefusalCause::UnknownHead => "holds a head word outside the five the grammar has",
            PlanRefusalCause::DuplicateAttempt => {
                "writes one snapshot name and attempt number twice"
            }
            PlanRefusalCause::AttemptsOutOfOrder => "writes one snapshot's attempts out of order",
            PlanRefusalCause::DestroyedHistoryReopened => {
                "writes a line for a snapshot whose history a `destroyed` line closed"
            }
            PlanRefusalCause::LineTooWide => "holds a line wider than the grammar's line bound",
            PlanRefusalCause::Oversized => "is larger than its declared attempt count allows",
            PlanRefusalCause::ForeignName => {
                "is filed under a name that is not a clone's canonical numeric GUID"
            }
            PlanRefusalCause::NameDisagreesWithTheRecord => {
                "is filed under a name its own `cloneGuid` does not carry"
            }
            PlanRefusalCause::NotARegularFile => "is not a regular file",
            PlanRefusalCause::Unreadable => "could not be read",
            PlanRefusalCause::FolderMode => "sits in a folder whose mode is not `0700`",
            PlanRefusalCause::WrongOwner => {
                "is not owned by the account that owns the common Git directory"
            }
            PlanRefusalCause::MissingErrno => "carries a `refused` line with no errno",
            PlanRefusalCause::ErrnoOnAnotherLine => {
                "carries an errno on a line whose outcome is not a refusal"
            }
            PlanRefusalCause::FieldTooWide => "holds a field wider than its own bound",
            PlanRefusalCause::WriterWouldNotReadItBack => {
                "is a record this build's own reader would refuse"
            }
        };
    }
}

/// What:
///  One refusal,
///  with the field or line it names and what was read there.
/// Why:
///  The message needs the file's path as well as the parser's finding,
///  and keeping the three together means a caller cannot print a cause without saying
///  which file and which value it came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { cause: Cause; field?: string; found: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanRefusal {
    /// What kind of unreadable this is.
    pub cause: PlanRefusalCause,
    /// The key or line the cause names,
    /// when it names one.
    pub field: Option<String>,
    /// What the file held there.
    pub found: String,
}

impl PlanRefusal {
    /// One refusal with the field it names.
    pub fn at(
        cause: PlanRefusalCause,
        field: impl Into<String>,
        found: impl Into<String>,
    ) -> PlanRefusal {
        return PlanRefusal {
            cause,
            field: Some(field.into()),
            found: found.into(),
        };
    }

    /// One refusal that names no single field.
    pub fn of(
        cause: PlanRefusalCause,
        found: impl Into<String>,
    ) -> PlanRefusal {
        return PlanRefusal {
            cause,
            field: None,
            found: found.into(),
        };
    }

    /// What:
    ///  The sentence a person reads about one plan.
    /// Why:
    ///  Every refusal says the same three things:
    ///  the file,
    ///  what was read,
    ///  and that this refusal destroyed nothing and waits for a person.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal.message(path) // "the plan at ... ; nothing was destroyed ..."
    /// ```
    pub fn message(
        &self,
        plan: &Path,
    ) -> String {
        let where_: String = match &self.field {
            Some(field) => format!("at `{field}`"),
            None => String::from("in its bytes"),
        };
        return format!(
            "the destruction plan at {} {} {where_}, where it holds `{}`. Nothing was destroyed by this refusal, and this plan is neither moved nor deleted. Ways forward: read the file and leave it in place; a removal that meets this refuses before its first delete, and a later release that knows a newer grammar reads it without changing anything.",
            plan.display(),
            self.cause.clause(),
            self.found
        );
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_refusal_tests.rs"]
mod tests;
