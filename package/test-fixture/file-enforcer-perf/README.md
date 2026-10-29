# file-enforcer-perf

Performance benchmark suite for `@monochromatic-dev/file-enforcer-core`.
Tests file-enforcer under VPS-like resource constraints using podman containers with CPU contention,
 memory limits,
 and IO throttling calibrated against a real VPS baseline.

## Quick start

```bash
# Micro-benchmarks (fast, runs locally)
mise run //package/test-fixture/file-enforcer-perf:perf:micro

# Grant CAP_PERFMON to node once so perf:micro reports hardware counters
mise run //package/test-fixture/file-enforcer-perf:perf:enable-counters

# End-to-end benchmarks with hyperfine (requires hyperfine)
mise run //package/test-fixture/file-enforcer-perf:perf:e2e

# Full constrained benchmark in podman (requires podman, ~2 min)
mise run //package/test-fixture/file-enforcer-perf:perf:constrained
```

See the hardware counters section below for what `perf:enable-counters` does.

## Benchmark fixture

`setup-fixture.ts` creates 240 files across 20 simulated packages at `/tmp/file-enforcer-perf/`:

- 5 markdown docs per package (100 lines each,
   shared header for dedup testing)
- 3 TypeScript source files per package
- 1 deep-nested module (6 levels:
   `lib/deep/nested/very/deep/module.ts`)
- 1 `settings.json` per package (for getProperty testing)
- 2 type definition files per package

`perf.config.ts` exercises all major operations:
 concatenation (5 groups of 4 packages),
 glob mirroring (3 patterns across 20 packages),
 getProperty extraction,
 dedup across all 20 readmes,
 and deep-glob mirroring.
Writes ~68 destination files total.

## Benchmark tiers

### Micro-benchmarks (`perf.bench.test.ts`)

11 standalone mitata benchmarks (declared with `bench()` from `mitata`,
 not the module-test harness;
 the file runs directly as `node <path>`).
Detect gross regressions (10x+ slowdowns) but are inherently unreliable due to JIT optimization,
 GC pauses,
 and scheduler jitter.
Use generous pass thresholds and consume results to prevent dead code elimination.

### End-to-end (`run-e2e.ts`)

Uses hyperfine for statistically rigorous timing across 4 scenarios:

- **Cold run**:
   dest directory empty,
   all files written fresh
- **Warm run**:
   all content unchanged,
   all writes skipped
- **1 source changed**:
   one source modified between runs
- **1 dest changed**:
   one dest modified externally between runs

### Constrained (`run-constrained.ts`)

Launches 5 podman containers simultaneously,
 all pinned to the same CPU core via `taskset`,
 creating natural scheduling contention that simulates a shared VPS:

- **CPU**:
   5 containers on 1 core via `taskset -c 0` (~20% throughput each)
- **Memory**:
   `--memory=1g`
- **IO**:
   `--device-read/write-iops=<auto-detected device>:100`,
   `--device-read/write-bps=<auto-detected device>:80mb` (the backing block device is resolved
    at runtime via `findmnt` against `MONOREPO_ROOT`,
   then canonicalized through any mapper symlinks;
   on a typical LUKS+btrfs setup this lands on `/dev/dm-0`)

Validates constraints by ensuring the **peak** container sysbench score stays below the known VPS baseline
 of 1605 events/sec.
Reports min,
 median,
 and max timings across all containers for each benchmark scenario.

## CPU scheduling approach

### Why contention instead of bandwidth throttling

The original approach used `--cpus=0.3` (CFS bandwidth throttling via `cpu.max`).
This created unrealistic benchmark results because CFS bandwidth control works by alternating between "run"
 and "stall" phases within fixed periods:

- With `cpu.max = "30000 100000"` (30ms quota per 100ms period),
   the process runs for up to 30ms,
   then is **frozen for the remaining 70ms** until the next period
- Operations under 30ms complete cleanly;
   operations crossing a period boundary get a random 70ms penalty
- This creates a **bimodal latency distribution** (fast or fast+70ms) rather than the uniformly slower execution
 of a real VPS
- p95/p99 numbers are inflated by stall injection,
   not by actual compute slowness
- Reducing the period (e.g.,
   `cpu.max = "3000 10000"`) shrinks the stalls to 7ms but introduces scheduler overhead and measurement noise
    from the 100Hz refill rate

### How real VPS providers schedule CPU

Real VPS providers use approaches that do not produce periodic stalls:

- **Dedicated vCPU pinning** (Hetzner,
   DigitalOcean,
   Vultr regular instances):
   KVM with `virsh vcpupin` gives the guest a full hardware thread of a weaker Xeon/EPYC.
   Execution is uniformly slower with zero periodic stalls.
- **Proportional sharing** (overcommitted hosts,
   some budget providers):
   Uses `cpu.weight` (cgroup v2) or `cpu.shares` (v1).
   No hard cap;
   throughput varies with neighbor activity but without artificial stall injection.
- **Burstable with `cpu.max.burst`** (AWS t3,
   Azure B-series,
   Ubicloud burstable):
   Uses `cpu.max` with accumulated burst credits that smooth out the stalls significantly.

None of these match the bare `cpu.max` sawtooth behavior.

### Current approach: contention via taskset

The benchmark launches 5 containers,
 each pinned to CPU core 0 via `taskset -c 0` inside the container.
All containers compete for the same physical core through the kernel's EEVDF scheduler,
 which preempts at sub-millisecond timeslice granularity.

Benefits over CFS bandwidth throttling:

- Preemption at natural scheduler boundaries,
   not at arbitrary 10-100ms walls
- No bimodal latency;
   just uniformly slower with realistic variance from contention
- The variance itself is useful data (it captures real scheduling jitter)
- Each container reports sysbench scores;
   with 5 containers on 1 core of a Ryzen 7 8700F (~5300 events/sec total),
   each gets ~1060-1090 events/sec,
   well below the 1605 VPS baseline

### Why taskset instead of cpuset cgroup

Rootless podman on systemd-managed cgroups v2 cannot use `--cpuset-cpus`
 because systemd does not delegate the cpuset controller to user slices by default.
The cpuset controller is a partitioning controller (it carves out exclusive core ownership),
 unlike proportional controllers like `cpu.weight` or `memory.max`.
Delegating it to unprivileged users would let them monopolize physical cores,
 starve system services,
 or simplify cache-timing side-channel attacks on multi-tenant servers.

Three workarounds exist:

- **systemd drop-in** (`Delegate=cpuset cpu io memory pids` in `/etc/systemd/system/user@.service.d/delegate.conf`):
   permanent,
   requires one-time sudo,
   safe on single-user workstations but inappropriate for multi-tenant systems
- **sudo podman**:
   works but requires root,
   and SELinux relabeling (`:Z`) conflicts when multiple containers mount the same volume simultaneously
- **taskset inside the container**:
   process-level CPU affinity that propagates to child processes,
   works with rootless podman,
   requires no system configuration changes

The benchmark uses `taskset` for maximum portability across contributor machines.

### SELinux and parallel volume mounts

When multiple rootless containers mount the same host directory,
 podman's `:Z` volume flag relabels files for one container's SELinux context,
 causing `EACCES` or `CouldntReadCurrentDirectory` in sibling containers.
The benchmark uses `--security-opt label=disable` to skip SELinux labeling entirely.
The mounted volume is read-only in practice (benchmark writes go to `/tmp` inside each container),
 so the security impact is negligible.

## Calibration

The host CPU is an AMD Ryzen 7 8700F with ~1.7x the IPC of a typical VPS Xeon.
With 5 containers sharing 1 core,
 each gets ~20% of a fast core (~1060-1090 sysbench events/sec),
 reliably below the 1605 events/sec VPS baseline.

IO throttling limits are set to 100 IOPS and 80 MB/s,
 simulating a cheap shared HDD.
Verified with `dd` (200 writes in 1.98s = 101 IOPS) and bandwidth tests (4.2 MB/s at 5mb limit).

## Hardware counters

Both the micro tier and the constrained tier can report CPU hardware counters (cycles,
 instructions,
 instructions-per-cycle,
 cache references and misses,
 branch mispredictions) through `@mitata/counters`.
On Linux this needs perf access:
 the default `kernel.perf_event_paranoid=2` blocks the kernel-inclusive counters the addon opens
  unless the process holds `CAP_PERFMON`.

### Micro tier

mitata loads `@mitata/counters` itself and prints counters inline in its benchmark table.
The workspace uses strict pnpm isolation (`hoist: false`,
 `nodeLinker: isolated`),
 so mitata cannot resolve the addon from its own location;
 a `packageExtensions` entry in `pnpm-workspace.yaml` declares the dependency for mitata
  so the runtime `import('@mitata/counters')` resolves.

Grant `CAP_PERFMON` to the node binary once,
 then run the micro benchmarks:

```bash
mise run //package/test-fixture/file-enforcer-perf:perf:enable-counters
mise run //package/test-fixture/file-enforcer-perf:perf:micro
```

`perf:enable-counters` runs `setcap cap_perfmon+ep` on the node binary (one sudo prompt).
Without the capability the benchmarks still run;
 mitata just omits the counter columns.
Two caveats:

- The capability is granted to the node binary system-wide,
   so every node process gains `CAP_PERFMON` until it is removed with `perf:disable-counters`.
- A mise node upgrade replaces the binary and drops the capability;
   re-run `perf:enable-counters` afterward.

### Constrained tier

`bench-in-container.ts` drives the `@mitata/counters` low-level API directly (it times with `performance.now()`,
 not mitata) and wraps each config run to capture per-region counters;
 `run-constrained.ts` reports the median instructions,
 cycles,
 and IPC per scenario alongside the timings.

This is best-effort.
Rootless podman cannot obtain perf access from `--cap-add=PERFMON` alone:
 the container's capabilities live in a child user namespace,
 but the kernel checks perf permission against the init user namespace,
 and the perf subsystem is not namespaced so the container inherits the host `perf_event_paranoid`.
At the default value of 2 the counters stay dormant and each scenario reports timing only.
To populate them,
 lower the host setting before running the constrained benchmark,
 for example `sudo sysctl kernel.perf_event_paranoid=1`,
 then restore it afterward.

The counter span covers thread-on-CPU work between the start and end of each measured region,
 including event-loop activity during awaits,
 so the constrained counters are approximate for these single-shot async runs.

## Results (as of 2026-02-22)

### Host (AMD Ryzen 7 8700F, NVMe SSD)

- Cold:
   ~18ms
- Warm:
   ~16ms
- Source changed:
   ~16ms
- Dest changed:
   ~16ms

### Constrained (5 containers on 1 core, 1GB RAM, HDD-like IO)

- Cold:
   min=72ms,
   median=92ms,
   max=111ms (n=5)
- Warm:
   min=14ms,
   median=25ms,
   max=47ms (n=50)
- Source changed:
   min=18ms,
   median=22ms,
   max=34ms (n=5)
- Dest changed:
   min=9ms,
   median=16ms,
   max=24ms (n=5)

Latency variance comes from natural EEVDF scheduling contention rather than CFS period-boundary stalls.
The smooth distribution (no bimodal clustering) confirms the contention approach produces realistic VPS-like behavior.

### Before contention approach (--cpus=0.3, CFS bandwidth throttling)

- Cold:
   ~100ms
- Warm:
   ~2ms (but individual runs ranged 2-55ms from CFS jitter)
- Source changed:
   ~2ms
- Dest changed:
   ~2ms

The 2-55ms warm variance was caused by CFS period boundary stalls,
 not actual performance variation.

## Source files

- `setup-fixture.ts`:
   creates the 240-file benchmark fixture
- `perf.config.ts`:
   exercises all file-enforcer operations against the fixture
- `perf.bench.test.ts`:
   11 micro-benchmarks with JIT limitation notes;
   mitata prints hardware counters inline when `CAP_PERFMON` is granted
- `run-e2e.ts`:
   hyperfine-based end-to-end benchmarks
- `validate-resources.ts`:
   CPU,
   memory,
   IO,
   and sysbench benchmarks for resource validation
- `bench-in-container.ts`:
   runs inside the container,
   validates CPU affinity and memory limits,
   times config execution,
   and captures best-effort hardware counters
- `container-counters.ts`:
   best-effort `@mitata/counters` wiring (load,
   probe,
   per-region measure) for the constrained tier
- `mitata-counters.d.ts`:
   ambient types for the untyped `@mitata/counters` addon
- `run-constrained.ts`:
   orchestrates podman build,
   parallel container launch,
   and constrained benchmarks
- `Containerfile`:
   CachyOS with Node and sysbench
