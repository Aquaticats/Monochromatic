//! What:
//!  Whether a unit test may go on to make a fixture whose name is not valid UTF-8,
//!  with this run's own fixture root.
//! Why:
//!  The decision and the tmpfs a refusal leads to live in one place
//!  (`super::fixture_root_taking_any_name`,
//!  whose doc says how the re-execution works),
//!  so this wrapper only supplies the root the unit tests use:
//!  the directory under the build output that `CLI_GIT_TEST_ROOT` moves.
//!  A control that writes such a name proves nothing where the file system refuses it,
//!  and it must not pass by returning early either.
//!  This file is declared by the unit tests alone,
//!  because the binary controls' own crate includes test support by path
//!  and has a fixture root of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!inAFixtureRootTakingAnyName('worktree::worktree_audit::tests::a_pointer_to_a_directory_that_is_not_text_names_it')) return;
//! ```

use super::fixture_root_taking_any_name::in_a_root_taking_any_name;
use super::test_support::fixture_root;

/// What:
///  Whether this run may go on to make a fixture whose name is not valid UTF-8,
///  starting the same test inside a namespace of its own where this run's fixture root refuses such a name.
/// Why:
///   `false` means this run is the outer half,
///  and the inner one has already run and passed,
///  or this host lets no unprivileged user namespace and the run said so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inAFixtureRootTakingAnyName(test: string): boolean;
/// ```
pub(crate) fn in_a_fixture_root_taking_any_name(test: &str) -> bool {
    return in_a_root_taking_any_name(fixture_root().as_path(), test);
}

/// Controls for the wrapper above.
#[path = "fixture_root_taking_any_name_of_unit_tests_tests.rs"]
mod tests;
