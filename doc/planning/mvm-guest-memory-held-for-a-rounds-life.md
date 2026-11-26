# How mvm guests could hold less of the host's memory

**Non-final draft by an agent,
unapproved.**
It proposes;
it changes no code and no `mvm` default,
and the owner chooses.
This is the second revision,
after the gate review of 2026-11-25;
every figure in this document is one run,
and each says so where it appears.
The readings and logs behind them are committed in
`doc/planning/mvm-guest-memory-held-for-a-rounds-life-evidence/`.

## What changed since the first draft

- Free page reporting is decided:
  every guest `mvm` creates will carry
  `<memballoon model="virtio" freePageReporting="on"/>`
  (`doc/decision/mvm-guests-report-free-pages.md`).
  This document records what that decision buys and costs,
  and no longer offers reporting as an option.
- The smaller-guest option was measured on its real arm:
  the crate's test binaries were built in an Ubuntu 24.04 container on the host,
  placed at the same absolute path in a 4 GiB guest,
  and the lock arm passed there in 1.13 seconds.
  The same arm in a default guest that builds everything,
  the control,
  ran 20 minutes later
  under the same four other guests and peaked at 7.98 GiB against the small guest's 4.04 GiB.
- The rank names the quantity it orders:
  host memory freed,
  with a guest's swapped pages counted at the size zram holds them.
  Counting swapped pages at their full size,
  as the first draft did,
  overstated every option's hold and put them in the wrong order.
- Each remaining decision is its own question with a recommendation,
  and each carries a compact form for a question dialog.
- The evidence is in the repository,
  beside this file.

## The question

On 2026-11-25 at 22:00:41Z,
with one round's guest destroyed and three other rounds' guests running,
the host charged 24.0 GiB to those three guests
(`Rss` plus `Swap` of each QEMU process's RAM mapping),
and its own RAM held 6.8 GiB of that charge.
The rest was in the guests' swap,
and the zram swap device held 1.13 GiB across everything on the host
at that moment
(`previous-round-samples-mem-b.jsonl`,
the `baseline-no-guest` line).
The machine's `MemAvailable` was 5.24 GiB at that moment,
and the ARC held 1.95 GiB,
about 19 MB above its floor.

One `mvm` guest asks for 8,192 MiB.
The host must be able to give it,
page for page,
for as long as the guest lives,
because nothing tells the host when a page stops mattering.
This document measures how much of that memory the host could get back
while rounds still run their guest arms,
and what each way back costs.

## How a guest's memory is backed and sized

### The domain `mvm` writes

`package/mvm/cli/src/config.ts` gives a guest 8,192 MiB and 4 vCPUs,
and no command-line flag changes either:
`DEFAULT_MEMORY_MIB` and `DEFAULT_VCPUS` are read by `package/mvm/cli/src/domain-xml.ts`,
which writes `<memory unit="MiB">8192</memory>` and `<vcpu>4</vcpu>`.

When the share decision is `virtiofs`,
`domain-xml.ts` also adds the share's device and,
beside the memory element,
the backing that device needs:

```xml
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

`mvm` writes no `memballoon` element today,
so libvirt adds its default one:
the running guests' libvirt-written XML holds
`<memballoon model='virtio'>` with no attributes.
That is the element the reporting decision fills in.

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
  but `Documentation/admin-guide/mm/ksm.rst` states that KSM only merges anonymous private pages,
  never page-cache pages,
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

This is the review's first High,
and it changes what the smaller-guest option is.

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
The lock arm this round measured is
`zfs_managed_lock_guest::the_managed_lock_outlives_the_pointer_and_goes_after_the_clone`,
which `managed-git-lock-guest.ts` selects with a nextest filterset for `test(zfs_managed_lock_guest)`:

```sh
cargo nextest run --release --locked --test native_binary --no-capture --status-level all \
  --filterset "test(zfs_managed_lock_guest)"
```

Every driver builds at its own crate path:
`/opt/git-policy-cli-lock` for the lock arm,
`/opt/git-policy-cli-adoption`,
`/opt/removal-lineage-proof`,
`/opt/settling-command`,
`/opt/deferred-origin-check`,
`/opt/git-policy-cli-slot-state`,
`/opt/git-policy-cli-doctor-route`,
`/opt/git-policy-cli-start-probe`.
Some build with `--jobs 1`,
`start-probe-guest.ts` builds a second copy with
`--features start-probe-raised-deadline --target-dir target/raised-deadline`,
and `removal-lineage-proof-guest.ts`,
`settling-command-guest.ts` and `deferred-origin-check-guest.ts`
run the `--lib` tests as well as the `native_binary` ones.
So the guest runs the crate's test binaries,
not only the 28 MB executable,
and one size does not fit every driver's path.

### The host paths the test binaries carry

`env!` writes its value into the binary's text at compile time.
The crate's sources use three such values:

- `env!("CARGO_MANIFEST_DIR")`:
  34 uses in 26 files,
  mostly to read the crate's own manifest,
  `clippy.toml`,
  `SPEC.md`,
  the word-merge corpus and the source tree at run time;
- `env!("CARGO_BIN_EXE_monochromatic-git-policy-cli")`:
  7 uses,
  the absolute path of the wrapper the
  test starts;
- `env!("CARGO_TARGET_TMPDIR")`:
  3 uses,
  a directory inside the target directory.

The full inventory,
with every file and line,
is
`doc/planning/mvm-guest-memory-held-for-a-rounds-life-evidence/compile-time-paths.txt`.
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

A one-file check with rustc 1.102.0 on this host shows the same:

```text
CARGO_MANIFEST_DIR=/original/build/path rustc --edition 2024 \
  --remap-path-prefix=/original/build/path=/remapped ./probe.rs -o ./probe
./probe
manifest=/original/build/path
```

So the only documented way is to give the guest the same absolute path.
This round does exactly that:
the container builds at `/opt/git-policy-cli-lock`,
the guest holds the same source tree and the two binaries at that path,
and the lock arm's driver already uses that path in the guest.
It is a real constraint on any future "build once on the host" plan:
each driver's own `/opt/...` path has to be the path the build used,
so either the drivers share one path or every build is made once per path.

### The glibc floor of the binaries, and the guest's own

`run-fixture.ts` records why it builds inside the guest:
"the host's glibc is newer than Ubuntu 24.04's,
so a host-built binary would not start there;
building in the guest keeps the test binary matched to the guest's own libc and needs no cross toolchain."
That reason is why the first draft's host-built binary was a lucky fit:
both it and the guest reached glibc 2.39,
with no margin.

This round builds where the guest's libc is:
an Ubuntu 24.04 container on the host,
with Rust 1.100.0,
the toolchain version `run-fixture.ts` pins.
What the build printed,
and the floors it produced,
are in
`doc/planning/mvm-guest-memory-held-for-a-rounds-life-evidence/` (`second-round-container-build.txt`):

- Ubuntu 24.04.5,
  glibc 2.39-0ubuntu8.12;
- the wrapper,
  the `native_binary` test binary and the feature variant each need at most `GLIBC_2.39`
  (`readelf --version-info`,
  keeping every `GLIBC_x.y` and taking the last in version order);
- both binaries started and listed tests in an Ubuntu 24.04 container holding the build tree at the
  same absolute path,
  before any guest ran.

The floor equals the guest's,
so the fit has no margin either:
a rebuild that picks up one newer versioned symbol would not start in the guest.
That is a property of the crate's use of `pidfd_spawnp` and `pidfd_getpid`
(`package/git-policy/cli/README.md`:
glibc 2.39 or newer),
not of this build.

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
and destroyed through `mvm` as soon as its readings are saved.
Before each create the round's memory gate is read in one command of its own
(`keep/gate.ts` in the round folder):
`MemAvailable` plus the ARC's `size` minus `c_min` at least 16 GiB,
and memory pressure's `some avg10` at or under its `avg60`.
Every create's gate line is in
`doc/planning/mvm-guest-memory-held-for-a-rounds-life-evidence/second-round-memory-gate-log.txt`.

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

The load in each guest is one real arm,
the lock arm `zfs_managed_lock_guest::the_managed_lock_outlives_the_pointer_and_goes_after_the_clone`,
run as its own driver runs it:
the default guest builds the crate inside the guest with
`cargo build --release --tests --locked` and then runs the test through `cargo nextest`;
the 4 GiB guest is given the same source tree and the two binaries built in the container,
and runs the same test binary directly with the same environment variables.
OpenZFS is built inside both guests by the same `prepare` step,
so the ZFS binding's kernel module and userspace are the guest's own.
After the arm,
each guest drops its page cache (`sync; echo 3 > /proc/sys/vm/drop_caches`),
and the host's samples show what that returns.
The host's charge to a guest is read as
the QEMU process's `smaps_rollup` `Rss` plus `Swap`,
and its RAM mapping is the largest mapping,
named in every sample.

Two measurements need a smaller memory and a domain attribute `mvm` has no flag for,
and `mvm` has no constant for either:
a 4 GiB guest,
and a `memballoon` with `freePageReporting="on"`.
For these guests only,
the worktree's `package/mvm/cli/src/domain-xml.ts` carried a temporary block
that read `MVM_MEMORY_PROBE` (`memory4g`,
`reporting`),
and `mvm` ran from source as `node package/mvm/cli/src/cli.ts`.
The block was reverted before any commit of this revision,
and its diff is in the round folder (`keep/probe-patch-for-the-measurement.diff`),
outside this repository.
No other code and no constant was changed.

## The quantity the rank orders

The rank orders **host memory freed**:
the bytes of the host's RAM that stop being held for a guest while it lives,
counted as

- every resident page of its RAM mapping,
  at its full size,
  and
- the size zram holds for the guest's swapped pages,
  at its compressed size,
  not the size the guest thinks it swapped.

Read as:
`smaps_rollup`'s `Rss` and `Swap` for the guest's QEMU process,
and `/sys/block/zram1/mm_stat`'s `mem_used_total` and `orig_data_size` at the same sample.
The compressed share of one guest's swap is its `Swap` multiplied by the device's
`mem_used_total` divided by `orig_data_size`.
The device's own deltas around a destroy are reported beside that estimate as a check.

The limits of that reading,
stated plainly:

- zram's counters are device-wide.
  Other rounds' guests share the device,
  so the ratio moves with
  their pages;
  over the 47 samples the first round kept,
  `mem_used_total` over `orig_data_size`
  ranged from about 3 percent to about 13 percent.
  The compressed share is therefore an estimate
  from a device-wide ratio,
  not a per-guest measurement.
- a page that is swapped is not resident,
  so `Rss` and `Swap` do not double count;
  but `Swap` is what the guest swapped,
  and the host's zram may hold it more or less compressed.
- resident bytes move with pressure even at the same load,
  so an at-rest figure is a reading at one
  moment,
  not a property of the option.

What the first round's figures become under that reading
(each a one-run reading,
with the ratio from the same sample):

- the default guest at rest (`mem-a`,
  reporting off):
  1.00 GiB resident plus 6.99 GiB swapped;
  at the round's baseline sample the device held 3.3 percent of what it had swapped,
  so about 0.23 GiB compressed,
  and the host held about **1.2 GiB of RAM** for the idle guest,
  not the 7.98 GiB the first draft ranked.
- the same guest with reporting on (`mem-b`) at rest:
  0.20 GiB resident plus 5.32 GiB swapped;
  at that sample the device held 13.2 percent,
  so about 0.70 GiB,
  and the host held about
  **0.9 GiB**.
- after the cache drop (`mem-b`):
  0.17 GiB resident plus 3.07 GiB swapped;
  at that sample the device held 6.7 percent,
  so about 0.21 GiB,
  and the host held about
  **0.38 GiB**.
- the 4 GiB guest at its peak (`mem-d`,
  reporting on,
  no build and no arm,
  from the first round):
  0.66 GiB resident plus 0.10 GiB swapped;
  at that sample the device held 4.4 percent,
  so about 0.004 GiB,
  and the host held about
  **0.67 GiB**.
- this revision's 4 GiB guest after its drop,
  which did run the real arm:
  1.86 GiB resident plus 0.14 GiB swapped;
  the device held 9.7 percent,
  so about 0.014 GiB,
  and the host held about
  **1.88 GiB**.
- this revision's default guest after its drop:
  5.24 GiB resident plus 0.20 GiB swapped;
  the device held 9.7 percent,
  so about 0.019 GiB,
  and the host held about
  **5.26 GiB**.

So counting the swap at its compressed size changes what the first draft's full-swap figures said:
the default guest at rest cost the host about 1.2 GiB of RAM,
not 7.98 GiB,
and reporting's saving at rest is smaller than those figures implied.
What it does not change is the peak:
a guest that compiles holds its live pages resident,
compressed swap or not,
and that peak is what the host has to be able to give at one moment.
The at-rest figures of the first round and of this revision were read under different pressure,
so they are not compared here;
the pair this revision measured is under "The order by that quantity".

### The order by that quantity

Two orders are worth stating,
and only the peak order is comparable across these runs.

At a working peak the guest's own RAM is the cap,
and the pressure of the day does not move it:

1. A 4 GiB guest on host-built binaries:
   4.04 GiB peak charge over its whole life,
   all resident,
   during its preparation,
   and 3.23 GiB at the arm's end,
   measured on the real arm
   in one run.
2. A default 8 GiB guest that builds the crate inside itself:
   7.98 GiB peak charge here,
   one run
   (8.16 GiB in the first round's default guest).
3. An 8 GiB guest with reporting on:
   8.20 GiB peak charge in the first round,
   one run;
   reporting returns nothing while a build's pages are live.
4. A guest destroyed between arms:
   nothing in the gap,
   and the whole charge during an arm.
5. The guest-agent route for KSM:
   nothing measured,
   unranked.

At rest the figures these runs give cannot be ordered against each other,
and the reason is the
quantity's own limit:
resident bytes move with pressure.
Read under the rule above,
the host held about:

- 0.38 GiB for an 8 GiB guest with reporting on after its cache drop (first round,
  one run);
- 0.90 GiB for an 8 GiB guest with reporting on (first round,
  one run);
- 1.23 GiB for the default guest without reporting (first round,
  one run);
- 1.88 GiB for the 4 GiB guest after its own drop (this revision,
  one run).

The first three were read while the host was thrashing and had swapped most of each guest out;
the 4 GiB guest's 1.88 GiB was read with host memory free,
so its pages stayed resident.
The pair this revision did measure is the two guests of today:
after their drops,
the 4 GiB guest held 2.00 GiB charged (about 1.88 GiB of host RAM) and the
default guest 5.44 GiB (about 5.26 GiB),
a bit over half.
The first round's three figures are not comparable with either,
and the first draft's order,
which
put the 4 GiB guest first on a smoke-script figure,
is withdrawn.

What is solid either way:
a 4 GiB guest cannot exceed 4.04 GiB of host RAM at any moment,
measured,
and reporting alone does not lower a build's peak,
measured.

## Results, one run each

Every figure here is one run,
under the host pressure the sample file records.
The run-to-run spread of the resident and swapped split is not measured.
The closest pair is the two guests this revision ran:
the same arm,
the same preparation recipe,
the same four other guests on the host,
20 minutes apart.
That is one run each,
not a spread,
and the figures are read as that.

### The default guest with reporting on, building the crate inside itself

Guest `mem-second-8g` was `mvm`'s default:
8,192 MiB asked for (`MemTotal` 7,755 MiB),
the virtiofs share,
the shared memfd backing,
and the probe block's reporting balloon.
The ZFS binding's recipe prepared it as it prepares the arms' guests today:
packages 71 seconds,
OpenZFS 3 minutes 2 seconds,
toolchain 4 minutes 31 seconds,
8 minutes 44 seconds in all.
It then built the crate inside itself,
as the round's own driver does:
`cargo build --release --tests --locked` with cargo's default jobs on 4 vCPUs,
which took 7 minutes 40 seconds and used almost the whole guest:
its own `AnonPages` peaked at 7.29 GiB of the 7.755 GiB,
with `MemAvailable` down to 85 MB.
`cargo nextest` ran the same lock test:
1 passed,
1354 skipped,
0.58 seconds,
and the pulled log carries all 15 marker lines.

The host's charge to it,
one run (02:10:07Z to 02:27:32Z),
with the same four other rounds' guests
running as during the 4 GiB guest:

- create:
  10.5 seconds.
- peak charge:
  7.98 GiB at 02:25:13Z (7.71 GiB resident,
  0.27 GiB swapped).
  Its swapped part peaked at 2.99 GiB at 02:21:23Z and fell back as the guest faulted pages in.
- the crate build:
  7 minutes 42 seconds from the push's end to its finish.
- the arm:
  0.58 seconds;
  at 02:26:59Z the charge read 6.12 GiB.
- the drop:
  the charge read 5.44 GiB at 02:27:22Z (5.24 GiB resident,
  0.20 GiB swapped),
  about 1.0 GiB less than the 6.45 GiB it read 35 seconds earlier,
  when the build had just ended and the drop had just run.
- destroy:
  02:27:32Z.

Against the 4 GiB guest,
same arm,
same recipe,
same four other guests,
20 minutes earlier:

- peak:
  7.98 GiB against 4.04 GiB.
- arm end:
  6.12 GiB against 3.23 GiB.
- after the drop:
  5.44 GiB against 2.00 GiB,
  or about 5.26 against 1.88 GiB when each guest's swapped pages are counted at the size zram holds
  them (9.7 percent at both readings).

So the 4 GiB guest held about half the host RAM at every point measured,
and the price was the in-guest build,
7 minutes 40 seconds,
replaced by a 6-second push of binaries the host had built.

### A 4 GiB guest on the binaries the container built

Guest `mem-second-4g` was `mvm`'s own create with the probe block's 4 GiB memory size and the
reporting balloon (`MemTotal` 3,915 MiB in the guest).
The ZFS binding's recipe prepared it exactly as it prepares the arms' guests today:
Ubuntu 24.04 packages,
OpenZFS 2.4.4 built in the guest with `make --jobs=4`,
Rust 1.100.0 and cargo-nextest 0.9.152 installed from source.
Then,
instead of building the crate in the guest,
it received the container's source tree and two
binaries at `/opt/git-policy-cli-lock`:
the path the container built them at,
which is also the path the lock arm's driver uses in the guest.
Both binaries started in the guest:
the wrapper printed `monochromatic-git-policy-cli 0.6.1-alpha.46, shadowing git at /usr/bin/git`,
and the test binary listed its tests.
The arm was the same test the control runs,
with the same four environment variables,
run directly
from the test binary with `--nocapture`.

It passed:

```text
test zfs_managed_lock_guest::the_managed_lock_outlives_the_pointer_and_goes_after_the_clone ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1354 filtered out; finished in 1.13s
```

The pulled log carries all 15 `LOCK` marker lines,
so the arm performed every step rather than
returning early (the driver fails a run whose log lacks any marker or holds a `SKIP` line).

The host's charge to it,
one run (01:50:42Z to 01:59:02Z),
with up to four other rounds' guests
running at the same time:

- create:
  10.7 seconds from the create command's first line to the guest agent's answer.
- the guest's whole-life peak charge:
  4.04 GiB,
  all resident and no swap,
  at 01:56:18Z,
  during the preparation's toolchain step (compiling `cargo-nextest` from source).
  The guest's own view did not peak there:
  its `AnonPages` reached 1.48 GiB at 01:57:51Z,
  with `MemAvailable` 1.88 GiB.
  So the host held the guest's whole 4 GiB while the guest called about half of it available;
  the difference is the guest's page cache,
  which the guest counts as reclaimable and the host holds.
- preparation:
  7 minutes 24 seconds (packages 46 seconds,
  OpenZFS 3 minutes 51 seconds,
  toolchain 2 minutes 57 seconds,
  from the three steps' log times).
  A 4 GiB guest does hold the ZFS binding's preparation as it stands,
  OpenZFS included,
  in this run.
- the push and unpack of the 12.7 MB source tarball and the two binaries:
  6 seconds.
- the pool:
  1.4 seconds.
- the arm:
  1.13 seconds;
  at its end the charge read 3.23 GiB (2.92 GiB resident,
  0.31 GiB swapped).
- the drop:
  the charge read 2.00 GiB 11 seconds later (1.86 GiB resident,
  0.14 GiB swapped),
  so the drop returned 1.23 GiB.
  The guest then read `MemFree` 3.58 GiB and `Cached` 50 MB.
  The host still held 1.86 GiB resident after both the drop and reporting,
  which is this run's reading,
  and this round names no mechanism for it.
- destroy:
  the guest's last sample is 01:59:00Z and `mvm destroy` answered at 01:59:02Z,
  with the charge at zero afterwards.

What it costs:
a container build before the guest (18 minutes here,
one per commit),
the same absolute path in every driver,
and each driver builds at its own path today;
the preparation still builds OpenZFS in the guest;
and the in-guest build of the crate is gone,
so no arm whose subject is that build can run this way.

What the run does not settle:
the two guests are one run each,
not a spread,
and the host had four other guests on it;
the paragraph under "The default guest with reporting on,
building the crate inside itself" gives
the pair.

### Free page reporting, from the first round

The first round measured the default guest and the same guest with reporting on,
one run each,
before reporting was decided:

- `mem-a`,
  reporting off:
  7.98 GiB charged at rest after one wrapper build
  (1.00 GiB resident,
  6.99 GiB swapped),
  a peak charge of 8.16 GiB,
  a peak of 5.51 GiB resident,
  and 6.99 GiB of the guest's memory in swap while the guest called 6.3 GiB of its own 7.75 GiB free.
  Dropping the guest's page cache returned nothing.
- `mem-b`,
  reporting on:
  5.52 GiB charged at rest (0.20 GiB resident,
  5.32 GiB swapped),
  a peak charge of 8.20 GiB,
  and 3.24 GiB after the guest dropped its page cache (0.17 GiB resident,
  3.07 GiB swapped).
  The drop returned 2.28 GiB here,
  where it returned nothing without reporting.
  The preparation alone already showed 0.69 GiB returned (2.30 GiB charged against 2.99 GiB).

What the readings show about swapped pages,
and what they do not:
between the samples at 23:01:12Z and 23:01:46Z the charge fell from 5.52 to 3.24 GiB,
the resident part rose from 0.20 to 0.25 GiB,
and the swapped part fell from 5.32 to 2.99 GiB.
So about 2.3 GiB of pages that were in swap were released after the guest
dropped its cache,
without being touched again.
The first draft said reporting cannot return pages the host has already swapped out;
this round's data does not support that,
and the proposal states no mechanism for the 3.07 GiB that stayed.
What is measured is that the drop released both resident and swapped pages in that run.

### The guest without the share, from the first round

`mem-c` was `mvm`'s guest-agent route,
which is what a host without `virtiofsd` gets:
no `filesystem` device and no `memoryBacking`,
`meta.json` reading `"fileTransfer": "guest-agent"`.
QEMU then holds the guest's RAM as `memory-backend-ram` instead of the memfd.

- the RAM mapping is `rw-p`,
  anonymous and private,
  where the shared guests' is `rw-s` on `/memfd:memory-backend-memfd`;
- it carries the mergeable flag:
  `VmFlags` included `mg` on that mapping,
  and 7 of the process's 816 mappings carried it (the memfd guests:
  6,
  never the RAM mapping);
- the session unit charged its bytes to `memory.stat` `anon` rather than `shmem`;
- KSM was off (`/sys/kernel/mm/ksm/run` read 0) and `pages_sharing` 0 throughout,
  so nothing was merged and the saving is unmeasured.
- the price was also visible:
  3.07 GiB charged after the same preparation the others ran
  (2.67 GiB resident,
  0.40 GiB swapped),
  `mvm push` of the 12.6 MB snapshot took 11 seconds through the guest agent where virtiofs took 3,
  and the guest has no `/mnt/shared`,
  which every arm driver's `guestStart` and `guestFinish` use.

KSM is a host change no agent round may make;
the question to the owner is
"Should the host turn KSM on so a round can measure it?".

### Destroying a guest between arms

Both rounds measured the two halves:

- `mvm destroy` returned the whole charge each time.
  For `mem-a` the swap device's free space rose by 7.12 GiB as its pages were released.
  The first round recorded 9,
  8,
  10 and 3 seconds for its four destroys and 11,
  17,
  13 and 16
  seconds for its four creates;
  those timings are in that round's report,
  not in a kept log,
  so
  this revision treats them as indicative only.
- the ZFS binding's preparation,
  OpenZFS included,
  took 8 minutes 44 seconds in the default guest
  and 7 minutes 24 seconds in the 4 GiB guest here,
  one run each.
- the crate build took 7 minutes 40 seconds with cargo's default jobs on 4 vCPUs here,
  one run,
  and 33 to 34 minutes with one job in the first round.

So destroying between arms is cheap only for arms that compile nothing:
the guest that built the crate pays the whole build again,
and destroying before the next arm throws that away.

## The decisions, each on its own

The review's fourth High was that complements were ranked against each other and the document asked
nothing.
These are the separable decisions,
each with the evidence that bears on it,
a recommendation,
and a compact form for a question dialog.

### Free page reporting: decided

Decided by the owner on 2026-11-25
(`doc/decision/mvm-guests-report-free-pages.md`):
every guest `mvm` creates carries the reporting balloon.
The decision's own limits are the ones measured here:
one run each,
a pressure-dependent split,
an unmeasured page-fault cost,
and no return while a build's
pages are live.
This document adds nothing to that decision;
it only measures the option's place among the others.

### Should a round drop its guest's page cache after an arm?

What it would change:
one command in each round's guest,
after the arm's evidence is saved.
No `mvm` code,
no arm's steps.

What is measured:
with reporting on,
one drop returned 2.28 GiB in the first round,
one run,
and without reporting the same drop returned nothing in one run.
In this revision,
the 4 GiB guest's drop returned 1.23 GiB (3.23 GiB charged before it,
2.00 GiB after),
and the default guest's about 1.0 GiB (6.45 to 5.44 GiB),
one run each.
What it costs:
the next arm starts with a cold page cache,
so an arm whose time is being measured
must not follow a drop.

Recommendation:
yes,
as a step after each arm's evidence is saved,
and never before a timed arm.
It buys back the page cache a build leaves behind,
which is most of what remains after an arm,
and its only cost is a cold cache for the next guest step,
which a round can choose to pay or not.

Question for the owner,
ready to paste (under 1,500 characters):

```text
Should a round drop its guest's page cache after each arm?
Adopt (Recommended): after an arm's evidence is pulled, run `sync; echo 3 > /proc/sys/vm/drop_caches`
in the guest. Measured, one run each: with free page reporting on, one drop returned 2.28 GiB
(5.52 GiB charged at rest, 3.24 GiB after); without reporting the same drop returned nothing; in the
4 GiB guest on host-built binaries it returned 1.23 GiB (3.23 GiB before, 2.00 GiB after). Cost: the
next arm starts with a cold page cache, so any arm whose time is measured must run before the drop.
No mvm code changes.
Not now: leave the cache alone; the guest keeps what it built until it is destroyed.
Never: rounds keep no guest-side step, and the memory stays held.
```

### Should a round destroy its guest between arms?

What it would change:
a round's own driver runs create,
preparation and the arm once per arm instead of
once per round.

What is measured:
destroy returns everything the guest held and took 3 to 10 seconds;
create took 11 to 17 seconds;
but the ZFS binding's preparation,
which also builds OpenZFS,
took 74 to 100 seconds in the first round for the smaller part of it and is not measured with OpenZFS in this revision,
and an arm that compiles pays the build again,
33 to 34 minutes in the first round.
The saving is proportional to the share of a round spent between arms,
which this round did not measure.

Recommendation:
destroy between arms only for rounds whose arms compile nothing and whose preparation
is short;
keep the guest for a round whose arms build inside it,
because the rebuild costs more time
than the gap saves.
The first draft ranked this ahead of the KSM route on measuredness;
with the quantity named,
it belongs
where the gap share decides,
and that is not measured.

Question for the owner,
ready to paste (under 1,500 characters):

```text
Should a round destroy its guest between arms instead of keeping it for the round?
Adopt for rounds that compile nothing (Recommended): destroying returns everything the guest held.
Measured here: create 10.5 s and 10.7 s, destroy 2 to 5 s, and the ZFS binding's preparation, OpenZFS
included, 7 m 24 s to 8 m 44 s. A round whose arms do not build pays only the create and that
preparation for each gap. An arm that builds pays the build again: 7 m 40 s with default jobs here,
33 to 34 minutes with one job in the first round.
Keep the guest: no create or preparation per gap, and the whole charge stays held between arms.
Adopt everywhere: every gap returns everything, at the cost of a preparation and a rebuild each time.
```

### Should mvm's guests shrink, with the arms no longer compiling inside them?

What it would change:
`DEFAULT_MEMORY_MIB` (or a per-guest flag),
and every arm driver,
because a 4 GiB guest cannot compile the crate
and each driver builds at its own `/opt/...` path.
It also needs the host to build the crate's test binaries in an image whose glibc is the guest's,
and to place them at the same absolute path,
which this round did once.

What is measured:
the real arm passed in a 4 GiB guest on the container's test binaries,
in 1.13 seconds,
with all 15 marker lines,
one run;
that guest's whole-life peak charge was 4.04 GiB,
all resident,
and 3.23 GiB at the arm's end;
the ZFS binding's preparation,
OpenZFS included,
fitted the same guest in 7 minutes 24 seconds;
and the guest held 2.00 GiB after its drop.
The details are under "A 4 GiB guest on the binaries the container built".

What it costs:
the container build per commit (18 minutes here),
the same-path constraint on every driver's path,
and each driver builds at its own path today;
OpenZFS still built in the guest (with `make --jobs=4`,
which fitted the 4 GiB guest here);
and the in-guest build of the crate,
so every arm whose subject is that build loses its arm.

Recommendation:
a 4 GiB guest halves the host RAM the measured arm holds,
which this round measured,
so it is worth adopting for the arms that do not compile in the guest;
it is not a new default yet,
because every driver builds at its own path,
the arms that build in the guest lose that step,
and one container build per commit has to replace it.

Question for the owner,
ready to paste (under 1,500 characters):

```text
Should mvm's default guest shrink below 8,192 MiB, with the crate's test binaries built once on the
host in an Ubuntu 24.04 container and placed at the same absolute path in the guest?
Adopt for arms that do not build in the guest (Recommended): measured on the real lock arm, which
passed in 1.13 s in a 4 GiB guest on test binaries built in an Ubuntu 24.04 container and placed at the
guest's own crate path. Peak host charge 4.04 GiB against the same arm in a default building guest at
7.98 GiB, and 2.00 against 5.44 GiB after each dropped its cache, under the same four other guests.
OpenZFS still built in the guest in 7 m 24 s, and the crate build (7 m 40 s) became a 6 s push.
Cost: every driver changes, because each builds at its own /opt path; one container build per commit
(18 min for both builds here); no arm whose subject is the in-guest build can run this way.
Keep 8,192 MiB: no driver changes, and every arm keeps its in-guest build.
```

### Should the host turn KSM on so a round can measure it?

What it would change:
a host setting (`/sys/kernel/mm/ksm/run`),
owned by the host's root account,
and a round built to measure merging between two guests of one template.
It also needs a guest whose RAM is private (the guest-agent route,
or a change to the share),
because KSM never merges a memfd's page cache.

What is measured:
the guest-agent guest's RAM mapping carries the mergeable flag
(`mg` in `VmFlags`),
the memfd guests' RAM mapping never does;
KSM read 0 throughout,
so nothing merged.
The guest-agent route also costs the file route (11 seconds against 3 for 12.6 MB)
and has no `/mnt/shared`,
which the arms use.

Recommendation:
not now,
unless the owner wants the guest-agent route weighed;
then turn KSM on for one round and measure two guests of one template.

Question for the owner,
ready to paste (under 1,500 characters):

```text
Should the host turn KSM on so a round can measure what it merges between guests?
Yes, for one measurement round (Recommended only if the guest-agent route is worth weighing): write 1
to /sys/kernel/mm/ksm/run, then run two guests of one template at once and record pages_shared,
pages_sharing and the host's RAM before and after. Measured so far: the guest-agent route's RAM is
private and carries the mergeable flag; the memfd route's never does, so the share must be given up
first, which costs the file route (11 s against 3 s for 12.6 MB) and /mnt/shared.
No: the guest-agent route stays unmeasured and the two-guest saving is unknown.
```

## What is not measured, and is owed

The run-to-run spread of the resident and swapped split is not measured:
the host's pressure changed between every run,
and one interleaved pair is not a spread.
What KSM would return is not measured,
for the reason under
"Should the host turn KSM on so a round can measure it?".
The page-fault cost of reporting is not measured:
a reported page is gone from the host,
so the guest's next touch allocates and zeroes a fresh page,
and this round measured no arm time difference that could show it.
A 6 GiB middle ground,
large enough to build OpenZFS and small enough to give memory back,
is not
measured.
Windows and Fedora guests,
and the Hetzner backend,
are not measured.
The share's page-cache cost under reporting is not measured:
how much of a guest's reported memory comes back as page-cache pages the host could reclaim anyway
is not separated from the rest.

## What shipping a choice would change in mvm

- Free page reporting (decided):
  one element in the domain XML,
  written by `package/mvm/cli/src/domain-xml.ts` beside `sharedMemoryBacking()`.
  If it were unconditional,
  every new guest would have it and every existing guest would have it
  from its next create.
- A smaller guest:
  the memory constant in `package/mvm/cli/src/config.ts`,
  and a per-guest value would need a flag and a `meta.json` field too.
  It is only safe together with moving the build off the guest:
  reporting does not lower a build's peak,
  and the first round's one-job builds peaked at 6.2 GiB of the guest's 7.75 GiB
  (the kept guest samples read `AnonPages` 6.21 and 6.20 GiB),
  so a 4 GiB guest cannot hold one.
- A cache drop or a destroy between arms:
  no `mvm` change at all;
  both are round steps.

## Options considered and not measured

### Turning KSM on

Covered by the KSM question.
It cannot be turned on by an agent round,
and one guest at a time cannot show a merge between two.

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
it is the one alternative that could shrink a guest's memory while it
runs,
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

The ARC was not at its floor when this round read it:
the first round's gate log shows `arcSize` 5,095,552,408 at 16:09:17 against `c_min` 2,069,377,152,
and 5,642,313,344 at 17:04:20.
An earlier reading in this round's samples found it at about 1.95 GiB,
about 19 MB above the floor.
So the ARC is not a fixed cost,
but the gate's own sum treats only `size` minus `c_min` as
reclaimable,
and nothing in the guests' measurements moves with it.
Not a lever for guest memory.

### Swapping guest memory out under pressure

The host already does this:
one of the four guests in the first round had 5.36 GiB of its RAM in the zram swap device.
It is not an option among the others because it is not free:
it costs zram's compression CPU and the guests' latency,
it happens only under pressure,
and swapping does not make the charge go away:
the three guests in the opening reading still held 24.0 GiB charged,
most of it swapped.
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md` records what that costs this machine.
