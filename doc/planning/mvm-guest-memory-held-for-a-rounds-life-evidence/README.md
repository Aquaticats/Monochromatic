# Readings behind the mvm guest memory proposal

The numbers in `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md` come from these files.
They are readings,
not scripts:
each line is what one command read at one moment on the host.

Every figure is one run.
The host was under other rounds' load throughout,
so a run's resident and
swapped split moves with pressure;
the proposal labels each figure with the run it came from.

## The first round's guests, 2026-11-25

Guests `mem-a` to `mem-d` were one throwaway `mvm` guest each,
destroyed after its readings were
pulled.
Times are UTC and every file keeps the host's own clock.

- `previous-round-samples-mem-a.jsonl` and `previous-round-samples-mem-b.jsonl`:
  the host's own
  samples every 10 seconds while `mem-a` (default guest,
  free page reporting off) and `mem-b`
  (reporting on) ran one wrapper build each.
  Each line holds `/proc/meminfo` fields,
  the session
  unit's `memory.current` and `memory.stat` fields,
  memory pressure,
  the zram device's counters,
  KSM's
  counters,
  and each QEMU process's `smaps_rollup`,
  largest mapping and mergeable count.
- `previous-round-samples-mem-c.jsonl` and `previous-round-samples-mem-d.jsonl`:
  the same sampler for
  the guest-agent guest and the 4 GiB guest.
- `previous-round-guest-samples-mem-a.log.txt` and `previous-round-guest-samples-mem-b.log.txt`:
  the guests'
  own `/proc/meminfo` readings every 5 seconds during a build.
- `previous-round-build-mem-a.log.txt` and `previous-round-build-mem-b.log.txt`:
  each guest's build output,
  whose timestamps give the build's length.
- `previous-round-prepare-mem-b.log.txt`:
  the guest's package installation output.
- `previous-round-guest-readings-mem-c.txt`:
  guest `mem-c`'s readings,
  transcribed by the round
  because the guest was destroyed before its own log was pulled.
- `previous-round-run-mem-d.log.txt`:
  the smoke script `mem-d` ran,
  which stopped at a Git ownership
  refusal;
  the round reported that failure rather than a passing arm.
- `previous-round-memory-gate-log.txt`:
  the memory gate reading before every create,
  and the reading
  each create fired at.

## The second revision's readings, 2026-11-26

Added as they land,
marked `second-round-`.

## What is not in this folder

- Binaries.
  The wrapper a guest built is about 30 MB and the test binary about 17 MB,
  and neither
  belongs in a source repository.
- Host command logs that carry the round folder's absolute path,
  such as `mvm push` invocations.
  They
  stay in the round folder.
- The sampler,
  gate and guest-driver scripts.
  They stay in the round folder;
  the proposal names what
  each one reads and how.
