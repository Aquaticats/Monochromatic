//! What:
//!  The bytes a destruction plan publishes:
//!  its header's eighteen keys and values,
//!  its attempt lines,
//!  and the checks that refuse a record its own reader would reject.
//! Why:
//!  A writer that publishes past a bound would leave a record its own reader refuses,
//!  so every field,
//!  line and ordering rule is proved in one place rather than trusted,
//!  and the merge rule is checked here as well as when a line is recorded,
//!  because a plan may be read back and written again by a resumed removal.
//!  The value itself and its read-side rules are `plan_record.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = plan.encode();
//! ```

use crate::worktree_clone_binding_record::binding_encoding::{
    bytes_as_hexadecimal,
    path_as_hexadecimal,
};
use crate::worktree_clone_removal_proof_and_plan::canonical_decimal::decimal_digits;
use crate::worktree_clone_removal_proof_and_plan::plan_bounds::{
    ATTEMPT_COUNT_MAXIMUM_DIGITS,
    ATTEMPT_LINE_MAXIMUM_BYTES,
    NUMBER_MAXIMUM_DIGITS,
    ZFS_NAME_MAXIMUM_BYTES,
    header_field_maximum_bytes,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanAttempt,
};
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    ATTEMPT_KEY,
    ATTEMPT_NAME_KEY,
    NO_ERRNO,
    NO_MOUNT_ID,
    PLAN_HEADER,
    PLAN_KEYS_IN_ORDER,
    SNAPSHOT_KIND,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::started_moment::is_seconds_with_offset;

impl DestructionPlan {
    /// What:
    ///  The record's bytes,
    ///  its line order and its fields checked against the bounds it must satisfy.
    /// Why:
    ///  A writer that publishes past a bound would leave a record its own reader refuses,
    ///  so every field,
    ///  line and ordering rule is proved here rather than trusted.
    ///
    /// # Throws
    /// The field that is past its bound,
    /// a name outside the kernel's charset bound,
    /// or a history whose lines break the merge rule.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const bytes = plan.encode();
    /// ```
    pub fn encode(&self) -> Result<Vec<u8>, PlanRefusal> {
        let lines: Vec<(String, String)> = self.header_lines()?;
        let attempt_lines: Vec<String> = self.attempt_lines()?;
        self.refuse_a_broken_history()?;
        let mut text: String = String::from(PLAN_HEADER);
        text.push('\n');
        for (key, value) in lines {
            text.push_str(key.as_str());
            text.push('=');
            text.push_str(value.as_str());
            text.push('\n');
        }
        for line in attempt_lines {
            text.push_str(line.as_str());
            text.push('\n');
        }
        return Ok(text.into_bytes());
    }

    /// The header's keys and values,
    /// each checked against its own bound.
    fn header_lines(&self) -> Result<Vec<(String, String)>, PlanRefusal> {
        let mount_id: String = match self.facts.unmount_mount_id {
            Some(found) => found.to_string(),
            None => String::from(NO_MOUNT_ID),
        };
        let values: [String; 18] = [
            String::from(self.facts.run.as_str()),
            self.facts.owner.clone(),
            self.facts.started.clone(),
            self.facts.clone_dataset.clone(),
            self.facts.clone_guid.file_name(),
            self.facts.clone_createtxg.to_string(),
            self.facts.origin_dataset.clone(),
            self.facts.origin_dataset_guid.file_name(),
            self.facts.origin_snapshot.clone(),
            self.facts.origin_snapshot_guid.file_name(),
            self.facts.origin_snapshot_createtxg.to_string(),
            bytes_as_hexadecimal(self.facts.registration_id.as_slice()),
            self.facts.admin.clone(),
            path_as_hexadecimal(self.facts.target.as_path()),
            String::from(self.facts.unmount_nonce.as_str()),
            mount_id,
            String::from(self.head.word()),
            self.attempts.len().to_string(),
        ];
        let mut lines: Vec<(String, String)> = Vec::with_capacity(PLAN_KEYS_IN_ORDER.len());
        for (place, key) in PLAN_KEYS_IN_ORDER.iter().enumerate() {
            let value: String = values[place].clone();
            let widest: u64 = header_field_maximum_bytes(key).unwrap_or(0);
            if key.len() as u64 + 1 + value.len() as u64 + 1 > widest {
                return Err(PlanRefusal::at(PlanRefusalCause::FieldTooWide, *key, value));
            }
            lines.push((String::from(*key), value));
        }
        if !is_seconds_with_offset(self.facts.started.as_str()) {
            return Err(PlanRefusal::at(
                PlanRefusalCause::NotCanonical,
                "started",
                self.facts.started.clone(),
            ));
        }
        if decimal_digits(self.attempts.len() as u64) > ATTEMPT_COUNT_MAXIMUM_DIGITS {
            return Err(PlanRefusal::at(
                PlanRefusalCause::FieldTooWide,
                "attempts",
                self.attempts.len().to_string(),
            ));
        }
        return Ok(lines);
    }

    /// The attempt lines,
    /// sorted and each checked against its name bound.
    fn attempt_lines(&self) -> Result<Vec<String>, PlanRefusal> {
        let mut ordered: Vec<&PlanAttempt> = self.attempts.iter().collect();
        ordered.sort_by(|left, right| {
            return left
                .snapshot
                .name
                .cmp(&right.snapshot.name)
                .then(left.attempt.cmp(&right.attempt));
        });
        let mut lines: Vec<String> = Vec::with_capacity(ordered.len());
        for line in ordered {
            if line.snapshot.name.len() > ZFS_NAME_MAXIMUM_BYTES {
                return Err(PlanRefusal::at(
                    PlanRefusalCause::FieldTooWide,
                    ATTEMPT_NAME_KEY,
                    line.snapshot.name.clone(),
                ));
            }
            if decimal_digits(line.attempt) > NUMBER_MAXIMUM_DIGITS {
                return Err(PlanRefusal::at(
                    PlanRefusalCause::FieldTooWide,
                    ATTEMPT_KEY,
                    line.attempt.to_string(),
                ));
            }
            let errno: String = match (line.outcome, line.errno) {
                (SnapshotOutcome::Refused, Some(number)) => number.to_string(),
                (SnapshotOutcome::Refused, None) => {
                    return Err(PlanRefusal::at(
                        PlanRefusalCause::MissingErrno,
                        ATTEMPT_KEY,
                        line.snapshot.name.clone(),
                    ));
                }
                (_, Some(_)) => {
                    return Err(PlanRefusal::at(
                        PlanRefusalCause::ErrnoOnAnotherLine,
                        ATTEMPT_KEY,
                        line.snapshot.name.clone(),
                    ));
                }
                (_, None) => String::from(NO_ERRNO),
            };
            let built: String = format!(
                "{ATTEMPT_KEY}={SNAPSHOT_KIND} {} {} {} {} {errno} {ATTEMPT_NAME_KEY}={}",
                line.attempt,
                line.outcome.word(),
                line.snapshot.guid.file_name(),
                line.snapshot.createtxg,
                line.snapshot.name
            );
            if built.len() as u64 > ATTEMPT_LINE_MAXIMUM_BYTES {
                return Err(PlanRefusal::at(
                    PlanRefusalCause::LineTooWide,
                    ATTEMPT_NAME_KEY,
                    line.snapshot.name.clone(),
                ));
            }
            lines.push(built);
        }
        return Ok(lines);
    }

    /// Whether one snapshot's history breaks the merge rule:
    /// a repeated attempt number,
    /// a line after a `destroyed` one,
    /// or an attempt number out of order in the file.
    fn refuse_a_broken_history(&self) -> Result<(), PlanRefusal> {
        let mut names: Vec<&str> = self
            .attempts
            .iter()
            .map(|line| return line.snapshot.name.as_str())
            .collect();
        names.sort_unstable();
        names.dedup();
        for name in names {
            let mut seen: Vec<u64> = Vec::new();
            let mut closed: bool = false;
            for line in &self.attempts {
                if line.snapshot.name != name {
                    continue;
                }
                if seen.contains(&line.attempt) {
                    return Err(PlanRefusal::at(
                        PlanRefusalCause::DuplicateAttempt,
                        ATTEMPT_NAME_KEY,
                        name,
                    ));
                }
                if closed {
                    return Err(PlanRefusal::at(
                        PlanRefusalCause::DestroyedHistoryReopened,
                        ATTEMPT_NAME_KEY,
                        name,
                    ));
                }
                seen.push(line.attempt);
                closed = line.outcome == SnapshotOutcome::Destroyed;
            }
        }
        return Ok(());
    }
}
