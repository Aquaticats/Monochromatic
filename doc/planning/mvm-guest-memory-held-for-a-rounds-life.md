# How mvm guests could hold less of the host's memory

This draft is written by an agent and is unapproved:
it proposes,
it changes no code and no `mvm` default,
and the owner chooses.
It was rewritten after a gate read of 2026-11-26T00:00-05:00.
Every figure here is one run,
and each says so where it appears.
The readings and logs behind them are committed in
`doc/planning/mvm-guest-memory-held-for-a-rounds-life-evidence/`,
whose `README.md` names each file.

The units are the samplers':
a guest's or the host's `/proc/meminfo` fields are kB,
as the kernel prints them,
and a QEMU process's charge is its `smaps_rollup` `Rss` plus `Swap`,
also kB;
the ARC,
zram and systemd counters are bytes.
Every figure below carries its unit,
and a byte count is never written where the sample holds kB.

## What changed since the gate read of 2026-11-26T00:00-05:00

Every memory figure was checked against its own sample's `MemTotal`,
which is 64,668,036 kB in every host sample and 8,131,780 kB or 4,009,156 kB in a guest's.
The `/proc/meminfo` fields are kB,
so the opening `MemAvailable` is 5,498,576 kB (5.24 GiB);
as bytes the same number reads 5,498,576 bytes (5.24 MiB),
which is a different quantity
and the error this revision removes.

"The host out of memory" against "memory free" was false and is gone.
`MemAvailable` was above 64 MiB in 167 of the 178 samples that hold a guest in
`mem-a`'s file,
and in 180 of the 182 in `mem-b`'s,
and the drops of both guests ran with the host's memory free.
Each kept drop is now a reading of its own,
one run,
with the `MemAvailable` and `some avg10` of the sample each one followed,
and nothing named as the variable that decides between them.

Every peak of one run set beside a peak of another is gone.
`mem-third-8g-prebuilt`'s 6.17 GiB charge was reached at 04:01:26,
before its binaries were pushed at 04:01:43,
so it is the preparation's footprint and not the arm's;
the 4 GiB guest's 4.04 GiB is its own ceiling;
and the two preparation peaks differ by 1.47 GiB under unlike pressure.
What the cells show about the build is the guest's own `AnonPages`:
7,641,948 kB (7.29 GiB) in the one guest that built the crate,
and 1.48 and 1.47 GiB in the two that only ran prebuilt binaries.
No effect size for building outside the guest is claimed.

The build-location question's form now carries the container build's own host cost,
9,651,453,952 bytes (9.0 GiB) of cgroup peak over 289 seconds,
says that one arm of one driver was measured and the rest are extrapolated,
and says a container build is owed per commit or per round.

The cache-drop and guest-per-arm questions were scoped to "rounds that keep one guest",
which no driver does:
every driver's `all` step creates,
prepares,
runs its arms and destroys its own guest.
Both now ask for a change of the drivers,
named with its cost,
and the section "How a round uses guests today" states what the code does,
with file and line.

Every question gives its options with pros and cons and a ranking that names a reason
for each adjacent pair,
and no question folds two decisions into one option.

The corrected sentence for `doc/decision/mvm-guests-report-free-pages.md` is now in this document,
under "Free page reporting:
decided,
and shipped",
because a report outside the repository is no place for text the record needs.

## What the owner is asked

Each decision has its own section under "The decisions,
each on its own",
with its evidence,
its cost,
a recommendation,
and a compact form for a question dialog.

- Drop a guest's page cache between the arms of one driver run:
  not now;
  the step costs a cold cache,
  it would live in the drivers,
  and the kept drops returned 0,
  2.28,
  1.25,
  1.23,
  0 and 0 GiB,
  one run each,
  with nothing naming what decides between them.
- Create,
  prepare and destroy a guest per arm instead of per driver run:
  not now;
  a create and the preparation are measured in minutes against arms measured in under two seconds,
  and no run measured the gap between two arms of one driver run.
- Build the crate's test binaries outside the guest,
  in an image whose glibc is the guest's,
  at the same absolute path:
  recommended for arms whose subject is not the crate's own build;
  the lock arm passed on such binaries in two guests,
  and the route costs a container build of 289 s with a 9.0 GiB host peak,
  plus one build path per driver.
- Keep the guest at 8,192 MiB while an arm builds the crate inside it:
  the one guest that built the crate put 7.29 GiB of its own `AnonPages` into the build,
  and no run here shows a smaller guest building it.
- Turn KSM on for a measurement round:
  no;
  the memfd route's RAM is page cache,
  which KSM never merges,
  and the one route that can merge has no `/mnt/shared`,
  which every driver's transfers need.

Free page reporting is decided and shipped
(`doc/decision/mvm-guests-report-free-pages.md`,
commit `269b812807`),
and the section "Free page reporting:
decided,
and shipped" records what it buys and costs.

## The question

At 2026-11-25T22:00:41Z,
with three other rounds' guests running,
the host charged 24.03 GiB to those three guests
(8.03,
8.01 and 7.99 GiB;
`Rss` plus `Swap` of each QEMU process),
and its own RAM held 6.80 GiB of that charge.
The rest was in the guests' swap,
which the zram device held as 1.13 GiB over 34.43 GiB swapped across everything on the host
(3.3 percent)
at that moment.
The machine's `MemAvailable` was 5,498,576 kB (5.24 GiB),
and the ARC held 2,088,789,208 bytes against a `c_min` of 2,069,377,152,
19,412,056 bytes above its floor
(`mem-b-host-samples.jsonl`,
line 1:
its `/proc/meminfo` fields are kB and its ARC fields bytes).

One `mvm` guest asks for 8,192 MiB.
The host must be able to give it,
page for page,
for as long as the guest lives,
because nothing tells the host when a page stops mattering.
This document measures how much of that memory the host could get back
while rounds still run their guest arms,
and what each way back costs.

## How a guest's memory is backed and sized

### The domain mvm writes

`package/mvm/cli/src/config.ts` gives a guest 8,192 MiB and 4 vCPUs,
and no command-line flag changes either:
`DEFAULT_MEMORY_MIB` and `DEFAULT_VCPUS` are read by `package/mvm/cli/src/domain-xml.ts`,
which writes `<memory unit="MiB">8192</memory>` and `<vcpu>4</vcpu>`.

When the share decision is `virtiofs`,
`domain-xml.ts` also adds the share's device and,
beside the memory element,
the backing that device needs:

```xml
<!-- package/mvm/cli/src/domain-xml.ts: the backing a virtiofs share needs -->
<memoryBacking>
  <source type="memfd"/>
  <access mode="shared"/>
</memoryBacking>
```

`package/mvm/cli/src/domain-xml-builders.ts` builds those two elements
(`sharedMemoryBacking`),
and its documentation says why:
a virtiofs share needs guest memory that `virtiofsd` can map,
so `access mode="shared"` is the mode,
and `source type="memfd"` the backing.
The guest-agent route,
taken only when libvirt finds no `virtiofsd`,
writes neither the share nor the backing,
which is the one case where a guest's memory is private.

`mvm` now writes the reporting balloon itself:
`domain-xml-builders.ts` builds `memoryBalloon()`
(`<memballoon model="virtio" freePageReporting="on"/>`),
`commonDevices()` puts it in every libvirt domain,
and commit `269b812807` (2026-11-25T19:47:08-05:00) is the change
`doc/decision/mvm-guests-report-free-pages.md` asked for.
No kept file records a guest's domain XML,
so what a guest carried is a claim about the source and about the round that created it:

- the guest of 2026-11-26T04:10:20Z,
  created by this worktree's source with no patch,
  saw the balloon as `virtio4`,
  device id `0x0005`,
  with `virtio_balloon` bound
  (`balloon-check.txt`);
- `mem-second-4g` and `mem-second-8g` were created after that commit,
  and their round added the element by a temporary probe block as well;
- the four guests of 2026-11-25T21:14Z to 23:35Z were created before the commit,
  and which of them carried the element is what their round recorded:
  `mem-a` without it,
  `mem-b` to `mem-d` with it,
  which is also the comparison `doc/decision/mvm-guests-report-free-pages.md` rests on.

### What QEMU then does with that memory

- `access mode="shared"` becomes QEMU's `memory-backend-memfd` with `share=on`,
  and `backends/hostmem-memfd.c` in QEMU v11.1.0 turns that into a RAM block that is `RAM_SHARED` and has a
  file descriptor (`memory_region_init_ram_from_fd`).
  Every page the guest touches becomes a page of that memfd,
  which the host accounts as shared memory,
  and the host's page cache holds it whether or not the guest still uses it.
- Free page reporting,
  the virtio-balloon feature that returns pages the guest has freed,
  is off in QEMU unless the domain asks for it.
  QEMU's property default is off (`DEFINE_PROP_BIT("free-page-reporting", ..., VIRTIO_BALLOON_F_REPORTING, false)`
  in `hw/virtio/virtio-balloon.c`),
  and libvirt writes the QEMU property only when the domain XML says
  `freePageReporting='on'`
  (`src/qemu/qemu_command.c` passes the XML's tristate straight through).
- When it is on,
  QEMU discards a reported page through `ram_block_discard_range`,
  which for a RAM block with a file descriptor calls
  `fallocate(FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE)` on the memfd and then
  `madvise(MADV_DONTNEED)` on the mapping (`system/physmem.c`).
  Punching a hole in a memfd frees the page from the host's page cache,
  so reporting is not blocked by `access mode="shared"`:
  nothing in the virtiofs route disables discards.
  `ram_block_discard_disable` is called by vfio,
  vhost-vDPA,
  SEV guests,
  `blkio` block devices and KVM without a synchronised MMU,
  none of which this guest has.
- KSM cannot touch that memory.
  QEMU marks a guest's RAM mergeable when merging is on
  (`backends/hostmem.c` calls `qemu_madvise(..., QEMU_MADV_MERGEABLE)`),
  but the kernel's KSM documentation states that KSM
  "only merges anonymous (private) pages,
  never pagecache (file) pages",
  and a memfd is page cache.
  The host's own view agrees:
  the running guests' RAM mappings carry no `mg` flag in `/proc/<pid>/smaps`,
  while six smaller mappings in the same processes do.

### What that costs per guest

A guest asked for 8,192 MiB,
so the host may hold up to 8 GiB of shared memory per guest,
and the unit's `MemoryMax` is half of `MemTotal`,
which is 30.8 GiB here,
so four guests at the default exceed the cap:
`package/mvm/cli/README.md` states that the kernel then kills the guest it picks rather than the daemon.

## What a real arm needs, and why a binary is not enough

### The arm a driver runs

The wrapper's guest arms are not "build the wrapper,
then run it".
Each driver under `package/git-policy/cli/e2e/*-guest.ts` creates an Ubuntu 24.04 guest through
`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`,
which installs the tools,
builds OpenZFS 2.4.4 from its release tarball inside the guest
against the guest's own kernel headers (`run-fixture.ts`,
the `prepare` step,
`make --jobs=4`),
installs `cargo-nextest` from source,
and then the driver builds and runs the crate's own test binaries in the guest.

The lock arm this document measures is
`zfs_managed_lock_guest::the_managed_lock_outlives_the_pointer_and_goes_after_the_clone`,
which `managed-git-lock-guest.ts` selects with a nextest filterset for `test(zfs_managed_lock_guest)`:

```sh
# package/git-policy/cli/e2e/managed-git-lock-guest.ts:210-212: the arm's own command
cargo nextest run --release --locked --features test-only-switches --test native_binary \
  --no-capture --status-level all --filterset "test(zfs_managed_lock_guest)"
```

The measuring cells did not run that command.
The guest that built the crate ran `cargo nextest` without `--features test-only-switches`,
and the two guests given the container's binaries ran the test binary directly with `--nocapture`,
so the cells' flags rest on this document's account and not on a kept log.

Its subject is a lock's lifetime in a ZFS dataset,
not the crate's build,
so it is an arm the build location can move without losing its subject.

### How a round uses guests today

A round is one agent's work on this repository,
a revising round or a test round alike.
A round that runs guest arms runs one or more drivers' `all` steps.
An arm is one test case,
nextest filterset or probe step a driver runs in its guest.

Every driver under `package/git-policy/cli/e2e/*-guest.ts` runs one guest per `all` step:
it creates the guest,
prepares it,
runs every arm of that driver run in it,
and destroys it
(`deferred-origin-check-guest.ts:252`,
`doctor-zfs-route-guest.ts:265`,
`managed-git-lock-guest.ts:239`,
`mount-git-worktree-clone-datasets-guest.ts:931`,
`removal-lineage-proof-guest.ts:473`,
`sanoid-coverage-guest.ts:271`,
`settling-command-guest.ts:295`,
`slot-state-and-move-refusal-guest.ts:249`,
`start-probe-guest.ts:859`).
So a guest lives for one driver run,
not for a round's arms,
and a round that runs several drivers creates and destroys one guest per driver.
A guest's charge matters between the arms of one driver run and not after them.

Every driver builds in that guest,
at a path of its own:

- `deferred-origin-check-guest.ts:36` and `:203`,
  at `/opt/deferred-origin-check`;
- `doctor-zfs-route-guest.ts:27` and `:191` to `:193`,
  at `/opt/git-policy-cli-doctor-route`;
- `managed-git-lock-guest.ts:28` and `:196`,
  at `/opt/git-policy-cli-lock`;
- `mount-git-worktree-clone-datasets-guest.ts:37` and `:435`,
  at `/opt/git-policy-cli-adoption`;
- `removal-lineage-proof-guest.ts:47` and `:383`,
  at `/opt/removal-lineage-proof`;
- `sanoid-coverage-guest.ts:31` and `:204`,
  at `/opt/git-policy-cli-sanoid-coverage`;
- `settling-command-guest.ts:35` and `:245`,
  at `/opt/settling-command`;
- `slot-state-and-move-refusal-guest.ts:33` and `:212`,
  at `/opt/git-policy-cli-slot-state`;
- `start-probe-guest.ts:40` and `:207` to `:208`,
  at `/opt/git-policy-cli-start-probe`.

The build's flags differ too:
every driver but `sanoid-coverage-guest.ts` passes `--features test-only-switches`
(`sanoid-coverage-guest.ts:204` passes none),
`sanoid-coverage-guest.ts` builds and runs without `--release`,
`start-probe-guest.ts:208` builds a second tree in `target/raised-deadline`,
and four drivers cap the compiler's parallelism:
`removal-lineage-proof-guest.ts:383` and `settling-command-guest.ts:245` with `--jobs 1`,
`mount-git-worktree-clone-datasets-guest.ts:435` with `CARGO_BUILD_JOBS=1`,
and `sanoid-coverage-guest.ts:203` with `CARGO_BUILD_JOBS=2`.
The guest recipe caps its own wrapper builds at `--jobs 2`
(`run-fixture.ts:660`)
and OpenZFS's at `make --jobs=4`
(`run-fixture.ts:386`).

Two drivers run more than one test binary in their one guest:
`doctor-zfs-route-guest.ts:212` runs `native_binary` through nextest and `:215` runs the lib through `cargo test`,
and `removal-lineage-proof-guest.ts:417` runs the lib and `:423` the `native_binary`,
both through nextest.
One driver runs four nextest invocations and a Python probe in its guest
(`mount-git-worktree-clone-datasets-guest.ts:457`,
`:510`,
`:650` and `:896`),
one runs its own probe steps against the built wrapper and no test binary
(`start-probe-guest.ts:859`),
and every other driver runs one nextest invocation.
So a build placed on the host must reproduce each driver's features,
path,
target directory and profile,
and a driver that runs two test binaries needs both.

Every driver's transfers go through `/mnt/shared`
(`deferred-origin-check-guest.ts:197`,
`doctor-zfs-route-guest.ts:183`,
`managed-git-lock-guest.ts:190`,
`mount-git-worktree-clone-datasets-guest.ts:421`,
`removal-lineage-proof-guest.ts:356`,
`sanoid-coverage-guest.ts:194`,
`settling-command-guest.ts:234`,
`slot-state-and-move-refusal-guest.ts:199`,
`start-probe-guest.ts:201`),
and three of them run a step itself from there through `guestStart` and `guestFinish`
(`mount-git-worktree-clone-datasets-guest.ts:321` and `:356`,
`removal-lineage-proof-guest.ts:248` and `:283`,
`settling-command-guest.ts:133` and `:163`).

### The host paths the test binaries carry

`env!` writes its value into the binary's text at compile time.
Read on `main` at commit `4aa035890d` on 2026-11-26,
the crate's sources hold 42 such path sites:

- `env!("CARGO_MANIFEST_DIR")`:
  34 sites in 26 files,
  mostly to read the crate's own manifest,
  `clippy.toml`,
  `SPEC.md`,
  the word-merge corpus and the source tree at run time;
- `env!("CARGO_BIN_EXE_monochromatic-git-policy-cli")`:
  7 sites in 7 files,
  the absolute path of the wrapper the test starts;
- `env!("CARGO_TARGET_TMPDIR")`:
  one site,
  a directory inside the target directory.

Fourteen further `env!` sites carry `CARGO_PKG_VERSION` and `CARGO_PKG_NAME`,
which are text,
not paths,
and no relocation touches them.
The full inventory,
with every file and line,
is `compile-time-paths.txt` in the evidence folder.

Two examples show the shape:
`binary_zfs_doctor_route_guest_tests.rs` builds
`{CARGO_MANIFEST_DIR}/target/release/monochromatic-git-policy-cli`,
and `binary_support_program_names_and_time_bound.rs` holds
`pub const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");`.

### What can place them where the guest finds them

Cargo's environment reference says what these values are
(Cargo book,
"Environment variables Cargo sets for crates",
`https://doc.rust-lang.org/cargo/reference/environment-variables.html`):
`CARGO_MANIFEST_DIR` is "The directory containing the manifest of your package",
and `CARGO_BIN_EXE_<name>` is "The absolute path to a binary target's executable".
They are paths of the machine that built the binary.

rustc's own remapping cannot move them.
The rustc book's "Remap source paths" page
(`https://doc.rust-lang.org/rustc/remap-source-paths.html`)
remaps "source paths prefixes ... in all compiler generated output",
and its `--remap-path-scope` scopes are `macro` (the `std::file!()` expansion),
`diagnostics`,
`debuginfo`,
`coverage`,
`object` and `all`.
Under "External tools" it adds:
"Paths introduced by external tools or environment variables may not be covered by
`--remap-path-prefix` unless explicitly accounted for."

A one-file check with rustc 1.102.0 on this host showed the same:

```text
# a one-file probe: rustc's remapping leaves the env! text alone
CARGO_MANIFEST_DIR=/original/build/path rustc --edition 2024 \
  --remap-path-prefix=/original/build/path=/remapped ./probe.rs -o ./probe
./probe
manifest=/original/build/path
```

nextest documents a second route,
for builds moved between machines
(nextest book,
"Archiving and reusing builds",
`https://nexte.st/docs/ci-features/archiving/`):
`cargo nextest archive --archive-file <name>.tar.zst` on the build machine,
then `cargo nextest run --archive-file <name>.tar.zst --workspace-remap <path-to-workspace-root>`
on the target machine,
where "If the workspace is remapped,
nextest automatically sets `CARGO_MANIFEST_DIR` to the new location".
That page's condition is what makes it a code change for this crate:
"For relocatable tests,
use the value of `CARGO_MANIFEST_DIR` at runtime.
This means `std::env::var("CARGO_MANIFEST_DIR")`,
not `env!("CARGO_MANIFEST_DIR")`."
For a crate's executables it names `NEXTEST_BIN_EXE_<name>` (0.9.113)
and,
from 0.9.130,
`CARGO_BIN_EXE_<name>`,
both set at run time;
the guest installs nextest 0.9.152,
so both are available there.
The route also asks the target machine to hold the source at the same revision,
which the guest does anyway.

So two routes exist:

- the same absolute path,
  which needs no code change and is the route the cells here used;
- a nextest archive with `--workspace-remap`,
  which needs the crate's 42 `env!` path sites changed to reads at run time,
  because `env!` bakes the build machine's text into the binary,
  and the remap cannot rewrite bytes already in the text segment.

The first is cheaper to adopt and the second survives a guest whose paths differ;
neither is measured on this crate,
and the archive route's own limits (its page also asks for the same source revision
and warns that archives carry no source) are unweighed here.

### The glibc floor of the binaries, and the guest's own

`run-fixture.ts` records why it builds inside the guest:
"the host's glibc is newer than Ubuntu 24.04's,
so a host-built binary would not start there;
building in the guest keeps the test binary matched to the guest's own libc and needs no cross toolchain."
Every build kept here was made where the guest's libc is:
an Ubuntu 24.04 container on the host,
with Rust 1.100.0,
the toolchain version `run-fixture.ts` pins.

This round's build,
`container-build-binaries-2026-11-26T03-51Z.txt` in the evidence folder,
took 289 seconds inside its capped scope and printed:

- Ubuntu 24.04.5,
  glibc 2.39-0ubuntu8.12;
- the wrapper,
  31,524,648 bytes,
  and the `native_binary` test binary,
  17,218,360 bytes,
  each needing at most `GLIBC_2.39`
  (`readelf --version-info`,
  keeping every `GLIBC_x.y` and taking the last in version order);
- a peak cgroup memory of 9,651,453,952 bytes (9.0 GiB) for the container itself,
  read from the build's own `/sys/fs/cgroup/memory.peak`.

The floor equals the guest's,
so the fit has no margin:
a rebuild that picks up one newer versioned symbol would not start in the guest.
That is a property of the crate's use of `pidfd_spawnp` and `pidfd_getpid`
(`package/git-policy/cli/README.md`:
glibc 2.39 or newer),
not of any one build.

What this says per image and backend:

- Ubuntu 24.04:
  measured here,
  one arm on host-built test binaries.
- Fedora 43:
  not measured.
  Its glibc is newer than 24.04's,
  so a binary with a 2.39 floor would start
  there,
  but nothing here proves the reverse direction for Fedora-built binaries.
- Alpine 3.23:
  cannot run these binaries at all;
  it is musl,
  and they are glibc executables.
- Windows:
  cannot;
  it runs no ELF.
- Hetzner:
  the same image would apply,
  and nothing was measured there.
- CPU:
  libvirt guests use `host-passthrough` (`package/mvm/cli/src/domain-xml.ts`),
  so the guest's CPU is the host's and CPU level is moot on that backend;
  it would not be on Hetzner.

The arms this repository runs use Ubuntu 24.04,
so the measured image is the one that matters here.

## Method

Every guest is a throwaway `mvm` libvirt guest on the Ubuntu template,
started only through `mvm`,
one at a time,
and destroyed by the program that created it as soon as its readings are saved.
The `mvm` that creates it is the worktree's own source,
run as `node package/mvm/cli/src/cli.ts` through a link named `mvm`,
so the domain is the one `main` writes and not whatever an installed copy was built from.
No `mvm` source was patched:
`main`'s own domain is what ran,
with the reporting balloon commit `269b812807` in it.

Before each create a memory gate is read in one command of its own:
`MemAvailable` plus the ARC's `size` minus `c_min` at least 16 GiB,
and memory pressure's `some avg10` at or under its `avg60`.
The gate reading and the create it permitted are in
`memory-gate-readings-2026-11-26T03-56Z.txt`,
and the create's own log follows it.

The host samples every 10 seconds,
before the create and until after the destroy:
`/proc/meminfo`,
memory pressure,
the ARC's `size` and `c_min`,
each zram device's `mm_stat`,
KSM's counters,
and each QEMU process's `smaps_rollup`,
largest mapping and mergeable-mapping count.
The guest samples its own `/proc/meminfo` every 5 seconds for its whole life,
as a transient unit inside the guest,
so a failure inside a step still leaves the guest's view.

The readings of `mem-second-4g` and `mem-second-8g` were taken
between 2026-11-26T01:50:43+00:00 and 2026-11-26T02:27:32+00:00,
and are kept in the evidence folder unchanged.
`mem-third-8g-prebuilt` was read at 2026-11-26T03:56:49+00:00,
and its step timings come from its driver's own clock
(`step-timings-2026-11-26T03-56Z.txt`),
because the preparation's three logs carry no timestamps.

`mem-second-4g` needed a memory size `mvm` has no flag for,
so the worktree that created it carried a temporary block in
`package/mvm/cli/src/domain-xml.ts`
that read `MVM_MEMORY_PROBE`;
that worktree was reverted before its commit,
and the block is in no commit here.
`mem-second-8g` and `mem-third-8g-prebuilt` needed none:
both ran `mvm`'s own defaults.

The gate readings those creates followed,
as the file shows them:
the openings that permitted them were at 2026-11-26T01:35:10+00:00 and 2026-11-26T02:02:06+00:00,
15 and 8 minutes before the creates at 2026-11-26T01:50:43+00:00 and 2026-11-26T02:10:07+00:00,
and two further openings (2026-11-26T00:25:01+00:00 and 2026-11-26T01:17:26+00:00) started no guest
(`memory-gate-readings-2026-11-26.txt`).
`mem-third-8g-prebuilt`'s create follows its gate line in the file directly.

Each cell's load was one real arm,
the lock arm:
the guest that built the crate ran `cargo build --release --tests --locked` inside itself
and then the test through `cargo nextest`;
the guests given the container's binaries received the same source tree and the two binaries
at the container's own path,
and ran the test binary directly with the same environment variables.
Neither command carries the driver's `--features test-only-switches`
(the section "How a round uses guests today" quotes the driver's own),
and no kept log records the flags the cells used.

After the arm,
each guest drops its page cache (`sync; echo 3 > /proc/sys/vm/drop_caches`),
and the host's samples show what that returns.

## What the charge measures, and its limits

The charge is what the host holds for a guest at one moment:
the QEMU process's `smaps_rollup` `Rss` plus `Swap`,
which the samplers keep as `rssKb` and `swapKb`,
its RAM mapping being the largest mapping,
named in every sample.
Read as host memory,
its two parts are:

- every resident page of that mapping,
  at its full size,
  and
- the guest's swapped pages,
  at the size zram holds them,
  not the size the guest thinks it swapped.

The second part is an estimate:
zram's counters are device-wide,
so the share of one guest's swap is its `Swap` multiplied by
`/sys/block/zram1/mm_stat`'s `mem_used_total` divided by `orig_data_size` at the same sample,
and other rounds' guests move that ratio.
The ranges the kept files give:

- `mem-b`,
  182 samples:
  1.31 to 16.79 percent;
- `mem-c`,
  28 samples:
  4.35 to 5.80 percent;
- `mem-d`,
  31 samples:
  2.36 to 6.40 percent;
- `mem-second-4g`,
  42 samples:
  3.12 to 10.82 percent;
- `mem-second-8g`,
  81 samples:
  0.31 to 28.88 percent;
- `mem-third-8g-prebuilt`,
  30 samples:
  1.89 to 2.81 percent.
- `mem-a` holds no zram counters at all,
  so its compressed share has no reading,
  and no figure here counts it.

The resident part has a limit of its own,
and it is the one that matters most for the at-rest figures:
the host reclaims a guest's page cache,
because that cache is the host's own page cache over the memfd.
The charge therefore falls while the guest still believes it holds the memory,
and one sample shows how far.
Measured in `mem-second-8g`:
at 2026-11-26T02:15:01Z the charge was 4.70 GiB,
and at 02:15:45Z it was 0.36 GiB (0.18 resident,
0.18 swapped),
while the guest's own `MemTotal` minus `MemFree` read 4.18 GiB,
3.42 GiB of it `Cached`
(`mem-second-8g-host-samples.jsonl` lines 24 and 28,
`mem-second-8g-guest-memory.log.txt` at 02:15:45Z).

So a charge is a reading at a moment,
not a property of an option,
and an at-rest figure is a lower bound of what the guest believes it uses.
A peak is the steadier reading,
because the guest's own anonymous pages must be resident or in swap to run,
but only within one configuration and one stage:
a peak taken while the preparation builds OpenZFS and a peak taken while the crate is built
measure different work,
and the questions turn on none of those peaks for an effect size.

## The cells, one run each

Each cell below ran the lock arm in one guest;
the guests under "The four guests of 2026-11-25T21:14:34+00:00 to 2026-11-25T23:35:20+00:00" measured other loads.

### A default guest that builds the crate inside itself

Guest `mem-second-8g` was `mvm`'s default:
8,192 MiB asked for (`MemTotal` 8,131,780 kB,
7,755 MiB),
the virtiofs share,
the shared memfd backing,
and reporting on.
The ZFS binding's recipe prepared it as it prepares the arms' guests today,
and then it built the crate inside itself,
as `managed-git-lock-guest.ts` does,
except that the build command carries no `--features test-only-switches`:
`cargo build --release --tests --locked`,
which took 7 minutes 40 seconds (`mem-second-8g-guest-crate-build.log.txt`)
and used almost the whole guest:
its own `AnonPages` peaked at 7,641,948 kB (7.29 GiB) of the 7,755 MiB at 02:24:26Z,
with `MemAvailable` down to 88,740 kB (86.7 MiB).
`cargo nextest` ran the same lock test:
1 passed,
1354 filtered out,
0.57 seconds,
and the pulled log carries all 15 marker lines.

The host's charge to it,
one run,
from the create at 02:10:07Z to the destroy at 02:27:32Z,
with three or four other rounds' guests on the host:

- create:
  10.5 seconds (02:10:07.836Z to 02:10:18.360Z).
- peak charge:
  7.98 GiB at 02:25:13Z (7.71 GiB resident,
  0.27 GiB swapped),
  with three other guests on the host,
  and the crate build's own `AnonPages` peak at 02:24:26Z.
  Its swapped part peaked at 2.99 GiB at 02:21:23Z and fell back as the guest faulted pages in.
- the crate build:
  7 minutes 40 seconds.
- the arm:
  0.57 seconds by the test binary's summary;
  at 02:26:59Z the charge read 6.12 GiB (5.92 resident,
  0.20 swapped).
- the drop:
  the charge read 6.69 GiB at 02:26:47Z (6.45 resident,
  0.24 swapped) and 5.44 GiB at 02:27:22Z (5.24 resident,
  0.19 swapped),
  1.25 GiB less over 35 seconds.
  In the same window the guest's own `AnonPages` fell from 2,072,236 kB (1.98 GiB) at 02:26:47Z
  to 46,288 kB (0.04 GiB) at 02:27:02Z,
  and its `Cached` from 488,648 kB (0.47 GiB) to 26,120 kB (0.02 GiB).
  The cache fall,
  0.44 GiB,
  is the part a cache drop frees;
  the anonymous collapse,
  1.93 GiB,
  is a candidate for the rest,
  and no mechanism is named here for that part.
- destroy:
  1.03 seconds (02:27:31.250Z to 02:27:32.278Z).

The guest's own view at the end is worth stating:
`MemTotal` 8,131,780 kB,
`MemFree` 7,769,540 kB (7.41 GiB),
`Cached` 26,080 kB,
`AnonPages` 46,648 kB,
while the host still held 5.24 GiB resident for its mapping
(`mem-second-8g-guest-drop.log.txt`).
The host had not discarded that memory by the time the guest was destroyed.

### A 4 GiB guest on container-built binaries

Guest `mem-second-4g` was `mvm`'s own create with a 4 GiB memory size,
4,096 MiB,
and the reporting balloon (`MemTotal` 4,009,156 kB,
3,915 MiB,
in the guest).
The ZFS binding's recipe prepared it exactly as it prepares the arms' guests today,
OpenZFS included.
Then,
instead of building the crate in the guest,
it received the container's source tree and two binaries at `/opt/git-policy-cli-lock`:
the path the container built them at,
which is also the path the lock arm's driver uses in the guest.
Both binaries started in the guest,
and the arm was the same test the other cells ran,
with the same four environment variables,
run directly from the test binary with `--nocapture`.

It passed:

```text
# mem-second-4g-lock-arm.log.txt: the arm's summary line
test zfs_managed_lock_guest::the_managed_lock_outlives_the_pointer_and_goes_after_the_clone ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1354 filtered out; finished in 1.13s
```

The pulled log carries all 15 `LOCK` marker lines,
so the arm performed every step rather than returning early.

The host's charge to it,
one run,
from the create at 01:50:43Z to the destroy at 01:59:02Z,
with three or four other rounds' guests on the host:

- create:
  10.7 seconds (01:50:43.675Z to 01:50:54.403Z).
- whole-life peak charge:
  4.04 GiB at 01:56:18Z,
  all resident and no swap,
  held at 4.03 GiB from 01:55:30Z to 01:57:42Z,
  with four other guests on the host.
  That is the guest's whole 4,096 MiB request plus QEMU's own overhead,
  so the reading measures the guest's ceiling,
  not its need.
  The guest's own view did not peak there:
  its `AnonPages` reached 1,555,568 kB (1.48 GiB) at 01:57:51Z,
  with `MemAvailable` 1,973,592 kB (1.88 GiB),
  so the rest of the charge was guest page cache,
  which the guest counts as reclaimable and the host holds.
- the arm:
  1.13 seconds by the test binary's summary;
  at 01:58:28Z the charge read 3.23 GiB (2.92 resident,
  0.31 swapped).
  The fall from 4.03 GiB at 01:57:42Z began before the drop:
  at 01:57:54Z the host's `MemAvailable` read 0.00 GiB with `some avg10` 5.17,
  at 01:58:05Z 5.29 GiB with `avg10` 8.52,
  and the charge read 3.14 GiB at 01:58:17Z
  (`mem-second-4g-host-samples.jsonl` lines 36 to 39).
- the drop:
  the charge read 2.00 GiB at 01:58:40Z at the next sample after 3.23 GiB
  (1.90 resident,
  0.11 swapped),
  and 2.00 GiB at 01:58:51Z (1.86 resident,
  0.14 swapped),
  so the charge fell 1.23 GiB in the drop's own window.
  The guest's own `Cached` fell from 1,521,904 kB (1.45 GiB) at 01:58:26Z to 51,288 kB (0.05 GiB) at 01:58:31Z,
  and `MemFree` rose to 3,759,068 kB (3.58 GiB),
  so the guest freed 1.40 GiB of cache in that window.
  The host still held 1.86 GiB resident after both the drop and reporting.
- destroy:
  0.85 seconds (01:59:01.311Z to 01:59:02.165Z).

The preparation's three steps carry no timestamps of their own,
so no split of its length is claimed here.

### A default guest on container-built binaries

Guest `mem-third-8g-prebuilt` was `mvm`'s default again,
8,192 MiB,
the share and the reporting balloon,
and it did not build the crate:
it received the source tree and the two binaries the same container built,
at `/opt/git-policy-cli-lock`,
and ran the same lock arm directly.

It passed:

```text
# mem-third-8g-prebuilt-lock-arm.log.txt: the arm's summary line
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1356 filtered out; finished in 0.39s
```

with all 15 `LOCK` marker lines in the pulled log,
and the two binaries started at the container's path first:
the wrapper printed `monochromatic-git-policy-cli 0.6.1-alpha.46, shadowing git at /usr/bin/git`.

The host's charge to it,
one run,
from the gate reading at 03:56:49Z to the destroy at 04:02:19Z,
with two other rounds' guests on the host and the host's `MemAvailable` between 9.08 and 14.06 GiB:

- create:
  9.78 seconds (03:56:49.411Z to 03:56:59.186Z),
  and 9.5 seconds between the two lines of `mvm create`'s own log.
- the preparation:
  284.22 seconds (4 minutes 44 seconds),
  the ZFS binding's recipe with OpenZFS built in the guest;
  it ended at 04:01:43.826Z.
- the push of the container's source tree and both binaries,
  and their placement at `/opt/git-policy-cli-lock`:
  2.17 seconds for a 12,994,872-byte source tarball and two binaries of
  31,524,648 and 17,218,360 bytes (61.7 MB in all),
  with both starting at the end of it.
- the pool:
  0.39 seconds.
- peak charge:
  6.16 GiB at 04:01:26Z and 6.17 GiB from 04:01:37Z,
  all resident and no swap,
  with the host's `some avg10` at or under 0.48 throughout.
  The peak was reached before the preparation ended at 04:01:43.826Z,
  so it is the preparation's footprint,
  and the arm at 04:01:46Z did not lift it (6.17 GiB at 04:01:48Z).
  This run therefore does not measure the arm's charge on the container's binaries;
  what it adds is the arm's pass and the guest's own readings.
- the arm:
  0.75 seconds by the driver's clock,
  0.39 seconds by the test binary's own summary,
  and the summary is the figure comparable with the other cells'.
- the drop:
  no return.
  It ran from 04:01:47.139Z to 04:02:18.067Z.
  The charge read 6.17 GiB at 04:01:37Z,
  04:01:48Z,
  04:01:59Z and 04:02:10Z,
  while the guest's own `Cached` fell from 3,912,288 kB (3.73 GiB) at 04:01:34Z
  to 59,012 kB (0.06 GiB) at 04:01:49Z,
  its peak having been 3,949,596 kB (3.77 GiB) at 04:01:39Z,
  and its `AnonPages` from 1,541,700 kB (1.47 GiB) to 49,328 kB (0.05 GiB).
  The guest ended reading `MemFree` 7,756,180 kB of 8,131,780 kB,
  `Cached` 58,648 kB and `AnonPages` 49,480 kB,
  so it had freed about 5 GiB by its own account,
  and the host still held 6.17 GiB resident for its mapping 23 seconds after the drop,
  until the destroy.
- destroy:
  1.17 seconds by the driver's clock,
  0.90 seconds between the two lines of `mvm destroy`'s own log (04:02:18.686Z to 04:02:19.581Z).

### The four guests of 2026-11-25T21:14:34+00:00 to 2026-11-25T23:35:20+00:00

Guests `mem-a` to `mem-d` were one throwaway `mvm` guest each,
destroyed after their readings were pulled.

- `mem-a`,
  reporting off,
  one wrapper build:
  240 host samples between 2026-11-25T21:14:34+00:00 and 2026-11-25T22:10:05+00:00,
  of which 178 hold its QEMU process;
  peak charge 8.16 GiB at 21:47:19Z (3.71 resident,
  4.45 swapped);
  7.98 GiB before its drop at 21:54:43Z (0.98 resident,
  7.00 swapped),
  which ran with `MemAvailable` 17.12 GiB and `some avg10` 0.13;
  and 7.98 GiB after it at 21:55:46Z (1.09 resident,
  6.89 swapped),
  so the drop returned nothing.
  `some avg10` reached 53.54 and was above 1 in 83 of its 178 samples;
  `MemAvailable` was at or under 64 MiB in 11 of those 178 samples,
  and its lowest was 0.00 GiB.
  Its guest-side log ends at 21:51:34Z,
  before the drop,
  so no guest-side reading of the drop is kept,
  and its file holds no zram counters.
- `mem-b`,
  reporting on,
  one wrapper build:
  183 host samples between 2026-11-25T22:00:41+00:00 and 2026-11-25T23:09:46+00:00,
  of which 182 hold its QEMU process;
  peak charge 8.20 GiB at 22:46:32Z (6.48 resident,
  1.72 swapped);
  at rest 5.52 GiB at 23:01:00Z (0.20 resident,
  5.32 swapped),
  the device holding 13.25 percent,
  with `MemAvailable` 6.75 GiB and `some avg10` 0.00;
  after the drop 3.24 GiB at 23:01:46Z (0.25 resident,
  2.99 swapped),
  the device holding 8.73 percent,
  and the sample after the drop read `MemAvailable` 0.73 GiB with `some avg10` 1.20.
  So the drop returned 2.28 GiB,
  all of it from the swapped part,
  while the resident part rose 0.05 GiB.
  `some avg10` reached 24.94 and was above 1 in 68 of its 182 samples;
  `MemAvailable` was at or under 64 MiB in 2 of them.
- `mem-c`,
  the guest-agent route,
  which is what a host without `virtiofsd` gets:
  no `filesystem` device and no `memoryBacking`,
  `meta.json` reading `"fileTransfer": "guest-agent"`.
  QEMU then holds the guest's RAM as `memory-backend-ram` instead of the memfd,
  the largest mapping is `rw-p`,
  anonymous and private,
  and it carries the mergeable flag:
  `VmFlags` included `mg` on that mapping,
  and 7 of the process's 816 mappings carried it (the memfd guests:
  6,
  never the RAM mapping).
  The session unit charged its bytes to `memory.stat` `anon` rather than `shmem`.
  It was charged 3.07 GiB after the same preparation the others ran
  (2.67 resident,
  0.40 swapped),
  with `MemAvailable` 9.96 GiB and `some avg10` 0.10,
  and its drop returned nothing:
  3.07 GiB before,
  3.08 GiB after,
  while the guest's own `Cached` fell from 1,564,048 kB (1.49 GiB) to 59,248 kB (0.06 GiB).
  The price of the route is also visible:
  the guest has no `/mnt/shared`,
  through which every driver copies its subject tarball and pulls its logs,
  and that round pushed its snapshot through the guest agent instead,
  where the memfd route uses the share.
  That round's push durations are not kept in this evidence folder,
  and neither are `mem-c`'s create and destroy logs.
- `mem-d`,
  a 4 GiB guest,
  ran a smoke script rather than the arm,
  which stopped at a worktree refusal
  (`fatal: 'main' is already used by worktree at '/root/work/repo'`,
  `mem-d-guest-run.log.txt`):
  peak charge 0.76 GiB at 23:31:17Z (0.66 resident,
  0.10 swapped),
  and no build.
  It measures the floor of a guest that does nothing heavy,
  not an arm.

### What the cells do not compare

The figures are one run each,
under unlike pressure:

- the heaviest host pressure was in `mem-a`
  (`some avg10` 53.54,
  `MemAvailable` at or under 64 MiB in 11 of the 178 samples that hold its guest),
  and the lightest in `mem-third-8g-prebuilt`
  (`some avg10` 0.48 at most,
  `MemAvailable` minimum 9.08 GiB);
- the same preparation read 4.70 GiB at 02:15:01Z in `mem-second-8g`
  and 0.36 GiB by 02:15:45Z as the host's `some avg10` passed 34.75,
  while `mem-third-8g-prebuilt`'s preparation peak of 6.17 GiB was held
  with `some avg10` at or under 0.48:
  two runs of the same work,
  1.47 GiB apart;
- the 4 GiB guest's 4.04 GiB is its whole request,
  so it measures a ceiling;
- `mem-a` and `mem-c` ran without a reporting balloon,
  `mem-b` and `mem-d` with one,
  which is what their round recorded;
  no kept file records a guest's domain XML,
  and `balloon-check.txt` shows a balloon device only,
  which libvirt adds to a domain that does not name one,
  so it does not show reporting negotiated;
- `mem-c` held private RAM and the rest shared memfd RAM;
- the other rounds' guests numbered none in `mem-a`'s samples,
  one in `mem-b`'s,
  `mem-c`'s and `mem-d`'s,
  two in `mem-third-8g-prebuilt`'s,
  and three or four in `mem-second-4g`'s and `mem-second-8g`'s,
  so "the same other guests" is true of no pair here;
- the two cells that differ in the guest's size and share a route to the binaries,
  `mem-second-4g` and `mem-third-8g-prebuilt`,
  ran two hours and five minutes apart,
  their peaks at 01:56:18Z and 04:01:26Z,
  and their test binaries' test counts differ (1354 and 1356 filtered out),
  so they do not even share a crate revision.

What the cells do support is a reading per option:
the lock arm passes from binaries built outside the guest,
a guest that builds the crate used 7.29 GiB of its own memory for the build,
and the drops returned 0,
2.28,
1.25,
1.23,
0 and 0 GiB with no mechanism named.

## The decisions, each on its own

### Free page reporting: decided, and shipped

Decided by the owner on 2026-11-25T18:58-05:00
(`doc/decision/mvm-guests-report-free-pages.md`),
and in `mvm`'s source since commit `269b812807`:
every libvirt guest `mvm` creates asks for the reporting balloon.
Its saving is smaller than the charge difference that led to it,
and that difference is a difference of charge,
not of host RAM:
the decision's own `Why` compares 7.98 GiB against 5.52 GiB
(`mem-a` at rest and `mem-b` at rest),
and charge counts swapped pages at the size the guest swapped them.
Counted at the size zram held them at the same sample:

- `mem-b`,
  0.20 GiB resident plus 5.32 GiB swapped at the device's 13.25 percent,
  is about 0.9 GiB of host RAM;
- `mem-a`'s file holds no zram counters,
  so its compressed part is estimated at the 3.3 percent the device held at
  2026-11-25T22:00:41+00:00,
  inside that guest's own window,
  and 0.98 GiB resident plus about 0.23 GiB gives about 1.2 GiB.

So the pair differs by about 0.3 GiB of estimated host RAM,
not 2.46 GiB,
and neither guest's compressed share was measured.
The decision record's measured sentence needs replacing with this one,
which is also the sentence the record should cite from here:

```text
# doc/decision/mvm-guests-report-free-pages.md: the measured sentence, corrected
Measured once each on this host (libvirt 12.9.0, QEMU 11.1.3) with an Ubuntu 24.04 guest after one build of
the git wrapper: the default guest's charge, its QEMU process's `Rss` plus `Swap`, stayed at 7.98 GiB at rest
and the guest with reporting on at 5.52 GiB. Charge counts swapped pages at the size the guest swapped them;
counting them at the size zram held them gives about 1.2 GiB for the default guest and about 0.9 GiB for the
guest with reporting on, about 0.3 GiB apart, and both are estimates from one run each under different
pressure. Reporting's saving on host RAM at rest is therefore smaller than that 2.46 GiB charge difference
(`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`, the section "Free page reporting: decided, and
shipped").
```

The section the record cites today,
"Free page reporting on,
the same guest with one domain attribute",
is not a heading of this document;
the section above is the one to cite.

The decision's limits are the ones measured here:
one run each,
a pressure-dependent split,
an unmeasured page-fault cost,
and no return while a build's pages are live.

### Should a driver drop its guest's page cache between its arms?

What it would change:
a driver change,
one command after each arm's evidence is pulled,
in the drivers whose `all` step runs more than one arm
(`doctor-zfs-route-guest.ts:212` and `:215`,
`removal-lineage-proof-guest.ts:417` and `:423`,
`mount-git-worktree-clone-datasets-guest.ts:457`,
`:510`,
`:650` and `:896`,
`start-probe-guest.ts:859`).
No `mvm` code and no arm's steps.
A driver that runs one arm and then destroys its guest has no next step to warm,
so a drop there buys nothing.

What is measured,
one run per guest.
Each line names what the host read at the drop and after it,
and what the guest's own `/proc/meminfo` read in the same window,
because the guest's view is what says whether a fall could be the drop's doing:

- `mem-b`,
  a default guest with reporting on,
  5.52 GiB (0.20 resident,
  5.32 swapped) at 23:01:00Z,
  with `MemAvailable` 6.75 GiB and `some avg10` 0.00;
  3.24 GiB (0.25 resident,
  2.99 swapped) at 23:01:46Z,
  so 2.28 GiB less,
  all of it from the swapped part,
  with the resident part up 0.05 GiB.
  The sample after the drop read `MemAvailable` 0.73 GiB with `some avg10` 1.20.
  The guest's own log ends at 22:56:01Z,
  five minutes before the drop,
  so this fall has no guest-side reading.
- `mem-second-8g`,
  a default guest that built the crate,
  6.69 GiB at 02:26:47Z with `MemAvailable` 13.12 GiB and `some avg10` 0.04;
  5.44 GiB at 02:27:22Z,
  so 1.25 GiB.
  In that window the guest's own `Cached` fell 0.44 GiB and its `AnonPages` 1.93 GiB;
  the cache is the part a drop frees,
  and no mechanism is named here for the rest.
- `mem-second-4g`,
  a 4 GiB guest on container-built binaries,
  3.23 GiB at 01:58:28Z;
  2.00 GiB at 01:58:40Z,
  so 1.23 GiB,
  while the guest's own `Cached` fell 1.40 GiB in the five seconds before it.
  The charge had already fallen 0.89 GiB from 4.03 GiB at 01:57:42Z,
  as `MemAvailable` went from 1.40 GiB to 0.00 GiB at 01:57:54Z
  with `some avg10` 8.52 at 01:58:05Z.
- `mem-third-8g-prebuilt`,
  a default guest on container-built binaries,
  6.17 GiB at 04:01:37Z with `MemAvailable` 12.57 GiB and `some avg10` 0.03;
  6.17 GiB at 04:01:48Z,
  04:01:59Z and 04:02:10Z,
  so nothing,
  while the guest's own `Cached` fell 3.67 GiB and its `AnonPages` 1.42 GiB.
- `mem-a`,
  the default guest of the decision's comparison,
  which carried no reporting balloon,
  7.98 GiB (0.98 resident,
  7.00 swapped) at 21:54:43Z
  with `MemAvailable` 17.12 GiB and `some avg10` 0.13;
  7.98 GiB (1.09 resident,
  6.89 swapped) at 21:55:46Z,
  so nothing,
  and the guest's own log ends at 21:51:34Z,
  before the drop.
- `mem-c`,
  a guest with private RAM whose round recorded no reporting balloon:
  3.07 GiB at 23:20:41Z with `MemAvailable` 9.96 GiB and `some avg10` 0.10;
  3.08 GiB at 23:21:02Z,
  so nothing,
  while the guest's own `Cached` fell 1.44 GiB.

Six readings,
one run each:
three returned nothing,
and 2.28,
1.25 and 1.23 GiB came back.
The `MemAvailable` at each drop's own sample ran from 6.75 to 17.12 GiB,
and no ordering holds:
the 2.28 GiB return came at the lowest of those readings,
the 1.23 GiB one at 14.05 GiB,
and `mem-a`'s drop at 17.12 GiB returned nothing.
Nothing here names what decides.

What it costs:
a cold page cache for the next arm in the same guest,
plus the driver change;
in a driver that runs one arm,
a drop before the destroy buys nothing.

Recommendation:
not now.
The returns run from nothing to 2.28 GiB with no mechanism named,
a drop only helps the next arm of the same driver run,
and the cost,
a cold cache,
is certain.
The scope is a driver change in the four drivers named above;
if a return is established later,
each owes one line.

Ranking:
not now > drop in the four drivers that run more than one arm > never.
Not now keeps a measured 2.28 GiB possibility open for a step that costs nothing today,
while the middle option spends a driver change and a cold next arm on a return nothing explains;
the middle option still beats never,
because never throws away the largest reading,
2.28 GiB.

Question for the owner,
ready to paste
(this one is under 1,500 characters,
as every form here is):

```text
# doc/planning/mvm-guest-memory-held-for-a-rounds-life.md: the cache-drop question
Should a driver drop its guest's page cache between its arms?
Not now (Recommended): six drops, one run each. Three returned nothing; 2.28, 1.25 and 1.23 GiB
came back. MemAvailable at each drop's own sample ran from 6.75 to 17.12 GiB, with no ordering,
so nothing here names what decides. A drop only helps the next arm of the same driver run, and
most drivers run one arm and destroy the guest. Pro: costs nothing today. Con: the host keeps a
guest's cache between arms.
Drop in the four drivers that run more than one arm (doctor-zfs-route, removal-lineage-proof,
mount-git-worktree-clone-datasets, start-probe): pro, a drop returned 2.28 GiB once. Con, a driver
change and a cold next arm, for a return nothing explains.
Never: pro, the question closes. Con, gives up the largest reading, 2.28 GiB.
Ranking: not now > drop in those four > never, because not now keeps the reading open at no cost.
```

### Should a driver create, prepare and destroy a guest per arm?

What it would change:
the drivers whose `all` step runs more than one arm
(`doctor-zfs-route-guest.ts:265`,
`removal-lineage-proof-guest.ts:473`,
`mount-git-worktree-clone-datasets-guest.ts:931`,
`start-probe-guest.ts:859`,
all quoted in "How a round uses guests today")
would create,
prepare and destroy a guest for each arm instead of for each `all` step.
No `mvm` code.

What is measured,
one run per guest:

- destroy returns the whole charge:
  the sample after `mem-b`'s destroy holds no QEMU process for it,
  and the destroys kept took 0.85,
  1.03 and 1.17 seconds by their drivers' clocks,
  the last being 0.90 seconds between the two lines of `mvm destroy`'s own log.
- create took 10.7,
  10.5 and 9.78 seconds by the drivers' clocks,
  the last being 9.5 seconds between the two lines of `mvm create`'s own log.
- the preparation each create owes:
  the guest installs packages,
  builds OpenZFS 2.4.4 and installs Rust and `cargo-nextest` from source.
  One preparation was timed,
  284.22 seconds (4 minutes 44 seconds);
  the other cells' preparation logs carry no timestamps,
  so no split of theirs is claimed.
- an arm that compiles pays the build again:
  7 minutes 40 seconds with cargo's default jobs in `mem-second-8g`;
  the arms themselves took 1.13,
  0.57 and 0.39 seconds by their test binaries' summaries.
- `mem-c`'s create and destroy were not kept in the evidence folder,
  so no duration of that route is quoted here.

So one more guest owes about 10 seconds of create plus the one timed preparation of 284.22 seconds,
before any build,
against arms of 0.39 to 1.13 seconds.
No run here measured the gap between two arms of one driver run:
each cell ran one arm,
and the drivers' own logs are not in this folder.

Recommendation:
not now.
A create plus the preparation is about five minutes per extra guest,
an arm is about a second,
and only four drivers would have a next arm to isolate;
no measured gap justifies that price.
The scope is those four drivers;
the rest already destroy their guest after their one arm.

Ranking:
keep one guest per driver run > clone a prepared image per arm > create per arm.
Keeping one guest is measured and costs one preparation per driver run;
cloning a prepared image could cut the 284.22 second preparation that makes per-arm guests dear,
but nothing here measured it;
creating per arm is measured to cost about five minutes per arm it isolates.

Question for the owner,
ready to paste
(this one is under 1,500 characters):

```text
# doc/planning/mvm-guest-memory-held-for-a-rounds-life.md: the guest-per-arm question
Should a driver create, prepare and destroy a guest per arm instead of per driver run?
Keep one guest per driver run (Recommended): pro, the preparation is paid once per run and the arms
are 0.39 to 1.13 s. Con, the guest's charge stays between arms.
Clone a prepared image per arm: pro, it could cut the 284.22 s preparation that makes per-arm guests
dear. Con, unmeasured, and it needs a prepared image and a driver change.
Create per arm: pro, every arm starts fresh and the charge returns at each gap. Con, about 10 s of
create plus the 284.22 s preparation per arm, plus 7 m 40 s if that arm builds the crate.
Ranking: keep one guest > clone a prepared image > create per arm, because keeping is measured and
cheapest, cloning could beat it but is unmeasured, and creating per arm is measured to cost minutes.
```

### Should the crate's test binaries be built outside the guest?

What it would change:
a container build on the host,
in an image whose glibc is the guest's,
whose binaries are placed at the absolute path each driver uses;
and the drivers' own build steps,
one per driver path,
with each driver's features,
target directory and profile.
No `mvm` code.

What is measured:

- the same lock arm passed in two guests on the container's binaries,
  in 1.13 and 0.39 seconds by their test binaries' summaries,
  0.75 seconds by `mem-third-8g-prebuilt`'s driver clock,
  each with all 15 marker lines
  (`mem-second-4g-lock-arm.log.txt`,
  `mem-third-8g-prebuilt-lock-arm.log.txt`),
  and the binary that ran in `mem-third-8g-prebuilt` began at the container's own path
  (`mem-third-8g-prebuilt-guest-unpack.log.txt`);
  no driver was run this way,
  and one arm of one driver is what these two runs cover;
- the guest that built the crate put 7,641,948 kB (7.29 GiB) of its own `AnonPages` at its peak,
  against a 7,755 MiB `MemTotal`,
  with `MemAvailable` down to 88,740 kB (86.7 MiB);
  the two guests that only ran the binaries peaked at 1.48 and 1.47 GiB of their own `AnonPages`.
  These are one run each;
  the guest's own count is the guest's accounting and the two prebuilt readings differ by 0.01 GiB,
  and no spread of repeated runs was measured;
- the host's charge does not separate the two routes:
  `mem-third-8g-prebuilt`'s 6.17 GiB was reached at 04:01:26Z,
  before its binaries were pushed at 04:01:43Z,
  so it is the preparation's footprint,
  and the two 8 GiB guests' charges (7.98 GiB building,
  6.17 GiB not building) are two runs
  two hours apart under unlike pressure.
  No saving of host charge is therefore claimed for this option;
- the container build took 289 seconds here,
  and its own cgroup peaked at 9,651,453,952 bytes (9.0 GiB);
- the source tree and the two binaries moved into the guest in 2.17 seconds:
  a 12,994,872-byte source tarball,
  a 31,524,648-byte wrapper and a 17,218,360-byte test binary
  (61.7 MB in all),
  with both binaries starting at the end of it.

What it costs:
a container build per commit or per round,
which charged this host up to 9,651,453,952 bytes (9.0 GiB) of RAM in its own container
and took 289 seconds;
one build path per driver;
each driver's own features,
target directory and profile;
a first run per driver that reproduces those flags,
because the measured cells carried neither the drivers' `--features test-only-switches` nor their job limits;
and the arms whose subject is the in-guest build,
which lose their subject.
The container's cgroup peak and a guest's charge are different quantities,
so they are not added and no net host effect is claimed:
the guest-side saving is what is unmeasured.
The alternative route,
a nextest archive with `--workspace-remap`
(the section "What can place them where the guest finds them"),
needs the crate's 42 `env!` path sites changed and is not measured;
it is a separate decision,
not a variant of this one.

Recommendation:
yes,
for arms whose subject is not the crate's own build.
The measured reason is the arm's pass:
the lock arm ran from binaries built outside the guest in two guests,
and the one guest that built the crate spent 7 m 40 s and 7.29 GiB of its own memory on the build.
The scope is every arm whose subject is not the build,
which the lock arm shows is not empty:
its subject is a lock's lifetime in a ZFS dataset.

Ranking:
build outside at the same path > keep building in the guest.
Building outside keeps every measured arm running and removes the build from each guest's life,
at the price of a 289 second container with a 9.0 GiB peak and a first run per driver;
keeping the build inside costs nothing new but leaves a 7 m 40 s,
7.29 GiB build in every guest that runs an arm,
and its memory benefit cannot be measured without the change.

Question for the owner,
ready to paste
(this one is under 1,500 characters):

```text
# doc/planning/mvm-guest-memory-held-for-a-rounds-life.md: the build-location question
Should the crate's test binaries be built outside the guest, in an image whose glibc is the guest's,
and placed at the absolute path each driver uses?
Build outside (Recommended), for arms whose subject is not the crate's build: pro, the lock arm passed
in two guests on such binaries, in 1.13 s and 0.39 s, and the guest that built the crate spent 7 m 40 s
and 7.29 GiB of its own memory on it. Con, a container build of 289 s and a 9.0 GiB host peak per
commit or round, one build path per driver, each driver's flags, a first run per driver to prove them,
and no arm whose subject is the in-guest build. One arm of one driver was measured; the rest is
extrapolated.
Keep building in the guest: pro, nothing changes and every arm keeps its subject. Con, every guest
owes a 7 m 40 s build and 7.29 GiB, and the saving stays unmeasured.
Ranking: build outside > keep inside, because outside keeps measured arms running and takes the build
out of every guest, while inside leaves it in each one.
```

### Should mvm's guests be smaller than 8,192 MiB?

What it would change:
`DEFAULT_MEMORY_MIB` in `package/mvm/cli/src/config.ts:38`,
and every driver's expectations,
because every arm here ran in a guest of the default size or larger.
A per-guest flag with a `meta.json` field would be a separate change,
not part of this question.

What is measured,
one run per guest:

- the one guest that built the crate used 7,641,948 kB (7.29 GiB) of its own `AnonPages`
  of a 7,755 MiB `MemTotal`,
  with `MemAvailable` down to 88,740 kB (86.7 MiB).
  That is what the build used with cargo's default jobs,
  not what it needs:
  four drivers cap the compiler's parallelism,
  and no run here built the crate in a guest smaller than 8 GiB;
- the 4 GiB guest on container-built binaries peaked at 4.04 GiB,
  its whole 4,096 MiB request,
  from 01:55:30Z to 01:57:42Z,
  so that reading measures the ceiling and not the guest's need;
- a 4 GiB guest does hold the preparation:
  `mem-second-4g` ran the ZFS binding's recipe with OpenZFS built in the guest
  and stayed up,
  its charge at that same ceiling;
- the 8 GiB guest on the same binaries peaked at 6.17 GiB during its preparation,
  two runs under unlike pressure and at different stages,
  so no difference between the guest sizes is claimed from the two.

The guest that only ran prebuilt binaries peaked at 1,541,700 kB (1.47 GiB) of its own `AnonPages`
over its whole life,
so it used far less than its 7,755 MiB;
no run here gave that guest less.
What would make a smaller default affordable is therefore the build's location,
which the section before asks about,
and this question is answerable without it only as "keep 8,192 MiB".

What it would cost:
every driver changes,
and an arm that builds the crate in the guest would have to be re-measured in the smaller guest,
which no run here did.

Recommendation:
keep 8,192 MiB.
No run here shows a smaller guest building the crate,
the only smaller guest that ran an arm was fed binaries built outside it,
and the measured saving of a smaller guest is not established.
The scope is every guest `mvm` creates today,
which the default covers.

Ranking:
keep 8,192 MiB > 4,096 MiB.
Keeping costs nothing and leaves every arm runnable;
a 4,096 MiB default would block every arm that builds the crate in the guest,
and its memory saving is unmeasured for the guests that would gain it.

Question for the owner,
ready to paste
(this one is under 1,500 characters):

```text
# doc/planning/mvm-guest-memory-held-for-a-rounds-life.md: the guest-size question
Should mvm's default guest memory stay at 8,192 MiB?
Keep 8,192 MiB (Recommended): pro, every arm stays runnable and nothing changes. Con, a guest that only
runs prebuilt binaries peaks at 1.47 GiB of its own pages, so 8 GiB is more than it uses.
4,096 MiB for every guest: pro, the preparation fits (a 4 GiB guest ran it, its charge at 4.04 GiB, its
ceiling). Con, an arm that builds the crate in the guest would need re-measuring in the smaller guest,
every driver changes, and the saving is unmeasured (the 8 GiB guest on the same binaries peaked at
6.17 GiB, a different run and stage).
Ranking: keep 8,192 MiB > 4,096 MiB, because keeping leaves every arm runnable at no cost, while 4,096
blocks every in-guest build for a saving nothing measured.
```

### Should the host turn KSM on so a round can measure it?

What it would change:
a host setting (`/sys/kernel/mm/ksm/run`),
which the kernel's KSM documentation says is writable only by root,
and a host where `ksmd` scans while guests run.
A round that wanted merged pages would also need guests whose RAM is private,
which today means the guest-agent route,
because a memfd's page cache is never merged.

What is measured:

- the memfd guests' RAM mapping carries no `mg` flag and is `rw-s` on a memfd;
  `mem-c`'s private mapping is `rw-p`,
  carries `mg`,
  and its bytes are charged to `anon`;
- KSM read 0 throughout every run and merged nothing,
  so the saving is unmeasured;
- the guest-agent route has no `/mnt/shared`,
  through which every driver copies its subject tarball and pulls its logs
  (the section "How a round uses guests today"),
  and that round pushed its snapshot through the guest agent instead of the share;
  the durations of that push are not kept in the evidence folder;
- the kernel's KSM documentation states that KSM "only merges anonymous (private) pages,
  never pagecache (file) pages",
  and that "KSM's scans may use a lot of processing power:
  some installations will disable KSM for that reason."
  It also states what each `run` value does:
  0 stops the daemon and keeps merged pages,
  2 stops it and unmerges every page it merged,
  and 1 runs it,
  so the revert is `2` followed by `0`.

What it would cost:
the host's `ksmd` scans every registered range while it runs,
spending CPU on the whole host;
and the guests that could merge are the guest-agent ones,
which no driver can run today,
so a measurement round owes the drivers another transfer route first.

Recommendation:
no.
The reason is measured and quoted:
the route every guest uses today cannot be merged,
because its RAM is page cache and KSM merges only anonymous pages,
and a round that chose guest-agent guests to merge would have to replace the
`/mnt/shared` transfers every driver depends on first.
So this is not a host switch with a measurement behind it;
it is a driver change followed by a measurement.
The scope is every guest `mvm` creates today,
which is the memfd route unless libvirt finds no `virtiofsd`.

Ranking:
no > yes,
for one measurement round.
No costs nothing and leaves every driver's transfers working;
yes would begin with a driver change every arm needs,
spend CPU host-wide on scans,
and measure a saving that no arm here could use today.

Question for the owner,
ready to paste
(this one is under 1,500 characters):

```text
# doc/planning/mvm-guest-memory-held-for-a-rounds-life.md: the KSM question
Should the host turn KSM on so a round can measure what it merges between guests?
No (Recommended): pro, no root-owned host-wide switch, no ksmd scans, no driver change. Con, the
saving stays unmeasured. Every guest mvm creates today holds its RAM as a memfd, and the kernel's
KSM documentation states that KSM only merges anonymous pages, never page cache.
Yes, for one measurement round: pro, it would measure pages_shared and pages_sharing between two
private-RAM guests. Con, only the guest-agent route has private RAM, and that route has no /mnt/shared,
which every driver copies its tarball and logs through, so the drivers need another transfer route
first; ksmd also scans host-wide (run=1 on, run=2 unmerges, run=0 stops).
Ranking: no > yes, because no costs nothing and yes begins with a driver change every arm needs.
```

## What is not measured, and is owed

- The run-to-run spread of every figure:
  each is one run,
  and the host's pressure changed between all of them.
  Until a spread exists,
  no difference between two runs here is an effect.
- The host-charge effect of building outside the guest:
  only the guest's own `AnonPages` separates the two routes,
  and its two prebuilt readings are one run each.
- A driver run that uses the container's binaries with the driver's own flags:
  the measured cells carried neither `--features test-only-switches` nor the drivers' job limits,
  and no kept log records what they did carry.
- The cache drop on a guest without reporting:
  reporting is unconditional now,
  so that arm cannot be run again,
  and `mem-a`'s single reading is all there is.
- The gap between two arms of one driver run,
  which is what the guest-per-arm question turns on.
- `mem-c`'s create and destroy durations,
  whose logs are not in the evidence folder,
  and the guest-agent push's duration.
- A prepared image cloned per arm,
  which the guest-per-arm question names and nothing measured.
- KSM's saving,
  for the reason under "Should the host turn KSM on so a round can measure it?".
- The page-fault cost of reporting:
  a reported page is gone from the host,
  so the guest's next touch allocates and zeroes a fresh page,
  and no arm here shows it.
- A 6 GiB middle ground,
  large enough to build and small enough to give memory back.
- Windows and Fedora guests,
  and the Hetzner backend.
- The nextest archive route of the section "What can place them where the guest finds them",
  which is documented and not measured on this crate.

## What shipping a choice would change in mvm

- Free page reporting:
  shipped as commit `269b812807`,
  one element in the domain XML,
  and every new guest has it from its next create.
- Building the crate's test binaries outside the guest:
  no `mvm` change at all,
  and a change to every driver under `package/git-policy/cli/e2e/`,
  plus a container build per commit or per round.
- A smaller guest:
  the memory constant in `package/mvm/cli/src/config.ts:38`,
  and a per-guest value would need a flag and a `meta.json` field too.
  It would need the build to move off the guest first,
  because nothing here shows an in-guest build fitting a smaller guest.
- A cache drop between arms,
  or a guest per arm:
  no `mvm` change at all;
  both are changes to the drivers named in their questions.

## Options considered and not measured

### Ballooning the guest down from the host

`virsh setmem` (libvirt's balloon) would make the guest give pages back on the host's command.
It reaches the same discard path as free page reporting
(`balloon_inflate_page` calls `ram_block_discard_range`),
so it would work with the shared backing too,
but it needs the guest's own balloon driver to find the pages,
it leaves the inflated pages unusable to the guest until they are deflated,
and `mvm` exposes no command that would carry it:
a probe would have to run `virsh` itself,
which no agent round here may do.
Free page reporting reaches the same place without asking the guest to surrender pages it might use.

### virtio-mem

QEMU's virtio-mem would let the host add and remove memory from a running guest in 2 MiB blocks.
It needs a virtio-mem device and a guest driver,
and libvirt's domain XML has to name it.
No run here measured it;
it is the one alternative that could shrink a guest's memory while it runs,
which reporting cannot do,
and it is the only alternative that would make the guest's own size
a live number rather than a boot-time one.

### Huge pages for the guest

`<memoryBacking><hugepages>` would make the guest's RAM 2 MiB pages.
QEMU's discard path handles huge pages with `fallocate` as well,
and the host would then have to hold a preallocated huge-page pool,
which is memory held for guests that no other process can use.
Not measured,
and the direction is away from the goal.

### Spreading the ARC instead of the guests

The ARC was not at its floor when the readings were taken:
the opening reading of the 2026-11-25T22:00:41+00:00 sample found it at 2,088,789,208 bytes against a `c_min` of
2,069,377,152,
19,412,056 bytes above the floor,
and `mem-a`'s samples hold it anywhere from 1.55 GiB,
below that floor of 1.93 GiB,
to 13.92 GiB.
So the ARC is not a fixed cost,
but the gate's own sum treats only `size` minus `c_min` as reclaimable,
and nothing in the guests' measurements moves with it.
Not a lever for guest memory.

### Swapping guest memory out under pressure

The host already does this:
one of the guests in the opening reading had 8.03 GiB charged with 7.97 GiB of it in swap.
It is not an option among the others because it is not free:
it costs zram's compression CPU and the guests' latency,
it happens only under pressure,
and swapping does not make the charge go away:
the three guests in the opening reading still held 24.03 GiB charged,
most of it swapped.
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md` records what that costs this machine.
