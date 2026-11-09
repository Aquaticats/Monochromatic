//! What:
//!  The staging start that puts unmerged files into a commit's own index.
//! Why:
//!  Its names are `:(top,literal)` pathspecs the wrapper wrote,
//! so no pathspec mode the caller set may reach them:
//!  under literal mode they would match nothing,
//! and two glob modes at once make Git refuse the start and fail the commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(unmergedStagingRequest(cwd, index, objects, ['sub/f']).ownPathspecs).toBe(true);
//! ```

use super::unmerged_staging_request;
use crate::conflict_side_modes::MODE_VARIABLES;
use crate::transaction_git::GitRequest;
use std::ffi::OsString;
use std::path::Path;

/// Every name the start carries is a pathspec the wrapper wrote,
/// so the request declares that and removes the four mode variables,
/// instead of appending one `--no-...` flag for one of them.
#[test]
fn the_unmerged_staging_start_declares_its_pathspecs_the_wrapper_own() {
    let request: GitRequest = unmerged_staging_request(Path::new("sub"), Path::new(".git/commit.index"), Path::new(".git/objects"), &[b"sub/f".to_vec()]);
    assert!(request.own_pathspecs, "the pathspecs are the wrapper's own");
    assert!(request.without_prefix, "the start names its repository by its directory");
    let arguments: Vec<String> = request.arguments.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect();
    assert_eq!(
        arguments,
        vec![String::from("add"), String::from("--all"), String::from("--"), String::from(":(top,literal)sub/f")],
        "one top-anchored literal spec per unmerged path, and no mode flag to forget"
    );
    for name in MODE_VARIABLES {
        assert!(request.unset.contains(&OsString::from(name)), "{name} is removed from the child's environment");
    }
}
