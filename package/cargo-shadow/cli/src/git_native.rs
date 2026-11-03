//! What:
//! The choice of the trust base made from the repository's files without starting Git:
//! in CI `HEAD`;
//! locally `@{upstream}`,
//! else `origin/main`,
//! else `origin/master`,
//! else `HEAD`.
//! Why:
//! The owner's rules are "Invoke real git as rarely as possible."
//! and "embed instead of spawning"
//! (`package/git-policy/cli/SPEC.md`);
//! `git_refs.rs` reads the files and this module applies the choice,
//! so each file stays under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-native.ts
//! export function nativeBase(root: string, inCi: boolean, environment: NativeEnvironment): NativeBase  // throws: ask Git
//! ```

/// What:
/// `use std::path::Path;` imports `Path`,
/// the borrowed view of a file path (sibling:
/// `PathBuf`,
/// the owned one).
/// Why:
/// The repository root is lent to `native_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;

/// What:
/// `use crate::git_config::ParsedConfig;` imports the parsed configuration type.
/// Why:
/// `native_base` reads the configuration once and passes it to the upstream lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ParsedConfig } from 'crate/git_config';
/// ```
use crate::git_config::ParsedConfig;
/// What:
/// `use crate::git_refs::{GitLayout, Head, NativeBase, NativeEnvironment, find_layout, lookup_ref, lookup_short, read_config, read_head, tracking_ref};` imports the file readers this choice is made from.
/// Why:
/// Each step of the choice (layout,
/// configuration,
/// `HEAD`,
/// upstream,
/// short names) is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { GitLayout, Head, NativeBase, NativeEnvironment, findLayout, lookupRef, lookupShort, readConfig, readHead, trackingRef } from 'crate/git_refs';
/// ```
use crate::git_refs::{GitLayout, Head, NativeBase, NativeEnvironment, find_layout, lookup_ref, lookup_short, read_config, read_head, tracking_ref};

/// What:
/// `fn head_base(layout) -> NativeBase` is `HEAD` as a base:
/// its spelling and the commit it names.
/// Why:
/// CI compares with `HEAD`,
/// and a local run falls back to it when no upstream or remote default branch exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function headBase(layout: GitLayout): NativeBase  // throws: ask Git
/// ```
fn head_base(layout: &GitLayout) -> Result<NativeBase, String> {
    let commit: Result<String, String> = match read_head(layout)? {
        Head::Detached(object) => Ok(object),
        Head::Branch(name) => match lookup_ref(layout, &name)? {
            Some(object) => Ok(object),
            // An unborn branch names no commit, which Git reports as an error too.
            None => Err(crate::git_base::not_a_commit_message("HEAD")),
        },
    };
    return Ok(NativeBase { spelling: "HEAD".to_string(), commit });
}

/// What:
/// `pub fn native_base(root, in_ci, environment) -> Result<NativeBase, String>` chooses the trust base without Git:
/// in CI `HEAD`;
/// locally `@{upstream}`,
/// else `origin/main`,
/// else `origin/master`,
/// else `HEAD`.
/// `Err(reason)` means Git must answer.
/// Why:
/// This is the choice `git_base::resolve_trust_base` makes with `git rev-parse`,
/// made from the files (delegated pick of
/// 2026-11-03,
/// with the owner's rule "Invoke real git as rarely as possible."
/// from 2026-11-01).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeBase(root: string, inCi: boolean, environment: NativeEnvironment): NativeBase  // throws: ask Git
/// ```
pub fn native_base(root: &Path, in_ci: bool, environment: &NativeEnvironment) -> Result<NativeBase, String> {
    let layout: GitLayout = find_layout(root, environment)?;
    // The configuration is read even for CI's `HEAD`, because a reftable store or an include changes what `HEAD` means.
    let config: ParsedConfig = read_config(&layout, environment)?;
    if in_ci {
        return head_base(&layout);
    }
    if let Head::Branch(full) = read_head(&layout)?
        && let Some(branch) = full.strip_prefix("refs/heads/")
        && let Some(tracking) = tracking_ref(&config, branch)?
        && let Some(object) = lookup_ref(&layout, &tracking)?
    {
        return Ok(NativeBase { spelling: "@{upstream}".to_string(), commit: Ok(object) });
    }
    for candidate in ["origin/main", "origin/master"] {
        if let Some(object) = lookup_short(&layout, candidate)? {
            return Ok(NativeBase { spelling: candidate.to_string(), commit: Ok(object) });
        }
    }
    return head_base(&layout);
}
