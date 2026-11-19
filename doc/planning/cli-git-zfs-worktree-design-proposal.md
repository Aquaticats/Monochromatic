# ZFS worktree design proposal for the Git wrapper

**Non-final and unapproved.** Written by an agent for the project owner's acceptance. No implementation or pool
changes were made. This proposal draws on the [ZFS planning record][zfs-plan], the [Rust binding vet][vet],
the [Btrfs design][btrfs-plan], the [wrapper specification][spec], the native source, and the recorded OpenZFS
and sanoid probes. The design choices and VM checks under [Acceptance gates](#acceptance-gates) remain open.

## Requirements and module ownership

The owner chose "ZED, Clones, reflink. There is a reason we have a doctor subcommand." and
"ZED handler (Recommended)" ([ZFS planning record][zfs-plan], lines 332 to 343). The wrapper owns Git validation,
staging, the unprivileged dataset lifecycle, and the reflink fallback. A root-installed ZFS Event Daemon (ZED)
handler owns mount and unmount, not Git or automatic deletion. The dataset binding owns the kernel request format;
it does not decide which worktrees or snapshots belong to the wrapper.

"Report and print commands (Recommended)" ([ZFS planning record][zfs-plan], lines 338 to 341) makes `doctor`
read-only. The commands it prints are for an administrator, not commands the wrapper executes.

## Binding crate and compatibility

The owner chose "A Rust crate", "Kernel interface (Recommended)", "Out (Recommended)" for crates that run `zfs`,
and "Our own small binding (Recommended)" ([ZFS planning record][zfs-plan], lines 343 to 361).
Use `package/zfs/kernel-interface`, the location in the vet answer's preview. `zfs` is the category,
`kernel-interface` is singular and names what the crate does, and its README will explain `/dev/zfs` and ioctl
(kernel request) to a Git user. The package's path-derived name is `@monochromatic-dev/zfs-kernel-interface`
where workspace naming applies; the Cargo package is `monochromatic-zfs-kernel-interface`, following the wrapper's
Rust naming precedent (`package/git-policy/cli/Cargo.toml:1`). This follows `AGENTS.md:1079` and `AGENTS.md:1457`.
Give it the standalone Cargo workspace, path dependency, `README.md`, and package `mise.toml` used by Rust siblings;
use the [Rust catalog decision][rust-catalog] for external dependencies and the `uses/` layout for source folders.
Reuse the existing `libc` and `sha2` dependencies for system calls and ABI fingerprints. Do not add a C build step,
link or load CDDL libraries, or run a ZFS CLI from the binding, wrapper or zedlet. The vet's section "Where the wrapper
ships and what it links today" explains why the binary must still start without OpenZFS installed.

### Requests and input ownership

The initial interface is Linux only. Expose typed operations, not a general-purpose ioctl or property setter.
Numbers are for OpenZFS `zfs-2.4.4`, `include/sys/fs/zfs.h:1502` to `:1550`.
The vet's "Result and the owner's decision" records the core set;
`SNAPSHOT_LIST_NEXT` and `GET_FSACL` were checked separately in the same header.

- `ZFS_IOC_OBJSET_STATS` (`0x5a12`): inspect dataset type, identity, origin, and the values and sources of properties.
- `ZFS_IOC_DATASET_LIST_NEXT` (`0x5a14`): enumerate immediate children via the kernel's cursor.
- `ZFS_IOC_SNAPSHOT_LIST_NEXT` (`0x5a15`): enumerate snapshots of one dataset for removal and recovery.
- `ZFS_IOC_SET_PROP` (`0x5a16`): change only the wrapper's user properties on a named clone.
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

Before the first ioctl, require a validated profile for the loaded module and target ABI: exact module version,
architecture, endianness and kernel/module BTF digests from a passing VM image. A missing device, BTF or profile,
or a mismatch, makes raw ZFS requests unavailable in both binaries, including reads. There is no force flag;
a new profile needs a passing layout check and live tests. Leave `linux-arm64` disabled for raw ZFS until separately
verified; the ordinary checkout route still runs. Exact profiles are conservative and can drop the clone route after
an update. A runtime field-by-field BTF check with a matching release/request policy would accept more rebuilds,
but adds a parser and trusts more unexercised builds. Ranking: **exact profiles > runtime layout discovery** for
first release, because an unchecked build gets a fallback, not a guessed kernel contract.

Test the codec, request packing, bounded decoding, and the fail-closed gate without a device. Fuzz the adapted codec.
Run every live ioctl, migration, handler and fault test against a file-backed pool inside a disposable VM, never
`zpcachyos` and never with the host's `/dev/zfs` passed through. Test as root for installation and as an account with
only the proposed grants for normal requests; a root-only pass proves nothing about delegation. Check codec native
encoding, cursor behavior, property sources, output resize and per-name errors. The full field match against the
new crate, clone properties at creation, effective delegation and release-archive linkage remain unverified.

## Dataset and path layout

The main checkout is a directory in `zpcachyos/ROOT/cos/home` today. A clone needs a dataset, and its destination
cannot be nested inside the checkout's user-owned directory. The [ZFS planning record][zfs-plan], lines 135 to 155
and 270 to 281, records why: a privileged mount followed a planted link in a user-owned parent. ZFS dataset ancestry
and Linux path ancestry need not match. For this machine, propose sibling datasets
`zpcachyos/ROOT/cos/home/monochromatic-main` and `zpcachyos/ROOT/cos/home/monochromatic-worktree`.
Each clone is one immediate child of `monochromatic-worktree`; both siblings are covered by home recursion.

Map them to a root-controlled anchor, for example `/var/lib/monochromatic-worktrees/monochromatic/`.
Use a separate configured anchor and dataset pair per repository and account. Every ancestor and the unmounted slot
folders are root-owned, not writable by the account (including via ACLs), and not symlinks. The main dataset mounts
at `main` under the anchor. The clone parent dataset is an unmounted container with `canmount=off`,
`mountpoint=legacy`, `setuid=off` and `devices=off`. Root owns its corresponding path `linked` under the anchor;
root makes or verifies empty `linked/<slot>` mountpoints. A mounted dataset gives the user their writable tree;
the user still cannot replace the mountpoint entry or its parent. The main and each clone have a `checkout` child,
so a clone can stage a tree and still give Git an empty directory on the same filesystem.

Use inherited `mountpoint=legacy` on clones, with no `fstab` or other auto-mount entry for them. This avoids a race
with boot-time `zfs mount -a`: the user's `clone -o canmount=noauto` was refused in the probe, so do not rely on it.
OpenZFS `lib/libzfs/libzfs_dataset.c:2679` handles inherited `legacy`, `lib/libzfs/libzfs_mount.c:296` skips it,
and `etc/systemd/system-generators/zfs-mount-generator.c:430` skips it too, at `zfs-2.4.4`. The handler uses a native
Linux mount at the derived path with `nosuid,nodev` enforced, not the unprivileged `fstab` route that failed.
The exact syscall and options, inheritance at clone creation, and real boot order (including encrypted pool import
and key availability) are VM gates, not results from the probe.

### One-time migration and permissions

With writers stopped, take or verify a recoverable backup, then create and mount the main dataset at its protected
path. Copy the whole checkout into `main/checkout` without crossing unrelated mounts; preserve IDs, modes, times,
links, ACLs and xattrs. Inventory nested mounts and linked worktrees first; preserve nested datasets independently
and do not copy over or lose their mounts. This is a cross-dataset migration, not an atomic rename or a reflink.
Verify the object database, status including untracked and ignored data, and each linked pointer and back-link.
Switch consumers to the canonical path and use Git's worktree repair for registrations affected by the move.
Keep the old tree and a reversal plan until verification succeeds; do not put a privileged mount beneath a writable
home path to keep the old spelling. A compatibility link may point from the old path to the new checkout, but it
is not an accepted managed mount target. The migration sequence and its repair/reversal need a disposable test.

Pin dataset names, GUIDs, anchor path and owner in root-owned configuration. Use a user grant local to the main
dataset for `mount,snapshot`, a descendants-only grant there for `clone,mount,destroy` (snapshots count as
descendants), a local `create,mount` grant on the clone parent, and `mount,destroy,userprop` on its descendants.
The `mount` grant is an ioctl prerequisite, not permission to change the Linux mount namespace.
Do not grant `mountpoint`, `canmount`, `setuid`, `devices`, `rename`, `promote` or permissions on the whole home tree.
This proposal lets that account destroy **any main dataset snapshot**, not just wrapper origins; ZFS delegation
cannot select an origin name prefix. It does not grant destroy on the main dataset itself. Use a checkout-only tree
with no other descendants and do not treat its snapshots as protection from that account. Sources at `zfs-2.4.4`:
`man/man8/zfs-allow.8:105`, `module/zfs/dsl_deleg.c:574` and `module/zfs/zfs_ioctl.c:826` and `:1128`.

## Add, mount and removal

### Create and bind the worktree

1.  For the clone path, accept only a canonical `linked/<slot>/checkout` destination under the configured anchor.
    Reserve the slot with a per-add claim; restrict slot names to a single lowercase ASCII name using letters,
    digits and hyphens. Validate the existing trusted `linked` directory; root creates the missing slot, so this
    special form cannot use the ladder's existing-parent requirement unchanged. All other paths keep their meaning.
2.  Require the ladder's chosen source to be the configured main checkout and prove it clean. Do not silently use
    the main checkout in place of a linked source. Freeze its index/tree proof and make a fresh, uniquely named
    `main-dataset@cli-git-origin-<nonce>` snapshot for this add. Clone it to the slot with a local mount request
    at creation. Bind the journal to the actual origin snapshot and clone GUIDs, not merely the requested names.
    Never reuse a sanoid snapshot as the origin or promote a clone. The [ZFS planning record][zfs-plan],
    lines 32 to 43, records the measured dependency: the origin cannot be destroyed while a clone uses it.
3.  Wait for ZED and verify the exact ZFS mount is visible and writable in the caller's namespace. At this point
    the filesystem is a clone, not yet a Git worktree. Rename the inherited `checkout` to a private staging name
    beside it inside the clone. Register `checkout` using Git's `worktree add --no-checkout` validation; bind the
    pointer and its reciprocal Git registration as in the current ladder. No Git invocation uses the copied `.git`
    directory: verify the frozen source in the stage, discard its copied Git metadata and apply the main-checkout
    leave-out rule. Verify conversion settings in the new worktree's context, install Git's new pointer and the
    proven index, then apply the existing tree difference. Exchange the two sibling directories only after
    rechecking the mount, claims and binding. Both are inside the clone, not one on either side of a mountpoint.
    `RENAME_EXCHANGE` is handled by `zfs_rename` in `module/os/linux/zfs/zfs_vnops_os.c:2818` at `zfs-2.4.4`;
    prove the actual add, exchange and crash recovery in the VM. Also prove that the source checks cover the
    instant snapshotted (index and tracked bytes); do not treat differing ZFS stat identities as clean.
4.  Record the completed binding in the Git common directory: canonical path, repository and registration
    identities, dataset and origin names and GUIDs, and request nonce. A lookup requires that binding plus the
    expected clone lineage and mount facts; a prefix or a user property is not proof of ownership.
    If clone creation failed and absence is certain, try to delete only the origin this add proved it created.
    Keep an interrupted add and its registration when ownership or cleanup is uncertain. Report the stage, clone,
    origin and actual registration, and whether to resume, remove, or rerun at an ordinary writable destination.
    Do not create a second checkout over an unverified mount, or destroy another process's stage. After Git binds
    an intact empty checkout, the existing bound Git checkout can finish there if the clone fill alone fails.

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
[ZFS planning record][zfs-plan], lines 14 to 30, records `EXDEV` across datasets despite block cloning being enabled. Do not hide
an ordinary byte copy inside rung 2. A managed path cannot be filled without its mount, because the anchor is
root-owned; print the requirement and the option to retry at an ordinary path. After migration, that ordinary path
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
snapshot history. Origin blocks may be pinned until the last dependent clone is gone. Pool pressure will not be
fixed by a guard that can no longer reach much of the churn. Do not expand the guard's deletion authority implicitly.

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

[zfs-plan]: cli-git-zfs-worktree.md
[vet]: ../audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md
[btrfs-plan]: cli-git-btrfs-worktree.md
[spec]: ../../package/git-policy/cli/SPEC.md
[rust-catalog]: ../decision/rust-dependency-catalog.md
