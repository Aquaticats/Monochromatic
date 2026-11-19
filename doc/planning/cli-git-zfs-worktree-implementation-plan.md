# cli-git ZFS worktree implementation plan

**Non-final plan,
written by an agent,
unapproved.**
This file puts the accepted design in build order,
so each step can be handed to one writer as one task and reviewed on its own.
It adds no decision of its own:
every statement about the design names the design heading it comes from,
and where the design states no order the entry says the order is this plan's suggestion.
Only the owner declares a plan final or approved,
so this one is neither.

## Status and sources

The design is accepted,
and nothing of the route is built:
there is no `package/zfs/`,
and the wrapper has no ZFS route.
The checkout still lives on the home dataset,
and no pool,
dataset or unit on this host was changed while this plan was written.

- The decision:
  `doc/decision/cli-git-zfs-worktree-design.md`,
  headings "Status",
  "Decision" and "What still gates shipping".
- The whole design,
  its evidence,
  the open items and the tests it owes:
  `doc/planning/cli-git-zfs-worktree-design-proposal.md`.
- The VM items this plan cites by number and heading:
  `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`.
- The execution record,
  including the container gate entry:
  [handover][].
- The interface the steps write into:
  [spec][];
  the wrapper's tasks:
  `package/git-policy/cli/mise.toml`.

The three design files sit on branch `docs/planning-cli-git-zfs-worktree-design-proposal`,
read here at `0ba9932042` (2026-11-19T16:48:07-05:00),
and are not on `main` yet.
Citations name their headings,
never their lines,
because the wording round moves lines without changing a decision.

## How to read a step

Each step is one writer's task and one review unit.
Each step's fields are:

- "What it builds":
  package,
  crate,
  module and binary names as the design gives them.
- "Design sections":
  the design headings it implements.
- "SPEC.md passages":
  what it writes in [spec][],
  naming a suggested heading where the design names none.
- "Acceptance tests":
  the items under "Acceptance tests the build still owes" it must make pass.
- "VM measurements":
  the VM item it repeats,
  and where that run happens.
- "Open items":
  the entries under "Open items the decisions leave" that block it,
  and the ones it proceeds without.
- "Depends on":
  the steps that must land first.

The steps are in build order.
A step whose entry says it is blocked waits for the named design,
and a step marked as proceeding may ship with that gap named in its own output.

Three designs block the route early,
and none of them is code:
the start probe and the runtime layout checker under "Open items the decisions leave",
and the staging design in the entry beginning "Staging under `~/.cache` and the wrapper's cleanup".
No production kernel request runs before the first two,
and no unmount runs before the third.

## Dependencies and their state

Each entry names whether the design picked the dependency,
quoting the line that picks it,
or says which step must pick it.

### Rust crates and libraries

- The binding crate itself.
  Picked by "Binding crate and compatibility",
  in its sentence beginning "Use":
  the path `package/zfs/request-to-openzfs-kernel-device`,
  the Cargo package `monochromatic-zfs-request-to-openzfs-kernel-device`,
  and the workspace name `@monochromatic-dev/zfs-request-to-openzfs-kernel-device`.
  Built by step 1.
- `libc`.
  Picked by "Binding crate and compatibility":
  "Reuse `libc`."
  The catalog already carries it as `libc = ">=0.2.191"` in [rust-catalog][],
  which [catalog][] renders into each crate's `[workspace.dependencies]`.
  Used by steps 1 and 4.
- The reused parts of `Ukko-Ylijumala/zfsdev-rs` at `4559ffc711`:
  codec,
  layout and fuzz code.
  Picked by "Requests and input ownership":
  "Reuse selected codec,
  layout and fuzz code from `Ukko-Ylijumala/zfsdev-rs` at `4559ffc711`",
  "not its vendored CDDL references or live-pool suite".
  It is adapted source with notices,
  not a manifest dependency,
  and step 1 ships the MIT and Apache-2.0 texts and the provenance record the same heading asks for.
- A profile-checking dependency for the runtime ABI checker.
  Not picked:
  "the profile-checking dependency that checker needs is chosen with its design,
  which has none yet" ("Binding crate and compatibility").
  Step 6's design picks it,
  and the dependency belongs to the owner's approval with that design.
- A BTF reader for the layout test.
  Not picked by the design,
  which names only the inspection command,
  `pahole --btf_base /sys/kernel/btf/vmlinux -C zfs_cmd /sys/kernel/btf/zfs` ("ABI gate and tests"),
  and asks for fields to be compared "in a test".
  Step 2 picks it;
  the plan's suggestion is in "Where the design is silent".
- `libfuzzer-sys` for the binding's fuzz sidecar.
  Not named by the design;
  the design asks to "Fuzz the adapted codec" ("ABI gate and tests"),
  and the repo's sidecar harness already uses it (`package/git-policy/cli.fuzz/Cargo.toml`).
  Step 1 picks the sidecar and the crate version through the catalog.
- JSONC parsing for the handler's config file.
  No new dependency:
  the wrapper already links `monochromatic-jsonc-edit`,
  and [cli-cargo][]'s comment calls it "the already adopted comment-preserving JSONC foundation",
  which both native tools use.
  Step 7 adds no JSON crate.
- The wrapper's path dependency on the binding.
  Picked in effect by "Binding crate and compatibility",
  which has both binaries "use the same binding and compatibility gate".
  It is an internal path dependency with a `=` version,
  per [catalog][],
  and step 5 adds it.

### Tools

- `mvm` for every guest.
  Picked by "Acceptance tests the build still owes":
  "make its disposable `mvm` recipe reproducible for the built programs".
  Installed in this repository at `node_modules/.bin/mvm`.
- OpenZFS userspace and module `2.4.4` inside the guest.
  Picked by "Requests and input ownership":
  "Numbers are for OpenZFS `zfs-2.4.4`",
  and by "Descriptor mount and unmount",
  whose demonstrated baseline is guest `6.8.0-147-generic` with OpenZFS `2.4.4`.
  The measurements built it from the release tarball in the guest.
- `pahole` for the layout comparison.
  Picked by "ABI gate and tests",
  in the inspection command the BTF reader entry quotes.
  It is installed on this host and both BTF files are readable here,
  but the comparison the design asks for runs against the guest's module.
- The `zfs` program,
  as root,
  for fixture setup inside the guest only.
  The design forbids it inside the products:
  "Do not add a C build step,
  link or load OpenZFS's CDDL libraries,
  or invoke the `zfs` program from the binding,
  wrapper or handler" ("Binding crate and compatibility").
- `zfs-zed` with a real ZED daemon,
  and systemd,
  inside the guest.
  Picked by "Boot and shutdown unit",
  which installs `reconcile-git-worktree-clone-mounts.service`,
  and the `history_event-mount_git_worktree_clone_datasets` zedlet,
  and by "Enable real ZED in the guest" under "Boot,
  shutdown and setup".
- Git `2.56.1` on the host and Git in the guest.
  Picked by "Binding,
  Git and lifecycle":
  "Repeat [VM item 1] on Git `2.56.1`;
  the guest used `2.43.0`."
- sanoid `2.3.0` in the guest.
  Picked by "Tooling,
  retention and performance":
  "Exercise sanoid 2.3.0".
- `podman` and the wrapper's gate image for the container gate.
  Not picked by the design;
  the repo's gate (`mise run //package/git-policy/cli:native:test:container`) owns it.
- `cargo-nextest` and Clippy for every Rust package's tests and type gate.
  Not picked by the design;
  repo rules `RNQ` and `CM6` own them.
- `file-enforcer` for the new crate's manifest and the catalog rows.
  Not picked by the design;
  repo rule `WC2` owns it,
  and the new crate's path matches the managed manifest patterns in `file-enforcer.config.jsonc`.
- A release path that can carry a second executable.
  Not picked by the design.
  The design decided the second binary ("Binding crate and compatibility";
  "Decided defaults"),
  and the release archive today holds one executable,
  as [cli-cargo][] and `src/native/source_guard/release_bins_tests.rs` both state.
  Step 5 picks the handling;
  the plan's suggestion is in "Where the design is silent".

### Wrapper code the design reuses

These are existing parts of `package/git-policy/cli` the design names,
not new dependencies,
and each step reuses rather than rewrites them.

- The Git registration and back-link check,
  the lock handling,
  the clean proof and the held-directory boundary checks,
  named in "Removal and recovery" and used by steps 10,
  12,
  13 and 14.
- The held-path and mount-fact helpers and the Git index,
  attribute,
  conversion and publication checks,
  named in "Rung 1:
  Snapshot clone" and used by step 10.
- `find_boundary_below` in `package/git-policy/cli/src/native/subvolume/subvolume_remove_walk.rs`,
  named in "Removal and recovery" and used by step 14.
- The read-only filesystem report `package/git-policy/cli/src/native/health/doctor_filesystem.rs`,
  named in "What doctor checks and prints" and extended by step 18.
- The pnpm,
  Cargo and mise repair and the watch handling,
  named in "Dependency repair and watchers" and reused by step 21.
- `REPORT_SCAN_CEILING` in `package/git-policy/cli/src/native/service/wait_ceilings.rs`,
  named in "Waiting and refusal" as a guest trial budget rather than a shipped limit,
  and replaced by step 22's
  measurement.

### What the design forbids

No C build step,
no CDDL link or load,
and no `zfs` program from the binding,
wrapper or handler ("Binding crate and compatibility").
No second request channel for mounting ("Boot and shutdown unit").
No `defer` key in the snapshot batch ("Removal and recovery").
No path-based mount or unmount on the account's tree ("Descriptor mount and unmount").

## Steps in build order

The decision record's "What still gates shipping" holds both owed VM measurements,
and steps 16 and 17 own them.

### Step 1: The binding package, its names and its codec

#### What it builds

`package/zfs/request-to-openzfs-kernel-device` with Cargo package `monochromatic-zfs-request-to-openzfs-kernel-device`,
a standalone Cargo workspace,
a `README.md` explaining `/dev/zfs` and ioctl to a Git user,
a package `mise.toml` mirroring sibling Rust packages,
the `uses/` folder layout,
and `LICENSES/` carrying the MIT and Apache-2.0 texts of the reused code.
Inside it:
the zeroed,
aligned `#[repr(C)]` `zfs_cmd_t` with its union and trailing fields,
the typed operation list of "Requests and input ownership",
checked NUL-terminated names,
fixed-width ABI addresses,
owned nvlist buffers,
bounded decode and resize,
and an error type returning operation,
target,
errno and per-name errors.
Its fuzz sidecar is `package/zfs/request-to-openzfs-kernel-device.fuzz`,
named after the package per rule `SBS`.
`file-enforcer.config.jsonc` gains the rows the new crate and its sidecar need.

#### Design sections

"Binding crate and compatibility";
"Requests and input ownership";
the sentences of "ABI gate and tests" that need no device ("Test the codec,
request packing,
bounded decoding,
and the fail-closed gate without a device.
Fuzz the adapted codec.").

#### SPEC.md passages

None in [spec][],
because the package is its own crate.
Its `README.md` carries what the design asks of it,
including the request table and the per-name results.

#### Acceptance tests

The device-free half of the first bullet under "Binding,
Git and lifecycle":
"exercise native nvlist encoding,
resizing" and the refusal of an encoded request that carries a `defer` key.
The fail-closed gate's unit tests may be written here but cannot pass until step 6.

#### VM measurements

None.
Unit tests and the fuzz targets run on this host,
with no device and no pool.

#### Open items

None.

#### Depends on

Nothing.

### Step 2: Live requests, the layout comparison and the guest fixture

#### What it builds

The live calls for the requests of "Requests and input ownership",
including the promote primitive that only tests may call,
and leaving the channel program the project did not select.
The reproducible guest fixture,
as an `mvm` recipe:
a throwaway Ubuntu guest with a file-backed encrypted pool (`aes-256-gcm`,
raw key,
`recordsize=32K`,
`compression=zstd-3`),
a `legacy`,
`canmount=off` clone parent,
a checkout dataset,
and the grants of "Scoped delegation",
as the measurements' item 12 fixture set them.
The layout comparison test against the running module's BTF,
the cursor and property-source tests,
the output-resize test,
and the per-name error tests.
The account arm runs with only the proposed grants,
as "ABI gate and tests" requires.

#### Design sections

"Requests and input ownership";
"ABI gate and tests";
"VM measurements preceding acceptance",
whose items 12 to 18 are the starting point for the fixture and the arms.

#### SPEC.md passages

The package `README.md`:
the request table,
the errno and per-name result shapes,
and the fixture's own instructions.

#### Acceptance tests

The first bullet under "Binding,
Git and lifecycle" asks for these:
"Verify exact ioctl values and Rust layout against the running module's BTF;
exercise native nvlist encoding,
resizing,
permissions and property sources.
Test both accepted and refused creation-time user properties plus post-create `ZFS_IOC_SET_PROP`".
The binding half of "Repeat the [VM item 14] mixed and own-snapshot batch controls through the binding,
including per-name errors,
GUID preservation and an assertion that the encoded request carries no `defer` key".
The fixture sentence of the "Acceptance tests the build still owes" introduction.

#### VM measurements

Items 12,
13,
14,
17 and 18,
re-run through the crate,
and item 5's clone under an unloaded key.
All of them run inside the throwaway guest;
the host is read only.

#### Open items

None.
The guest kernel stays the design's `6.8.0-147-generic` until step 23 supplies the host kernel.

#### Depends on

Step 1.

### Step 3: ZFS recognition in the reflink rung and the ladder's refusals

#### What it builds

ZFS recognition where `fs_kind.rs` and `worktree_snapshot_source.rs` limit the route today,
so rung 2 recognizes a proven same dataset and refuses a mounted clone destination,
a clone as source for a checkout-dataset slot,
and any cross-dataset case.
No byte copy is recast as a reflink.

#### Design sections

"Ladder and fallback";
"Rung 2:
Same-dataset reflink";
"Rung 3:
Ordinary Git";
"Confined callers".

#### SPEC.md passages

The ladder passage:
rung 2's ZFS recognition and its refusals,
and rung 3 unchanged.
The plan's suggestion keeps rungs 2 and 3 in the existing ladder section rather than a new one.

#### Acceptance tests

The introduction of "Acceptance tests the build still owes" asks:
"Distinguish same-dataset reflink from cross-dataset copies with the same or different data keys".
The second bullet under "Binding,
Git and lifecycle" asks:
"Use ordinary Git and same-dataset reflink controls and a cross-dataset negative control".
The last clause of "H1 gate 7" under "Descriptor safety and mount authority",
which runs the reflink fallback inside the real confined consumer.

#### VM measurements

A guest arm for the reflink and cross-dataset controls on the fixture's datasets.
The host runs only the unit tests of `fs_kind`.

#### Open items

None.

#### Depends on

Nothing,
so it can run beside step 1.

### Step 4: The descriptor attach, its identity checks and its rollback

#### What it builds

A module in the wrapper's library for the attach half of "Descriptor mount and unmount":
the no-crossing `openat2` walk from the pinned checkout,
the recorded slot identity compare (device,
inode,
UID,
mode,
access and default ACLs),
`fsmount` with enforced `nosuid,nodev`,
`move_mount` onto the opened target,
the fresh canonical post-check that finds the mount at the slot,
the immediate verified rollback of a misplaced attach,
and the refusal messages naming the failing component or field.
It also reads and rechecks the slot identity store,
at this path:
`/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`,
and walks the `adopt-worktree-parent` checks,
without taking an adoption decision.
The plan's suggestion names the module `worktree_clone_mount` under `src/native/`.

#### Design sections

"Descriptor mount and unmount" (attach,
identity record,
post-check and rollback);
"Misplaced attach" under "Project decisions";
the mount-flag clause of "Mounts held by other namespaces".

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"The descriptor mount of a ZFS worktree clone"):
the walk,
the recorded identity,
the flags,
the post-check,
the rollback and the refusals.

#### Acceptance tests

Under "Descriptor safety and mount authority":
"Repeat [VM item 2] with the handler and account-owned slots";
"H1 gate 1",
the decoy positive control;
"H1 gate 3",
the repeated bounded renames and the post-attach check;
"H1 gate 5 recheck",
the flags in `mountinfo` and the setuid and device controls.

#### VM measurements

Items 2 and 7 repeated with the built code,
and the decoy control of H1 gate 1.
Every arm runs in the throwaway guest.

#### Open items

None.
Adoption needs administrator quiescence,
which the design states,
not a missing design.

#### Depends on

Nothing,
so it can run beside step 1.

### Step 5: The second executable and what the release ships

#### What it builds

The `[[bin]]` `mount-git-worktree-clone-datasets` in `package/git-policy/cli`,
with mode dispatch for `boot-reconcile`,
`shutdown-unmount` and `adopt-worktree-parent`;
the two install paths `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets`,
and `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`;
and the wrapper's path dependency on the binding.
Its modes refuse until the step that implements each one lands.
It also changes what a second executable costs the release:
the source guard's one-executable test,
`bin/native-release-bins.unit.test.ts`,
the binstall table in [cli-cargo][],
the install task and the README's install text.

#### Design sections

"Binding crate and compatibility" (the second binary and its shared gate);
"ZED reconciliation and mount safety" (the install paths,
root ownership and the config path);
"Decided defaults" (the second binary's cost).

#### SPEC.md passages

`## Single executable and shipped policies` must change,
because the release no longer holds one executable.
The plan's suggestion is in "Where the design is silent".

#### Acceptance tests

None of its own;
the container gate and the release checks cover it,
and the install is exercised by step 19's boot test.

#### VM measurements

None.

#### Open items

None by the design.
The release-archive shape is a gap this plan names,
not an open item.

#### Depends on

Steps 1 and 4.

### Step 6: The start-time compatibility gate and the start probe

#### What it builds

The gate both binaries run before their first ioctl:
module release,
architecture,
endianness,
request contract and the kernel/module BTF layout,
with a missing device,
missing BTF or mismatch disabling raw requests,
including reads,
and no force flag.
The refusal path reports what failed,
keeps registrations and durable recovery state,
and leaves an unused slot to the applicable reflink or ordinary Git route.
Also the start probe the kernel-release answer asks for,
and the runtime layout checker.

#### Design sections

"ABI gate and tests";
"ABI verification" and "Kernel-release readiness" under "Project decisions".

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"The start-time ZFS compatibility gate"),
beside the existing `### Compatibility and degradation`:
what is proved,
what a refusal disables,
what it prints and which route an unused slot takes.

#### Acceptance tests

"The start probe of the kernel-release answer" and "The start-time layout checker of the ABI answer",
both under "Decided behaviours with no test yet".
The gate's own device-free tests from step 1 must pass here too.

#### VM measurements

Two arms in the throwaway guest:
an untested release,
and a module whose field layout differs from the approved one,
each asserting that raw requests are disabled.
The BTF comparison runs against the guest's `/sys/kernel/btf/zfs`.

#### Open items

Blocked by "The start probe a kernel-release answer asks for" and "The runtime layout checker",
which are this step's own designs.
The release,
architecture,
endianness and request-contract checks are decided and can be written beside them.

#### Depends on

Steps 1,
2 and 5.

### Step 7: The intent property, the receipts and the readiness record

#### What it builds

The durable intent format,
exactly `v1:<nonce>:mount` or `v1:<nonce>:unmount` with the 32-hex-digit nonce and the byte bound before parsing;
the clone-local sweep nonce,
which is a nudge and not an instruction;
the receipts under `/run/git-worktree-clone-mount-results`,
named by the clone's GUID,
root-owned `0644` in a root-owned `0755` directory,
opened relative to that directory,
carrying the nonce as data and never in a filename;
the readiness record (boot,
pool,
checkout,
sweep);
the per-slot root claim;
and the wrapper's side that writes intent and reads receipts and readiness.

#### Design sections

"Requests,
results and ordering";
"Waiting and refusal" (what a result and a timeout report);
"Decided defaults" (the receipt decision and its cost);
"Lost last event without a waiter" under "Project decisions".

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"The clone's mount intent,
receipts and readiness"):
the grammar,
the bounds,
the file modes,
the readiness fields and the report on a refusal.

#### Acceptance tests

The first bullet under "Binding,
Git and lifecycle" asks:
"Test malformed,
oversized or inherited mount intent and stale results with no unauthorized filesystem action".
The nudge half of the same bullet:
delivered with only the proposed grants,
and refused on the parent.
The no-waiter readiness refusal in "Enable real ZED in the guest and exercise the configured ordering" under "Boot,
shutdown and setup".

#### VM measurements

Item 5's property event to a test zedlet,
extended to the product handler and the product receipt.
Runs in the throwaway guest.

#### Open items

Proceeds without "Which triggers a lost last event waits for".
The design keeps the coalescing and rate rule unwritten and bounds one sweep's work instead;
step 22 measures the sweep cost before any shipped bound.

#### Depends on

Steps 1,
2 and 5.

### Step 8: The attaching sweep for a clone that carries a mount request

#### What it builds

The sweep:
enumerate the pinned parent's immediate children,
verify clone type,
GUID,
origin and the inherited legacy and safety properties,
derive the slot from one name under the grammar,
parse the intent and ignore inherited intent,
record the slot's first-observation identity before attach,
attach through step 4,
publish the receipt and the readiness,
and refuse malformed intent before any filesystem operation on its behalf.
Before attach it checks the encryption root and key state,
and reports an unavailable key separately from identity or permission failures.

#### Design sections

"ZED reconciliation and mount safety";
"Requests,
results and ordering";
"Descriptor mount and unmount" (the first-observation record and the no-crossing walk);
the private-parent and staging conditions of "Mounts held by other namespaces".

#### SPEC.md passages

The descriptor-mount section:
one sweep's checks,
the first-observation record,
the key refusal and the receipt.

#### Acceptance tests

The unloaded-key control of the introduction of "Acceptance tests the build still owes":
"Repeat the host-shaped encrypted fixture with the built programs,
including an unloaded-key control for the handler and clone fill".
The handler's own refusals from "Repeat [VM item 2] with the handler and account-owned slots".

#### VM measurements

Items 1 and 5 repeated with the handler attached through a real sweep.
Runs in the throwaway guest.

#### Open items

None for attach.

#### Depends on

Steps 2,
4,
5 and 7.

### Step 9: Unmount from a fresh process, the staging folder and reattach

#### What it builds

The unmount half of "Descriptor mount and unmount":
reopen the recorded parent,
open only the final slot with the flags the heading names,
compare the mount ID from `fdinfo`,
device and inode from `fstat`,
and the slot,
parent mount ID,
ZFS root and source from `mountinfo`,
plus the live dataset GUID from the binding;
recheck the canonical lookup before the move;
move to the staging folder under `~/.cache`,
verify the mount there,
release the descriptors that would hold it busy,
and unmount from the held descriptor without force or lazy detach;
the staging folder's creation,
its private bind and the wrapper's cleanup of it;
staged state tracked across interruption;
and the checked reattach of "Busy unmount after a staging move".

#### Design sections

"Descriptor mount and unmount" (the unmount half);
"Mounts held by other namespaces";
"Busy unmount after a staging move" and "No implicit remount after a failure or restart" under "Decided defaults";
"Staging in an account-owned place" under "Decided defaults";
"Checkout propagation policy" for the private parent it verifies.

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"Unmount,
staging and reattach of a ZFS worktree clone").

#### Acceptance tests

Under "Descriptor safety and mount authority":
"H1 gate 6",
the unmount from a handler invocation that did not attach,
and "Repeat [VM item 15] with the built handler",
shared with step 15.
"The wrapper's staging cleanup" under "Decided behaviours with no test yet".

#### VM measurements

Items 3,
4 and 15 repeated with the built code.
Runs in the throwaway guest,
in its boot namespace where a unit must see the mount.

#### Open items

Blocked by "Staging under `~/.cache` and the wrapper's cleanup",
which also owes the interface that unmounts a staged mount from the held descriptor.
Steps 14 and 17 wait on it as well,
so also on that design.

#### Depends on

Steps 2,
4 and 8.

### Step 10: The add route from registration to verified fill

#### What it builds

Add steps 1 to 5 of "Add and Git registration":
the shared `doctor` preflight,
with the handler readiness report and the source-eligibility check;
the per-add claim record in the Git common directory;
the account-owned empty slot and Git's `worktree add --no-checkout` registration,
with the pointer bytes recorded;
the checkout snapshot and fresh clone,
with each GUID and `createtxg` recorded as it is confirmed,
and the `ZFS_IOC_SET_PROP` fallback when creation-time properties refuse;
the wait for the matching mount,
verified in the caller's namespace;
the frozen-content comparison inside the clone,
the omission of the boundaries the design names,
and the discarded copied Git administration;
the pointer install,
the verified fill and the dependency-repair queue;
the durable binding record keyed on repository,
registration and clone GUID;
and the resume states and `ENOSPC` behavior.

#### Design sections

"Add and Git registration";
"Dataset and path layout";
"Rung 1:
Snapshot clone";
"Walks and Git clean";
"Scoped delegation";
"Eligible clone origins" and "Dirty source on rung 1" under "Project decisions".

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"Cheap worktree creation and removal as ZFS clones");
`### The records an add writes for itself` gains the claim and binding records.

#### Acceptance tests

The second bullet under "Binding,
Git and lifecycle" asks to drive the full register-first add at exactly
`.claude/worktrees/<name>`,
with descriptor mounting and account-owned slots,
and to repeat [VM item 1] on Git `2.56.1`.
"Kill add at the interruption points in item 7 of the nested-clone research,
and around the origin and clone identity writes.
Test `ENOSPC` at snapshot and clone" (third bullet).
"Linked-source refusal" under "Decided behaviours with no test yet".
The dirty-source clauses when step 11 lands;
until then a dirty main checkout is refused and falls to rung 3,
which is the plan's suggestion and not the design's order.

#### VM measurements

Item 1 repeated on Git `2.56.1` with the built wrapper,
the kill points,
and `ENOSPC`.
Runs in the throwaway guest.

#### Open items

Blocked in part by "The add-time Git lock" and "The dirty-main restore on rung 1";
the clean-source path can ship without either,
and the lock is step 13.
The binding record's home is a gap this plan names.

#### Depends on

Steps 2,
6 and 8.

### Step 11: The dirty-main restore and the frozen proof

#### What it builds

The defined Git index and ignored-state restore a clone of a dirty main checkout needs,
and the frozen proof that the clone matches the captured source before fill.

#### Design sections

"Rung 1:
Snapshot clone";
"Dirty source on rung 1" under "Project decisions";
"Add and Git registration" steps 4 and 5.

#### SPEC.md passages

The add section:
what a dirty source restores,
what the proof compares,
and the refusal when they differ.

#### Acceptance tests

The second bullet under "Binding,
Git and lifecycle" also exercises a write between preflight and snapshot,
which the frozen proof refuses,
and the dirty-source clauses of the register-first add's own test.

#### VM measurements

The dirty-source arms of item 1's repeat,
with the preflight and snapshot window exercised.
Runs in the throwaway guest.

#### Open items

Blocked by "The dirty-main restore on rung 1".

#### Depends on

Step 10.

### Step 12: Unmounted, pruned and moved worktrees

#### What it builds

The underlay pointer kept for prune,
independent of the clone's own pointer;
the read-only distinction between a known unmounted clone and a pending add or removal;
the report of slot,
dataset and next safe action;
the refusal of `git worktree move` for a bound managed clone,
mounted or not;
and the repair report that says what a damaged link can and cannot restore.

#### Design sections

"Unmounted,
pruned or moved worktrees";
"Prune protection while unmounted" under "Project decisions",
for the pointer half only.

#### SPEC.md passages

The add section and the removal section both carry the states;
`### Doctor` carries the report.

#### Acceptance tests

"With the clone unmounted,
verify `prune` keeps the registration with the pointer intact" (third bullet under "Binding,
Git and lifecycle").

#### VM measurements

Item 6 repeated with the built wrapper,
for the pointer and prune arms.
Runs in the throwaway guest.

#### Open items

Proceeds without "The add-time Git lock" for these states and the move refusal,
which is step 13.

#### Depends on

Step 10.

### Step 13: The managed Git lock at registration

#### What it builds

The managed lock taken at registration and kept through unmounts and restarts,
its reason,
`doctor`'s and removal's distinction of it from an operator or unexplained lock,
and its release only in the authorized registration-cleanup step,
after clone destruction is proven.

#### Design sections

"Unmounted,
pruned or moved worktrees";
"Prune protection while unmounted" under "Project decisions".

#### SPEC.md passages

The add record section and `### Doctor`:
how the lock is written,
how it is told apart,
and when it is released.

#### Acceptance tests

The third bullet under "Binding,
Git and lifecycle" asks:
"exercise the decided Git lock with the pointer missing",
and "Check that operator locks continue to refuse removal".

#### VM measurements

Item 6's lock arms,
extended to installing the lock after pointer loss.
Runs in the throwaway guest.

#### Open items

Blocked by "The add-time Git lock".

#### Depends on

Steps 10 and 12.

### Step 14: Removal: the lineage proof, the destruction plan and the Git settle

#### What it builds

Steps 1,
2,
4 and 5 of "Removal and recovery":
the live pool,
clone and origin GUID proof;
the origin and snapshot `createtxg` checks;
the clone inventory and bookmarks by kind,
name,
GUID,
owning dataset and `createtxg`;
the comparison against the origin-side inventory recorded at add;
the bookmark rule the project decided,
`A: Live or planned`,
with each refused bookmark named before the first delete;
the persisted destruction plan,
with each verified tuple and its outcome;
the refusal before the first delete,
the removal intent and the printed list of names to be lost;
the origin cleanup under the account's descendants-only `mount,destroy`;
and the Git registration and pointer settle after proven clone destruction,
with the separate origin-cleanup record left standing.

#### Design sections

"Removal and recovery" steps 1,
2,
4 and 5;
"Clone removal identity protection";
"Bookmarks on a clone being removed";
"Origin cleanup authority" under "Project decisions";
"Git registration when origin cleanup is blocked" under "Project decisions".

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"Removal of a ZFS worktree clone");
`### Records of a worktree that is removed` gains the removal record and the plan.

#### Acceptance tests

"Repeat [VM item 11] against the binding's promote primitive.
With the built wrapper,
refuse a promoted or mismatched clone before the first snapshot delete,
and assert that every protected snapshot and bookmark identity survives" and the own-bookmark controls,
in the fourth bullet under "Binding,
Git and lifecycle".
"Registration settled while origin cleanup is blocked" under "Decided behaviours with no test yet".
"Check that operator locks continue to refuse removal" (third bullet under "Binding,
Git and lifecycle").
"Check the before-destroy inventory and final loss report on successful and partial removals" (fifth bullet).

#### VM measurements

Items 11,
12 and 13 repeated with the built wrapper.
Runs in the throwaway guest.

#### Open items

Blocked in part by "The record that carries origin cleanup" for the settle-while-blocked arm,
and by step 9's unmount for the order.
The destruction plan's home and the binding record's home are gaps this plan names.

#### Depends on

Steps 2,
8 and 10.

### Step 15: The snapshot batch and the head destroy under delegated destroy

#### What it builds

Step 3 of "Removal and recovery":
the full verified snapshot set in one `ZFS_IOC_DESTROY_SNAPS` request that omits the `defer` key,
with the per-name error list kept;
the skip for an empty set;
the split by largest fitting prefix when the set exceeds the measured budget,
with a fresh proof and fresh all-namespace verification before each chunk;
the head destroy under the delegated identity policy after the batch's known outcome and a fresh enumeration;
the before-destroy inventory against the final loss report;
the resume states the design names,
including planned snapshots all gone with the head refused,
and the re-proof rules for a new own snapshot.

#### Design sections

"Removal and recovery" step 3;
"Clone removal identity protection";
"Scoped delegation";
"Removal when the namespace exclusion cannot be shown" under "Project decisions".

#### SPEC.md passages

The removal section:
the batch,
the split,
the head destroy,
the loss report and the resume states.

#### Acceptance tests

The rest of the fourth bullet under "Binding,
Git and lifecycle":
its requirement that an ordinary authorized removal destroy the verified clone and its own snapshots,
and print the loss in the same invocation,
the split or replan control,
and the declared same-account residual.
The fifth bullet under "Binding,
Git and lifecycle" asks to repeat these through the binding:
the [VM item 14] mixed and own-snapshot batch controls,
and the same batch run as the delegated account under the planned grants.
The interruption arms of "Kill removal across unmount,
snapshot destruction,
clone and origin cleanup,
and Git and directory cleanup" (sixth bullet).

#### VM measurements

Items 14,
17 and 18 repeated through the binding,
plus the request budget the design asks the VM to establish.
Runs in the throwaway guest.

#### Open items

None beyond step 9's staging.

#### Depends on

Steps 9 and 14.

### Step 16: The deferred-origin checks

#### What it builds

The check before the first snapshot delete and again before the head destroy:
the clone and its origin are the planned ones,
and the origin is not marked for deferred destroy.
What each refusal prints,
and whether a refusal keeps the removal record.

#### Design sections

"Clone removal identity protection";
"Origin cleanup authority" under "Project decisions";
"Removal and recovery" step 3.

#### SPEC.md passages

The removal section:
where the checks read the identities and the mark,
and what a refusal prints.

#### Acceptance tests

"The refusal of a removal whose origin is marked for deferred destroy" under "Forced removal and deferred origins".
"Include a replacement clone whose different origin is marked for deferred destroy" (sixth bullet under "Binding,
Git and lifecycle").

#### VM measurements

The owed item itself:
mark a checkout snapshot,
run removal,
then set the mark between the two checks.
Runs in the throwaway guest.

#### Open items

Blocked by "How the deferred-destroy checks are designed" and by that measurement.

#### Depends on

Steps 14 and 15.

### Step 17: Cancellation, the cancel command and the forced arm

#### What it builds

Auto-cancellation of a blocked removal without `--force`;
with `--force`,
the ending of the holders other than the caller and its session,
once the owed measurement shows it works;
the identification of the caller and its session among the holders;
the guarded `cancel-worktree-removal <slot>`,
which proves the slot under the per-slot claim,
settles outstanding destroy requests,
reports every snapshot already destroyed and restores none;
what a cancelled removal restores,
and what a timed-out or killed removal leaves.

#### Design sections

"Cancelling a blocked removal";
"Blocked-removal cancellation" under "Project decisions";
"Busy unmount after a staging move" under "Project decisions".

#### SPEC.md passages

The removal section:
the two arms,
the never-end set,
the cancel command and the reattach.

#### Acceptance tests

"The caller and its session spared when `--force` ends other holders",
"The kept cancel command" and "No holder ended without `--force`",
all under "Decided behaviours with no test yet".
"Forcing the removal of a clone a holder keeps" under "Forced removal and deferred origins".
The forced arm of the busy-holder bullet under "Descriptor safety and mount authority".
The post-unmount restart arm of the sixth bullet under "Binding,
Git and lifecycle".

#### VM measurements

The owed forced-arm measurement,
extending item 15's post-attach readers and item 4's busy holder.
Runs in the throwaway guest.

#### Open items

Blocked by "Who ends a holder,
how a holder is identified,
and by what authority",
"How the wrapper tells its caller's session apart from the other holders",
"What a cancelled removal restores",
"What the kept cancel command proves and does",
"What a timed-out or killed removal leaves besides its manual way back",
and the forced-arm measurement.
The authority over another account's process is the owner's question;
the rest is design work.

#### Depends on

Steps 9 and 15.

### Step 18: `doctor`

#### What it builds

The read-only extension of `doctor_filesystem.rs`:
the `/dev/zfs`,
ABI,
pool,
dataset,
key and property checks;
the checkout mount,
boot protection,
propagation and caller-visibility checks,
and the nested mounts' path reachability;
the recorded parent and slot identities with the failing field or component;
the effective delegation including ancestor grants,
and the absence of the lineage-changing grants;
the handler installation,
config,
root-owned state and result directories,
ZED and service wiring,
latest readiness and unresolved requests;
the staging folder and its live bind's propagation;
the dataset tree against recursive coverage,
the exclusion sections and the guard's scope;
unmounted and staged clones,
interrupt states,
damaged registrations,
each clone's origin and retention;
the holders of a clone by namespace ID and representative PID;
the blocked-removal state and the kept cancel command;
the backup coverage.
It prints the concrete root-once and setup commands without running them,
and reports what has no design yet as unknown.

#### Design sections

"What doctor checks and prints";
the `doctor` sentences of "Add and Git registration";
"Requests,
results and ordering" for the readiness fields.

#### SPEC.md passages

`### Doctor` under `## Management grammar`:
the checks,
the findings' wording and exit status,
and the printed commands.

#### Acceptance tests

The `doctor` clauses of "Exercise migration and rollback on the disposable layout" under "Boot,
shutdown and setup":
"Check that `doctor` reports partial reads and prints the tested root-once commands for the decided setup,
without mutating it",
and that it reports an unreadable or unproven check as unknown.
The doctor clauses of the removal bullets:
it names the holders,
and it reports the deferred-origin check as not yet designed.

#### VM measurements

The `doctor` readings inside the boot,
removal and migration arms.
Runs in the throwaway guest;
on this host `doctor` is read only.

#### Open items

It reports each open item this plan lists as unknown until its step lands,
so it depends on them but blocks none.

#### Depends on

Steps 7,
9 and 14.

### Step 19: The one-time migration, the immutable entry and the boot and shutdown units

#### What it builds

The root-run migration of the checkout onto its own dataset,
with the recoverable backup,
the inventory of linked worktrees,
nested mounts and free space,
the copy that preserves ownership,
permissions,
times,
links,
ACLs and xattrs,
the staged verification,
the mount switch at the same path,
and the tested rollback.
The immutable covered entry of the checkout underlay,
verified before automated attach and after reimport or rollback.
The gate that orders every path descendant after a verified checkout mount.
`reconcile-git-worktree-clone-mounts.service` with its `Type=oneshot`,
`RemainAfterExit=yes` and its ordering after the checkout and key providers,
`ExecStart` running `boot-reconcile` and `ExecStopPost` running `shutdown-unmount`.
The private propagation the unit sets on the verified checkout,
and what re-applies it after a replacement mount.
The provider hook that triggers reconciliation after post-key-load checkout readiness.

#### Design sections

"One-time move and checkout boot protection";
"Boot and shutdown unit";
"Checkout boot mount protection" and "Checkout propagation policy" under "Project decisions";
"Snapshots and setup reporting" for the dataset names and sizes it creates.

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"Boot,
shutdown and the checkout's own dataset");
`### Doctor` gains the migration and boot findings.

#### Acceptance tests

"Perform an actual reboot and shutdown with idle and busy legacy clones",
"Verify direct checkout/home unmount names held clones,
idle export/import with a clone succeeds and recovers",
and "Exercise migration and rollback on the disposable layout",
all under "Boot,
shutdown and setup".
"Re-applying checkout propagation" under "Decided behaviours with no test yet".
The ordering and provider-hook arms of "Enable real ZED in the guest and exercise the configured ordering".

#### VM measurements

Items 8,
9 and 10 repeated with the built units and a real reboot.
Runs in the throwaway guest.

#### Open items

Partly blocked by "What re-applies private propagation after a replacement mount".
The migration itself is a runbook that needs no design beyond the design's own protection decision.

#### Depends on

Steps 5,
7,
8,
9,
12 and 18.

### Step 20: Sanoid coverage, the exclusions and the guard's report

#### What it builds

The `sanoid.conf` sections of "Snapshots and setup reporting",
with `recursive = yes` on home and the named exclusion sections in the order that takes effect,
the `com.sun:auto-snapshot=false` marker on every exclusion,
`doctor`'s comparison of the measured dataset tree against recursive coverage,
with the concrete INI or `zfs set` correction for confirmed drift,
and the report of the space guard's actual scope.

#### Design sections

"Snapshots and setup reporting";
"Space-pressure pruning" under "Project decisions".

#### SPEC.md passages

`### Doctor` for the coverage finding;
a new page under `doc/runbook/` for the host setup (suggested).

#### Acceptance tests

"Exercise sanoid 2.3.0,
including prune behavior not tested by the 2.2.0 guest.
Confirm recursive coverage,
explicit exclusions,
section-order and `recursive = zfs` controls,
and separate retention for checkout underlays,
clone snapshots and pinned origins" (first bullet under "Tooling,
retention and performance").

#### VM measurements

Sanoid arms in the guest on the built layout,
with the 2.2.0 controls kept beside them.
Runs in the throwaway guest.

#### Open items

None.

#### Depends on

Steps 18 and 19.

### Step 21: Dependency repair and watchers after a clone fill

#### What it builds

The wrapper's `pnpm install --offline --frozen-lockfile` repair,
queued when the service answers and inline otherwise;
the Cargo prewarm and workspace-member cleanup with the cross-filesystem lockfile fallback;
the mise launcher repair and the frozen-lockfile reinstall;
the reopening or rescanning of watches on the mounted tree;
and the skip of the separate ignored-file copy only after a successful clone fill.

#### Design sections

"Dependency repair and watchers".

#### SPEC.md passages

`### Package-manager repair of a new worktree`:
the clone route's repair,
the fallbacks it keeps,
and the watch rebinding.

#### Acceptance tests

The second bullet under "Tooling,
retention and performance" asks:
"Test real dependency preparation after add and after migration.
Interleave the same-dataset control with the new checkout and mounted clone using the same lockfile and home store".

#### VM measurements

The pnpm,
Cargo and mise arms and the inotify check in the guest.
Runs in the throwaway guest.

#### Open items

None.

#### Depends on

Steps 10 and 19.

### Step 22: The measurements that size waits and retention

#### What it builds

The mount and unmount budgets from the acceptance workload,
replacing the guest trial budget;
the sweep's measured cost and the lost-event behavior,
before any bound on sweeps per period ships;
the retention and performance comparison of clone add against ordinary Git,
stage by stage;
what expiration,
removal and the current guard can free;
and every shipped limit cited beside its measurement.

#### Design sections

"Waiting and refusal";
"Lost last event without a waiter" under "Project decisions";
"Snapshots and setup reporting";
"Tooling,
retention and performance".

#### SPEC.md passages

The wait section,
and any passage carrying a shipped number,
so each number has its measurement beside it.

#### Acceptance tests

"Compare clone-add and fallback stages against ordinary Git under the same real workload and interleaved control.
Separate Git fill,
mount/request wait,
discarding the copied `.git` and `.claude/worktrees` stubs,
and downstream preparation".
The same bullet asks to use measured reconciliation and shutdown workloads,
to size any shipped deadline or optional scan interval.
The retention half of the first bullet,
and the sweep cost the "Lost last event without a waiter" decision requires.

#### VM measurements

The guest,
with the host-shaped fixture,
beside a host-shaped control arm.
Runs in the throwaway guest;
the storage comparison stays separate from guest semantics,
as "VM measurements preceding acceptance" requires.

#### Open items

Proceeds without "Which triggers a lost last event waits for" and measures before any rate rule ships.

#### Depends on

Steps 9,
15,
19 and 21.

### Step 23: The host-kernel qualification run

#### What it builds

The custom-template guest the design describes:
a bootable x86-64 CachyOS qcow2 with `linux-cachyos` `7.3.5-1`,
its matching module or OpenZFS `2.4.4` built against that guest's headers,
OpenZFS `2.4.4` userspace,
and an enabled `qemu-guest-agent`.
Then item 1's encrypted file-backed fixture and the built mount,
unmount,
refusal and namespace controls on that release,
with `uname --kernel-release` and `zfs version` printed before any pool exists.

#### Design sections

"Acceptance tests the build still owes",
its `7.3.5-1-cachyos` paragraph;
"Kernel-release readiness" under "Project decisions".

#### SPEC.md passages

None;
the run's evidence record goes beside the measurement record it repeats.

#### Acceptance tests

That paragraph itself:
"Qualify `7.3.5-1-cachyos` in a disposable VM,
not on the host pool."

#### VM measurements

The whole step is one,
in the new template guest.
The host is read only,
as item 16 was.

#### Open items

None of the open items blocks it,
but it repeats steps 2,
8 and 15 on the host's release,
so it comes after them.

#### Depends on

Steps 2,
8,
15 and 19.

## Where the tests run

Kernel requests,
the live binding,
the handler,
the clone route,
removal and boot are tested only on a file-backed pool inside a throwaway VM,
never against this host's pool `zpcachyos`,
and never with this host's `/dev/zfs` passed through.
"ABI gate and tests" states the rule,
and "Acceptance tests the build still owes" repeats it.
Host reads are allowed and were used here:
`zfs list`,
`zfs get`,
`/sys/module/zfs/version` and `/sys/kernel/btf/zfs`.

- Steps with unit tests only:
  1, 5.
- Steps with unit tests and a VM arm:
  2,
  3,
  4,
  6,
  7,
  8,
  9,
  10,
  11,
  12,
  13,
  14,
  15,
  16,
  17,
  18,
  19,
  20, 21.
- Steps that are VM runs with no shipped code:
  22, 23.
- Steps that cannot run their VM arm before another design exists:
  6 (the probe and the layout checker),
  9 and the removal steps 14,
  15 and 17 that unmount (the staging design),
  11 (the dirty restore),
  13 (the lock),
  16 (the deferred checks),
  17 (cancellation and the forced arm),
  and 19's propagation item.
- Every step that changes `package/git-policy/cli` also runs the wrapper's container gate,
  which is the gate "Integration" discusses.
- The binding package's steps run that package's own tasks,
  `cargo nextest` and its type gate,
  per rules `RNQ` and `CM6`.

## What can start now

Step 1 can start today.
It has no open item,
it is its own package,
and its checks are that package's own,
so the wrapper's container gate is not in its way.

These can run beside it without touching a file it touches:

- Step 3,
  the ladder's ZFS recognition,
  in `package/git-policy/cli/src/native/subvolume/fs_kind.rs`,
  `package/git-policy/cli/src/native/worktree/worktree_snapshot_source.rs`,
  and the SPEC ladder passage.
- Step 4,
  the descriptor attach module,
  in a new folder under `package/git-policy/cli/src/native/`.
- The guest recipe inside step 2,
  which step 1's live calls then use.
- The three blocking designs,
  as documents:
  the start probe,
  the runtime layout checker and staging under `~/.cache`.
  They are the critical path for steps 6 and 9,
  and nothing else in the plan unblocks them.

Steps 3,
4 and 5 can be written and reviewed now,
but their merge waits for the container gate,
because they change the wrapper.

## Integration

Every wrapper change reaches `main` through an integration pass.
The wrapper's container gate cannot pass on this ZFS host yet.
The entry in [handover][] opens "The wrapper's container gate cannot pass on this ZFS host",
and records the last run:
`9189 tests run: 9002 passed, 187 failed`.
Searching `container gate cannot pass` finds it,
together with the Clippy step's `E0514` and the investigation that runs on it.
Until that investigation lands a gate result,
no wrapper integration pass can show one.

- Steps that need the container gate:
  3,
  4,
  5,
  6,
  7,
  8,
  9,
  10,
  11,
  12,
  13,
  14,
  15,
  16,
  17,
  18,
  19,
  20 and 21,
  because each changes `package/git-policy/cli`.
- Steps that merge on their own package's checks:
  1 and 2.
  The binding package carries its own `mise.toml`,
  tests and type gate,
  and the wrapper does not link it until step 5.
- Steps that ship no package code:
  22 and 23.
  They produce measurements and reports.
- A step that lands while the gate is out still needs its integration pass at merge time,
  so the wrapper steps queue behind that repair.
  The plan cannot schedule the repair;
  the handover holds it.

Step 5 is a special case:
it adds the wrapper's dependency on the binding,
so it needs both the binding package's release and the wrapper gate,
and its release-archive change reaches the release workflow,
which is not the gate.

## Open items and the step that needs each

Each entry names the open item as "Open items the decisions leave" opens it,
the step that needs its design,
whether it blocks that step,
and whether the answer is the owner's or a writer's design work.

- "The start probe a kernel-release answer asks for":
  step 6,
  blocking,
  writer design work.
- "The runtime layout checker":
  step 6,
  blocking,
  writer design work;
  its design picks the profile dependency,
  which goes to the owner with that design.
- "Who ends a holder,
  how a holder is identified,
  and by what authority":
  step 17,
  blocking,
  mixed.
  Identification and the receiving path are writer work;
  by what authority the wrapper ends another account's process or a unit's task is the owner's question.
- "How the wrapper tells its caller's session apart from the other holders":
  step 17,
  blocking,
  writer design work.
- "What a cancelled removal restores":
  step 17,
  blocking,
  the owner's question,
  with a writer's options and their costs,
  because it fixes what a caller gets back after history is lost.
- "What the kept cancel command proves and does":
  step 17,
  blocking,
  writer design work.
- "What a timed-out or killed removal leaves besides its manual way back":
  step 17,
  blocking,
  writer design work,
  with the restored state put with the cancelled-removal question.
- "How the deferred-destroy checks are designed":
  step 16,
  blocking,
  writer design work.
- "The record that carries origin cleanup":
  steps 14 and 17,
  blocking for the settle-while-blocked acceptance item,
  writer design work,
  with the owner asked only if its home changes the settle decision.
- "The add-time Git lock":
  step 13,
  blocking,
  writer design work.
- "The dirty-main restore on rung 1":
  step 11,
  blocking,
  writer design work.
- "Staging under `~/.cache` and the wrapper's cleanup":
  step 9,
  blocking for every unmount,
  writer design work.
- "What re-applies private propagation after a replacement mount":
  step 19,
  blocking for its acceptance item,
  writer design work.
- "Which triggers a lost last event waits for":
  steps 7 and 22,
  not blocking either,
  writer design work;
  the rate rule waits for step 22's measurement.

No open item needs a design before step 1.

## Gaps the design's open-item list does not name

Four things the design leaves unstated are not in "Open items the decisions leave",
and each blocks a step listed here.

- Where the durable binding record lives.
  "Add and Git registration" says "This design does not name where that binding and lineage record lives:
  only the per-add claim record has a named home,
  the account-owned Git common directory".
  Steps 10,
  14 and 18 write or read it.
- Where the persisted destruction plan lives.
  "Bookmarks on a clone being removed" says the plan is persisted under every option,
  and calls its unnamed home "a gap common to all three".
  Steps 14 and 15 write and read it.
- How the release carries two executables.
  `## Single executable and shipped policies` in [spec][] states that the release artifact is one executable,
  while "Binding crate and compatibility" and "Decided defaults" keep a second binary in the same package.
  Step 5 changes this.
  If the artifact shape is the owner's choice,
  it is a question before step 5 merges.
- What removes a receipt for a clone that no longer exists.
  "Requests,
  results and ordering" and "Decided defaults" say nothing removes it.
  No step deletes it without a decision.

## Where the design is silent: this plan's suggestions

Each entry says what the design states and what this plan suggests instead,
and no entry changes a decision.

- The order of the three blocking designs.
  The design states no order.
  The plan suggests writing the start probe,
  the layout checker and the staging design first,
  beside steps 1 to 5,
  because steps 6 and 9 wait on them.
- The order of the add route's two halves.
  The design decided dirty support but states no order.
  The plan suggests shipping the clean-source route first,
  refusing a dirty main checkout to rung 3 until the restore lands.
- Where the ZFS interface sits in [spec][].
  The design names no heading.
  The plan suggests one new section per area under `## Lifecycle`,
  with rungs 2 and 3 kept in the existing ladder section and its heading's list gaining ZFS,
  and `### Doctor` extended in place.
- How the release carries the second executable.
  The design states the binary and the install paths,
  not the archive.
  The plan suggests one archive carrying both,
  with the source guard,
  the binstall table and the install task changed in the same commit.
- Where the binding record and the destruction plan live.
  The design names only the claim record's home.
  The plan suggests the account-owned Git common directory,
  beside that claim,
  for both.
- Where the guest recipe and its wrapper arms live.
  The design names no path.
  The plan suggests the binding package's `e2e/` folder for the pool and request probes,
  and the wrapper's `e2e/` folder for the wrapper-level arms against the same guest image.
- When a wait number may ship.
  The design says the wrapper's 5 s `REPORT_SCAN_CEILING` is "a guest trial budget,
  not a ZED latency finding or a shipped limit" ("Waiting and refusal").
  The plan suggests no shipped number before step 22,
  and the existing configured ceiling named as provisional until then.
- Which BTF reader the layout test uses.
  The design names the `pahole` command and asks for a test.
  The plan suggests running `pahole` in the guest and comparing its output in the test,
  rather than adding a Rust BTF parser.
- The module and mode names inside the wrapper.
  The design names the binary,
  the modes and the installed paths,
  not the modules.
  The plan suggests `worktree_clone_mount` for step 4's module and one sibling module per handler mode.
- What to do with a stale receipt.
  The design leaves it standing.
  The plan suggests `doctor` reports it and nothing deletes it,
  since a deletion step would need the decision the design does not make.

## What this plan did not do

- It ran no ZFS command on this host,
  read only from the pool and the module's version,
  and changed no pool,
  dataset,
  mount or unit.
- It ran no guest.
  No acceptance test here is a result,
  and no measurement in it was repeated.
- It wrote none of the open items' designs and made none of their choices.
- It read no OpenZFS,
  Git or systemd source,
  so every claim about them is the design's,
  cited to the design's heading,
  not re-verified here.
- It read the design at the branch tip `0ba9932042` and once more before its last commit;
  the wording round changed one heading ("VM measurements preceding acceptance") and no decision.
- It contains no timing claim and no forecast.

## Links

[handover]: ../handover/cli-git-rust-implementation.md
[spec]: ../../package/git-policy/cli/SPEC.md
[catalog]: ../decision/rust-dependency-catalog.md
[rust-catalog]: ../../rust-catalog.toml
[cli-cargo]: ../../package/git-policy/cli/Cargo.toml
