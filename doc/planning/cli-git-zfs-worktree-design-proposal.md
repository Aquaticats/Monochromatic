# ZFS worktree design proposal for the Git wrapper

**Non-final draft by an agent, unapproved.** This revision (2026-11-18T20:40:28-05:00) replaces the previous draft's
root-owned anchor, `checkout` subdirectory and directory exchange with the checkout at its existing path and
account-owned slots mounted directly over their Git registrations. It folds in the project's decisions in
[Owner's answers of 2026-11-18][answers], the accepted [cross-vendor review and its disposition][handover],
the [nested-clone research][nested], and the [checkout-dataset and pnpm research][pnpm]. The Rust binding vet,
Btrfs design, wrapper specification and probe records support specific parts of the design. This is a documentation
revision, not an implementation or a pool change. The build still owes the acceptance tests; the project still
owns the listed open choices.

## Requirements and module ownership

The project chose "ZED, Clones, reflink. There is a reason we have a doctor subcommand." The Git wrapper owns Git
validation and registration, the unprivileged snapshot and clone lifecycle, and the same-dataset reflink fallback.
The root-installed ZFS Event Daemon (ZED) handler owns clone mount and unmount, not Git or automatic deletion.
The Rust binding owns the `/dev/zfs` request format, not ownership policy for Git worktrees or snapshots.

The checkout stays at `~/Monochromatic`, on its own dataset after a one-time move. Every worktree of this repository
is directly under `~/Monochromatic/.claude/worktrees/` (`AGENTS.md` rule `WTU`); the account owns the directory and
each slot. Recursive home snapshots cover the checkout, clone datasets and the pnpm store, with explicit exclusions.

"Report and print commands (Recommended)" makes `git monochromatic-git-policy-cli doctor` read-only: it diagnoses
the root-once setup and prints concrete commands for the administrator without running them.

## Binding crate and compatibility

The owner chose "A Rust crate", "Kernel interface (Recommended)", "Out (Recommended)" for crates that run `zfs`,
and "Our own small binding (Recommended)" ([ZFS planning record][zfs-plan], lines 343 to 361).
Use `package/zfs/request-to-openzfs-kernel-device`, with Cargo package
`monochromatic-zfs-request-to-openzfs-kernel-device` and path-derived workspace name
`@monochromatic-dev/zfs-request-to-openzfs-kernel-device` where workspace naming applies. Its README explains
`/dev/zfs` and ioctl (kernel request) to a Git user. The location and names follow `AGENTS.md` rules `MKY`, `VGW`,
`AP1`, `AP2` and `SGD`; use the standalone Cargo workspace, path dependencies, `README.md`, package `mise.toml`,
[Rust dependency catalog][rust-catalog], and `uses/` layout (`LYD`). Reuse `libc` and, for profile digests, `sha2`.
Do not add a C build step, link or load OpenZFS's CDDL libraries, or invoke the `zfs` program from the binding,
wrapper or handler. The vet's "Where the wrapper ships and what it links today" explains why the wrapper must
start even when OpenZFS is absent. Administrative commands printed by `doctor` do not run inside the wrapper.

Build `mount-git-worktree-clone-datasets` as a separate Linux Rust binary target of `package/git-policy/cli`,
alongside `monochromatic-git-policy-cli`. Both use the same binding and compatibility gate, even for reads.
The handler uses child enumeration and dataset/property reads for its sweep and Linux mount calls for the actual
attachment and removal. It has no Git, pool administration, or dataset destruction operation.

### Requests and input ownership

The initial interface is Linux only. Expose typed operations, not a general-purpose ioctl or property setter.
Numbers are for OpenZFS `zfs-2.4.4`, `include/sys/fs/zfs.h:1502` to `:1550`.
The vet's "Result and the owner's decision" records the core set. The cross-vendor review checked
`SNAPSHOT_LIST_NEXT` at line 1505 and `GET_FSACL` at line 1525 of that header.

- `ZFS_IOC_OBJSET_STATS` (`0x5a12`): inspect dataset type, identity, origin, and the values and sources of properties.
- `ZFS_IOC_DATASET_LIST_NEXT` (`0x5a14`): enumerate immediate children via the kernel's cursor.
- `ZFS_IOC_SNAPSHOT_LIST_NEXT` (`0x5a15`): enumerate snapshots of one dataset for removal and recovery.
- `ZFS_IOC_SET_PROP` (`0x5a16`): change only the wrapper's desired state and sweep nonce on a named clone.
- `ZFS_IOC_DESTROY` (`0x5a18`): destroy one named, unmounted worktree dataset.
- `ZFS_IOC_SNAPSHOT` (`0x5a23`): create a named origin; the request names the pool and carries a `snaps` nvlist
  (the kernel's name/value list), not a recursive flag.
- `ZFS_IOC_GET_FSACL` (`0x5a29`): read delegations for `doctor`; inspect grants on ancestors too.
- `ZFS_IOC_DESTROY_SNAPS` (`0x5a3b`): destroy an explicit snapshot name set, with no deferred destroy,
  preserving per-name failures.
- `ZFS_IOC_CLONE` (`0x5a42`): place the target in `zc_name` and send `origin` and optional `props` in the nvlist.
  The contract is in `lib/libzfs_core/libzfs_core.c:328` and `module/zfs/zfs_ioctl.c:3844` at `zfs-2.4.4`.

Do not ship `ZFS_IOC_PROMOTE` (`0x5a22`), which can change the snapshot dependencies that removal relies on.
There are no pool management, dataset rename, or generic dataset creation requests. Mounts use Linux system calls,
not a dataset ioctl.

Keep the unsafe call in one place. Own a zeroed, aligned `#[repr(C)]` `zfs_cmd_t` representation, including the union
and trailing fields. Use checked, NUL-terminated names, fixed-width integers for ABI addresses, and owned nvlist
buffers that stay live for the call. Treat returned sizes, cursors and nvlists as untrusted; distinguish end of list
from a failed read, bound decode and resize, and never replay a mutation merely because its outcome is unknown.
Return the operation, target, errno and per-name errors when supplied. Reuse selected codec, layout and fuzz code from
`Ukko-Ylijumala/zfsdev-rs` at `4559ffc711`, not its vendored CDDL references or live-pool suite. Preserve notices,
ship the applicable MIT and Apache-2.0 license texts and provenance (revision, original paths and modifications),
and annotate the imported files. The [vet][vet], lines 414 to 461, records that revision, license and missing clone.

### ABI gate and tests

The vet's BTF record (kernel type metadata), from module version `2.4.4-1`, reports `sizeof(zfs_cmd_t) = 13744`,
`zc_inject_record` at byte `13296` with size `368`, and the overlapping `zc_defer_destroy`, `zc_flags` and
`zc_action_handle` at `13648`, `13652` and `13656`. The equal overall size on `2.2.2` does not prove compatibility.
In the guest, compare Rust's compiled sizes, alignment and offsets with the running module's BTF and pinned
headers, including nested structs, array extents and union members. Use
`pahole --btf_base /sys/kernel/btf/vmlinux -C zfs_cmd /sys/kernel/btf/zfs` as the documented inspection
and compare fields in a test, not just the printed total. Verify the request values against the matching headers.
Evidence: [vet][vet], lines 429 to 449, and its `keep/running-kernel-zfs-cmd-layout-from-btf.txt`.

Before the first ioctl, both binaries must prove compatibility with the loaded module and target ABI, including
module release, architecture, endianness, request contract and kernel/module BTF layout. A missing device, missing
required BTF or mismatch disables raw requests, including reads; no force flag. Unverified architectures, including
`linux-arm64` until separately tested, still run the ordinary checkout route. Whether verification uses exact
approved BTF digests or compares fields at runtime is an [open choice](#open-project-choices). The gate must report
what failed, also when an update makes existing clones temporarily unavailable. A failed gate does not authorize
an unrelated mount or cleanup; keep registrations and durable recovery state intact.

Test the codec, request packing, bounded decoding, and the fail-closed gate without a device. Fuzz the adapted codec.
Run every live ioctl, migration, handler and fault test against a file-backed pool inside a disposable VM, never
`zpcachyos` and never with the host's `/dev/zfs` passed through. Test as root for installation and as an account with
only the proposed grants for normal requests; a root-only pass proves nothing about delegation. Check codec native
encoding, cursor behavior, property sources, output resize and per-name errors. The full field match against the
new crate, clone properties at creation, effective delegation and release-archive linkage remain unverified.

## Dataset and path layout

Under `zpcachyos/ROOT/cos/home`, put two sibling datasets: `monochromatic-git-checkout`, mounted at
`/home/user/Monochromatic` (the account's `~/Monochromatic`), and `monochromatic-git-worktree-clone-datasets`,
an unmounted container with `mountpoint=legacy`, `canmount=off`, `setuid=off` and `devices=off`.
Each managed clone is an immediate child of the container and mounts at the corresponding account-owned
`~/Monochromatic/.claude/worktrees/<name>`. There is no intervening filesystem or directory at that slot.
Home's sanoid recursion reaches both siblings and the clones; a checkout snapshot includes the slots' underlying
entries, not the live clone bytes. Each clone needs its own snapshots ([nested-clone research][nested], items 1 and 4).

Clones must inherit `mountpoint=legacy`. The container's `canmount=off` is local; the probe saw `canmount=on` on
its child. No clone gets an automatic mount or an `fstab` entry. `zfs mount -a` and the systemd ZFS mount generator
skip legacy datasets; an ordinary clone mountpoint would reopen the early-mount hazard even if only root could
set it. See [nested-clone research][nested], item 2, and OpenZFS `zfs-2.4.4`,
`lib/libzfs/libzfs_mount.c:295` to `:297` and `etc/systemd/system-generators/zfs-mount-generator.c:430` to `:432`.

### One-time move and checkout boot protection

Root moves the checkout once with every session in it stopped. Verify a recoverable backup and inventory linked
worktrees, nested mounts and required free space. Measure logical bytes on the source filesystem, excluding nested
mounts, before budgeting the copy; the [planning record][answers] measurement includes compressed allocation and
nested data and is not the size of the copy. Copy across datasets preserving ownership, permissions, times, links,
ACLs and xattrs. Stage and verify the new dataset before switching the mount at the same checkout path. Preserve
nested mounts separately, including `zpcachyos/nosnap/vm-builder-output` at `package/vm-builder/cli/output`.
That output dataset remains under `nosnap` in the ZFS tree; only its mount path is inside the checkout. Never copy
its contents into the checkout or write through its mountpoint during the move. Verify the Git object database,
tracked, untracked and eligible ignored state, and each linked pointer and back-link. The paths do not change,
so no consumer rewrite or Git repair is needed solely for relocation. Keep the original tree and a tested rollback
until the checkout, linked worktrees and nested mounts pass verification. Reopen watches on the replaced directory.

The account can replace `~/Monochromatic` while it is unmounted, regardless of the entry's owner. Never use an
unprotected privileged mount by path. The [checkout boot protection choice](#open-project-choices) remains open;
the installation must prove the selected protection before enabling the checkout mount and make the checkout
available before account sessions and clone reconciliation. An absent or wrong checkout mount blocks clone attach,
including when pool import, encrypted key loading or mounting fails. Do not fall back to mounting clones into home.

### Scoped delegation

Root configuration pins the account UID, the checkout path, pool and dataset names and GUIDs. The proposed
unprivileged lifecycle needs `mount,snapshot` local to the checkout dataset, `clone,mount` on its descendants
(snapshots count as descendants), `create,mount` local to the clone parent, and `mount,destroy,userprop` on the
clone parent's descendants. Checkout-origin deletion is separately open: adding `destroy` on checkout descendants
lets the account destroy **any checkout snapshot**, not just wrapper origins, but not the checkout dataset itself.
Do not assume those snapshots protect against that account or put unrelated ZFS descendants there.

The `mount` grant is an ioctl prerequisite, not Linux mount-namespace authority. Do not grant `mountpoint`,
`canmount`, `setuid`, `devices`, `rename`, `promote` or permissions over the whole home tree.
OpenZFS `zfs-2.4.4` sources: `man/man8/zfs-allow.8:110` to `:120`,
`module/zfs/dsl_deleg.c:574` to `:582`, and `module/zfs/zfs_ioctl.c:826` to `:832` and `:1128` to `:1148`.

## Add and Git registration

Follow item 7 of the [nested-clone research][nested]: register while the slot is empty, mount at the slot itself,
then install the same Git pointer in the clone. Git accepts an existing empty slot, not a populated clone root.
The research's 2026-11-18 correction confirms account ownership for the registration probe. The actual guest
script (`keep/probe.ts:71` to `:91`) took the origin snapshot before registering and used `mount -t zfs` in that
sequence. The descriptor attach was a separate probe. The combined ordered wrapper flow is still an acceptance test.

1.  Share preflight with `doctor`. Before a claim, slot, registration, snapshot or clone is written, verify the
    supported Git form, eligible source and canonical destination, the checkout mount and caller's visibility,
    `/dev/zfs` access, ABI, effective grants, slot parent identity and owner, mount propagation, installed
    root-owned handler and configuration, live ZED, and usable root-owned status/results. Require a current
    handler readiness report for the actual pool, checkout and boot, not just a present executable. Check the
    clean source and capture its index/tree verification inputs. On refusal, name the check and choose the
    applicable fallback without leaving a managed object. Never substitute the main checkout for a linked source.
2.  Claim the destination as the account with an exclusive per-add record in the Git common directory. Write
    the proposed clone name and unique `git-worktree-clone-origin-<nonce>` snapshot name before any mutation
    beyond the claim. Create or verify an empty account-owned slot, then run Git's supported
    `worktree add --no-checkout` form. Record the reciprocal registration and exact `.git` pointer bytes; leave it in
    the slot on the checkout dataset. Have the handler verify and record the slot identity before any attach.
3.  Snapshot the checkout, then clone that fresh origin, never a sanoid snapshot. Record the actual snapshot and
    clone GUIDs as each is confirmed. Request `mount` via the local desired-state user property described in
    [ZED reconciliation and mount safety](#zed-reconciliation-and-mount-safety), not via `mountpoint`. Send it in
    clone `props` only if the delegated creation path is verified. If that path refuses and no clone exists,
    create it without `props`, then set the local property with `ZFS_IOC_SET_PROP`. If a matching clone already
    exists, verify it before setting the property and do not issue another clone request. If the outcome is
    unknown, inspect state before any retry. A clone without a valid local request is not mounted.
4.  Wait for the matching mount and verify it is visible and writable at the slot in the caller's namespace.
    The clone still holds the checkout's copied `.git` directory. Compare the frozen index and tracked content
    in the clone itself against the captured proof, without running Git against that directory. The in-snapshot
    check proves the bytes; the pre-snapshot check alone does not cover a write between check and snapshot.
    Discard copied Git administration before running Git inside the clone. Omit copied `.claude/worktrees`,
    wrapper caches and stages per the main-checkout leave-out rule, and omit nested mount locations identified
    by the source walk, including `package/vm-builder/cli/output`. Do not descend into, copy or mount at those
    boundaries. A snapshot omits nested mount contents but can carry real underlay files and stale Git stubs;
    remove only the verified clone-local copies, never the live mounts or source underlays.
5.  Install the saved Git pointer inside the clone. Use the verified index and the existing conversion checks
    and tree difference in the new worktree context. Verify the worktree root, common directory, reciprocal
    registration, mount identity and checked-out state. Mark the binding usable only after the fill is verified;
    then run or queue the existing dependency repairs. The wrapper and callers must not use a pending slot as
    a worktree. A mount can be externally visible before the fill; an interrupted add can expose copied `.git`,
    which is neither the linked registration nor a success signal.

The durable binding keys on repository and Git registration identity plus clone GUID, not spelling alone. Store
its expected slot path and identity, origin name and GUID, clone name, pointer bytes, request nonce and progress.
On resume distinguish a claimed empty slot, registration without a dataset (including a possible origin only),
an unmounted clone, a mounted but unfinished clone and a verified checkout. Recheck recorded identities, the mount
and the visible pointer before resuming; never use names, properties or a matching result alone to prove ownership.
If snapshot or clone returns `ENOSPC`, report the operation, object and pool, keep the claim, and clean up only
objects whose creation and identity are proven and whose destruction is allowed. A clone pins its origin even before
it is mounted. If mutation succeeded but its identity was not recorded before interruption, keep and report it
unless identity can be independently recovered. No silent fallback over an unresolved managed state. An explicitly
recovered Git-only add may finish at the same slot only after the mount and all outstanding intents are settled.

### Unmounted, pruned or moved worktrees

Keep the underlay pointer intact for `git worktree prune`, independent of the clone's pointer. It is a registration
stub, not a checkout. `doctor` and wrapper commands that encounter it distinguish a known unmounted clone (ZED off,
ABI refusal, mount failure or pending request) from a pending add or removal. Report the slot, dataset and next
safe action instead of treating absent files as deleted tracked files or offering an ordinary add over them.

A damaged link with surviving admin state can use `git worktree repair` from the main checkout, with the verified
slot passed explicitly. If Git already pruned the admin entry, repair is not a reconstruction of HEAD, index or lock
state. Restore verified registration data from backup before repairing links; if there is none, report what is lost
and require an explicit recovery or discard, never guessing from a branch or path. A removal whose recorded clone
and origin are already gone can settle a missing registration only after proving there is no replacement.

Refuse `git worktree move` for a bound managed clone whether mounted or not: report that the slot is fixed and
must not be renamed. The mounted guest control failed with `Device or resource busy`; a wrapper refusal also covers
an unmounted slot, where that kernel protection does not apply.

## ZED reconciliation and mount safety

Install the mount binary as a root-owned executable at
`/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets` for service calls, and install the same
build as `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets` for ZED events. Keep code, executable search
paths and `/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc` root-controlled. The handler runs in the
host mount namespace. It does not load code from the checkout, run Git, create or remove account slots, or widen
sandbox access to `/dev/zfs`.

### Requests, results and ordering

The durable mount intent is the clone's local `org.monochromatic:git-worktree-clone-mount-desired-state` property.
Accept exactly `v1:<nonce>:mount` or `v1:<nonce>:unmount`, with a nonce of 32 lowercase ASCII hex digits. Bound the
byte length before parsing. Ignore inherited intent and reject malformed intent before any filesystem operation on
its behalf. No request supplies a path, dataset name or mount flags. On an event, enumerate current immediate
children of the pinned parent, verify filesystem clone type, GUID and origin, inherited `legacy` and safety
properties, and derive the slot only from a single name. Accept lowercase ASCII letters and digits with interior
hyphens, bounded by the slot component limit and the remaining dataset-name budget. A malformed property, name or
origin never authorizes work on another mount. Recheck desired state and identities under a per-slot root claim;
parallel handlers can sweep at once but cannot take conflicting actions on the same mount.

Start a reconcile pass after pool import, required keys and the verified checkout mount, not merely after ZED
starts. Verify the expected checkout dataset and mount identity in the host mount namespace before each action.
No clone attaches while the checkout mount is missing or wrong. On shutdown the same handler owns clone unmounts,
with stop work ordered before checkout/home unmount and pool export. This matters because legacy clones have no
systemd mount unit and `zfs unmount -a` skips them. Report an expected checkout/home busy refusal by naming the
mounted clones; never force or detach them. Ordinary idle `zpool export` must still work. A busy clone may block
shutdown teardown or export; a residual mount is not a successful unmount. Boot must recover without relying on an
old ZED event queue. A recorded `unmount` remains in effect across restart until destruction or explicit reversal.
Periodic reconciliation without a waiter depends on the [lost-event choice](#open-project-choices).

Publish an atomic receipt in `/run/git-worktree-clone-mount-results`, using only the canonical numeric dataset GUID
as the receipt filename. The root-owned directory is mode `0755`; receipts are root-owned mode `0644`, read-only to
the wrapper, and opened relative to the trusted directory. Nonces appear only as validated data in the receipt,
never in a filename. Include the operation, matching nonce, outcome, mount identity and failed step/errno where
applicable. Readiness also records the current boot, pool, checkout and sweep state. The runtime files are a cache,
not authority to act; rebuild them from live state and durable intent. Do not write acknowledgement properties from
root. A waiter can change the local `org.monochromatic:git-worktree-clone-mount-sweep-nonce` once to wake a new sweep;
it uses the same hex format and is not an instruction to mount. The [ZFS planning probes][zfs-plan] show replay after a ZED stop and recovery of
a lost event on a later property change, not recovery from a lost last event with no new event.

### Descriptor mount and unmount

Apply item 3 of the [nested-clone research][nested] as the mount contract: "The mount must not resolve a path
that the account can change", "The mount must check the target folder's recorded identity, not just its kind",
"The mount must be attached to the opened folder, not to a path", and "What unmount needs, and where the probe was
incomplete". In this design root records identities, not ownership of the account's folders. At setup root records
the identity of `.claude/worktrees`. For each new slot root observes and records the verified account-created
underlay before attach; it must match the claimed registration slot and hold only the expected Git pointer.
Record and recheck device, inode, UID, permissions and access/default ACLs from the opened folders. Keep records
under root control in `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`. Never refresh an
existing slot identity just because a new inode or permissions appeared at the expected path.

Use the research's descriptor-relative `openat2` constraints, `fsmount` with enforced `nosuid,nodev`, and
`move_mount` onto the opened target. Report the failing component or mismatched identity field. After attach, prove
both that the held identity received the expected mount and that the canonical slot exposes that mount ID. A moved
inode can receive the mount at another account-controlled location. Treat that as a refusal, not a worktree: roll
back the exact attached mount by descriptor, or report and retain its identity and both paths if rollback cannot
finish. Never substitute a path-based attach or unmount on the account-controlled tree.

For unmount, prove the mounted identity, move the mount by descriptor to a root-owned private staging mount
hierarchy, and unmount it there without force or lazy detach. Require private checkout mounts and ancestors and
verify staging propagation before action. On a propagation mismatch refuse and report drift; whether to support
administrative isolation on hosts with shared ancestors is open. If the final unmount is busy after the move, restore
only the same mount to the revalidated original slot if safe. Otherwise retain and report its staged identity and
location. Track a staged move across handler interruption so no subsequent pass treats a mount missing from the
slot as gone. Publish success only after checking actual mount state, not path absence. The runtime test must prove
this for busy mounts and propagation peers; the probe did not combine the descriptor move with a busy unmount.

### Waiting and refusal

Use a monotonic deadline and bounded waits; a matching receipt must still be confirmed by the caller's mount view.
Report a hidden mount separately from a timeout. The wrapper's 5 s `REPORT_SCAN_CEILING`
(`package/git-policy/cli/src/native/service/wait_ceilings.rs:1265` to `:1280`) is a guest trial budget, not a ZED
latency finding or a shipped limit. Measure the mount and unmount budgets using the acceptance workload.

Name the failed operation, dataset, slot, actual state and reason. For `EBUSY`, name the holder if known, or explain
open handles and working directories; tell the caller to leave the mount and retry from another checkout. For
staging or rollback failure, give the staged location and administrator recovery instructions. For timeout, show
the last observation, the matching request and how to inspect the handler with `doctor` or the journal. A deadline
is not cancellation: the durable request may act later. Do not retry against an unverified mount or submit a
conflicting request without verifying and recording recovery.

## Removal and recovery

Reuse the wrapper's Git registration/back-link, lock, clean and held-directory boundary checks. One supported
`--force` can waive cleanliness, not a lock, mount boundary or identity check. Inspect the mount tree and the dataset
tree separately for nested worktrees, datasets and mounts, including binds; inspect the Git registration too. If an
inspection cannot prove its result, refuse instead of falling through to Git's recursive removal for a managed
slot. See the [wrapper specification][spec], lines 2932 to 2943 and 3064 to 3190, and `find_boundary_below` in
`package/git-policy/cli/src/native/subvolume/subvolume_remove_walk.rs`.

1.  Record the removal intent and the affected snapshot history, including manual and sanoid snapshots. Name that
    history as data this removal will destroy. Release local handles that keep the clone busy; request `unmount`.
    Wait for the matching result, then verify the clone is not mounted, including in handler staging, and that the
    request, recorded clone GUID, underlay identity and Git registration still match. An unavailable, mismatched
    or busy unmount blocks destruction; mount absence at the slot alone does not prove it is safe.
2.  Recheck the named clone's snapshots and children. Destroy only that clone's explicitly enumerated snapshots,
    then the unmounted clone. Never pass a recursive dataset or dependent-clone destroy. If sanoid adds a snapshot
    during removal, report the changed set and recheck the deletion authorization before continuing. Holds or
    dependencies are named, not bypassed. Record partial deletes, including per-snapshot failures.
3.  Check the exact recorded origin identity before attempting single-snapshot cleanup, using the authority the
    project selects. The kernel's `snapshot has dependent clones` refusal is the check, not a scan of immediate
    children or a name test. Leave a refused origin and report it; never destroy a dependent clone to free it.
    Preserve the incomplete cleanup record even if the clone and Git registration can now be settled.
4.  After clone destruction is proven and origin cleanup is attempted or recorded as blocked, settle only the
    matching Git registration and pointer. Remove the leftover underlay folder as the account only after checking
    it has the recorded identity and is empty. Do not remove an unfamiliar entry, a replacement registration or
    any mounted tree. Legacy dataset destruction does not remove the slot. Older checkout snapshots can still
    hold that underlay and pointer until they expire; this is not surviving live worktree data.

Resume by reconciling the durable record with the mount table, dataset and snapshot identities, and Git
registration and pointer. Distinguish unmount still pending or staged, clone still present after partial snapshot
cleanup, clone gone with a retained origin, and Git or empty-folder cleanup still owed. Report specific blockers:
`EBUSY` and holder if known, snapshot history at risk, a hold, nested dataset/mount, dependent clone, a mismatched
identity or a missing/pruned registration. Never infer ownership from a name or let a stale acknowledgement authorize
cleanup. ZFS destroy is name-based; rechecking its GUID is not atomic with the destroy. The design does not claim to
exclude a concurrent replacement by the same delegated account between check and use. Test replacement, partial
removal and resumption in the guest.

## Ladder and fallback

**Rung 1** is the ZFS snapshot clone alongside the existing Btrfs route. Keep `git worktree add <path>` semantics
and the ladder's already recognized forms ([wrapper specification][spec], lines 1121 to 1189). The described add
uses the configured clean main checkout as origin. A dirty or otherwise unproved source is refused for this rung;
this is not Btrfs's dirty-main restore. Reuse held-path and mount-fact helpers and the current Git index, attribute,
conversion and publication checks. Respect Btrfs's main-checkout leave-out policy for ignored state, but verify
ZFS-specific mounts and identities instead of assuming Btrfs's inode/subvolume semantics. If tracked output would
occupy an omitted mount location, refuse the clone fill rather than writing through a boundary. See the
[Btrfs design][btrfs-plan], "Implemented flow and what superseded the proposal".

**Rung 2** uses `FICLONE` of eligible files only on a proven same dataset, with the existing clean source, held
files and index checks. Add ZFS recognition where `fs_kind.rs` and `worktree_snapshot_source.rs` currently limit
the route. The main checkout and an ordinary slot on its filesystem, under `.claude/worktrees/`, can use it.
A mounted clone and a slot in the checkout dataset cannot, and a path elsewhere in home is not on the checkout
filesystem. Choose the applicable fallback at the same requested slot if managed preflight fails, confinement
blocks cloning, or a valid slot does not meet the clone-name grammar and no managed mutation has started.
Do not recast an ordinary byte copy as a successful reflink. **Rung 3** is the existing Git checkout and ignored
state behavior when a reflink is refused, cross-dataset, or unproved. In particular, `FICLONE` returns `EXDEV`
across datasets, but a successful `copy_file_range` can share blocks across datasets with the same data key.
An encryption root alone is not the same key; this does not qualify a cross-dataset copy for rung 2 ([pnpm study][pnpm],
"When a copy still shares blocks").

The Pi driver already creates its worktree before starting the confined unit
(`package/pi-shared/subagent-driver/src/start-pi-run-in-a-worktree.ts:238`). The planning probes found a confined
user namespace could not see the source dataset through `/dev/zfs`, while same-dataset reflinking worked.
A read-only bind of the worktrees parent can hide a mount made after a unit starts. Do not relax confinement
or infer mount visibility from host success. Prefer creating the worktree outside the unit as the driver already
does, and verify visibility and writability in the consumer's actual context.

### Walks and Git clean

Use the wrapper's read-only boundary walk for the clean proof. Stop at mounted filesystems and exclude the
ignored `.claude` subtree; never hand `git clean` a path under `.claude/worktrees` or force cleanup across a mount.
A mount or a tree containing one is a refusal for destructive cleanup, not an ordinary ignored directory.
The guest's single-force removal case ran with the clone's `.git` moved aside; double-force dry runs also show
why ordinary nested-repository protection is insufficient. Git ignore affects `status` and ordinary `rg`, not
walkers asked to ignore nothing. For tooling that inventories the checkout without descending into mounts, use
`rg --one-file-system`, `find -xdev` or `du --one-file-system` as applicable. The wrapper must also detect bind
mounts, which a device-only test can miss ([nested-clone research][nested], item 5).

### Dependency repair and watchers

Keep the pnpm store in home at `${HOME}/.local/share/pnpm/store`. With `virtualStoreType: global` and
`packageImportMethod: clone-or-copy` (`pnpm-workspace.yaml`), package payload moves from `files` to `links` inside
the store, not across the boundary into the checkout. The research traced zero imported regular package files
and zero package-payload bytes in the cross-dataset checkout. Paired offline installs took 0.984 to 1.101 times
their home controls, comparable to the home control's 1.091 spread, not proof of no cost. The project-local store
positive control really did copy across the boundary. The measured treatment was a stand-in, not the migrated
checkout or built clone route ([pnpm study][pnpm], "Where pnpm writes" and "Offline install cost on the host").

- A clone inherits relative store links at the source's depth. Keep the wrapper's
  `pnpm install --offline --frozen-lockfile` repair when a lockfile and real `node_modules` are present, queued
  when the service answers and inline otherwise. Skip the separate ignored-file copy only after successful clone fill; raw clone symlinks
  were not rebased by the ignored-copy path. The ordinary path still stages and publishes ignored files using its
  existing reflink, hard-link and explicit copy fallbacks. A clone source or mounted destination can separate
  source, stage and destination. Rebased links alone do not repair a path-specific global-store graph.
- Cargo intermediate output already lives under `~/.cargo/build` on a different dataset, keyed by workspace path.
  Keep external-slot prewarm and workspace-member cleanup; inherited `target/` does not prove those artifacts safe
  to reuse. Final-artifact link-to-copy fallback already crosses that boundary. The repair's displaced lockfile
  between a true clone and the common Git administration uses the cross-filesystem fallback, not an atomic swap
  ([wrapper specification][spec], lines 4761 to 4796 and 4843 to 4847).
- `mise` still runs tasks in the checkout and delegates toolchain install to `mpm-shadow`. Keep repair of
  `.cache/toolchain-launchers` pointing at another checkout's `toolchain.lock.json` and the existing frozen-lockfile
  reinstall when needed ([wrapper specification][spec], lines 4888 to 4895). A task writing a file across the
  test boundary worked; the research did not do a full toolchain install there.
- A watch on the old checkout inode or an unmounted slot does not see writes on the newly mounted filesystem.
  Reopen or rescan after migration or clone mount; install watches on the mounted tree. Ordinary editor discovery
  and ignore policy are not verified by the guest's inotify test. Exercise the editor in use after cutover.

## Snapshots and setup reporting

"Worktrees are not necessarily throwaway." Keep the home hourly template with `recursive = yes`. It reaches
both sibling datasets and each clone by dataset ancestry, not by mount path. Keep the pnpm store in home and
snapshotted. Use the eight exclusions named in the [planning decision][answers]; the option's label miscounted,
not its list. A directory can be excluded only once it is its own dataset.

```ini
# /etc/sanoid/sanoid.conf (affected sections only)
# Use yes, not zfs. Put exclusions after every recursive ancestor's section.
# /home
[zpcachyos/ROOT/cos/home]
  use_template = home
  recursive = yes

# /var/cache
[zpcachyos/ROOT/cos/varcache]
  autosnap = no

# Unmounted container (mountpoint=none)
[zpcachyos/nosnap]
  autosnap = no

# /home/user/.cargo/build
[zpcachyos/nosnap/cargo-build]
  autosnap = no

# /home/user/.cargo/registry
[zpcachyos/nosnap/cargo-registry]
  autosnap = no

# /home/user/.npm
[zpcachyos/nosnap/npm]
  autosnap = no

# /home/user/.cache
[zpcachyos/nosnap/user-cache]
  autosnap = no

# /home/user/Monochromatic/package/vm-builder/cli/output
[zpcachyos/nosnap/vm-builder-output]
  autosnap = no

# /var/lib/libvirt/images
[zpcachyos/nosnap/libvirt-images]
  autosnap = no
```

Set `com.sun:auto-snapshot=false` on each exclusion, including `/var/cache`, which lacked it in the probe.
This marker does not implement sanoid's exclusion. Sanoid reads sections in file order: an `autosnap = no`
section before a recursive ancestor can be overwritten, and `recursive = zfs` offers no per-child exclusion.
Both versions tested reproduced the order effect. The current exclusions are outside the home subtree; name and
order them explicitly so a later move under recursion does not silently include them ([nested-clone research][nested],
item 4, and [planning record][answers]).

Each living clone pins its checkout origin, retaining blocks changed in the checkout since the snapshot. Persist
and report the exact origin and measured retention where available. Checkout snapshots contain only underlays,
including reflink/Git worktree data when those slots are ordinary directories; they do not contain live clone
contents. Clone snapshots can capture an unfinished add or removal and are deliberately lost when that clone is
removed. Neither hourly snapshots nor a mounted path imply a usable registration.

`zfs-space-guard` currently prunes only `<dataset>@autosnap_*` of its configured dataset (default
`zpcachyos/ROOT/cos/home`, `/usr/local/sbin/zfs-space-guard:7` and `:44`). That cannot prune descendant hourlies or
`git-worktree-clone-origin-` snapshots, and cannot free blocks retained only by them. Leave guard expansion an
[open choice](#open-project-choices); it is not implicit in making home recursive.

## What doctor checks and prints

Extend the read-only filesystem report (`package/git-policy/cli/src/native/health/doctor_filesystem.rs:1` to `:30`).
Use the same facts as managed preflight. Distinguish healthy, known drift and unreadable or unsupported checks;
name which add route remains available. Setup findings go to standard error and do not change `doctor`'s exit
status. `doctor` does not start a service, create a test dataset, repair registrations or change configuration.

- Check `/dev/zfs`, the ABI, the pinned pool and dataset identities, clone parent and clone properties, checkout
  mount, checkout boot protection, propagation and caller visibility. For the root-once move print the verified
  cutover and rollback steps, commands to create the datasets and install the protection selected by the project,
  and paths with the actual owner and expected properties. Do not propose chowning the account's slots to root.
  On ABI refusal name the loaded module and target plus the supported compatibility evidence, not a bypass.
- Check recorded parent and slot identities and name the failing field or path component. Inspect effective
  delegation including ancestor grants. Print scoped commands such as `zfs allow -l -u` and `zfs allow -d -u`
  with the actual account and datasets, only for grants in the selected policy. Do not grant origin cleanup more
  broadly just to make an interrupted command pass.
- Check the handler installation, config and root-owned state and result directories, ZED and service wiring,
  latest readiness and reconciliation, and unresolved requests. Print root install/enable commands and the
  journal query for the actual unit. Show other enabled zedlets before recommending ZED enablement, including
  applicable syslog, notification, enclosure or scrub effects.
- Compare the measured dataset tree and store location against recursive coverage. Verify the named exclusion
  sections, ancestor order, mount paths and `com.sun:auto-snapshot=false`; print the concrete INI correction or
  `zfs set com.sun:auto-snapshot=false <verified-dataset>` for confirmed drift. This keeps the planning record's
  veto-open adoption of blocklist checks. Report the space guard's actual scope.
- Report unmounted and staged clones, busy checkout teardown with the holding clones named, interrupted actions,
  damaged or pruned Git registrations, and each living clone's recorded origin. Report snapshot retention using
  named ZFS accounting fields where measurable; do not sum shared blocks as if each clone owned them. Explain
  retained underlays in checkout snapshots. Print the applicable guarded resume or link-repair steps, not a
  blanket destroy or a guessed registration.
- Report whether the configured backup job can see the checkout and whether filesystem boundaries are known to
  be enforced. A walker that crosses into a clone may protect the same contents again under the checkout path.
  `kopia-snapshot-all.service` was inactive with a missing executable here; another host's recorded ignore
  policy is not evidence for this one. Report unknown coverage as unknown until tested, not as a backup failure
  ([nested-clone research][nested], item 5).

The command text, service ordering and findings are tested in a disposable VM before presenting commands as
working setup. This proposal has neither installed nor run the root-once commands.

## Open project choices

The choices do not reopen the checkout path, account ownership, ZED, the fallback or recursive snapshot policy.
No option here is selected by this revision.

- Checkout boot mount protection. Keep an ordinary checkout mountpoint and use `chattr +i` on the covered entry,
  if the guest proves that the flag survives rename, unlink and reboot checks; or use `mountpoint=legacy` for the
  checkout with a dedicated descriptor-based boot attach that validates the underlay identity. The first keeps
  ordinary ZFS boot ordering but needs immutable-entry and rollback proof. The second keeps privileged attachment
  off the pathname but adds a checkout boot service. Both must reject an unverified target before account sessions.
- Shared ancestor policy. Refuse on a shared ancestor and ask the administrator to restore private propagation;
  or support an explicit, administrator-configured isolation arrangement after proving checkout/worktree visibility,
  staging and unmount behavior with affected consumers. Neither option silently changes propagation or lets the
  handler operate on an unverified shared hierarchy.
- Managed path interface. Accept the canonical path from ordinary `git worktree add <path>` only, with fallback at
  the same slot; or also add an opt-in named allocator that reports the chosen path under `.claude/worktrees`.
  The allocator could simplify driver integration but adds a caller-facing command.
- Eligible clone origins. Keep only the main checkout as a snapshot source; or also permit linked clone origins.
  A linked origin preserves that worktree's ignored state but chains dependencies and complicates retention and
  removal. Do not substitute a different origin from the one the caller supplied.
- ABI verification. Require exact approved kernel/module BTF profiles per target, or verify the field layout at
  runtime against the approved release/request contract. Exact profiles can disable clones after a rebuild;
  runtime checking can accept more builds but adds a parser and a larger compatibility test surface.
- Lost last event without a waiter. Reconcile on startup, events and caller nudges only, or add a periodic scoped
  sweep. The first avoids idle scans but delays recovery until another trigger. The second needs scan-cost and
  recovery measurements before choosing a shipped interval.
- Origin cleanup authority. Add descendants-only `destroy` delegation on the checkout, or use a separate root-owned
  cleanup operation limited to proven wrapper origins. Delegation keeps privilege out of the handler but allows
  the account to destroy any checkout snapshot. Separate cleanup needs its own privileged ownership and recovery
  proof; it is not implied by the ZED mount handler.
- Space-pressure pruning. Keep the guard's current scope and report its limits, or extend it to an explicit
  managed-descendant allowlist for sanoid hourlies with newest-per-dataset retention. Expansion can release more
  blocks, but deletes some living worktree history before normal expiry; measure what it would reclaim first.

### Measurements and checks still needed

- On the VM's file-backed pool, run the wrapper end to end. Cover recognized and rejected add forms, clean and
  dirty sources, ignored/staged data and nested mounts, birth with local clone properties, boot import/key delay,
  dropped events including after a caller exits, busy unmount, caller confinement, GUID/name replacement,
  held snapshots, and sanoid racing removal. Include an ordinary Git and a same-dataset reflink control.
- Measure `pnpm install` with the same lockfile and store in home, interleaving the same-dataset control and the
  moved main and clone datasets. Record cold and warm store/cache state, elapsed stages, clone attempts and
  copies, link targets, and dataset/pool space deltas. `packageImportMethod: clone-or-copy` and
  `virtualStoreType: global` (`pnpm-workspace.yaml:548` and `:591`) make cross-dataset copy plausible, but links to
  `<storeDir>/links` need no copy into the clone. Count both and distinguish paths, physical blocks and timing.
- Measure an hourly interval containing real builds in a **living** worktree. Attribute changes and retained bytes
  to `node_modules`, `dist`, caches and other outputs; contrast a no-build interval and the separate cost of add and
  remove. Check bytes pinned by the origin, what sanoid and the guard can free, and what remains after removal.
  The planning page's `17:00` to `18:00` sample was dominated by creation and deletion and had no build in a living
  worktree ([ZFS planning record][zfs-plan], lines 386 to 425). It cannot answer this question.
- Check that the backup includes the new mount paths and root-once config, that recovery works with origin snapshots
  still in use, and that a failed migration can return to the previous Git registration layout. None was tested here.

### Relationship to the planning record

Keep the recommended clones, ZED, reflink fallback and root-owned parent. Replace the recommendation's `zfs`
program driver with the owner's Rust `/dev/zfs` decision. Do not reopen sudoers, a mount broker or overlay as the
production mount mechanism. The probe's ordinary inherited mountpoint and result property demonstrated the workflow,
not the final boot or trust contract; this design instead proposes inherited `legacy`, persistent desired state,
root-owned results and startup/periodic reconciliation. It specifies a safe managed path rather than privileged
mounting at arbitrary `git worktree add` paths. Snapshotting living clones follows the owner, not the draft's
throwaway framing; removal deliberately deletes their own snapshots and only their confirmed origin. Performance,
mount-path validation and boot behavior for this exact design have not been demonstrated.

[answers]: cli-git-zfs-worktree.md#owners-answers-of-2026-11-18
[zfs-plan]: cli-git-zfs-worktree.md
[handover]: ../handover/cli-git-rust-implementation.md
[nested]: cli-git-zfs-clones-mounted-under-the-checkout.md
[pnpm]: cli-git-zfs-checkout-dataset-and-pnpm.md
[vet]: ../audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md
[btrfs-plan]: cli-git-btrfs-worktree.md
[spec]: ../../package/git-policy/cli/SPEC.md
[rust-catalog]: ../decision/rust-dependency-catalog.md
