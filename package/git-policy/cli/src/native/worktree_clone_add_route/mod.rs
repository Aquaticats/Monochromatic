//! What:
//!  The execution half of the ZFS add route: from the claimed slot to a verified
//!  fill.
//! Why:
//!  The registration half beside this folder owns the record family, the
//!  managed lock's words, the finder, the recipe and the resume states
//!  (`package/git-policy/cli/src/native/worktree_clone_binding_record/`).
//!  This folder owns what the wrapper does with them: the gate that keeps the
//!  route on the design's checkout dataset, the preflight, the claim, Git's own
//!  `${git worktree add}` with the managed lock, the scan, the snapshot and
//!  clone with their inventories, the mount request, the frozen proof, the
//!  leave-outs, the private index, the pointer's exchange, the check of the
//!  result, and the resume a later add makes
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`, A0 to A12;
//!  `doc/planning/cli-git-zfs-add-time-git-lock-design.md`).
//!  Plan step 10 builds it; the fallback that finishes a failed add with Git's
//!  own checkout, and the conversion fence's writes, are step 11's.
//!
//! Files in this folder,
//! each using only the files listed before it:
//!
//! - `add_route_gate.rs`: which add the route serves, and its facts.
//! - `add_route_plan.rs`: the names it chooses, its request, and the test-only stop switch.
//! - `add_capture_tree.rs`: one descriptor-relative walk of tracked paths.
//! - `add_capture.rs`: the frozen capture's facts and the walk that takes them.
//! - `add_capture_record.rs`: the capture's durable grammar and its refusals.
//! - `add_capture_store.rs`: where the capture and the proof marker live, and how they are published.
//! - `add_frozen_proof.rs`: the clone's index and tracked files against the capture.
//! - `add_index.rs`: the fill's private index and its publication.
//! - `add_git_words.rs`: the locked add's arguments and the person's own-lock refusal.
//! - `add_registration.rs`: the scan of `<common>/worktrees` and what it settles.
//! - `add_zfs.rs`: the pool's reads, the snapshot, the clone, the inventories and the mount request.
//! - `add_grants.rs`: the account's effective delegation, refused unless it is the decided one.
//! - `add_mount.rs`: the wait for the mount and its verification in this namespace.
//! - `add_exchange.rs`: the pointer's exchange with the copied Git administration.
//! - `add_fill.rs`: the proof, the leave-outs, the index, the exchange and the check in order.
//! - `add_route_steps_facts_and_refusal.rs`: the facts every step is given, the refusal a step stops with, and the one record write.
//! - `add_route_steps_slot_and_registration.rs`: the empty slot's identity and Git's registration with the scan that settles it.
//! - `add_route_steps_snapshot_and_capture.rs`: the frozen capture and the checkout snapshot it precedes.
//! - `add_route_steps.rs`: one step per phase, each reading what a kill left.
//! - `add_source_the_restore_cannot_prove.rs`: the source shapes the fill's restore would refuse.
//! - `add_preflight.rs`: what A0 proves before the claim writes anything.
//! - `add_route_run.rs`: the entry point, the claim, the phase machine and the end.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finished = runAZfsAdd(realGit, args, overlay, facts); // null: the ordinary add
//! ```

/// Which add the route serves, and the facts every step uses.
pub mod add_route_gate;
/// The names one add chooses, its request, and the stop switch the guest arms use.
pub mod add_route_plan;
/// One descriptor-relative walk of many tracked paths.
pub mod add_capture_tree;
/// The frozen capture and the walk that takes it.
pub mod add_capture;
/// The capture's durable record.
pub mod add_capture_record;
/// Where the capture and the proof's marker live.
pub mod add_capture_store;
/// The frozen proof of a mounted clone.
pub mod add_frozen_proof;
/// The fill's private index.
pub mod add_index;
/// The Git runner the fill's own restore steps start.
pub mod add_route_git;
/// The words of the one locked Git invocation.
pub mod add_git_words;
/// The scan of the registrations and what it settles.
pub mod add_registration;
/// The pool's calls.
pub mod add_zfs;
/// The account's effective delegation over the route's two datasets.
pub mod add_grants;
/// The wait for the mount.
pub mod add_mount;
/// The pointer's exchange.
pub mod add_exchange;
/// The fill's order.
pub mod add_fill;
/// One step per phase.
pub mod add_route_steps;
/// The source shapes the fill's restore would refuse after the clone is made.
pub mod add_source_the_restore_cannot_prove;
/// What A0 proves before the claim writes anything.
pub mod add_preflight;
/// The route's entry point and runner.
pub mod add_route_run;

pub use add_fill::{FillInput, Filled, fill_the_clone};
pub use add_frozen_proof::prove_the_clone;
pub use add_route_gate::{CheckoutMount, RouteFacts, decide_the_routes_add, destination_is_the_routes_slot, the_routes_add};
pub use add_route_plan::{AddNames, STOP_AFTER_VARIABLE, STOP_STEPS, Stops, name_the_add, request_of};
pub use add_route_run::{FILLED_NOTE, run_a_zfs_add};
pub use add_capture::{CapturedEntry, CapturedIndex, CapturedKind, Capture, CaptureInput, capture_the_source};
pub use add_capture_record::{CAPTURE_HEADER, capture_bound, encode_capture, parse_capture};
pub use add_capture_store::{capture_name, marker_name, publish_capture, read_capture, remove_capture};
pub use add_git_words::{ROUTE_OPTIONS, person_lock_word, the_locked_add_words};
pub use add_grants::{read_the_grants, refuse_unless_the_decided_ones, refuse_without_the_decided_grants};
pub use add_registration::{Registration, ScanInput, ScanOutcome, scan_and_settle};
