//! What:
//!  One attempt region of a destruction plan,
//!  read into one line per attempt:
//!  its seven pieces,
//!  the named snapshot,
//!  its declared count,
//!  and the two order rules.
//! Why:
//!  The line is last-named and holds a snapshot name that may carry the space the kernel's charset
//!  allows,
//!  so the seventh piece is read to the end of the line and never split again
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The grammar" and "Every transition,
//! and the rule that allows it").
//!  A gap in one name's numbers is not an error:
//!  it is the trace of a write that was lost before it landed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const attempts = parseAttempts(region, count, cloneDataset);
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::canonical_decimal::parse_canonical_u64;
use crate::worktree_clone_removal_proof_and_plan::plan_bounds::NUMBER_MAXIMUM_DIGITS;
use crate::worktree_clone_removal_proof_and_plan::plan_lines::take_line;
use crate::worktree_clone_removal_proof_and_plan::plan_name::{
    is_a_snapshot_of,
    is_snapshot_name,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    PlanAttempt,
    PlanSnapshot,
};
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    ATTEMPT_KEY,
    ATTEMPT_NAME_KEY,
    NO_ERRNO,
    SNAPSHOT_KIND,
    SnapshotOutcome,
};

/// What:
///  One attempt line,
///  its seven pieces and its named snapshot.
/// Why:
///  The name is last,
///  so the line is split into seven pieces and the seventh is read to the end of the line,
///  whatever a snapshot's name holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [kind, attempt, outcome, guid, createtxg, errno, name] = line.split(' ', 7);
/// ```
fn parse_attempt(
    line: &str,
    clone_dataset: &str,
) -> Result<PlanAttempt, PlanRefusal> {
    let Some(after_key) = line.strip_prefix(ATTEMPT_KEY) else {
        return Err(PlanRefusal::at(
            PlanRefusalCause::NotCanonical,
            "line",
            line,
        ));
    };
    let Some(words) = after_key.strip_prefix('=') else {
        return Err(PlanRefusal::at(
            PlanRefusalCause::NotCanonical,
            "line",
            line,
        ));
    };
    let mut pieces = words.splitn(7, ' ');
    let Some(kind) = pieces.next() else {
        return Err(PlanRefusal::at(
            PlanRefusalCause::WrongAttemptKind,
            "tuple",
            line,
        ));
    };
    if kind != SNAPSHOT_KIND {
        return Err(PlanRefusal::at(
            PlanRefusalCause::WrongAttemptKind,
            "tuple",
            kind,
        ));
    }
    let attempt_text: &str = pieces
        .next()
        .ok_or_else(|| return PlanRefusal::at(PlanRefusalCause::NotCanonical, "attempt", line))?;
    let outcome_text: &str = pieces
        .next()
        .ok_or_else(|| return PlanRefusal::at(PlanRefusalCause::NotCanonical, "outcome", line))?;
    let guid_text: &str = pieces
        .next()
        .ok_or_else(|| return PlanRefusal::at(PlanRefusalCause::NotCanonical, "guid", line))?;
    let createtxg_text: &str = pieces
        .next()
        .ok_or_else(|| return PlanRefusal::at(PlanRefusalCause::NotCanonical, "createtxg", line))?;
    let errno_text: &str = pieces
        .next()
        .ok_or_else(|| return PlanRefusal::at(PlanRefusalCause::NotCanonical, "errno", line))?;
    let named: &str = pieces.next().ok_or_else(|| {
        return PlanRefusal::at(PlanRefusalCause::NotCanonical, ATTEMPT_NAME_KEY, line);
    })?;
    let attempt: u64 =
        parse_canonical_u64(attempt_text, NUMBER_MAXIMUM_DIGITS).ok_or_else(|| {
            return PlanRefusal::at(PlanRefusalCause::NotCanonical, "attempt", attempt_text);
        })?;
    let outcome: SnapshotOutcome = SnapshotOutcome::parse(outcome_text).ok_or_else(|| {
        return PlanRefusal::at(PlanRefusalCause::UnknownOutcome, "outcome", outcome_text);
    })?;
    let guid: DatasetGuid = DatasetGuid::parse(guid_text)
        .map_err(|_| return PlanRefusal::at(PlanRefusalCause::NotCanonical, "guid", guid_text))?;
    let createtxg: u64 =
        parse_canonical_u64(createtxg_text, NUMBER_MAXIMUM_DIGITS).ok_or_else(|| {
            return PlanRefusal::at(PlanRefusalCause::NotCanonical, "createtxg", createtxg_text);
        })?;
    let errno: Option<u32> = match (outcome, errno_text) {
        (SnapshotOutcome::Refused, NO_ERRNO) => {
            return Err(PlanRefusal::at(
                PlanRefusalCause::MissingErrno,
                "errno",
                errno_text,
            ));
        }
        (SnapshotOutcome::Refused, digits) => Some(
            parse_canonical_u64(digits, NUMBER_MAXIMUM_DIGITS)
                .and_then(|number| return u32::try_from(number).ok())
                .ok_or_else(|| {
                    return PlanRefusal::at(PlanRefusalCause::NotCanonical, "errno", digits);
                })?,
        ),
        (_, NO_ERRNO) => None,
        (_, digits) => {
            return Err(PlanRefusal::at(
                PlanRefusalCause::ErrnoOnAnotherLine,
                "errno",
                digits,
            ));
        }
    };
    let Some(name) = named
        .strip_prefix(ATTEMPT_NAME_KEY)
        .and_then(|after| return after.strip_prefix('='))
    else {
        return Err(PlanRefusal::at(
            PlanRefusalCause::NotCanonical,
            ATTEMPT_NAME_KEY,
            named,
        ));
    };
    if !is_snapshot_name(name) || !is_a_snapshot_of(clone_dataset, name) {
        return Err(PlanRefusal::at(
            PlanRefusalCause::ForeignName,
            ATTEMPT_NAME_KEY,
            name,
        ));
    }
    return Ok(PlanAttempt {
        snapshot: PlanSnapshot {
            name: String::from(name),
            guid,
            createtxg,
        },
        attempt,
        outcome,
        errno,
    });
}

/// What:
///  Every attempt line of one region,
///  in the grammar's order and with its declared count.
/// Why:
///  The declared count is what refuses a record that ends early,
///  and the order rules are what let a later reader act on the highest line per name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempts = parseAttempts(bytes, count, cloneDataset);
/// ```
pub fn parse_attempts(
    region: &[u8],
    count: u64,
    clone_dataset: &str,
) -> Result<Vec<PlanAttempt>, PlanRefusal> {
    let text: &str = std::str::from_utf8(region).map_err(|_| {
        return PlanRefusal::at(
            PlanRefusalCause::NotCanonical,
            "attempt line",
            "bytes that are not text",
        );
    })?;
    let mut left: &str = text;
    let mut attempts: Vec<PlanAttempt> = Vec::new();
    while !left.is_empty() {
        let line: &str = take_line(&mut left, PlanRefusalCause::MissingAttemptLines)?;
        let parsed: PlanAttempt = parse_attempt(line, clone_dataset)?;
        refuse_a_broken_order(&attempts, &parsed)?;
        attempts.push(parsed);
    }
    if attempts.len() as u64 != count {
        return Err(PlanRefusal::at(
            PlanRefusalCause::CountDisagrees,
            "attempts",
            count.to_string(),
        ));
    }
    return Ok(attempts);
}

/// Whether one new line keeps the order and the merge rule the earlier lines established.
fn refuse_a_broken_order(
    earlier: &[PlanAttempt],
    next: &PlanAttempt,
) -> Result<(), PlanRefusal> {
    let Some(last) = earlier.last() else {
        return Ok(());
    };
    let order = last
        .snapshot
        .name
        .cmp(&next.snapshot.name)
        .then(last.attempt.cmp(&next.attempt));
    if order == std::cmp::Ordering::Greater {
        return Err(PlanRefusal::at(
            PlanRefusalCause::AttemptsOutOfOrder,
            ATTEMPT_NAME_KEY,
            next.snapshot.name.clone(),
        ));
    }
    if order == std::cmp::Ordering::Equal {
        return Err(PlanRefusal::at(
            PlanRefusalCause::DuplicateAttempt,
            ATTEMPT_NAME_KEY,
            next.snapshot.name.clone(),
        ));
    }
    let mut closed: bool = false;
    for line in earlier {
        if line.snapshot.name == next.snapshot.name && line.outcome == SnapshotOutcome::Destroyed {
            closed = true;
        }
    }
    if closed {
        return Err(PlanRefusal::at(
            PlanRefusalCause::DestroyedHistoryReopened,
            ATTEMPT_NAME_KEY,
            next.snapshot.name.clone(),
        ));
    }
    return Ok(());
}
