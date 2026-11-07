//! What:
//! The three small wordings of a cargo update that `update.rs` asks about:
//! whether one argument is `--precise`,
//! whether one is `--breaking`,
//! and how a raised catalog floor is worded as a warning line.
//! Why:
//! `update.rs` holds the whole update transaction and stays under the file-size limit by keeping these
//! pure text functions in a file of their own;
//! they use nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const isPreciseOption = (argument: string) => argument === '--precise' || argument.startsWith('--precise=');
//! ```

/// What:
/// `fn is_precise_option(argument: &str) -> bool` tells whether one cargo argument is `--precise` in either spelling (`--precise x` or `--precise=x`).
/// Why:
/// An update that pins an exact version must not have another version set aside,
/// so this run skips the set-aside step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPreciseOption = (argument: string) => argument === '--precise' || argument.startsWith('--precise=');
/// ```
pub(crate) fn is_precise_option(argument: &str) -> bool {
    return argument == "--precise" || argument.starts_with("--precise=");
}

/// What:
/// `fn is_breaking_option(argument: &str) -> bool` tells whether one cargo argument is `--breaking`.
/// Why:
/// Only a `--breaking` update raises the catalog floors,
/// as `pnpm update --latest` raises the ranges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isBreakingOption = (argument: string) => argument === '--breaking';
/// ```
pub(crate) fn is_breaking_option(argument: &str) -> bool {
    return argument == "--breaking";
}

/// What:
/// `fn raised_floor_warning(change: &str) -> String` words one raised floor as a warning line.
/// Why:
/// The user sees which catalog floors the `--breaking` update raised.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const raisedFloorWarning = (change: string) => `Raised the catalog floor ${change}`;
/// ```
pub(crate) fn raised_floor_warning(change: &str) -> String {
    return format!("Raised the catalog floor {change}");
}
