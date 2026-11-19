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
    beyond the claim. Create or verify an empty account-owned slot, then run Git's supported `worktree add
    --no-checkout` form. Record the reciprocal registration and exact `.git` pointer bytes; leave the file in
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

### ZED desired state and wait

The wrapper leaves a local `org.monochromatic:worktree-request` property containing a protocol version,
request nonce, and desired state `mount` or `unmount`. It also records its own durable intent in the add or removal
journal. Never use an inherited property as a request. A history event is only a wakeup: sweep current immediate
children of the configured parent, not a path or command named in the event. Only consider filesystem clones with
the allowed origin, slot grammar and safety properties. Missing or malformed intent leaves the mount alone.
Reject a wrong dataset or mount at the target; check the
entire protected directory chain and make an empty placeholder without following links. The privileged zedlet and
its executable are root-owned, with no user-writable code, configuration or search path. It does not run Git,
load code from the checkout, accept arbitrary mount flags/targets, or widen access to `/dev/zfs` for sandboxes.

Events, a startup pass after pool import, and a periodic pass call the same reconciler in the host mount namespace.
Serialize the check and action for one slot, not the repository; reread its desired state, dataset identity and
actual mount state under that claim. Mount only when absent; unmount only the matching mount, without force or lazy
detach. Do not unmount something else at the same path. An `unmount` request persists until that cloned dataset is
destroyed or an explicit recovery reverses it, so reboot does not remount a worktree being removed. The handler can
clear an empty root-owned placeholder after verifying the dataset is gone and that no mount now occupies it.

Write an atomic root-owned result under `/run/monochromatic-zfs-worktree`, keyed by dataset GUID and request nonce,
with operation, outcome and errno when applicable. `/run` is only a result cache and liveness signal; the next
reconcile rebuilds it from durable requests. The wrapper waits using a monotonic deadline and bounded sleeps or
result-file notifications, then verifies mount identity and visibility itself. Halfway to the deadline, if the
result is still pending, change a separate local `org.monochromatic:worktree-wake` nonce once to trigger a new event.
Do not write properties from the handler merely to acknowledge a request; that would generate more events.
Propose a **5 s experimental wait ceiling** per mount or unmount request, borrowing the wrapper's non-commit
`REPORT_SCAN_CEILING` (`package/git-policy/cli/src/native/service/wait_ceilings.rs:1265` to `:1280`) as an initial
budget, not as a ZED latency measurement. Validate the deadline and periodic interval from guest latency under
ordinary concurrent work and after boot before setting shipped values. The wait includes readiness checks; ending
it does not cancel a kernel request or revoke a durable intent. Never retry with conflicting state on that assumption.

If the handler refuses, the caller sees the exact failed operation, dataset, expected path and reason. On `EBUSY`,
name the mount, explain that open handles or a working directory can keep it busy, and advise leaving it and retrying
from another checkout. Explain that the request remains live and a later sweep can unmount it unless it is reversed.
On timeout, report the last observed state, that a late action is possible, the handler's log and `doctor` commands,
and the matching request to resume. Treat a mount hidden from the caller as a separate failure,
not a timeout success. The [ZFS planning record][zfs-plan], lines 157 to 185 and 195 to 226, documents replay after
ZED stops, a dropped event recovered by a later property write, a busy unmount reported and retried, parallel zedlets,
and the confined caller that could not see a later mount. A reboot starts with no usable old event queue; durable
intent, the boot-safe mount property, and startup reconciliation are necessary even if replay works after a stop.

### Remove without crossing a boundary

Reuse the linked registration and back-link checks, the lock and clean checks (one supported `--force` can waive
cleanliness, not a lock or an identity check), and the held-directory boundary walk. Check the clone, its checkout,
and its registration for other worktrees, nested datasets and mounts, including bind mounts. Check actual dataset
children and snapshots as well. An unsupported remove of a known managed clone must not fall through to Git's
recursive removal. Report the blocker and do not touch its files. The guards come from the
[wrapper specification][spec], lines 2932 to 2943 and 3064 to 3190, and
`src/native/subvolume/subvolume_remove_walk.rs:465` under the CLI package.

Record removal intent, warn that successful removal deletes snapshot history of that worktree, release handles that
would hold it busy, and request `unmount`. After a matching root result, verify the mount is absent and the recorded
clone GUID, request and registration still match. Re-enumerate that **clone's** snapshots, including sanoid's and
manual ones. Destroy only the explicit names belonging to it, then the unmounted dataset, then the exact recorded
origin once no clone depends on it. This deletes the clone and its own snapshots without handing the kernel a
recursive dependency destroy. Refuse on nested datasets, holds or dependent clones; never remove another
clone to free an origin. Sanoid can take a new snapshot between enumeration and destroy; a resulting refusal is
reported or retried from a fresh enumeration within the command budget, not turned into a recursive destroy.

Only settle Git's registration after confirming dataset destruction. Record partial success, including any deleted
snapshots, a surviving origin, a retained unmounted clone, or failed Git cleanup. A later command can resume only the
matching recorded work and must recheck what it affects. `doctor` reports interrupted work and never reaps by name.
Destruction is irreversible; the ZFS destroy is name-based and the final identity check is not atomic with it.
Do not claim this closes same-user check/use races. A matched receipt alone is no authority to delete. VM tests
must cover replacement, stale acknowledgements, unmount refused or delayed, and an origin that is still in use.

## Ladder and fallback

Make ZFS clone the ZFS implementation of **rung 1**, alongside Btrfs snapshot, not a rung ahead of Btrfs. Keep
ordinary `git worktree add <path>` semantics. The existing ladder covers its recognized add forms only
([wrapper specification][spec], lines 1121 to 1189). Reuse the clean-source and frozen-index checks, attribute and
conversion checks, checked Git registration, tree difference and guarded publication. For ignored state, reuse
Btrfs's present main-checkout snapshot leave-out policy, not an unconditional whole-directory copy: keep the
eligible ignored data as of the snapshot, leave out nested worktrees and wrapper caches/stages as the existing rule
specifies, and skip the separate ignored copy only after a successful snapshot fill. The current Btrfs spec also
has a dirty-main restore path; the ZFS first release declines dirty sources instead of claiming that parity.
See [wrapper specification][spec], lines 1178 to 1271 and 1379 to 1525, and [Btrfs design][btrfs-plan],
"Implemented flow and what superseded the proposal". Reuse `fs_beneath.rs` for held, no-follow paths,
`fs_mount_options.rs` for caller mount facts, and `fs_clone_exchange.rs:46` and `:70` for reflink and exchange.
`fs_kind.rs:63` and `:87` do not classify ZFS today; `src/native/worktree/worktree_snapshot_source.rs:279`
currently gates reflink to Btrfs, XFS and overlayfs. Add ZFS recognition and dataset-specific checks rather than
reusing Btrfs inode or subvolume identity rules. Source and removal checking code remains in the wrapper.

On ordinary writable paths, **rung 2** gains ZFS `FICLONE` if a clean source and destination are on the same dataset
and the kernel accepts the reflink. Use held descriptors and existing clone, index, and ignored-copy rules.
Choose it if managed setup or a safe mount is unavailable before mutation, the caller cannot clone in its namespace,
or the requested path is outside the managed anchor. For a refused `FICLONE`, cross-dataset path, or unproved
source, continue to **rung 3**, Git's checkout plus the existing ignored-state behavior. The
[ZFS planning record][zfs-plan], lines 14 to 30, records `EXDEV` across datasets despite block cloning being enabled.
Do not hide an ordinary byte copy inside rung 2. A managed path needs its mount because the anchor is root-owned.
Print the requirement and the option to retry at an ordinary path. After migration, that ordinary path
must be under the source's dataset to reflink. A same-pool path in home is no longer enough.

The fallback still walks and clones files and runs the existing Git checks and dependency preparation; it does not
get a clone's whole tree in one operation. The recorded full-worktree reflink copied 56,478 entries in 5.63 s
([ZFS planning record][zfs-plan], lines 14 to 25); that single sample is context, not a prediction for this rung or
an install comparison. The confinement probe had a successful same-dataset reflink and a cross-dataset control is
needed in the VM. The Pi driver creates worktrees before entering the unit
(`package/pi-shared/subagent-driver/src/start-pi-run-in-a-worktree.ts:238`), but its callers must explicitly choose a
managed path to get the clone. A long-lived unit must not bind the worktrees anchor read-only if it needs to see later
mounts. Do not turn that observation into a policy that relaxes the caller's confinement.

## Snapshots and setup reporting

The owner said, "Make home recursive and blocklist dirs and datasets that shouldn't be snapshoted.
Worktrees are not necessarily throwaway." and chose "Explicit list of all seven (Recommended)"
([ZFS planning record][zfs-plan], lines 362 to 385 and 426 to 436). Keep the home hourly template, include the main
and clone datasets, and keep `${HOME}/.local/share/pnpm/store` in home's dataset. The owner's reason was
"you missed how pnpm store shouldn't become its own datasets due to not being able to take advantage of clones."
That preserves reflinks within home, not between home and the migrated checkout or clone datasets.

```ini
# /etc/sanoid/sanoid.conf (affected sections only)
# Use yes, not zfs. Put exclusions after every recursive ancestor's section.
[zpcachyos/ROOT/cos/home]
  use_template = home
  recursive = yes

[zpcachyos/ROOT/cos/varcache]
  autosnap = no

[zpcachyos/nosnap]
  autosnap = no

[zpcachyos/nosnap/cargo-build]
  autosnap = no

[zpcachyos/nosnap/cargo-registry]
  autosnap = no

[zpcachyos/nosnap/npm]
  autosnap = no

[zpcachyos/nosnap/user-cache]
  autosnap = no

[zpcachyos/nosnap/vm-builder-output]
  autosnap = no

[zpcachyos/nosnap/libvirt-images]
  autosnap = no
```

Also require `com.sun:auto-snapshot=false` on every listed dataset, including `/var/cache` (`varcache`), which
lacks it in the recorded probe. It is a separate policy marker, not the sanoid exclusion mechanism.
Sanoid `2.3.0` copies recursive parent settings onto descendants in file order. The probe used
`/usr/bin/sanoid:665`, `:936` and `:1110`: the later child section beat `recursive = yes`, removing it was a positive
control, reversing the order made the child snapshot again, and `recursive = zfs` included it regardless.
The listed exclusions are currently outside home's subtree; place them after any recursive ancestors regardless.
If a folder later needs exclusion, it must first be separated as a dataset. Evidence: [ZFS planning record][zfs-plan],
lines 369 to 385 and 400 to 437, and `snapshot-blocklist-candidates-measured-2026-11-18/keep/`.

`zfs-space-guard` currently scopes pruning to its configured dataset, `zpcachyos/ROOT/cos/home` by default, and the
`autosnap_` prefix (`/usr/local/sbin/zfs-space-guard:6` and `:44`). It will not prune the main or clone hourlies or any
`cli-git-origin-` snapshots. Sanoid keeps and prunes its own hourlies; normal worktree removal deletes that clone's
snapshot history. An hourly snapshot is not a publication marker: it can capture a clone during add or removal.
Account for those intermediate snapshots in the retention and recovery checks. Origin blocks may be pinned until
no clone depends on them. A home-only guard cannot reclaim blocks held only by descendant snapshots.
Do not expand the guard's deletion authority implicitly.

### What doctor checks and prints

Extend the existing read-only filesystem report (`package/git-policy/cli/src/native/health/doctor_filesystem.rs:1`).
Separate observed facts from what the caller cannot inspect; do not show green for a denied or unsupported read.
Report individual failures and the resulting add route, without changing `doctor`'s exit status for setup findings.

- Check the loaded ABI, `/dev/zfs` accessibility, source type, GUID and canonical mount, configured parent and
  pool, protected anchor ancestors and slot ownership, clone safety properties, and any namespace visibility issue.
  Print the tested migration instructions, root `install -d` and dataset creation commands with resolved names,
  and the exact expected properties. On ABI mismatch print the module/target and validated profiles, not a bypass.
- Inspect effective grants for the intended UID. If they are missing, print the applicable `zfs allow -l -u`
  and `zfs allow -d -u` commands for the scoped grants in this design. Do not create a probe dataset as part of doctor.
- Check root ownership and permissions of the handler, its config, and startup/timer wiring, plus ZED running status
  and last reconcile. Print the root install/enable commands and journal query for the actual unit. Surface the other
  enabled zedlets before enabling ZED (the probe identifies syslog, notify, enclosure and scrub behavior).
- Check `recursive = yes`, ordered `autosnap = no` sections, and `com.sun:auto-snapshot=false` on every blocked
  dataset, the home store's containing dataset, and coverage of the main and clones. Print the concrete INI repair
  and `zfs set com.sun:auto-snapshot=false <verified-dataset>` only for drift. Report the old guard's actual scope
  and any retained origins or interrupted requests, with guarded recovery instructions, not a blanket destroy.

The final command text, units, partial-read diagnostics and setup detection are VM checks. This proposal does not
install package files or assert that any of the root-once commands have been run.

## Acceptance gates

### Choices for the owner

- **Managed path interface.** An explicit canonical path such as `linked/run-a/checkout` keeps Git's path contract
  and needs no new allocator, but the caller must opt in and supply a different writable path to use fallback
  if setup is missing. A separate opt-in named allocator could accept `run-a` plus an explicit fallback path and
  return the actual path after choosing one; it would help the Pi driver, but adds a new user-facing command and
  caller integration. Ranking: **explicit path > named allocator** for the first release, because it changes no
  argument's meaning. The allocator can follow without changing the existing command.
- **Eligible origin sources.** Main-only keeps each worktree independently removable; an add invoked in a linked
  clone cannot inherit that clone's ignored state via rung 1. Linked origins preserve more of that state, but a
  worktree with a dependent clone cannot be fully removed and snapshotted rollback gets more tangled.
  Ranking: **main-only > linked origins** until there is an accepted retention and removal design for those chains.
- **ABI coverage.** Exact verified profiles versus runtime layout discovery, ranked under "ABI gate and tests".
- **Lost events without a waiter.** Startup, events and caller nudges avoid background scans, but a lost last event
  after an interrupted caller waits for another wakeup. Adding a periodic scoped reconcile has background cost but
  repairs that state without a caller. Ranking: **periodic plus startup and nudges > startup and nudges alone**
  for recovery of living worktrees. Measure the scan cost before choosing the interval.
- **Origin cleanup privilege.** Descendants-only delegation makes the wrapper finish its origin cleanup without
  giving root a delete path, but gives the account the snapshot rights named in "One-time migration and permissions".
  A separate root-owned reaper that deletes only proven wrapper origins preserves other snapshots against that
  account, but adds privileged deletion, durable ownership and recovery work. Ranking: **scoped delegation > root
  reaper** for a checkout owned by the same account, provided the owner accepts that snapshot trust boundary.
- **Space-pressure pruning.** Keep the guard unchanged and surface its limits, or extend it to an explicit allowlist
  of managed descendants and their sanoid hourlies while retaining the newest per dataset. The second reaches more
  retained blocks but makes early deletion of living worktree history normal under pressure, without measured
  benefit. Ranking: **keep and report > expand pruning** until the retention and space measurements justify a change.

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
