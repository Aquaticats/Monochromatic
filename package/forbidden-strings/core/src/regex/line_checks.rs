//! The per-line checks of a [`RegexSet`] that the gate's candidate walk and the line scans use:
//! named types that carry a line's state in place of closures,
//! and the small loops over groups and engines that a scan repeats.

/// Imports the set these checks read.
use super::RegexSet;
/// Imports the traits a candidate walk of the gate takes.
use crate::gate::{CandidateCheck, CandidateVisit};
/// Imports the matcher engine and the line-start match.
use crate::build::line_start_match;
/// Imports the engine type a group or a line-start rule holds.
use crate::engine::Engine;

/// A stack-only set of rule ids already fully checked on the current line.
///
/// What:
///  a 256-bit set over rule ids,
///  allocation-free.
///  Why:
///  a non-anchored rule's
/// whole-line check ignores the hit position,
///  so it need run only once per line even
/// when its seed occurs many times;
///  this records which have run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckedFull = {
///   // fields documented in Rust above
/// };
/// ```
pub(super) struct CheckedFull {
    /// What:
    ///     Bit `r` set means rule `r` has already had its whole-line check this line.
    /// Why:
    ///      `bits` stores bit `r` set means rule `r` has already had its whole-line check
    ///          this line,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bits: number[];
    /// ```
    bits: [u64; 4],
}

/// What:
///     Construction and the first-seen test for the per-line dedup set.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl CheckedFull {
    /// Builds an empty set for one line.
    ///
    /// What:
    ///  all bits clear.
    ///  Why:
    ///  a fresh set per `is_match`/`matches` call.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function new(): CheckedFull {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(super) fn new() -> CheckedFull {
        return CheckedFull { bits: [0; 4] }
    }

    /// Reports whether `rule` is seen for the first time,
    ///  recording it.
    ///
    /// What:
    ///  true the first time a rule id is passed,
    ///  false after;
    ///  rule ids at or above
    /// the set's 256 capacity always report true.
    ///  Why:
    ///  the caller runs the whole-line
    /// check only on the first sighting;
    ///  beyond capacity it simply re-runs,
    ///  still sound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function first_time(rule: number): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub(super) fn first_time(&mut self, rule: usize) -> bool {
        if rule >= 256 {
            return true;
        }
        let (word, bit) = (rule / 64, 1u64 << (rule % 64));
        let fresh = self.bits[word] & bit == 0;
        self.bits[word] |= bit;
        return fresh
    }
}

/// Checks each gate candidate against the rule it names for one line,
/// remembering the rules already fully checked.
pub(super) struct RuleCheck<'set> {
    /// The set whose rules are checked.
    pub(super) set: &'set RegexSet,
    /// The line being matched.
    pub(super) line: &'set [u8],
    /// The rules without an anchored seed that this line already ran in full.
    pub(super) checked: CheckedFull,
}

/// Lets the gate stop at the first candidate whose rule matches.
impl CandidateCheck for RuleCheck<'_> {
    /// Whether the line matches the rule whose seed starts at `pos`.
    fn check(&mut self, rule: usize, pos: usize) -> bool {
        return self.set.matches_rule(self.line, rule, pos, &mut self.checked);
    }
}

/// Collects the rules among the gate candidates that match one line.
pub(super) struct RuleCollector<'set> {
    /// The set whose rules are checked.
    pub(super) set: &'set RegexSet,
    /// The line being matched.
    pub(super) line: &'set [u8],
    /// The rules without an anchored seed that this line already ran in full.
    pub(super) checked: CheckedFull,
    /// The rules that matched,
    /// in candidate order.
    pub(super) hits: Vec<usize>,
}

/// Lets the gate hand every candidate to the collector.
impl CandidateVisit for RuleCollector<'_> {
    /// Records the rule when the line matches it.
    fn visit(&mut self, rule: usize, pos: usize) {
        if self.set.matches_rule(self.line, rule, pos, &mut self.checked) {
            self.hits.push(rule);
        }
    }
}

/// Rejects every candidate,
/// which times the gate's own walk alone.
pub(super) struct RejectEveryCandidate;

/// Lets the gate walk all its candidates without ever stopping.
impl CandidateCheck for RejectEveryCandidate {
    /// Never matches.
    fn check(&mut self, _rule: usize, _pos: usize) -> bool {
        return false;
    }
}

/// Checks only the candidates whose rule has an anchored seed engine.
pub(super) struct AnchoredRuleCheck<'set> {
    /// The set whose anchored engines are checked.
    pub(super) set: &'set RegexSet,
    /// The line being matched.
    pub(super) line: &'set [u8],
}

/// Lets the gate stop at the first candidate whose anchored engine matches.
impl CandidateCheck for AnchoredRuleCheck<'_> {
    /// Whether the anchored engine of the rule matches from `pos`.
    fn check(&mut self, rule: usize, pos: usize) -> bool {
        return match &self.set.anchored[rule] {
            Some(engine) => engine.is_match(&self.line[pos..]),
            None => false,
        };
    }
}

/// Whether any line-start engine matches the line.
pub(super) fn any_line_start_match(engines: &[Engine], line: &[u8]) -> bool {
    for engine in engines {
        if line_start_match(engine, line) {
            return true;
        }
    }
    return false;
}

/// Whether any seedless group matches the line.
pub(super) fn any_group_matches(groups: &[Engine], line: &[u8]) -> bool {
    for group in groups {
        if group.is_match(line) {
            return true;
        }
    }
    return false;
}

/// Whether any attribution id is at or past the limit.
pub(super) fn has_id_at_least<'ids>(ids: impl Iterator<Item = &'ids usize>, limit: usize) -> bool {
    for &id in ids {
        if id >= limit {
            return true;
        }
    }
    return false;
}

/// Whether a trimmed piece of ruleset text holds anything.
pub(super) fn is_nonempty_text(text: &&str) -> bool {
    return !text.is_empty();
}
