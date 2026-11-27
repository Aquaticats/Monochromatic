//! What:
//! Whether `install` may resolve plan entries the lock lacks,
//! and the refusal a frozen run prints when the plan is ahead of the lock.
//! Why:
//! It mirrors `pnpm install`:
//! resolve and write the lockfile entries a changed manifest needs when not frozen,
//! refuse when frozen (the default in CI),
//! and the refusal names the switch or the environment that froze it.
//! A caller imports from `sync` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type FrozenReason = 'flag' | 'ci';
//! export type LockPolicy = { kind: 'resolve' } | { kind: 'frozen'; reason: FrozenReason };
//! function frozenRefusal(verdict: Disagreement, reason: FrozenReason): ToolchainError;
//! ```

/// Imports the disagreement a frozen run refuses on,
/// and the refusal it joins.
use super::super::agree::{
    Disagreement,
    refusal,
};
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// What:
/// The reason a frozen `install` did not resolve a plan that is ahead of the lock.
/// Why:
/// The refusal tells the person which switch or environment stopped `install` from locking the entries itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrozenReason {
    /// `--frozen-lockfile` was given.
    Flag,
    /// A CI environment was detected (the `ci-info` rules pnpm uses).
    Ci,
}

/// What:
/// Whether `install` may resolve plan entries the lock lacks.
/// Why:
/// It mirrors `pnpm install`:
/// resolve and write the lockfile entries a changed manifest needs when not frozen,
/// refuse when frozen (the default in CI).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockPolicy = { kind: 'resolve' } | { kind: 'frozen'; reason: 'flag' | 'ci' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockPolicy {
    /// Resolve new and re-edited entries alone,
    /// write only their rows,
    /// then install.
    Resolve,
    /// Refuse a plan that is ahead of the lock and name `mpm-shadow update <ids>`;
    /// the reason says which switch or environment froze it.
    Frozen(
        /// Why the lock is frozen.
        FrozenReason,
    ),
}

/// The refusal for a frozen `install` whose plan is ahead of the lock:
/// the usual disagreement message plus why `install` did not resolve.
pub(super) fn frozen_refusal(
    verdict: &Disagreement,
    reason: FrozenReason,
) -> ToolchainError {
    let base: ToolchainError = refusal(verdict);
    let why: &str = match reason {
        FrozenReason::Flag => "`--frozen-lockfile` was given",
        FrozenReason::Ci => {
            "a CI environment was detected (the CI variable or another marker, by the rules of pnpm and ci-info), \
             where the lock is frozen; `CI=false` or `--no-frozen-lockfile` turns that off"
        }
    };
    return ToolchainError::drift(format!(
        "{}\nmpm-shadow install did not resolve them itself because {why}; run `mpm-shadow update <ids>` (or `mpm-shadow install --no-frozen-lockfile`), commit the lock, then install again",
        base.message
    ));
}
