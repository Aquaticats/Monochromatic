# file-enforcer-perf TODO

## Statistical rigor

- The micro-benchmarks use generous 10-50x thresholds.
  Track a history of benchmark results and alert on sustained drift rather than single-run spikes.
- Report the percentage of runs that complete within a target latency (e.g.,
   "95% under 10ms") in addition to min/median/max.

## Benchmark coverage

- No benchmark for the `exec()` operation.
  Add a scenario that shells out to a command whose output is written to a destination file.
- No benchmark for `overwriteIfNotExists`:
   only `overwrite` and `overwriteEach` are exercised.
- No benchmark for watch mode latency (time from fs event to re-run completion).
  This would require a different harness that starts the watcher,
   modifies a file,
   and measures the response time.
- No benchmark for config files with many independent rules running in `Promise.all`.
  The current perf.
  `config.ts` has 12 parallel operations;
   test with 50-100 to stress the event loop.

## Calibration

- The container count (5) is calibrated for a Ryzen 7 8700F (~5300 sysbench events/sec per core).
  On a weaker host CPU,
   fewer containers may be needed;
   on a stronger one,
   more.
  Consider auto-calibrating:
   run sysbench on host with `taskset -c 0`,
   divide by 1605,
   round up to get the container count.
- IO throttling (`--device-read/write-iops`) only applies to IO that bypasses the page cache.
  Small files (like file-enforcer's workload) mostly hit page cache,
   so the IOPS limit barely affects results.
  This is realistic (VPS also benefit from page cache),
   but worth noting for interpretation.

## Container setup

- The Containerfile installs sysbench via pacman on every build (cache miss).
  Pre-bake a base image with sysbench to speed up cold builds.
- The container mounts the entire monorepo as a volume.
  For more isolated benchmarks,
   copy only the necessary packages into the container.

## Hardware counters

- Constrained-tier counters are dormant under the default rootless podman setup
  (`perf_event_paranoid=2`),
   since `--cap-add=PERFMON` cannot grant perf access
  across the container's user namespace.
  They populate only when the host paranoid level is lowered before the run.
  Consider a throwaway sysctl toggle around the constrained pipeline if
  per-scenario counters under contention become a routine need.

## Host baseline

- sysbench is not installed on the host (`-1 events/sec` in output).
  Install sysbench on the host for a direct host-vs-container comparison.
  Currently the host baseline only uses the SHA-256 hash benchmark.
- No hyperfine integration in the constrained pipeline.
  `run-e2e.ts` uses hyperfine on the host but `run-constrained.ts` uses single-run timing inside the container.
  Run hyperfine inside the container for consistent methodology.
