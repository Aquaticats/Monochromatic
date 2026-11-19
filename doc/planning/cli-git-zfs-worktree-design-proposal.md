# ZFS worktree design proposal for the Git wrapper

**Non-final draft by an agent, unapproved.** This revision (2026-11-18T20:40:28-05:00) replaces the previous draft's
root-owned anchor, `checkout` subdirectory and directory exchange with the checkout at its existing path and
account-owned slots mounted directly over their Git registrations. It folds in the project's decisions in
[Owner's answers of 2026-11-18][answers], the accepted [cross-vendor review and its disposition][handover],
the [nested-clone research][nested], and the [checkout-dataset and pnpm research][pnpm]. The Rust binding vet,
Btrfs design, wrapper specification and probe records support specific parts of the design. This is a documentation
revision, not an implementation or a pool change. The proposal still owes pre-acceptance measurements, the build
still owes acceptance tests, and the project still owns the listed open choices.

## Requirements and module ownership

The project chose "ZED, Clones, reflink. There is a reason we have a doctor subcommand." The Git wrapper owns Git
validation and registration, the unprivileged snapshot and clone lifecycle, and the same-dataset reflink fallback.
The root-installed ZFS Event Daemon (ZED) handler and its lifecycle unit own clone mount and unmount, not Git
or automatic deletion.
The Rust binding owns the `/dev/zfs` request format, not ownership policy for Git worktrees or snapshots.

The checkout stays at `~/Monochromatic`, on its own dataset after a one-time move. Every worktree of this repository
is directly under `~/Monochromatic/.claude/worktrees/` (`AGENTS.md` rule `WTU`); the account owns the directory and
each slot. Recursive home snapshots cover the checkout, clone datasets and the pnpm store, with explicit exclusions.

"Report and print commands (Recommended)" makes `git monochromatic-git-policy-cli doctor` read-only: it diagnoses
the root-once setup and prints concrete commands for the administrator without running them.

## Binding crate and compatibility

The owner chose "A Rust crate", "Kernel interface (Recommended)", "Out (Recommended)" for crates that run `zfs`,
and "Our own small binding (Recommended)" ([ZFS planning record][zfs-plan], lines 346 to 364).
Use `package/zfs/request-to-openzfs-kernel-device`, with Cargo package
`monochromatic-zfs-request-to-openzfs-kernel-device` and path-derived workspace name
`@monochromatic-dev/zfs-request-to-openzfs-kernel-device` where workspace naming applies. Its README explains
`/dev/zfs` and ioctl (kernel request) to a Git user. The location and names follow `AGENTS.md` rules `MKY`, `VGW`,
`AP1`, `AP2` and `SGD`; use the standalone Cargo workspace, path dependencies, `README.md`, package `mise.toml`,
[Rust dependency catalog][rust-catalog], and `uses/` layout (`LYD`). Reuse `libc`. Select any profile-checking
dependency only once the verification method is chosen.
Do not add a C build step, link or load OpenZFS's CDDL libraries, or invoke the `zfs` program from the binding,
wrapper or handler. The vet's "Where the wrapper ships and what it links today" explains why the wrapper must
start even when OpenZFS is absent. Administrative commands printed by `doctor` do not run inside the wrapper.

This draft puts `mount-git-worktree-clone-datasets` in `package/git-policy/cli` as a separate Linux Rust binary
alongside `monochromatic-git-policy-cli`, rather than making a second package. Both use the same binding and
compatibility gate, even for reads.
The handler uses child enumeration and dataset/property reads for its sweep and Linux mount calls for the actual
attachment and removal. It has no Git, pool administration, or dataset destruction operation.

### Requests and input ownership

The initial interface is Linux only. Expose typed operations, not a general-purpose ioctl or property setter.
Numbers are for OpenZFS `zfs-2.4.4`, `include/sys/fs/zfs.h:1502` to `:1550`.
The vet's "Result and the owner's decision" records the core set. This draft adds `OBJSET_STATS` for identity
and property sources, `SNAPSHOT_LIST_NEXT` and `DESTROY_SNAPS` for exact removal accounting, and `GET_FSACL`
for `doctor`. The cross-vendor review checked `SNAPSHOT_LIST_NEXT` at line 1505 and `GET_FSACL` at line 1525.

- `ZFS_IOC_OBJSET_STATS` (`0x5a12`): inspect dataset type, identity, origin, and the values and sources of properties.
- `ZFS_IOC_DATASET_LIST_NEXT` (`0x5a14`): enumerate immediate children via the kernel's cursor.
- `ZFS_IOC_SNAPSHOT_LIST_NEXT` (`0x5a15`): enumerate snapshots of one dataset for removal and recovery.
- `ZFS_IOC_SET_PROP` (`0x5a16`): change only the wrapper's desired state and sweep nonce on a named clone.
- `ZFS_IOC_DESTROY` (`0x5a18`): destroy one named, unmounted worktree dataset.
- `ZFS_IOC_PROMOTE` (`0x5a22`): name a clone in `zc_name`; preserve any conflicting snapshot name in the error
  (`module/zfs/zfs_ioctl.c:6350` to `:6355`). It is not a mount operation.
- `ZFS_IOC_SNAPSHOT` (`0x5a23`): create a named origin; the request names the pool and carries a `snaps` nvlist
  (the kernel's name/value list), not a recursive flag.
- `ZFS_IOC_GET_FSACL` (`0x5a29`): read delegations for `doctor`; inspect grants on ancestors too.
- `ZFS_IOC_DESTROY_SNAPS` (`0x5a3b`): destroy an explicit snapshot name set, with no deferred destroy,
  preserving per-name failures.
- `ZFS_IOC_CLONE` (`0x5a42`): place the target in `zc_name` and send `origin` and optional `props` in the nvlist.
  The contract is in `lib/libzfs_core/libzfs_core.c:328` and `module/zfs/zfs_ioctl.c:3844` at `zfs-2.4.4`.

The [project decided][answers] "B: Include promote". `zfs-promote(8)` says promoting a checkout clone makes
the checkout depend on the promoted dataset and moves the checkout's origin and all older snapshots into it.
Checkout history must never move into a worktree that ordinary removal can destroy. Neither normal add,
reconciliation nor removal calls promote; grant no `promote` delegation to the account. Keep any blocked origin
and report its dependency rather than promoting a managed clone to clear it.

Exercise the binding's promote operation in the disposable VM. A possible live use is explicit administrator
recovery onto a protected clone outside the removable worktree tree, never promotion of a disposable worktree
of the checkout. No production procedure is authorized by this draft. The [VM measurement record][measurements]
is testing dependency reversal, which snapshot names and GUIDs move, and what destruction then refuses. The
exact promotion and recovery rule waits on that result; adding linked origins does not by itself authorize promote.
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

The [project decided][answers] "A: Test plus runtime gate (Recommended)" for a changed ZFS module. Keep the BTF
test. At start, before the first ioctl, both binaries must prove compatibility with the loaded module and target
ABI, including module release, architecture, endianness, request contract and kernel/module BTF layout. A missing device, missing
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
Each managed clone is an immediate child of the container and mounts directly at the corresponding account-owned
`~/Monochromatic/.claude/worktrees/<name>`, which is the worktree root.
Home's sanoid recursion reaches both siblings and the clones; a checkout snapshot includes the slots' underlying
entries, not the live clone bytes. Each clone needs its own snapshots ([nested-clone research][nested], items 1 and 4).

Clones must inherit `mountpoint=legacy`. The container's `canmount=off` is local; the probe saw `canmount=on` on
its child. No clone gets an automatic mount or an `fstab` entry. `zfs mount -a` and the systemd ZFS mount generator
skip legacy datasets. The OpenZFS `zfs-2.4.4` source shows that explicit `zfs mount <clone>` refuses a legacy
mountpoint rather than deriving a path (`cmd/zfs/zfs_main.c:7240` to `:7248`, the mount branch). The runtime
control is pending in the [VM measurement record][measurements]. An ordinary clone mountpoint would reopen the
early-mount hazard even if only root could set it. See item 2 of the [nested-clone research][nested] and the
skip paths in `lib/libzfs/libzfs_mount.c:295` to `:297` and
`etc/systemd/system-generators/zfs-mount-generator.c:430` to `:432`.

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
unprotected privileged checkout mount by path. The [planning record][zfs-plan] notes the same path-substitution
exposure for existing dataset mountpoints inside account-owned home paths, including `~/.cache`, `~/.npm`,
`~/.cargo/build`, `~/.cargo/registry` and the vm-builder output. Hardening those existing mount targets is outside
this proposal; gating all mounts nested under the checkout on checkout readiness is not. The [checkout boot protection choice](#open-project-choices) remains open;
the installation must prove the selected protection before enabling the checkout mount and make the checkout
available before account sessions and clone reconciliation. Inventory **every dataset mounted under the checkout
path**, regardless of ZFS ancestry, including the independently mounted vm-builder output. No such dataset may
attach until the checkout mount is verified. An absent or wrong checkout mount must leave the home underlay free
of nested mounts and mountpoint folders, even when import, key loading or mounting fails. In the legacy option,
exclude every nonclone nested dataset from the automatic scan (candidate: `canmount=noauto`); for the vm-builder
output and any other nested dataset, order its explicit mount unit after and require the verified descriptor
checkout provider. Do not let the checkout mount cover a nested mount on the home underlay. The actual
`zfs mount -a` ordering and failed-checkout controls are pending in the [VM measurement record][measurements].

### Scoped delegation

Root configuration pins the account UID, the checkout path, pool and dataset names and GUIDs. The proposed
unprivileged lifecycle needs `snapshot` local to the checkout dataset, `clone` on its descendants (snapshots count
as descendants), `create,mount` local to the clone parent, and `mount,destroy,userprop` on the clone parent's
descendants. Checkout-origin deletion is separately open: adding `mount,destroy` on checkout descendants
lets the account destroy **any checkout snapshot**, not just wrapper origins, but not the checkout dataset itself.
Do not assume those snapshots protect against that account or put unrelated ZFS descendants there.

The `mount` grant is an ioctl prerequisite, not Linux mount-namespace authority. Do not grant `mountpoint`,
`canmount`, `setuid`, `devices`, `rename`, `promote` or permissions over the whole home tree.
OpenZFS `zfs-2.4.4` sources: `man/man8/zfs-allow.8:110` to `:120`,
`module/zfs/dsl_deleg.c:574` to `:582`, and `module/zfs/zfs_ioctl.c:826` to `:832`, `:1005` to `:1037` and
`:1128` to `:1148`.

## Add and Git registration

Follow item 7 of the [nested-clone research][nested]: register while the slot is empty, mount at the slot itself,
then install the same Git pointer in the clone. Git accepts an existing empty slot, not a populated clone root.
The research's 2026-11-18 correction confirms account ownership for the registration probe. The actual guest
script (`keep/probe.ts:71` to `:91`) took the origin snapshot before registering and used `mount -t zfs` in that
sequence. The descriptor attach was a separate probe. The register-first, descriptor-mounted order has never
run end to end with account ownership. The [VM measurement record][measurements] is testing the sequence before
acceptance; the built wrapper must still exercise it.

1.  Share preflight with `doctor`. Before a claim, slot, registration, snapshot or clone is written, verify the
    supported Git form, eligible source and canonical destination, the checkout mount and caller's visibility,
    `/dev/zfs` access, ABI, effective grants, slot parent identity and owner, mount propagation, installed
    root-owned handler and configuration, live ZED, and usable root-owned status/results. Require a current
    handler readiness report for the actual pool, checkout and boot, not just a present executable. Check the
    selected source eligibility policy and capture the index/tree verification inputs. On refusal, name the check and choose the
    applicable fallback without leaving a managed object. Never substitute the main checkout for a linked source.
2.  Claim the destination as the account with an exclusive per-add record in the Git common directory. Write
    the proposed clone name and unique `git-worktree-clone-origin-<nonce>` snapshot name before any mutation
    beyond the claim. Create or verify an empty account-owned slot, then run Git's supported
    `worktree add --no-checkout` form. Record the reciprocal registration and exact `.git` pointer bytes; leave it in
    the slot on the checkout dataset. Have the handler verify and record the slot identity before any attach.
3.  Snapshot the checkout, then clone that fresh origin, never a sanoid snapshot. Record the actual snapshot and
    clone GUIDs as each is confirmed. Request `mount` via the local desired-state user property described in
    [ZED reconciliation and mount safety](#zed-reconciliation-and-mount-safety), not via `mountpoint`. Send it in
    clone `props` only if the delegated creation path is verified. OpenZFS `zfs-2.4.4` creates the clone before
    applying `props` and attempts to destroy the new clone if property setting fails
    (`module/zfs/zfs_ioctl.c:3844` to `:3872`). This is not atomic: a crash between the steps may leave a clone
    without the request, and failed cleanup is not proof of absence. If the path refuses and inspection proves
    no clone exists, create it without `props`, then set the local property with `ZFS_IOC_SET_PROP`. If a matching
    clone already exists, verify it before setting the property and do not issue another clone request. If the
    outcome is unknown, inspect state before any retry. A clone without a valid local request is not mounted.
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
The account can delete the underlay pointer while the clone is unmounted. The nested probe's unlocked
registration was prunable with the pointer missing (`guest-exact-zfs.log:42` to `:44`).

`git-worktree(1)` documents a lock in the common Git directory to prevent pruning when a worktree is unavailable.
The candidate is to acquire it at registration and retain it through unmounts and restarts. A lock protects
registration, not the pointer itself or an account deliberately deleting Git admin state. A root-set immutable
pointer could hinder accidental deletion of that file, but adds privileged file handling in an account-owned tree,
does not protect Git admin state, and needs removal and parent-rename proof. Neither is selected pending the
[VM lock and missing-pointer controls][measurements].

A damaged link with surviving admin state can use `git worktree repair <verified-slot>` from the main checkout.
If Git already pruned the admin entry, repair does not reconstruct HEAD, index or lock state. Restore verified
registration data from backup before repairing links; if there is none, report what is lost
and require an explicit recovery or discard, never guessing from a branch or path. A removal whose recorded clone
and origin are already gone can settle a missing registration only after proving there is no replacement.

If the Git lock is adopted, `doctor` must distinguish the wrapper's recorded protection from an operator lock,
a missing or changed lock, and a missing pointer. Never treat the reason text alone as ownership or use
`--force` to bypass a lock. An operator or unexplained lock still refuses removal. A verified wrapper lock
would be released only at the authorized registration-cleanup step, after clone destruction and subject to the
origin-cleanup policy; a missing pointer still needs recovery. The lock procedure is pending with the mechanism.

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

### Boot and shutdown unit

ZED alone cannot cover boot and shutdown. Its event list (`zpool-events(8)`, "Events", and OpenZFS
`zfs-2.4.4` `include/sys/sysevent/eventdefs.h:100` to `:139`) has `pool_import`, `pool_export` and
`history_event`, but no checkout-mount-ready or pre-unmount/shutdown event. Import does not imply the keys
and checkout are ready; export is reported after the pool is exported, too late to unmount clones. Replaying
or polling after an import cannot provide the missing shutdown ordering. The proposed mechanism is the project's
ZED handler for in-session requests **plus** a systemd unit for boot and shutdown, not a second request channel.

Install the root-owned system unit `reconcile-git-worktree-clone-mounts.service` with `Type=oneshot`,
`RemainAfterExit=yes` and `WantedBy=multi-user.target`. For the ordinary checkout, require and order after
the protected checkout mount provider (the generated `home-user-Monochromatic.mount` or, where it mounts
the checkout, `zfs-mount.service`); for a legacy checkout, require and order after
`mount-monochromatic-checkout-by-descriptor.service`. The selected provider must require the actual home and
key-loading providers and order after them and `zfs-import.target`. Order reconciliation after those providers
as well. Run both modes in the host mount namespace. Order startup before `systemd-user-sessions.service`;
checkout-dependent account services must require the checkout provider too.

- `ExecStart` runs the installed `mount-git-worktree-clone-datasets boot-reconcile`. It verifies ABI, keys,
  checkout and selected propagation before reconciling current intent and publishing this boot's readiness.
  A failed prerequisite authorizes no attachment. Report per-clone failures without discarding the unit's
  stop action for mounts that did succeed.
- `ExecStop` runs `mount-git-worktree-clone-datasets shutdown-unmount`. Withdraw readiness and close admission
  to new ZED attaches before draining in-flight slot operations. Unmount the verified managed mounts only;
  do not change durable intent. Reverse stop ordering puts the sweep before the checkout/home providers
  stop and before key unload and pool export. Add `Before=umount.target`; wire any separate export job to
  run after the stop pass. Ordering does not turn a failed stop into a successful unmount.

Verify the live checkout before every event action, not just at service start. A busy clone can block normal
teardown or export: report which clone holds the mount, and never force or detach it. Legacy clones have no
systemd mount units and `zfs unmount -a` skips them. Idle export/import must still work, and boot must recover
without an old event queue. A recorded `unmount` remains in effect across restart until destruction or explicit
reversal. Periodic reconciliation without a waiter depends on the [lost-event choice](#open-project-choices).

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

This draft uses atomic `/run` receipts, not root-written acknowledgement properties, so a result does not create
another ZED event. Publish in `/run/git-worktree-clone-mount-results`, using only the canonical numeric dataset GUID
as the receipt filename. The root-owned directory is mode `0755`; receipts are root-owned mode `0644`, read-only to
the wrapper, and opened relative to the trusted directory. Nonces appear only as validated data in the receipt,
never in a filename. Include the operation, matching nonce, outcome, mount identity and failed step/errno where
applicable. Readiness also records the current boot, pool, checkout and sweep state. The runtime files are a cache,
not authority to act; rebuild them from live state and durable intent. Do not write acknowledgement properties from
root. A waiter can change the local `org.monochromatic:git-worktree-clone-mount-sweep-nonce` once to wake a new sweep;
The nudge uses the same hex format and is not a mount instruction. The [ZFS planning probes][zfs-plan], lines 208 to 213, saw replay after restarting ZED and recovery when a user property on the parent was set. They did not
show recovery of the lost last event without another trigger.

### Descriptor mount and unmount

Apply item 3 of the [nested-clone research][nested] as the mount contract: "The mount must not resolve a path
that the account can change", "The mount must check the target folder's recorded identity, not just its kind",
"The mount must be attached to the opened folder, not to a path", and "What unmount needs, and where the probe was
incomplete". In this design root records identities, not ownership of the account's folders. At setup root records
the identity of `.claude/worktrees`. For each new slot root observes and records the verified account-created
underlay before attach; it must match the claimed registration slot and hold only the expected Git pointer.
Treat the Git claim and pointer as untrusted; check only a bounded, regular, no-follow pointer file and never
resolve a path from its contents as root. Record and recheck device, inode, UID, permissions and access/default
ACLs from opened folders. Keep records in `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`
under root control. Never refresh a slot identity just because a new inode or permissions appeared there. The
handler uses the verified slot descriptors for mount management, not privileged writes into the account's tree.

Only an administrator may explicitly adopt a legitimate replacement after checkout restore or folder recreation.
The installed root tool's `adopt-worktree-parent` mode runs with managed operations quiesced. Starting from the
pinned checkout mount, walk the canonical components with the no-crossing, no-follow resolver. Verify the intended
replacement's owner, mode and access/default ACLs against policy and the administrator's verified identity;
record them and the device and inode from the held descriptor. First reconcile all affected bindings against live
and staged mounts, claims and Git registrations. If any cannot be resolved, refuse adoption and report what needs
recovery. Never overwrite surviving slot identities or adopt a replacement for an existing binding implicitly.
Until adoption, refuse managed mutations on the mismatched parent. On an otherwise unused and verified slot,
ordinary same-dataset reflink (rung 2), or native Git when reflink does not qualify (rung 3), remains available
without root. Never use fallback to write over an unresolved managed slot.

The provisional kernel floor for the managed mount path is Linux `6.8.0-147-generic`, the build on which the
nested-clone guest exercised `openat2`, `fsopen`, `fsconfig`, `fsmount` and `move_mount` with OpenZFS 2.4.4
([nested-clone research][nested], "What was checked and how" and item 3). This is a tested baseline for those
calls, not a claim that every kernel at or beyond it supports the complete mechanism. Check the actual kernel
and required operations at handler readiness, separately from the ioctl ABI gate. If unverified, refuse managed
mounts and report the applicable reflink or Git route.

Use the research's descriptor-relative `openat2` constraints, `fsmount` with enforced `nosuid,nodev`, and
`move_mount` onto the opened target. Report the failing component or mismatched identity field. After attach, prove
both that the held identity received the expected mount and that the canonical slot exposes that mount ID. A moved
inode can receive the mount at another account-controlled location. Treat that as a refusal, not a worktree.
Record its mount identity and canonical and observed paths. Whether to attempt descriptor rollback immediately
or retain the mount for administrator recovery is an [open choice](#open-project-choices). Never substitute a
path-based attach or unmount on the account-controlled tree.

For unmount, the process that attached the clone is usually gone. The [VM measurement record][measurements] is
testing this candidate, not an established unmount mechanism: reopen the recorded parent from the verified
checkout with the same no-crossing resolver. Open only the final slot from that parent with `O_PATH | O_DIRECTORY`
and `RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS`. Omit `RESOLVE_NO_XDEV` only for that hop so
the open can cross into the attached mount. Verify that the descriptor is the mount root: compare its mount ID
and `st_dev` with the matching host `mountinfo` entry, which must show the expected slot, parent mount ID, ZFS
root and source. Compare the source's live dataset GUID with the binding and the observed mount ID with any
recorded ID for this boot. Refuse mismatches or ambiguous entries, including an overmount or a bind of a
subdirectory. Recheck the canonical lookup before moving; the move itself takes the verified descriptor, not a
lookup of the slot path. Move the attached mount to root-owned private staging and unmount there, without force
or lazy detach. The guest's move out of a shared checkout refused
with `EINVAL` (`guest-exact-zfs.log:230`); `move_mount(2)` identifies the source mount's immediate parent as the
constraint. Do not attach where safe unmount and staging propagation have not been verified. The exact propagation
checks and supported topologies are pending in the [VM measurement record][measurements]; see
[Shared ancestor policy](#open-project-choices). If final unmount is busy after the move, the choice of trying
reattach or retaining it staged is [open](#open-project-choices). Track and report staged identity and location
across handler interruption so no subsequent pass treats a mount missing from the
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

The [project decided][answers] "B: Destroy at once". A normal authorized removal destroys the clone and its own
snapshots in that invocation. It does not prompt, require a separate snapshot-deletion flag, or wait out retention.
It does wait for verified unmount within the handler deadline and for the destroy results, not for background
space reclamation.

1.  Record the removal intent and enumerate the clone's snapshot history, including manual and sanoid snapshots.
    Before destruction, print the slot, clone dataset and exact snapshot names to be lost. Release local handles
    that keep the clone busy and request `unmount`. Wait for the matching result, then verify the clone is not
    mounted, including in handler staging, and that the
    request, recorded clone GUID, underlay identity and Git registration still match. An unavailable, mismatched
    or busy unmount blocks destruction; mount absence at the slot alone does not prove it is safe.
2.  Recheck the named clone's snapshots and children. Destroy only that clone's explicitly enumerated snapshots,
    then the unmounted clone. Never pass a recursive dataset or dependent-clone destroy. If sanoid adds a snapshot
    during removal, report the changed set and recheck its identity and removal scope before continuing. Holds or
    dependencies are named, not bypassed. Print which snapshots and whether the clone were actually destroyed,
    including on partial failure; name anything retained and why. Record per-snapshot failures for recovery.
3.  Check the exact recorded origin identity before attempting single-snapshot cleanup, using the authority the
    project selects. The kernel's `snapshot has dependent clones` refusal is the check, not a scan of immediate
    children or a name test. Leave a refused origin and report it; never destroy a dependent clone to free it.
    Preserve the incomplete cleanup record while an origin remains.
4.  Settle only the matching Git registration and pointer after clone destruction is proven. Whether to settle
    them while origin cleanup is blocked remains an [open choice](#open-project-choices). Remove the leftover underlay folder as the account only after checking
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

### Rung 1: Snapshot clone

The ZFS snapshot clone sits alongside the existing Btrfs route. Keep `git worktree add <path>` semantics
and the ladder's already recognized forms ([wrapper specification][spec], lines 1121 to 1189). The described add
uses the configured main checkout as origin. An unproved source never qualifies. Whether to require a clean source
or support verified dirty-main restoration as Btrfs does is an [open choice](#open-project-choices). Reuse held-path
and mount-fact helpers and the current Git index, attribute, conversion and publication checks. Respect Btrfs's main-checkout leave-out policy for ignored state, but verify
ZFS-specific mounts and identities instead of assuming Btrfs's inode/subvolume semantics. If tracked output would
occupy an omitted mount location, refuse the clone fill rather than writing through a boundary. See the
[Btrfs design][btrfs-plan], "Implemented flow and what superseded the proposal".

### Rung 2: Same-dataset reflink

Use `FICLONE` for eligible files only on a proven same dataset, with the existing clean source, held
files and index checks. Add ZFS recognition where `fs_kind.rs` and `worktree_snapshot_source.rs` currently limit
the route. With the main checkout as the source, an ordinary destination slot in the checkout dataset can use
rung 2. A mounted clone at the destination cannot, because its files are on a different dataset. A destination
elsewhere in home, outside the checkout dataset, cannot either. A mounted clone used as the source also cannot
reflink to a slot in the checkout dataset. Choose the applicable fallback at the same requested slot if managed
preflight fails, confinement blocks cloning, or a valid slot does not meet the clone-name grammar and no managed
mutation has started. Do not recast an ordinary byte copy as a successful reflink.

### Rung 3: Ordinary Git

Use the existing Git checkout and ignored-state behavior when a reflink is refused, cross-dataset, or unproved. In particular, `FICLONE` returns `EXDEV`
across datasets, but a successful `copy_file_range` can share blocks across datasets with the same data key.
An encryption root alone is not the same key, and a cross-dataset copy does not qualify for rung 2.
See the [pnpm study][pnpm], "When a copy still shares blocks".

### Confined callers

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
  when the service answers and inline otherwise. Skip the separate ignored-file copy only after successful clone
  fill. Raw clone symlinks were not rebased. The ordinary path still stages and publishes ignored files using
  its existing reflink, hard-link and explicit copy fallbacks. A clone source or mounted destination can separate
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
order them explicitly so moving an exclusion under a recursive ancestor does not silently include it
([nested-clone research][nested], item 4, and [planning record][answers]).

Each living clone pins its checkout origin, retaining blocks changed in the checkout since the snapshot. Persist
and report the exact origin and measured retention where available. Checkout snapshots contain only underlays,
including reflink/Git worktree data when those slots are ordinary directories; they do not contain live clone
contents. Clone snapshots can capture an unfinished add or removal. The project's destroy-at-once policy removes
those snapshots with the clone and [reports the loss](#removal-and-recovery), without keeping a last snapshot.
Neither hourly snapshots nor a mounted path imply a usable registration. Checkout and clone snapshots
capture pnpm symlink texts, not their external targets. Restore needs compatible home-store contents; an offline
repair cannot fetch missing packages.

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
- Check recorded parent and slot identities and name the failing field or path component. For a legitimately
  recreated parent, print the administrator's verified `adopt-worktree-parent` command and prerequisites, any
  binding that needs separate recovery, and the applicable fallback for an unused slot. Inspect effective
  delegation including ancestor grants. Print scoped commands such as `zfs allow -l -u` and `zfs allow -d -u`
  with the actual account and datasets, only for grants in the selected policy. Do not grant origin cleanup more
  broadly just to make an interrupted command pass.
- Check the handler installation, config and root-owned state and result directories, ZED and service wiring,
  latest readiness and reconciliation, and unresolved requests. Print root install/enable commands and the
  journal queries for `zfs-zed.service` and `reconcile-git-worktree-clone-mounts.service`. Show other enabled zedlets
  before recommending ZED enablement, including
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
No option here is selected or ranked by this revision. At the owner handoff, present each decision separately
in the question tool with its options, pros and cons, and a recommendation that explains each adjacent pair
in the ranking. Use the pending measurements where they affect the options; a ranking is not a decision.

- Checkout boot mount protection. Keep an ordinary checkout mountpoint and use `chattr +i` on the covered entry
  (immutable-entry and rollback proof pending in the [VM record][measurements]); or use `mountpoint=legacy` with a
  descriptor boot service that verifies the checkout underlay. The first retains normal checkout mounting if all
  nested mounts can be gated on verified success. The second avoids a privileged checkout attach by pathname but
  needs a separate provider and explicit ordering of the vm-builder output and all other nonclone datasets
  under the checkout. Both must prevent automatic mounts into the home underlay if checkout mounting fails.
- Shared ancestor policy. On this host, `findmnt --output TARGET,PROPAGATION` read `shared` for `/`, `/home` and
  the ZFS mounts under `/home`, including `~/.cache` and the vm-builder output. The new checkout mount does not
  exist here yet; inheriting shared propagation from `/home` is an inference, not a reading of that mount.
  Options: refuse the managed route until an administrator makes the checkout and its ancestors private (no
  product-driven propagation change, but needs broad host changes here); make only the checkout mount private
  in the root-installed boot unit after verifying the checkout and before enabling clone mounts, reapplying on
  each boot (leaves `/` and `/home` alone); or support a different explicit administrator-configured isolation
  arrangement (can fit other consumers, but needs its own staging and visibility proof). A private checkout is
  expected not to propagate new clone mounts into another already-running mount namespace
  (`mount_namespaces(7)`); that could lose the visibility the planning probes saw without a read-only parent
  bind. The [VM measurement record][measurements] is testing both states. Do not silently change propagation.
- Managed path interface. Accept the canonical path from ordinary `git worktree add <path>` only, with fallback at
  the same slot; or also add an opt-in named allocator that reports the chosen path under `.claude/worktrees`.
  The allocator could simplify driver integration but adds a caller-facing command.
- Eligible clone origins. Keep only the main checkout as a snapshot source; or also permit linked clone origins.
  A linked origin preserves that worktree's ignored state but chains dependencies and can block origin cleanup.
  Neither option makes promotion automatic. Do not substitute a different origin from the one the caller supplied.
- ABI verification. Require exact approved kernel/module BTF profiles per target, or verify the field layout at
  runtime against the approved release/request contract. Exact profiles can disable clones after a rebuild;
  runtime checking can accept more builds but adds a parser and a larger compatibility test surface.
- Lost last event without a waiter. Reconcile on startup, events and caller nudges only, or add a periodic scoped
  sweep. The first avoids idle scans but delays recovery until another trigger. The second needs scan-cost and
  recovery measurements before choosing a shipped interval.
- Origin cleanup authority. Add descendants-only `mount,destroy` delegation on the checkout, or use a root-owned
  cleanup operation limited to proven wrapper origins. Delegation keeps privilege out of the handler but allows
  the account to destroy any checkout snapshot. Separate cleanup needs its own privileged ownership and recovery
  proof; it is not implied by the ZED mount handler.
- Space-pressure pruning. Keep the guard's current scope and report its limits, or extend it to an explicit
  managed-descendant allowlist for sanoid hourlies with newest-per-dataset retention. Expansion can release more
  blocks, but deletes some living worktree history before normal expiry; measure what it would reclaim first.
- Misplaced attach. Attempt immediate rollback of the exact mount by verified descriptor, retaining and reporting
  it only if rollback fails; or leave it attached and report both paths and its identity for administrator
  cleanup. Rollback can recover without intervention but adds another privileged mount operation in a race.
  Retention avoids that operation but leaves a mount stranded at an account-selected location.
- Busy unmount after a staging move. Attempt reattach of the same mount to the original slot only if it is
  revalidated and safe, retaining it staged otherwise; or keep it staged until administrator recovery. Reattach
  can restore the worktree's visibility but adds a second attach and race; retaining it keeps the mount under
  root control but leaves the worktree unavailable. Both require interruption tracking.
- Dirty source on rung 1. Require a clean source, using the applicable fallback for a dirty checkout; or implement
  a verified dirty-main restore comparable to Btrfs. A clean-only route narrows the proof but loses clone speed on
  a dirty main checkout. Dirty support needs a defined Git/index and ignored-state restore, with a frozen proof.
  Neither option accepts an unproved source.
- Git registration when origin cleanup is blocked. Keep the registration until the recorded origin is cleaned up;
  or settle the registration after clone destruction and keep an independent durable origin cleanup record. The
  first preserves Git visibility but reserves the slot until cleanup; the second frees the slot but requires
  recovery without its Git entry.

## Measurements due before design acceptance

The [planning record][answers] says of the account-owned slot design: "every refusal there is measured again with
account ownership before the design is accepted". That is a condition on accepting this proposal, not work to defer
until the wrapper is built. The [VM measurement record][measurements] is being prepared on
`docs/planning-cli-git-zfs-measurements-owed-before-acceptance`. Its results are pending here.

- Run the [item 7 sequence][nested] end to end: register first with account-owned slots, snapshot and clone,
  attach by descriptor, discard copied Git administration, install the pointer and check the common directory
  and worktree state. The nested research's 2026-11-18 correction identifies the untested combination.
- On the descriptor-mounted clone, verify `nosuid` with a setuid execution control, alongside `nodev` and
  mountinfo. The nested probe measured the device refusal, not setuid execution (`guest-exact-zfs.log:222`
  to `:223`). The built handler repeats the control.
- Re-run **every item 3 refusal** with the account owning `.claude/worktrees` and the slots: a link at `.claude`, at
  `.claude/worktrees` and at the slot; a link created by the account inside the parent; a parent escape, magic link
  and bind mount with the same device and inode; replaced inode, wrong UID and widened mode. The account can plant
  a link under this ownership, so the handler must refuse, not the filesystem permissions from the root-owned
  fixture. Include access and default ACL drift, a target already mounted, and a valid account-owned attach control.
  Assert refusals identify the component or field and do not attach a clone or disturb an existing mount.
  Include the bounded rename/replacement race and post-attach check in item 3's unmeasured cases.
- Compare a shared checkout (move-back refusal control) with a private checkout under a shared parent
  (staging and unmount control). Start a confined consumer before clone attach in both states; measure visibility
  and writability with the parent outside every `Read*Paths=`, and with the read-only bind control.
- Have a fresh process with no inherited mount descriptors reopen and verify an already-attached clone and
  move it to staging. Include wrong-mount and changed-path controls. The nested-clone guest used the original
  `fsmount` descriptor for the move (`guest-extra-commands.txt:158` and `:171` to `:175`).
- Run `zfs mount <clone>` on a disposable legacy clone. Verify the `legacy mountpoint` refusal and no attached
  mount; use an ordinary, mountable dataset as the success control.
- Exercise both checkout boot shapes with a nonclone dataset nested by path only, like the vm-builder output.
  Record `zfs mount -a` parent IDs and ordering with a delayed or failed checkout mount, plus a legacy checkout
  with its explicit nested-mount provider ordering. For the ordinary checkout option, test `chattr +i` on the
  underlay against rename and unlink by the account, reboot, home rollback and ordinary mounting. These are
  pending results, not authorization to use either boot option.
- Run on an encrypted file-backed pool shaped like this host. Read-only `zfs get` reported `aes-256-gcm`,
  encryption root `zpcachyos`, `recordsize=32K` and `compression=zstd-3` for home. Use the VM's own pool,
  key and corresponding inherited properties, never the host's key or device. Test loaded and unloaded key
  states for handler readiness, attach and clone fill, with same-dataset reflink and cross-dataset controls
  that distinguish a shared data key from separately created data keys. The nested-clone probe used an
  unencrypted pool; matching behavior on the encrypted fixture is pending in the [VM record][measurements].
- With the clone unmounted, remove the pointer in the fixture and test `git worktree lock` against `prune`,
  with an unlocked missing-pointer positive control and an intact-pointer baseline. Check whether the lock can
  be installed or recovered with the pointer already absent, and its effects on removal and move.
- Promote a disposable checkout clone in the VM. Record origin/dependency reversal, exact snapshot names and GUIDs
  before and after, and destruction refusals. Do not treat that VM operation as permission to promote live clones.

## Acceptance tests the build still owes

Run with the built wrapper and handler on a disposable VM's file-backed pool, without passing through the host's
pool or `/dev/zfs`. Repeat the [host-shaped encrypted fixture](#measurements-due-before-design-acceptance) with the
built programs, including an unloaded-key control for the handler and clone fill. Test root installation and then
the account with only the proposed delegation. Capture versions,
assertions and controls, not just logs. Make the VM recipe reproducible: retry the documented `mvm` template route
that failed with `guest agent on template-setup did not respond within 120s`, or verify and document the disposable
QEMU recipe using the same OpenZFS version. Retain evidence of guest and pool teardown, not system changes on the
host. These gates carry the [handover's H1 and H2 disposition][handover] and the nested study's
[What I could not show and why][nested-limits].

### Descriptor safety and mount authority

- H1 gate 1: reproduce the old check-then-path-mount behavior as a positive control, using only a disposable decoy
  inside the VM. A path mount reached a decoy in the research; that did not test a substitution after the check.
  Prove the harness observes attachment outside the intended slot before trusting any no-mount assertion.
- H1 gate 3: exercise repeated, bounded concurrent slot renames and replacements during real mounts. Assert no
  unrelated mount is touched; an attach on a moved recorded inode is detected by the post-attach mount-ID check.
  Verify the selected policy for rollback or retention and the stranded report. Test the post-attach check itself,
  not just the open-time checks. In `guest-extra-commands.txt:191` to `:195`, the deterministic rename probe
  moved the mount to staging by its descriptor and unmounted it. It did not assert the resulting mount-table
  state at the relocated path.
- H1 gate 5 recheck: verify mount flags in `/proc/self/mountinfo` and enforce both `nosuid` and `nodev` in the VM
  even where dataset properties would permit them. The research measured the device refusal but not a setuid
  execution control. Use the built handler, not the probe's mount command.
- H1 gate 6: run unmount from a handler invocation that did not attach the clone. Change a path between resolution
  and unmount; assert no unrelated mount unmounts. Use a real busy holder and exercise move to staging, busy refusal,
  and the selected policy for reattach or retention. Test interruption there. Re-run the measured shared-checkout refusal and private-checkout
  control with the selected policy. A held procfd's `EBUSY` is not the busy-holder test.
- H1 gate 7: exercise a confined user namespace without extra authority. Show that it cannot attach the clone with
  different flags despite OpenZFS's `FS_USERNS_MOUNT` declaration. Exercise a permitted host mount as the control,
  verify late-mount visibility in confined consumers under shared and private checkout propagation (with and
  without the read-only parent bind), and run the same-dataset reflink fallback
  in the actual confined consumer context.

### Boot, shutdown and setup

- Perform an actual reboot and shutdown with idle and busy legacy clones. Prove the selected protection at the
  unchanged checkout path and verify startup and stop ordering of `reconcile-git-worktree-clone-mounts.service`.
  Include every nested dataset, including a nonclone fixture like the vm-builder output. Delay or fail import,
  key loading and checkout mounting; assert no nested dataset attaches or creates mountpoint folders in the
  home underlay. Check mount parent IDs and visibility after success. For the legacy checkout, verify the
  nonclone mount is excluded from the automatic scan and the explicit unit cannot start before verified checkout
  success. Repeat the selected ordering with real boot units; ordinary clone mountpoints are a negative control,
  not a shipped setup.
- Enable real ZED in the guest and exercise the configured ordering, replay, dropped events with and without a live
  caller, concurrent zedlets, stale receipts, and pending unmount across restart. Measure elapsed stages and use
  readiness/refusal controls with the handler absent, ABI unsupported and ZED stopped. Repeat with a second host's
  sanoid and ZED configuration in isolation. The [nested-clone guest][nested-limits] did not start ZED.
- Verify direct checkout/home unmount names held clones, idle export/import with a clone succeeds and recovers, and
  busy export fails without a forced teardown. Record root's exact-version behavior on destroy of a mounted clone
  in the guest; the wrapper must refuse before attempting it regardless of the tool result.
- Exercise migration and rollback on the disposable layout, including linked Git pointers, an independently mounted
  vm-builder output dataset, ignored underlay files and a recoverable backup. Check that `doctor` reports partial
  reads and prints the tested root-once commands for the selected setup, without mutating it. Test parent adoption
  after recreation or rollback, and refuse it with a link, boundary, ACL mismatch or unresolved live/staged binding.
  Verify ordinary fallback on an unused slot and no fallback over a binding. Test which paths the
  actual Kopia job can see, whether it crosses a ZFS boundary, and restoration of the checkout and configuration
  with origins still pinned. An unavailable job is an unknown, not proof of exclusion.

### Binding, Git and lifecycle

- Verify exact ioctl values and Rust layout against the running module's BTF; exercise native nvlist encoding,
  resizing, permissions and property sources. Test both accepted and refused creation-time user properties plus
  post-create `ZFS_IOC_SET_PROP`. Test malformed, oversized or inherited mount intent and stale results with no
  unauthorized filesystem action. Verify the release wrapper starts without OpenZFS and has no CDDL dependency.
- Drive the full register-first add at exactly `.claude/worktrees/<name>` with descriptor mounting and account-owned
  slots. Use Git 2.56.1 at runtime; the nested guest used 2.43.0. Recheck empty vs. non-empty registration, the
  common directory and index, boundary-safe ignored cleanup, nested path omissions and mounted/unmounted `move`.
  Use ordinary Git and same-dataset reflink controls and a cross-dataset negative control. Cover recognized and
  rejected forms, dirty sources and a tracked write between preflight and snapshot that the frozen proof refuses.
- Kill add at each of item 7's five points and around the origin and clone identity writes. Test `ENOSPC` at
  snapshot and clone; report what survives, resume only proven objects and never fall through onto a pending slot.
  With the clone unmounted, verify `prune` keeps the registration with the pointer intact, and exercise the
  selected protection with the pointer missing. Check that operator locks continue to refuse removal. Test link
  repair with surviving admin data, and already-pruned admin state with and without a verified backup.
- Kill removal across unmount, snapshot destruction, clone and origin cleanup, and Git and directory cleanup.
  Verify busy and staged mounts, nested datasets, same-name replacements, held or new sanoid snapshots and another
  clone pinning the origin each block the unsafe next action. Check the before-destroy inventory and final loss
  report on successful and partial removals, with both manual and sanoid snapshots. Distinguish completed clone
  removal from origin or Git cleanup still owed. Verify safe retries under the actual request and mount state.

### Tooling, retention and performance

- Exercise sanoid 2.3.0, including prune behavior not tested by the 2.2.0 guest. Confirm recursive coverage, explicit
  exclusions, section-order and `recursive = zfs` controls, and separate retention for checkout underlays, clone
  snapshots and pinned origins. Measure a living worktree with real builds versus a no-build interval, separate
  from add and removal churn. Attribute retained bytes to real output and verified ZFS accounting; measure what
  expiration, removal and the current guard can free.
- Test real dependency preparation after add and after migration. Interleave the same-dataset control with the new
  checkout and mounted clone using the same lockfile and home store. Separate payload imports, relink, elapsed
  stages and allocation; use `project` virtual store as a copy-detecting control. Check queued and inline repair,
  Cargo prewarm and cross-filesystem lock recovery, mise and launcher repair, and watch rebinding in the editor.
  The research did not measure the shipped wrapper, complete toolchain install, cold registry fetch or editor GUI.
- Compare clone-add and fallback stages against ordinary Git under the same real workload and interleaved control.
  Separate Git fill, mount/request wait, discarding the copied `.git` and `.claude/worktrees` stubs, and downstream
  preparation. Report disposal separately against the same control arm; it has no counterpart on ordinary Git add.
  The nested probe only moved `.git` aside (`keep/probe.ts:83`), so it provides no deletion measurement. Use measured reconciliation and shutdown workloads to size any
  shipped deadline or optional scan interval. Prior isolated mount latency is not a prediction of ready-to-use
  worktree time.

## Appendix: review and research coverage

### Cross-vendor review

The review is `~/temp/agent/zfs-worktree-design-proposal-review-2026-11-18/review.md`.
All findings are accepted in the [handover][handover]. This map distinguishes design answers from tests still owed.

- H1: [Dataset and path layout](#dataset-and-path-layout), [Add and Git registration](#add-and-git-registration),
  and [Descriptor mount and unmount](#descriptor-mount-and-unmount) give the slot, fill and mount contract.
  [Descriptor safety and mount authority](#descriptor-safety-and-mount-authority) carries the outstanding gates.
- H2: [One-time move and checkout boot protection](#one-time-move-and-checkout-boot-protection) keeps the path;
  [Add and Git registration](#add-and-git-registration) omits nested mounts, and
  [Dependency repair and watchers](#dependency-repair-and-watchers) states the measured pnpm consequence.
  [Open project choices](#open-project-choices) and [Boot, shutdown and setup](#boot-shutdown-and-setup) retain
  the boot guard decision and proof.
- H3: [Unmounted, pruned or moved worktrees](#unmounted-pruned-or-moved-worktrees) preserves the underlay pointer,
  distinguishes unmounted and pruned states and gives the limits of repair.
- H4: [Requests, results and ordering](#requests-results-and-ordering) bounds property values, keeps nonces out
  of filenames and gives root-owned receipt permissions.
- M1: [Add and Git registration](#add-and-git-registration) puts the shared `doctor` preflight before mutation.
- M2: [Binding crate and compatibility](#binding-crate-and-compatibility),
  [Dataset and path layout](#dataset-and-path-layout), [Add and Git registration](#add-and-git-registration) and
  [ZED reconciliation and mount safety](#zed-reconciliation-and-mount-safety) name the crate, datasets, origin,
  handler, properties, config and state by purpose, using `MKY` and `VGW`.
- M3: [Binding crate and compatibility](#binding-crate-and-compatibility) defines the separate Rust binary,
  shared ABI gate and requests; [ZED reconciliation and mount safety](#zed-reconciliation-and-mount-safety)
  defines its installed entry points.
- M4: [Add and Git registration](#add-and-git-registration) distinguishes optional local user properties at
  clone creation from `mountpoint` and gives the `ZFS_IOC_SET_PROP` path.
- M5: [Requests, results and ordering](#requests-results-and-ordering) does not assume a timer;
  [Open project choices](#open-project-choices) keeps the periodic sweep conditional.
- L1: [Snapshots and setup reporting](#snapshots-and-setup-reporting) gives the eight exclusions with mount paths
  or unmounted status. [What doctor checks and prints](#what-doctor-checks-and-prints) checks the configured order.
- L2: [Requests and input ownership](#requests-and-input-ownership) attributes the extra opcode checks to the
  review; [Binding crate and compatibility](#binding-crate-and-compatibility) cites rule codes;
  [Scoped delegation](#scoped-delegation) and [Snapshots and setup reporting](#snapshots-and-setup-reporting)
  correct the delegation and guard citations.
- L3: [Add and Git registration](#add-and-git-registration) writes the origin claim first and handles `ENOSPC`
  and killed adds. [Removal and recovery](#removal-and-recovery) uses the kernel's dependent-clone refusal.
- L4: [Add and Git registration](#add-and-git-registration) requires proof from the frozen content;
  [Binding, Git and lifecycle](#binding-git-and-lifecycle) tests a write in the window.
- L5: [Unmounted, pruned or moved worktrees](#unmounted-pruned-or-moved-worktrees) refuses managed moves;
  [Add and Git registration](#add-and-git-registration) keys ownership on registration identity and clone GUID.
- L6: [Open project choices](#open-project-choices) uses plain list items, not bold-label bullets.

### Nested-clone research requirements

Items refer to the [nested-clone research][nested]. Tests that close remaining evidence gaps are grouped in
[Acceptance tests the build still owes](#acceptance-tests-the-build-still-owes).

- Item 1, nested snapshots and unmount/export: [Dataset and path layout](#dataset-and-path-layout) separates
  underlays from clone snapshots; [Boot and shutdown unit](#boot-and-shutdown-unit) owns shutdown
  unmounts and busy reports. Idle export/import and real shutdown still need VM coverage.
- Item 2, boot order: [Dataset and path layout](#dataset-and-path-layout) keeps clones out of automatic mounts;
  [Boot and shutdown unit](#boot-and-shutdown-unit) requires the verified checkout mount first.
  [Boot, shutdown and setup](#boot-shutdown-and-setup) tests delayed and failed mount order.
- Item 3, planted links and descriptor operations: [Descriptor mount and unmount](#descriptor-mount-and-unmount)
  adopts the named requirements and records the propagation limit and rollback failure state.
  [Measurements due before design acceptance](#measurements-due-before-design-acceptance) requires every refusal
  under account ownership; [Descriptor safety and mount authority](#descriptor-safety-and-mount-authority) tests
  the built handler.
- Item 4, sanoid coverage and retention: [Snapshots and setup reporting](#snapshots-and-setup-reporting) defines
  coverage and order; [What doctor checks and prints](#what-doctor-checks-and-prints) reports origins and retention;
  [Removal and recovery](#removal-and-recovery) removes explicit snapshots and checks origin dependencies.
- Item 5, Git, walks and backup: [Walks and Git clean](#walks-and-git-clean) stops at boundaries and records the
  corrected clean fixture; [What doctor checks and prints](#what-doctor-checks-and-prints) reports backup coverage.
  Kopia boundary behavior is a test, not a result.
- Item 6, removal: [Removal and recovery](#removal-and-recovery) defines order, blockers, account-owned underlay
  cleanup and resume; [Binding, Git and lifecycle](#binding-git-and-lifecycle) tests interruption.
- Item 7, registration and fill: [Add and Git registration](#add-and-git-registration) defines the order,
  copied `.git` rule and durable record; [Unmounted, pruned or moved worktrees](#unmounted-pruned-or-moved-worktrees)
  refuses moves. [Binding, Git and lifecycle](#binding-git-and-lifecycle) tests the combined sequence and kill points.

[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[answers]: cli-git-zfs-worktree.md#owners-answers-of-2026-11-18
[zfs-plan]: cli-git-zfs-worktree.md
[handover]: ../handover/cli-git-rust-implementation.md
[nested]: cli-git-zfs-clones-mounted-under-the-checkout.md
[nested-limits]: cli-git-zfs-clones-mounted-under-the-checkout.md#what-i-could-not-show-and-why
[pnpm]: cli-git-zfs-checkout-dataset-and-pnpm.md
[vet]: ../audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md
[btrfs-plan]: cli-git-btrfs-worktree.md
[spec]: ../../package/git-policy/cli/SPEC.md
[rust-catalog]: ../decision/rust-dependency-catalog.md
