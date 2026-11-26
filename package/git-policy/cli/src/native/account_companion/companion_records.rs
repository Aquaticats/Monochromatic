//! What:
//!  One step's record on the companion child's status pipe,
//!  its fixed byte layout,
//!  and the parent's read of them.
//! Why:
//!  The child cannot build text without allocating,
//!  so a step's outcome crosses the pipe as four bytes of meaning in a fixed
//!  eight,
//!  and a number the parent does not know is the parent's own arm failure
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "Every ending,
//! and the line it prints").
//!  A partial record is what the design calls a status line the parent cannot
//!  parse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pipe.write(encodeRecord({ step: 'walk', answer: 'passed' })); // 8 bytes
//! ```

use super::companion_calls::{
    CompanionCall,
    Failure,
};
use super::companion_steps::CompanionStep;
use std::os::fd::RawFd;

/// How many bytes one step's record takes.
pub const RECORD_BYTES: usize = 8;

/// What:
///  One step's outcome,
///  as the child writes it and the parent reads it.
/// Why:
///  A passing step names no call,
///  and a failing one names the call that answered and its errno,
///  so the parent's line can say both without a sentence from the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Record = { step: Step; pass: true } | { step: Step; pass: false; call: Call; errno: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepRecord {
    /// The step this outcome belongs to.
    pub step: CompanionStep,
    /// The call that answered,
    /// where the step failed.
    pub failure: Option<Failure>,
}

impl StepRecord {
    /// One passing step.
    pub fn passed(step: CompanionStep) -> StepRecord {
        return StepRecord {
            step,
            failure: None,
        };
    }

    /// One failed step,
    /// with the call that refused and the number it answered.
    pub fn failed(
        step: CompanionStep,
        failure: Failure,
    ) -> StepRecord {
        return StepRecord {
            step,
            failure: Some(failure),
        };
    }

    /// The eight bytes this record is.
    pub fn encode(self) -> [u8; RECORD_BYTES] {
        let (answer, call, errno): (u8, u8, i32) = match self.failure {
            Some(failure) => (1, failure.call as u8, failure.errno),
            None => (0, 0, 0),
        };
        let mut bytes: [u8; RECORD_BYTES] = [0; RECORD_BYTES];
        bytes[0] = self.step as u8;
        bytes[1] = answer;
        bytes[2] = call;
        bytes[4..8].copy_from_slice(errno.to_le_bytes().as_slice());
        return bytes;
    }

    /// What:
    ///  The record eight bytes hold,
    ///  or nothing when they name no step or no call of this build.
    /// Why:
    ///  A child that sent a number this build does not know did not say what
    ///  happened,
    ///  which is an arm failure rather than a guess.
    pub fn decode(bytes: &[u8]) -> Option<StepRecord> {
        if bytes.len() != RECORD_BYTES {
            return None;
        }
        let step: CompanionStep = CompanionStep::from_number(bytes[0])?;
        if bytes[1] == 0 {
            return Some(StepRecord::passed(step));
        }
        let call: CompanionCall = CompanionCall::from_number(bytes[2])?;
        let errno: i32 = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        return Some(StepRecord::failed(
            step,
            Failure {
                call,
                errno,
            },
        ));
    }
}

/// What:
///  Write one step's record to the child's status pipe.
/// Why:
///  A record that cannot be written is the child's own last word,
///  so the write is best-effort and the child ends either way.
pub fn send(
    status: RawFd,
    record: StepRecord,
) {
    let bytes: [u8; RECORD_BYTES] = record.encode();
    let _written = super::companion_file_calls::write_all(
        status,
        bytes.as_slice(),
        CompanionCall::WriteRecord,
    );
}

/// What:
///  Every whole record one read of the status pipe left,
///  and whether a part of one came after them.
/// Why:
///  The parent judges a run from whole records only,
///  and a trailing part is the design's own "the parent cannot parse it".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { records, partial } = decodeAll(bytes);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decoded {
    /// The records the bytes hold.
    pub records: Vec<StepRecord>,
    /// Whether bytes were left that make no whole record,
    /// or one whole record named nothing this build knows.
    pub unreadable: bool,
}

/// What:
///  The whole records of one status read.
pub fn decode_all(bytes: &[u8]) -> Decoded {
    let mut records: Vec<StepRecord> = Vec::new();
    let mut unreadable: bool = false;
    let mut at: usize = 0;
    while at + RECORD_BYTES <= bytes.len() {
        match StepRecord::decode(&bytes[at..at + RECORD_BYTES]) {
            Some(record) => records.push(record),
            None => unreadable = true,
        }
        at += RECORD_BYTES;
    }
    if at != bytes.len() {
        unreadable = true;
    }
    return Decoded {
        records,
        unreadable,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_records_tests.rs"]
mod tests;
