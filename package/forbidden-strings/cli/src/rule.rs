//! Rule compilation for the forbidden-strings scanner.
//!
//! After the engine swap (#385 teardown) this module carries only the
//! forbidden-regex rule compiler.
//!  The resharp/`regex`-crate pipeline that once
//! lived beside it (set-algebra pre-validators,
//!  the aho-corasick shard
//! architecture,
//!  the engine-routing predicate,
//!  and their loaders) was deleted
//! once the binary stopped reaching it;
//!  the live load and scan paths are
//! `crate::frx_load` and `crate::frx_scan`,
//!  both built on `crate::rule::frx`.

/// Registers the `frx` child module:
///  the forbidden-regex rule compiler.
///
/// `pub` so `lib.rs` can re-export its entry points as crate-public API.
///  It owns
/// the two-form rule-file format,
///  the flag policy,
///  UTF-8 BOM stripping,
///  redacted
/// load-error reporting,
///  and both construction paths (from rule text and from a
/// serialized precompiled `RegexSet`).
pub mod frx;
