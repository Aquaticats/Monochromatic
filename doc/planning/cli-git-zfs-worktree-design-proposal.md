# ZFS worktree design proposal for the Git wrapper

**Non-final draft by an agent, unapproved.** Decision fold-in: this revision states the project's answers of
2026-11-19 to every open choice and every default this file listed, quoting the words recorded for each answer.
Those words are in `doc/handover/questions-for-the-owner.md`, section "Answered in the question tool, started
2026-11-19T11:30:00-05:00", which is their record. This file's own revision stamp is what
`git log --format=%cI -1` prints for the file; this revision began from `b5294a7325`
(2026-11-19T02:21:09-05:00), and a section heading's stamp is the commit time of the commit that landed that
section.

This revision changes these statements. No framing here leaves a choice undecided, because the project answered
them all: each item under [Project decisions](#project-decisions) and [Decided defaults](#decided-defaults) states
its answer as the project's decision with the record's words, keeps the measured facts, citations and measurement
item numbers it already carried, and names what the project did not select where the item listed an alternative.
An answer that went against the ranking its question carried is stated the same way as any other. Where an answer
created a part nobody has designed, [Open items the decisions leave](#open-items-the-decisions-leave) names it in
plain words beside the answer's words. Two VM measurements are added to the acceptance tests the build owes. The
sections that stated these choices still open are brought in line with the answers, including the framing that no
arm was operative, the `--force` rule, staging, the bookmark rule and the identity pair. The section headings
"Open project choices" and "Adopted defaults open to veto" become "Project decisions" and "Decided defaults".

The fifth delta fold-in (tip `b5294a7325`) answered the bounded check of `b26aa27f3c` to `1975c5c4cc` and changed
these statements. The refusal of removal while the namespace exclusion cannot be shown was worded as one option of
its open choice everywhere, with item 15's measured facts beside the option that destroys anyway and the scan and
busy unmount that option keeps. The bookmark rule was an open choice of three options, with the first option's
remaining needs stated, and removal applied whichever one the owner picked. The receipts, the second binary and the
no-implicit-remount defaults stated their real costs. The revision entries were corrected, with all five item 18
commits named, item 18 reported what each topology kept under the clone, the record quoted the checkout's `clone`
grant and pointed item 17 to item 18, and the bookmark citations named the lines that hold their text.

Fourth delta fold-in, tip `1975c5c4cc` (2026-11-19T01:35:05-05:00), answered the bounded check of `9a1a47e190` to
`b26aa27f3c`, its residues and its bookmark verdict, and folded in [item 18][vm-18]. It made the rule that refused
removal wherever the namespace exclusion cannot be shown one option of an open choice, because no described setup
can show that exclusion. It made a bookmark whose snapshot the removal itself destroys pass through the plan
comparison instead of stranding the next chunk or the replan, and gave a bookmark whose snapshot was already gone a
refusal with its cost and the way out; the fifth delta fold-in turned that rule into an open choice, and this
revision states the project's answer. It also added two
acceptance controls, stated which clone topology items 17 and 18 measured, put the coalescing and rate gap in the
lost last event choice, stated the excluded pair as reasoning from the grants, and gave each choice its facts and
each veto-open default its evidence, its cost and how to veto it.

The third delta fold-in answered the bounded check of `7fe84c7be8` to `68690e60be`, its findings N1 to N11 and one
tension with the decided removal. That revision kept every previous answer and changed these statements. Where
removal refuses for want of the namespace exclusion, it said that no option or deployment here can show that
exclusion today, so no described setup reaches "B: Destroy at once". The bookmark check passed on a live clone
snapshot's tuple instead of a recorded bookmark. Item 17's delegated snapshot batch and the root option's two
unnamed mechanisms stand in the identity choice. The receipts, the second binary, the refusal of an implicit
remount and the private staging path are listed as veto-open defaults. The sweep's coalescing is named
unspecified, the fresh scans are counted once with the chunk rule, a partial snapshot set belongs to whichever
program failed, the coupled pair has one presentation rule, and each file now dates its revision from its own
commit.

[Item 17][vm-17]'s fold-in (`64cdbc5263`, 2026-11-19T00:44:01-05:00; `2e7be7d71d`, 2026-11-19T00:44:56-05:00)
recorded the delegated account's snapshot batch in the [measurement record][measurements] and stated its result
here. [Item 18][vm-18]'s fold-in (`9b2cd94680`, 2026-11-19T01:25:34-05:00; `a6b0ec5913`, 2026-11-19T01:25:41-05:00;
`6f317a849f`, 2026-11-19T01:26:08-05:00; `9331b1101a`, 2026-11-19T01:26:48-05:00; `02592290ee`,
2026-11-19T01:27:16-05:00) recorded the same requests on an unpromoted clone and stated them beside item 17's.
Both ran in guests booting `6.8.0-147-generic` with OpenZFS `2.4.4-1`.

Second delta fold-in, tip `68690e60be` (2026-11-19T00:30:21-05:00), answering the second delta review of
`7fe84c7be8`, including decisions that review showed the body made without the project: that revision omitted the
`defer` key from the snapshot batch request, because the kernel reads deferral from the key's presence; limited the
whole-request precheck to check failures and named the state after one batch "planned snapshots all gone with the
head refused"; had item 14 run as root; began naming the observables that prove absence; gave the bookmark inventory
a pass and refuse rule; and said how the fresh scans are requested. Item 17 landed after that tip. That revision
made kernel-release readiness and blocked-removal cancellation owner choices, coupled the identity choice to
origin cleanup authority, and marked the receipts, the second binary and the refusal of an implicit remount as
adopted defaults open to veto.

The delta gate revision (`aeb7d6c0a8`, 2026-11-18T23:30:12-05:00) answered findings on `652b51d74d` and folded in
[VM items 12 to 16][vm-delta], including the dropped private-ancestor requirement:
[VM item 3][vm-propagation] made only the checkout private and enabled staging, at the cost of late-mount
visibility in the confined reader. It requires proof of snapshot lineage immediately before each destruction
request: [VM item 11][vm-promote] lost older checkout history to an explicit delete despite the recursive refusal,
and [VM item 14][vm-batch] tested a single batch. It also requires live dataset identity and staged recovery state
(item 4), separates clone creation from key and mount readiness (item 5), and requires boot-time verification of
underlay protection and nested mount reachability (items 8 and 9). The other observations confirm fixture
behavior, not the implementation.

The VM evidence fold-in (`652b51d74d`, 2026-11-18T22:23:49-05:00) recorded items 1 to 11 and their limits. The gate
review fold-in (`34e5fa8fc1`, 2026-11-18T21:55:35-05:00) answered the accepted findings on `71fa25ece8`,
incorporated the [project decisions][answers] on removal, promote and the runtime module gate, and described the
companion boot and shutdown unit.

The design keeps the checkout at its existing path and uses account-owned slots over their Git registrations,
not the first draft's root-owned anchor, `checkout` subdirectory or directory exchange. It draws on the
[cross-vendor review of the original design and its disposition][handover], the [nested-clone research][nested]
and the [checkout-dataset and pnpm research][pnpm], including the nested-clone record's dated corrections.
This proposal changes documentation only; it does not install, implement or alter a pool.

## Requirements and module ownership

The project chose "ZED, Clones, reflink. There is a reason we have a doctor subcommand." The Git wrapper owns Git
validation and registration, unprivileged snapshot and clone creation, the removal workflow and the same-dataset
reflink fallback.
The project decided that the account executes managed clone destruction under delegated `destroy`
([Clone removal identity protection](#clone-removal-identity-protection)).
The root-installed ZFS Event Daemon (ZED) handler and its lifecycle unit own clone mount and unmount, not Git
or automatic deletion.
The Rust binding owns the `/dev/zfs` request format, not ownership policy for Git worktrees or snapshots.
The [handover][handover] records that the ZFS route comes first among wrapper work and that neither old drive
is wiped before the ZFS wrapper has full parity and is built, reviewed and released. The VM results do not
satisfy that condition.

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

The project kept `mount-git-worktree-clone-datasets` in `package/git-policy/cli` as a separate Linux Rust binary
alongside `monochromatic-git-policy-cli`, rather than making a second package, answering "Keep inside cli
(Recommended)". The placement shares the binding and the compatibility gate with the wrapper without adding a
package. Both use the same binding and compatibility gate, even for reads. Its cost is stated under
[Decided defaults](#decided-defaults).
The handler uses child enumeration and dataset/property reads for its sweep and Linux mount calls for the actual
attachment and removal. It has no Git, pool administration, or dataset destruction operation.

### Requests and input ownership

The initial interface is Linux only. Expose typed operations, not a general-purpose ioctl or property setter.
Numbers are for OpenZFS `zfs-2.4.4`, `include/sys/fs/zfs.h:1502` to `:1556`.
The vet's "Result and the owner's decision" records the core set. This draft adds `OBJSET_STATS` for identity
and property sources, `SNAPSHOT_LIST_NEXT`, `GET_BOOKMARKS` and `DESTROY_SNAPS` for exact removal accounting,
and `GET_FSACL` for `doctor`. The cross-vendor review checked `SNAPSHOT_LIST_NEXT` at line 1505 and `GET_FSACL`
at line 1525.

- `ZFS_IOC_OBJSET_STATS` (`0x5a12`): inspect dataset type, identity, origin, and the values and sources of properties.
- `ZFS_IOC_DATASET_LIST_NEXT` (`0x5a14`): enumerate immediate children via the kernel's cursor.
- `ZFS_IOC_SNAPSHOT_LIST_NEXT` (`0x5a15`): enumerate snapshots of one dataset for removal and recovery.
- `ZFS_IOC_SET_PROP` (`0x5a16`): change only the wrapper's desired state and sweep nonce on a named clone.
- `ZFS_IOC_DESTROY` (`0x5a18`): destroy one named, unmounted worktree dataset.
- `ZFS_IOC_PROMOTE` (`0x5a22`, `include/sys/fs/zfs.h:1518`): name a clone in `zc_name`; preserve any conflicting
  snapshot name in the error (`module/zfs/zfs_ioctl.c:6350` to `:6355`). It is not a mount operation.
- `ZFS_IOC_SNAPSHOT` (`0x5a23`): create a named origin; the request names the pool and carries a `snaps` nvlist
  (the kernel's name/value list), not a recursive flag.
- `ZFS_IOC_GET_FSACL` (`0x5a29`): read delegations for `doctor`; inspect grants on ancestors too.
- `ZFS_IOC_DESTROY_SNAPS` (`0x5a3b`): destroy an explicit snapshot name set, preserving per-name failures; the
  request omits the `defer` key, so no snapshot is marked for deferred destroy.
- `ZFS_IOC_CLONE` (`0x5a42`): place the target in `zc_name` and send `origin` and optional `props` in the nvlist.
  The contract is in `lib/libzfs_core/libzfs_core.c:328` and `module/zfs/zfs_ioctl.c:3844` at `zfs-2.4.4`.
- `ZFS_IOC_GET_BOOKMARKS` (`0x5a44`, `include/sys/fs/zfs.h:1552`): enumerate one dataset's bookmarks. The
  registration (`module/zfs/zfs_ioctl.c:7621` to `:7624`, OpenZFS `zfs-2.4.4`) shows the read policy and an
  optional wildcard property list (`zfs_keys_get_bookmarks`, `:4192` to `:4194`); [VM item 13][vm-13] read each
  bookmark's GUID and creation transaction group.
- `ZFS_IOC_CHANNEL_PROGRAM` (`0x5a48`, `include/sys/fs/zfs.h:1556`): run one fixed root-owned program for
  conditional destruction, and only under the root removal alternative the project did not select
  ([Clone removal identity protection](#clone-removal-identity-protection)).

The [project decided][answers] "B: Include promote". Promoting a clone of the checkout makes the checkout a
clone of the promoted dataset and transfers the origin snapshot and the checkout's older snapshots into the
promoted dataset (OpenZFS `zfs-2.4.4`, `man/man8/zfs-promote.8:53` to `:54`; [VM item 11][vm-promote]).
Checkout history must never move into a worktree that ordinary removal can destroy. Neither normal add,
reconciliation nor removal calls promote; grant no `promote` delegation to the account. Keep any blocked origin
and report its dependency rather than promoting a managed clone to clear it.

[VM item 11][vm-promote] measured the promoted worktree becoming independent and the checkout becoming its
clone. The origin and older checkout snapshots moved to the worktree with unchanged GUIDs; later checkout
snapshots and the worktree's own snapshot stayed put. In this draft, "include" means test-only use: expose the
primitive in the binding and exercise it in the disposable VM; no production caller invokes it. For the owner
handoff, distinguish inclusion in the binding from approval of a live operation. A possible future use is explicit
administrator recovery onto a protected clone outside the removable worktree tree (unmeasured, not proposed for
activation here). Prove the ownership of affected snapshot
GUIDs, resulting dependencies and location of protected history before any explicit lineage change. The reverse
promotion in the VM was a control, not a recovery guarantee; no production promote procedure is authorized here.
Adding linked origins does not itself authorize promote.
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
ABI, including module release, architecture, endianness, request contract and kernel/module BTF layout. A missing
device, missing required BTF or mismatch disables raw requests, including reads; no force flag. Unverified
architectures, including `linux-arm64` until separately tested, do not qualify. On gate refusal the wrapper
uses the applicable same-dataset reflink or ordinary Git route for an unused slot. The project decided to verify
the field layout at runtime against the approved release and request contract, answering "Runtime layout check
(Recommended)" ([ABI verification](#project-decisions)); that checker has no design yet, and
[Open items the decisions leave](#open-items-the-decisions-leave) names it. Report what failed, including when an
update makes existing clones temporarily unavailable. A failed gate does not authorize
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
skip legacy datasets. [VM item 10][vm-legacy] measured explicit `zfs mount <clone>` exiting 1 with
`legacy mountpoint` and leaving the slot unmounted despite `canmount=on`. The same clone attached by descriptor.
This confirms the OpenZFS `zfs-2.4.4` mount branch (`cmd/zfs/zfs_main.c:7240` to `:7248`), not the unmount branch.
Treat the refusal as expected; never substitute a path mount. An ordinary clone mountpoint would reopen the
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
this proposal; gating mounts nested under the checkout on checkout readiness is not.
The project decided an ordinary checkout mountpoint plus an immutable covered entry, answering "Immutable entry
(Recommended)" ([Checkout boot mount protection](#project-decisions)). [VM item 8][vm-immutable] measured
that an immutable checkout underlay refused account rename and removal while unmounted, still allowed ordinary
mounting, and retained the flag across reboot with manual import and key load, so that is the entry this route
verifies. Parent rollback to a pre-flag snapshot removed the protection and allowed substitution again. Verify the
intended underlay identity and
protection before automated checkout attach, including after reimport or rollback, not merely at installation.
The decided protection must make the checkout available before account sessions and clone reconciliation.
The probe did not replace an ancestor of the protected entry or exercise unattended boot.
The alternative the project did not select, `mountpoint=legacy` with a descriptor boot service, would have needed
exclusion from automatic mounting and explicit ordering; the legacy mount refusal [VM item 10][vm-legacy] measured
is about the clones, which stay legacy under either answer.

Inventory **every dataset mounted under the checkout path**, regardless of ZFS ancestry. The recorded host
inventory identifies `zpcachyos/nosnap/vm-builder-output`. In [VM item 9][vm-nested-boot], `zfs mount -a` skipped
the legacy checkout but attached its nonclone path descendant to the home underlay. The later checkout attach
hid the output and its marker, but the mount ID remained and the marker returned after checkout unmount. Verify
path reachability as well as mount-table membership. A failure of import, key load or checkout mount must not add
mounts or mountpoint folders to the exposed home underlay. The decided route uses an ordinary checkout dataset, so
it explicitly gates nested mounts on verified success where path ordering cannot fail closed, and a failed
ordinary checkout mount was not measured: prove the failure control and production ordering before enabling it.
The alternative the project did not select, a legacy checkout, would instead exclude every nonclone path
descendant from the automatic scan (candidate: `canmount=noauto`) and require the verified descriptor checkout
provider before its explicit mount unit can run.

### Scoped delegation

Root configuration pins the account UID, the checkout path, pool and dataset names and GUIDs. The proposed
unprivileged lifecycle needs `snapshot` local to the checkout dataset, `clone` on its descendants (snapshots count
as descendants), `create,mount` local to the clone parent, and `mount,userprop` on the clone parent's descendants.
The decided delegated route adds `destroy` there
([Clone removal identity protection](#clone-removal-identity-protection); design alternatives, not product-tested).
The project also decided the checkout-origin grant: adding `mount,destroy` on checkout descendants, which
lets the account destroy **any checkout snapshot**, not just wrapper origins, but not the checkout dataset itself
([Origin cleanup authority](#project-decisions)).
Do not assume those snapshots protect against that account or put unrelated ZFS descendants there.

The `mount` grant is an ioctl prerequisite, not Linux mount-namespace authority. Do not grant `mountpoint`,
`canmount`, `setuid`, `devices`, `rename`, `promote`, `receive` (including `receive:append`) or permissions over the
whole home tree. The removal threat boundary relies on the absence of lineage-changing grants (reasoning from
[VM items 11][vm-promote] and [12][vm-replacement], to be checked in the product).
OpenZFS `zfs-2.4.4` sources: `man/man8/zfs-allow.8:110` to `:120`,
`module/zfs/dsl_deleg.c:574` to `:582`, and `module/zfs/zfs_ioctl.c:826` to `:832`, `:1005` to `:1037` and
`:1128` to `:1148`.

## Add and Git registration

Follow item 7 of the [nested-clone research][nested]: register while the slot is empty, mount at the slot itself,
then install the same Git pointer in the clone. Git accepts an existing empty slot, not a populated clone root.
The research's 2026-11-18 correction distinguishes its snapshot-first path mount from its separate descriptor
probe. [VM item 1][vm-register] measured the combined order: account-owned registration first, root snapshot and
clone, descriptor attach with a fresh canonical check, then the account moved the copied `.git` aside, installed
the saved pointer and ran `git read-tree HEAD`. Git reported the intended worktree and common directory, empty
tracked status and an unchanged underlay pointer after unmount. This supports the order and keeping both pointer
copies. It does not measure deletion of copied administration, delegated requests, or interruption recovery.

1.  Share preflight with `doctor`. Before a claim, slot, registration, snapshot or clone is written, verify the
    supported Git form, eligible source and canonical destination, the checkout mount and caller's visibility,
    `/dev/zfs` access, ABI, effective grants, slot parent identity and owner, mount propagation, installed
    root-owned handler and configuration, live ZED, and usable root-owned status/results. Require a current
    handler readiness report for the actual pool, checkout and boot, not just a present executable. Check the
    decided source eligibility (main checkout only) and capture the index/tree verification inputs. On refusal, name
    the check and choose the applicable fallback without leaving a managed object. Never substitute one source for
    another: a linked source is refused, not replaced by the main checkout.
2.  Claim the destination as the account with an exclusive per-add record in the Git common directory. Write
    the proposed clone name and unique `git-worktree-clone-origin-<nonce>` snapshot name before any mutation
    beyond the claim. Create or verify an empty account-owned slot, then run Git's supported
    `worktree add --no-checkout` form. Record the reciprocal registration and exact `.git` pointer bytes; leave it in
    the slot on the checkout dataset. No handler attestation occurs at registration; the handler first records
    the slot on a qualifying sweep after the clone and its local mount request exist (proposed event ordering).
3.  Snapshot the checkout, then clone that fresh origin, never a sanoid snapshot. Record the actual snapshot and
    clone GUIDs and their `createtxg` values as each is confirmed, as required by the removal proof proposed from
    [VM item 13][vm-13]. A successful clone does not imply mount readiness: [VM item 5][vm-key]
    created a clone with its encryption key unloaded, but detached mount setup refused with `EACCES`. Request
    `mount` via the local desired-state user property described in
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
its expected slot path and identity, pool GUID, origin dataset name and GUID, origin snapshot and clone names,
GUIDs and `createtxg` values, pointer bytes, request nonce and progress. At creation, record the origin-side
inventory of snapshots through the origin snapshot's transaction group and the origin dataset's bookmarks, by
kind, name, GUID and `createtxg`. Record the clone's initial inventory separately; an empty new clone inventory
cannot identify transferred history. This is a proposed proof record based on the transfer in [VM item 13][vm-13].
[Removal and recovery](#removal-and-recovery) defines how to use it. This draft does not name where that binding and
lineage record lives: only the per-add claim record has a named home, the account-owned Git common directory, and
the account writes the binding record wherever it is placed (reasoning: the wrapped add writes it as the account).
Root's conditional destruction therefore has to keep its own copy of the identities it checks, which matters only
for the alternative the project did not select
([Clone removal identity protection](#clone-removal-identity-protection)).
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
The account can delete the underlay pointer while the clone is unmounted. In [VM item 6][vm-prune], Git `2.43.0`
pruned the unlocked registration with a missing pointer on explicit `git worktree prune`, both without `--expire`
and with `--expire now`. A lock set before pointer loss preserved the registration in both arms. Explicit prune
without the flag does not wait for a grace period (Git `v2.43.0`, `builtin/worktree.c:236` to `:248`,
`worktree.c:732` to `:733` and `:772` to `:782`). By contrast, `git gc` defaults to `--expire 3.months.ago`
(`git-config(1)`, `gc.worktreePruneExpire`), so a recent index can keep an unlocked entry with a missing pointer.
Item 6 did not measure the `git gc` path. After pointer restoration, the unmounted locked worktree refused
ordinary and single-`--force` move and remove; double force bypassed both, and Git removal
left the dataset. This supports a lock as a prune guard, not as an authority boundary or a replacement for the
wrapper's managed move refusal. Installing or recovering the lock after pointer loss and Git `2.56.1` behavior
remain unmeasured.

The project decided to acquire a Git lock at registration and retain it through unmounts and restarts, answering
"Git lock + pointers (Recommended)" ([Prune protection while unmounted](#project-decisions)). A lock protects
registration, not the pointer itself or an account deliberately deleting Git admin state. The root-set immutable
pointer the project did not select could hinder accidental deletion of that file, but adds privileged file handling
in an account-owned tree, does not protect Git admin state, and needs removal and parent-rename proof.

A damaged link with surviving admin state can use `git worktree repair <verified-slot>` from the main checkout.
If Git already pruned the admin entry, repair does not reconstruct HEAD, index or lock state. Restore verified
registration data from backup before repairing links; if there is none, report what is lost
and require an explicit recovery or discard, never guessing from a branch or path. A removal whose recorded clone
and origin are already gone can settle a missing registration only after proving there is no replacement.

Because the project decided the Git lock, `doctor` must distinguish the wrapper's recorded protection from an
operator lock, a missing or changed lock, and a missing pointer. Never treat the reason text alone as ownership or
use
`--force` to bypass a lock. An operator or unexplained lock still refuses removal. A verified wrapper lock
would be released only at the authorized registration-cleanup step, after clone destruction and subject to the
origin-cleanup policy; a missing pointer still needs recovery. Installing or recovering the lock after pointer loss
stays unmeasured with the item 6 gap above.

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
`zfs-2.4.4` `include/sys/sysevent/eventdefs.h:97` to `:137`) has `pool_import`, `pool_export` and
`history_event`, but no checkout-mount-ready or pre-unmount/shutdown event. Import does not imply the keys
and checkout are ready. The export command unmounts datasets before issuing export
(`cmd/zpool/zpool_main.c:2410` to `:2434`); the export event is posted during kernel export
(`module/zfs/spa.c:7829` to `:7839`), too late to order those unmounts. Replaying or polling after an
import cannot provide the missing shutdown ordering. The proposed mechanism is the project's
ZED handler for in-session requests **plus** a systemd unit for boot and shutdown, not a second request channel
for mounting; the root removal alternative the project did not select would add one for destruction.

Install the root-owned system unit `reconcile-git-worktree-clone-mounts.service` with `Type=oneshot`,
`RemainAfterExit=yes` and `WantedBy=multi-user.target`. Wire `Requires=` and `After=` to the checkout provider
the boot protection decision names:

- Ordinary checkout, the decided route: the protected `home-user-Monochromatic.mount` if using a generated mount
  unit, or `zfs-mount.service` where it mounts the checkout. The existing host service runs `zfs mount -a`; it
  cannot provide readiness on its own. The checkout is not yet a dataset here and there is no generated unit for
  it.
- Legacy checkout, which the project did not select: the proposed
  `mount-monochromatic-checkout-by-descriptor.service`.

The decided provider must require the actual home and key-loading providers and order after them and
`zfs-import.target`. Order reconciliation after those providers as well, and after `home.mount` and
`home-user-Monochromatic.mount` when systemd tracks those mounts separately, so their stop jobs cannot
race the clone sweep. Run both modes in the host mount namespace. Order startup before
`systemd-user-sessions.service`; install the checkout provider dependency for services of the account that
need the checkout rather than relying on login ordering alone.

- `ExecStart` runs the installed `mount-git-worktree-clone-datasets boot-reconcile`. It verifies ABI, keys,
  checkout and the private propagation the boot unit itself sets
  ([Checkout propagation policy](#project-decisions)), and establishes [private staging](#descriptor-mount-and-unmount)
  before reconciling current intent and publishing this boot's readiness (proposed from [item 3][vm-propagation]).
  A failed prerequisite authorizes no attachment. Report per-clone failures without discarding the unit's
  stop action for mounts that did succeed.
- `ExecStopPost` runs `mount-git-worktree-clone-datasets shutdown-unmount`, including after a failed or interrupted
  start (`systemd.service(5)`, `ExecStopPost=`). Withdraw readiness and close admission to new ZED attaches
  before draining in-flight slot operations. Unmount the verified managed mounts only;
  do not change durable intent. Reverse stop ordering puts the sweep before the checkout/home providers
  stop and before key unload and pool export. Use `Before=umount.target` with `Conflicts=umount.target`;
  when a host has a separate export job, order its teardown after the stop pass. Ordering does not turn a
  failed stop into a successful unmount.

Verify the live checkout before every event action, not just at service start. A busy clone can block normal
teardown or export: report which clone holds the mount, and never force or detach it. Legacy clones have no
systemd mount units and `zfs unmount -a` skips them. Idle export/import must still work, and boot must recover
without an old event queue. A recorded `unmount` remains in effect across restart until destruction or the
cancellation the decision states ([Cancelling a blocked removal](#cancelling-a-blocked-removal)). Periodic
reconciliation without a waiter is not part of this route: the project decided triggers only, and
[Lost last event without a waiter](#project-decisions) names the triggers the design controls.

[VM item 5][vm-key] observed an unavailable key on a successfully created encrypted clone, detached mount
`fsconfig(create)` failing with `EACCES` before attachment, and a root test zedlet receiving a history event
and deferring. After manual key load and verified home and checkout mounts, descriptor attach worked. Before
attach, check the encryption root and key availability and report an unavailable key separately from identity
or permission refusal. Preserve durable intent. The decided trigger set puts one obligation on the checkout
provider: it must trigger the root handler's reconciliation after successful post-key-load checkout readiness,
including after boot; a key-load event before the checkout is ready does not qualify. This proposed provider hook
reuses the lifecycle handler and requires no new add request or waiter. Item 5 used manual key load and checkout
mount; it did not exercise the hook, product handler, delegated clone with the key unloaded, or automatic post-load
recovery.

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

The project kept atomic `/run` receipts, answering "Keep receipt files (Recommended)", rather than root-written
acknowledgement properties, so a result does not create another ZED event. Acknowledgement properties were the
alternative, and each one would create the event the receipts avoid (reasoning from the property event
[VM item 5][vm-key] delivered). Publish in `/run/git-worktree-clone-mount-results`, using only the
canonical numeric dataset GUID as the receipt filename. The root-owned directory is mode `0755`; receipts are
root-owned mode `0644`, read-only to the wrapper, and opened relative to the trusted directory. Nonces appear only as
validated data in the receipt, never in a filename. Include the operation, matching nonce, outcome, mount identity and
failed step/errno where applicable. Readiness also records the current boot, pool, checkout and sweep state. The
runtime files are a cache, not authority to act; rebuild them from live state and durable intent. Do not write
acknowledgement properties from root. A waiter may change the local
`org.monochromatic:git-worktree-clone-mount-sweep-nonce` as often as it needs on the verified managed clone it is
waiting for, not on the clone parent. The nudge uses the same hex format and is not a mount instruction. The handler
cannot limit the nudge to one write: the account owns that property and may change it repeatedly, and each change may
start a sweep that scans every task, so bound the work of one sweep instead of counting nudges. How sweeps
coalesce, and how many may run in a period across every clone, is unspecified here: a rule would have to merge
the nudges pending for one clone into a single sweep and cap sweeps per unit time, and nothing measured sizes
either. The [Lost last event without a waiter](#project-decisions) decision keeps that gap and its cost, and
the triggers-only answer leaves it standing because a nudge is one of the triggers. This
fits the proposed descendant `userprop` grant without expanding it to the parent. The [ZFS planning probes][zfs-plan],
lines 208 to 213, used a parent property for lost-event recovery under their setup; [VM item 5][vm-key]
delivered a clone property event to a test zedlet. Neither measured the product's delegated clone-local nudge or
recovery of the lost last event without another trigger.

Removal needs a fresh all-namespace scan at each point where the removal workflow opens a destructive step: the
removal-ready unmount report, every snapshot-destroy request, whether it carries the first chunk or a later one, and
the head destroy. The wrapper asks for each scan by writing a new
`v1:<nonce>:unmount` intent on the clone it removes, and the handler answers under that same nonce, so a receipt
always names the scan that produced it and no receipt is reused for a later one. The handler repeats its
all-namespace check under each request rather than trusting an earlier clear result. This is a proposed protocol;
[VM item 15][vm-namespaces] ran three root scans per arm, after attach, after the host unmount with both readers
still holding the clone, and after the readers stopped (`item-15-exact.log:87`, `:126`, `:153` for the private arm;
`:237`, `:273`, `:290` for the shared one), and did not exercise repeat requests or a split.

### Descriptor mount and unmount

Apply item 3 of the [nested-clone research][nested] as the mount contract: "The mount must not resolve a path
that the account can change", "The mount must check the target folder's recorded identity, not just its kind",
"The mount must be attached to the opened folder, not to a path", and "What unmount needs, and where the probe was
incomplete". In this design root records identities, not ownership of the account's folders. At setup root records
the identity of `.claude/worktrees`. On the first qualifying sweep after a new clone has a valid local mount
request, root opens, checks and records the current account-created underlay before attach. It must match the
claimed registration slot and hold only the expected Git pointer. This is a first-observation record, not proof
that the slot has not changed since Git created it. Its value is to pin the checked descriptor for attach and to
compare later lookups and unmounts. The no-crossing walk and fresh post-attach check remain mandatory. This is
proposed ordering from the register-first sequence and descriptor controls of [items 1][vm-register] and
[2][vm-refusals], not a measurement of handler timing. Treat the claim and pointer as untrusted; check only a
bounded, regular, no-follow pointer file and never resolve a path from its contents as root. Record and recheck
device, inode, UID, permissions and access/default ACLs from opened folders. Keep records in
`/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities` under root control. Never refresh a slot
identity just because a new inode or permissions appeared there. The handler uses the verified slot descriptors
for mount management, not privileged writes into the account's tree.

Only an administrator may explicitly adopt a legitimate replacement after checkout restore or folder recreation.
The installed root tool's `adopt-worktree-parent` mode runs with managed operations quiesced. Starting from the
pinned checkout mount, walk the canonical components with the no-crossing, no-follow resolver. The administrator
must identify the intended replacement by device and inode, independently verified from the restored checkout or
controlled recreation. Check the held descriptor against those values and its owner, mode and access/default ACLs
against policy. Reconcile all affected bindings against live and staged mounts, claims and Git registrations, then
check a fresh canonical lookup matches the held descriptor before recording its identity. Refuse adoption if
any binding cannot be resolved and report what needs recovery. Never overwrite surviving slot identities or
implicitly adopt a replacement for an existing binding.
Until adoption, refuse managed mutations on the mismatched parent. On an otherwise unused and verified slot,
ordinary same-dataset reflink (rung 2), or native Git when reflink does not qualify (rung 3), remains available
without root. Never use fallback to write over an unresolved managed slot.

The syscall floor is Linux 5.6 for `openat2`; the other mount APIs used here start at 5.2 (`openat2(2)`,
`fsmount(2)` and `move_mount(2)`, "History"). The demonstrated guest baseline is `6.8.0-147-generic` with
OpenZFS 2.4.4 ([nested-clone research][nested], "What was checked and how" and item 3). That validates the
exercised calls on that build, not the mechanism on all kernels at or beyond the syscall floor.

The project decided a functional probe at start, answering "Probe at start", against the ranking the question round
carried, which put an exact allowlist first. The probe's steps, its side effects at start and the way it refuses have
no design yet ([Open items the decisions leave](#open-items-the-decisions-leave)); the project's answer, "test plus
runtime gate", covers the ZFS module check rather than a kernel-release list, and that gate stays. The unselected
allowlist would qualify a release only by passing built handler attach, staging unmount, refusal and namespace
controls on that release, compared
independently of the ZFS ioctl ABI gate; fixture results alone do not qualify it, and `doctor` and preflight report
the running release and any missing qualification. It would also keep the managed route disabled on
`7.3.5-1-cachyos` until a passing run exists there: the applicable same-dataset reflink or ordinary Git path for
unused slots, and preserve existing bindings and intent rather than falling back over them. This is proposed
fail-closed behavior, not a measured readiness gate ([VM item 16][vm-kernel]). The probe admits untested releases
and leaves unmeasured behaviour to appear on the host, where a failure shows as a refused mount or as one the probe
did not detect (reasoning); its own design is not described here, and neither is the allowlist's qualification run,
which is named under [Acceptance tests the build still owes](#acceptance-tests-the-build-still-owes).

Use descriptor-relative `openat2`, `fsmount` with enforced `nosuid,nodev`, and `move_mount` onto the opened target.
[VM item 2][vm-refusals] measured the account creating links successfully and the descriptor walk refusing
links, escapes, mount crossings and recorded identity changes. Access and default ACL drift each passed a mode-only
check, so compare both ACLs. Report the failing component or field; an identity change is not permission to adopt
it. [VM item 7][vm-nosuid] measured a root-owned setuid executable gaining effective UID 0 without the mount
restrictions and retaining the account UID with them. Enforce and verify mount flags independently of dataset
properties; the probe did not execute the handler or test device use in that arm.

After attach, prove both that the held identity received the expected mount and that a fresh walk from the checkout
finds that mount at the canonical slot. In [VM item 2][vm-refusals], a held descriptor moved with a renamed `.claude`
and the attach landed there; the fresh walk detected it. That is a refusal, not a worktree. The synchronized
trials in a private namespace moved the exact mount to staging, unmounted it and verified absence at the canonical
slot and staging. They do not prove atomic placement or cover free-running interleavings. Record the mount identity
and canonical and observed paths. The project decided immediate descriptor rollback, answering "Roll back at once
(Recommended)" ([Misplaced attach](#project-decisions)). Never substitute a path-based attach or unmount on the
account-controlled tree.

For unmount, the process that attached the clone is usually gone. [VM item 4][vm-unmount] measured reopening
and moving an already-attached clone from a fresh process. Reopen the recorded parent from the verified
checkout using the no-crossing `openat2` walk. In a separate `openat2` call, open only the final slot from that
parent with `O_PATH | O_DIRECTORY` and `RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS`.
Omit `RESOLVE_NO_XDEV` only for that hop; the attach-side flags refused it with `EXDEV`. Compare mount ID from
`fdinfo`, device and inode from `fstat`, and the expected slot, parent mount ID, ZFS root and source in the
handler's `mountinfo`. The wrong-clone control reused mount ID, device and inode; only the source differed.
Verify the source's live dataset GUID from the binding as well, and prove this is the handler's attachment
for this boot, not an unrelated bind. The wrong-clone control did not replace a dataset under the same name.
Refuse mismatches and ambiguous identity, including an interrupted attach lacking adequate evidence.

For private staging, the project vetoed the root-owned `0700` bind at `/run/git-worktree-clone-mount-staging` and
decided an account-owned place, then answered where: "Under ~/.cache, and make the wrapper clean after itself."
What the `/run` bind protected, and this answer gives up, is placement outside trees the account controls: a folder
under `~/.cache` is one the account can rename, replace or delete, so the account can substitute or remove the
staging target, and `~/.cache` is itself a recorded substitution exposure for a dataset mountpoint inside an
account-owned home path ([One-time move and checkout boot protection](#one-time-move-and-checkout-boot-protection)),
on the excluded `zpcachyos/nosnap/user-cache` dataset. Nothing here replaces that protection. The descriptor
identity checks still run before each move, and they catch a substitution made before them, not one made after.
The folder's creation, its private bind, and the wrapper's cleanup of it have no design yet
([Open items the decisions leave](#open-items-the-decisions-leave)). Reuse only a verified staging mount
on subsequent sweeps; validate its identity and private propagation before each move. [VM item 3][vm-propagation]
used a private bind of a directory on the guest root filesystem, not this location or a wrapper-created folder. The
location, its lifecycle and the cleanup are untested product requirements, and the placement was one of the
defaults the project answered ([Decided defaults](#decided-defaults)).

Recheck the canonical lookup before moving. The synchronized substitution in [VM item 4][vm-unmount] was
caught by that check; the move uses the verified descriptor, not a new slot path lookup. Move to the decided
staging folder, verify the mount there, release descriptors that would hold it busy, then unmount without
force or lazy detach. The busy-holder arm moved the clone successfully but unmount refused until the holder
exited. Track and report staged identity and location across interruption: absence at the slot does not mean
the mount is gone. Treat absence from the handler's namespace as local detach, not completed unmount.
The project decided reattach when safe, answering "Reattach when safe (Recommended)"
([Busy unmount after a staging move](#project-decisions)), and neither reattach nor staged recovery after a
crash was measured.

#### Mounts held by other namespaces

"Unmounted" means absent in every mount namespace holding the clone's ZFS source, including binds and staging.
[VM item 15][vm-namespaces] started an `unshare` reader and a `PrivateMounts=yes` unit after the attach. With a
private checkout, host-side staging and unmount succeeded, but root's scan found both readers still holding the
clone. The probe could destroy an own snapshot while those copies lived; the head destroy refused with
`dataset is busy`. Head destroy succeeded after both readers stopped. With a shared checkout, descriptor move refused
with `EINVAL`; the positive control used a path unmount solely to exercise propagation. It removed the peer and slave
copies while both readers were alive, and head destroy succeeded. This does not authorize path unmount in the design.

Proposed check based on item 15: the root mount handler groups live tasks by `/proc/<pid>/ns/mnt`, scans the matching
`mountinfo` for each distinct namespace and checks ZFS filesystem and exact dataset source, not slot spelling or cwd.
Before reporting removal-ready unmount, exclude new managed attaches with the standing `unmount` intent and return
holders and incomplete reads in a fresh, nonce- and GUID-bound result. Removal requires this check immediately before
every snapshot-destroy request, the first chunk or a later one, and again before the head destroy. Report unreadable
or racing entries rather than treating them as clear. The scan only detected process-visible holders in the VM; it
does not prove race-free absence, coverage of pinned namespaces without a visible task, or coverage through different
process root directories.
For a managed-consumer deployment, propose that the wrapper and consumer launcher close per-worktree admission and
finish draining consumers and namespace handles before the root handler scans. Keep admission closed through the
head destroy. The deployment must also exclude untracked copies and pinned handles and prevent independent new
mounts during that interval; stopping the managed consumers alone does not do that. A channel program excludes ZFS
administrative changes, not Linux namespace changes (`man/man8/zfs-program.8:35` to `:40`).

Item 15 ran no rootless container: an unrestricted caller such as a container attaching the clone after a scan is
unmeasured, and nothing it ran shows an exclusion that keeps one out, so a clean scan alone does not qualify that
deployment. It found no complete namespace or mount-reference exclusion: its scan saw readable, live representatives
of the observed namespaces only, and does not generalize to pinned namespaces or racing task changes
([VM delta limits][vm-delta-limits], "What this repeat did not show"). Nothing this draft describes shows one
either, because the descriptor checks and the namespace scan act on what root can observe (reasoning from
[item 15][vm-namespaces], not a measured interleaving). The project decided what removal does in that state,
answering "Destroy anyway (Recommended)": the scan that refuses holders it sees stays, then the clone's snapshots
and the head are destroyed
([Removal when the namespace exclusion cannot be shown](#project-decisions)). A deployment that can show the
exclusion still accepts the same residual: a clean scan that misses a later copy leaves snapshots gone with the
head still busy, the resume state
[Removal and recovery](#removal-and-recovery) names. That loses no history the project chose to keep, since
"B: Destroy at once" already declines to retain a removed worktree's own snapshots (reasoning from
[item 15][vm-namespaces], not a measured interleaving). The
[propagation decision](#project-decisions) carries these conditions as facts.

[VM item 3][vm-propagation] measured the move refusing with `EINVAL` when its immediate parent, the checkout,
was shared. Making **only the checkout mount private** let the fresh process move to private staging and unmount;
`/` and both home ancestors stayed shared. This descriptor route requires a verified private immediate parent and
verified private staging, not private ancestors of the checkout. Other non-shared propagation was not measured.
The clone mount rows also read `private` in the descriptor trials of items [2][vm-refusals] and [4][vm-unmount]
(`item-2-exact.log:73` and `item-4-exact.log:42`); do not infer those results transfer to a shared checkout.
The tested private checkout stopped delivery
of the later clone mount to a reader whose already-running unit had a separate mount namespace. Check the
caller's actual visibility and refuse managed attach where safe unmount and staging topology are unproved.
The project decided that the boot unit sets the checkout's propagation, answering "Boot unit sets it
(Recommended)" ([Checkout propagation policy](#project-decisions)).

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

Reuse the wrapper's Git registration/back-link, lock, clean and held-directory boundary checks. The project gave one
supported `--force` a second meaning beside waiving cleanliness: it ends the holders of the clone (the cancellation
decision under [Project decisions](#project-decisions)). It does not waive an operator or unexplained lock, mount
boundary or identity check. Because the project decided the Git lock mechanism, handle a proven wrapper lock only
as specified in
[Unmounted, pruned or moved worktrees](#unmounted-pruned-or-moved-worktrees). Inspect the mount tree and the dataset
tree separately for nested worktrees, datasets and mounts, including binds; inspect the Git registration too. If an
inspection cannot prove its result, refuse instead of falling through to Git's recursive removal for a managed
slot. See the [wrapper specification][spec], lines 2932 to 2943 and 3064 to 3190, and `find_boundary_below` in
`package/git-policy/cli/src/native/subvolume/subvolume_remove_walk.rs`.

The [project decided][answers] "B: Destroy at once". A normal authorized removal destroys the clone and its own
snapshots in that invocation. It does not prompt, require a separate snapshot-deletion flag, or wait out retention.
It waits for verified unmount within the [deadline](#waiting-and-refusal) once sized, and for the destroy results,
not for background space reclamation. No described setup can show the namespace exclusion, so the project decided
what removal does in that state: "Destroy anyway (Recommended)", keeping the scan that refuses the holders it sees
([Removal when the namespace exclusion cannot be shown](#project-decisions) and
[Mounts held by other namespaces](#mounts-held-by-other-namespaces)).

[VM item 11][vm-promote] shows why the lineage check must precede **any** snapshot delete. Recursive destroy of
the promoted clone refused and left the snapshot set unchanged. But an explicit delete erased the relocated
older checkout snapshot before the kernel refused to delete the relocated origin with
`snapshot has dependent clones`. Promoting back did not restore the lost snapshot. The dependent-destroy dry run
included the checkout and its later snapshot (`item-11-exact.log:69` to `:94` and `:123` to `:135`). Never issue
recursive or dependent-clone destruction; a dependency refusal protects the specific snapshot it names, not the
rest of the history.

[VM item 14][vm-batch] sent the mixed `@earliest` and `@origin` batch through both paths. The CLI refused a
snapshot with a dependent clone before issuing any `ZFS_IOC_DESTROY_SNAPS` ioctl. The direct request returned
`EEXIST`, named only `@origin` in the error list, and kept both snapshots. The own-snapshot control deleted both
own snapshots in one ioctl. The kernel prechecks the whole request before deleting when a check fails
(OpenZFS `zfs-2.4.4`, `module/zfs/dsl_destroy.c:598` to `:600` and `:630` to `:653`). That guarantee covers check
failures only: a program error returns the errno with no per-name list (`dsl_destroy.c:662` to `:671`), and an
exhausted program can leave partial effects (`man/man8/zfs-program.8:152` to `:155`). The kernel's own batch runs
with an instruction limit of 0, which means no limit (`module/zfs/dsl_destroy.c:656` to `:660` passes it;
`module/zfs/zcp.c:796` tests the limit only when it is not 0), so only the memory limit applies;
a root program chooses its own limits. The request is still name-based, ignores missing snapshot names
(`lib/libzfs_core/libzfs_core.c:442` to `:456`), and is not atomic with the later head destroy. Neither a
successful batch nor a dependency refusal establishes provenance.

The kernel reads a snapshot's deferral from the presence of a `defer` key, never from its value: the accepted key
is a bare boolean (OpenZFS `zfs-2.4.4`, `module/zfs/zfs_ioctl.c:4119`), the handler asks only whether the key exists
(`:4132`), and `libzfs_core` adds it only when it defers (`lib/libzfs_core/libzfs_core.c:475` to `:476`). A pair of
another type is refused before that (`zfs_ioctl.c:7922` to `:7924`, `ZFS_ERR_IOC_ARG_BADTYPE`), but a bare boolean
carries no value, so a binding that meant "false" and added the key would still defer. Deferring marks a snapshot
with dependent clones for deferred destroy instead of refusing it, which would reintroduce item 11's loss and mark a
foreign relocated origin for later reaping (`libzfs_core.c:442` to `:451`). The binding therefore omits the key, and
a test asserts that the encoded request has no `defer` key.

1.  Prove the live pool, clone and origin dataset GUIDs match the binding. Then prove lineage in this order:
    - Necessary check: read the clone's live `origin` and resolve it to the recorded origin dataset and origin
      snapshot GUID. Require the recorded direct origin, not just a name match. Check the recorded clone and
      origin snapshot `createtxg` values against the live ones as well.
    - Independent check: inventory all clone snapshots and bookmarks by kind, name, GUID, owning dataset GUID
      and `createtxg`. Require each snapshot's `createtxg` to be strictly greater than the clone's own. Include
      manual and sanoid snapshots. A bookmark carries the GUID and birth transaction group of the snapshot it was
      made from, not its own (`module/zfs/dsl_bookmark.c:352` to `:353` copies them from the snapshot and `:719`
      to `:728` reports them as the bookmark's `guid` and `createtxg`, OpenZFS `zfs-2.4.4`; [VM item 13][vm-13]
      saw the tuple unchanged after a move). Apply the
      [Bookmarks on a clone being removed](#bookmarks-on-a-clone-being-removed) rule the project decided, "A: Live or
      planned (Recommended)": name every bookmark that rule refuses and stop before the first delete, and
      treat a bookmark list that could not be read completely as a refusal. Dataset GUIDs are random
      (`module/zfs/dsl_dataset.c:1203` to `:1204` and `:1768` to `:1769`, `zfs-2.4.4`), so a matching tuple
      identifies the bookmarked snapshot of this clone in practice, not against a 64-bit collision. A bookmark's
      transaction group is the bookmarked snapshot's birth group, so it says nothing about when the bookmark
      itself was made. OpenZFS destroys a dataset's bookmark entries with the head
      (`module/zfs/dsl_destroy.c:1123` to `:1154`, `zfs-2.4.4`), so an unaccounted bookmark goes with it.
    - Third check: compare the live clone inventory against the recorded origin-side GUID inventory, including
      snapshots and bookmarks. Any origin-side hit or unproved provenance refuses destruction. Never update the
      recorded lineage to match an external change.

    [VM item 13][vm-13] measured this separation only on OpenZFS `2.4.4-1`: the transferred snapshots kept
    their GUIDs and transaction groups at or below the origin snapshot's, the own snapshot was newer than the
    clone, the clone lost its direct origin, and the bookmark moved with an unchanged tuple. These are layered
    proposed checks, not a proof that transaction groups establish provenance for other topologies, encryption
    roots, lineage changes or objects created in the same transaction group.
2.  If lineage, ownership or the safety of the destructive operation cannot be proved, refuse before the
    first delete; the namespace exclusion is not one of these proofs, and the project decided what removal does
    when it cannot be shown ([Removal when the namespace exclusion cannot be shown](#project-decisions)). Name the
    mismatched dataset, snapshot or dependency, keep the registration and removal
    record, and report the need for administrator lineage recovery before retry. Do not try a subset, promote
    back automatically, or invoke ordinary Git removal to clear the slot. For a proven set, record removal
    intent and print the slot, dataset and exact snapshot names to be lost. Release local handles and request
    `unmount`. Require the root handler's [all-namespace unmount check](#mounts-held-by-other-namespaces) and
    verify the request, clone GUID, underlay identity and Git registration still match. A missing, mismatched
    or busy unmount blocks destruction, even when the slot is clear (proposed from [item 15][vm-namespaces]).
3.  After unmount, persist the verified inventory and expected clone identity as the destruction plan. That inventory
    is step 1's, which lists bookmarks as well as snapshots; the plan records each snapshot tuple this removal
    verified and submitted for destruction, with that tuple's outcome, and no bookmark entry. Require fresh root
    all-namespace verification before the first snapshot delete. Reprove lineage and
    candidate identities immediately before submitting the full verified snapshot set in one
    `ZFS_IOC_DESTROY_SNAPS` request that omits the `defer` key; retain the per-name error list (direct route run as
    root, [item 14][vm-batch]).
    Item 12's grant set, which [Scoped delegation](#scoped-delegation) proposes except for the checkout's local
    `snapshot`, let the delegated account submit the clone's own snapshots in one request: it returned 0, and a name
    with a dependent clone returned `EEXIST` with both kept ([item 17][vm-17] on OpenZFS `2.4.4-1`;
    [item 18][vm-18] repeated it on an unpromoted clone with the same returns).
    The root alternative, should it ever be selected, would require equivalent whole-batch checks and per-name
    results inside its single channel program request, not an external name-based ioctl after a conditional check
    (unmeasured).
    Skip the snapshot request for an empty set. Only then destroy the verified unmounted head under the decided
    [delegated identity policy](#clone-removal-identity-protection). A per-slot claim only serializes cooperating
    wrapper operations. [VM item 12][vm-replacement] shows that another GUID read does not bind the later name-based
    request.
    Under either policy, administrators must quiesce managed operations before outside lineage changes and invalidate
    affected removal proofs (unmeasured operating rule).

    If the verified set exceeds the supported request budget, the proposed split is to sort by name and take the
    largest fitting prefix of the remaining recorded tuples, repeatedly. Budget for encoded inputs and per-name
    results, and for program limits on the root route. Establish those limits in the VM; if even one tuple cannot
    fit, or the error left an unknown outcome, stop rather than retrying by name. Before every chunk, verify prior
    outcomes and reprove the full remaining inventory and lineage, not just that chunk. A split reopens the
    inspection-to-use window and can leave partial cleanup across requests; item 14 did not test limits or a split.
    Holds and dependencies are named, not bypassed. An inventory change invalidates the submitted plan for any
    future request. If the only change is newly created own snapshots, the wrapper, not an operator, reruns all
    lineage and unmount checks, records the amended plan with every earlier tuple and its outcome kept, prints the
    additional names and proceeds without a prompt.
    This includes a proven own sanoid snapshot; a missing or changed recorded identity, unproved new history, or
    unknown prior outcome stops the attempt. After each known batch outcome, enumerate again before head destroy.
    A snapshot created after the final enumeration can still block head destroy (`module/zfs/dsl_destroy.c:783`
    to `:790`, OpenZFS `zfs-2.4.4`); retry only after identifying the new own snapshot and reproving the plan, not
    on unexplained `EBUSY`. Before head destroy, also run the check the paired answer requires, "Keep both, check
    first (Recommended)": that the clone and its origin are the planned ones and that the origin is not marked for
    deferred destroy, refusing otherwise, because
    destroying the last clone can reap an origin already marked for deferred destroy outside the separate
    origin-cleanup step (`dsl_destroy.c:1028` to `:1037` and `:1168` to `:1172`). That check has no design yet, and a
    same-account mark between the check and the destroy stays inside the accepted residual
    ([Open items the decisions leave](#open-items-the-decisions-leave)). Bound replanning by
    the same removal deadline, without restarting it. Print what disappeared and what remains and keep per-name
    results. This is proposed handling for recursive sanoid coverage ([project decision][answers], lines 365 to
    371) and the separate batch and head requests ([VM item 14][vm-batch]); the interleavings are unmeasured.
4.  For separate origin cleanup, prove the exact recorded origin dataset and snapshot GUIDs and confirm that
    the snapshot still belongs to that dataset. Apply the same identity rule using the delegated authority the
    project decided, the descendants-only `mount,destroy` grant on the checkout
    ([Origin cleanup authority](#project-decisions)). The kernel's `snapshot has dependent clones` refusal is the
    dependency check, not a scan of children or a name test.
    Leave a refused origin and report it; never destroy a dependent clone to free it. Preserve the incomplete
    cleanup record while an origin remains.
5.  Settle only the matching Git registration and pointer after clone destruction is proven. The project decided to
    settle them even while origin cleanup is blocked, answering "Settle + own record": a separate durable record
    carries the origin cleanup, and that record has no design yet
    ([Open items the decisions leave](#open-items-the-decisions-leave)). Remove the leftover
    underlay folder as the account only after checking it has the recorded identity and is empty. Do not remove
    an unfamiliar entry, a replacement registration or any mounted tree. Legacy dataset destruction does not
    remove the slot. Older checkout snapshots can still hold that underlay and pointer until they expire; this is
    not surviving live worktree data.

Resume against the persisted destruction plan and the live mount, dataset, snapshot and Git identities. Settle
recorded destroys only after proving the expected objects are gone, never from a missing name alone. While the head is
present, require it to answer with its recorded live GUID and an empty live snapshot list. Once the head is gone, that
read is impossible, so settle the head and its snapshots only when its name is absent and no child of the pinned
parent carries the recorded GUID. These observables are proposed, not measured. If interruption left an outcome
unprovable, retain it as unresolved and stop; never regenerate an authorized set from whatever names are left.
Distinguish unmount pending or staged, planned snapshots all gone with the head refused, a partial snapshot set left
by a program that failed part way, which covers the kernel's own batch as much as a root program
(`module/zfs/dsl_destroy.c:656` to `:660` and `:662` to `:671`, OpenZFS `zfs-2.4.4`), clone gone with its recorded
origin retained, and Git or empty-folder cleanup still owed.
Report specific blockers: `EBUSY` and holder if known, snapshot history at risk, a hold, nested dataset or mount,
dependent clone, a mismatched identity or a missing or pruned registration. Never let a stale acknowledgement
authorize cleanup.

### Cancelling a blocked removal

A refusal after setting `unmount` leaves removal intent in force, and the project decided what happens next in
two parts, in the owner's words: "Kill or otherwise remove or destroy the holders whenever --force is specified,
auto-cancel if not." With `--force`, removal ends the holders and goes on; without it, the blocked removal cancels
itself. No `cancel-worktree-removal <slot>` command and no report-only recovery exist: the earlier draft offered
those as this choice's options, and the project answered over both.

Auto-cancellation can happen after the removal has already destroyed history. The project's "B: Destroy at once"
([project decision][answers], lines 491 to 494) fixed the order, and the
[destroy-anyway decision](#project-decisions) keeps it: the clone's own snapshots are destroyed first, and the head
destroy is the step a holder blocks. A removal that cancels because a clean scan missed a holder therefore reverses
intent on a clone whose own snapshots are already gone. What it restores is that clone without its snapshot
history, and nobody has designed whether the clone is reattached as it stands, whether the removal record is
reversed or kept, or what `doctor` reports ([Open items the decisions leave](#open-items-the-decisions-leave)). The
reattach of the busy-unmount decision ([Busy unmount after a staging move](#project-decisions)) is the one remount
the no-implicit-remount decision allows, and it restores visibility, not those snapshots: the loss is reported and
is not recoverable.

The project also asked for the forced arm to be measured in a VM, answering "Measure in a VM (Recommended)", and
the route rests on that measurement working
([Forced removal and deferred origins](#forced-removal-and-deferred-origins)). Who ends a holder that is not a
process of the account, and by what authority, has no design yet either
([Open items the decisions leave](#open-items-the-decisions-leave)).

What the earlier draft's guarded reversal had to prove still applies to whatever restores a cancelled removal:
under the per-slot claim, settle all outstanding destroy requests and prove the clone, original lineage, Git
registration, mount state and exact destruction outcomes before reversing intent. Report any confirmed history loss
before doing so and retain that report. If a destroy outcome is unknown or the head is gone, do not issue `mount`;
report the matching resume step or administrator recovery required. This is proposed recovery reasoning from the
staged holder and namespace cases in [VM items 4][vm-unmount] and [15][vm-namespaces], not a tested reversal.

## Ladder and fallback

### Rung 1: Snapshot clone

The ZFS snapshot clone sits alongside the existing Btrfs route. Keep `git worktree add <path>` semantics
and the ladder's already recognized forms ([wrapper specification][spec], lines 1121 to 1189). The described add
uses the configured main checkout as origin. An unproved source never qualifies. The project decided a verified
dirty-main restore comparable to Btrfs, answering "Verified restore (Recommended)"
([Dirty source on rung 1](#project-decisions)); a clean-only rule would have skipped the clone route whenever other
sessions hold uncommitted edits in the main checkout. Reuse held-path
and mount-fact helpers and the current Git index, attribute, conversion and publication checks. Respect Btrfs's
main-checkout leave-out policy for ignored state, but verify
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

Use the existing Git checkout and ignored-state behavior when a reflink is refused, cross-dataset, or unproved.
`FICLONE` returns `EXDEV` across datasets, but a successful `copy_file_range` can share blocks across datasets
with the same data key.
An encryption root alone is not the same key, and a cross-dataset copy does not qualify for rung 2.
See the [pnpm study][pnpm], "When a copy still shares blocks".

### Confined callers

The Pi driver already creates its worktree before starting the confined unit
(`package/pi-shared/subagent-driver/src/start-pi-run-in-a-worktree.ts:238`). The planning probes found a confined
user namespace could not see the source dataset through `/dev/zfs`, while same-dataset reflinking worked.
A read-only bind of the worktrees parent can hide a mount made after a unit starts. Without that bind, the
already-confined reader in [VM item 3][vm-propagation] saw the later clone and its marker with a shared checkout,
not with a private checkout. The marker was written by the mounting process, so the control showed reader
visibility, not consumer writability. Do not relax confinement or infer mount visibility from host success.
Prefer creating the worktree outside the unit as the driver already does, and verify visibility and writability
in the consumer's actual context. [VM item 15][vm-namespaces] showed post-attach consumers inheriting the clone
under a private checkout and keeping it after host unmount. The proposed removal gate requires those holders to be
released before deleting history; that item did not run the installed Pi driver.

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
`git-worktree-clone-origin-` snapshots, and cannot free blocks retained only by them. The project kept the guard's
current scope, answering "Current scope (Recommended)" ([Space-pressure pruning](#project-decisions)), so guard
expansion is not implicit in making home recursive and the limit is reported.

## What doctor checks and prints

Extend the read-only filesystem report (`package/git-policy/cli/src/native/health/doctor_filesystem.rs:1` to `:30`).
Use the same facts as managed preflight. Distinguish healthy, known drift and unreadable or unsupported checks;
name which add route remains available. Setup findings go to standard error and do not change `doctor`'s exit
status. `doctor` does not start a service, create a test dataset, repair registrations or change configuration.

- Check `/dev/zfs`, the ABI, the pinned pool and dataset identities, encryption root and key availability, clone
  parent and clone properties, checkout mount, checkout boot protection, propagation and caller visibility.
  Check path reachability of nested mounts; a mount-table entry alone can describe a hidden mount.
  For the root-once move print the verified cutover and rollback steps, the decided dataset and the immutable
  covered entry with its setup commands, and paths with the actual owner and expected properties. Do not propose
  chowning the account's slots to root.
  On ABI refusal name the loaded module and target plus the supported compatibility evidence, not a bypass.
- Check recorded parent and slot identities and name the failing field or path component. For a legitimately
  recreated parent, print the administrator's verified `adopt-worktree-parent` command and prerequisites, any
  binding that needs separate recovery, and the applicable fallback for an unused slot. Inspect effective
  delegation including ancestor grants, account groups and grants to everyone. For managed removal, check that
  `promote`, `rename`, `receive` and `receive:append` are absent; the delegated `destroy` grants the project decided
  are present and scoped, and the deferred-origin check the paired answer requires is described
  ([Clone removal identity protection](#clone-removal-identity-protection)). The check that the account cannot
  destroy managed clones belongs to the root alternative the project did not select (proposed checks for
  [item 12][vm-replacement]).
  Print scoped commands such as `zfs allow -l -u` and `zfs allow -d -u`
  with the actual account and datasets, only for grants in the decided policy. Do not grant origin cleanup more
  broadly just to make an interrupted command pass.
- Check the handler installation, config and root-owned state and result directories, ZED and service wiring,
  latest readiness and reconciliation, and unresolved requests. Check the staging folder under `~/.cache` and the
  propagation of its live bind separately from the checkout ([VM item 3][vm-propagation]); the folder's creation
  and the wrapper's cleanup of it have no design yet
  ([Open items the decisions leave](#open-items-the-decisions-leave)). Print root install/enable
  commands and the journal queries for `zfs-zed.service` and `reconcile-git-worktree-clone-mounts.service`.
  Show other enabled zedlets before recommending ZED enablement, including applicable syslog, notification,
  enclosure or scrub effects.
- Compare the measured dataset tree and store location against recursive coverage. Verify the named exclusion
  sections, ancestor order, mount paths and `com.sun:auto-snapshot=false`; print the concrete INI correction or
  `zfs set com.sun:auto-snapshot=false <verified-dataset>` for confirmed drift. This keeps the planning record's
  veto-open adoption of blocklist checks. Report the space guard's actual scope.
- Report unmounted and staged clones, busy checkout teardown with the holding clones named, interrupted actions,
  damaged or pruned Git registrations, and each living clone's recorded origin. For the proposed root scan from
  [VM item 15][vm-namespaces], name each namespace holding the clone by namespace ID and representative PID, with
  matched source and path, and report incomplete coverage as unknown. Do not require root in the `doctor` caller.
  For a blocked removal, show standing `unmount` intent, the last verified loss and remaining objects, whether the
  removal will cancel itself without `--force` or end the holders with it, and which resume or administrator
  recovery step remains (proposed recovery state, not measured).
  Report snapshot retention using named ZFS accounting fields where measurable; do not sum shared blocks as if each
  clone owned them. Explain retained underlays in checkout snapshots. Print the applicable guarded resume or
  link-repair steps, not a blanket destroy or a guessed registration.
- Report whether the configured backup job can see the checkout and whether filesystem boundaries are known to
  be enforced. A walker that crosses into a clone may protect the same contents again under the checkout path.
  `kopia-snapshot-all.service` was inactive with a missing executable here; another host's recorded ignore
  policy is not evidence for this one. Report unknown coverage as unknown until tested, not as a backup failure
  ([nested-clone research][nested], item 5).

The command text, service ordering and findings are tested in a disposable VM before presenting commands as
working setup. This proposal has neither installed nor run the root-once commands.

## Project decisions

The project decided every choice and every default this file listed, answering in the question tool on 2026-11-19.
The words recorded for each answer are in `doc/handover/questions-for-the-owner.md`, section "Answered in the
question tool, started 2026-11-19T11:30:00-05:00"; each item below quotes its own answer and keeps the measured
facts, citations and measurement item numbers the item already carried. The answers do not reopen checkout path,
account ownership, ZED, the fallback or recursive snapshot policy. The four defaults are stated separately under
[Decided defaults](#decided-defaults). The two alternatives under
[Clone removal identity protection](#clone-removal-identity-protection) are one decision, and the three bookmark
rules under [Bookmarks on a clone being removed](#bookmarks-on-a-clone-being-removed) are another. The identity
decision and [Origin cleanup authority](#project-decisions) were asked twice, once as a pair, because the grants
couple them; the pair's answer adds a check rather than a ranking between them. The bookmark rule is coupled to the
identity decision, because the plan every rule persists is written by the program that decision puts in charge and
the decided bookmark rule reads it (reasoning). Parts an answer created with no design yet are listed under
[Open items the decisions leave](#open-items-the-decisions-leave). Nothing here
is approved, released or implemented: this file remains a non-final draft.

- Checkout boot mount protection. Decided: "Immutable entry (Recommended)", the ordinary checkout mountpoint plus
  an immutable covered entry. [VM item 8][vm-immutable] showed the flag surviving ordinary mount and reboot, but
  not rollback to a pre-flag parent snapshot. [VM item 9][vm-nested-boot] showed an automatically mounted path
  descendant hidden by a later legacy checkout attach, which the alternative the project did not select,
  `mountpoint=legacy` with a descriptor boot service, would have had to exclude from automatic mounting and order
  explicitly. The decided route gates all path descendants on verified checkout readiness. Failed ordinary mount
  and unattended boot remain unmeasured.
- Checkout propagation policy. Decided: "Boot unit sets it (Recommended)". The installed boot unit applies private
  propagation to the verified checkout at boot and repeats it after a replacement mount. The [VM record's host
  inventory][vm-fixture] observed `shared` for `/`, `/home` and `~/.cache`; the checkout does not yet have a
  separate mount, so its propagation is not measured here. The alternative the project did not select, an
  administrator setting it once, does not persist across reboot or replacement of the checkout mount: propagation
  belongs to a mount instance (`mount_namespaces(7)`, "Shared subtrees"). Recheck it regardless of who sets it.
  [VM item 3][vm-propagation] measured successful staging with only the checkout private, and loss of late
  delivery to the already-confined reader in that arm. The shared arm preserved delivery but could not move
  the clone to staging. [VM item 15][vm-namespaces] measured post-attach namespaces holding the clone after private
  host unmount; the shared-path unmount removed the peer and slave copies, as a control only. Private staging
  does not eliminate the all-namespace check. The decided setup qualifies how it excludes untracked holders and
  new copies through destruction, not merely what the handler's namespace sees (unmeasured; see
  [Mounts held by other namespaces](#mounts-held-by-other-namespaces)). No alternative descriptor unmount was
  tested. Do not silently change propagation. Two facts belong to this decision: a deployment that can show the
  exclusion still accepts planned snapshots all gone with the head refused when a copy survives the scan (the
  outcome [VM item 15][vm-namespaces] measured), and no arrangement this draft describes shows that exclusion, so
  [Removal when the namespace exclusion cannot be shown](#project-decisions) settles what removal does there. No
  option here, and no deployment this draft names, can show that exclusion today: [VM item 15][vm-namespaces]
  found no complete namespace or mount-reference exclusion, and this draft adds none. A third arrangement would
  have to bring its own unmount and visibility proof.
- Removal when the namespace exclusion cannot be shown. Decided: "Destroy anyway (Recommended)": the scan that
  refuses the holders it sees stays, then the clone's snapshots and the head are destroyed.
  [VM item 15][vm-namespaces] found no complete namespace or mount-reference exclusion: its scan saw readable, live
  representatives of the observed namespaces only, and did not generalize to pinned namespaces without a visible
  task or to racing task changes
  ([VM delta limits][vm-delta-limits], "What this repeat did not show"). Nothing this draft describes shows one
  either, so this decision settles what removal does in that state. The measured facts: with a private checkout, the
  host unmounted while both post-attach readers still held a ZFS row for the clone
  in `mountinfo` and neither worked in the clone directory, a disposable own snapshot was destroyed while they
  were alive, and the head destroy refused `dataset is busy` until both stopped ([item 15][vm-namespaces]). No
  read through the mount was measured. The rest is reasoning: the snapshots this order destroys are gone for good,
  so a removal whose holder outlasts them ends in "planned snapshots all gone with the head refused" until the
  holder lets go, and [Cancelling a blocked removal](#cancelling-a-blocked-removal) states what cancellation leaves.
  Those facts are about holders the scan sees: both of item 15's readers appeared in its scan with a hit
  (`item-15-exact.log:126` to `:138`), while a holder the kept scan missed, and one that attached after a clean
  scan, are both unmeasured. The route keeps the scan: it still needs the fresh root
  all-namespace verification before the first snapshot delete and before the head destroy
  ([Requests, results and ordering](#requests-results-and-ordering)), and a missing, mismatched or busy unmount
  still blocks destruction even when the slot is clear ([Removal and recovery](#removal-and-recovery), step 2),
  unless `--force` ends the holders (the cancellation decision below).
  That scan is proposed protocol; item 15 ran three root scans per arm, after attach, after the host unmount
  and after the readers stopped (`item-15-exact.log:87`, `:126`, `:153`; `:237`, `:273`, `:290`), and did not
  exercise repeat requests. The alternative the project did not select, refusing before the first snapshot delete,
  would leave the decided "B: Destroy at once" ([project decision][answers], lines 491 to 494) unreached for every
  deployment whose arrangement this draft describes, and an authorized removal would be destroyable only through
  action outside the wrapper (reasoning). The project declined a refusal of that kind once: when it chose
  "B: Destroy at once", the offered "refuse without an explicit flag" was declined
  ([project decision][answers], lines 491 to 494).
- Bookmarks on a clone being removed. Decided: "A: Live or planned (Recommended)", a bookmark passes when a live
  passing clone snapshot or a recorded plan tuple explains its tuple. The three options, their facts, their costs
  and what the decided rule still needs are under
  [Bookmarks on a clone being removed](#bookmarks-on-a-clone-being-removed).
- Kernel-release readiness. Decided: "Probe at start", a functional probe at start, against the ranking the
  question round carried, which put an exact allowlist first. The probe's own steps, its side effects at start and
  the way it refuses have no design yet; [Open items the decisions leave](#open-items-the-decisions-leave) names
  them. The runtime module gate of "A: Test plus runtime gate" stays, and the unselected allowlist's facts stay:
  it would qualify a release only by built-product runs on that release and would disable the managed route after
  every CachyOS kernel package update until a run passes, which makes existing unmounted clones unavailable in the
  meantime. No release on `7.3.5-1-cachyos` has been qualified: [VM item 16][vm-kernel] searched the built-in images
  and the local image directory only, did not query the remote catalog, and the guests this record used boot
  `6.8.0-147-generic`; the custom-template route named
  under [Acceptance tests the build still owes](#acceptance-tests-the-build-still-owes) stays open. A functional
  probe admits untested releases and leaves unmeasured behaviour to appear on the host, where a failure would show
  as a refused mount or as one the probe did not detect (reasoning). The decided route
  neither authorizes a path mount nor withdraws the runtime module gate, and it preserves existing bindings and
  intent while the route is unavailable.
- Blocked-removal cancellation. Decided, in the owner's words: "Kill or otherwise remove or destroy the holders
  whenever --force is specified, auto-cancel if not." So `--force` gains a second meaning beside waiving
  cleanliness: the holders are ended. Without it, a blocked removal reverts by itself, and no
  `cancel-worktree-removal <slot>` command exists: the earlier draft offered the guarded command and report-only
  recovery as this choice's two options, and the project answered over both. The second part of the answer is a
  measurement, "Measure in a VM (Recommended)", and the forced arm is adopted only if it works. Who ends a holder
  that is not a process of the account, and by what authority, has no design yet, and neither does what the
  auto-cancellation restores once the clone's own snapshots are already destroyed: both are under
  [Open items the decisions leave](#open-items-the-decisions-leave). [VM item 4][vm-unmount] and
  [VM item 15][vm-namespaces] measured the staged and namespace cases this rule must survive, not the holders it
  ends or the reversal; forcing the removal of a clone a holder keeps is one of the two VM measurements the build
  now owes ([Forced removal and deferred origins](#forced-removal-and-deferred-origins)).
- Managed path interface. Decided: "Canonical path only (Recommended)". The route accepts the canonical path from
  ordinary `git worktree add <path>` only, with fallback at the same slot; the unselected opt-in named allocator
  would have reported a chosen path under `.claude/worktrees` and added a caller-facing command.
  [VM item 1][vm-register] used an explicit canonical path under that parent, not an allocator or the wrapper's
  command forms.
- Eligible clone origins. Decided: "Main checkout only (Recommended)". A linked origin would preserve that
  worktree's ignored state but chains dependencies and can block origin cleanup. [VM item 11][vm-promote] measured
  how promotion transfers custody and reverses a dependency, not a safe removal or recovery procedure for a chain
  of linked origins. Neither route makes promotion automatic.
  Do not substitute a different origin from the one the caller supplied.
- ABI verification. Decided: "Runtime layout check (Recommended)": verification of the field layout at runtime
  against the approved release and request contract, re-ranked after the probe-at-start answer above. Exact
  approved kernel/module BTF profiles were the alternative the project did not select, and they would have
  disabled clones after a rebuild. Runtime checking can accept more builds but adds a parser and a larger
  compatibility test surface, and that checker has no design yet
  ([Open items the decisions leave](#open-items-the-decisions-leave)). Neither verifier was exercised by
  [the fixture][vm-fixture]; the test plus runtime gate decision stands.
- Lost last event without a waiter. Decided, in the owner's words: "Triggers only, and we control the triggers,
  which makes the con of "unavailable until next trigger" moot." The triggers the design controls are the boot
  unit's start reconcile, the verified checkout provider's post-key-load readiness hook, ZED's delivery of events
  to the installed handler, and the clone-local sweep nudge a caller writes; the answer's reason is that the
  project sets these triggers itself. The unselected periodic scoped sweep would have needed the coalescing and
  rate rule this draft does not have, and triggers alone keep that gap: how the nudges pending for one clone merge
  into a single sweep, and how many sweeps may run in a period across every clone, is unspecified, and nothing
  measured sizes either ([Requests, results and ordering](#requests-results-and-ordering)). Without such a cap a
  repeated nudge can start a sweep that scans every task as often as the account writes the property (reasoning
  from the property event [VM item 5][vm-key] delivered, not measured). A lost event waits for the next trigger the
  project fires, and readiness still needs the provider hook for delayed key and checkout readiness rather than
  another add. [VM item 5][vm-key] delivered one property event to a test zedlet; it did
  not show automatic retry after key load or recovery from a lost last event. Measure the cost of a sweep and the
  lost-event behaviour before choosing any shipped interval.
- Origin cleanup authority. Decided: "Account grant", against the ranking the question round carried, which put
  root-owned cleanup first: descendants-only `mount,destroy` on the checkout. Delegation keeps privilege out of the
  handler but allows the account to destroy any checkout snapshot, sanoid's included. The alternative the project
  did not select, a root-owned cleanup operation, would need its own privileged ownership and recovery proof, which
  the ZED mount
  handler does not imply. [VM item 11][vm-promote] showed why both authorities must
  prove origin identity and lineage before submission; a dependency refusal does not protect other snapshots.
  Delegated origin `destroy` also lets the account mark a checkout snapshot for deferred destroy, so this decision
  is the one that pairs with [clone removal identity protection](#clone-removal-identity-protection); the project
  answered that pair "Keep both, check first (Recommended)", and the check it asked for has no design yet
  ([Open items the decisions leave](#open-items-the-decisions-leave)). Neither of the two
  programs on this host that prune snapshots passes a deferred destroy (`/usr/bin/sanoid:399`,
  `/usr/local/sbin/zfs-space-guard:50`), other callers were not inspected, and that exposure is reasoning from the
  grant model (`lib/libzfs_core/libzfs_core.c:442` to `:451`, OpenZFS `zfs-2.4.4`), not a measured account-run
  deferred destroy ([Clone removal identity protection](#clone-removal-identity-protection)).
- Space-pressure pruning. Decided: "Current scope (Recommended)". The guard keeps its current scope and the route
  reports its limits; the unselected managed-descendant allowlist for sanoid hourlies would have released more
  blocks while deleting some living worktree history before normal expiry. The [VM record][vm-limits] does not
  measure host storage performance or what the guard would reclaim.
- Misplaced attach. Decided: "Roll back at once (Recommended)", immediate rollback of the exact mount by verified
  descriptor, retaining and reporting it only if rollback fails. [VM item 2][vm-refusals] measured successful
  rollback in synchronized trials in a private namespace,
  not free-running interleavings or failed rollback. The unselected alternative, leaving the mount attached and
  reporting both paths and its identity, avoids another privileged move but leaves the mount at an
  account-selected location.
- Busy unmount after a staging move. Decided: "Reattach when safe (Recommended)", against the ranking the question
  round carried, which put staged retention first, and asked as following from the cancellation decision: it
  moves the same mount back to the original slot only after revalidation, and retains it staged otherwise.
  Reattach restores the worktree's visibility but adds a second attach and a race; the unselected alternative
  leaves the worktree unavailable. [VM item 4][vm-unmount] measured busy refusal in staging and successful retry
  after the holder exited, not reattach or interruption recovery. Both routes require staged-state tracking.
- Dirty source on rung 1. Decided: "Verified restore (Recommended)", a verified dirty-main restore comparable to
  Btrfs. The unselected clean-only route would have narrowed the proof but lost clone speed on a dirty main
  checkout, and the main checkout here usually holds other sessions' uncommitted edits. Dirty support needs a
  defined Git/index and ignored-state restore, with a frozen proof, and
  neither route accepts an unproved source. [VM item 1][vm-register] ran `git read-tree HEAD` then reported empty
  tracked status: Git saw the fixture's tracked content matching `HEAD`. It did not capture the source index and
  tree before the snapshot or compare that proof to frozen content. That comparison, conversion checks, dirty
  restoration and concurrent writes remain unmeasured.
- Prune protection while unmounted. Decided: "Git lock + pointers (Recommended)", a managed Git lock in addition
  to preserving the pointer copies. The unselected root-set immutable flag would need removal and
  parent-rename proof and does not protect Git admin state. [VM item 6][vm-prune] measured the lock with an
  absent pointer. Explicit prune removed the unlocked entry immediately; the `git gc` grace period is not a
  safeguard against explicit prune (Git `v2.43.0`, `builtin/worktree.c:244`;
  `git-config(1)`, `gc.worktreePruneExpire`).
  Item 6 did not measure `git gc`, lock recovery after pointer loss, or Git `2.56.1`.
- Git registration when origin cleanup is blocked. Decided: "Settle + own record", against the ranking the
  question round carried, which put keeping the registration first: the registration is settled once the clone is
  destroyed, and a separate durable record carries the origin cleanup. Settling frees the slot but requires
  recovery without its Git entry, and that record has no design or home yet
  ([Open items the decisions leave](#open-items-the-decisions-leave)). [VM item 11][vm-promote] showed a dependency
  blocking a snapshot delete, not either registration policy. If clone lineage itself is unproved, removal refuses
  and keeps the registration.

### Decided defaults

These were the draft's own defaults. The project answered all four on 2026-11-19, so each is now a decision, stated
with the words recorded for it, keeping the section that uses it, its evidence and its own cost. The answer to the
staging default replaced the draft's default with an account-owned place, and what that gives up is stated plainly
below.

- Receipts as atomic files under `/run/git-worktree-clone-mount-results`, rather than root-written
  acknowledgement properties: kept, answering "Keep receipt files (Recommended)"
  ([Requests, results and ordering](#requests-results-and-ordering)). Evidence: reasoning
  from the property event [VM item 5][vm-key] delivered, not a measured receipt. Cost: a receipt for a clone the
  removal destroyed stays with no clone to update it, and nothing in this draft removes that entry; a live clone's
  next result overwrites its own entry, because the filename is its GUID. The directory is root-owned mode `0755`,
  and each receipt is root-owned mode `0644`, read-only to the wrapper, so the caller cannot reap them, and no
  handler step, timer or install action is assigned that role; an acknowledgement property would ride reads the
  handler already makes.
- The mount handler as a second Linux binary inside `package/git-policy/cli`, rather than a package of its own:
  kept, answering "Keep inside cli (Recommended)"; it shares the binding and the ABI gate without adding a package
  ([Binding crate and compatibility](#binding-crate-and-compatibility)). Evidence: none measured; this is a
  placement choice about this repository's packages, not a compatibility result. Cost: the handler's code, tests
  and release move with the wrapper, so a wrapper change can alter what root runs, a wrapper rollback also changes
  the mount handler, and the handler cannot be released or pinned apart from the wrapper; a package of its own
  would version and gate separately at the price of one more package. The two root-owned install paths belong to
  either placement ([ZED reconciliation and mount safety](#zed-reconciliation-and-mount-safety)).
- No implicit remount after a failure or restart: kept, with one exception, answering "Keep, one exception
  (Recommended)". The exception is the checked reattach of a removal that cancels itself without `--force`, which
  the cancellation and busy-unmount decisions otherwise require, because an implicit remount would race the
  standing `unmount` intent and any staged mount ([Cancelling a blocked removal](#cancelling-a-blocked-removal)).
  Evidence: reasoning, not measured. Cost: a worktree whose removal was blocked stays unmounted until that checked
  reattach or an administrator recovers it, and neither restores destroyed snapshots; the exception itself is a
  remount this default otherwise refuses, so it adds the race the default exists to avoid.
- Staging in an account-owned place, decided as "Account-owned place" against the ranking the question round
  carried, and then placed by a further answer: "Under ~/.cache, and make the wrapper clean after itself." The
  draft had proposed a root-owned `0700` bind at `/run/git-worktree-clone-mount-staging`, outside the checkout and
  account-owned paths, so that the mount to be unmounted leaves trees the account controls
  ([Descriptor mount and unmount](#descriptor-mount-and-unmount)). That is the protection this decision gives up: a
  folder under `~/.cache` is one the account can rename, replace or delete, so the account can substitute or
  remove the staging target between checks and moves, and `~/.cache` is itself a recorded substitution exposure
  for a dataset mountpoint inside an account-owned home path
  ([One-time move and checkout boot protection](#one-time-move-and-checkout-boot-protection)). Nothing here
  replaces that protection: the descriptor identity checks still run before each move, and they catch a
  substitution made before them, not one made after. Evidence: the location, its lifecycle and the wrapper's own
  cleanup are untested (unmeasured). Cost: a cleaning step nobody has designed, which has to cover an interrupted
  or failed removal as well ([Open items the decisions leave](#open-items-the-decisions-leave)).

### Clone removal identity protection

The project decided both questions on 2026-11-19: clone destruction is delegated, answering "Delegated destroy
(Recommended)", and the pair with [Origin cleanup authority](#project-decisions), asked again because the grants
couple them, is admitted with a check, answering "Keep both, check first (Recommended)". Just before the head
destroy, removal checks that the clone and its origin are the planned ones and that the origin is not marked for
deferred destroy, refusing otherwise. That check has no design yet, and a same-account mark between the check and
the destroy stays inside the account-versus-itself residual the delegated option already accepts
([Open items the decisions leave](#open-items-the-decisions-leave)). With descendants `mount,destroy` on the
checkout, the account becomes an allowed actor that can mark a checkout snapshot for deferred destroy; the earlier
draft excluded the pair until an origin rule was proved across the cloneable scope, and this answer admits it with
the check in front instead. That exposure is reasoning
from the grant model and from what the `defer` key does
(`lib/libzfs_core/libzfs_core.c:442` to `:451`, OpenZFS `zfs-2.4.4`), not a measured account-run deferred destroy.
Of the two programs that prune snapshots on this host, neither passes a deferred destroy
(`/usr/bin/sanoid:399`, `/usr/local/sbin/zfs-space-guard:50`); other callers were not inspected, so the grant is
the one this design can account for. Neither alternative changes "B: Destroy at once"
([project decision][answers], lines 491 to 494).

[VM item 12][vm-replacement] measured the delegated account destroying an unmounted clone and reusing its name,
including with another clone of the same origin. A fresh GUID read can see a replacement but does not bind a later
name-based request to that GUID (`module/zfs/zfs_ioctl.c:2244` to `:2300` and `:4105` to `:4150`, OpenZFS `zfs-2.4.4`).
The fixture's root channel program refused stale tuples for both replacements and destroyed on a matching tuple;
the account's attempt with a matching tuple was denied. The substitutions were sequenced, not concurrent.

Item 17's delegated arm, under the grants this draft proposes except the checkout's local `snapshot`
([Scoped delegation](#scoped-delegation)), sent one `ZFS_IOC_DESTROY_SNAPS` request naming two of the clone's own
snapshots: it returned 0 and deleted both, a name with a dependent clone returned `EEXIST` with both kept, and root's
control request matched ([VM item 17][vm-17] on OpenZFS `2.4.4-1`), on a promoted clone, whose snapshot list
therefore held the two transferred origin names as well, and both stayed listed under it after the batch.
[Item 18][vm-18] sent the same three requests on an unpromoted clone, the shape removal destroys, and got the same
returns: its own two snapshots destroyed, its snapshot list then empty, and the checkout's three snapshots
unchanged. No guest ran both topologies: the two results agree across guests on the same releases, not
within one guest, and neither measured the binding or the head destroy that follows (unmeasured). Item 18's arm is
the one removal reaches, since removal destroys verified unpromoted clones and refuses a promoted clone before the
first snapshot delete. Beside those measured results stand the two things
the alternative the project did not select still lacks: where root records the origin-side lineage inventory at
add time, since the claim record has a named home in the account-owned Git common directory, the binding record's
home is unnamed, and the handler records slot identity rather than lineage; and how the unprivileged wrapper
reaches a root removal operation, because "authenticate the caller against the configured UID" names no transport
here.

- Scope delegated destruction to protecting foreign history, the decided option. Keep descendant `destroy` and
  require `doctor` and removal preflight to verify the scoped effective grants. Without `promote`, `rename` or
  `receive` authority, the delegated account cannot move foreign history into the removal set (reasoning from the
  grant model and items 11 and 12, not a measured proof of all routes). Require administrators to quiesce
  operations before changing lineage or delegations. This option explicitly accepts the account-versus-itself
  residual: a noncooperating
  process of the same account can substitute its own objects between a check and a named delete. It neither
  prevents that loss nor protects against an administrator who bypasses quiescence.
  The pair's answer, "Keep both, check first", replaces the invariant this paragraph used to require across the
  cloneable scope. Removal now checks, just before the head destroy, that the clone and its origin are the planned
  ones and that the origin is not marked for deferred destroy, and refuses otherwise. A replacement whose different
  origin is marked for deferred destroy is caught by that check, because the clone itself is not the planned one.
  What the check does not catch is a mark made by a same-account process between the check and the destroy, which
  stays the residual this option accepts, and the check itself has no design yet
  ([Open items the decisions leave](#open-items-the-decisions-leave)). This is reasoning from the head destroy
  path (`dsl_destroy.c:1028` to `:1037` and `:1168` to `:1172`), not a measured race or a claim that the grants in
  item 12 already enforce the invariant.
- Use root-run identity-conditional destruction, the alternative the project did not select. That option would
  remove effective account `destroy` for managed clones and their snapshots, which the delegated route does not;
  the requirements the draft stated for it follow, kept as the record of what the project answered over. Propose a
  separate root-installed managed removal operation,
  owned by the wrapper's removal workflow, not the mount handler. Its sole input is a request to remove a recorded
  managed binding, not caller-supplied code, an arbitrary target or an administrator shell. Authenticate the caller
  against the configured UID and verify the binding and target scope against independently root-controlled state.
  Use a typed `ZFS_IOC_CHANNEL_PROGRAM` request (`0x5a48`, OpenZFS `zfs-2.4.4`, `include/sys/fs/zfs.h:1556`), with a
  fixed root-owned program and exact identity tuples. Check the expected clone GUID, origin and `createtxg` and the
  approved history identities in the program, not just before submission. The program excludes concurrent ZFS admin
  changes while it runs (`man/man8/zfs-program.8:35` to `:40`); it does not settle Linux mount or Git races. This
  adds a privileged request and recovery surface. It is also a second request channel beside the mount intent
  property, and a third program beside the account-called wrapper and the clone-mount handler. Before enabling it,
  prove scope against spoofed or replayed requests and same-name replacements. Persist the verified plan under root
  control before mutation;
  after a lost response or restart, prove which recorded identities survive or are gone before resuming, never
  replay by name.
  In the snapshot program, precheck every approved snapshot and its delete prerequisites before any snapshot
  mutation (`module/zfs/dsl_destroy.c:630` to `:653` is the kernel's batch pattern). Prove no-loss refusal and
  per-name results for snapshot precheck failures. Run a separate conditional head request after verifying the
  snapshot outcome: the head check would refuse while own snapshots remain (`dsl_destroy.c:783` to `:790`).
  Check the actual origin's identity and deferred-destroy state inside the head program and refuse a deferred
  origin, so the head request cannot implicitly reap it (proposed from `dsl_destroy.c:1028` to `:1037`).
  Before enabling this option, prove execution and result budgets for a whole snapshot batch and exercise failure
  at those limits. An exhausted program can leave partial effects (`man/man8/zfs-program.8:146` to `:155`);
  neither an error nor a lost response proves no mutation. Item 12 proved only a single conditional clone destroy,
  not the request interface, batch equivalence or recovery. Those are unmeasured design requirements.

### Bookmarks on a clone being removed

The project decided this choice on 2026-11-19, answering "A: Live or planned (Recommended)": the first option
below is the rule removal applies ([Removal and recovery](#removal-and-recovery), step 1), and the other two are
the alternatives it did not select, kept with their facts and costs. A bookmark carries the tuple of the snapshot it
was made from, and a dataset's bookmarks are destroyed with its head
(`module/zfs/dsl_destroy.c:1123` to `:1154`, OpenZFS
`zfs-2.4.4`), so a bookmark on a clone is explained by a snapshot or it is not. The kernel keeps a bookmark whose
snapshot it destroys: for a bookmark carrying `ZBM_FLAG_HAS_FBN`, which an ordinary bookmark on a pool with the
`bookmark_written` feature sets (`module/zfs/dsl_bookmark.c:371` to `:375`, OpenZFS `zfs-2.4.4`), it clears
`ZBM_FLAG_SNAPSHOT_EXISTS` and keeps the entry, and a bookmark without that flag is passed over unchanged
(`:1386` to `:1397`), so a bookmark outlives the snapshot it marks (source reading), and the page calls a bookmark
the point in time its snapshot was created and an incremental source for `zfs send`
(`man/man8/zfs-bookmark.8:48` to `:51`). No option here is measured:
[VM item 13][vm-13] moved a bookmark with an unchanged tuple, and no arm destroyed a bookmarked snapshot or removed
a clone that carried a bookmark.

- Pass a bookmark whose tuple a live passing clone snapshot or a recorded plan tuple explains, and refuse the
  rest; the decided option. Refuse destruction when a bookmark's tuple matches neither, when it matches the
  origin-side inventory, or when the bookmark list could not be read completely, and name each refused bookmark.
  Dataset GUIDs are random
  (`module/zfs/dsl_dataset.c:1203` to `:1204` and `:1768` to `:1769`, OpenZFS `zfs-2.4.4`), so the matching pair
  identifies the bookmarked snapshot of this clone in practice, not against a 64-bit collision. This asks for no
  recorded bookmark: the recorded clone-side inventory is written once at add and the third check forbids updating
  it. Revision `9a1a47e190` held the earlier rule, which required the bookmark's own recorded tuple, and that rule
  would have refused the removal of any clone that someone had bookmarked; revision `b26aa27f3c` held the
  live-only rule, the second option here. Its cost is a bookmark whose snapshot
  was already gone when the plan was made: that refuses before the first delete, the clone, its remaining
  snapshots, the registration and the underlay stay, and every retry refuses the same way, because nothing in this
  design destroys a bookmark. A person destroys it with `zfs destroy <clone>#<bookmark>`
  (`man/man8/zfs-destroy.8:180` to `:182`) and retries; under the delegated option that request checks the
  account's descendant `destroy` grant (`module/zfs/zfs_ioctl.c:1092` to `:1093`); the unselected root alternative
  would leave it to an administrator, since the account holds no destroy authority there (source reading, not a
  measured account run). What this option still needs is stated here rather than settled. The plan is persisted
  under every option ([Removal and recovery](#removal-and-recovery), step 3), so its unnamed home is a gap common
  to all three,
  and its writer follows the identity option: the wrapper runs as the account, so the account writes the plan under
  delegated destruction, and a same-account writer could forge a tuple (reasoning), which is the same-account
  residual that option already accepts ([Clone removal identity protection](#clone-removal-identity-protection));
  under root destruction, root persists the verified plan under root control. An amended plan keeps every earlier
  tuple and its outcome ([Removal and recovery](#removal-and-recovery), step 3), or a later chunk would refuse the
  bookmark of a snapshot an earlier chunk destroyed (reasoning). The plan records each snapshot tuple the removal
  verified and submitted, with that tuple's outcome, and never a bookmark. The comparison relies on an unknown prior
  outcome stopping the attempt before the recheck ([Removal and recovery](#removal-and-recovery), step 3), so an
  unexplained gap is never read as a pass. After a first chunk the stranded state is the partial snapshot set, and
  "planned snapshots all gone with the head refused" needs the whole batch (reasoning). The acceptance control must
  force a split or a replan, because a one-batch removal passes under every option here
  ([Acceptance tests the build still owes](#acceptance-tests-the-build-still-owes)).
- Read live clone snapshots only, the rule this draft used through revision `b26aa27f3c` and the alternative the
  project did not select. It does not read the
  plan for a bookmark and needs no recorded tuple for one, though any option here persists that plan
  ([Removal and recovery](#removal-and-recovery), step 3). It strands a removal after its first mutation: the first
  chunk or the replan destroys the snapshot a bookmark matches, the next lineage proof refuses, and the head, the
  standing `unmount` intent and the remaining snapshots stay while the destroyed ones are gone (reasoning from the
  kernel keeping the bookmark after its snapshot goes, `module/zfs/dsl_bookmark.c:1386` to `:1397`, and this
  option's live-snapshot pass, not measured). After a split that stranded state is the partial snapshot set; after a
  whole batch it is planned snapshots all gone with the head refused.
- Judge by transaction group and the origin side, an option a check of this draft proposed rather than one the
  draft ever stated, and one the project did not select: pass a bookmark whose tuple is ordered after the clone's
  own creation and matches no
  origin-side tuple. It reads no plan for a bookmark, though every option persists one
  ([Removal and recovery](#removal-and-recovery), step 3), and it refuses no stale bookmark. Its basis is a source
  reading: a bookmark's source snapshot must be an earlier point in its dataset's timeline
  (`module/zfs/dsl_bookmark.c:250` to `:256`, OpenZFS `zfs-2.4.4`), and promotion moves bookmarks by creation
  transaction group into the promoted dataset (`module/zfs/dsl_dataset.c:3747` to `:3757`, OpenZFS `zfs-2.4.4`;
  [VM item 13][vm-13] saw a bookmark move with an unchanged tuple). Its limit is the one step 1 states:
  transaction-group order is not proof for other topologies, encryption roots, lineage changes or objects created
  in the same transaction group ([Removal and recovery](#removal-and-recovery), step 1). It is unmeasured.

## Open items the decisions leave

Each item below is a part an answer created or left with no design. This section states what has no design, in plain
words, with the answer's own words beside it, and it designs none of them; nothing in this draft claims one of these
parts works.

- The start probe a kernel-release answer asks for. The answer's words: "Probe at start". What the probe exercises
  (descriptor attach, staging unmount, the namespace refusal), in which namespace and as which user it runs, what
  it changes on the machine at start, what it leaves behind when it fails or is interrupted, and what a refusal
  prints for the caller and `doctor` are all undesigned. Until that exists, the runtime module gate stays the only
  start check this draft describes ([ABI gate and tests](#abi-gate-and-tests)).
- The runtime layout checker. The answer's words: "Runtime layout check". Comparing the loaded module's field layout
  with the approved release and request contract at runtime needs a parser, a set of accepted values, diagnostics
  and tests, and none of them is designed here. The vet's BTF field comparison the draft keeps is a test-time check,
  not this start-time one ([ABI gate and tests](#abi-gate-and-tests)).
- Who ends a holder that is not the account's process, and by what authority. The answer's words: "Kill or
  otherwise remove or destroy the holders whenever --force is specified, auto-cancel if not." Ending a process of
  the account is the account's own act, but a holder can be another account's process, a unit's task the caller
  cannot signal, or a namespace whose task is gone; the design names no actor, no mechanism and no authority for
  those, and a refusal rule would contradict the answer. What such a holder itself sees is the thing the owed VM
  measurement records ([Forced removal and deferred origins](#forced-removal-and-deferred-origins)).
- What a cancelled removal restores. The answer's words: "Kill or otherwise remove or destroy the holders whenever
  --force is specified, auto-cancel if not." Cancellation happens after the order "B: Destroy at once" keeps: the
  clone's own snapshots go first and the head destroy is the step a holder blocks, so a cancelled removal can find
  its own snapshots already destroyed. What the caller gets back is undesigned: whether the clone is reattached as
  it stands, whether the removal record is reversed or kept, and what `doctor` reports. The loss is plain: those
  snapshots do not come back, and the reattach the busy-unmount answer allows restores visibility, not history
  ([Cancelling a blocked removal](#cancelling-a-blocked-removal) and
  [Busy unmount after a staging move](#project-decisions)).
- The check before the head destroy. The pair's answer: "Keep both, check first (Recommended)". Where the check
  reads the planned clone and origin identities, how it reads the origin's deferred-destroy mark, what it prints on
  refusal, and which refusals keep the removal record are undesigned. The window between the check and the destroy,
  which a same-account process can use to mark the origin, is the residual the delegated answer accepts rather than
  a gap this section asks anyone to close
  ([Clone removal identity protection](#clone-removal-identity-protection)). The refusal itself is the second VM
  measurement the build owes ([Forced removal and deferred origins](#forced-removal-and-deferred-origins)).
- The record that carries origin cleanup. The answer's words: "Settle + own record": the Git registration is
  settled once the clone is destroyed and a separate durable record carries the origin cleanup. That record has no
  home, no writer, no fields and no state machine, and nothing says what a settled registration means while the
  record still holds an origin to destroy. The only record with a named home is the per-add claim in the
  account-owned Git common directory ([Removal and recovery](#removal-and-recovery), step 5).
- Staging under `~/.cache` and the wrapper's cleanup. The answer's words: "Under ~/.cache, and make the wrapper
  clean after itself." The folder's creation, the private bind that makes a staged mount safe to unmount there, and
  what the wrapper removes and when are undesigned, including after an interrupted or failed removal. This answer
  gives up the protection the draft proposed, a root-owned `0700` bind outside account-controlled trees, so the
  account can substitute or remove the staging target, and nothing replaces that protection
  ([Decided defaults](#decided-defaults)).
- Which triggers a lost last event waits for. The answer's words: "Triggers only, and we control the triggers, which
  makes the con of "unavailable until next trigger" moot." The triggers this design controls are the boot unit's
  start reconcile in `reconcile-git-worktree-clone-mounts.service`, the verified checkout provider's post-key-load
  readiness hook, ZED's delivery of events to the installed handler, and the clone-local sweep nudge the wrapper
  writes on a caller's behalf. The provider hook is proposed and unrun, and the coalescing and rate rule for nudges
  is unwritten ([Lost last event without a waiter](#project-decisions)); neither is designed here.

## Pre-acceptance VM measurements

The [planning record][answers] requires the descriptor refusals to be measured again with account-owned slots
before design acceptance. The [VM measurement record][measurements] reports that repeat and the other fixture
results, not acceptance of the design. Each item ran on a separate encrypted file-backed pool with OpenZFS
`2.4.4-1` and guest kernel `6.8.0-147-generic`; the Git tests used `2.43.0`. See the [fixture][vm-fixture] for the
read-only host comparison and the [remaining limits][vm-limits]. Items 12 through 15, 17 and 18 repeat selected cases
on the same guest kernel and OpenZFS release; item 16 records the separate exact-kernel image search, and items 17
and 18 ran in guests of their own with the same releases.

- [Item 1][vm-register], measured: register first, snapshot, clone, attach by descriptor and fill in the verified
  Git context. The underlay pointer survived. Moving copied `.git` aside did not measure deletion or recovery.
- [Item 2][vm-refusals], measured: account-owned symlinks were created and refused by the walk, not by permissions.
  The probe rejected escapes, a procfs magic link, bind and ZFS mount crossings, changed inode, UID, mode, and
  access and default ACLs. A valid target attached. The bounded rename trials detected and removed misplaced
  attaches in a private namespace; no renamed-in link or free-running substitution schedule was demonstrated.
- [Item 3][vm-propagation], measured: a shared checkout delivered the later clone to the confined reader but
  blocked staging; making only the checkout private permitted staging and unmount but stopped delivery. The
  reader did not test consumer writes, a read-only worktrees bind, or a different unmount mechanism.
- [Item 4][vm-unmount], measured: a fresh process reopened the attached clone and moved it to staging. A changed
  path and wrong dataset source refused, even with reused mount ID, device and inode in the latter control. A
  busy unmount left the mount staged until retry. Same-name dataset replacement, the decided reattach and crash
  recovery remain unmeasured.
- [Item 5][vm-key], measured: the fixture matched the recorded encryption, record size and compression values,
  not the host topology, feature flags or I/O. Root cloned with the key unloaded; detached mount setup refused,
  a test zedlet deferred, and attach worked after manual key load and checkout mount. Delegated clone, product
  readiness and automatic retry were not measured.
- [Item 6][vm-prune], measured: a Git lock set before pointer loss prevented prune; the unlocked entry was
  pruned. Double force bypassed the lock for move and remove after pointer restoration. No Git `2.56.1` runtime
  check, post-loss lock setup or recovery, or immutable-pointer control was run.
- [Item 7][vm-nosuid], measured: `nosuid` stopped setuid elevation with an unrestricted positive control on the
  same clone. This did not test the built handler's mount flags or device use in that arm.
- [Item 8][vm-immutable], measured: the immutable checkout underlay refused account rename and removal, allowed
  ordinary mounting and survived reboot. Rollback to the pre-flag parent snapshot removed the protection.
  Manual import and key load did not test unattended boot or ancestor replacement.
- [Item 9][vm-nested-boot], measured: a path descendant mounted on the home underlay and was hidden by the
  later legacy checkout mount, not unmounted. Failed ordinary checkout mounting and explicit provider units
  were not exercised.
- [Item 10][vm-legacy], measured: explicit `zfs mount` on the legacy clone refused without attaching; descriptor
  attach on the same clone was the success control. This did not test automated product mounting or boot.
- [Item 11][vm-promote], measured: promotion moved the origin and older snapshots without changing GUIDs.
  Recursive destroy refused, but an explicit delete lost checkout history before its origin's dependency
  refused. The dependent destroy listing was a dry run. Reversal with name conflicts, other encrypted lineage
  shapes or additional dependents, and the wrapper's pre-delete refusal are still unmeasured.

- [Item 12][vm-replacement], measured: the delegated account reused a clone name, including with a new clone of the
  same origin. A root channel program refused stale identities and destroyed a matching clone. Concurrent timing,
  product request authorization and recovery remain unmeasured.
- [Item 13][vm-13], measured: origin, transaction groups and GUIDs separated own and promoted snapshot
  history in the fixture. A bookmark moved and returned on reverse promotion; a snapshot-only inventory missed it.
- [Item 14][vm-batch], measured: the CLI refused the mixed set before an ioctl; the direct batch returned `EEXIST`
  and preserved it. The positive control deleted two own snapshots in one batch. Request limits remain unmeasured.
- [Item 15][vm-namespaces], measured: root found both post-attach namespace holders after private host unmount.
  The shared path control propagated; the shared descriptor move refused. A process-visible scan did not prove
  absence of all mount references.
- [Item 16][vm-kernel], recorded: no built-in or locally supplied VM image booted `7.3.5-1-cachyos` with OpenZFS
  `2.4.4`. The remote catalog was not queried; the fixture did not run on the target kernel.
- [Item 17][vm-17], measured: under item 12's grants the delegated account destroyed two clone-own snapshots in
  one `ZFS_IOC_DESTROY_SNAPS` request on a promoted clone, which returned 0 and left the two transferred origin
  names listed under it; a name with a dependent clone returned `EEXIST` and neither snapshot was deleted; root's
  control request returned 0. A user hold, a held or larger set, request limits, the binding and the head destroy
  that follows remain unmeasured, and that topology is not the one removal destroys.
- [Item 18][vm-18], measured: the same three requests on an unpromoted clone, the shape removal destroys,
  returned the same results: 0 for the account's two clone-own snapshots and for root's control, `EEXIST` with
  both kept for the name with a dependent clone, and the checkout's three snapshots unchanged. A user hold, a held
  or larger set, request limits, the binding and the head destroy that follows remain unmeasured, and the two
  topologies agree across guests on the same releases, not within one guest.

### Open measurements

The delta review's `GH1` is partly resolved. [VM items 2][vm-refusals] and [4][vm-unmount] cover the refusal
repeat with account-owned slots, ACL drift, mount-table checks and descriptor-based busy unmount. Item 2 measured
synchronized substitution and a directly created link, not free-running rename/create during repeated mounts or a
link planted by another route. Keep both gaps in the owner's handoff and the built-handler gates; do not count them
as measured or require a synthetic host-load loop. See "What remains open after the probe" in the
[nested-clone research][nested].

The [VM record][vm-limits] and [delta limits][vm-delta-limits] leave execution on `7.3.5-1-cachyos` and Git
`2.56.1`, the product wrapper, Rust binding and ZED handler, confinement variants and unattended boot and shutdown
reconciliation, free-running races, concurrent same-name replacement, failed rollback or reattach, and recovery
from interruption unmeasured. Measure host storage costs separately from guest semantics. The acceptance tests name
the product controls;
none of the VM observations authorizes host-pool changes.

## Acceptance tests the build still owes

Run with the built wrapper and handler on a disposable VM's file-backed pool, without passing through the host's
pool or `/dev/zfs`. Repeat the [host-shaped encrypted fixture][vm-key] with the built programs, including an
unloaded-key control for the handler and clone fill. Test root installation and then the account with only the
proposed delegation. Distinguish same-dataset reflink from cross-dataset copies with the same or different data
keys. Capture versions, assertions and controls, not just logs. Start from the [VM fixture][vm-fixture] and make
its disposable `mvm` recipe reproducible for the built programs. Retain evidence of guest and pool teardown, not
system changes on the host. These gates carry the [handover's H1 and H2 disposition][handover] and the nested study's
[What I could not show and why][nested-limits].

Qualify `7.3.5-1-cachyos` in a disposable VM, not on the host pool. [VM item 16][vm-kernel] found no offered local
image for the combination. The custom-template candidate needs a bootable x86-64 CachyOS qcow2 with
`linux-cachyos` `7.3.5-1`, matching `linux-cachyos-zfs` or OpenZFS `2.4.4` built for the exact guest headers,
OpenZFS `2.4.4` userspace and an enabled `qemu-guest-agent`. Verify the booted release with
`uname --kernel-release` and the loaded module and userspace with `zfs version`; installed packages alone do not
qualify the run. Repeat item 1's encrypted file-backed fixture and the built mount, unmount, refusal and namespace
controls on that release. For the unselected allowlist, test readiness with a release not on the list as the
negative control, including fallback only for unused slots. These are unrun acceptance requirements, not evidence
from the Ubuntu guest.

### Forced removal and deferred origins

The two decisions of 2026-11-19 that rest on a VM measurement place these two items on the build.

- Forcing the removal of a clone a holder keeps. The cancellation decision's second part is "Measure in a VM
  (Recommended)", with the rule adopted only if it works. Extend [VM item 15][vm-namespaces]'s post-attach readers
  and [VM item 4][vm-unmount]'s busy holder: with `--force`, force-unmount or end the holders in each namespace
  kind and destroy the clone, recording what each holder sees (an error, a vanished mount, stale content, or a
  signal) and whether the destroy then succeeds. If the holder survives or the head destroy still refuses
  `dataset is busy`, the answer has nothing behind it and the rule goes back to the owner rather than shipping.
- The refusal of a removal whose origin is marked for deferred destroy. The paired answer "Keep both, check
  first (Recommended)" asks for a VM item showing the refusal. Mark a checkout snapshot for deferred destroy with
  the `defer` key, point the planned clone (or a same-name replacement) at it, and run removal to the head
  destroy: record that the check refuses before that request, that the origin and the marked snapshot still
  exist, that the removal record and the Git registration survive, and what the caller and `doctor` see. The
  same-account mark made between the check and the destroy is the accepted residual, not a case this item has to
  close ([Clone removal identity protection](#clone-removal-identity-protection)).

### Descriptor safety and mount authority

- H1 gate 1: reproduce the old check-then-path-mount behavior as a positive control, using only a disposable decoy
  inside the VM. A path mount reached a decoy in the research; that did not test a substitution after the check.
  Prove the harness observes attachment outside the intended slot before trusting any no-mount assertion.
- Repeat [VM item 2][vm-refusals] with the handler and account-owned slots. As the unrun alternative planting
  case, rename a pre-existing link into the slot rather than creating it there. Assert every refusal names the
  component or field and leaves existing mounts alone.
- H1 gate 3: exercise repeated, bounded concurrent slot renames and replacements during real guest mounts, with
  schedules not synchronized to the handler's checks. This is an unrun `GH1` control, not synthetic host load.
  Assert no unrelated mount is touched and the post-attach mount-ID check detects an attach on a moved recorded inode.
  Verify the decided rollback rule, the report of a retained mount when rollback fails, and the stranded state.
  Test the post-attach check itself,
  not just the open-time checks. [VM item 2][vm-refusals] measured synchronized rollback in a private namespace,
  not the built handler's behavior under free-running changes or rollback failure. Test staging setup at
  boot and refuse shared or replaced staging and checkout mounts until the decided private setup is verified
  (proposed controls from [item 3][vm-propagation]).
- H1 gate 5 recheck: verify mount flags in `/proc/self/mountinfo` and enforce both `nosuid` and `nodev` in the VM
  even where dataset properties would permit them. Repeat the [VM item 7][vm-nosuid] setuid control and the
  nested research's device refusal (`guest-exact-zfs.log:222` to `:223`) with the built handler.
- H1 gate 6: run unmount from a handler invocation that did not attach the clone. Change a path between resolution
  and unmount; assert no unrelated mount unmounts. Use a real busy holder and exercise move to staging, busy refusal,
  and the decided reattach rule. Test interruption there. [VM item 4][vm-unmount] measured
  an independent busy holder, not reattach or recovery. Repeat the [VM item 3][vm-propagation] shared and private
  checkout controls with the built programs and the decided propagation.
- The forced arm belongs to this gate's busy holder: with `--force`, end the holder and record what it sees, as
  [Forced removal and deferred origins](#forced-removal-and-deferred-origins) states.
- Repeat [VM item 15][vm-namespaces] with the built handler, starting both readers and the confined Pi consumer
  after attach. Prove a host-only detach with another namespace holding the source blocks the first snapshot request
  and that `doctor` names the holders; release them and show normal removal. Keep the shared path unmount as a
  fixture-only propagation control. Exercise task churn, incomplete scans, divergent process roots and pinned
  namespaces without live members; bound and document what the deployment can exclude, and record how the run
  reports the residual the destroy-anyway decision accepts
  ([Removal when the namespace exclusion cannot be shown](#project-decisions)).
- H1 gate 7: exercise a confined user namespace without extra authority. Show that it cannot attach the clone with
  different flags despite OpenZFS's `FS_USERNS_MOUNT` declaration. Exercise a permitted host mount as the control,
  verify late-mount visibility in confined consumers under shared and private checkout propagation (with and
  without the read-only parent bind), and run the same-dataset reflink fallback in the actual confined consumer.

### Boot, shutdown and setup

- Perform an actual reboot and shutdown with idle and busy legacy clones. Prove the decided immutable covered entry
  at the
  unchanged checkout path and verify startup and stop ordering of `reconcile-git-worktree-clone-mounts.service`.
  Include every nested dataset, including a nonclone fixture like the vm-builder output. Delay or fail import,
  key loading and checkout mounting; assert no nested dataset attaches or creates mountpoint folders in the
  home underlay. Check mount parent IDs and visibility after success. For the unselected legacy checkout, verify the
  nonclone mount is excluded from the automatic scan and the explicit unit cannot start before verified checkout
  success. Repeat the decided ordering with real boot units; ordinary clone mountpoints are a negative control,
  not a shipped setup.
- Enable real ZED in the guest and exercise the configured ordering, replay, dropped events with and without a live
  caller, concurrent zedlets, stale receipts, and pending unmount across restart. Measure elapsed stages and use
  readiness/refusal controls with the handler absent, ABI unsupported and ZED stopped. Repeat with a second host's
  sanoid and ZED configuration in isolation. [VM item 5][vm-key] used real ZED with a test zedlet, not the product.
  With no waiter or timer, deliver a key-load event before the checkout mounts and show the readiness refusal.
  Then verify the provider's post-mount hook reconciles the pending clone without another add request (unrun).
- Verify direct checkout/home unmount names held clones, idle export/import with a clone succeeds and recovers, and
  busy export fails without a forced teardown. Record root's exact-version behavior on destroy of a mounted clone
  in the guest; the wrapper must refuse before attempting it regardless of the tool result.
- Exercise migration and rollback on the disposable layout, including linked Git pointers, an independently mounted
  vm-builder output dataset, ignored underlay files and a recoverable backup. Check that `doctor` reports partial
  reads and prints the tested root-once commands for the decided setup, without mutating it. Test parent adoption
  after recreation or rollback, and refuse it with a link, boundary, ACL mismatch or unresolved live/staged binding.
  Verify ordinary fallback on an unused slot and no fallback over a binding. Test which paths the
  actual Kopia job can see, whether it crosses a ZFS boundary, and restoration of the checkout and configuration
  with origins still pinned. An unavailable job is an unknown, not proof of exclusion.

### Binding, Git and lifecycle

- Verify exact ioctl values and Rust layout against the running module's BTF; exercise native nvlist encoding,
  resizing, permissions and property sources. Test both accepted and refused creation-time user properties plus
  post-create `ZFS_IOC_SET_PROP`. With only the proposed grants, verify the clone-local sweep nudge is delivered
  and enough to wake the intended sweep, and that the account cannot set it on the parent (unrun [item 5][vm-key]
  extension). Test malformed, oversized or inherited mount intent and stale results with no
  unauthorized filesystem action. Verify the release wrapper starts without OpenZFS and has no CDDL dependency.
- Drive the full register-first add at exactly `.claude/worktrees/<name>` with descriptor mounting and account-owned
  slots. Repeat [VM item 1][vm-register] on Git `2.56.1`; the guest used `2.43.0`. Recheck empty vs. non-empty
  registration, the
  common directory and index, boundary-safe ignored cleanup, nested path omissions and mounted/unmounted `move`.
  Use ordinary Git and same-dataset reflink controls and a cross-dataset negative control. Cover recognized and
  rejected forms, dirty sources and a tracked write between preflight and snapshot that the frozen proof refuses.
- Kill add at the interruption points in item 7 of the [nested-clone research][nested], and around the origin
  and clone identity writes. Test `ENOSPC` at snapshot and clone; report what survives, resume only proven
  objects and never fall through onto a pending slot.
  With the clone unmounted, verify `prune` keeps the registration with the pointer intact, and exercise the
  decided Git lock with the pointer missing. [VM item 6][vm-prune] did not exercise installing or
  recovering the Git lock after loss; test it, because the project decided that protection. Check that operator
  locks continue
  to refuse removal. Test repair with surviving admin data and already-pruned state, with and without backup.
- Repeat [VM item 11][vm-promote] against the binding's promote primitive. With the built wrapper, refuse a
  promoted or mismatched clone before the first snapshot delete, and assert that every protected snapshot and
  bookmark identity survives. Repeat the `origin`, `createtxg` and transferred-bookmark controls of
  [VM item 13][vm-13], including reverse promotion. Add the own-bookmark controls for the decided bookmark rule:
  a bookmark made from a clone snapshot the plan includes passes and lets the removal finish; a bookmark whose
  snapshot was already destroyed refuses before the first snapshot delete and names the bookmark, which the decided
  first option does; the second option would refuse it too, and the third would pass it (reasoning from each
  option's own statement in
  [Bookmarks on a clone being removed](#bookmarks-on-a-clone-being-removed), not measured); and one control forces
  a split or a replan, by a request budget that splits the set into two chunks or
  by a new own sanoid snapshot injected after the first chunk, with the bookmark on a snapshot the first chunk
  destroyed. A one-batch removal passes under every option, so the split or replan control is the one that tells
  them apart. Include a verified unpromoted clone with manual and sanoid snapshots as the removal control. Test
  proof invalidation after administrator lineage changes. Repeat [VM item 12][vm-replacement] for the decided
  [delegated identity policy](#clone-removal-identity-protection): verify absence of the lineage-changing grants for
  delegated destruction and exercise the declared same-account residual. An ordinary authorized removal must
  destroy the verified clone and its own snapshots and print the loss in the same invocation. Refusal controls test
  unsafe cases; a route that only refuses does not pass, and a refusal that outlasts a holder's own lifetime does
  not replace the forced arm the build now measures
  ([Forced removal and deferred origins](#forced-removal-and-deferred-origins)).
- Repeat the [VM item 14][vm-batch] mixed and own-snapshot batch controls through the binding, including per-name
  errors, GUID preservation and an assertion that the encoded request carries no `defer` key. Run the same batch
  as the delegated account under the planned grants; item 14 ran it as root, and [VM items 17][vm-17] and
  [18][vm-18] measured the account's own-snapshot batch on a promoted and an unpromoted clone in the fixture, not
  through the binding. Exercise the decided delegated route and its supported size limit, a split with a changed
  inventory between chunks, and interruption before and after the separate head destroy. The root alternative the
  project did not select would also need batch-equivalent refusal proved, including bounded program execution;
  item 12 did not test it.
- Kill removal across unmount, snapshot destruction, clone and origin cleanup, and Git and directory cleanup.
  Verify busy and staged mounts, nested datasets, same-name replacements, held snapshots and another clone
  pinning the origin each block the unsafe next action. Inject a new own sanoid snapshot between proof and batch
  and again before head destroy; verify automatic re-proof and reporting without prompting, deadline exhaustion,
  and refusal for unknown or foreign changes. Include a replacement clone whose different origin is marked
  for deferred destroy: the check before the head destroy must refuse it, and the run records what the caller and
  `doctor` see (unrun; the second measurement under
  [Forced removal and deferred origins](#forced-removal-and-deferred-origins)). Check the before-destroy inventory
  and final loss
  report on successful and partial removals, with both manual and sanoid snapshots. Distinguish completed clone
  removal from origin or Git cleanup still owed. Verify safe retries under the actual request and mount state.
  After a post-unmount refusal, test that restart honors the standing intent, `doctor` shows it, and a removal that
  cancels itself without `--force` reattaches only for a surviving, verified binding with known outcomes (unrun).

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
  preparation. Report disposal as a contribution against the same ordinary Git control's total; it has no matching
  stage in that arm. [VM item 1][vm-register] and item 7 of the [nested-clone research][nested] only moved `.git`
  aside, so neither measures deletion. Use measured reconciliation and shutdown workloads to size any
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
  [Project decisions](#project-decisions) and [Boot, shutdown and setup](#boot-shutdown-and-setup) retain
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
  [Project decisions](#project-decisions) settles the lost last event on the triggers the design controls.
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
- L6: [Project decisions](#project-decisions) uses plain list items, not bold-label bullets.

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
  [Pre-acceptance VM measurements](#pre-acceptance-vm-measurements) records the refusal repeat
  under account ownership and its limits.
  [Descriptor safety and mount authority](#descriptor-safety-and-mount-authority) tests the built handler.
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
  refuses moves. [VM item 1][vm-register] measured the combined fixture sequence.
  [Binding, Git and lifecycle](#binding-git-and-lifecycle) tests the built flow and kill points.

[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[vm-register]: cli-git-zfs-measurements-owed-before-acceptance.md#1-register-snapshot-attach-and-fill
[vm-fixture]: cli-git-zfs-measurements-owed-before-acceptance.md#fixture-and-evidence
[vm-refusals]: cli-git-zfs-measurements-owed-before-acceptance.md#2-refusals-with-account-owned-parent-and-slot
[vm-propagation]: cli-git-zfs-measurements-owed-before-acceptance.md#3-shared-propagation-and-a-confined-reader
[vm-unmount]: cli-git-zfs-measurements-owed-before-acceptance.md#4-unmount-from-a-fresh-process
[vm-key]: cli-git-zfs-measurements-owed-before-acceptance.md#5-encrypted-fixture-and-an-unloaded-key
[vm-prune]: cli-git-zfs-measurements-owed-before-acceptance.md#6-deleted-underlay-pointer-prune-and-lock
[vm-nosuid]: cli-git-zfs-measurements-owed-before-acceptance.md#7-setuid-execution-on-the-clone
[vm-immutable]: cli-git-zfs-measurements-owed-before-acceptance.md#8-immutable-underlay-boot-and-parent-rollback
[vm-nested-boot]: cli-git-zfs-measurements-owed-before-acceptance.md#9-automatic-mounts-below-a-legacy-checkout
[vm-legacy]: cli-git-zfs-measurements-owed-before-acceptance.md#10-explicit-zfs-mount-of-a-legacy-clone
[vm-promote]: cli-git-zfs-measurements-owed-before-acceptance.md#11-promote-and-the-checkouts-snapshot-history
[vm-limits]: cli-git-zfs-measurements-owed-before-acceptance.md#what-this-did-not-show
[vm-replacement]: cli-git-zfs-measurements-owed-before-acceptance.md#12-same-name-replacement-with-delegated-grants
[vm-13]: cli-git-zfs-measurements-owed-before-acceptance.md#13-creation-transaction-groups-and-a-bookmark-after-promote
[vm-batch]: cli-git-zfs-measurements-owed-before-acceptance.md#14-one-snapshot-destroy-request-after-promote
[vm-17]: cli-git-zfs-measurements-owed-before-acceptance.md#17-delegated-snapshot-batch-2026-11-19t004401-0500
[vm-18]: cli-git-zfs-measurements-owed-before-acceptance.md#18-delegated-batch-unpromoted-clone-2026-11-19t012534-0500
[vm-namespaces]: cli-git-zfs-measurements-owed-before-acceptance.md#15-mount-namespaces-created-after-attach
[vm-kernel]: cli-git-zfs-measurements-owed-before-acceptance.md#16-availability-of-a-guest-with-the-host-kernel
[vm-delta-limits]: cli-git-zfs-measurements-owed-before-acceptance.md#what-this-repeat-did-not-show
[vm-delta]: cli-git-zfs-measurements-owed-before-acceptance.md#delta-gate-measurements-2026-11-18t230453-0500
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
