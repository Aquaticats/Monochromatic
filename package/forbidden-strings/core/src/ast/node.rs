//! What:
//!     The derivative-regex node algebra.
//! Why:
//!      This file is the Rust module that groups the node implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module node: see exported functions and types below.
//! ```

/// What:
///     Imports the byte-set leaf type used by `Class` nodes.
/// Why:
///      The code below uses `ByteSet` directly;
///  importing from `crate/charset` keeps each
///          call site focused on the matcher logic instead of the full Rust path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ByteSet } from "crate/charset";
/// ```
use crate::charset::ByteSet;

/// A regular expression as a value that derivatives transform.
///
/// What:
///  a small algebra closed under Brzozowski derivatives,
///  including the
/// boolean operators `Inter` (intersection) and `Comp` (complement) plus the
/// distinguished constants `Empty` (epsilon),
///  `Fail` (the empty language),
///  and
/// `Top` (every string).
///  Why:
///  derivatives map each variant to another node,
///  so
/// the same type represents both the parsed pattern and every residual state;
/// `Eq`/`Ord`/`Hash` let smart constructors dedup and sort,
///  and let the DFA
/// builder intern states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Node =
///   | { kind: "variant" };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    /// What:
    ///     Matches the empty string (epsilon).
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Empty,
    /// What:
    ///     Matches nothing (the empty language);
    ///  arises as a dead derivative.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Fail,
    /// What:
    ///     Matches every string (sigma star);
    ///  arises from complement and as the
    ///          unanchored-search prefix.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Top,
    /// What:
    ///     Matches one byte drawn from the set.
    ///  What:
    ///  literals,
    ///  `.`,
    ///  `[...]`,
    ///  and the
    ///          shorthands all reduce to this.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Class(
        /// What:
        ///     Byte set the matched byte must belong to.
        /// Why:
        ///      The surrounding function uses this step to keep the matcher behavior
        ///          correct at this point.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
        /// ```
        ByteSet,
    ),
    /// What:
    ///     Sequence:
    ///  each child matches in order.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Concat(
        /// What:
        ///     Ordered factors matched one after another.
        /// Why:
        ///      The surrounding function uses this step to keep the matcher behavior
        ///          correct at this point.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
        /// ```
        Vec<Node>,
    ),
    /// What:
    ///     Alternation:
    ///  any one child matches.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Alt(
        /// What:
        ///     Branches,
        ///  any one of which may match.
        /// Why:
        ///      The surrounding function uses this step to keep the matcher behavior
        ///          correct at this point.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
        /// ```
        Vec<Node>,
    ),
    /// What:
    ///     Intersection:
    ///  every child must match the same input.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Inter(
        /// What:
        ///     Operands that must all match the same input.
        /// Why:
        ///      The surrounding function uses this step to keep the matcher behavior
        ///          correct at this point.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
        /// ```
        Vec<Node>,
    ),
    /// What:
    ///     Complement:
    ///  matches exactly the strings the child does not.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Comp(
        /// What:
        ///     Inner expression whose language is complemented.
        /// Why:
        ///      The surrounding function uses this step to keep the matcher behavior
        ///          correct at this point.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
        /// ```
        Box<Node>,
    ),
    /// Bounded repetition:
    ///  match the child between `min` and `max` times.
    ///
    /// What:
    ///  kept un-unrolled so a counter register,
    ///  not DFA states,
    ///  carries the
    /// count.
    ///  Why:
    ///  unrolling `{n,m}` bakes the count into the automaton and blows
    /// it up exponentially under unanchored search.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    Repeat {
        /// What:
        ///     Repeated sub-expression.
        /// Why:
        ///      `node` stores repeated sub-expression,
        ///  so matcher code reads that
        ///          precomputed state by name instead of recomputing or passing it separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// node: Node;
        /// ```
        node: Box<Node>,
        /// What:
        ///     Minimum number of repetitions still required.
        /// Why:
        ///      `min` stores minimum number of repetitions still required,
        ///  so matcher code
        ///          reads that precomputed state by name instead of recomputing or passing it
        ///          separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// min: number;
        /// ```
        min: usize,
        /// What:
        ///     Maximum number of repetitions still allowed.
        /// Why:
        ///      `max` stores maximum number of repetitions still allowed,
        ///  so matcher code
        ///          reads that precomputed state by name instead of recomputing or passing it
        ///          separately.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// max: number;
        /// ```
        max: usize,
    },
    /// What:
    ///     Zero-width `^`:
    ///  nullable only at a line start.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    LineStart,
    /// What:
    ///     Zero-width `$`:
    ///  nullable only at a line end.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    LineEnd,
    /// What:
    ///     Zero-width `\b`:
    ///  nullable only where word-ness changes.
    /// Why:
    ///      The surrounding function uses this step to keep the matcher behavior correct
    ///          at this point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // Same step as the Rust statement below, written with ordinary TS objects/functions.
    /// ```
    WordBoundary,
}
