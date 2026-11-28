# How mvm guests could hold less of the host's memory

## Shelved until the crate split (owner, 2026-11-26T00:20-05:00)

After four revisions and four gate reads,
the fourth read found that no figure here flips a recommendation
but that three of the questions still rest on evidence that does not support them:
the second quotes arm times of a driver outside its scope,
the third recommends a move with a measured host cost (a 9.0 GiB container build) and no measured gain,
and the first pools guests without the reporting balloon with guests that have it.
Asked what should happen to the proposal,
the owner answered "Shelve until the split (Recommended)":
nearly every figure measures guests building today's single wrapper crate,
which the crate split in `doc/planning/cli-git-source-layout.md` replaces with smaller crates.
So no question here goes to the owner,
guests keep `mvm`'s 8,192 MiB default,
and guest memory is measured again once the split's crates build in a guest,
starting from the fourth read's findings.

On 2026-11-28 the owner reopened one of those questions
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"Building the test binaries outside the guest"):
where the crate's test binaries are built.
The section "Should the crate's test binaries be built outside the guest?"
is the one revised,
and every other question in this document stays shelved.

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
Every figure here carries its unit,
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
  only for a round that holds several guests at once;
  the driver's narrowed build now uses 2.88 GiB and 746 s inside a guest,
  against the 7.29 GiB the earlier case rested on,
  and a container build costs about the same compiling time (0.94 to 1.04)
  with 2.86 to 3.63 GiB of this host's memory at its peak
  and a 35 MB push per guest.
- Keep the guest at 8,192 MiB while an arm builds the crate inside it:
  the one guest that built the crate put 7.29 GiB of its own `AnonPages` into the build,
  and no run here shows a smaller guest building it.
- Turn KSM on for a measurement round:
  no;
  the memfd route's RAM is page cache,
  which KSM never merges,
  and the one route that can merge has no `/mnt/shared`,
  which
  every driver's transfers need.

Free page report
ing is decided and shipped
(`doc/decision/mvm-guests-report-free-pages.md`,
commit `269b812807`),
and the section "Free page reporting:
decided,
and shipped" records w
hat it buys and
costs.

## The question

At 2026-11-25T22:00:41Z,
with three other
rounds'
guests ru
nning,
the host charged 24.03 GiB to those three guests
(8.03,
8.01 and 7.99 GiB;
`Rss` plus `Swap` of each QEMU process),
and its own RAM held 6.80 GiB of that charge.
The rest was in the guests' swap,
which the zram devic
e held as 1.13 GiB over 34.43 GiB swapped across everything on the host
(3.3 percent)
at that moment.
The machine's `MemAvailable` was 5,498,576 kB (5
.24 GiB),
and the ARC held 2,088,789,208 bytes against a `c_min` of 2,069,377,152,
19,412,056 bytes abo
ve its floor
(`mem-b-host-samples.jsonl`,
line 1:
its `/proc/meminfo` fields are kB and its ARC fields bytes).

One `mvm` guest asks for 8,192 MiB.
The host must be able to give it,
page for page,
for as long as the guest lives,
because nothing tells the host when a pa
ge stops mattering.
This document measures how much of that memory the host could get back
while rounds still run the
ir guest arms,
and what each way back costs.

## How a guest's memory is backe
d and sized

### The domain mvm writes

`package/mvm/cli/src/config.ts` gives a guest 8,192
MiB and 4 vCPUs,
and no command-line flag changes either:
`DEFAULT_MEMORY_MIB` and `DEFAULT_VCPUS` are read by `package/mvm/cli/src/domain-xml.ts`,
which writes `<memory unit="MiB">8192</memory>` and `<vcpu>4</vcpu>`.

When the share decision is `virtiofs`,
`domain-xml.ts` also adds the share's device and,
beside the memor
y element,
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
  so reporting is n
  ot blocked by `access mode="shared"`:
  nothing in t
  he virtiofs route disables discards.
  `ram_block_discard_disable` is called by vfio,
  vhost-vDPA,
  SEV guests,
  `blkio` block devices and KVM without a synchronised MMU,
  none of which this guest has.
- KSM cannot touch that memory.
  QEMU marks a guest's RAM mergeable when merging is on
  (`backends/hostmem.c` calls `qemu_madvise(..., QEMU_MADV_MERGEABLE)`),
  but the kern
  el's KSM documentation states that KSM
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
# package/git-policy/cli/e2e/managed-git-lock-guest.ts:366-367: the arm's own command
cargo nextest run --release --locked --features test-only-switches --test native_binary \
  --no-capture --status-level all --filterset "test(zfs_managed_lock_guest)"
```

Every line number this document gives names the state of `main` at `9e13d87bbd`
(2026-11-27),
after the rounds of issue 6581 that rewrote the guest drivers to a clean oxlint
and then split three of them:
their shared plumbing lives in `package/git-policy/cli/bin/mvm-guest-*.ts`
and in `e2e/guest-host/`,
the removal arm's two guest scripts in `e2e/removal-lineage-proof-guest-arm-scripts.ts`,
and the arms of the adoption,
start-probe and step-22 drivers in
`e2e/adoption-guest-arms/`,
`e2e/start-probe-arms/` and
`e2e/step-22-arms/`.
Issue 6611's round changed the lock driver's build alone
(`managed-git-lock-guest.ts`).
Before those commits the same sentences pointed at other lines of the same files.
The commands those lines hold,
and the flags the cells differ in,
did not change,
except for that one build.

The measuring cells did not run that command.
The guest that built the crate ran `cargo nextest` without `--features test-only-switches`,
and the two guests given the container's binaries ran the test binary directly with `--nocapture`,
so the cells' flags rest on this document's account and not on a kept log.

Its subject is a lock's lifetime in a ZFS dataset,
not the crate's build,
so it is an arm the build location can move without losing its subject.

Issue 6612's round then left one set of guest-step helpers where three stood:
every shared step is in `package/git-policy/cli/bin/mvm-guest-*.ts`,
`bin/guest-route-*.ts` and `e2e/guest-host/` are deleted,
and the line numbers this document gives for the shared plumbing name
files that no longer exist.
The drivers' own line numbers moved with that round too,
so a reader who wants the text of today reads the section below,
`A guest build's job count, and one set of guest steps`,
which states what each driver builds now and what its build measured.

### A guest build's job count, and one set of guest steps

Issue 6612's round measured the three drivers that built at two jobs,
and the lock driver's build of the one target its arm runs,
in one prepared guest of its own
(`issue-6612-guest`:
`MemTotal` 8,131,780 kB,
7.76 GiB of the 8,192 MiB the guest asks for,
four vCPUs).
Each build's figure is the guest's own `/proc/meminfo` and its largest `rustc`,
sampled every two seconds inside the guest for the whole build;
the sampler and the raw samples are in that round's kept logs,
and these are one run each.

- `slot-state-and-move-refusal-guest` builds `cargo build --release --test native_binary … -j 2`
  and keeps two jobs:
  `AnonPages` peaked at 3,279,656 kB (3.13 GiB),
  the largest `rustc` at 2,817,788 kB (2.69 GiB),
  `MemAvailable` never fell below 4,306,628 kB (4.11 GiB),
  and the build took 6m 07s.
- `deferred-origin-check-guest` built `cargo test --release --lib --no-run … -j 2`
  in 9m 21s,
  reaching `AnonPages` 7,158,060 kB (6.83 GiB),
  one `rustc` at 7,355,484 kB (7.01 GiB)
  and `MemAvailable` 475,396 kB (0.45 GiB).
  It builds one job now,
  and the same step measured again at that job count took 13m 59s,
  reaching `AnonPages` 6,914,812 kB (6.59 GiB),
  one `rustc` at 7,111,408 kB (6.78 GiB)
  and `MemAvailable` 719,948 kB (0.69 GiB),
  with the arm passing as it had at two.
- `forced-act-guest` built the same target at two jobs in 8m 30s,
  reaching `AnonPages` 7,249,568 kB (6.91 GiB),
  one `rustc` at 7,418,336 kB (7.07 GiB)
  and `MemAvailable` 398,408 kB (0.38 GiB).
  It builds one job now,
  and the same step measured again at that job count took 14m 35s,
  reaching `AnonPages` 6,993,068 kB (6.67 GiB),
  one `rustc` at 7,190,092 kB (6.86 GiB)
  and `MemAvailable` 633,212 kB (0.60 GiB),
  with the arm passing as it had at two.
- `managed-git-lock-guest` builds only `--test native_binary`,
  the target its arm starts,
  at one job.
  Issue 6611's round measured the same driver's `--tests` build at that job count:
  22m 22s,
  `AnonPages` 7,019,956 kB,
  one `rustc` at 7,174,296 kB
  and `MemAvailable` 625,704 kB at its lowest,
  and the lib test's compile is the peak the arm never reads.
  This round measured the narrowed build in its own guest:
  `cargo build --release --test native_binary --locked --features test-only-switches --jobs 1`
  finished in 11m 31s
  and reached `AnonPages` 2,970,104 kB (2.83 GiB),
  one `rustc` at 2,698,380 kB (2.57 GiB)
  and `MemAvailable` 4,667,288 kB (4.45 GiB) at its lowest,
  with the same fifteen `LOCK` lines and no `SKIP`,
  and one test passed.
  So the arm's own target costs 3.9 GiB less at its peak and half the time.

The rule these figures give is for a driver that builds the library's test binary at two jobs in a guest of this size.
One `rustc` of that build takes 7.0 GiB of the guest's 7.76 GiB,
so a second has nowhere to go and the job count is one.
A driver that builds a single smaller test target may keep two,
which `slot-state-and-move-refusal-guest`'s 4.11 GiB of headroom shows.

### How a round uses guests today

Every line number in this section resolves against the same `main` (`9e13d87bbd`),
and each driver's `all` step is cited as the `async function all` its table hands the run.
A driver whose arms were split names the file of the step it runs,
such as `adoption-guest-arms/crate-build.ts`,
because the driver file no longer holds that step.

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
(`deferred-origin-check-guest.ts:304`,
`doctor-zfs-route-guest.ts:377`,
`forced-act-guest.ts:314`,
`managed-git-lock-guest.ts:488`,
`mount-git-worktree-clone-datasets-guest.ts:309`,
`removal-lineage-proof-guest.ts:413`,
`sanoid-coverage-guest.ts:548`,
`settling-command-guest.ts:491`,
`slot-state-and-move-refusal-guest.ts:405`,
`start-probe-guest.ts:298`,
`zfs-step-22-measurements-guest.ts:206`).
So a guest lives for one driver run,
not for a round's arms,
and a round that runs several drivers creates and destroys one guest per driver.
A guest's charge matters between the arms of one driver run and not after them.

Every driver builds in that guest,
at a path of its own:

- `deferred-origin-check-guest.ts:53` and `:201`,
  at `/opt/deferred-origin-check`;
- `doctor-zfs-route-guest.ts:49` and `:255` to `:257`,
  at `/opt/git-policy-cli-doctor-route`;
- `forced-act-guest.ts:56` and `:211`,
  at `/opt/forced-act`;
- `managed-git-lock-guest.ts:55` and `:358`,
  at `/opt/git-policy-cli-lock`;
- `mount-git-worktree-clone-datasets-guest.ts`'s `adoption-guest-arms/adoption-fixture.ts:21`
  and `adoption-guest-arms/crate-build.ts:71`,
  at `/opt/git-policy-cli-adoption`;
- `removal-lineage-proof-guest-arm-scripts.ts:14` and `:260`,
  at `/opt/removal-lineage-proof`;
- `sanoid-coverage-guest.ts:59` and `:403`,
  at `/opt/git-policy-cli-sanoid-coverage`;
- `settling-command-guest.ts:66` and `:379`,
  at `/opt/settling-command`;
- `slot-state-and-move-refusal-guest.ts:50` and `:253`,
  at `/opt/git-policy-cli-slot-state`;
- `start-probe-guest.ts`'s `start-probe-arms/probe-fixture.ts:21`
  and `start-probe-arms/crate-build.ts:60` to `:61`,
  at `/opt/git-policy-cli-start-probe`;
- `zfs-step-22-measurements-guest.ts`'s `step-22-arms/measurement-fixture.ts:20`
  and `step-22-arms/crate-build.ts:76`,
  at `/opt/step22-measurement`.

The build's flags differ too:
eight drivers pass `--features test-only-switches`,
and three pass none:
`sanoid-coverage-guest.ts:403`,
`forced-act-guest.ts:211` and
`zfs-step-22-measurements-guest.ts`'s `step-22-arms/crate-build.ts:76`.
`sanoid-coverage-guest.ts` builds and runs without `--release`,
`start-probe-guest.ts`'s `start-probe-arms/crate-build.ts:60` to `:61` builds the executable twice,
the second time in `target/raised-deadline`,
and nine drivers cap what they start:

- `removal-lineage-proof-guest-arm-scripts.ts:260`,
  `settling-command-guest.ts:379` and `managed-git-lock-guest.ts:358` with `--jobs 1`,
  each beside a swap file in the guest
  (`:258`,
  `:377` and `:350`):
  one release rustc of this crate's tests takes 6.7 to 6.9 GiB of the guest's 8 GiB
  (`doc/handover/cli-git-rust-implementation.md`,
  its list of rounds),
  and issue 6611's round measured one at 7,019,956 kB of the guest's 7,755 MiB
  while four jobs reached 7,701,208 kB and the guest's killer ended the build;
- `mount-git-worktree-clone-datasets-guest.ts`'s `adoption-guest-arms/crate-build.ts:71`
  and `zfs-step-22-measurements-guest.ts`'s `step-22-arms/crate-build.ts:76`
  with `CARGO_BUILD_JOBS=1`,
  each beside the swap file of the shared bounds
  (`e2e/guest-host/guest-host-bounds.ts:36`,
  `:70` and `:74`),
  for the same one-rustc reason;
- `sanoid-coverage-guest.ts:402` with `CARGO_BUILD_JOBS=2`,
  which builds without `--release`,
  so its two rustcs and their linker stay inside the guest;
- `slot-state-and-move-refusal-guest.ts:253`,
  `deferred-origin-check-guest.ts:201` and `forced-act-guest.ts:211` with `-j 2`,
  each building the one test target its arm starts rather than every target:
  a release rustc of this crate's tests took 6.9 GiB of that guest's 8 GiB,
  and four jobs is what the kernel killed.

`doctor-zfs-route-guest.ts:255` to `:257` and
`start-probe-guest.ts`'s `start-probe-arms/crate-build.ts:60` to `:61`
cap nothing:
the first builds executables and one test target at the compiler's own count,
and the second builds executables alone.
The guest recipe caps its own wrapper builds at `--jobs 2`
(`run-fixture.ts:747`,
and `:1151`,
`:2053` and `:2225` for its other steps)
and OpenZFS's at `make --jobs=4`
(`run-fixture.ts:470` to `:471`).

Two drivers run more than one test binary in their one guest:
`doctor-zfs-route-guest.ts:292` runs `native_binary` through nextest and `:294` runs the lib through `cargo test`,
and `removal-lineage-proof-guest-arm-scripts.ts:284` runs the lib and `:291` the `native_binary`,
both through nextest.
One driver runs four nextest invocations and a Python probe in its guest:
`mount-git-worktree-clone-datasets-guest.ts`'s
`adoption-guest-arms/adoption-arm.ts:73`,
`sweep-arm.ts:116`,
`holder-scan-arm.ts:98` and
`unmount-arm.ts:83`,
beside `store-dataset-and-fsync.ts:175` for the probe.
`zfs-step-22-measurements-guest.ts` runs two more:
`step-22-arms/measurement-arm.ts:129` and
`holder-scan-arm.ts:72`.
One driver runs its own probe steps against the built wrapper and no test binary
(`start-probe-guest.ts`'s `start-probe-arms/`),
and every other driver runs one nextest invocation.
So a build placed on the host must reproduce each driver's features,
path,
target directory and profile,
and a driver that runs two test binaries needs both.

Every driver's transfers go through `/mnt/shared`.
The eight drivers that still unpack the archive themselves name it in the guest
(`deferred-origin-check-guest.ts:192`,
`doctor-zfs-route-guest.ts:247`,
`forced-act-guest.ts:199`,
`managed-git-lock-guest.ts:342`,
`removal-lineage-proof-guest-arm-scripts.ts:233`,
`sanoid-coverage-guest.ts:393`,
`settling-command-guest.ts:368`,
`slot-state-and-move-refusal-guest.ts:237`),
and the three whose arms were split hand it to `pushWorktreeSnapshot`
(`e2e/guest-host/host-side-commands.ts:164`,
which `adoption-guest-arms/crate-build.ts:57`,
`start-probe-arms/crate-build.ts:47` and
`step-22-arms/crate-build.ts:51` call).
A step that runs detached from the command that started it does so through `detachedStep`
(`adoption-guest-arms/detached-step.ts:73`,
`step-22-arms/detached-step.ts:55`)
or through `startDetachedGuestScript` and `awaitDetachedGuestScript`
(`removal-lineage-proof-guest.ts:310` and `:318`,
`settling-command-guest.ts:381` and `:389`).

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

### This round's method, 2026-11-28

This round measured the container side of the build-location route three times
and the route's control arm inside one guest,
under the backend decision of the same day.

- The guest was `mvm`'s own `create` at its default,
  8,192 MiB,
  made by the worktree's built `mvm`
  (`mise run //package/mvm/cli:build`,
  the dist that names `freePageReporting`),
  prepared by the ZFS binding's recipe's own `prepare` step,
  and destroyed by its `destroy` step.
  It was made only while `mvm --backend libvirt list` showed fewer than six guests alive,
  which the project's bound of 2026-11-28 set
  (`gb-i1-inside-lock-gate.log.txt`).
- Its own `/proc/meminfo` was sampled every five seconds by a script pushed into it and started detached,
  so a step that ended the guest would still leave the reading behind
  (`gb-i1-inside-lock-guest-memory.log.txt`).
- The guests' unit was sampled every ten seconds from this host:
  `/proc/meminfo`,
  the unit's `memory.current`,
  `memory.swap.current`,
  `memory.max` and its `max` event count,
  and the guest names in the running QEMU command lines
  (`gb-i1-inside-lock-host-samples.jsonl`).
  A guest's own QEMU process is not a reading of what it is charged,
  because its RAM is a shared memfd and `Rss` plus `Swap` counts swapped pages and the pages mapped at that instant
  (`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
  "The guests' unit at its own memory cap,
  2026-11-28").
- The containers ran as `podman run --rm --cpus 2 --memory 10g --pids-limit 1024 --blkio-weight 10`
  inside their own `systemd-run --user --scope --slice=agent-work.slice`
  with `MemoryMax=12G`,
  `CPUQuota=400%`,
  `TasksMax=1024`,
  `IOWeight=1` and `RuntimeMaxSec=2400`,
  each building this round's own snapshot at the driver's crate path,
  and each reading its own `memory.peak`.
- No 4,096 MiB guest was made.
  `mvm` had no flag for a guest's memory when the round began,
  so the earlier round's 4 GiB guest came from a temporary change to `domain-xml.ts`;
  this round would have copied the built dist and changed `DEFAULT_MEMORY_MIB`,
  which is the mechanism its design asked `mvm` to offer.
  The policy of 2026-11-28 then stopped every local guest,
  and the backend it moved them to sizes a machine by its flavour rather than by a memory request.
  So the smaller-guest question is not measured again here.
- The outside arm ran on a rented machine,
  because the backend decision of 2026-11-28 stopped local guests:
  `MVM_BACKEND=ovh MVM_OVH_FLAVOR=d2-4`,
  with the round's own built `mvm` on `PATH`,
  the backend's own names taken from the environment `mise exec` hands a command in this worktree
  (never printed and never written to a file),
  and the recipe's own `create`,
  `prepare` and `destroy` steps.
  `d2-4` is 2 vCPUs and 4 GiB,
  the smallest flavour the region offered that fits the 4 GiB the question names.
  The project's bound of six machines was read before the create
  (`gb-o1-outside-lock-create-and-destroy.log.txt`),
  and again after it,
  and the machine was destroyed by its name at the end of the arm.
- The round's scripts are its own and live outside the repository:
  a round script that drives the recipe's steps and the driver's commands in order and records each step's clock,
  a snapshot maker that calls the drivers' own `copyWrapperSnapshot`,
  a container build script,
  a host sampler and a guest sampler,
  and a summarizer that reads the round's logs.
  No driver and no `mvm` source was changed.

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

Each cell in this section ran the lock arm in one guest;
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
  Its swapped part peaked at 2.99 GiB at 02:21:23Z and fell back as the guest faulted pages in,
  so the guest's pages went past its 8,192 MiB request
  and the charge sat just under it:
  the figure is bounded by the request,
  not a measure of what the guest needed.
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

### The lock driver's build at one job, and its arm

Guest `git-policy-lock-bounded` was `mvm`'s default,
8,192 MiB asked for (`MemTotal` 8,131,780 kB,
7,755 MiB),
prepared by the ZFS binding's recipe as the arms' guests are.
It then ran `managed-git-lock-guest.ts`'s `run` step as issue 6611's round left it:
a 6 GiB swap file in the guest,
then `cargo build --release --tests --locked --features test-only-switches --jobs 1`.

The same driver ran the same step on a guest of the same recipe before that change,
at cargo's default of one job per vCPU,
four in this guest,
and the guest's own killer ended it.
Its build log reads:

```text
# issue-6611-main-guest-lock-build.log.txt: the four-job build's own reasons
error: could not compile `monochromatic-git-policy-cli` (lib test)
Caused by:
```

The `Caused by` line holds the rustc command line,
34,157 characters of it,
and ends `(signal: 9, SIGKILL: kill)`;
the log's next line is `warning: build failed, waiting for other jobs to finish...`.
The kernel's own line,
and the guest's figures at that moment,
are in `issue-6611-main-guest-oom.log.txt`:

```text
# issue-6611-main-guest-oom.log.txt: the guest's sampler and its kernel at the kill
1795798798 MemTotal:=8131780 MemFree:=141500 MemAvailable:=63928 Cached:=18272 SwapTotal:=0 SwapFree:=0 AnonPages:=7701208 rustc_rss_kb=960704 cargo_rss_kb=116792
[Fri Nov 27 16:59:59 2026] Out of memory: Killed process 149027 (rustc) total-vm:8964104kB, anon-rss:6668028kB, file-rss:2292kB, shmem-rss:0kB, UID:0 pgtables:15928kB oom_score_adj:0
```

That guest had no swap at all (`SwapTotal` 0 kB),
and the killed rustc is the lib test's,
the target this arm never runs.
The host's own `guest-crate-build.log` for that run was empty:
`set -eu` ended the script at the build,
before the `tail` that would have printed the log,
which is why the step now prints it and exits with the build's status.

At one job the same guest's `/proc/meminfo`,
sampled every two seconds for the whole build,
reads `AnonPages` at 7,019,956 kB of 8,131,780 kB at the highest,
`MemAvailable` at 625,704 kB at the lowest,
and one `rustc` at a time at 7,174,296 kB of resident memory at its highest
(`issue-6611-one-job-guest-memory.log.txt`).
The swap file never held more than 7,216 kB of its 6 GiB,
and the build took 22 minutes 22 seconds,
inside the 40-minute bound the step keeps
(`issue-6611-one-job-guest-build.log.txt`).
The arm then ran to its end,
and the log the driver pulled carries all 15 `LOCK` lines and no `SKIP`:

```text
# issue-6611-one-job-lock-arm.log.txt: the arm's own summary line
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1369 filtered out; finished in 0.79s
```

So one release rustc of this crate's tests is the 6.7 to 6.9 GiB the handover names,
and a second one beside it is what this guest cannot hold:
that is what one job and the swap file answer,
and the figure the bound rests on is the guest's own `AnonPages` beside one rustc's resident memory.

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
this section,
"Free page reporting:
decided,
and shipped",
is the one to cite.

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
`removal-lineage-proof-guest-arm-scripts.ts:284` and `:291`,
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
The scope is a driver change in the four drivers "What it would change" names;
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
`removal-lineage-proof-guest.ts:438`,
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
Keeping beats cloning because keeping is measured at one preparation per driver run,
while cloning is unmeasured;
cloning beats creating per arm because cloning aims at the 284.22 second preparation,
while creating per arm is measured to cost about five minutes per arm it isolates.

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
Ranking: keep one guest > clone a prepared image > create per arm, because keeping is measured at
one preparation per run, cloning could cut that preparation but is unmeasured, and creating per arm
is measured to cost minutes per arm.
```

### Should the crate's test binaries be built outside the guest?

The owner reopened this one question on 2026-11-28
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"Building the test binaries outside the guest"),
shelved before it for the crate split.
The owner's chosen words were:

> one round measures the route on the ZFS recipe's guest:
> the test binaries are built once per commit in a container whose glibc is the guest's (Ubuntu 24.04),
> at the same absolute path,
> then pushed into a 4 GiB guest that runs no compiler.
> It returns as a measured design for you to accept;
> no driver changes before that.

Everything else in this document stays shelved until the crate split.
This section is the one revised,
and it is written by an agent:
unapproved and non-final,
it reports what one round measured,
what it could not measure and why,
and ends as a design with the owner's questions.

#### The round's two arms

- Inside,
  the control:
  the ZFS recipe's guest at `mvm`'s default,
  8,192 MiB,
  on this host,
  building the lock driver's test target inside itself with that driver's own flags
  (`--release --test native_binary --locked --features test-only-switches --jobs 1`),
  then running its arm through `cargo nextest`,
  which is the driver's own command.
- Outside:
  the same target built in a rootless `podman` container on this host,
  in an Ubuntu 24.04 image whose glibc is the guest's,
  at the driver's own crate path,
  pushed into a 4 GiB machine that ran no compiler,
  then the same arm started from the binary the container built.

The project decided on 2026-11-28 that all guest work runs on OVHcloud
(`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`,
"All guest work runs on OVHcloud,
2026-11-28"),
so the outside arm ran on a rented machine,
guest `gb-o1`,
and the control arm had run on this host,
guest `gb-i1`,
before that decision reached the round.
The rented machine was flavour `d2-4`:
2 vCPUs and 4 GiB,
Ubuntu 24.04.4 with kernel 6.8.0-136-generic,
which is the release the recipe prepares
(`gb-o1-outside-lock-release.log.txt`,
`gb-o1-outside-lock-create-and-destroy.log.txt`).
That flavour was this round's own choice,
the smallest the region offered that fits the 4 GiB the route was asked about,
at 0.033 Canadian dollars an hour,
rather than the coordinating session's `b3-16` of 4 vCPUs and 16 GiB.
Both machines were destroyed by the round that made them,
and the listing after each destroy names neither
(`gb-i1-inside-lock-create-and-destroy.log.txt`,
`gb-o1-outside-lock-create-and-destroy.log.txt`).

So the two arms ran on two different machines.
Every figure below is a reading of one machine,
and the guest's own `AnonPages` is the only quantity this round compares across them.

#### The control arm, measured once

Guest `gb-i1` was `mvm`'s own `create` with no option:
8,192 MiB asked for,
4 vCPUs,
the virtiofs share and the shared memfd backing,
and `freePageReporting='on'` in the domain `mvm` wrote
(`gb-i1-inside-lock-domain.xml.txt`,
which is the domain file as libvirt kept it:
`<memory unit='KiB'>8388608</memory>`,
`<memballoon model='virtio' freePageReporting='on'>`).
The guest was made by the worktree's own built `mvm`
(`mise run //package/mvm/cli:build`,
the dist that names `freePageReporting`),
and the ZFS binding's recipe prepared it exactly as a driver's `prepare` step does,
OpenZFS 2.4.4 built inside the guest included.

Its steps,
from this host's clock (`gb-i1-inside-lock-steps-2026-11-28T12-15Z.txt`):

- slot:
  1,560 s of waiting for fewer than six guests alive,
  which is where that time went;
- create:
  9.8 s (12:15:41.067Z to 12:15:50.892Z);
- prepare:
  360 s (12:15:51Z to 12:21:51Z);
- the source push:
  0.11 s,
  because a libvirt `mvm push` lands in the host's own virtiofs folder;
- the crate build,
  as one detached script:
  746 s by this host's clock,
  of which the guest's `cargo build` reports 12 m 10 s
  (12:21:52Z to 12:34:03Z,
  `gb-i1-inside-lock-crate-build.log.txt`);
- the pool:
  0.47 s;
- the arm through `cargo nextest`:
  2.376 s by this host's clock,
  and 1.49 s by the test binary's own summary;
- destroy:
  1.9 s.

The arm passed with all fifteen `LOCK` marker lines and no `SKIP` line
(`gb-i1-inside-lock-arm.log.txt`):

```text
# gb-i1-inside-lock-arm.log.txt: the arm's summary line
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1383 filtered out; finished in 1.49s
```

The guest's own `/proc/meminfo`,
sampled every five seconds for its whole life
(`gb-i1-inside-lock-guest-memory.log.txt`,
219 samples),
gives these peaks:

- the preparation (apt,
  then OpenZFS's `make --jobs=4`):
  `AnonPages` 1,542,168 kB (1.47 GiB),
  `MemAvailable` never below 6,023,868 kB (5.75 GiB);
- the crate build:
  `AnonPages` 3,018,988 kB (2.88 GiB) at 12:27:57Z,
  `MemAvailable` down to 4,619,820 kB (4.41 GiB) at the same sample,
  `Committed_AS` 3,511,192 kB;
- the arm and the pool:
  `AnonPages` 66,920 kB in the one sample that caught them,
  `MemAvailable` 7,486,764 kB.

The guest's swap file of 6 GiB,
which the driver's own build step makes,
was not needed:
`SwapFree` never fell below 6,291,452 kB of `SwapTotal` 6,291,456 kB,
so four kilobytes of it were used.

The guests' unit,
read from its own cgroup
(`gb-i1-inside-lock-host-samples.jsonl`,
111 samples every ten seconds),
with what else was alive in it:

- before the create,
  at 12:15:41Z,
  with five other rounds' guests alive:
  `memory.current` 23,389,204,480 bytes and `memory.swap.current` 8,512,380,928 bytes;
- during the preparation,
  at 12:18:53Z,
  with `gb-i1` and five others:
  26,074,988,544 and 8,683,679,744 bytes;
- during the build,
  at 12:27:56Z,
  with `gb-i1` and five others:
  22,157,574,144 and 20,633,591,808 bytes;
- at the highest charge of the guest's life,
  12:34:10Z,
  with `gb-i1` and five others:
  29,492,166,656 and 16,602,726,400 bytes,
  46,094,893,056 in all,
  while the machine's `MemAvailable` read 2,080,968 kB;
- the sampler was stopped 1.6 s after the arm and 2 s before the destroy returned,
  so no sample of the unit after this guest was gone is in the file.

No effect size is claimed from those five readings:
five other guests came and went inside the guest's life,
one of them leaving and another arriving between two samples,
so the unit's charge moved for reasons this run did not cause.
The figure this round can attribute to its own guest is the guest's own `AnonPages`,
which is the guest's accounting of its own anonymous memory.

#### The outside arm, on a rented 4 GiB machine

Guest `gb-o1` was made on the ovh backend,
prepared by the same recipe step the control used,
given the container's source tree and its two binaries at `/opt/git-policy-cli-lock`,
and asked to run the same arm from the test binary itself.
It passed,
with all fifteen `LOCK` marker lines and no `SKIP` line
(`gb-o1-outside-lock-arm.log.txt`):

```text
# gb-o1-outside-lock-arm.log.txt: the arm's summary line
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1383 filtered out; finished in 0.83s
```

The wrapper the test binary starts ran too,
from the container's path:
`monochromatic-git-policy-cli 0.6.1-alpha.52, shadowing git at /usr/bin/git`
(`gb-o1-outside-lock-place.log.txt`).
Nothing was compiled on the machine:
the step that unpacked the two binaries printed
`cargo is not on PATH in this guest`,
and no step of this round ran one there.

Its steps,
from this host's clock (`gb-o1-outside-lock-steps.txt`):

- the listing and its count:
  6.1 s before the create and 4.3 s after it;
- create:
  83 s (13:02:18Z to 13:03:41Z),
  of which the instance was BUILD for 6 s and answered SSH 61 s later;
- the release it booted,
  read from the machine:
  6.8 s;
- prepare:
  1,304.8 s (13:04:01Z to 13:25:46Z),
  the recipe's apt step,
  its OpenZFS 2.4.4 build and its Rust and nextest install,
  on the machine's two vCPUs;
- the push of the 14,807,265-byte source tree and the 20,234,246-byte artifacts tarball:
  13.2 s;
- unpacking both at the crate path:
  6.7 s;
- the pool:
  6.3 s;
- the arm:
  6.567 s by this host's clock for the whole `mvm exec`,
  and 0.83 s by the test binary's own summary;
- pulling the arm's log and the guest's samples:
  4.5 s and 8.5 s;
- destroy:
  64.8 s,
  of which 2.7 s was the delete being accepted and 56 s the listing letting the instance go.

The machine's own `/proc/meminfo`,
sampled every five seconds for its whole life
(`gb-o1-outside-lock-guest-memory.log.txt`,
269 samples),
with `MemTotal` 3,911,564 kB (3.73 GiB):

- the preparation:
  `AnonPages` 1,545,504 kB (1.47 GiB) at 13:24:28Z,
  `MemAvailable` down to 1,937,976 kB (1.85 GiB) at the same sample;
- the push,
  the unpacking,
  the pool and the arm,
  in seven samples:
  `AnonPages` at most 67,120 kB,
  `MemAvailable` never below 3,363,596 kB (3.21 GiB).

So a guest of 4,096 MiB ran the arm from the container's binaries,
and the whole machine's peak was the preparation's 1.47 GiB,
with 1.85 GiB of it to spare.
What the arm itself needs above the machine's idle is under 0.1 GiB.

A second run of the same arm was started on the same flavour and did not reach its end:
`mvm create` stopped while it waited for SSH on the new instance,
the recipe's create step refused,
and the round's own cleanup did not run.
The machine stayed in the project's listing and this round destroyed it by its name
(`gb-o2`,
91 s after the listing permitted the create),
then read the listing again and found none of its own.
No gb-o2 record or step table was written,
so nothing of that attempt is a reading
except the machine's own presence,
which the destroy log and the listing show.
#### The container side of the route, measured three times

Each container ran in its own `systemd-run --user --scope --slice=agent-work.slice`
with `MemoryMax=12G`,
`CPUQuota=400%`,
`TasksMax=1024`,
`IOWeight=1` and `RuntimeMaxSec=2400`,
as `podman run --rm --cpus 2 --memory 10g --pids-limit 1024 --blkio-weight 10`,
and each built this round's own snapshot of commit `f855df5467`
(the drivers' `copyWrapperSnapshot` list,
14,807,265 bytes)
at the path the driver holds its crate at.

- The lock driver's target,
`build --release --test native_binary --locked
  --features test-only-switches --jobs 1`,
  built at `/opt/git-policy-cli-lock`:
  801 s from the container's start to its end,
  of which the `cargo` build is 762 s (11:49:55Z to 12:02:37Z),
  and the container's own `memory.peak` reads 3,073,142,784 bytes (2.86 GiB)
  (`container-build-lock-2026-11-28T11-49Z.txt`).
- The same build repeated on unchanged input,
  at 12:39:54Z:
  736 s from the container's start to its end,
  of which the `cargo` build is 690 s (12:39:54Z to 12:51:24Z),
  and the container's own `memory.peak` reads 3,897,663,488 bytes (3.63 GiB)
  (`container-build-lock-repeat-2026-11-28T12-47Z.txt`).
- The forced act driver's own build,
`test --release --lib --no-run --locked -j 1`,
  which carries no `--features test-only-switches`,
  built at `/opt/forced-act`:
  1,128 s from the container's start to its end,
  of which the `cargo` build is 1,091 s (12:08:00Z to 12:26:11Z),
  and the container's own `memory.peak` reads 8,031,784,960 bytes (7.48 GiB)
  (`container-build-forced-act-2026-11-28T12-07Z.txt`).

The image was `docker.io/library/ubuntu:24.04`,
Ubuntu 24.04.5 with glibc 2.39 (2.39-0ubuntu8.12 once its packages were installed),
and the toolchain was the one the recipe pins,
Rust 1.100.0 and cargo 1.100.0.
Each binary needs at most `GLIBC_2.39`,
which is the guest's own floor,
so the fit has no margin
(`container-artifacts-paths-and-floors-2026-11-28.txt`).

The paths the binaries carry are the ones the guest will hold them at:

- the lock driver's test binary,
  17,682,608 bytes,
  names `/opt/git-policy-cli-lock` 27 times,
  and its wrapper,
  32,547,448 bytes,
  names it 97 times;
- the forced act's library test binary,
  89,659,368 bytes,
  names `/opt/forced-act` 111 times and the lock driver's path not at all;
- neither lock artifact carries this host's checkout path,
  and the one occurrence of `/home/user/Monochromatic` in the test binary
  is a fixture string in the crate's own tests
  (`package/git-policy/cli/src/native/worktree/worktree_clone_domain_tests.rs:86`).

Packed for pushing,
the lock driver's two artifacts are 20,234,246 bytes
and the forced act's one artifact is 32,271,900 bytes.

#### What the numbers change in this question

The proposal's earlier case rested on a build the driver no longer runs.
The one guest that built the crate in the 2026-11-26 cells put 7,641,948 kB (7.29 GiB)
of its own `AnonPages` into `cargo build --release --tests --locked`
at cargo's own job count.
Issue 6612's round narrowed the lock driver to the one target its arm starts,
at one job,
and this round's control run measures that narrowed build:
`AnonPages` 3,018,988 kB (2.88 GiB) of the guest's 7,755 MiB,
with `MemAvailable` never below 4.41 GiB
and the guest's 6 GiB swap file untouched.

The outside route's guest-side saving is now measured,
on a machine of its own:

- the preparation is the same work on both routes and peaks at the same figure,
  1,542,168 kB (1.47 GiB) here and 1,545,504 kB (1.47 GiB) there;
- the crate build is what the inside route adds above it:
  3,018,988 kB (2.88 GiB) against the preparation's 1.47 GiB;
- the arm from the container's binaries adds almost nothing:
  67,120 kB at most across the seven samples that hold the push,
  the unpacking,
  the pool and the arm;
- so the inside route's peak is 1.41 GiB above the outside route's,
  and the outside route's whole machine peak,
  preparation included,
  was 1.47 GiB of a 4 GiB machine,
  with 1.85 GiB to spare.

Two things follow that the earlier case did not have:

- a 4,096 MiB machine runs the arm from the container's binaries,
  preparation and all;
- what the inside route would need of a 4,096 MiB machine is not measured.
  Its peak,
  2.88 GiB,
  is 0.85 GiB under that machine's 3.73 GiB of usable memory,
  and this round did not build inside a machine that small,
  so the question of whether it fits is open rather than answered.

Against the saving,
the container is not free:
it took 762 s and 690 s of compiling in two runs of unchanged input
against the guest build's 731 s on this host's four vCPUs,
a ratio of 0.94 to 1.04 whose two ends are wider apart than the gap between the two routes,
so no difference in compiling time is claimed.
It charged this host 2.86 and 3.63 GiB at its peak in those two runs,
and it moved 20,234,246 bytes of artifacts into the guest on top of the 14,807,265-byte source tree,
which is 2.37 times what the inside arm pushes on its own
and 13.2 s of pushing over `scp` on the rented machine.
The forced act's build costs more:
1,091 s of compiling against the lock driver's 731 s (a ratio of 1.49),
and 7.48 GiB of this host's memory at the container's peak,
for one 89,659,368-byte binary that carries no wrapper beside it,
because `cargo test --lib --no-run` builds the library's test binary alone.

One caveat belongs to every time in this section:
the inside arm ran on this host's machine and the outside arm on a rented one,
so a time from one is not a time from the other.
The container builds and the arm's own `finished in` line are the only times measured the same way on both.

#### What the design would be

Five parts,
each with how it works,
what it stores or changes for good,
and what it gives up.

**Where the container build lives.**
One helper in `package/git-policy/cli/bin/`,
`mvm-guest-test-binaries.ts`,
that takes from a driver its crate path,
its cargo words,
the paths of the executables it expects,
and the run's log folder;
it starts the container in the run's own scope,
waits for it,
packs the executables cargo reported
(`--message-format json`,
whose `executable` fields are the only reliable answer,
since this crate's test binary is written under `target/<profile>/build/<package>/<hash>/out/`
and not under `target/<profile>/deps/`),
and answers with one tarball at their paths relative to the crate root.
It lives where the drivers' shared guest steps already live,
and takes the shape `copyWrapperSnapshot` has:
one list,
called by every driver,
so a new path crate or a new target reaches every caller.
What it stores for good:
a helper and one call site per driver's build step;
and a per-commit artifacts tarball in the run's log folder,
which is what a second arm of the same commit can reuse instead of building again.
What it gives up:
the drivers' in-guest `cargo build` steps,
and with them the last reason a guest needs a Rust toolchain at all.

**Which image.**
`docker.io/library/ubuntu:24.04`,
named by digest in the helper,
with the recipe's pinned Rust 1.100.0 installed inside it the way the recipe installs it
(the same `rustup-init` archive and the same sha256).
What it stores for good:
one image constant,
and a Rust version that must move with
`RUST_TOOLCHAIN` in `run-fixture.ts:150`.
What it gives up:
any margin on libc.
The image's glibc and the binaries' floor are both 2.39,
and the crate needs 2.39 for `pidfd_spawnp` and `pidfd_getpid`,
so a build that picks up one newer versioned symbol would not start in the guest,
and nothing here detects that before an arm fails.

**How the path is kept the same.**
The container extracts the drivers' snapshot at the driver's own crate path
and builds there,
so `env!("CARGO_MANIFEST_DIR")` and `env!("CARGO_BIN_EXE_monochromatic-git-policy-cli")`
hold the path the guest uses.
Measured:
the lock driver's binaries carry `/opt/git-policy-cli-lock` and the forced act's carry
`/opt/forced-act`,
and neither carries this host's checkout.
What it stores for good:
nothing;
no source changes.
What it gives up:
the route only works where the guest's crate path is the container's,
so a guest that mounts the tree elsewhere would need the nextest archive route instead,
with the crate's 42 `env!` path sites changed to reads at run time.

**What each driver's build step becomes.**
The step that ran `cargo build` inside the guest becomes two host steps and one guest step:
the helper's container build,
a push of the source tarball and the artifacts tarball,
and a guest script that extracts both at the crate path.
Measured:
the push of a 14.8 MB source tree took 0.11 s on libvirt,
and the 2026-11-26 cell moved the same shape (a 12,994,872-byte source tarball and two binaries,
61.7 MB in all) in 2.17 s,
so the push is small beside the build it replaces.
What it stores for good:
one artifacts tarball per driver per commit,
in the run's log folder.
What it gives up:
nothing yet on libvirt,
but the transfer itself is a libvirt arrangement:
an OVHcloud guest has no `/mnt/shared`,
`mvm push` there writes the absolute path it is given over `scp`,
and every arm script's log copy would have to name that folder instead.
That is a change to the arm scripts and their log names,
and it is the part of this design that the new backend decision makes mandatory
rather than optional.

**What `mvm` must offer for a smaller guest.**
A memory size (and,
for the same reason,
a vCPU count) per guest:
`mvm create --memory <MiB> [--vcpus <n>]`,
stored in the guest's own `meta.json` so a later step reads it back,
with `DEFAULT_MEMORY_MIB` staying 8,192 for a guest that names neither.
This round made its one guest at the default because no such flag exists:
a smaller one would be made by copying the built dist and changing one constant,
`DEFAULT_MEMORY_MIB=8192` to `DEFAULT_MEMORY_MIB=4096`,
which is exactly the change the flag would carry.
What it stores for good:
one field in `meta.json`,
one flag,
and each driver's choice of size.
What it gives up:
the single default every driver relies on today,
and the guarantee that a guest is big enough for whatever a driver builds in it,
since nothing checks a guest's size against its work.

#### What this round did not measure

- The outside arm,
  in any guest:
  no local guest may be made since 2026-11-28,
  and no OVHcloud guest can run an arm until the arm scripts stop reading `/mnt/shared`.
- The run-to-run spread of the control arm:
  one guest,
  one build,
  one arm.
- The alternation the round was asked for:
  there is no second arm to alternate with.
- A repeat of the forced act's container build:
  its one run is the only reading of that cost.
- Whether a 4,096 MiB guest builds this driver's narrowed target.
  The control guest had 4.41 GiB of `MemAvailable` at its lowest with 8 GiB installed,
  which is not a measurement of the same build in a 4 GiB guest.
- The second driver's arm through the outside route,
  to any point:
  the forced act's container build ran,
  its arm did not.
- The forced act's wrapper:
  `cargo test --lib --no-run` builds no binary,
  so that driver would need a second cargo word if its arm starts the wrapper.
- What an arm costs in wall time on the outside route against this round's control:
  the 2026-11-26 cells' 1.13 s and 0.39 s are of an older revision of the crate,
  whose suite then held 1,355 tests against today's 1,384,
  and are not comparable with today's 1.49 s.

#### The questions for the owner

**Question one:
does the crate's test binaries' build leave the guest?**
The measured build inside an 8 GiB guest uses 2.88 GiB and 746 s,
where the earlier case rested on 7.29 GiB and 7 m 40 s;
a container build costs about the same compiling time (0.94 to 1.04) and 2.86 to 3.63 GiB of this host's memory,
and the arm then runs from 20.2 MB of artifacts pushed beside a 14.8 MB source tree.
On a rented machine the question is what the guest must be:
a 4 GiB flavour ran the arm from the container's binaries with 1.85 GiB to spare,
and no measurement shows the build fitting that flavour.

- Build outside for every arm whose subject is not the build:
  the guest needs the preparation's 1.47 GiB and almost nothing for the arm,
  which a 4 GiB flavour holds (measured),
  so every such arm can run on the smallest flavour the region offers,
  at the price of a container build per commit (11 to 13 minutes here),
  of one build path per driver,
  and of a libc fit with no margin.
- Keep building inside the guest:
  nothing changes and every arm keeps its subject,
  at the price of 2.88 GiB and 746 s in each guest,
  of a flavour large enough for a build whose fit in 4 GiB is unmeasured,
  and of the compiling time being paid on the rented machine,
  where a build would take longer per vCPU than here.
- Build outside only for a round that holds several machines at once:
  the same container and push,
  spent only where the project's six machines or its bill are the scarce thing,
  at the price of two build paths to keep working and of one-guest rounds still paying for the build.

Ranking:
build outside always > build outside only for many machines > keep building inside,
because the measured 4 GiB fit belongs to the outside route and no measurement gives it to the inside one,
a round of one machine is most rounds and so gains most of the saving,
and keeping the build inside pays on a rented machine for a fit nothing measured.

**Question two:
where does the container build live?**
- One shared helper in `package/git-policy/cli/bin/`,
  called by every driver's build step:
  one place to fix,
  one place to add an image or a toolchain,
  at the price of one more argument list per driver (its crate path,
  cargo words and executables).
- A step inside each driver:
  each driver reads whole,
  at the price of the image,
  the toolchain and the packing written out eleven times,
  which is the shape this package's helpers were collapsed out of in issue 6612.
- A published image with the toolchain baked:
  no rustup and no apt inside the build,
  at the price of building,
  tagging and publishing an image per toolchain change,
  which is a package of its own to own and to keep current.

Ranking:
one shared helper > a step per driver > a published image,
because the helper is the shape the package already uses for shared guest steps,
a step per driver repeats the image and the packing eleven times,
and a published image adds a published artifact to maintain for a saving this round did not measure.

**Question three:
which image builds the binaries?**
- `docker.io/library/ubuntu:24.04`,
  by digest:
  its glibc is the guest's,
  which every arm requires,
  measured three times here with the binaries' floor at exactly 2.39,
  at the price of downloading the toolchain inside every build and of no margin on libc.
- The `mvm` template image the guest itself boots:
  its libc is the guest's by construction,
  at the price of that image being a build input too,
  so a template change moves the binaries and the guest together,
  and nothing here measured it.
- Fedora 43:
  a newer glibc,
  so its binaries start in an Ubuntu 24.04 guest,
  at the price of a second distribution's toolchain and its own Rust,
  and nothing here measured it.

Ranking:
Ubuntu 24.04 by digest > the mvm template image > Fedora 43,
because Ubuntu 24.04 is measured and needs no new input,
the template ties the binaries to a guest image whose contents no driver pins,
and Fedora adds a distribution this repository does not otherwise build in.

**Question four:
what must `mvm` offer for a smaller guest?**
- `mvm create --memory <MiB> --vcpus <n>`,
  stored in the guest's `meta.json`,
  8,192 MiB by default:
  measured here as one changed constant in the dist (`DEFAULT_MEMORY_MIB`),
  at the price of one flag,
  one field and every driver's own choice of size.
- Size presets (`--size small|default|large`):
  one word at a call site,
  at the price of a table of sizes nothing measures,
  and of a guest whose size cannot be read back from its own record.
- Nothing:
  every guest keeps 8,192 MiB:
  no change at all,
  at the price of the 4,096 MiB and smaller guests the outside route was meant to allow.

Ranking:
a memory and vCPU flag with a `meta.json` field > presets > nothing,
because the flag carries exactly the change this round measured,
presets hide the number a report needs,
and nothing leaves every future guest at 8 GiB for no reason a measurement gives.

**Question five:
how do the binaries reach an OVHcloud guest?**
The backend decision makes every guest an OVHcloud instance,
and that guest has no `/mnt/shared`,
so the push and the log copies this route relies on must change before any arm runs there.
- Read a folder from the environment in every arm script (`CLI_GIT_GUEST_FOLDER`),
  pushed to by `mvm push` over `scp` at an absolute path,
  and copy logs back from it:
  one variable and a default that keeps libvirt's `/mnt/shared`,
  at the price of touching every arm script that names the folder,
  and of a push that copies bytes over the network instead of into a local folder.
- The nextest archive route,
`cargo nextest archive` then `run --archive-file --workspace-remap`:
  it moves the binaries and their test metadata in one artifact and needs no shared folder,
  at the price of the crate's 42 `env!` path sites being changed to reads at run time,
  and of the guest holding the source at the same revision.
- Keep the arms on libvirt until the transfer route exists:
  no change,
  at the price of every arm being unrunnable from 2026-11-28,
  which is the state the round this section comes from found itself in.

Ranking:
the environment-variable folder > the nextest archive > keeping the arms on libvirt,
because the variable is the smallest change that makes the arms run on the new backend,
the archive is a larger change with a code-wide edit this document cannot verify,
and keeping the arms on libvirt blocks every guest arm the project now has.

Question one,
ready to paste:

```text
# doc/planning/mvm-guest-memory-held-for-a-rounds-life.md: should the crate's test binaries be built outside the guest?
Measured: the build inside an 8 GiB guest uses 2.88 GiB and 746 s (the driver's narrowed --test native_binary build at
one job), against 7.29 GiB and 7 m 40 s that this question rested on; a container build compiles in about the same
time (0.94x to 1.04x over two runs) and peaks at 2.86 to 3.63 GiB of host memory; 20.2 MB of artifacts then go into
the guest beside the 14.8 MB source tree. On a rented 4 GiB flavour the arm passed from those binaries with 1.85 GiB
to spare, and the guest's peak was the preparation's 1.47 GiB.
Build outside for every arm whose subject is not the build (Recommended): pro, the smallest flavour the region offers
runs the arm, and nothing compiles on the rented machine. Con, a container build per commit (11 to 13 minutes here),
one build path per driver, a libc fit with no margin.
Keep building inside: pro, nothing changes and every arm keeps its subject. Con, 2.88 GiB and 746 s per guest, a
flavour big enough for a build whose fit in 4 GiB is unmeasured, and compiling paid on the rented machine.
Ranking: outside always > outside only for many machines > inside, because the measured 4 GiB fit is the outside
route's and every one-guest round gains it.
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
which "Should the crate's test binaries be built outside the guest?"
asks about,
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
- The outside arm's second run:
  its create stopped while it waited for SSH and that machine never ran an arm
  ("Should the crate's test binaries be built outside the guest?",
  "The outside arm,
  on a rented 4 GiB machine").
- A guest smaller than 8,192 MiB running this driver's narrowed build:
  the control guest had 4.41 GiB of `MemAvailable` left at its lowest,
  which is not the same measurement in a 4 GiB guest.
- A driver run of the container's binaries through the driver's own code:
  this round's outside arm ran the container's test binary with the driver's own flags on a rented machine,
  but it ran from the round's own script and not from the driver,
  so the driver's step was not exercised on that route.
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
- The container build's time against the guest build's,
  at more than one run of each:
  two container runs and one guest run are all this round has.
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
