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

## The four guests of 2026-11-25T21:14Z to 23:35Z

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
  transcribed by that round because the guest was destroyed before its own log was pulled.
- `mem-d-guest-run.log.txt`:
  the smoke script `mem-d` ran,
  which stopped at a worktree refusal;
  that round reported the failure rather than a passing arm.
- `memory-gate-readings-2026-11-25.txt`:
  the memory gate's readings while that round waited for a guest window.

## The guests of 2026-11-26T01:50Z to 02:27Z

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
  the gate's readings every 30 seconds while that round waited,
  each line naming `MemAvailable`,
  the ARC's `size` and `c_min`,
  and memory pressure's two averages.

## The guest of 2026-11-26T03:56Z

One default guest on the binaries the container built,
which measures the cell the two earlier guests did not.

- `container-build-binaries-2026-11-26T03-51Z.txt`:
  this round's container build,
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

## The paths the crate's test binaries carry

- `compile-time-paths.txt`:
  every `env!` site of `CARGO_MANIFEST_DIR`,
  `CARGO_BIN_EXE_monochromatic-git-policy-cli` and `CARGO_TARGET_TMPDIR` in
  `package/git-policy/cli/src`,
  with the counts and the command that read them,
  against the commit named at its top.

## Editing

Three mechanical removals only:
the 2026-11-25 files were renamed `.log` to `.log.txt` when they were committed,
because this repository's `.gitignore` ignores `*.log`;
every line naming a round folder's own home path is removed from the copies,
because the repository is public;
and one file that held no bytes (`mem-second-8g`'s push log) was removed rather than kept.
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
