//! What:
//! What counts as a release archive of this repository:
//! the `archive` route,
//! a request that names `github:Aquaticats/Monochromatic` (in any letter case),
//! and an `options.tagPrefix` that starts with `monochromatic-`.
//! Why:
//! The owner's rule ("no older repo-maintained tools should ever exist when newer is present")
//! is about this repository's tools,
//! and this repository tags every program it publishes with the same prefix it uses for its crates,
//! so the plan needs no second list that could drift from the releases.
//! The prefix alone would also match a tool of another repository that happens to start its tags the same way,
//! so the request has to name this repository too,
//! and GitHub owner and repository names are case-insensitive,
//! so letter case is not compared.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function isPublishedArchive(tool: LockedTool): boolean
//! ```

/// Imports what this module uses from `crate::constants`.
use crate::constants::{
    OWN_CRATE_PREFIX,
    OWN_REPOSITORY_SOURCE,
};

/// Imports what this module uses from `super::super::model`.
use super::super::model::{
    LockedTool,
    Route,
};

/// Whether a tag prefix starts the way every release tag of this repository does.
fn starts_with_own_crate_prefix(prefix: &str) -> bool {
    return prefix.starts_with(OWN_CRATE_PREFIX);
}

/// Whether a request source and a tag prefix name releases of this repository:
/// the source is `OWN_REPOSITORY_SOURCE` apart from letter case
/// (GitHub owner and repository names are case-insensitive,
/// and the names are ASCII),
/// and the prefix starts with `monochromatic-`.
pub(super) fn names_own_releases(
    request_source: &str,
    tag_prefix: Option<&str>,
) -> bool {
    return request_source.eq_ignore_ascii_case(OWN_REPOSITORY_SOURCE)
        && tag_prefix.is_some_and(starts_with_own_crate_prefix);
}

/// What:
/// Whether a tool is an archive-route tool published by this repository:
/// the `archive` route,
/// a request for `github:Aquaticats/Monochromatic` (in any letter case),
/// and an `options.tagPrefix` that starts with `monochromatic-`.
/// Why:
/// The tag prefix is what names one program's releases in this repository's many tags,
/// and this repository tags every program it publishes that way (`OWN_CRATE_PREFIX`,
/// the rule that already marks its crates),
/// so the plan needs no second list that could drift from the releases.
/// The prefix alone would also match a tool of another repository that happens to start its tags the same way,
/// and the owner's rule ("no older repo-maintained tools") is about this repository's tools,
/// so the request has to name this repository too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isPublishedArchive(tool: LockedTool): boolean
/// ```
pub fn is_published_archive(tool: &LockedTool) -> bool {
    if tool.declaration.route != Route::Archive {
        return false;
    }
    return names_own_releases(
        &tool.declaration.request.source,
        tool.declaration.options.tag_prefix.as_deref(),
    );
}
