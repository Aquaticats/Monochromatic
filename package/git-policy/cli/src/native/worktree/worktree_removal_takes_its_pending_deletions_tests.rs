//! What:
//!  Controls for the names of the subvolumes this program leaves for its commit service to delete,
//!  which is what a walk below a worktree reads before it decides whether an entry is its own.
//! Why:
//!  A name is no proof of what stands at it,
//!  so the walk recognizes only the one shape `deleting_name` makes,
//!  and every other entry below the root is somebody's and is refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isALeavingName('cli-git-deleting-000000000001f5f7')).toBe(true);
//! ```

use super::*;

/// What:
///  A name this program makes is recognized,
///  and every name that only looks like one is not.
/// Why:
///   A leftover is moved out of the worktree and recorded for the service,
/// and a name that is not exactly the one
///  `deleting_name` makes must stay a volume the removal refuses rather than one it moves
///  (a review finding,
/// F4).
#[test]
fn a_leaving_name_is_the_one_this_program_makes() {
    let mine: &str = "cli-git-deleting-000000000001f5f7";
    assert!(is_a_leaving_name(Path::new(mine)), "the name a hand-over makes is this program's");
    assert!(is_a_leaving_name(Path::new(format!("/w/{mine}").as_str())), "the folder above it is no part of the name");
    for other in [
        "cli-git-deleting-0000000000001f5f7",
        "cli-git-deleting-000000000001f5f",
        "cli-git-deleting-0000000000001f5g",
        "cli-git-deleting-",
        "cli-git-deleting-000000000001F5F7",
        "cli-git-deleted-000000000001f5f7",
        "removing-00000000-0000-0000-0000-000000000000",
    ] {
        assert!(!is_a_leaving_name(Path::new(other)), "{other} is not a name this program makes");
    }
}
