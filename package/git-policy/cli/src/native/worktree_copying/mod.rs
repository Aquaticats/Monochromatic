//! What:
//!  Copying the ignored state of a linked worktree into a new one,
//!  with staging,
//!  journal and recovery.
//! Why:
//!  The copy is a transaction of its own with many small steps.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `worktree_copy_error.rs`:
//!   the one failure type of the linked-worktree ignored-state copy and its diagnostics.
//! - `worktree_copy.rs`:
//!   the copy's contract:
//!   the facts a settlement is handed,
//!   the steps it asks its Git runner for,
//!   the settlement it answers with,
//!   and the summary line and exit status a person reads.
//! - `worktree_copy_after_lock.rs`:
//!   what is left to do once a created worktree holds its ignored state:
//!   its package-manager repair,
//!   and the trash a snapshot left there,
//!   handed to the commit service.
//! - `worktree_copy_ceiling.rs`:
//!   the bound one command holds when it finishes a copy it did not start,
//!   and the words a copy stopped at that bound is reported with.
//! - `worktree_copy_interrupted_add_settlement_checkout_proof.rs`:
//!   what proves the checkout of an interrupted `git worktree add` whole or gone:
//!   Git's index,
//!   the lock a killed Git left beside it,
//!   and what the destination holds beside its pointer.
//! - `worktree_copy_owed_copy_notices.rs`:
//!   every line the pass that finishes an owed worktree copy prints:
//!   the one that names a copy owed into another worktree,
//!   and the three that say what this command's own copy is doing.
//! - `worktree_copy_owed_copy_source.rs`:
//!   whether the worktree a copy takes its state from can still be one,
//!   and the reason a copy is given up on when it cannot.
//! - `worktree_copy_owner.rs`:
//!   the process that owns one item of a copy transaction,
//!   written into the item's name,
//!   and the proof that it still runs.
//! - `worktree_copy_publish_file_temporary_name.rs`:
//!   the stable name a fallback's temporary is filled under,
//!   computed from the stage token and the entry's path.
//! - `worktree_copy_publish_file_making_the_file.rs`:
//!   the calls that make one file in a directory held open:
//!   an unnamed file where the filesystem has one,
//!   and a file under a name this copy chose otherwise.
//! - `worktree_copy_publish_file_giving_it_its_name.rs`:
//!   the calls that give the filled file its destination name in one step,
//!   and the removal of a name an interrupted fallback left.
//! - `worktree_copy_publish_file.rs`:
//!   the publication of one staged regular file at its destination name in one step:
//!   unnamed while it is filled,
//!   with the entry's mode before it can be read,
//!   and linked into place only when whole.
//! - `worktree_copy_registration_naming.rs`:
//!   which of a repository's registrations names one destination,
//!   read from the administrative directories rather than from the destination's pointer.
//! - `worktree_copy_repository_path.rs`:
//!   the bytes of one repository path,
//!   how its components are joined and compared,
//!   and the validation every one passes.
//! - `worktree_copy_stage_prefix.rs`:
//!   the prefix every private stage container's name begins with.
//! - `worktree_copy_prune.rs`:
//!   what a copy from a main checkout leaves out:
//!   caches,
//!   optionally run records,
//!   and other checkouts.
//! - `worktree_copy_stage_claim_pointer_presence.rs`:
//!   the name of Git's pointer file in a worktree,
//!   and the answers a look at the destination's pointer holds.
//! - `worktree_copy_stage_claim_owner_verdicts.rs`:
//!   which claim owners were found dead,
//!   decided before anything is removed.
//! - `worktree_copy_stage_entries_the_record_accounts_for.rs`:
//!   the first entry of a stage container that the record of its creation's object cannot account for.
//! - `worktree_copy_symlink_target.rs`:
//!   the target a copied symbolic link must hold at its new place,
//!   and the rule that tells two links naming one place.
//!   It is path arithmetic only,
//!   so every system that copies links compiles it.
//! - `worktree_copy_vanished_entry.rs`:
//!   whether an entry a walk of a live checkout listed is still there when the walk reaches it,
//!   and that its absence is no failure.
//! - `worktree_copy_selection.rs`:
//!   the ignored source paths a linked-worktree copy selects,
//!   and the path validation every selection passes.
//! - `worktree_copy_manifest.rs`:
//!   the deterministic parent-first manifest of a copy,
//!   and the mode application to a staged copy.
//! - `worktree_copy_stage_snapshot.rs`:
//!   the validated private snapshot of one source,
//!   ready to install.
//! - `worktree_copy_compare_entry_match.rs`:
//!   the comparison of one manifest entry against the filesystem:
//!   its kind,
//!   its mode,
//!   its link target or its bytes.
//! - `worktree_copy_compare_walks.rs`:
//!   the source's walk under the stage's leave-out rule,
//!   and the stage's own walk,
//!   each one job of the blocking pool.
//! - `worktree_copy_compare.rs`:
//!   exact comparison of manifest entries against the filesystem,
//!   and the source stability check after staging.
//! - `worktree_copy_install_entry.rs`:
//!   the exclusive creation of one destination entry from the stage,
//!   and the identity captured after it.
//! - `worktree_copy_journal_names_and_shapes.rs`:
//!   the names a copy journal lives under,
//!   the folder that keeps the records nothing can read,
//!   the phase a copy reached,
//!   and the shape of one journal record.
//! - `worktree_copy_journal_encoding_and_validation.rs`:
//!   the exact JSON encoding of a copy journal and the strict validation of every field recovery will act on.
//! - `worktree_copy_private_path.rs`:
//!   the check that a path the copy acts on is canonical,
//!   this account's,
//!   private,
//!   and of the kind it claims.
//! - `worktree_copy_atomic_write.rs`:
//!   the publication of one record of the copy whole:
//!   the name of its temporary,
//!   the write into it,
//!   and the one rename that gives it its final name.
//! - `worktree_copy_journal.rs`:
//!   the durable journal of one copy:
//!   private storage,
//!   creating a journal and reading the root.
//! - `worktree_copy_stage_binding.rs`:
//!   the registration one staged copy is bound to,
//!   recorded beside its payload and checked before every write into a worktree.
//! - `worktree_copy_stage_snapshot_origin.rs`:
//!   what the filesystem says about the snapshot a command recorded before making it:
//!   the source it is taken from,
//!   and the transaction that creation must be later than.
//! - `worktree_copy_install_log.rs`:
//!   the append-only install log of one stage:
//!   per-batch intents and creations,
//!   with resumption.
//! - `worktree_copy_notices.rs`:
//!   whose destination a copy may write into,
//!   how the reservation answers it,
//!   and the line that says so when it may not.
//! - `worktree_copy_after_refusal.rs`:
//!   what becomes of the destination a refused `git worktree add` left:
//!   the empty folder this command made is removed,
//!   and one it left with something in it is named on the command's output.
//! - `worktree_removal_of_a_bound_clone.rs`:
//!   the refusal of a `git worktree remove` of a slot a whole managed clone is bound to,
//!   with the slot's state printed beside it.
//! - `worktree_copy_owed_taking.rs`:
//!   the marker one command takes in the journal root before it stages an owed copy,
//!   the removal of the markers whose owners no longer run,
//!   and the name order the markers are judged in.
//! - `worktree_copy_recovery_destinations.rs`:
//!   which destinations one recovery pass may finish a copy into,
//!   and whether a destination is still a registered worktree of the repository.
//! - `worktree_copy_rollback.rs`:
//!   removal of the paths a failed installation created,
//!   only where their identity and content still prove them monochromatic-git-policy-cli's own.
//! - `worktree_copy_stage_claim_record.rs`:
//!   the words one stage claim file holds:
//!   the container it names,
//!   its bytes,
//!   and the owner,
//!   and the readers that answer what a claim file says.
//! - `worktree_copy_stage_file_copying.rs`:
//!   creating one file in the stage:
//!   the private directories it lands in,
//!   the clone request that copies its bytes when the filesystem can,
//!   the exclusive creation that fills it,
//!   and the change time that proves a clone's bytes.
//! - `worktree_copy_stage_entry_copying.rs`:
//!   copying one manifest entry into the stage:
//!   a file's bytes,
//!   a link's text written for its new place,
//!   or the answer that the source no longer holds it.
//! - `worktree_copy_stage_leaf_copying.rs`:
//!   the copying of a staged snapshot's files and links into the stage,
//!   one worker per slice of the manifest.
//! - `worktree_copy_install_recorder.rs`:
//!   the durable record of an installation in progress:
//!   every batch's claims and creations,
//!   and the recorder that keeps nothing.
//! - `worktree_copy_install_leaves.rs`:
//!   one batch's files and links created in slices:
//!   the entries below which one worker is faster,
//!   and the state and the runner of one slice.
//! - `worktree_copy_install.rs`:
//!   installation of a validated stage into a new worktree,
//!   with preflight,
//!   batches and rollback on failure.
//! - `worktree_copy_journal_removal.rs`:
//!   the removal of a finished or abandoned copy transaction's stage and journal.
//! - `worktree_copy_settling.rs`:
//!   settling a staged copy against its source:
//!   the passes that copy a changed entry again and leave a vanished one out.
//! - `worktree_copy_stage_left_out_paths.rs`:
//!   which roots and paths a stage leaves out:
//!   the registered worktrees nested under the source,
//!   and any path through a private stage.
//! - `worktree_copy_stage.rs`:
//!   the private,
//!   validated stage of one source's ignored state beside a destination worktree.
//! - `worktree_copy_stage_made_object.rs`:
//!   the durable record of the object one stage's creation made:
//!   its identity,
//!   and the instant the wrapper last wrote into it.
//! - `worktree_copy_stage_claim_swept_files.rs`:
//!   the two sweeps of the journal root that remove files rather than stages,
//!   kept beside the reaping so that file stays under the line limit.
//! - `worktree_copy_stage_reaping_proof_and_words.rs`:
//!   whether that record proves the folder standing at a container's name,
//!   and the words a sweep says about a stage it keeps or removes.
//! - `worktree_copy_stage_claim_held_by_a_command.rs`:
//!   the claim one command holds on a stage container's name:
//!   the marking of its add as complete,
//!   the naming of the Git it started,
//!   the binding recorded once Git returned,
//!   and the release that removes its three files.
//! - `worktree_copy_stage_claim.rs`:
//!   the durable claim on a stage container's name,
//!   and the removal of stages an interrupted copy left behind.
//! - `worktree_copy_interrupted_add_settlement.rs`:
//!   what an interrupted `git worktree add` settles to,
//!   with the proof its checkout is whole or gone.
//! - `worktree_copy_interrupted_checkout.rs`:
//!   the checkout an interrupted `git worktree add` never finished,
//!   finished by a command in the worktree Git registered for it.
//! - `worktree_copy_stage_claim_owed_copy_elsewhere.rs`:
//!   whether the copy an add's dead record still owes has a future in a worktree other than the one the command runs
//!   in,
//!   kept beside the reaping so that file stays under the line limit.
//! - `worktree_copy_swapped_add_settlement.rs`:
//!   the add whose own snapshot was swapped into place before its command was killed:
//!   the two identities that prove it,
//!   and the settlement of its stage and records.
//! - `worktree_copy_registry.rs`:
//!   which linked worktrees a worktree-creating Git command registered,
//!   before and after it ran.
//! - `worktree_copy_stages_being_filled.rs`:
//!   the stage containers a live command is filling that stand below a tree,
//!   read from the claims beside the journals.
//! - `worktree_copy_unfilled_record_words.rs`:
//!   the words one unfilled-copy record holds:
//!   its suffix,
//!   its five fields,
//!   and the name the record of one journal takes.
//! - `worktree_copy_unfilled_record.rs`:
//!   the durable record of a worktree left without part of its source's ignored state,
//!   reported by every later command until the worktree is gone or replaced.
//! - `worktree_copy_source_still_being_filled.rs`:
//!   whether a source worktree is itself the destination of a copy that still runs:
//!   a copy stages nothing from it and tells the person why.
//! - `worktree_copy_transaction.rs`:
//!   one journaled installation to an end,
//!   and resumption from the log.
//! - `worktree_copy_synchronize.rs`:
//!   the staging,
//!   journaling and installation of one created worktree's ignored state,
//!   bound to the registration real Git made.
//! - `worktree_copy_synchronize_after_git.rs`:
//!   what one command does around the copy once real Git returned:
//!   whose destination the worktree is,
//!   which registration Git made for it,
//!   and the summary line of a copy that ran.
//! - `worktree_copy_unreadable_record_aside.rs`:
//!   where a record of an interrupted copy that cannot be read is kept,
//!   and the one rename that puts it there.
//! - `worktree_copy_unreadable_record_words.rs`:
//!   the one line each such record is settled with:
//!   what it was,
//!   where it went,
//!   and what may still stand on disk.
//! - `worktree_copy_records_that_name_a_worktree.rs`:
//!   the records a copy leaves that name a worktree rather than a private stage:
//!   the record of a worktree whose copy did not finish,
//!   and the destination record of an add whose claim is gone.
//! - `worktree_copy_unreadable_records.rs`:
//!   the records of an interrupted copy that cannot be read:
//!   each is moved aside with one line,
//!   what it named is found again from the records beside it where it can be,
//!   and the command that met it runs.
//! - `worktree_copy_stage_claim_reaping.rs`:
//!   the judging of stage claims and the removal of the stages that interrupted copies left behind.
//! - `worktree_copy_owed_copy_proofs.rs`:
//!   what must still hold before the pass that finishes an owed copy touches the disk:
//!   the destination is still the registration the add made,
//!   the source is still a worktree of this repository,
//!   and anything at the stage's name is provably this copy's own.
//! - `worktree_copy_owed_copy_records.rs`:
//!   what the claim and the two records beside it say about the copy an add owes:
//!   its selection,
//!   its destination,
//!   and the identity that proves the worktree there is still that one.
//! - `worktree_copy_owed_copy_making_it.rs`:
//!   how one owed copy is made:
//!   the checks that still hold,
//!   the staging and installation under the journal,
//!   and the record of an unfilled worktree when it cannot be made.
//! - `worktree_copy_owed_copy.rs`:
//!   the ignored-state copy an interrupted `git worktree add` still owes,
//!   made by a command in the worktree it fills.
//! - `worktree_copy_recovery.rs`:
//!   recovery of interrupted copies:
//!   discards with notices,
//!   resumption,
//!   and the pending check.
//! - `worktree_copy_creation_settlement.rs`:
//!   what one creating command does around real Git:
//!   the record of the add it makes,
//!   the cheap-worktree ladder and the child Git it starts,
//!   the ignored-state copy into the worktree it created,
//!   and the settlement of a destination Git refused.
//! - `worktree_copy_settles_one_invocation.rs`:
//!   the settlement of one forwarded invocation:
//!   its bypasses,
//!   what a dead owner left to recover,
//!   and the creating command handed to its own settlement.
//! - `worktree_copy_forward_outcome.rs`:
//!   how one forwarded run ended,
//!   the words a stopped step is named with,
//!   the repository directories every query of this route reads,
//!   and the plain start of real Git that reports that ending.
//! - `worktree_copy_forward_step_runner.rs`:
//!   the runner that starts real Git as a child,
//!   for each step the copy route runs:
//!   the person's own command with the checkout options it needs,
//!   and the ladder's own steps under their ceilings.
//! - `worktree_copy_forward_settled_run.rs`:
//!   the settled run of one forwarded invocation:
//!   the copy's lifecycle with real Git as its runner,
//!   and how its ending becomes this command's answer.
//! - `worktree_copy_forward.rs`:
//!   which forwarded invocations the copy applies to,
//!   and the runs that answer them:
//!   the plain start,
//!   the settled run,
//!   and the removal a subvolume destroy answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // worktree_copying/index.ts: one `export * from './file'` per module below.
//! ```

/// The one failure type of the linked-worktree ignored-state copy and its diagnostics.
pub mod worktree_copy_error;

/// Which linked worktrees a worktree-creating Git command registered,
/// before and after it ran.
pub mod worktree_copy_registry;

/// The bytes of one repository path,
/// how its components are joined and compared,
/// and the validation every one passes.
pub mod worktree_copy_repository_path;

/// Whether an entry a walk of a live checkout listed is still there when the walk reaches it,
/// and that its absence is no failure.
pub mod worktree_copy_vanished_entry;

/// The ignored source paths a linked-worktree copy selects,
/// and the path validation every selection passes.
pub mod worktree_copy_selection;

/// The prefix every private stage container's name begins with.
pub mod worktree_copy_stage_prefix;

/// What a copy from a main checkout leaves out:
/// caches,
/// optionally run records,
/// and other checkouts.
pub mod worktree_copy_prune;

/// The deterministic parent-first manifest of a copy,
/// and the mode application to a staged copy.
pub mod worktree_copy_manifest;

/// The validated private snapshot of one source,
/// ready to install.
pub mod worktree_copy_stage_snapshot;

/// The comparison of one manifest entry against the filesystem:
/// its kind,
/// its mode,
/// its link target or its bytes.
pub mod worktree_copy_compare_entry_match;

/// The source's walk under the stage's leave-out rule,
/// and the stage's own walk,
/// each one job of the blocking pool.
pub mod worktree_copy_compare_walks;

/// Exact comparison of manifest entries against the filesystem,
/// and the source stability check after staging.
pub mod worktree_copy_compare;

/// Which roots and paths a stage leaves out.
pub mod worktree_copy_stage_left_out_paths;

/// The private,
/// validated stage of one source's ignored state beside a destination worktree.
pub mod worktree_copy_stage;

/// Creating one file in the stage:
/// the private directories it lands in,
/// the clone request that copies its bytes when the filesystem can,
/// the exclusive creation that fills it,
/// and the change time that proves a clone's bytes.
#[cfg(unix)]
pub mod worktree_copy_stage_file_copying;

/// Copying one manifest entry into the stage:
/// a file's bytes,
/// a link's text written for its new place,
/// or the answer that the source no longer holds it.
#[cfg(unix)]
pub mod worktree_copy_stage_entry_copying;

/// The copying of a staged snapshot's files and links into the stage,
/// one worker per slice of the manifest.
#[cfg(unix)]
pub mod worktree_copy_stage_leaf_copying;

/// Settling a staged copy against its source:
/// the passes that copy a changed entry again and leave a vanished one out.
#[cfg(unix)]
pub mod worktree_copy_settling;

/// The exclusive creation of one destination entry from the stage,
/// and the identity captured after it.
#[cfg(unix)]
pub mod worktree_copy_install_entry;

/// The stable name a fallback's temporary is filled under.
#[cfg(unix)]
pub mod worktree_copy_publish_file_temporary_name;

/// The calls that make one file in a directory this module holds open.
#[cfg(unix)]
pub mod worktree_copy_publish_file_making_the_file;

/// The calls that give a filled file its destination name in one step.
#[cfg(unix)]
pub mod worktree_copy_publish_file_giving_it_its_name;

/// The publication of one staged regular file at its destination name in one step:
/// unnamed while it is filled,
/// with the entry's mode before it can be read,
/// and linked into place only when whole.
#[cfg(unix)]
pub mod worktree_copy_publish_file;

/// The durable record of an installation in progress:
/// every batch's claims reported before a path is created,
/// every proven creation reported after,
/// and the recorder that keeps nothing.
#[cfg(unix)]
pub mod worktree_copy_install_recorder;

/// One batch's files and links created in slices:
/// the entries below which one worker is faster,
/// one entry created when nothing is already at its destination,
/// and the slices that run side by side.
#[cfg(unix)]
pub mod worktree_copy_install_leaves;

/// Installation of a validated stage into a new worktree,
/// with preflight,
/// batches and rollback on failure.
#[cfg(unix)]
pub mod worktree_copy_install;

/// Removal of the paths a failed installation created,
/// only where their identity and content still prove them monochromatic-git-policy-cli's own.
#[cfg(unix)]
pub mod worktree_copy_rollback;

/// The check that a path the copy acts on is canonical,
/// this account's,
/// private,
/// and of the kind it claims.
#[cfg(unix)]
pub mod worktree_copy_private_path;

/// The publication of one record of the copy whole:
/// the name of its temporary,
/// the write into it,
/// and the one rename that gives it its final name.
#[cfg(unix)]
pub mod worktree_copy_atomic_write;

/// The durable journal of one copy:
/// private storage,
/// creating a journal and reading the root.
#[cfg(unix)]
pub mod worktree_copy_journal;

/// The names a copy journal lives under,
/// the folder that keeps the records nothing can read,
/// and the shapes it is written as.
#[cfg(unix)]
pub mod worktree_copy_journal_names_and_shapes;

/// The exact JSON encoding of a copy journal and the strict validation of every field recovery will act on.
#[cfg(unix)]
pub mod worktree_copy_journal_encoding_and_validation;

/// The removal of a finished or abandoned copy transaction's stage and journal.
#[cfg(unix)]
pub mod worktree_copy_journal_removal;

/// The words one stage claim file holds,
/// and the readers that answer what it says.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_record;

/// The claim one command holds on a stage container's name:
/// the marking of its add as complete,
/// the naming of the Git it started,
/// the binding recorded once Git returned,
/// and the release that removes its three files.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_held_by_a_command;

/// The durable claim on a stage container's name,
/// and the removal of stages an interrupted copy left behind.
#[cfg(unix)]
pub mod worktree_copy_stage_claim;

/// The judging of stage claims and the removal of the stages that interrupted copies left behind.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_reaping;

/// Which claim owners were found dead,
/// decided before anything is removed.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_owner_verdicts;

/// The two sweeps of the journal root that remove files rather than stages,
/// kept beside the reaping so that file stays under the line limit.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_swept_files;

/// Whether the copy an add's dead record still owes has a future in a worktree other than the one the command runs in,
/// kept beside the reaping so that file stays under the line limit.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_owed_copy_elsewhere;

/// The stage containers a live command is filling that stand below a tree,
/// read from the claims beside the journals.
#[cfg(unix)]
pub mod worktree_copy_stages_being_filled;

/// The durable record of the object one stage's creation made:
/// its identity,
/// and the instant the wrapper last wrote into it.
#[cfg(unix)]
pub mod worktree_copy_stage_made_object;

/// The first entry of a stage container that the record of its creation's object cannot account for.
#[cfg(target_os = "linux")]
pub mod worktree_copy_stage_entries_the_record_accounts_for;

/// What the filesystem says about the snapshot a command recorded before making it:
/// the source it is taken from,
/// and the transaction that creation must be later than.
#[cfg(unix)]
pub mod worktree_copy_stage_snapshot_origin;

/// Whether that record proves the folder standing at a container's name,
/// and the words a sweep says about a stage it keeps or removes.
#[cfg(unix)]
pub mod worktree_copy_stage_reaping_proof_and_words;

/// What proves the checkout of an interrupted `git worktree add` whole or gone.
#[cfg(unix)]
pub mod worktree_copy_interrupted_add_settlement_checkout_proof;

/// What an interrupted `git worktree add` settles to,
/// with the proof its checkout is whole or gone.
#[cfg(unix)]
pub mod worktree_copy_interrupted_add_settlement;

/// The add whose own snapshot was swapped into place before its command was killed:
/// the two identities that prove it,
/// and the settlement of its stage and records.
#[cfg(unix)]
pub mod worktree_copy_swapped_add_settlement;

/// The registration one staged copy is bound to,
/// recorded beside its payload and checked before every write into a worktree.
#[cfg(unix)]
pub mod worktree_copy_stage_binding;

/// The words one unfilled-copy record holds.
#[cfg(unix)]
pub mod worktree_copy_unfilled_record_words;

/// The durable record of a worktree left without part of its source's ignored state,
/// reported by every later command until the worktree is gone or replaced.
#[cfg(unix)]
pub mod worktree_copy_unfilled_record;

/// The target a copied symbolic link must hold at its new place,
/// and the rule that tells two links naming one place.
/// It is path arithmetic only,
/// so every system that copies links compiles it.
pub mod worktree_copy_symlink_target;

/// The append-only install log of one stage:
/// per-batch intents and creations,
/// with resumption.
#[cfg(unix)]
pub mod worktree_copy_install_log;

/// One journaled installation to an end,
/// and resumption from the log.
#[cfg(unix)]
pub mod worktree_copy_transaction;

/// Whose destination a copy may write into,
/// how the reservation answers it,
/// and the line that says so when it may not.
#[cfg(unix)]
pub mod worktree_copy_notices;

/// The staging,
/// journaling and installation of one created worktree's ignored state,
/// bound to the registration real Git made.
#[cfg(unix)]
pub mod worktree_copy_synchronize;

/// What one command does around the copy once real Git returned:
/// whose destination the worktree is,
/// which registration Git made for it,
/// and the summary line of a copy that ran.
#[cfg(unix)]
pub mod worktree_copy_synchronize_after_git;

/// What becomes of the destination a refused `git worktree add` left:
/// the empty folder this command made is removed,
/// and one it left with something in it is named on the command's output.
#[cfg(unix)]
pub mod worktree_copy_after_refusal;

/// The process that owns one item of a copy transaction,
/// written into the item's name,
/// and the proof that it still runs.
#[cfg(unix)]
pub mod worktree_copy_owner;

/// Which of a repository's registrations names one destination,
/// read from the administrative directories rather than from the destination's pointer.
#[cfg(unix)]
pub mod worktree_copy_registration_naming;

/// The bound one command holds when it finishes a copy it did not start,
/// and the words a copy stopped at that bound is reported with.
#[cfg(unix)]
pub mod worktree_copy_ceiling;

/// The checkout an interrupted `git worktree add` never finished,
/// finished by a command in the worktree Git registered for it.
#[cfg(unix)]
pub mod worktree_copy_interrupted_checkout;

/// Every line the pass that finishes an owed worktree copy prints:
/// the one that names a copy owed into another worktree,
/// and the three that say what this command's own copy is doing.
#[cfg(unix)]
pub mod worktree_copy_owed_copy_notices;

/// The marker one command takes in the journal root before it stages an owed copy,
/// the removal of the markers whose owners no longer run,
/// and the name order the markers are judged in.
#[cfg(unix)]
pub mod worktree_copy_owed_taking;

/// What must still hold before the pass that finishes an owed copy touches the disk:
/// the destination is still the registration the add made,
/// the source is still a worktree of this repository,
/// and anything at the stage's name is provably this copy's own.
#[cfg(unix)]
pub mod worktree_copy_owed_copy_proofs;

/// What the claim and the two records beside it say about the copy an add owes:
/// its selection,
/// its destination,
/// and the identity that proves the worktree there is still that one.
#[cfg(unix)]
pub mod worktree_copy_owed_copy_records;

/// How one owed copy is made:
/// the checks that still hold,
/// the staging and installation under the journal,
/// and the record of an unfilled worktree when it cannot be made.
#[cfg(unix)]
pub mod worktree_copy_owed_copy_making_it;

/// The ignored-state copy an interrupted `git worktree add` still owes,
/// made by a command in the worktree it fills.
#[cfg(unix)]
pub mod worktree_copy_owed_copy;

/// Whether the worktree a copy takes its state from can still be one,
/// and the reason a copy is given up on when it cannot.
#[cfg(unix)]
pub mod worktree_copy_owed_copy_source;

/// Where a record of an interrupted copy that cannot be read is kept,
/// and the one rename that puts it there.
#[cfg(unix)]
pub mod worktree_copy_unreadable_record_aside;

/// The one line each such record is settled with:
/// what it was,
/// where it went,
/// and what may still stand on disk.
#[cfg(unix)]
pub mod worktree_copy_unreadable_record_words;

/// The refusal of a `git worktree remove` of a slot a whole managed clone is bound to,
/// with the slot's state printed beside it.
#[cfg(unix)]
pub mod worktree_removal_of_a_bound_clone;

/// Which destinations one recovery pass may finish a copy into,
/// and whether a destination is still a registered worktree of the repository.
#[cfg(unix)]
pub mod worktree_copy_recovery_destinations;

/// Recovery of interrupted copies:
/// registration checks,
/// discards with notices,
/// resumption,
/// and the pending check.
#[cfg(unix)]
pub mod worktree_copy_recovery;

/// The records a copy leaves that name a worktree rather than a private stage:
/// the record of a worktree whose copy did not finish,
/// and the destination record of an add whose claim is gone.
#[cfg(unix)]
pub mod worktree_copy_records_that_name_a_worktree;

/// The records of an interrupted copy that cannot be read:
/// each is moved aside with one line,
/// what it named is found again from the records beside it where it can be,
/// and the command that met it runs.
#[cfg(unix)]
pub mod worktree_copy_unreadable_records;

/// Whether a source worktree is itself the destination of a copy that still runs:
/// a copy stages nothing from it and tells the person why.
#[cfg(unix)]
pub mod worktree_copy_source_still_being_filled;

/// The copy's contract:
/// the facts a settlement is handed,
/// the steps it asks its Git runner for,
/// and the settlement it answers with.
#[cfg(unix)]
pub mod worktree_copy;

/// What is left to do once a created worktree holds its ignored state:
/// its package-manager repair,
/// and the trash a snapshot
///  left there,
/// handed to the commit service.
#[cfg(unix)]
pub mod worktree_copy_after_lock;

/// What one creating command does around real Git:
/// the record of the add it makes,
/// the cheap-worktree ladder and the child Git it starts,
/// the ignored-state copy into the worktree it created,
/// and the settlement of a destination Git refused.
#[cfg(unix)]
pub mod worktree_copy_creation_settlement;

/// The settlement of one forwarded invocation:
/// its bypasses,
/// what a dead owner left to recover,
/// and the creating command handed to its own settlement.
#[cfg(unix)]
pub mod worktree_copy_settles_one_invocation;

/// How one forwarded run ended,
/// the words a stopped step is named with,
/// the repository directories every query of this route reads,
/// and the plain start of real Git that reports that ending.
pub mod worktree_copy_forward_outcome;

/// The runner that starts real Git as a child,
/// for each step the copy route runs:
/// the person's own command with the checkout options it needs,
/// and the ladder's own steps under their ceilings.
pub mod worktree_copy_forward_step_runner;

/// The settled run of one forwarded invocation:
/// the copy's lifecycle with real Git as its runner,
/// and how its ending becomes this command's answer.
pub mod worktree_copy_forward_settled_run;

/// Which forwarded invocations the copy applies to,
/// and the runs that answer them:
/// the plain start,
/// the settled run,
/// and the removal a subvolume destroy answers.
pub mod worktree_copy_forward;
