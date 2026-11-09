//! What:
//!  The staging start that puts the working copy of every unmerged file into the commit's own index.
//! Why:
//!   A preparation whose commit names no paths stages each unmerged file as `git add` would,
//!  and the start is a Git command of its own that the preparation only makes once:
//!  a file of its own keeps the preparation under its line bound
//!  (`service_prepare.rs`,
//!  "unmergedFiles").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await git(['add', '--all', '--', ...paths.map((path) => `:(top,literal)${path}`)], { index, objectDirectory });
//! ```

use crate::private_index_request::indexed_request;
use crate::transaction_git::GitRequest;
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The staging start that puts the working copy of every unmerged file into the commit's own index,
/// each named with `:(top,literal)` magic.
/// Why:
///   Every name is a pathspec the wrapper wrote,
/// and this start skips the caller's prefix,
/// so every mode the caller set is removed from the child's environment:
/// under literal mode the names would match nothing,
/// and two glob modes at once make Git refuse the start and fail the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unmergedStagingRequest = (cwd, index, objects, paths) => ownPathspecs(indexedRequest(cwd, ['add', '--all', '--', ...paths.map((path) => `:(top,literal)${path}`)], index, objects));
/// ```
pub(crate) fn unmerged_staging_request(cwd: &Path, index: &Path, object_directory: &Path, paths: &[Vec<u8>]) -> GitRequest {
    let mut staging: Vec<OsString> = vec![
        OsString::from("add"),
        OsString::from("--all"),
        OsString::from("--"),
    ];
    for path in paths {
        let mut spec: Vec<u8> = b":(top,literal)".to_vec();
        spec.extend_from_slice(path.as_slice());
        staging.push(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(spec));
    }
    return indexed_request(cwd, staging.as_slice(), Some(index), Some(object_directory)).own_pathspecs();
}
