//! What:
//!  The staging start that puts every unmerged file's working copy into the commit's own index.
//! Why:
//!  An unmerged file has no working copy of its own to add until this start writes one,
//! so the step is the preparation's own start and is named beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const request = ownPathspecs(indexedRequest(cwd, ['add', '--all', '--', ...paths.map((path) => `:(top,literal)${path}`)], index, objects));
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
