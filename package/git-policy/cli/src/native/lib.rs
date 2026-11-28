//! What:
//!  Internal native monochromatic-git-policy-cli implementation,
//!  not yet the production executable.
//! Why:
//!  The Rust rewrite preserves Git's own argument,
//!  repository and transaction semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Internal modules used by the forthcoming native entry point and consumer verification.
//! ```
//!
//! Layout:
//!  Every folder below is one subsystem,
//!  and its `mod.rs` declares the modules inside it.
//!  A glob re-export beside each folder keeps the old flat paths (`crate::module_name`) working.
//!  `README.md` ("Source layout") says where a new file belongs.

//region What one command and its policies do

/// What one invocation does from arguments to Git:
///  the decision,
///  real Git resolution,
///  forwarding,
///  startup and management words.
pub mod invocation;

/// What:
///  `pub use invocation::*;` also names every module declared in `invocation/mod.rs` directly under the crate root,
///  with the visibility each module was declared with.
/// Why:
///  The modules moved into folders but keep their paths,
///  `crate::name` and `git_policy_cli::name`,
///  so no caller,
///  fuzz target or test changes and the move stays mechanical.
/// Gotcha:
///  A glob re-export is shadowed by a same-named item written in this file,
///  so no folder may share a name with a module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './invocation';
/// ```
pub use invocation::*;

/// Reading and validating `monochromatic-git-policy-cli.config.jsonc` and the Git configuration one invocation is judged by.
pub mod config;

/// The same re-export for `config`.
pub use config::*;

/// Per-command handling:
///  the option grammar of each Git command the wrapper understands,
///  and the rules that rewrite,
///  refuse or hint at a command.
pub mod command;

/// The same re-export for `command`.
pub use command::*;

/// The policy engine:
///  the registry,
///  triggers,
///  events,
///  the passes over candidates and direct fixes.
pub mod policy;

/// The same re-export for `policy`.
pub use policy::*;

/// The `markdown/autofix` policy and its in-process linter.
pub mod markdown;

/// The same re-export for `markdown`.
pub use markdown::*;

/// The dependent-version policy:
///  manifest and import facts,
///  the release plan and the policy over them.
pub mod dependent_version;

/// The same re-export for `dependent_version`.
pub use dependent_version::*;

/// The standards-review shadow run:
///  its request,
///  the patch it covers,
///  its one HTTPS call and the result it appends.
pub mod standards_review;

/// The same re-export for `standards_review`.
pub use standards_review::*;

/// Candidate content:
///  what a commit or push would add,
///  read from the index or objects,
///  and the scanner run over it.
pub mod candidate;

/// The same re-export for `candidate`.
pub use candidate::*;

//endregion

//region The commit path and the resident commit service

/// The wrapper's commit path:
///  arguments,
///  preparation,
///  convergence,
///  conclusion and timing.
pub mod commit;

/// The same re-export for `commit`.
pub use commit::*;

/// The shadow repository and the replay of captured changes onto it.
pub mod shadow;

/// The same re-export for `shadow`.
pub use shadow::*;

/// Durable transaction state:
///  owner records,
///  journal,
///  phases and recovery.
pub mod transaction;

/// The same re-export for `transaction`.
pub use transaction::*;

/// What the commit service does to land a commit:
///  the serialized step,
///  the index lock hold,
///  preparation,
///  replay and after-commit work.
pub mod landing;

/// The same re-export for `landing`.
pub use landing::*;

/// The resident commit service:
///  protocol,
///  server,
///  client,
///  journal,
///  provisioning and logging.
pub mod service;

/// The same re-export for `service`.
pub use service::*;

/// The references the wrapper keeps commits on,
///  and the pruning that bounds them by age and by size.
pub mod kept_references;

/// The same re-export for `kept_references`.
pub use kept_references::*;

//endregion

//region Pushes, merges and conflicts

/// Pushes:
///  the automatic push after a commit,
///  manual push handling,
///  bounded Git queries,
///  retries and progress watching.
pub mod push;

/// The same re-export for `push`.
pub use push::*;

/// Git LFS support:
///  pre-push planning,
///  pointer scanning and bounded uploads.
pub mod lfs;

/// The same re-export for `lfs`.
pub use lfs::*;

/// Merges:
///  the word-level second pass,
///  isolated merges and pull handling.
pub mod merge;

/// The same re-export for `merge`.
pub use merge::*;

/// Conflict handling around commits and checkouts:
///  side flags,
///  saved copies of conflicted files and leftover markers.
pub mod conflict;

/// The same re-export for `conflict`.
pub use conflict::*;

//endregion

//region Worktrees, filesystems and reports

/// Worktree commands:
///  add,
///  remove,
///  clean,
///  convert,
///  trash,
///  snapshots and the placement ladder.
pub mod worktree;

/// The same re-export for `worktree`.
pub use worktree::*;

/// Copying the ignored state of a linked worktree into a new one,
///  with staging,
///  journal and recovery.
pub mod worktree_copying;

/// The same re-export for `worktree_copying`.
pub use worktree_copying::*;

/// The descriptor attach of a ZFS worktree clone's mount:
///  the walk,
///  the recorded identities,
///  the post-attach check,
///  the rollback and the parent-adoption decision.
/// It is Linux only,
/// like every module it builds on:
/// `openat2`,
/// `flock` and `/proc/self/fdinfo` are the kernel's own.
#[cfg(target_os = "linux")]
pub mod worktree_clone_mount;

/// The same re-export for `worktree_clone_mount`.
#[cfg(target_os = "linux")]
pub use worktree_clone_mount::*;

/// The protocol between the account's wrapper and the root-run clone-mount
/// handler:
///  the durable mount intent,
///  the clone-local sweep nudge,
///  the receipts under `/run`,
///  the readiness record,
///  and the wrapper's wait for a matching receipt.
/// The wrapper and the root modes it carries both read it,
///  so the two read one grammar,
///  and it is Linux only,
///  like the directory-relative calls its receipts use.
#[cfg(target_os = "linux")]
pub mod worktree_clone_mount_protocol;

/// The same re-export for `worktree_clone_mount_protocol`.
#[cfg(target_os = "linux")]
pub use worktree_clone_mount_protocol::*;

/// The unmount half of the ZFS worktree clone route:
///  one removal's staging folder under the account's `~/.cache`,
///  the record beside it,
///  the `user-cache` pin,
///  the identity comparisons,
///  the rollback and the checked reattach.
/// It is Linux only,
/// like the mount calls it makes.
#[cfg(target_os = "linux")]
pub mod worktree_clone_unmount;

/// The same re-export for `worktree_clone_unmount`.
#[cfg(target_os = "linux")]
pub use worktree_clone_unmount::*;

/// The snapshot half of a ZFS worktree clone's removal:
///  the verified set submitted as one `ZFS_IOC_DESTROY_SNAPS` request that
///  names no `defer` key,
///  the kernel's per-name outcome kept,
///  the skip for an empty set,
///  and the split by largest fitting prefix against a budget the caller passes
///  in.
/// It is Linux only,
/// like the `/dev/zfs` requests it sends,
/// and no command calls it yet:
/// step 16's wiring is what reaches it.
#[cfg(target_os = "linux")]
pub mod worktree_clone_snapshot_batch;

/// The same re-export for `worktree_clone_snapshot_batch`.
#[cfg(target_os = "linux")]
pub use worktree_clone_snapshot_batch::*;

/// The registration half of the ZFS add route:
///  the add's claim and durable binding record,
///  the two snapshot inventories,
///  the managed Git lock's sentence,
///  the finder and the recipe for what a kill inside Git's add left,
///  the add's read of a removal's plan,
///  and the command each resume state names.
/// It is Linux only,
/// like the `worktrees` scan and the descriptor-relative removals it makes.
#[cfg(target_os = "linux")]
pub mod worktree_clone_binding_record;

/// The execution half of the ZFS add route:
///  the gate,
///  the preflight,
///  the claim,
///  Git's locked add,
///  the scan,
///  the snapshot and clone,
///  the mount,
///  the frozen proof,
///  the fill and the resume a later add makes.
/// It is Linux only,
/// like the descriptor walks and the pool calls it makes.
#[cfg(target_os = "linux")]
pub mod worktree_clone_add_route;

/// The same re-export for `worktree_clone_add_route`.
#[cfg(target_os = "linux")]
pub use worktree_clone_add_route::*;

/// The account companion the ZFS add route's preflight runs before its first write:
///  a small user-namespace check that repeats the probe's syscall sequence over
///  a tmpfs of its own,
///  the skip a host that refuses it a user namespace prints,
///  and the deadline its parent enforces.
/// It sits inside the wrapper's one executable and touches no ZFS.
/// It is Linux only,
/// like the descriptor calls it makes.
#[cfg(target_os = "linux")]
pub mod account_companion;

/// The account companion's group drop as `doctor` reports it:
///  one probe of this host and the line a skipped drop prints.
/// It is Linux only,
///  like the user namespace it makes.
#[cfg(target_os = "linux")]
pub mod group_drop_report;

/// The same re-export for `worktree_clone_binding_record`.
#[cfg(target_os = "linux")]
pub use worktree_clone_binding_record::*;

/// The deferred-origin checks of a ZFS worktree clone's removal:
///  the one verdict function a removal calls before the first snapshot request and
///  again before the head destroy,
///  the look the lineage proof makes before anything is taken down,
///  and the words a refusal prints.
/// Plan step 15's piece,
/// built ahead of the plan's order:
/// no command reaches it until that
///  step wires it in.
/// It is Linux only,
///  like the two reads it makes through the binding.
#[cfg(target_os = "linux")]
pub mod worktree_removal_deferred_origin_check;

/// The same re-export for `worktree_removal_deferred_origin_check`.
#[cfg(target_os = "linux")]
pub use worktree_removal_deferred_origin_check::*;

/// The managed Git lock's life after the add wrote it:
///  the states `doctor` and the removal tell apart,
///  the write that puts a lost lock back,
///  and the release the registration-cleanup step makes after the clone is proven gone.
/// The piece is built ahead of the plan's order and unwired:
///  no command of the wrapper calls it until plan step 13 wires it in.
/// It is Linux only,
///  like the descriptor-relative release it makes.
#[cfg(target_os = "linux")]
pub mod worktree_clone_managed_lock;

/// The same re-export for `worktree_clone_managed_lock`.
#[cfg(target_os = "linux")]
pub use worktree_clone_managed_lock::*;

/// The removal-side half of the ZFS worktree route that needs no command yet:
///  the lineage proof a removal makes before its first delete,
///  the clone inventory and the decided bookmark rule,
///  the persisted destruction plan with its two-stage reader,
///  the removal's `unmount` intent and verified answer,
///  and the names it prints as lost.
/// Plan step 14's piece,
/// built ahead of the plan's order:
/// no command reaches it until that step wires it in.
/// It is Linux only,
/// like the pool reads and the folder modes it checks.
#[cfg(target_os = "linux")]
pub mod worktree_clone_removal_proof_and_plan;

/// The same re-export for `worktree_clone_removal_proof_and_plan`.
#[cfg(target_os = "linux")]
pub use worktree_clone_removal_proof_and_plan::*;

/// Plan step 17's piece,
/// built ahead of the plan's order:
/// no command reaches it until the round that switches the removal's command path on wires it in.
/// It is Linux only,
/// like the mount table and the claim it reads.
#[cfg(target_os = "linux")]
pub mod worktree_clone_removal_cancel;

/// The same re-export for `worktree_clone_removal_cancel`.
#[cfg(target_os = "linux")]
pub use worktree_clone_removal_cancel::*;

/// The record that carries an origin snapshot's cleanup past the clone and the registration:
/// its folder,
/// its name grammar,
/// its lines and states,
/// its bound,
/// the identity proof one attempt makes,
/// and the look the worktree commands make at it.
/// Built ahead of the plan's order:
/// no command reaches it until that step wires it in.
/// It is Linux only,
/// like the pool reads and the folder modes it checks.
#[cfg(target_os = "linux")]
pub mod worktree_clone_origin_cleanup;

/// The same re-export for `worktree_clone_origin_cleanup`.
#[cfg(target_os = "linux")]
pub use worktree_clone_origin_cleanup::*;

/// The finish pass that removes a destruction plan whose head is `gone`,
///  settles the Git registration it names,
///  removes the binding's files and then the plan,
///  and releases the slot's claim last.
/// Plan step 16's piece,
///  built ahead of the plan's order:
///  no command reaches it until that step wires it in.
/// It is Linux only,
///  like the claim,
/// the pool reads and the folder modes it checks.
#[cfg(target_os = "linux")]
pub mod worktree_clone_removal_finish_pass;

/// The same re-export for `worktree_clone_removal_finish_pass`.
#[cfg(target_os = "linux")]
pub use worktree_clone_removal_finish_pass::*;

/// The boot and shutdown units of the ZFS route,
/// as the setup writes them:
///  the gate that reconciles clone mounts after boot,
///  its two drop-ins,
///  the propagation service and the mount drop-in that starts it,
///  one unit per dataset whose mountpoint lies inside the checkout,
///  and the list of files a setup run writes.
/// Plan step 19's piece,
/// built ahead of the plan's order and unwired:
/// nothing writes or starts one of these files until that step's setup run
/// and the owner's own act install them,
/// and no command of this build reads them.
/// It is Linux only,
/// like the modes those lines run.
#[cfg(target_os = "linux")]
pub mod checkout_boot_units;

/// The same re-export for `checkout_boot_units`.
#[cfg(target_os = "linux")]
pub use checkout_boot_units::*;

/// What one root run does about the checkout's mount propagation:
///  the check that the mount passes no mounts on,
///  the set that makes it private through the descriptor it verified,
///  the read that proves the set,
///  the report of the shared mounts beneath the checkout,
///  and the root-owned record of the answer.
/// Plan step 19's piece,
/// built ahead of the plan's order and unwired:
/// its call at the head of `boot-reconcile` and of the default sweep is that
/// step's wiring,
/// so no command and no root mode reaches it in this release,
/// and the guest arm beside its files calls it itself.
/// It is Linux only,
/// like the `statmount(2)`,
/// `mount_setattr(2)` and `/proc/self/mountinfo` calls it makes.
#[cfg(target_os = "linux")]
pub mod checkout_propagation;

/// The immutable covered entry of the checkout's underlay:
///  what the kernel says about the entry at the checkout path and its parent,
///  why a reading is refused,
///  and the protected,
///  unprotected or unproven answer a boot takes before it
///  attaches a clone or trusts a reimported or rolled-back checkout.
/// Plan step 19's piece,
/// built ahead of the plan's order and unwired:
/// no command and no boot check reads it until that step's wiring calls it.
/// It is Linux only,
/// like the `statx(2)` attributes it reads.
#[cfg(target_os = "linux")]
pub mod checkout_underlay_entry;

/// The same re-export for `checkout_underlay_entry`.
#[cfg(target_os = "linux")]
pub use checkout_underlay_entry::*;

/// The settling command a state a resume meets names and this release does not have yet:
///  its grammar and the two values a person confirms,
///  its six reads in the design's order,
///  its two modes over the records and the registration,
///  and its refusal.
/// Built ahead of the plan's order and unwired:
/// no path from a `git` command,
/// `doctor`,
/// the mount handler or a boot unit
/// calls it until the round that switches removal on adds it to the grammar.
/// It is Linux only,
/// like the record modes and the pool reads it makes.
#[cfg(target_os = "linux")]
pub mod worktree_clone_record_settling;

/// The same re-export for `worktree_clone_record_settling`.
#[cfg(target_os = "linux")]
pub use worktree_clone_record_settling::*;

/// What one worktree-clone slot is,
///  read without changing anything:
///  a known unmounted clone,
///  a pending add,
///  a pending removal,
///  a mounted managed clone,
///  a slot the route holds nothing in,
///  or a state no reading proves,
///  and the report of the slot,
///  its dataset and the one next safe action.
/// It reads the add's binding record,
///  the removal's plan folder,
///  the clone's mount intent and this process's mount table,
///  and it is Linux only,
///  like the mount table it reads.
#[cfg(target_os = "linux")]
pub mod worktree_clone_slot_state;

/// The same re-export for `worktree_clone_slot_state`.
#[cfg(target_os = "linux")]
pub use worktree_clone_slot_state::*;

/// The refusal of `git worktree move` for a slot a managed clone is bound to,
///  mounted or not,
///  and the sentence that says why the slot is fixed.
/// The step that wires `worktree move` calls it;
///  no wrapper command reaches it yet,
///  and it is Linux only,
///  like the record it reads.
#[cfg(target_os = "linux")]
pub mod worktree_clone_move_refusal;

/// The same re-export for `worktree_clone_move_refusal`.
#[cfg(target_os = "linux")]
pub use worktree_clone_move_refusal::*;

/// The underlay pointer the add keeps for `git worktree prune`,
///  independent of the clone's own pointer:
///  what it says against the bytes the record holds,
///  the one write that puts it back,
///  and the pass a prune runs before Git starts.
/// It is Linux only,
///  like the record and the mount table it reads.
#[cfg(target_os = "linux")]
pub mod worktree_clone_prune_pointer;

/// The same re-export for `worktree_clone_prune_pointer`.
#[cfg(target_os = "linux")]
pub use worktree_clone_prune_pointer::*;

/// The watches a clone's mount leaves blind:
///  whether the slot still names the directory its registration recorded,
///  and the sentence that names the editor watcher and the fsmonitor daemon a
///  person must reopen.
/// Plan step 21 builds it ahead of its wiring,
///  and no wrapper command reaches it yet.
/// It is Linux only,
///  like the identity it reads.
#[cfg(target_os = "linux")]
pub mod worktree_clone_watch_rebinding;

/// The same re-export for `worktree_clone_watch_rebinding`.
#[cfg(target_os = "linux")]
pub use worktree_clone_watch_rebinding::*;

/// The report of a damaged link between a worktree and its Git registration:
///  what was found,
///  what is kept,
///  what a repair can and cannot restore,
///  and the one next safe command,
///  read live for one slot.
/// It is Linux only,
///  like the record and the mount table it reads.
#[cfg(target_os = "linux")]
pub mod worktree_clone_link_repair;

/// The same re-export for `worktree_clone_link_repair`.
#[cfg(target_os = "linux")]
pub use worktree_clone_link_repair::*;

/// The ZFS datasets this process's own mount table shows mounted,
///  and the design names the route finds the checkout,
///  its container and the probe dataset by when the configuration leaves an
///  entry out.
/// The wrapper and the root modes both read it,
///  and neither may read the other's folder,
///  so it sits above both,
///  and it is Linux only,
///  like the table it reads.
#[cfg(target_os = "linux")]
pub mod zfs_mounted_dataset;

/// The ZFS route's root modes:
///  the one start name and the group word,
///  the mode grammar,
///  the paths one root-owned build serves,
///  the modes whose plan steps have not landed,
///  the administrator's parent adoption,
///  the checkout record and the nested dataset placed under it.
/// They are modes of this one executable,
///  so they are named through this path rather than a glob re-export:
///  the modes' words (`root_modes_name`,
/// `run`,
/// the install paths) belong to the handler,
///  and no old flat path of the wrapper names them.
/// It is Linux only,
/// like the module it builds on.
#[cfg(target_os = "linux")]
pub mod worktree_clone_mount_handler;

/// The root handler's all-namespace holder scan of one ZFS clone:
///  the live tasks grouped by mount namespace,
///  one mount table read per namespace,
///  the exact ZFS source match,
///  the managed attach a standing `unmount` intent sets aside,
///  and the result that names the holders,
///  the reads it could not complete,
///  and the GUID and nonce it answers under.
/// The plan's step 16 consumes the result before each snapshot-destroy request
///  and before the head destroy,
///  and step 18 reports its holders,
/// and it is Linux only,
/// like the `/proc` files it reads.
#[cfg(target_os = "linux")]
pub mod clone_holder_scan;

/// The forced arm of a ZFS worktree clone's removal:
///  who the command's caller and its session are,
///  the protected set a forced removal never ends,
///  what it may do to each kind of holder and what it refuses,
///  the reads it makes before its first act and change,
///  and every word it answers with.
/// Plan step 17 builds it ahead of the removal's cut,
/// so no person's command
///  reaches it and no sentence offers the flag,
///  and its waits ship with no value until the owner decides one or step 22's
///  measurement sizes them.
/// It is Linux only,
/// like the `/proc` reads,
/// the pidfds and the mount calls it would make.
#[cfg(target_os = "linux")]
pub mod worktree_clone_forced_removal;

/// The same re-export for `worktree_clone_forced_removal`.
#[cfg(target_os = "linux")]
pub use worktree_clone_forced_removal::*;

/// The mount-namespace read two of the wrapper's scans share:
///  every live task grouped by the kernel's own namespace identity,
///  one mount table read per namespace,
///  and the exact ZFS source match.
/// It is shared by the root start probe and the root handler's holder scan,
///  so the two cannot disagree about what a mount of one dataset is,
/// and it is Linux only,
/// like the `/proc` files it reads.
#[cfg(target_os = "linux")]
pub mod mount_namespace_scan;

/// The root start probe the kernel-release answer asks for:
/// the private namespace and the scratch tmpfs,
/// the exclusive attach,
/// the holder in a second namespace,
/// the move and the unmount from a second process,
/// the record under `/run` and the mode's one deadline.
/// It sits beside the gate and replaces none of it,
/// and it is Linux only,
/// like the mount calls it makes.
#[cfg(target_os = "linux")]
pub mod start_probe;

/// The start-time ZFS compatibility gate the root-run handler runs before its
///  first ioctl:
/// the qualified target,
/// the device,
/// the module's identity,
/// the binding's layout comparison,
/// the per-process verdict and the words a person reads.
/// It is Linux only,
/// like the binding it calls.
#[cfg(target_os = "linux")]
pub mod zfs_compatibility_gate;

/// Btrfs subvolume worktrees and the filesystem primitives under them.
pub mod subvolume;

/// The same re-export for `subvolume`.
pub use subvolume::*;

/// Whether the repository path is a usable link to the 860 drive,
/// and the gate that asks it before a command touches the repository.
pub mod repository_link_check;

/// The same re-export for `repository_link_check`.
pub use repository_link_check::*;

/// Package-manager and toolchain repair after a worktree is created.
pub mod repair;

/// The same re-export for `repair`.
pub use repair::*;

/// The `doctor` report and the checks of hooks and configuration it runs.
pub mod health;

/// The same re-export for `health`.
pub use health::*;

//endregion

//region Reading and locking the repository, processes and shared building blocks

/// In-process repository reads:
///  objects,
///  packs,
///  refs,
///  layout and configuration without starting Git.
pub mod repo_read;

/// The same re-export for `repo_read`.
pub use repo_read::*;

/// In-process reading and writing of the Git index file and its extensions.
pub mod index;

/// The same re-export for `index`.
pub use index::*;

/// Locks and their owners:
///  the index lock,
///  stale reference locks,
///  and the claims that let one command work on a worktree.
pub mod lock;

/// The same re-export for `lock`.
pub use lock::*;

/// Processes:
///  identities,
///  trees,
///  sockets,
///  signals and quiet-time ledgers.
pub mod process;

/// The same re-export for `process`.
pub use process::*;

/// Small shared building blocks:
///  private files,
///  scratch locations,
///  JSON,
///  time,
///  random ids,
///  paths and bounded parallelism.
pub mod foundation;

/// The same re-export for `foundation`.
pub use foundation::*;

//endregion

//region Test-only code, never part of the release executable

/// Test-only fixtures shared by many unit tests,
///  never part of the release executable.
#[cfg(test)]
#[path = "foundation/room_for_blocking_jobs_in_a_control.rs"]
pub mod room_for_blocking_jobs_in_a_control;

#[cfg(test)]
pub mod testing;

/// The same re-export for `testing`.
#[cfg(test)]
pub(crate) use testing::*;

/// Controls that read the crate's own source text.
#[cfg(test)]
pub mod source_guard;

/// The same re-export for `source_guard`.
#[cfg(test)]
pub(crate) use source_guard::*;

//endregion
