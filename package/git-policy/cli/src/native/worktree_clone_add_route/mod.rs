//! What:
//!  The execution half of the ZFS add route:
//!  from the claimed slot to a verified
//!  fill.
//! Why:
//!  The registration half beside this folder owns the record family,
//!  the managed lock's words,
//!  the finder,
//!  the recipe and the resume states
//!  (`package/git-policy/cli/src/native/worktree_clone_binding_record/`).
//!  This folder owns what the wrapper does with them:
//!  the gate that keeps the
//!  route on the design's checkout dataset,
//!  the preflight,
//!  the claim,
//!  Git's own `${git worktree add}` with the managed lock,
//!  the scan,
//!  the snapshot and
//!  clone with their inventories,
//!  the mount request,
//!  the frozen proof,
//!  the leave-outs,
//!  the private index,
//!  the pointer's exchange,
//!  the check of the
//!  result,
//!  and the resume a later add makes
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  A0 to A12;
//!  `doc/planning/cli-git-zfs-add-time-git-lock-design.md`).
//!  Plan step 10 builds it;
//!  the fallback that finishes a failed add with Git's
//!  own checkout,
//!  and the conversion fence's writes,
//!  are step 11's.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `add_capture_tree.rs`:
//!   one descriptor-relative walk of tracked paths.
//! - `add_capture.rs`:
//!   the frozen capture's facts and the walk that takes them.
//! - `add_capture_record_bound.rs`:
//!   the record's header,
//!   keys and widths,
//!   and the bound they name.
//! - `add_capture_record_fields.rs`:
//!   one field of the record's grammar and the refusal that names it.
//! - `add_capture_record.rs`:
//!   the record's bytes in order and its reading.
//! - `add_capture_store.rs`:
//!   where the capture and the proof marker live,
//!   and how they are published.
//! - `add_exchange_trash.rs`:
//!   the trash the exchange keeps the copied administration in,
//!   and its own manifest.
//! - `add_exchange_preparation.rs`:
//!   the trash entry the saved pointer takes,
//!   and the identities on both sides.
//! - `add_exchange.rs`:
//!   the one swap of the two entries,
//!   and the check of what each holds after it.
//! - `add_frozen_proof.rs`:
//!   the clone's index and tracked files against the capture.
//! - `add_git_words.rs`:
//!   the locked add's arguments and the person's own-lock refusal.
//! - `add_index.rs`:
//!   the fill's private index and its publication.
//! - `add_mount.rs`:
//!   the wait for the mount and its verification in this namespace.
//! - `add_registration_reads.rs`:
//!   what the scan reads:
//!   the entries of `<common>/worktrees`,
//!   a `gitdir`,
//!   one record that may be absent,
//!   and the slots the home holds.
//! - `add_registration_lock.rs`:
//!   the lock the adopted registration owes,
//!   settled or refused.
//! - `add_registration_clearing.rs`:
//!   one incomplete registration cleared,
//!   and the names a person reads.
//! - `add_registration.rs`:
//!   the scan of `<common>/worktrees` and what it settles.
//! - `add_route_gate.rs`:
//!   which add the route serves,
//!   and its facts.
//! - `add_route_git.rs`:
//!   the Git runner the fill's own restore steps start.
//! - `add_route_plan.rs`:
//!   the names it chooses,
//!   its request,
//!   and the test-only stop switch.
//! - `add_fill_input_and_result.rs`:
//!   what one fill reads and what a filled clone hands back.
//! - `add_fill_restore.rs`:
//!   whether the restore still runs,
//!   and the reset of the clone.
//! - `add_fill_leave_outs.rs`:
//!   the leave-out's trash and the record of what it holds.
//! - `add_fill_check.rs`:
//!   the check of the filled worktree.
//! - `add_fill.rs`:
//!   the order the fill's steps hold:
//!   the proof,
//!   the marker,
//!   the index,
//!   the exchange and the end.
//! - `add_source_the_restore_cannot_prove.rs`:
//!   the source shapes the fill's restore would refuse.
//! - `add_zfs_pool_answers.rs`:
//!   what the pool answered,
//!   read back and refused:
//!   the identities,
//!   the clone's properties,
//!   the mountpoint's sentence.
//! - `add_zfs.rs`:
//!   the device opened behind the gate,
//!   and the snapshot,
//!   the clone and the mount request it sends.
//! - `add_zfs_inventories.rs`:
//!   the origin's and the clone's snapshots and bookmarks.
//! - `add_plan_clone_presence.rs`:
//!   what the pool answers about the clone a standing plan records.
//! - `add_route_steps_facts_and_refusal.rs`:
//!   the facts every step is given and the refusal each stops with.
//! - `add_route_steps_slot_and_registration.rs`:
//!   the empty slot's identity and Git's registration with the scan that settles one.
//! - `add_route_steps_snapshot_and_capture.rs`:
//!   the frozen capture and the checkout snapshot taken after it.
//! - `add_route_steps.rs`:
//!   one step per phase,
//!   each reading what a kill left.
//! - `add_grants.rs`:
//!   the account's effective delegation,
//!   refused unless it is the decided one.
//! - `add_plan_stop.rs`:
//!   the stop a standing plan leaves,
//!   with the command that ends it.
//! - `add_preflight.rs`:
//!   what A0 proves before the claim writes anything.
//! - `add_route_run_record.rs`:
//!   the record's generation,
//!   a standing record proved to be this add's,
//!   its first write,
//!   and the slot identity every step relies on.
//! - `add_route_run_end.rs`:
//!   the end of one add,
//!   repairing the new worktree and handing its trash over.
//! - `add_route_run.rs`:
//!   the entry point,
//!   the claim and the phase machine.
//! - `the_removals_cancelling.rs`:
//!   the account's answer to a removal that a holder stopped,
//!   handed to the forwarded removal path from the one folder it may name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finished = runAZfsAdd(realGit, args, overlay, facts); // null: the ordinary add
//! ```

/// The frozen capture and the walk that takes it.
pub mod add_capture;
/// The capture's durable record.
pub mod add_capture_record;
/// The record's header,
/// keys and widths,
/// and the bound they name.
pub mod add_capture_record_bound;
/// One field of the record's grammar and the refusal that names it.
pub mod add_capture_record_fields;
/// Where the capture and the proof's marker live.
pub mod add_capture_store;
/// One descriptor-relative walk of many tracked paths.
pub mod add_capture_tree;
/// The pointer's exchange.
pub mod add_exchange;
/// What the exchange is given before it swaps the two entries.
pub mod add_exchange_preparation;
/// The trash that holds the copied administration after the exchange.
pub mod add_exchange_trash;
/// The fill's order.
pub mod add_fill;
/// The check of the filled worktree.
pub mod add_fill_check;
/// What one fill reads and what it hands back.
pub mod add_fill_input_and_result;
/// The leave-out's trash and its record.
pub mod add_fill_leave_outs;
/// Whether the restore still runs,
/// and the reset itself.
pub mod add_fill_restore;
/// The frozen proof of a mounted clone.
pub mod add_frozen_proof;
/// The words of the one locked Git invocation.
pub mod add_git_words;
/// The account's effective delegation over the route's two datasets.
pub mod add_grants;
/// The fill's private index.
pub mod add_index;
/// The wait for the mount.
pub mod add_mount;
/// What the pool answers about the clone a standing removal plan records.
pub mod add_plan_clone_presence;
/// The stop a standing removal plan leaves,
/// with the one command that ends it.
pub mod add_plan_stop;
/// The scan of the registrations and what it settles.
pub mod add_registration;
/// One incomplete registration cleared through the recipe.
pub mod add_registration_clearing;
/// The lock the adopted registration owes.
pub mod add_registration_lock;
/// What the scan of the registrations reads.
pub mod add_registration_reads;
/// Which add the route serves,
/// and the facts every step uses.
pub mod add_route_gate;
/// The Git runner the fill's own restore steps start.
pub mod add_route_git;
/// The names one add chooses,
/// its request,
/// and the stop switch the guest arms use.
pub mod add_route_plan;
/// One step per phase.
pub mod add_route_steps_facts_and_refusal;
/// The pool's calls.
pub mod add_zfs;
/// The origin's and the clone's snapshots and bookmarks.
pub mod add_zfs_inventories;
/// What the pool answered,
/// read back and refused.
pub mod add_zfs_pool_answers;

pub mod add_route_steps_slot_and_registration;

pub mod add_route_steps_snapshot_and_capture;

/// What A0 proves before the claim writes anything.
pub mod add_preflight;
/// The route's entry point and runner.
pub mod add_route_run;
/// The end of one add,
/// after the fill.
pub mod add_route_run_end;
/// The add's durable record.
pub mod add_route_run_record;
pub mod add_route_steps;
/// The source shapes the fill's restore would refuse after the clone is made.
pub mod add_source_the_restore_cannot_prove;

/// The account's answer to a removal that a holder stopped.
pub mod the_removals_cancelling;

pub use add_capture::{
    Capture,
    CaptureInput,
    CapturedEntry,
    CapturedIndex,
    CapturedKind,
    capture_the_source,
};
pub use add_capture_record::{
    CAPTURE_HEADER,
    capture_bound,
    encode_capture,
    parse_capture,
};
pub use add_capture_store::{
    capture_name,
    marker_name,
    publish_capture,
    read_capture,
    remove_capture,
};
pub use add_fill::{
    FillInput,
    Filled,
    fill_the_clone,
};
pub use add_frozen_proof::prove_the_clone;
pub use add_git_words::{
    ROUTE_OPTIONS,
    person_lock_word,
    the_locked_add_words,
};
pub use add_grants::{
    read_the_grants,
    refuse_unless_the_decided_ones,
    refuse_without_the_decided_grants,
};
pub use add_registration::{
    Registration,
    ScanInput,
    ScanOutcome,
    scan_and_settle,
};
pub use add_route_gate::{
    CheckoutMount,
    RouteFacts,
    decide_the_routes_add,
    destination_is_the_routes_slot,
    the_routes_add,
};
pub use add_route_plan::{
    AddNames,
    STOP_AFTER_VARIABLE,
    STOP_STEPS,
    Stops,
    name_the_add,
    request_of,
};
pub use add_route_run::{
    FILLED_NOTE,
    run_a_zfs_add,
};
pub use the_removals_cancelling::the_removals_cancelling;
