//! What:
//!  The ways the controls start real Git in a fixture:
//! one that answers its output,
//! one that requires it to have succeeded,
//! and one that keeps a bound on the query and answers,
//!  for the last status the wait returned,
//! whether that status was the signal this helper sends at the bound.
//! Why:
//!  Every control that needs a repository,
//!  a commit or an answer from Git starts it the same way,
//! with no system or global
//!  configuration and a fixed identity,
//!  so nothing of the account's own configuration reaches a fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync(REAL_GIT, args, { cwd, env: { PATH: '/usr/bin:/bin', GIT_CONFIG_NOSYSTEM: '1', ... } })
//! ```
//!
//! The parts of this module,
//! each using only the parts listed before it:
//!
//! - `fixture_git_query_bounds.rs`:
//!   every bound one of these queries is held to,
//!   beside the measurement that sizes it.
//! - `fixture_git_commands_with_no_account_setting.rs`:
//!   the two ways a fixture starts real Git,
//!   with none of the account's configuration.
//! - `fixture_git_query_ending.rs`:
//!   what ended a query,
//!   and the seam a control places an ending in.
//! - `fixture_git_query_answer.rs`:
//!   what one bounded query answered.
//! - `fixture_git_pipes_read_beside_the_query.rs`:
//!   one pipe of a query read on a thread of its own,
//!   and the kernel's answer about it.
//! - `fixture_git_bounded_query.rs`:
//!   the wait itself,
//!   under a bound this helper keeps.

/// What:
///  Every bound one of these queries is held to,
///  beside the measurement or the bound of this crate that sizes it.
/// Why:
///  The wait below reads all of them,
/// and a control reads the ones its own assertion is measured against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { FIXTURE_GIT_BOUND, PIPE_GRACE } from './fixture_git_query_bounds'
/// ```
#[path = "fixture_git_query_bounds.rs"]
mod query_bounds;

/// What:
///  The two ways a fixture starts real Git:
///  one that answers its output and one that requires success.
/// Why:
///  Many controls need a repository or a commit and nothing of the account's configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { git, gitOutput } from './fixture_git_commands_with_no_account_setting'
/// ```
#[path = "fixture_git_commands_with_no_account_setting.rs"]
mod with_no_account_setting;

/// What:
///  What ended one bounded query,
///  and the seam a control arms to place an ending before the bound's signal.
/// Why:
///  A caller answers each ending with its own sentence,
/// so the ending it reads has to be the last status the wait returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { QueryEnding, armTheEndingSeam } from './fixture_git_query_ending'
/// ```
#[path = "fixture_git_query_ending.rs"]
mod query_ending;

/// What:
///  What one bounded query answered:
///  its status and bytes,
/// its ending,
/// and what a descendant still holds.
/// Why:
///  A caller and a control both read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { QueryAnswer } from './fixture_git_query_answer'
/// ```
#[path = "fixture_git_query_answer.rs"]
mod query_answer;

/// What:
///  One pipe of a query,
/// read on a thread of its own,
///  and the kernel's own answer about that pipe.
/// Why:
///  A pipe a descendant holds must not hold the answer past its bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { draining, hasEndedWithin, theReaderHandedOverWhatItRead } from './fixture_git_pipes_read_beside_the_query'
/// ```
#[path = "fixture_git_pipes_read_beside_the_query.rs"]
mod pipes_read_beside_the_query;

/// What:
///  The one bounded query itself:
///  it starts the query in a group of its own,
///  watches the clock,
///  and at the bound ends that group and reaps the query.
/// Why:
///  The bound is this helper's own promise,
/// so this helper keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { realGitWithin } from './fixture_git_bounded_query'
/// ```
#[path = "fixture_git_bounded_query.rs"]
mod bounded_query;

/// What:
///  The one bounded query.
/// Why:
///  A control that runs Git needs an answer with a bound,
/// not a wait that can last for ever.
pub(crate) use bounded_query::real_git_within;
/// What:
///  What a query answered.
/// Why:
///  A caller and a control both read it.
pub(crate) use query_answer::QueryAnswer;
/// What:
///  How long a query this helper killed may take to be reaped.
/// Why:
///  The controls that place an ending inside the bound's window read it.
#[cfg(test)]
use query_bounds::REAPING;
/// What:
///  The two bounds a caller reads:
/// how long a query may run,
/// and how long a pipe gets to end.
/// Why:
///  A caller sizes its own assertion from them,
/// so they travel with the query they bind.
pub(crate) use query_bounds::{
    FIXTURE_GIT_BOUND,
    PIPE_GRACE,
};
/// What:
///  What ended a query.
/// Why:
///  A caller answers each ending with its own sentence,
/// so it reads the ending itself.
pub(crate) use query_ending::QueryEnding;
/// What:
///  The seam a control arms and disarms,
/// which exists only in a build with controls.
/// Why:
///  A control about which ending is reported places the ending itself.
#[cfg(test)]
pub(crate) use query_ending::{
    arm_the_ending_seam,
    disarm_the_ending_seam,
};
/// What:
///  The two paths of a fixture that run real Git:
/// one that answers and one that requires success.
/// Why:
///  `crate::test_support` re-exports these names,
/// which many controls call.
pub(crate) use with_no_account_setting::{
    git,
    git_output,
};

/// The controls of the bounded query above.
#[cfg(test)]
#[path = "fixture_git_commands_tests.rs"]
mod tests;
