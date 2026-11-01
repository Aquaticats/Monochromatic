//! What:
//!  The first cache-warning reason each platform reports when a regular file blocks the cache root.
//! Why:
//!   The loader maps only a not-found read error to `missing`.
//!  Operating systems disagree on which error
//!       a path below a regular file produces,
//!  and both consumer suites must assert the same platform fact.
//!       The scanner's behavior is deliberately left platform-dependent here (decision of 2026-10-05).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const blockedRootReason = process.platform === 'win32' ? 'missing' : 'unreadable';
//! ```

/// What:
///  `#[cfg(unix)]` compiles this definition only for Unix targets.
///  `-> &'static str` returns a borrowed
///       string literal that lives for the whole program (sibling:
///  `String`,
///  an owned heap copy nobody needs here).
/// Why:
///   POSIX `stat` fails with ENOTDIR when a path-prefix component is an existing file that is not a directory,
///       and Rust reports ENOTDIR as `NotADirectory`,
///  not `NotFound`,
///  so the loader calls the artifact unreadable.
///       Measured on Linux;
///  other Unix targets follow from POSIX.1-2024 and the standard library's errno mapping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blockedRootReason(): string { return 'unreadable'; }
/// ```
#[cfg(unix)]
pub(crate) fn blocked_root_reason() -> &'static str {
    return "unreadable";
}

/// What:
///  The Windows definition of the same function;
///  exactly one of the two is compiled for a target,
///       and a target that is neither Unix nor Windows fails to compile instead of asserting an unmeasured reason.
/// Why:
///   Windows reports a path below a regular file with OS error 3,
///  the path-not-found error it also gives
///       for a path below an absent directory.
///  Rust reports `NotFound`,
///  so the loader calls the artifact missing.
///       Measured on Windows Server 2025 on 2026-10-05.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blockedRootReason(): string { return 'missing'; }
/// ```
#[cfg(windows)]
pub(crate) fn blocked_root_reason() -> &'static str {
    return "missing";
}
