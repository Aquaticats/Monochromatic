//! What:
//!  Model of rule V's dependency validation at recovery (revision 14, R13-4): the records of a lane that name other records, which of
//!  them survive when requests are reclaimed and old segments are deleted, and what recovery accepts under revision 13's direct
//!  presence checks against revision 14's discharge and transitive validation. The specification is a function of the surviving set:
//!  a record is recoverable when it is present and each thing it names is recoverable or discharged by a terminal outcome.
//! Why:
//!  Revision 13 validated an `adopt` line against its request file, so retiring the request and deleting the file made the chain break
//!  and dropped every later adoption (R1 and R2 adopted in one chain, R1 retired, restart: R2 lost). It named a predecessor segment in a
//!  checkpoint, so deleting the old segment made a valid checkpoint unrecoverable. It checked a referenced record's presence and not the
//!  records that one names, so a checkpoint whose transitive dependency was absent was accepted. Revision 14 discharges a request
//!  dependency by the terminal outcome, makes the checkpoint self-contained, and validates the transitive closure.
//!  Acceptance is compared with the specification over every surviving subset, so both a false drop and a false accept are found.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const accepts = (rules, present: number, record: number): boolean => ...; // compared with spec(present, record) for all present
//! ```

/// A request file of R1.
pub const REQ1: u32 = 1;
/// A request file of R2.
pub const REQ2: u32 = 2;
/// The terminal outcome line of R1 (the ledger line).
pub const OUT1: u32 = 4;
/// The order-file adopt line of R1.
pub const ADOPT1: u32 = 8;
/// The order-file adopt line of R2, chained after R1's.
pub const ADOPT2: u32 = 16;
/// The old state segment.
pub const SEG1: u32 = 32;
/// A checkpoint.
pub const CKPT: u32 = 64;
/// A record in another file that a second checkpoint references.
pub const REFX: u32 = 128;
/// What `REFX` itself names.
pub const DEPX: u32 = 256;
/// A second checkpoint that references `REFX`.
pub const CKPT2: u32 = 512;
/// The number of facts.
pub const FACTS: u32 = 10;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 13: direct presence checks; the checkpoint names its predecessor segment.
    Thirteen,
    /// Revision 14: a terminal outcome discharges a request dependency, the checkpoint is self-contained, validation is transitive.
    Fourteen,
}

fn has(present: u32, fact: u32) -> bool {
    return present & fact != 0;
}

/// What a correct recovery accepts from the surviving set `present` (the specification).
pub fn spec(present: u32, record: u32) -> bool {
    return match record {
        ADOPT1 => has(present, ADOPT1) && (has(present, REQ1) || has(present, OUT1)),
        ADOPT2 => has(present, ADOPT2) && has(present, REQ2) && spec(present, ADOPT1),
        CKPT => has(present, CKPT),
        REFX => has(present, REFX) && has(present, DEPX),
        CKPT2 => has(present, CKPT2) && spec(present, REFX),
        _ => has(present, record),
    };
}

/// What recovery accepts under `rules`.
pub fn accepts(rules: Rules, present: u32, record: u32) -> bool {
    if rules == Rules::Fourteen {
        return spec(present, record);
    }
    return match record {
        ADOPT1 => has(present, ADOPT1) && has(present, REQ1),
        ADOPT2 => has(present, ADOPT2) && has(present, REQ2) && accepts(rules, present, ADOPT1),
        CKPT => has(present, CKPT) && has(present, SEG1),
        REFX => has(present, REFX),
        CKPT2 => has(present, CKPT2) && has(present, REFX),
        _ => has(present, record),
    };
}

/// The records whose recovery is checked.
pub const RECORDS: [u32; 5] = [ADOPT1, ADOPT2, CKPT, REFX, CKPT2];

/// Every (surviving set, record) where `rules` disagrees with the specification: a false drop (a recoverable record rejected) or a
/// false accept (an unrecoverable record trusted).
pub fn disagreements(rules: Rules) -> Vec<(u32, u32, bool)> {
    let mut found: Vec<(u32, u32, bool)> = Vec::new();
    for present in 0..(1_u32 << FACTS) {
        for record in RECORDS {
            let accepted: bool = accepts(rules, present, record);
            let wanted: bool = spec(present, record);
            if accepted != wanted {
                found.push((present, record, accepted));
            }
        }
    }
    return found;
}
