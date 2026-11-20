# cli-git ZFS worktree implementation plan

**Accepted by the owner on 2026-11-19T17:45:53-05:00,
answering "Accept (Recommended)" in the question tool,
at commit `9137efc5b4`.**
Written by an agent and revised after a Sonnet gate review
(32 findings, all applied in `55cab4075e`).
This file puts the accepted design in build order,
so each step can be handed to one writer as one task and reviewed on its own.
It adds no decision of its own:
every statement about the design names the design heading it comes from,
and where the design states no order the entry says the order is this plan's suggestion.
A change to the order is a plan edit the owner sees.

## Status and sources

The design is accepted,
and step 1 alone is built,
as of its merge on 2026-11-19:
`package/zfs/request-to-openzfs-kernel-device` and its fuzz sidecar,
released only as the alpha `0.1.0-alpha.1`,
which never goes to crates.io.
Its merge checks passed in its container,
except that the repository linter skipped its Dylint rule `rust/avoidable-closure`,
as it does on every Rust package here while `cargo dylint` is not installed.
The wrapper has no ZFS route.
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

The three design files are on `main` since `38ed22c5ea`,
which merged branch `docs/planning-cli-git-zfs-worktree-design-proposal`
at `0ba9932042` (2026-11-19T16:48:07-05:00).
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
- "How it merges":
  the checks and installs its merge waits on,
  stated only where the plan knows them.

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
- The guest's contents, and how the built programs reach it.
  Not picked by the design.
  The measurements' guests were Ubuntu 24.04 with Git `2.43.0`,
  OpenZFS `2.4.4` built from the release tarball inside the guest,
  and their controls name `pahole` and `strace`.
  The design asks for Git `2.56.1` in the guest ("Binding, Git and lifecycle")
  and `pahole` there ("ABI gate and tests"),
  and says nothing about whether Rust builds inside the guest
  or host-built binaries are pushed to it and run against its libc.
  Step 2's recipe fixes it,
  and the plan's suggestion is in "Where the design is silent".
- `podman` and the wrapper's gate image for the container gate.
  Not picked by the design;
  the repo's gate (`mise run //package/git-policy/cli:native:test:container`) owns it.
- `cargo-nextest` and Clippy for every Rust package's tests and type gate.
  Not picked by the design;
  repo rule `RNQ` owns the tests,
  and the sibling Rust crate `package/jsonc-edit/core-rust` defines the type gate
  its `mise.toml` runs (`cargo check` plus
  `cargo clippy --release --all-targets -- -D warnings`).
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
- `REPORT_SCAN_CEILING` in `package/git-policy/cli/src/native/service/wait_ceilings.rs`.
  It is the bound on one read of the health report's own filesystem facts,
  already measured on this host on 2026-11-09,
  and no step replaces it.
  "Waiting and refusal" cites it as a guest trial budget;
  no ZFS mount or unmount wait constant exists yet.
  Step 9 introduces the first (the unmount deadline) and step 10 the add's mount wait,
  and step 22 sizes both.
  The design's citation of this constant is its owner's to correct,
  as "Where the design is silent" records.
- `openat2` with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS`, `RESOLVE_NO_XDEV` and
  `RESOLVE_NO_MAGICLINKS` in
  `package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
  which "Descriptor mount and unmount" names as the base of the no-crossing walk.
  Step 4 extends it for attach;
  the final-hop flag difference unmount needs is step 9's.

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
The release registry `package/crate-release/cli/releasable-crates.json` gains the crate's entry,
with its `crateDir`,
`scope`,
`checkBins`,
`pinFiles` and `lockDirs`,
and the sidecar's lock file is recorded by the package task that pins it,
as the sibling Rust packages do.
The crate is also wired into both release workflows and versioned `0.1.0-alpha.1`,
so the merge uploads nothing to crates.io,
by the decision of 2026-11-19T20:03:36-05:00 ("Wire it as an alpha").

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
resizing".
The absence assertion of the fifth bullet in the same section:
"an assertion that the encoded request carries no `defer` key".
This step asserts it on the encoder;
the wrapper's own request asserts it again in step 16.
The fail-closed gate's device-free tests are step 6's,
not this step's.

#### VM measurements

None of its own.
Unit tests run on this host with no device and no pool,
and the fuzz targets run through the sidecar's bounded smoke task in a capped scope,
not as an unbounded host run (rules `RXI` and `NLG`).

#### Open items

None of the design's.
One of this plan's own:
whether the CachyOS image carries `cargo-fuzz`,
and so whether the container needs `cargo-binstall` first.
"How it merges" names both routes;
no probe here checked the image's packages.

#### Depends on

Nothing.

#### How it merges

Step 1's checks all run in a container,
and nothing is installed on this host.
That is narrower than the owner's answer of 2026-11-19,
"Container, narrow install (Recommended)",
which allowed a narrow host install;
the plan adopts the narrower route,
the coordinator records it as adopted and open to the owner's veto,
and a one-tool host install would not have helped anyway:
`package/mpm-shadow/cli/README.md`'s entry
"`mpm-shadow install [<tool>...]`" says that with a tool it
"then installs every locked tool that is not at its locked version",
so naming one tool still converges the whole toolchain.

The container is built from the wrapper's gate base image,
`package/git-policy/cli/native-test-base.Containerfile`,
which takes Rust and `cargo-nextest` from pacman,
so the tests (rule `RNQ`) and Clippy run there.
That base carries the pinned browserslist data that `file-enforcer`'s `browserslist` rule reads
(`package/git-policy/cli/README.md`, "Container base images"),
so step 1's `file-enforcer write` and `check` need no home mount;
`mise run //package/git-policy/cli:test:git-image` is how step 1 gets that base.
Inside it,
`file-enforcer` (`cargo:monochromatic-file-enforcer-cli` `0.1.3`)
and the repository linter (`cargo:monochromatic-lint` `0.5.2-alpha.8`)
come from their `toolchain.lock.json` rows' `linux-x64` archive and sha256,
which is how rule `CM7` has a container install a locked tool.
`cargo:cargo-fuzz` `0.13.2` records route `cargo-binstall` and carries no archive of its own,
so the step names how the container gets it:
pacman's package if the CachyOS image carries one,
which the step checks first,
otherwise install `cargo-binstall` `1.27.0` from its lock row's `linux-x64` archive and sha256
and run it to install `cargo-fuzz` `0.13.2`,
the version and route the lock records.
The container never builds `cargo-fuzz` from source.
Its row stays in `toolchain.lock.json` for the host install later,
which still waits for the mpm-shadow change the owner put first on 2026-11-17
in [handover][]'s new-system section,
answering "A: mpm-shadow change first (Recommended)".

The step merges when its tests and Clippy pass in that container,
`file-enforcer write` and `check` pass there,
the bounded fuzz smoke passes,
and the repository linter reports no finding.

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
as the measurements' item 12 fixture set them
(a note added at step 2's merge `1ead5ca9fa` of 2026-11-20:
that fixture left out the checkout's local `snapshot` and its descendent `mount,destroy`,
so step 2 sets the whole "Scoped delegation" list,
which the acceptance text's "only the proposed grants" names).
The recipe also says what the guest carries and how the built programs reach it:
the Git version,
OpenZFS userspace,
`pahole` and `strace`,
and whether Rust builds inside the guest
or host-built binaries are pushed and run against its libc.
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
including per-name errors and GUID preservation".
The absence of a `defer` key is step 1's encoder assertion,
not a test here.
The fixture sentence of the "Acceptance tests the build still owes" introduction.

#### VM measurements

Items 12,
13,
14,
17 and 18,
re-run through the crate,
item 11's promote controls through the crate's promote primitive,
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
The plan's suggestion is that the proof reads the mount source and the device only,
with no call into the binding;
if it needs `ZFS_IOC_OBJSET_STATS`,
the step also depends on steps 1,
2,
5 and 6.

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
"H1 gate 7" under "Descriptor safety and mount authority" in full:
"exercise a confined user namespace without extra authority",
show "that it cannot attach the clone with different flags despite OpenZFS's `FS_USERNS_MOUNT` declaration",
"exercise a permitted host mount as the control",
verify "late-mount visibility in confined consumers under shared and private checkout propagation
(with and without the read-only parent bind)",
"and run the same-dataset reflink fallback in the actual confined consumer".

#### VM measurements

A guest arm for the reflink and cross-dataset controls on the fixture's datasets.
The host half runs the `fs_kind` recognition,
the clone-domain decision,
the ladder's choice and one real read of this host's mounts;
only the reflink and cross-dataset controls need the guest.
The guest arm owes this list,
recorded here because nothing else in the repository carried it:

- Four binary-level controls beside the existing btrfs clone controls: a same-dataset add serves rung 2, asserting
  `clones served` and FIEMAP-shared extents as `binary_worktree_clone_tests.rs` does; a destination on the second
  dataset is refused and Git fills the worktree; a slot a mounted clone occupies is refused; a mounted clone used as
  the source of a checkout-dataset slot is refused.
- Two arms on data keys: a same-key cross-dataset `copy_file_range` that shares blocks must not qualify, and a
  different-key pair likewise, since the domain decision never reads the keys.
- A pool with block cloning enabled, and a second arm with it off, to prove the rung ends in Git's checkout rather
  than a copy of the bytes.
- A `ZFS` gate helper beside `on_btrfs_or_acknowledged` for `CLI_GIT_ZFS_ABSENT_ACKNOWLEDGED`, or a guest run with
  no acknowledgement, so `zfs_a_real_mount_answers_with_its_kind` is a real control there.

The host half is measured against a release build of `main` on this ZFS host, in throwaway repositories of this
repository's 24,313 tracked files, the arms interleaved and alternating:
a clean source with its data written out took 0.51 to 1.57 times as long as Git's own checkout, median 0.66 over
eighteen interleaved rounds in four runs, once the clones ran in slices,
where one thread took 1.76 to 2.23 times as long.
A source with uncommitted changes still pays the source checks and then ends in Git's own checkout:
0.88 to 1.59 times the same build of `main`, median 1.17 in one run and 1.25 in another, about 0.2 seconds.
That cost is the check that rung 2 requires and no snapshot can carry out here; the measurements, the options and
their costs are with the integration pass that found it.

#### Open items

None.

#### Depends on

Step 2,
for the guest recipe its VM arm needs.
Its host half needs no other step.

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
records the `.claude/worktrees` parent identity the attach compares against,
and takes and records the administrator adoption decision
its `adopt-worktree-parent` checks decide.
The setup run that writes the parent record is step 19's install and boot unit;
the mode that runs the decision is step 5's dispatch.
The attach code extends the walk `fs_beneath.rs` already implements.
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
the repeated bounded renames,
the post-attach check,
and its closing clause:
"Test staging setup at boot and refuse shared or replaced staging and checkout mounts
until a private setup is verified";
"H1 gate 5 recheck",
the flags in `mountinfo` and the setuid and device controls;
"Test parent adoption after recreation or rollback, and refuse it with a link, boundary, ACL mismatch
or unresolved live/staged binding" (fourth bullet under "Boot, shutdown and setup"),
whose decision table this step tests and whose end-to-end arm step 5's mode runs.

#### VM measurements

Items 2 and 7 repeated with the built code,
and the decoy control of H1 gate 1.
Every arm runs in the throwaway guest.
Until step 5's binary exists,
the arms run through a root-run integration test in the wrapper crate that calls the module directly,
not as part of the account-side suite;
that driver is the plan's suggestion.

#### Open items

None of the design's.
Adoption needs administrator quiescence and the recorded parent identity,
both built here;
its end-to-end arm waits for step 5's mode.

#### Depends on

Step 2,
for the guest recipe its VM arms need;
its host half needs no other step.

### Step 5: The second executable and what the release ships

#### What it builds

The `[[bin]]` `mount-git-worktree-clone-datasets` in `package/git-policy/cli`,
with mode dispatch for `boot-reconcile`,
`shutdown-unmount` and `adopt-worktree-parent`;
the two install paths `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets`,
and `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`;
and the wrapper's path dependency on the binding.
The default invocation the zedlet makes,
with no argument,
runs the sweep;
the plan's suggestion is in "Where the design is silent".
Its modes refuse until the step that implements each one lands.
It also changes what a second executable costs the release:
the source guard's one-executable test,
`bin/native-release-bins.unit.test.ts`,
the binstall table in [cli-cargo][],
the wrapper's `checkBins` and `pinFiles` rows in
`package/crate-release/cli/releasable-crates.json`,
the lock's `bins` row in `toolchain.lock.json`,
a sidecar lock refresh (`mise run //package/git-policy/cli:lint:sidecar-locks`),
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

"Verify the release wrapper starts without OpenZFS and has no CDDL dependency"
(first bullet under "Binding, Git and lifecycle").
The end-to-end arm of
"Test parent adoption after recreation or rollback, and refuse it with a link, boundary, ACL mismatch
or unresolved live/staged binding",
through the mode this step wires.
Otherwise the container gate and the release checks cover it,
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
The gate's device-free tests live here:
the request packing they build on comes from step 1's package,
and the gate's decisions are this step's own tests.

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
and the wrapper's side that writes intent and reads receipts and readiness,
including the mount-request deadline it spends waiting for a receipt.
That deadline is the first ZFS wait constant the route has;
step 22 sizes it.

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
2,
5 and 6.

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
5,
6 and 7.

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
It also builds the root handler's all-namespace holder scan
of "Mounts held by other namespaces":
group live tasks by `/proc/<pid>/ns/mnt`,
scan the matching `mountinfo` for each distinct namespace and its ZFS filesystem and exact dataset source,
exclude new managed attaches with the standing `unmount` intent,
and return holders and incomplete reads in a fresh, nonce- and GUID-bound result.
Step 16 consumes it before each snapshot-destroy request and before the head destroy,
and step 18 reports its holders.

#### Design sections

"Descriptor mount and unmount" (the unmount half);
"Mounts held by other namespaces";
"Busy unmount after a staging move" under "Project decisions"
and "No implicit remount after a failure or restart" under "Decided defaults";
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
and "Repeat [VM item 15] with the built handler" in full:
start "both readers and the confined Pi consumer after attach",
"Prove a host-only detach with another namespace holding the source blocks the first snapshot request
and that `doctor` names the holders; release them and show normal removal",
"Exercise task churn, incomplete scans, divergent process roots and pinned
namespaces without live members; bound and document what the deployment can exclude, and record how the run
reports the residual the destroy-anyway decision accepts".
The same bullet's removal half is step 16's.
"The wrapper's staging cleanup" under "Decided behaviours with no test yet".

#### VM measurements

Items 3,
4 and 15 repeated with the built code.
Runs in the throwaway guest,
in its boot namespace where a unit must see the mount.

#### Open items

Blocked by "Staging under `~/.cache` and the wrapper's cleanup",
which also owes the interface that unmounts a staged mount from the held descriptor.
Steps 14,
16 and 17 wait on it as well,
so also on that design.
Open item 3's receiving path,
how root's finding reaches the unprivileged wrapper,
lands in step 17;
step 16 and step 18 read the result this step records.

#### Depends on

Steps 2,
4,
6 and 8.

### Step 10: The add route from registration to verified fill

#### What it builds

Add steps 1 to 5 of "Add and Git registration":
the shared `doctor` preflight,
with the handler readiness report and the source-eligibility check;
the per-add claim record in the Git common directory;
the account-owned empty slot and Git's `worktree add --no-checkout` registration,
with the pointer bytes recorded;
the managed Git lock taken here, with that registration,
with its recorded reason and its pointer bytes;
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
"Cover recognized and rejected forms, dirty sources and a tracked write between preflight and snapshot
that the frozen proof refuses" (second bullet under "Binding, Git and lifecycle"),
for the recognized and rejected forms here,
and "boundary-safe ignored cleanup, nested path omissions" from the same bullet.
The owner decided the interim on 2026-11-19,
answering "Ship step 10 first (Recommended)":
while step 11 is unlanded,
an add from a dirty main checkout takes rung 3,
as today.
Step 11 then brings the verified dirty restore.

#### VM measurements

Item 1 repeated on Git `2.56.1` with the built wrapper,
the kill points,
and `ENOSPC`.
Runs in the throwaway guest.

#### Open items

Blocked by "The add-time Git lock" and "The dirty-main restore on rung 1".
The lock lands here, in step 10,
with the registration it protects:
"Add and Git registration" step 2 says "Take the decided managed Git lock here, with the registration",
and VM item 6 measured an unlocked registration pruned at once
("Unmounted, pruned or moved worktrees").
That placement is the plan's suggestion;
the alternative, keeping the shipped add route refusing until step 13 lands,
holds the whole route back for nothing it protects,
so the plan takes the design's own order.
Step 13 then adds `doctor`'s distinction,
recovery after pointer loss and the release.
The dirty half stays step 11's,
with the interim the owner decided on 2026-11-19,
answering "Ship step 10 first (Recommended)".
The binding record's home is a gap this plan names.

#### Depends on

Steps 2,
3,
6 and 8.

### Step 11: The dirty-main restore and the frozen proof

#### What it builds

The defined Git index and ignored-state restore a clone of a dirty main checkout needs,
and the frozen proof that the clone matches the captured source before fill.
It is the verified dirty restore the design decided
("Dirty source on rung 1" under "Project decisions",
"Verified restore (Recommended)"),
and the owner's answer "Ship step 10 first (Recommended)" puts it after step 10.

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
"boundary-safe ignored cleanup, nested path omissions and mounted/unmounted `move`" from the second bullet,
for the move refusal.
"Test repair with surviving admin data and already-pruned state, with and without backup" (third bullet).

#### VM measurements

Item 6 repeated with the built wrapper,
for the pointer and prune arms.
Runs in the throwaway guest.

#### Open items

Proceeds without step 13's distinction,
recovery and release of the managed lock the add already writes.
Taking the pointer half and the rest of the lock in separate steps is the plan's split of one decided entry;
the pointer half alone is what these states and the move refusal need.

#### Depends on

Step 10.

### Step 13: The managed Git lock at registration

#### What it builds

The lock's life after the add:
`doctor`'s and removal's distinction of it from an operator or unexplained lock,
recovery when the pointer is lost,
and its release only in the authorized registration-cleanup step,
after clone destruction is proven.
The write path landed in step 10, with the registration.

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

Blocked by the part of "The add-time Git lock" this step builds:
how `doctor` and removal tell the wrapper's lock from an operator's.
The write path landed in step 10 with the registration.

#### Depends on

Steps 10 and 12.

### Step 14: Removal: the lineage proof, the plan and the removal intent

#### What it builds

Steps 1 and 2 of "Removal and recovery":
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
the removal intent,
the printed list of names to be lost,
and the `unmount` request with its proof.
Steps 4 and 5,
origin cleanup and the Git settle,
need a destroyed clone and are built in step 16.

#### Design sections

"Removal and recovery" steps 1 and 2;
"Clone removal identity protection";
"Bookmarks on a clone being removed";
"Origin cleanup authority" under "Project decisions".

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
The arm whose bookmark the rule passes and whose removal then finishes is step 16's;
the refusal arms are this step's.
"Repeat the `origin`, `createtxg` and transferred-bookmark controls of [VM item 13],
including reverse promotion" from the same bullet.
"Test proof invalidation after administrator lineage changes" from the same bullet:
after a lineage change by an administrator the next removal refuses before its first delete.

#### VM measurements

Items 11,
12 and 13 repeated with the built wrapper,
including the reverse-promotion control.
Runs in the throwaway guest.

#### Open items

Blocked in part by step 9's unmount for the order.
The destruction plan's home and the binding record's home are gaps this plan names.
The origin-cleanup record belongs to step 16, which settles the registration.

#### Depends on

Steps 2,
8,
9,
10 and 13.

### Step 15: The deferred-origin checks

#### What it builds

The check before the first snapshot delete and again before the head destroy:
the clone and its origin are the planned ones,
and the origin is not marked for deferred destroy.
What each refusal prints,
and whether a refusal keeps the removal record.
The project placed the check before the first snapshot delete on 2026-11-19,
answering "Also check first (Recommended)"
("Clone removal identity protection"),
so this step lands ahead of step 16 and no shipped build reaches the snapshot delete without it.

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

Step 14.

### Step 16: The snapshot batch, the head destroy, origin cleanup and the Git settle

#### What it builds

Steps 3,
4 and 5 of "Removal and recovery":
step 15's check before the first snapshot delete;
the full verified snapshot set in one `ZFS_IOC_DESTROY_SNAPS` request that omits the `defer` key,
with the per-name error list kept;
the skip for an empty set;
the split by largest fitting prefix when the set exceeds the measured budget,
with a fresh proof,
fresh all-namespace verification from step 9's scan,
and a fresh step 15 check before each chunk;
the head destroy under the delegated identity policy after the batch's known outcome and a fresh enumeration,
with step 15's check again before it;
the before-destroy inventory against the final loss report;
the resume states the design names,
including planned snapshots all gone with the head refused,
and the re-proof rules for a new own snapshot;
the origin cleanup under the account's descendants-only `mount,destroy`;
and the Git registration and pointer settle after proven clone destruction,
with the separate origin-cleanup record left standing.

#### Design sections

"Removal and recovery" steps 3,
4 and 5;
"Clone removal identity protection";
"Scoped delegation";
"Origin cleanup authority" and "Git registration when origin cleanup is blocked" under "Project decisions";
"Removal when the namespace exclusion cannot be shown" under "Project decisions".

#### SPEC.md passages

The removal section:
the batch,
the split,
the head destroy,
the loss report,
the resume states,
origin cleanup and the settle.

#### Acceptance tests

The rest of the fourth bullet under "Binding,
Git and lifecycle":
its requirement that an ordinary authorized removal destroy the verified clone and its own snapshots,
and print the loss in the same invocation,
the split or replan control,
and the declared same-account residual,
including "verify absence of the lineage-changing grants for
delegated destruction".
The arm whose passing bookmark "lets the removal finish" from the same bullet;
its refusal arms are step 14's.
"a verified unpromoted clone with manual and sanoid snapshots as the removal control" from the same bullet.
The fifth bullet under "Binding,
Git and lifecycle" asks to repeat these through the binding:
the [VM item 14] mixed and own-snapshot batch controls,
and the same batch run as the delegated account under the planned grants.
The interruption arms of "Kill removal across unmount,
snapshot destruction,
clone and origin cleanup,
and Git and directory cleanup" (sixth bullet),
including the wrapper's refusal to destroy a mounted clone from the third bullet under "Boot,
shutdown and setup".
"Check the before-destroy inventory and final loss report on successful and partial removals" (sixth bullet).
"Registration settled while origin cleanup is blocked" under "Decided behaviours with no test yet".

#### VM measurements

Items 14,
17 and 18 repeated through the binding,
plus the request budget the design asks the VM to establish.
Runs in the throwaway guest.

#### Open items

Blocked in part by "The record that carries origin cleanup" for the settle-while-blocked arm,
and by step 9's staging for its unmount.
The destruction plan's home and the binding record's home are gaps this plan names.

#### Depends on

Steps 9,
14 and 15.

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
"What a cancelled removal restores" is mostly settled:
"Decided defaults" keeps no implicit remount with one exception,
the checked reattach of a removal that cancels itself without `--force`,
and "Busy unmount after a staging move" under "Project decisions" answers "Reattach when safe (Recommended)".
What a writer still designs is whether the removal record is reversed or kept,
and what `doctor` prints.

#### Depends on

Steps 9 and 16.

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
the holders of a clone by namespace ID and representative PID,
read from step 9's all-namespace scan result;
the blocked-removal state and the kept cancel command;
the backup coverage.
It prints the concrete root-once and setup commands without running them,
and reports the deferred-origin check as not yet designed until step 15 lands.

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
the built `doctor`'s kernel reads run on this host only after those guest arms pass
and the owner authorises them.
Until then a host `doctor` run refuses at the gate or reports the check as not yet run.

#### Open items

It depends on those open items through the steps that build them,
and blocks none.
It reports the deferred-origin check before step 15 lands,
and any check it cannot read or prove, as unknown.

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
None of this runs on this host.
Applying the migration,
the immutable flag,
the units,
the zedlet,
`zfs allow`,
`zfs set`,
`sanoid.conf`,
property changes on existing datasets,
or any reboot here is the owner's act on the owner's authorisation,
printed by `doctor` and never run by this step.
The reboot this step's arms perform is the guest's.

#### Design sections

"One-time move and checkout boot protection";
"Boot and shutdown unit";
"Checkout boot mount protection" and "Checkout propagation policy" under "Project decisions";
"Snapshots and setup reporting" for the dataset names it creates.

#### SPEC.md passages

A new section under `## Lifecycle` (suggested heading,
"Boot,
shutdown and the checkout's own dataset");
`### Doctor` gains the migration and boot findings.

#### Acceptance tests

"Perform an actual reboot and shutdown with idle and busy legacy clones",
including "Delay or fail import,
key loading and checkout mounting; assert no nested dataset attaches or creates mountpoint folders in the
home underlay",
"Verify direct checkout/home unmount names held clones,
idle export/import with a clone succeeds and recovers, and
busy export fails without a forced teardown",
"Record root's exact-version behavior on destroy of a mounted clone in the guest;
the wrapper must refuse before attempting it regardless of the tool result",
and "Exercise migration and rollback on the disposable layout",
all under "Boot,
shutdown and setup".
"Re-applying checkout propagation" under "Decided behaviours with no test yet".
The ordering and provider-hook arms of "Enable real ZED in the guest and exercise the configured ordering",
including "replay, dropped events with and without a live caller, concurrent zedlets, stale receipts,
and pending unmount across restart",
the "readiness/refusal controls with the handler absent, ABI unsupported and ZED stopped",
and the repetition "with a second host's sanoid and ZED configuration in isolation"
(whose sanoid half step 20 repeats).
"Test which paths the actual Kopia job can see, whether it crosses a ZFS boundary, and restoration of
the checkout and configuration with origins still pinned" from the fourth bullet under "Boot,
shutdown and setup".

#### VM measurements

Items 8,
9 and 10 repeated with the built units and a guest reboot.
Runs in the throwaway guest;
nothing here runs on this host.

#### Open items

Partly blocked by "What re-applies private propagation after a replacement mount".
Two parts the design leaves open block its acceptance arms:
the checkout provider stays conditional in "One-time move and checkout boot protection",
which names "the protected `home-user-Monochromatic.mount` if using a generated mount unit,
or `zfs-mount.service` where it mounts the checkout",
and "Checkout boot mount protection" says
to prove the failure control and production ordering before enabling the gate.
The staging design blocks its boot arm through step 9's open item.
The migration itself is a runbook that needs no design beyond the design's own protection decision;
that reading is the plan's suggestion.

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
None of it is applied to this host by a step.
Writing `sanoid.conf`,
setting `com.sun:auto-snapshot=false`,
or changing a property on an existing dataset here is the owner's act on the owner's authorisation,
printed by `doctor` and never run by this step.

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
"Check queued and inline repair, Cargo prewarm and cross-filesystem lock recovery, mise and launcher repair,
and watch rebinding in the editor" from the same bullet.

#### VM measurements

The pnpm,
Cargo and mise arms and the inotify check in the guest.
Runs in the throwaway guest.

#### Open items

None.

#### Depends on

Steps 3,
10 and 19.

### Step 22: The measurements that size waits and retention

#### What it builds

The mount and unmount budgets from the acceptance workload,
sizing step 7's mount-request deadline and step 9's unmount deadline;
no existing constant changes,
and `REPORT_SCAN_CEILING` stays the health report's scan bound;
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

Two halves.
The guest half runs the guest,
with the host-shaped fixture,
beside a host-shaped control arm,
for waits,
retention and add against ordinary Git.
The storage half measures this host's storage costs,
which a file-backed guest pool cannot,
as "Tooling,
retention and performance" and "Open measurements" require:
"Measure host storage costs separately from guest semantics",
and the measurement record's "What this did not show" says a file-backed encrypted pool measures semantics,
not performance on the host's storage.
That half names its method before it runs (a read-only measurement, or a spare device),
and it is the owner's act on the owner's authorisation,
never a step's.
No shipped limit rests on either half until its measurement exists.

#### Open items

Proceeds without "Which triggers a lost last event waits for" and measures before any rate rule ships.

#### Depends on

Steps 3,
9,
16,
19,
20 and 21.

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
as item 16 was;
building that qcow2 may need host root (loop devices,
or `pacstrap` on some routes),
which is the owner's act on the owner's authorisation,
never a step's.

#### Open items

None of the open items blocks it,
but it repeats steps 2,
8 and 16 on the host's release,
so it comes after them.

#### Depends on

Steps 2,
8,
16 and 19.

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

The products' kernel-touching work is built and tested in the throwaway guest.
Applying anything to this host is the owner's act on the owner's authorisation,
never a step's:
the migration,
the immutable flag,
the units,
the zedlet,
`zfs allow`,
`zfs set`,
`sanoid.conf`,
property changes on existing datasets,
any reboot of this host,
the built `doctor`'s kernel reads,
and a measurement of this host's storage.
Steps 18,
19,
20 and 22 name the part each would need.
Every VM run also retains evidence of guest and pool teardown,
never of system changes on the host,
and captures versions,
assertions and controls,
not just logs,
as the introduction of "Acceptance tests the build still owes" asks.

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
  16 and 17 that unmount (the staging design),
  11 (the dirty restore),
  13 (the lock's distinction and release),
  15 (the deferred checks),
  17 (cancellation and the forced arm),
  and 19's propagation item.
- Every step that changes `package/git-policy/cli` also runs the wrapper's container gate,
  which is the gate "Integration" discusses.
- The binding package's steps run that package's own tasks,
  `cargo nextest` and the type gate the sibling Rust crate `package/jsonc-edit/core-rust` defines,
  per rule `RNQ`.

### What each step runs where

Each entry names the part that needs no kernel or pool,
which runs on this host,
and the part that does.

- Step 1:
  the codec,
  request packing,
  bounded decoding and the bounded fuzz smoke in the gate container on this host;
  nothing in a VM.
- Step 2:
  no host half beyond step 1's;
  every live call runs in the VM.
- Step 3:
  `fs_kind` recognition and the ladder's choice on the host;
  the reflink and cross-dataset controls in the VM.
- Step 4:
  the identity compare,
  the refusal wording and the rollback plan on fixture folders on the host;
  real mounts,
  flags and renames in the VM.
- Step 5:
  mode dispatch and the release and source-guard checks on the host;
  the install in the VM at step 19.
- Step 6:
  the gate's decisions from supplied facts on the host;
  the real module,
  BTF mismatch and probe in the VM.
- Step 7:
  the grammar,
  the byte bounds,
  receipt parsing and the file modes on the host;
  delivery and refusal in the VM.
- Step 8:
  the name grammar,
  intent parsing and the refusal order on the host;
  clone verification and attach in the VM.
- Step 9:
  the reopen flags and the identity compare's inputs on the host;
  the move,
  busy unmount and reattach in the VM.
- Step 10:
  the claim record,
  the managed lock,
  the resume states and `ENOSPC` handling on fixture repositories on the host;
  the real add
  in the VM.
- Step 11:
  the restore and the proof's comparisons on fixture repositories on the host;
  the dirty source and the write
  window in the VM.
- Step 12:
  the state classification and the move refusal on fixture repositories on the host;
  prune against a real
  clone in the VM.
- Step 13:
  recognizing and releasing the lock on fixture repositories on the host;
  the post-loss install in
  the VM.
- Step 14:
  the lineage proof's arithmetic,
  the bookmark rule and plan persistence on captured tuples on the host;
  live
  GUIDs and the refusals in the VM.
- Step 15:
  the checks' decision table on the host;
  the mark and each refusal in the VM.
- Step 16:
  the batch's encoding without a `defer` key,
  the split arithmetic and per-name result parsing on the host;
  the delegated request,
  its budget and the interruption arms in the VM.
- Step 17:
  the cancel command's proof and the caller-session classification on fixture processes on the host;
  ending
  holders and the arms in the VM.
- Step 18:
  report assembly,
  wording and exit status on the host;
  the checks and the printed commands in the VM.
- Step 19:
  the unit text parsed and asserted in a Rust test on the host;
  the real reboot,
  shutdown and migration in the VM,
  where `systemd-analyze` or any service manager runs (rule `MHX`).
- Step 20:
  config parsing and the coverage comparison on the host;
  sanoid 2.3.0 in the VM.
- Step 21:
  the repair stage selection on fixture repositories on the host;
  real installs and watches in the VM.
- Step 22:
  no host half;
  the guest half runs in the VM,
  and the host storage half is the owner's.
- Step 23:
  no host half;
  the qualification run is the VM.


## What can start now

Step 1 can start today.
It has no open item of the design's,
it is its own package,
and its checks are its own, in the container "How it merges" describes,
so the wrapper's container gate is not in its way.

These can run beside it without touching a file it touches:

- Step 3's host half,
  the ladder's ZFS recognition,
  in `package/git-policy/cli/src/native/subvolume/fs_kind.rs`,
  `package/git-policy/cli/src/native/worktree/worktree_snapshot_source.rs`,
  and the SPEC ladder passage.
  Its VM arm waits for step 2's recipe.
- Step 4's attach module,
  in a new folder under `package/git-policy/cli/src/native/`,
  with the module-level tests its root-run harness runs.
  Its VM arms wait for step 2's recipe.
- The three blocking designs,
  as documents:
  the start probe,
  the runtime layout checker and staging under `~/.cache`.
  They are the critical path for steps 6 and 9,
  and nothing else in the plan unblocks them.

Step 2's recipe is not on this list:
it lives in the package step 1 creates,
and its `README.md` and `mise.toml` are files step 1 writes,
so it starts once step 1 lands.
The live calls the recipe serves are step 2's,
not step 1's.

Step 5 starts once steps 1 and 4 land,
because it adds the wrapper's path dependency on the binding,
and steps 3,
4 and 5 change the wrapper,
so each merge waits for the container gate.

While all three designs are unwritten,
the steps whose lists name no design are 1 to 5.
Behind step 6, the start probe and the layout checker,
sit steps 6 to 23;
behind step 9, staging,
sit step 9 and steps 14 to 23.

Parallel writers here need a worktree each.
[handover][]'s entry "No heavy worktree use before the wrapper's ZFS route"
(2026-11-18T21:47:42-05:00) records the owner's words
"We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper".
For the ZFS build steps themselves the owner answered on 2026-11-19,
asked how many worktrees they may use at once:
"Up to 64".
Runs are bounded by each provider pool's measured limit;
on this drive 20 is the first level of the search in `doc/handover/agents-at-once-on-this-machine.md`,
not a cap.
The held queue of other work stays held.

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
- Step 1 merges on its own package's checks in that container.
- Step 2's acceptance is a guest run through `mvm`,
  so its own package's checks include that run.
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
  writer design work.
  Reattach is settled:
  "Decided defaults" keeps "No implicit remount after a failure or restart"
  with the checked reattach as its one exception,
  and "Busy unmount after a staging move" under "Project decisions"
  answers "Reattach when safe (Recommended)".
  What a writer still designs is whether the removal record is reversed or kept,
  and what `doctor` prints.
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
  step 15,
  blocking,
  writer design work.
- "The record that carries origin cleanup":
  step 16,
  blocking for the settle-while-blocked acceptance item,
  writer design work,
  with the owner asked only if its home changes the settle decision.
- "The add-time Git lock":
  steps 10 and 13,
  blocking each,
  writer design work.
  Step 10 takes the lock's write path with the registration,
  which is the plan's choice,
  and step 13 takes the distinction,
  recovery and release.
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

Four things the design leaves unstated are not in "Open items the decisions leave".
Three of them block a step listed here;
the fourth is an accepted cost that `doctor` reports.

- Where the durable binding record lives.
  "Add and Git registration" says "This design does not name where that binding and lineage record lives:
  only the per-add claim record has a named home,
  the account-owned Git common directory".
  Steps 10,
  14 and 18 write or read it.
- Where the persisted destruction plan lives.
  "Bookmarks on a clone being removed" says the plan is persisted under every option,
  and calls its unnamed home "a gap common to all three".
  Steps 14 and 16 write and read it.
- How the release carries two executables.
  `## Single executable and shipped policies` in [spec][] states that the release artifact is one executable,
  while "Binding crate and compatibility" and "Decided defaults" keep a second binary in the same package.
  Step 5 changes this.
  One archive carrying both is forced rather than chosen:
  "Decided defaults" keeps the handler inside `package/git-policy/cli`
  ("Keep inside cli (Recommended)"),
  and that entry says "the handler's code, tests and release move with the wrapper";
  the wrapper manifest's comment says cargo-binstall refuses an archive that lacks any `[[bin]]` of the crate.
  So this is writer work in step 5, not an owner question.
- What removes a receipt for a clone that no longer exists.
  "Requests,
  results and ordering" and "Decided defaults" say nothing removes it,
  and "Decided defaults" records the stale entry as the accepted cost of "Keep receipt files (Recommended)".
  No step deletes it;
  `doctor` reports it,
  which is this plan's suggestion.

## Where the design is silent: this plan's suggestions

Each entry says what the design states and what this plan suggests instead,
and no entry changes a decision.

- The order of the three blocking designs.
  The design states no order.
  The plan suggests writing the start probe,
  the layout checker and the staging design first,
  beside steps 1 to 5,
  because steps 6 and 9 wait on them.
- Where the ZFS interface sits in [spec][].
  The design names no heading.
  The plan suggests one new section per area under `## Lifecycle`,
  with rungs 2 and 3 kept in the existing ladder section and its heading's list gaining ZFS,
  and `### Doctor` extended in place.
- Where the add-time Git lock lands.
  The design takes it at the registration:
  "Take the decided managed Git lock here, with the registration"
  ("Add and Git registration" step 2),
  and VM item 6 measured an unlocked registration pruned at once.
  The plan follows that order rather than keeping the add route refusing until step 13,
  so step 10 writes the lock and step 13 adds its distinction,
  recovery and release.
- The split between the pointer half and the rest of prune protection.
  "Prune protection while unmounted" decides one entry,
  a Git lock in addition to the pointer copies.
  The plan suggests building the pointer half in step 12,
  the lock's write with the registration in step 10,
  and its distinction, recovery and release in step 13,
  because the pointer half alone is what the unmounted states and the move refusal need.
- How step 3 proves a same dataset.
  "Rung 2:
  Same-dataset reflink" says only "a proven same dataset".
  The plan suggests proving it from the mount source and the device,
  with no binding call,
  which keeps step 3 free of the kernel gate;
  if the proof needs `ZFS_IOC_OBJSET_STATS`,
  step 3 also waits on steps 1,
  2,
  5 and 6.
- What the guest carries and how the built programs reach it.
  The design names Git `2.56.1` in the guest and `pahole` there,
  not whether Rust builds inside the guest or host-built binaries are pushed.
  The plan suggests installing Git,
  `pahole` and `strace` in the guest,
  building the crates inside it against its libc,
  and keeping OpenZFS `2.4.4` from the release tarball as the measurements did;
  step 2's recipe decides it.
- How step 4 drives its code before step 5's binary exists.
  The design puts attach in the root handler.
  The plan suggests a root-run integration test in the wrapper crate that calls the module directly,
  outside the account-side suite,
  until step 5 wires the modes.
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
  `REPORT_SCAN_CEILING` is the health report's scan bound,
  already measured on this host on 2026-11-09,
  and no step replaces it.
  No ZFS mount or unmount wait constant exists yet:
  step 7 introduces the mount-request deadline and step 9 the unmount deadline,
  and step 22 sizes both.
  The design's "Waiting and refusal" names that same constant as "a guest trial budget,
  not a ZED latency finding or a shipped limit".
  Correcting that citation in the design is its owner's act;
  this plan does not edit the design.
  The plan suggests no shipped wait number before step 22's measurement.
- Which BTF reader the layout test uses.
  The design names the `pahole` command and asks for a test.
  The plan suggests running `pahole` in the guest and comparing its output in the test,
  rather than adding a Rust BTF parser.
- The module and mode names inside the wrapper.
  The design names the binary,
  the modes and the installed paths,
  not the modules,
  and names no argument for the ZED invocation.
  The plan suggests `worktree_clone_mount` for step 4's module,
  one sibling module per handler mode,
  and the default invocation with no argument running the sweep,
  which is how the installed zedlet calls the binary.
- What to do with a stale receipt.
  The design leaves it standing.
  The plan suggests `doctor` reports it and nothing deletes it,
  since a deletion step would need the decision the design does not make.

## What this plan did not do

- It ran no ZFS command that changes anything on this host,
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
