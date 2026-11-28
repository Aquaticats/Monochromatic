# Readings behind the mvm guest memory proposal

The numbers in `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md` come from these files.
Each line is what one command read at one moment on the host.
Every figure is one run.
The host was under other rounds' load throughout,
so a run's resident and swapped split moves with pressure,
and the proposal states that limit where it uses the figures.

Each file is named for the guest it describes,
a guest's name being the `mvm` name the run gave it:
`mem-a` to `mem-d`,
`mem-second-4g`,
`mem-second-8g`,
`mem-third-8g-prebuilt`.
Times are UTC and every file keeps the host's own clock.

## The four guests of 2026-11-25T21:14:34+00:00 to 2026-11-25T23:35:20+00:00

Guests `mem-a` to `mem-d` were one throwaway `mvm` guest each,
destroyed after its readings were pulled.

- `mem-a-host-samples.jsonl`,
  `mem-b-host-samples.jsonl`,
  `mem-c-host-samples.jsonl`,
  `mem-d-host-samples.jsonl`:
  the host's own samples every 10 seconds while each guest ran,
  holding `/proc/meminfo` fields,
  memory pressure,
  the zram device's counters (except `mem-a`'s file,
  which holds none),
  KSM's counters,
  and each QEMU process's `smaps_rollup`,
  largest mapping and mergeable-mapping count.
- `mem-a-guest-samples.log.txt`,
  `mem-b-guest-samples.log.txt`:
  each guest's own `/proc/meminfo` every 5 seconds during a build.
- `mem-a-guest-build.log.txt`,
  `mem-b-guest-build.log.txt`:
  each guest's build output.
- `mem-b-guest-prepare.log.txt`:
  the guest's package installation output.
- `mem-c-guest-readings.txt`:
  guest `mem-c`'s readings,
  transcribed when the guest was destroyed before its own log was pulled.
- `mem-d-guest-run.log.txt`:
  the smoke script `mem-d` ran,
  which stopped at a worktree refusal;
  the failure was reported rather than a passing arm.
- `memory-gate-readings-2026-11-25.txt`:
  the memory gate's readings while a guest window was waited for.

## The guests of 2026-11-26T01:50:43+00:00 to 2026-11-26T02:27:32+00:00

Two guests,
one that built the crate inside itself and one that did not,
each on the lock arm.

- `container-build-binaries-2026-11-26T00-16Z.txt`:
  the Ubuntu 24.04 container build at `/opt/git-policy-cli-lock`,
  its versions,
  the sizes and sha256 of the wrapper and of the `native_binary` test binary,
  the highest `GLIBC_x.y` each needs (`readelf --version-info`),
  the absolute paths the test binary baked,
  and the smoke run in which both binaries started in the same Ubuntu image at the same paths.
- `mem-second-4g-host-samples.jsonl`,
  `mem-second-8g-host-samples.jsonl`:
  the host's own samples every 10 seconds around each guest,
  in the format of the 2026-11-25 files,
  with each guest named in the sample's `guests` list.
- `mem-second-4g-guest-memory.log.txt`,
  `mem-second-8g-guest-memory.log.txt`:
  each guest's own `/proc/meminfo` every 5 seconds for its whole life.
- `mem-second-4g-guest-prepare.log.txt`,
  `mem-second-4g-guest-openzfs-build.log.txt`,
  `mem-second-4g-guest-toolchain.log.txt`,
  and the same three for `mem-second-8g`,
  plus `mem-second-8g-guest-crate-build.log.txt`:
  the preparation's steps inside the guest and the crate build,
  none of which carries timestamps of its own.
- `mem-second-4g-guest-unpack.log.txt`,
  `mem-second-4g-guest-pool.log.txt`,
  `mem-second-4g-lock-arm.log.txt`,
  `mem-second-4g-guest-drop.log.txt`,
  and the same four for `mem-second-8g`:
  the binaries starting in the guest,
  the pool,
  the arm's own log with its 15 marker lines and its `test result` line,
  and the cache drop.
- `mem-second-4g-create.log.txt`,
  `mem-second-4g-destroy.log.txt`,
  and the same two for `mem-second-8g`:
  `mvm create` and `mvm destroy` output,
  whose timestamps give the two durations.
- `memory-gate-readings-2026-11-26.txt`:
  the gate's readings every 30 seconds while the guests were waited for,
  each line naming `MemAvailable`,
  the ARC's `size` and `c_min`,
  and memory pressure's two averages.

## The guest of 2026-11-26T03:56:49+00:00

One default guest on the binaries the container built,
the control the other two cells lacked:
a default guest on the same container-built binaries.

- `container-build-binaries-2026-11-26T03-51Z.txt`:
  the container build of 2026-11-26T03:51+00:00,
  its versions,
  its two binaries' sizes and glibc floors,
  its wall time,
  and the cgroup memory peak the container itself reached.
- `mem-third-8g-prebuilt-host-samples.jsonl`:
  the host's own samples every 10 seconds around the guest,
  in the same format.
- `mem-third-8g-prebuilt-guest-memory.log.txt`:
  the guest's own `/proc/meminfo` every 5 seconds for its whole life.
- `mem-third-8g-prebuilt-guest-prepare.log.txt`,
  `mem-third-8g-prebuilt-guest-openzfs-build.log.txt`,
  `mem-third-8g-prebuilt-guest-toolchain.log.txt`:
  the preparation's three steps.
- `mem-third-8g-prebuilt-guest-unpack.log.txt`,
  `mem-third-8g-prebuilt-guest-pool.log.txt`,
  `mem-third-8g-prebuilt-lock-arm.log.txt`,
  `mem-third-8g-prebuilt-guest-drop.log.txt`:
  the binaries starting at the container's path,
  the pool,
  the arm,
  and the cache drop.
- `mem-third-8g-prebuilt-create.log.txt`,
  `mem-third-8g-prebuilt-destroy.log.txt`:
  `mvm create` and `mvm destroy` output.
- `memory-gate-readings-2026-11-26T03-56Z.txt`,
  `step-timings-2026-11-26T03-56Z.txt`:
  the gate reading that permitted the create,
  and every step's start and end taken from the driver's own clock,
  which is where the preparation's length comes from,
  since its three logs carry no timestamps.

## The round of 2026-11-28: three container builds, one local guest and one rented machine

The round the owner asked for on 2026-11-28
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"Building the test binaries outside the guest")
measured the container side of the build-location route three times
and the route's control arm inside one guest,
`gb-i1`,
which built the lock driver's narrowed test target inside itself at the default 8,192 MiB.
The outside arm,
a guest running the container's binaries,
ran on a rented machine the same day,
because the policy of that day stopped local guests;
its files are in the section below.

- `container-build-lock-2026-11-28T11-49Z.txt`:
  the lock driver's container build at `/opt/git-policy-cli-lock`,
  `cargo build --release --test native_binary --locked --features test-only-switches --jobs 1`,
  its image and glibc,
  the toolchain,
  the paths cargo reported,
  each artifact's size and sha256 and glibc floor,
  its wall time,
  and the container's own `memory.peak`.
- `container-build-lock-repeat-2026-11-28T12-47Z.txt`:
  the same build,
  image,
  snapshot,
  flags and scope run again on unchanged input,
  which is the container build's run-to-run spread.
- `container-build-forced-act-2026-11-28T12-07Z.txt`:
  the forced act driver's build at `/opt/forced-act`,
  `cargo test --release --lib --no-run --locked -j 1`,
  in the same shape.
- `container-artifacts-paths-and-floors-2026-11-28.txt`:
  what the three binaries carry and need:
  each one's bytes and glibc floor,
  how often each names its driver's crate path,
  how often each names this host's checkout,
  and the one host path a lock artifact carries,
  which is a fixture string in the crate's own tests.
- `gb-i1-inside-lock-steps-2026-11-28T12-15Z.txt`:
  the guest's steps,
  each with this host's clock before and after it.
- `gb-i1-inside-lock-gate.log.txt`:
  the listing that permitted the create,
  and how many guests it showed.
- `gb-i1-inside-lock-domain.xml.txt`:
  the domain `mvm` wrote for the guest,
  which is where its memory and its reporting balloon are written down.
- `gb-i1-inside-lock-create-and-destroy.log.txt`:
  `mvm`'s own lines for the create and the destroy,
  whose timestamps give those two durations.
- `gb-i1-inside-lock-guest-memory.log.txt`:
  the guest's own `/proc/meminfo` every five seconds for its whole life.
- `gb-i1-inside-lock-host-samples.jsonl`:
  the host's samples every ten seconds,
  holding `/proc/meminfo`,
  memory pressure,
  the guests' unit's `memory.current`,
  `memory.swap.current`,
  `memory.peak`,
  `memory.max` and its `max` event count,
  and the guests alive in that unit at each sample.
- `gb-i1-inside-lock-crate-build.log.txt`:
  the detached build step's own log,
  which holds the guest's `free` before and after its swap file,
  the clock's lines around the compiler,
  and the script's status line.
- `gb-i1-inside-lock-arm.log.txt`:
  the arm's log as the guest's step printed it back,
  with its fifteen `LOCK` marker lines and its `test result` line.

## The outside arm of the same round, on a rented 4 GiB machine

The outside arm ran on OVHcloud,
on flavour `d2-4` (2 vCPUs,
4 GiB),
because the backend decision of 2026-11-28 stopped local guests.
Its machine was destroyed by its name when the arm ended,
and its logs have the machine's address and instance identifier replaced by `<address>` and `<instance id>`,
and this round's own paths by `<round folder>` and `<worktree>`,
because the repository is public.

- `gb-o1-outside-lock-steps.txt`:
  the machine's steps,
  each with this host's clock before and after it.
- `gb-o1-outside-lock-guest-memory.log.txt`:
  the machine's own `/proc/meminfo` every five seconds for its whole life.
- `gb-o1-outside-lock-arm.log.txt`:
  the arm's log,
  with its fifteen `LOCK` marker lines and its `test result` line.
- `gb-o1-outside-lock-place.log.txt`:
  what the push left at the crate's path,
  the wrapper starting from it,
  and the guest's own answer that `cargo`
  is not on `PATH`.
- `gb-o1-outside-lock-release.log.txt`,
  `gb-o1-outside-lock-kernel-release.log.txt`:
  the release and kernel the machine booted,
  its vCPU count and its `MemTotal`.
- `gb-o1-outside-lock-pushes.log.txt`:
  `mvm push`'s own lines for the source tree and the artifacts tarball.
- `gb-o1-outside-lock-create-and-destroy.log.txt`:
  `mvm`'s lines for the create and the destroy,
  the project's machines right after the create,
  and the listing after the destroy.

## The one guest that lists its own devices

- `balloon-check.txt`,
  `memory-gate-readings-2026-11-26T04-10Z.txt`:
  a guest created by this worktree's `mvm` source on 2026-11-26T04:10:20Z,
  the virtio devices it saw with their device ids and bound drivers,
  its create and destroy lines,
  and the gate reading that permitted it.
  It is there because no kept file records any guest's domain XML:
  this one's own view says the balloon device was present.

## The paths the crate's test binaries carry

- `compile-time-paths.txt`:
  every `env!` site of `CARGO_MANIFEST_DIR`,
  `CARGO_BIN_EXE_monochromatic-git-policy-cli` and `CARGO_TARGET_TMPDIR` in
  `package/git-policy/cli/src`,
  with the counts and the command that read them,
  against the commit named at its top.

## Editing

Four mechanical edits only:
the 2026-11-25 files were renamed `.log` to `.log.txt` when they were committed,
because this repository's `.gitignore` ignores `*.log`;
every line naming a round folder's own home path is removed from the copies,
because the repository is public;
and one file that held no bytes (`mem-second-8g`'s push log) was removed rather than kept.
The 2026-11-28 copies have their round's own scratch folder path replaced by `<round folder>`
and its worktree path by `<worktree>`,
because the repository is public and neither is a fact about a reading;
no reading changed.
Three hand-written title lines,
the first line of `step-timings-2026-11-26T03-56Z.txt`,
`container-build-binaries-2026-11-26T03-51Z.txt` and `container-build-binaries-2026-11-26T00-16Z.txt`,
name the file and its guest or time instead of a round ordinal;
no reading changed.
Nothing else is changed.

## What is not in this folder

- Binaries.
  The wrapper is about 30 MB and the test binary about 17 MB,
  and neither belongs in a source repository.
- Host command logs that carry a round folder's absolute path,
  such as `mvm push` invocations.
  They stay in the round folder.
- The sampler,
  gate,
  container-build and guest-driver programs.
  They stay in the round folder;
  the proposal's "Method" section names what each one reads and does.
- The artifacts the containers built.
  They are 17.7 MB,
  32.5 MB and 89.7 MB of executables and are recreated by one build;
  their paths,
  sizes and sha256 are in the files above.
