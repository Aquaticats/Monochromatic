# How mvm guests could hold less of the host's memory

This draft is written by an agent and is unapproved:
it proposes,
it changes no code and no `mvm` default,
and the owner chooses.
It was rewritten after the gate read of 2026-11-25T22:10-05:00.
Every figure here is one run,
and each says so where it appears.
The readings and logs behind them are committed in
`doc/planning/mvm-guest-memory-held-for-a-rounds-life-evidence/`,
whose `README.md` names each file.

## What changed since the gate read of 2026-11-25T22:10-05:00

The cache drop no longer rests on a reason the guests' own readings contradict.
Its evidence is all six kept readings,
and they do not agree:
one default guest kept every byte while its guest freed 5 GiB,
another returned 1.23 GiB with the host's memory free,
and the section "What the charge measures,
and its limits" says what the charge counts and how pressure moves it.
Its recommendation is now "not now".

The smaller-guest question is split in two,
so each can be answered alone:
where the crate's test binaries are built
("Should the crate's test binaries be built outside the guest?")
and how much memory a guest gets
("Should mvm's guests be smaller than 8,192 MiB?").
Each recommendation now names a scope that holds guests today.

The destroy-between-arms numbers are the kept logs' own spans
and this round's timings;
the preparation split that no log held is gone,
and so is the push duration that no log held.

A cell the readings of 2026-11-26T01:50Z to 02:27Z did not measure is measured here:
a default 8 GiB guest running the same arm on the same container-built binaries,
which separates the in-guest build from the guest's size.

The order the cells were ranked in is gone:
the figures are one run each,
the runs differ in pressure,
and no question needs a rank.

KSM's recommendation is now "no",
with the reasons measured and the kernel's own words quoted.

The opening reading's `MemAvailable` is 5,498,576 bytes (5.24 MiB);
an earlier draft printed it as 5.24 GiB.

## What the owner is asked

Five decisions stand,
each with its own section under "The decisions,
each on its own",
its evidence,
a recommendation,
and a compact form for a question dialog.

- Drop each guest's page cache after an arm:
  not now;
  it is one command and its cost is small,
  but the six kept readings disagree:
  nothing returned in the default guest that freed 5 GiB with the host's memory free,
  while a 4 GiB guest under the same free memory returned 1.23 GiB,
  matching the 1.4 GiB of cache it freed.
- Destroy the guest between arms:
  not now;
  destroying returns the whole charge,
  but a create and the preparation cost minutes against arms measured in about a second,
  and no run here measured the share of a round spent between arms.
- Build the crate's test binaries outside the guest,
  in an image whose glibc is the guest's,
  and place them at the same absolute path:
  recommended for arms whose subject is not the crate's own build;
  measured to lower the arm's peak charge from 7.98 GiB to 6.17 GiB,
  and it costs a container build per commit and one build path per driver.
- Keep the guest at 8,192 MiB while an arm builds the crate inside it:
  the build needed 7.29 GiB of the guest's own `AnonPages`,
  and the guest's size was worth 0.66 GiB at most with the build in the guest,
  2.13 GiB once it is out.
- Turn KSM on for a measurement round:
  no;
  the memfd route's RAM is page cache,
  which KSM never merges,
  and the only mergeable route gives up the share every driver's arms use.

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
The machine's `MemAvailable` was 5,498,576 bytes (5.24 MiB),
and the ARC held 2,088,789,208 bytes against a `c_min` of 2,069,377,152,
19,412,056 bytes above its floor
(`mem-b-host-samples.jsonl`,
line 1).

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
- the two cells of 01:50Z and 02:27Z were created after that commit,
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

Nine drivers run that way,
and each builds at a path of its own:

1.  `deferred-origin-check-guest.ts` at `/opt/deferred-origin-check`.
2.  `doctor-zfs-route-guest.ts` at `/opt/git-policy-cli-doctor-route`.
3.  `managed-git-lock-guest.ts` at `/opt/git-policy-cli-lock`.
4.  `mount-git-worktree-clone-datasets-guest.ts` at `/opt/git-policy-cli-adoption`.
5.  `removal-lineage-proof-guest.ts` at `/opt/removal-lineage-proof`.
6.  `sanoid-coverage-guest.ts` at `/opt/git-policy-cli-sanoid-coverage`.
7.  `settling-command-guest.ts` at `/opt/settling-command`.
8.  `slot-state-and-move-refusal-guest.ts` at `/opt/git-policy-cli-slot-state`.
9.  `start-probe-guest.ts` at `/opt/git-policy-cli-start-probe`.

The build each driver runs differs too,
in its own source:
most build with `--features test-only-switches`,
`removal-lineage-proof-guest.ts` and `settling-command-guest.ts` add `--jobs 1`,
`mount-git-worktree-clone-datasets-guest.ts` sets `CARGO_BUILD_JOBS=1`,
`start-probe-guest.ts` builds a second copy into `target/raised-deadline`,
three drivers run the `--lib` tests as well as the `native_binary` ones,
and `sanoid-coverage-guest.ts` builds without `--release`.
So a build placed on the host must reproduce each driver's features,
target directory and profile,
not only its path,
and the arms of three drivers run more than one test binary.

The lock arm this document measures is
`zfs_managed_lock_guest::the_managed_lock_outlives_the_pointer_and_goes_after_the_clone`,
which `managed-git-lock-guest.ts` selects with a nextest filterset for `test(zfs_managed_lock_guest)`:

```sh
# package/git-policy/cli/e2e/managed-git-lock-guest.ts: the arm's own command
cargo nextest run --release --locked --test native_binary --no-capture --status-level all \
  --filterset "test(zfs_managed_lock_guest)"
```

Its subject is a lock's lifetime in a ZFS dataset,
not the crate's build,
so it is an arm the build location can move without losing its subject.

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
  which needs no code change and is what this round measured;
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
  measured,
  this round,
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

Before each create the round's memory gate is read in one command of its own
(`scratch/gate.ts` in the round folder):
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

Two of the three cells were read on 2026-11-26 between 01:50Z and 02:27Z,
and their readings are kept in the evidence folder unchanged.
The third is this round's,
and its step timings come from the driver's own clock
(`step-timings-2026-11-26T03-56Z.txt`),
because the preparation's three logs carry no timestamps.

One of the two earlier cells needed a memory size `mvm` has no flag for,
so that round's worktree carried a temporary block in `package/mvm/cli/src/domain-xml.ts`
that read `MVM_MEMORY_PROBE`,
and the diff is in that round's folder,
outside this repository,
reverted before its commit.
This round's cell needed no block:
it ran `mvm`'s own defaults.

About the earlier cells' gate readings,
stated as the file shows them:
the openings that permitted their creates were at 01:35:10Z and 02:02:06Z,
15 and 8 minutes before the creates at 01:50:43Z and 02:10:07Z,
and two further openings (00:25:01Z and 01:17:26Z) started no guest
(`memory-gate-readings-2026-11-26.txt`).
This round's create follows its gate line in the file directly.

A guest's load is one real arm,
the lock arm,
run as `managed-git-lock-guest.ts` runs it.
The guest that builds builds the crate inside itself with
`cargo build --release --tests --locked` and runs the test through `cargo nextest`;
the guests given the container's binaries are given the same source tree and the two binaries at the
container's own path,
and run the test binary directly with the same environment variables.

After the arm,
each guest drops its page cache (`sync; echo 3 > /proc/sys/vm/drop_caches`),
and the host's samples show what that returns.

## What the charge measures, and its limits

The charge is what the host holds for a guest at one moment:
the QEMU process's `smaps_rollup` `Rss` plus `Swap`,
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
the host reclaims a guest's page cache freely,
because that cache is the host's own page cache over the memfd.
Under pressure the charge therefore falls while the guest still believes it holds the memory.
Measured in `mem-second-8g`:
at 02:15:01Z the charge was 4.70 GiB,
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
and it is the peak the questions turn on.

## The cells, one run each

Three cells were measured for the lock arm,
and the four guests of 2026-11-25T21:14Z to 23:35Z for the rest.

### A default guest that builds the crate inside itself

Guest `mem-second-8g` was `mvm`'s default:
8,192 MiB asked for (`MemTotal` 7,755 MiB),
the virtiofs share,
the shared memfd backing,
and reporting on.
The ZFS binding's recipe prepared it as it prepares the arms' guests today,
and then it built the crate inside itself,
as `managed-git-lock-guest.ts` does:
`cargo build --release --tests --locked`,
which took 7 minutes 40 seconds (`mem-second-8g-guest-crate-build.log.txt`)
and used almost the whole guest:
its own `AnonPages` peaked at 7,641,948 kB (7.29 GiB) of the 7,755 MiB at 02:24:26Z,
with `MemAvailable` down to 88,740 kB.
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
  with three other guests on the host.
  Its swapped part peaked at 2.99 GiB at 02:21:23Z and fell back as the guest faulted pages in.
- the crate build:
  7 minutes 40 seconds.
- the arm:
  0.57 seconds;
  at 02:26:59Z the charge read 6.12 GiB (5.92 resident,
  0.20 swapped).
- the drop:
  the charge read 6.69 GiB at 02:26:47Z (6.45 resident,
  0.24 swapped) and 5.44 GiB at 02:27:22Z (5.24 resident,
  0.19 swapped),
  1.25 GiB less over 35 seconds.
  In the same window the guest's own `AnonPages` fell from 2,072,236 kB to 46,288 kB
  and its `Cached` from 488,648 kB to 26,120 kB at 02:27:02Z,
  so 0.57 GiB of the fall went with the guest's own collapse of anonymous pages,
  the remaining 0.68 GiB came after it,
  and no mechanism is named here for that part.
- destroy:
  1.03 seconds (02:27:31.250Z to 02:27:32.278Z).

The guest's own view at the end is worth stating:
`MemTotal` 8,131,780 kB,
`MemFree` 7,769,540 kB,
`Cached` 26,080 kB,
`AnonPages` 46,648 kB,
while the host still held 5.24 GiB resident for its mapping
(`mem-second-8g-guest-drop.log.txt`).
The host had not discarded that memory by the time the guest was destroyed.

### A 4 GiB guest on container-built binaries

Guest `mem-second-4g` was `mvm`'s own create with a 4 GiB memory size,
4,096 MiB,
and the reporting balloon (`MemTotal` 3,915 MiB in the guest).
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
  during the preparation's toolchain step,
  with three other guests on the host.
  The guest's own view did not peak there:
  its `AnonPages` reached 1,555,568 kB (1.48 GiB) at 01:57:51Z,
  with `MemAvailable` 1,973,592 kB (1.88 GiB).
  So the host held the guest's whole 4 GiB while the guest called about half of it available;
  the difference is the guest's page cache,
  which the guest counts as reclaimable and the host holds.
- the arm:
  1.13 seconds;
  at 01:58:28Z the charge read 3.23 GiB (2.92 resident,
  0.31 swapped).
- the drop:
  the charge read 2.00 GiB at 01:58:40Z,
  11 seconds after the 3.23 GiB sample
  (1.90 resident,
  0.11 swapped),
  and 2.00 GiB at 01:58:51Z (1.86 resident,
  0.14 swapped),
  so the drop returned 1.23 GiB.
  The guest's own `Cached` fell from 1,521,904 kB at 01:58:26Z to 51,288 kB at 01:58:31Z,
  and `MemFree` rose to 3,759,068 kB,
  so the guest freed 1.4 GiB of cache and the host's charge fell 1.23 GiB.
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
  the ZFS binding's recipe with OpenZFS built in the guest.
- the push of the container's source tree and both binaries,
  and their placement at `/opt/git-policy-cli-lock`:
  2.17 seconds for a 12,994,872-byte source tarball and the two binaries,
  with both starting at the end of it.
- the pool:
  0.39 seconds.
- peak charge:
  6.17 GiB at 04:01:37Z,
  all resident and no swap,
  first reached at 04:01:26Z at the end of the preparation and held to the destroy;
  with the host's `some avg10` at or under 0.48 throughout.
- the arm:
  0.75 seconds by the driver's clock,
  0.39 seconds by the test binary's own summary.
- the drop:
  no return.
  The charge read 6.17 GiB at 04:01:37Z,
  04:01:48Z,
  04:01:59Z and 04:02:10Z,
  while the guest's own `Cached` fell from 3,912,288 kB at 04:01:34Z to 59,012 kB at 04:01:49Z
  and its `AnonPages` from 1,541,700 kB to 49,328 kB.
  The guest ended reading `MemFree` 7,756,180 kB of 8,131,780 kB,
  `Cached` 58,648 kB and `AnonPages` 49,480 kB,
  so it had freed about 5 GiB by its own account,
  and the host still held 6.17 GiB resident for its mapping 23 seconds after the drop,
  until the destroy.
- destroy:
  1.17 seconds by the driver's clock,
  0.90 seconds between the two lines of `mvm destroy`'s own log (04:02:18.686Z to 04:02:19.581Z).

### The four guests of 2026-11-25T21:14Z to 23:35Z

Guests `mem-a` to `mem-d` were one throwaway `mvm` guest each,
destroyed after their readings were pulled.

- `mem-a`,
  reporting off,
  one wrapper build:
  240 host samples between 21:14:34Z and 22:10:05Z,
  of which 178 hold its QEMU process;
  peak charge 8.16 GiB at 21:47:19Z (3.71 resident,
  4.45 swapped);
  7.98 GiB before its drop at 21:54:43Z (0.98 resident,
  7.00 swapped);
  and 7.98 GiB after it at 21:55:46Z (1.09 resident,
  6.89 swapped),
  so the drop returned nothing.
  `some avg10` reached 53.54 and was above 1 in 83 of its 178 samples;
  `MemAvailable` was at or under 64 MiB in all 178.
  Its guest-side log ends at 21:51:34Z,
  before the drop,
  so no guest-side reading of the drop is kept,
  and its file holds no zram counters.
- `mem-b`,
  reporting on,
  one wrapper build:
  183 host samples between 22:00:41Z and 23:09:46Z;
  peak charge 8.20 GiB at 22:46:32Z (6.48 resident,
  1.72 swapped);
  at rest 5.52 GiB at 23:01:00Z (0.20 resident,
  5.32 swapped),
  the device holding 13.25 percent;
  after the drop 3.24 GiB at 23:01:46Z (0.25 resident,
  2.99 swapped),
  the device holding 8.73 percent.
  So the drop returned 2.28 GiB,
  all of it from the swapped part,
  while the resident part rose 0.05 GiB.
  `some avg10` reached 24.94 and was above 1 in 68 of its 182 samples;
  `MemAvailable` was at or under 64 MiB in all of them.
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
  and its drop returned nothing:
  3.07 GiB before,
  3.08 GiB after,
  while the guest's own `Cached` fell from 1,564,048 kB to 59,248 kB.
  The price of the route is also visible:
  `mvm push` of the 12.6 MB snapshot took 11 seconds through the guest agent where virtiofs took 3,
  and the guest has no `/mnt/shared`,
  which every arm driver's `guestStart` and `guestFinish` use.
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
under unlike pressure,
and no rank is drawn from them:

- the heaviest host pressure was in `mem-a`
  (`some avg10` 53.54,
  `MemAvailable` at or under 64 MiB in all 178 samples),
  and the lightest in `mem-third-8g-prebuilt`
  (`some avg10` 0.48 at most,
  `MemAvailable` minimum 9.08 GiB);
- `mem-a` ran without a reporting balloon,
  which the decision comparing it states,
  and the two cells of 2026-11-26T01:50Z and 02:27Z ran with one;
  no kept file records a guest's domain XML,
  and `balloon-check.txt` is the one guest whose own view of its devices is kept;
- `mem-c` held private RAM and the rest shared memfd RAM;
- the other rounds' guests numbered two in `mem-third-8g-prebuilt`,
  two to four in the two cells of 2026-11-26T01:50Z and 02:27Z,
  and up to four on 2026-11-25T21:14Z,
  so "the same other guests" is true of no pair here;
- the two cells that differ in the guest's size and share a route to the binaries,
  `mem-second-4g` and `mem-third-8g-prebuilt`,
  ran two hours and five minutes apart,
  their peaks at 01:56:18Z and 04:01:37Z.

What the cells do support is a reading per option,
which is what the questions use.

## The decisions, each on its own

### Free page reporting: decided, and shipped

Decided by the owner on 2026-11-25T18:58-05:00
(`doc/decision/mvm-guests-report-free-pages.md`),
and in `mvm`'s source since commit `269b812807`:
every libvirt guest `mvm` creates carries the reporting balloon.
Its saving is real and smaller than the charge difference that led to it:
the decision's own `Why` compares 7.98 GiB against 5.52 GiB of charge
(`mem-a` at rest and `mem-b` at rest),
and charge counts swapped pages at the size the guest swapped them.
Counting them at the size zram held them,
which is the host's RAM,
gives about 0.9 GiB for `mem-b`;
`mem-a`'s file holds no zram counters,
so its share is estimated from `mem-b`'s at the same sample,
about 0.2 GiB,
and that estimate is not a measurement.
The saving is therefore somewhere near 0.7 GiB of host RAM at rest in that pair,
not 2.46 GiB,
and the report accompanying this round proposes the corrected sentence for the decision record.

The decision's limits are the ones measured here:
one run each,
a pressure-dependent split,
an unmeasured page-fault cost,
and no return while a build's pages are live.

### Should a round drop its guest's page cache after an arm?

What it would change:
one command in each round's guest,
after the arm's evidence is saved.
No `mvm` code,
no arm's steps.

What is measured,
one run per guest,
in the order the drops ran.
Each line names what the guest's own `/proc/meminfo` read beside the host's charge,
because the guest's view is what says whether a fall could be the drop's doing:

- `mem-b`,
  a default guest with reporting on,
  host out of memory (`MemAvailable` at or under 64 MiB in all 183 samples):
  5.52 GiB before,
  3.24 GiB after,
  2.28 GiB returned,
  all of it swapped pages,
  with the resident part rising 0.05 GiB.
  The guest's own log ends 5 minutes before the drop,
  so its side of this fall has no reading.
- `mem-second-8g`,
  a default guest that built the crate,
  host with memory free (`MemAvailable` 13.12 GiB at the sample before the drop):
  6.69 GiB before,
  5.44 GiB after 35 seconds,
  1.25 GiB.
  In that window the guest's own `AnonPages` fell 2.02 GiB and its `Cached` 0.45 GiB;
  0.57 GiB of the charge went with the anonymous pages,
  the remaining 0.68 GiB came after them,
  and no mechanism is named here for it.
- `mem-second-4g`,
  a 4 GiB guest on container-built binaries,
  host with memory free (`MemAvailable` 14.05 GiB):
  3.23 GiB before,
  2.00 GiB after 11 seconds,
  1.23 GiB,
  and the guest's own `Cached` fell 1.4 GiB in the same 5 seconds,
  which matches the fall and names no cause beyond the drop.
- `mem-third-8g-prebuilt`,
  a default guest on container-built binaries,
  host with memory free (`MemAvailable` 12.57 GiB):
  6.17 GiB before,
  6.17 GiB at 11 and 23 seconds after,
  nothing,
  while the guest's own `Cached` fell 3.66 GiB and its `AnonPages` 1.42 GiB.
- `mem-a`,
  the default guest of the decision's comparison,
  which carried no reporting balloon,
  host thrashing (`MemAvailable` at or under 64 MiB in all 178 samples):
  7.98 GiB before and after,
  nothing,
  and the guest's own log ends before the drop.
- `mem-c`,
  a guest with private RAM whose round recorded no reporting balloon:
  3.07 GiB before,
  3.08 GiB after,
  nothing,
  while the guest's own `Cached` fell 1.5 GiB.

What it costs:
the next step of the same guest starts with a cold page cache,
so an arm whose time is being measured must not follow a drop,
and a guest that is destroyed after its readings pays nothing.

Recommendation:
not now.
The reason is in the two readings taken with the host's memory free:
the 4 GiB guest returned 1.23 GiB,
matching the cache it freed,
and the default guest returned nothing at all,
keeping 6.17 GiB of the host's RAM for a guest that read `MemFree` 7.76 GiB of 8.13 GiB,
freed 5 GiB of its own,
and sat that way for the 23 seconds before its destroy.
Nothing here names what decides between the two,
and the step's return is therefore unestablished,
while its cost,
a cold cache,
is certain.
The scope is every round that keeps its guest after an arm,
which is every round that runs a guest arm today;
if the return is established later,
the step costs one line per round.

Question for the owner,
ready to paste
(this one is 812 characters,
and every form here is under 1,500):

```text
Should a round drop its guest's page cache after each arm?
Not now (Recommended): the six kept readings disagree. With the host's memory free, a 4 GiB guest
returned 1.23 GiB (its own Cached fell 1.4 GiB) and a default guest returned nothing in 23 s while it
freed 3.7 GiB of cache and 1.4 GiB of its own pages and the host kept 6.17 GiB. With the host out of
memory, a default guest's drop removed 2.28 GiB of swapped pages. The step costs a cold page cache for
the next step, so an arm whose time is measured must not follow it, and no mechanism here says when it
returns anything.
Adopt: one `sync; echo 3 > /proc/sys/vm/drop_caches` after each arm's evidence is pulled, accepting that
it may return nothing and that the next step starts cold.
Never: rounds keep no guest-side step, and the memory stays held.
```

### Should a round destroy its guest between arms?

What it would change:
a round's own driver runs create,
preparation and the arm once per arm instead of
once per round.

What is measured,
one run per guest:

- destroy returns the whole charge:
  the sample after `mem-b`'s destroy holds no QEMU process for it,
  and the destroys kept took 0.85,
  1.03 and 1.17 seconds
  (each `mvm destroy` log carries both timestamps).
- create took 10.7,
  10.5 and 9.78 seconds.
- the preparation each create owes:
  the guest that runs the ZFS binding's recipe installs packages,
  builds OpenZFS 2.4.4 and installs Rust and `cargo-nextest` from source.
  The third cell's preparation took 284.22 seconds (4 minutes 44 seconds),
  timed by its driver;
  the two earlier cells' preparation logs carry no timestamps,
  so no split of theirs is claimed here,
  and the 7-minute figures an earlier draft gave for them are gone.
- an arm that compiles pays the build again:
  7 minutes 40 seconds with cargo's default jobs in `mem-second-8g`;
  the arms themselves took 0.57,
  1.13 and 0.75 seconds.
- the saving is the share of a round spent between arms,
  and nothing here measured that share:
  each of the three cells ran one arm,
  so no kept reading holds a gap between two arms.

Recommendation:
not now.
Destroying returns everything the guest held,
but every new guest owes a create and a preparation,
measured in minutes,
against arms measured in about a second,
so a round pays more than it saves unless its gaps are long and its arms cheap.
The scope is every round that runs guest arms today,
all of which keep one guest for the round's arms.

Question for the owner,
ready to paste:

```text
Should a round destroy its guest between arms instead of keeping it for the round?
Keep the guest (Recommended): destroying returns everything, but each new guest owes a create (9.78 to
10.7 s measured, one run each) and the ZFS binding's preparation, which installs packages, builds OpenZFS
2.4.4 and installs cargo-nextest from source (284 s in the one preparation timed by its driver). The arms
measured take 0.57 to 1.13 s, and the crate build an arm may need took 7 m 40 s. No run here measured the
share of a round's time spent between arms, so a gap has to be long to pay that.
Destroy between arms: everything returns at each gap, and every gap pays a create and a preparation.
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
  in 1.13 and 0.75 seconds,
  each with all 15 marker lines
  (`mem-second-4g-lock-arm.log.txt`,
  `mem-third-8g-prebuilt-lock-arm.log.txt`);
- the crate's build is what raises a guest's peak:
  `mem-second-8g`,
  which built inside the guest,
  put 7,641,948 kB (7.29 GiB) of the guest's own `AnonPages` at its peak
  and charged the host 7.98 GiB;
  the same arm on container-built binaries,
  in a guest of the same 8,192 MiB,
  peaked at 6.17 GiB
  (`mem-third-8g-prebuilt`);
- the container build took 289 seconds here,
  and its own container charged 9,651,453,952 bytes (9.0 GiB) at most;
- the source tree and the two binaries moved into the guest in 2.17 seconds:
  a 12,994,872-byte source tarball and the two binaries,
  61 MB in all,
  with both binaries starting at the end of it.

What it costs:
a container build per commit,
which charged this host up to 9,651,453,952 bytes (9.0 GiB) of RAM in its own container and took 289 seconds;
one build path per driver;
each driver's own features and target directory;
and the arms whose subject is the in-guest build,
which lose their subject.
The host's peak is not what a guest holds,
so the two figures beside each other are the guest's peak charge on each route,
not a sum.
The alternative route,
a nextest archive with `--workspace-remap`
(the section "What can place them where the guest finds them"),
needs the crate's 42 `env!` path sites changed and is not measured.

Recommendation:
yes,
for arms whose subject is not the crate's own build.
The reason is measured:
the same arm passed on the container's binaries,
and moving the build out of the guest lowered the peak charge from 7.98 GiB to 6.17 GiB
for a guest of the same size.
The scope is every arm whose subject is not the build,
which the lock arm shows is not empty:
its subject is a lock's lifetime in a ZFS dataset.

Question for the owner,
ready to paste:

```text
Should the crate's test binaries be built once on the host, in an image whose glibc is the guest's, and
placed at the absolute path each guest arm driver uses?
Adopt for arms whose subject is not the crate's build (Recommended): the lock arm passed in 1.13 s in a
4 GiB guest and in 0.75 s in a default guest, each on binaries built in an Ubuntu 24.04 container
at /opt/git-policy-cli-lock, with all 15 marker lines. A guest that builds the crate itself put 7.29 GiB
of its own memory into cargo build and charged the host 7.98 GiB; the same arm on the container's binaries
charged 6.17 GiB. Cost: a container build per commit (289 s here), one build path per driver
(nine today), each driver's own features and target dir, and no arm whose subject is the in-guest build.
Keep building in the guest: nothing changes, and every arm keeps its in-guest build.
```

### Should mvm's guests be smaller than 8,192 MiB?

What it would change:
`DEFAULT_MEMORY_MIB`,
or a per-guest flag with a `meta.json` field,
and every driver,
because a 4 GiB guest cannot hold the crate's build.

What is measured,
one run per cell:

- the build does not fit in 4 GiB:
  `mem-second-8g`'s own `AnonPages` peaked at 7,641,948 kB (7.29 GiB) of its 7,755 MiB,
  with `MemAvailable` down to 88,740 kB,
  so no 4 GiB guest can run an arm that builds;
- with the build kept inside the guest,
  the size was worth 0.66 GiB at most:
  the default guest's charge at its pre-build peak was 4.70 GiB at 02:15:01Z
  (`mem-second-8g-host-samples.jsonl` line 24),
  against the 4 GiB guest's whole-life peak of 4.04 GiB,
  both during the preparation's toolchain step;
- with the build moved out of the guest,
  the size was worth 2.13 GiB:
  the 4 GiB guest on container-built binaries peaked at 4.04 GiB,
  and the default guest on the same binaries at 6.17 GiB;
- a 4 GiB guest does hold the preparation itself:
  `mem-second-4g` built OpenZFS 2.4.4 in the guest,
  and its whole-life peak never passed 4.04 GiB.

What it would cost:
every driver changes,
and an arm whose subject is the in-guest build cannot run at all.

Recommendation:
keep 8,192 MiB,
and treat the size as a follow-on to the build location,
not a decision that stands while arms build in the guest.
The reason is measured:
a guest that runs the crate's build needs 7.29 GiB of its own memory,
so a 4 GiB guest cannot hold that arm at all,
and with the build moved out the size is worth 2.13 GiB,
which the build location's own question decides first.
The scope is every guest `mvm` creates today,
which the default covers.

Question for the owner,
ready to paste:

```text
Should mvm's default guest memory stay at 8,192 MiB?
Keep 8,192 MiB (Recommended): a guest that runs the crate's build needs 7.29 GiB of its own memory
(measured, one run) with MemAvailable down to 85 MB, so a 4 GiB guest cannot hold an arm that builds.
With the build kept inside the guest, the size was worth 0.66 GiB at most; with the build moved out it was
worth 2.13 GiB (4.04 GiB peak in a 4 GiB guest against 6.17 GiB in a default one, one run each).
4 GiB, once the build is moved out: the preparation fits (4.04 GiB peak measured), the host saves
2.13 GiB of peak charge, every driver changes, and no arm whose subject is the in-guest build can run.
```

### Should the host turn KSM on so a round can measure it?

What it would change:
a host setting (`/sys/kernel/mm/ksm/run`),
which the kernel's KSM documentation says is writable only by root,
and a round built to measure merging between two guests of one template.
It would also need a guest whose RAM is private,
which today means the guest-agent route,
because a memfd's page cache is never merged.

What is measured:

- the memfd guests' RAM mapping carries no `mg` flag and is `rw-s` on a memfd;
  `mem-c`'s private mapping is `rw-p`,
  carries `mg`,
  and its bytes are charged to `anon`;
- KSM read 0 throughout every run and merged nothing,
  so the saving is unmeasured;
- the guest-agent route costs the file route:
  11 seconds against 3 for a 12.6 MB push,
  and no `/mnt/shared`,
  which every arm driver's `guestStart` and `guestFinish` use;
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
spending CPU on the whole host,
and the guests that could merge would have to give up the share,
which costs every driver its `/mnt/shared`.

Recommendation:
no.
The reason is measured and quoted:
the route every guest uses today cannot be merged,
because its RAM is page cache and KSM merges only anonymous pages,
and the one route that can be merged is the guest-agent route,
which every arm driver's `guestStart` and `guestFinish` cannot run without `/mnt/shared`.
The scope is every guest `mvm` creates today,
which is the memfd route unless libvirt finds no `virtiofsd`.

Question for the owner,
ready to paste:

```text
Should the host turn KSM on so a round can measure what it merges between guests?
No (Recommended): every guest mvm creates today holds its RAM as a memfd, which is page cache, and the
kernel's KSM documentation states that KSM only merges anonymous pages, never page cache. Measured: the
memfd guests' RAM mapping carries no mergeable flag; the guest-agent route's private mapping does, and
that route costs every driver /mnt/shared and a 11 s push where virtiofs takes 3 s. Turning KSM on is a
root-owned host-wide setting whose scans cost CPU on the whole host (run=1 on, run=2 unmerges, run=0
stops), and no arm here could use it.
Yes, for one measurement round: write 1 to /sys/kernel/mm/ksm/run, run two guest-agent guests at once,
record pages_shared and pages_sharing, then write 2 and 0 to unmerge and stop, and measure what the
guest-agent route costs every driver.
```

## What is not measured, and is owed

- The run-to-run spread of the charge's resident and swapped split:
  every figure is one run,
  and the host's pressure changed between all of them.
- The cache drop on a guest without reporting:
  reporting is unconditional now,
  so that arm cannot be run again,
  and `mem-a`'s single reading is all there is.
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
- The share of a round spent between arms,
  which is what the destroy question turns on.

## What shipping a choice would change in mvm

- Free page reporting:
  shipped as commit `269b812807`,
  one element in the domain XML,
  and every new guest has it from its next create.
- Building the crate's test binaries outside the guest:
  no `mvm` change at all,
  and a change to every driver under `package/git-policy/cli/e2e/`.
- A smaller guest:
  the memory constant in `package/mvm/cli/src/config.ts`,
  and a per-guest value would need a flag and a `meta.json` field too.
  It is only safe together with moving the build off the guest.
- A cache drop or a destroy between arms:
  no `mvm` change at all;
  both are round steps.

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
which this round's rules do not allow.
Free page reporting reaches the same place without asking the guest to surrender pages it might use.

### virtio-mem

QEMU's virtio-mem would let the host add and remove memory from a running guest in 2 MiB blocks.
It needs a virtio-mem device and a guest driver,
and libvirt's domain XML has to name it.
This round did not measure it;
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
the opening reading of the 2026-11-25T22:00:41Z sample found it at 2,088,789,208 bytes against a `c_min` of
2,069,377,152,
19,412,056 bytes above the floor,
and `mem-a`'s samples hold it anywhere from 1.55 GiB,
below that floor,
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
