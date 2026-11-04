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

/// Finding F2 and the seventh review's F2A and F2B: owner lifetime, descendants and the new owner's cleanup.
#[path = "model_f2_lifetime.rs"]
mod model_f2_lifetime;

/// The controls of finding F2.
#[path = "model_f2_tests.rs"]
mod model_f2_tests;

/// Finding F3: restarting a quiet owner against hook admission, restarter death, a wedged gate holder, PID namespaces.
#[path = "model_f3_restart.rs"]
mod model_f3_restart;

/// The controls of finding F3.
#[path = "model_f3_tests.rs"]
mod model_f3_tests;

/// Finding F4: cancelling a stalled `update-ref` and cleaning up its lock.
#[path = "model_f4_ref_lock.rs"]
mod model_f4_ref_lock;

/// The controls of finding F4.
#[path = "model_f4_tests.rs"]
mod model_f4_tests;

/// Finding F5: the attachment guard on the first-try and deferred index paths.
#[path = "model_f5_attachment.rs"]
mod model_f5_attachment;

/// The controls of finding F5.
#[path = "model_f5_tests.rs"]
mod model_f5_tests;

/// Finding F6: consistent capture against the lane's own mutations.
#[path = "model_f6_capture.rs"]
mod model_f6_capture;

/// The controls of finding F6.
#[path = "model_f6_tests.rs"]
mod model_f6_tests;

/// Finding F8: maintenance coordination across worktrees of one object store.
#[path = "model_f8_maintenance.rs"]
mod model_f8_maintenance;

/// The controls of finding F8.
#[path = "model_f8_tests.rs"]
mod model_f8_tests;

/// Finding F9: the hook-bypass ordering model.
#[path = "model_f9_hook_bypass.rs"]
mod model_f9_hook_bypass;

/// The controls of finding F9.
#[path = "model_f9_tests.rs"]
mod model_f9_tests;

/// Finding F1: publication evidence and the order recovery reads it in.
#[path = "model_f1_evidence.rs"]
mod model_f1_evidence;

/// The controls of finding F1.
#[path = "model_f1_tests.rs"]
mod model_f1_tests;

/// Disposable directories and a hermetic Git runner for the fixtures of steps 3 and 4.
#[path = "model_fixture.rs"]
mod model_fixture;

/// Step 3: the immutable request, spool, identity binding and permanent ledger components, with no reclamation.
#[path = "../commit/commit_components_tests.rs"]
mod commit_components_tests;

/// Step 4: pure tree comparison and captured-delta replay, differential against Git.
#[path = "../commit/commit_replay_tests.rs"]
mod commit_replay_tests;

/// R8-2: two lanes, one repairer.
#[path = "model_ledger_lanes.rs"]
mod model_ledger_lanes;

/// The controls of R8-2.
#[path = "model_ledger_lanes_tests.rs"]
mod model_ledger_lanes_tests;

/// R8-8: M1 killed at each boundary.
#[path = "model_m1_crash.rs"]
mod model_m1_crash;

/// The controls of the M1 crash boundaries.
#[path = "model_m1_crash_tests.rs"]
mod model_m1_crash_tests;

/// R8-8: merge refusal, lost swap and concurrent edits.
#[path = "model_merge.rs"]
mod model_merge;

/// The controls of merges.
#[path = "model_merge_tests.rs"]
mod model_merge_tests;

/// R9-8: E1 classification of a hook refusal followed by a foreign move.
#[path = "model_e1_classify.rs"]
mod model_e1_classify;

/// The controls of E1.
#[path = "model_e1_classify_tests.rs"]
mod model_e1_classify_tests;

/// The observation vocabulary (visible, durable, evidence) and its demonstration model.
#[path = "model_observe.rs"]
mod model_observe;

/// The controls of the observation vocabulary.
#[path = "model_observe_tests.rs"]
mod model_observe_tests;

/// Ordering and race controls that need the test seams (compiled only with the feature `test-seams`).
#[cfg(feature = "test-seams")]
#[path = "../commit/commit_seam_tests.rs"]
mod commit_seam_tests;

/// Revision 11, rule B for the foreground wait: W1 against P2 and the decision race.
#[path = "model_w1_p2.rs"]
mod model_w1_p2;

/// The controls of W1 against P2.
#[path = "model_w1_p2_tests.rs"]
mod model_w1_p2_tests;

/// Revision 11, rule B: the register of waits.
#[path = "model_register.rs"]
mod model_register;

/// The controls of the register.
#[path = "model_register_tests.rs"]
mod model_register_tests;

/// The bounded process runner of the fixtures.
#[path = "model_fixture_tests.rs"]
mod model_fixture_tests;

/// Rule V (revision 12): one-file requests, the double-buffered state, the departure barrier and the sync count of the waiting path.
#[cfg(feature = "test-seams")]
#[path = "../commit/commit_inbox_tests.rs"]
mod commit_inbox_tests;

/// Revision 13's hash-chained state log with checkpoint rotation.
#[cfg(feature = "test-seams")]
#[path = "../commit/commit_state_log_tests.rs"]
mod commit_state_log_tests;

/// The detached helper launch: no inherited descriptor, a new session, no program start.
#[cfg(target_os = "linux")]
#[path = "../commit/commit_helper_launch_tests.rs"]
mod commit_helper_launch_tests;

/// The one-shot receipt flusher: the lock, the recheck sweep and the error record.
#[cfg(target_os = "linux")]
#[path = "../commit/commit_flusher_tests.rs"]
mod commit_flusher_tests;

/// Revision 13's W1 against publication: the pipe, the mark, E1's three kinds, PREPARED-HOLD and a transaction process that outlives its owner.
#[path = "model_w1_r13.rs"]
mod model_w1_r13;

/// The controls of revision 13's W1 against publication.
#[path = "model_w1_r13_tests.rs"]
mod model_w1_r13_tests;

/// Revision 13's register of waits with a clock, failing and stalling stages, and the one-deadline budget.
#[path = "model_register_r13.rs"]
mod model_register_r13;

/// The controls of revision 13's register.
#[path = "model_register_r13_tests.rs"]
mod model_register_r13_tests;

/// Revision 18's effective ordered base: the support of a deferred selection as the fold of the live entries in tail order.
#[path = "model_support_r18.rs"]
mod model_support_r18;

/// The controls of revision 18's effective ordered base.
#[path = "model_support_r18_tests.rs"]
mod model_support_r18_tests;

/// Revision 18's receipt responsibility: the receipt directory as registration, the seal, the final sweep and the one-shot flusher.
#[path = "model_receipt_r18.rs"]
mod model_receipt_r18;

/// The controls of revision 18's receipt responsibility.
#[path = "model_receipt_r18_tests.rs"]
mod model_receipt_r18_tests;

/// Decision 19's fresh attempt with a stored notice, after a power loss with no decisive positive authority.
#[path = "model_fresh_attempt_r19.rs"]
mod model_fresh_attempt_r19;

/// The controls of decision 19's fresh attempt.
#[path = "model_fresh_attempt_r19_tests.rs"]
mod model_fresh_attempt_r19_tests;

/// The coverage map of the revision 18 acceptance target, checked against the tests it names.
#[path = "model_target_tests.rs"]
mod model_target_tests;

/// Rule V's barriers (departure, reclamation, preservation) at the filesystem's level of detail.
#[path = "model_barrier_r13.rs"]
mod model_barrier_r13;

/// The controls of rule V's barriers.
#[path = "model_barrier_r13_tests.rs"]
mod model_barrier_r13_tests;

/// Capture against a withdrawn base, the deferred merge install against later staging, and the lagging file's three-way capture.
#[path = "model_capture_r13.rs"]
mod model_capture_r13;

/// The controls of the capture models.
#[path = "model_capture_r13_tests.rs"]
mod model_capture_r13_tests;

/// `attach-watch` with MG6c and T1's registry comparison.
#[path = "model_watch_r13.rs"]
mod model_watch_r13;

/// The controls of `attach-watch` and T1's registry comparison.
#[path = "model_watch_r13_tests.rs"]
mod model_watch_r13_tests;

/// P2a's pipe bootstrap and discipline, and the syncer's separation from the owner's loop.
#[path = "model_pipe_r13.rs"]
mod model_pipe_r13;

/// The controls of the pipe discipline.
#[path = "model_pipe_r13_tests.rs"]
mod model_pipe_r13_tests;

/// The evidence labels of the model modules.
#[path = "model_labels_tests.rs"]
mod model_labels_tests;

/// M1's lock handling with comparison and removal as two steps.
#[path = "model_m1_split_r13.rs"]
mod model_m1_split_r13;

/// The controls of M1's split compare and unlink.
#[path = "model_m1_split_r13_tests.rs"]
mod model_m1_split_r13_tests;

/// Revision 14: dependency validation at recovery with discharge by a terminal outcome.
#[path = "model_discharge_r14.rs"]
mod model_discharge_r14;

/// The controls of dependency discharge.
#[path = "model_discharge_r14_tests.rs"]
mod model_discharge_r14_tests;

/// Revision 14: C2's `bound` (client lock before the link).
#[path = "model_bound_r14.rs"]
mod model_bound_r14;

/// The controls of C2's `bound`.
#[path = "model_bound_r14_tests.rs"]
mod model_bound_r14_tests;

/// Revision 14: MG2's plain staged capture during the lag, the stat condition and MG3's overlapping versions.
#[path = "model_merge_r14.rs"]
mod model_merge_r14;

/// The controls of MG2 and MG3.
#[path = "model_merge_r14_tests.rs"]
mod model_merge_r14_tests;

/// Revision 15: the submission window of P2b and what a drain may infer from the FIFO.
#[path = "model_submission_r15.rs"]
mod model_submission_r15;

/// The controls of the submission window.
#[path = "model_submission_r15_tests.rs"]
mod model_submission_r15_tests;
