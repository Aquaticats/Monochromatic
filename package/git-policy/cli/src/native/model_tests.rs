//! What:
//!  The test target of the commits-never-fail protocol models (`doc/planning/cli-git-commits-never-fail.md`, step 1
//!  of "Verification plan and build order"): an exhaustive schedule explorer and one model per finding of the sixth review.
//! Why:
//!  The models are verification code and ship nothing; each file is declared here by path so the library never compiles them.
//!  Each finding is a red control: the checker finds the review's schedule under the revision-6 rule and finds none
//!  under the revision-7 rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // import './model_explore'; import './model_explore_tests';
//! ```
#![cfg(unix)]

/// The explorer under control and every model.
#[path = "model_explore.rs"]
mod model_explore;

/// The explorer's own controls.
#[path = "model_explore_tests.rs"]
mod model_explore_tests;

/// Finding F7: the same identity with different content from two clients.
#[path = "model_f7_identity.rs"]
mod model_f7_identity;

/// The red control of finding F7.
#[path = "model_f7_tests.rs"]
mod model_f7_tests;
