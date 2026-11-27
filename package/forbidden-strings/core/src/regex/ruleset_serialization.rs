//! What:
//!     Persisting a built [`RegexSet`] to bytes and loading one back,
//!  including the
//!  structural check that rejects a decoded set whose parallel vectors disagree.
//! Why:
//!      This file is the Rust module that groups the ruleset (de)serialization,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module ruleset_serialization: see the impl block below.
//! ```

/// What:
///     Imports the ruleset type this module encodes and decodes.
/// Why:
///      The code below uses `RegexSet` directly;
///  importing from `./super` keeps each call
///          site focused on the decoding logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet } from "./super";
/// ```
use super::RegexSet;

/// What:
///     Imports the range test an attribution id list must pass.
/// Why:
///      The code below uses `has_id_at_least` directly;
///  importing from `./line_checks` keeps
///          each call site focused on the decoding logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { has_id_at_least } from "./line_checks";
/// ```
use super::line_checks::has_id_at_least;

/// What:
///     Imports the error type a failed decode returns.
/// Why:
///      The code below uses `CompileError` directly;
///  importing from `crate/error` keeps each
///          call site focused on the decoding logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompileError } from "crate/error";
/// ```
use crate::error::CompileError;

/// What:
///     Encoding a ruleset to bytes and decoding one back.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl RegexSet {
    /// Checks the decoded set's parallel-vector indices are mutually consistent.
    ///
    /// What:
    ///  `anchored` must be one per rule,
    ///  the line-start engines and ids must pair
    /// up,
    ///  and every attribution id (`seedless_ids`,
    ///  `line_start_ids`) must index a real
    /// rule.
    ///  Why:
    ///  matching indexes `anchored[rule]`/`rules[id]` from the gate and these
    /// id lists,
    ///  so a hostile or corrupt serialization with mismatched lengths would
    /// read out of bounds;
    ///  this rejects it before any match runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function validate_structure(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn validate_structure(&self) -> Result<(), CompileError> {
        let rules = self.rules.len();
        if self.anchored.len() != rules {
            return Err(CompileError::invalid(
                "anchored length does not match rule count",
            ));
        }
        if self.line_start.len() != self.line_start_ids.len() {
            return Err(CompileError::invalid(
                "line-start engines and ids length mismatch",
            ));
        }
        if has_id_at_least(self.seedless_ids.iter().chain(&self.line_start_ids), rules) {
            return Err(CompileError::invalid("attribution id out of range"));
        }
        return Ok(());
    }

    /// Serializes the compiled ruleset to bytes.
    ///
    /// What:
    ///  bincode-encodes every rule engine.
    ///  Why:
    ///  the pre-serialized form the
    /// throughput benchmark loads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function to_bytes(): number[] {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>, CompileError> {
        return bincode::serialize(self).map_err(CompileError::serialize_failed);
    }

    /// Loads a compiled ruleset from bytes,
    ///  validating every engine.
    ///
    /// What:
    ///  decodes then validates each rule engine.
    ///  Why:
    ///  every engine is
    /// executed,
    ///  so all must be proven in-bounds before use.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function from_bytes(bytes: Uint8Array): RegexSet {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<RegexSet, CompileError> {
        let mut set: RegexSet = bincode::deserialize(bytes).map_err(CompileError::undecodable)?;
        // What: prove the cross-vector indices are consistent before anything indexes
        // them. Why: the per-engine validation below cannot catch a decoded set whose
        // parallel vectors disagree (e.g. a `seedless_id` past `rules`, or `anchored`
        // shorter than `rules`), which would panic at match time on hostile bytes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        set.validate_structure()?;
        // What: validate the decoded graph, then rebuild its runtime prefilter. Why:
        // both must happen before any engine runs on untrusted input.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        for engine in set
            .rules
            .iter_mut()
            .chain(set.seedless_groups.iter_mut())
            .chain(set.line_start.iter_mut())
            .chain(set.anchored.iter_mut().flatten())
        {
            engine.validate()?;
            engine.prepare();
        }
        // What: validate the decoded counting union before it runs. Why: it is
        // executed against untrusted input like every other back-end.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        if let Some(nfa) = &set.seedless_union {
            nfa.validate()?;
        }
        // What: rebuild the combined gate from the prepared engines. Why: it is not
        // serialized, and the fast path depends on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        set.prepare();
        return Ok(set);
    }
}
