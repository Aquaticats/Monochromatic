//! What:
//!     What a built [`RegexSet`] holds,
//!  reported as counts:
//!  how many rules,
//!  anchored rules,
//!  line-start rules,
//!  literal-free rules,
//!  union groups,
//!  and positions in
//!  the counting union.
//! Why:
//!      This file is the Rust module that groups the ruleset counts,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module ruleset_diagnostics: see the impl block below.
//! ```

/// What:
///     Imports the ruleset type these counts read.
/// Why:
///      The code below uses `RegexSet` directly;
///  importing from `./super` keeps each call
///          site focused on the counting logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegexSet } from "./super";
/// ```
use super::RegexSet;

/// What:
///     Reporting how many of each thing a built ruleset holds.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl RegexSet {
    /// Returns the position count of the seedless counting union,
    ///  or zero when absent.
    ///
    /// What:
    ///  how many NFA positions the single counting pass carries.
    ///  Why:
    ///  a
    /// diagnostic for the per-byte cost of the counting union against the DFA groups.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seedless_union_size(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn seedless_union_size(&self) -> usize {
        return match &self.seedless_union {
            Some(nfa) => nfa.elements.len(),
            None => 0,
        };
    }

    /// Returns how many seeded rules have an anchored DFA fast-check.
    ///
    /// What:
    ///  the count of rules whose seed is their leading literal.
    ///  Why:
    ///  a diagnostic
    /// for how much of the gate fallback avoids the slow counting scan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function anchored_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn anchored_count(&self) -> usize {
        let mut count: usize = 0;
        for engine in &self.anchored {
            if engine.is_some() {
                count += 1;
            }
        }
        return count;
    }

    /// Returns how many union DFAs the seedless rules collapsed into.
    ///
    /// What:
    ///  the group count,
    ///  counting a rule whose union exceeded the group state cap as its own
    /// engine.
    ///  Why:
    ///  a diagnostic for how well the literal-free rules
    /// combine;
    ///  zero means every literal-free rule folded into the one gate pass,
    ///  and
    ///  a count that grew by the over-cap rules shows the cap costing passes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seedless_group_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn seedless_group_count(&self) -> usize {
        return self.seedless_groups.len();
    }

    /// Returns how many `^`-anchored rules are checked at line starts.
    ///
    /// What:
    ///  the line-start rule count.
    ///  Why:
    ///  a diagnostic for the fold,
    ///  since these
    /// rules left the per-line scan for a cheap anchored check at line starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function line_start_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn line_start_count(&self) -> usize {
        return self.line_start.len();
    }

    /// Returns how many rules have no required-literal prefilter.
    ///
    /// What:
    ///  counts engines whose seed set is empty.
    ///  Why:
    ///  a diagnostic for tuning
    /// the prefilter,
    ///  since seedless rules run against every line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function seedless_count(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn seedless_count(&self) -> usize {
        let mut count: usize = 0;
        for engine in &self.rules {
            if engine.seeds().is_none() {
                count += 1;
            }
        }
        return count;
    }

    /// Returns the number of rules.
    ///
    /// What:
    ///  the rule count.
    ///  Why:
    ///  callers index `matches` results against rules.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function len(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn len(&self) -> usize {
        return self.rules.len();
    }

    /// Reports whether the set has no rules.
    ///
    /// What:
    ///  rule count is zero.
    ///  Why:
    ///  the conventional companion to `len`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function is_empty(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        return self.rules.is_empty();
    }
}
