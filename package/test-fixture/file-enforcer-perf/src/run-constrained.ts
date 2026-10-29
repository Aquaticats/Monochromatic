/**
 * Builds a podman container and runs file-enforcer benchmarks under
 * VPS-like resource contention. Launches multiple containers, each
 * pinned to the same CPU core via taskset inside the container,
 * creating natural scheduling contention that simulates a shared VPS
 * without CFS bandwidth throttling artifacts.
 *
 * Uses taskset (process-level CPU affinity) instead of cgroup cpuset
 * because rootless podman on systemd-managed cgroups v2 does not
 * delegate the cpuset controller to user slices. taskset inside the
 * container achieves the same pinning effect without requiring sudo
 * or systemd configuration changes.
 *
 * Validates constraints by:
 * 1. Running sysbench on host (baseline)
 * 2. Launching N containers on the same CPU core simultaneously
 * 3. Ensuring PEAK container sysbench stays below the real VPS baseline
 *    of 1605 events/sec
 * 4. Running file-enforcer benchmarks under natural CPU contention
 */

import { cachyosBuildArguments, } from '@monochromatic-dev/test-harness-core-matrix/ts';
import { resolve, } from 'node:path';

import {
  CONTAINER_COUNT,
  CONTAINERFILE,
  CPUSET_CPU,
  IMAGE_NAME,
  MONOREPO_ROOT,
  RESOURCE_FLAGS,
  VPS_SYSBENCH_BASELINE,
} from './run-constrained-config.ts';
import {
  collectCounters,
  collectTimings,
  formatCountersSummary,
  formatTimingSummary,
} from './run-constrained-timing.ts';
import {
  type ContainerBenchResult,
  extractSysbenchScore,
  type HostBenchResult,
  parseLastJsonLine,
  runCapture,
  runInherit,
} from './run-constrained-utils.ts';

//region Host baseline

console.log('=== HOST BASELINE ===',);
/** Raw JSON output from the host baseline benchmark */
const hostJson = await runCapture(
  ['node', resolve(import.meta.dirname, 'validate-resources.ts',),],
  'Running host baseline benchmark',
);
/** Parsed host baseline benchmark results */
// oxlint-disable-next-line no-unsafe-type-assertion -- JSON structure matches the known validate-resources.ts output format
const hostResult = JSON.parse(hostJson,) as HostBenchResult;
console.log(`Host sysbench: ${String(hostResult.sysbench.eventsPerSec,)} events/sec`,);
console.log(`Host serial CPU: ${String(hostResult.serial.ms,)}ms`,);
console.log(`Host parallel CPU: ${String(hostResult.parallel.ms,)}ms`,);
console.log(
  `Host IO: ${String(hostResult.io.ms,)}ms (${
    String(hostResult
      .io
      .filesPerSec,)
  } files/sec)`,
);

//endregion Host baseline

//region Container build

console.log('\n=== CONTAINER BUILD ===',);
/** `--build-arg CACHYOS_IMAGE=<image>` for the CachyOS image this host's CPU level supports (no Alpine variant exists) */
const baseImageArgs = await cachyosBuildArguments({
  imageName: IMAGE_NAME,
  alpineVariant: false,
},);
await runInherit(
  ['podman', 'build', ...baseImageArgs, '-t', IMAGE_NAME, '-f', CONTAINERFILE, MONOREPO_ROOT,],
  'Building container image',
);

//endregion Container build

//region Parallel contention benchmark

console.log(
  `\n=== PARALLEL CONTENTION BENCHMARK (${
    String(CONTAINER_COUNT,)
  } containers on core ${CPUSET_CPU}) ===`,
);

/**
 * Runs a single container benchmark and parses the JSON result.
 *
 * @param _unused - Unused array element placeholder
 *
 * @param containerIndex - Index of the container (0-based)
 *
 * @returns Parsed benchmark result from the container
 */
async function runContainerBench(_unused: unknown,
  containerIndex: number,): Promise<ContainerBenchResult>
{
  const output = await runCapture(
    [
      'podman',
      'run',
      '--rm',
      // Disable SELinux labeling: the :Z volume flag relabels files for
      // one container's context, conflicting when multiple containers
      // mount the same volume simultaneously.
      '--security-opt',
      'label=disable',
      ...RESOURCE_FLAGS,
      '-v',
      `${MONOREPO_ROOT}:/app`,
      IMAGE_NAME,
      'taskset',
      '-c',
      CPUSET_CPU,
      'node',
      'package/test-fixture/file-enforcer-perf/src/bench-in-container.ts',
    ],
    `Container ${String(containerIndex,)}`,
  );
  // oxlint-disable-next-line no-unsafe-type-assertion -- JSON structure matches ContainerBenchResult from bench-in-container.ts
  return parseLastJsonLine(output,) as ContainerBenchResult;
}

/** Benchmark results from all containers run in parallel */
const containerResults = await Promise.all(
  Array.from({ length: CONTAINER_COUNT, }, runContainerBench,),
);

//endregion Parallel contention benchmark

//region Sysbench validation

console.log('\n=== SYSBENCH VALIDATION ===',);
for (let containerIndex = 0; containerIndex < containerResults.length; containerIndex++) {
  const result = containerResults[containerIndex];
  if (result !== undefined) {
    console.log(
      `  Container ${String(containerIndex,)}: ${
        String(result
          .sysbench
          .eventsPerSec,)
      } events/sec`,
    );
  }
}

/** Peak sysbench score across all containers */
const peakSysbench = Math.max(...containerResults.map(function getScore(result,) {
  return extractSysbenchScore(result,);
},),);
console.log(`  Peak: ${String(peakSysbench,)} events/sec`,);
console.log(`  VPS baseline: ${String(VPS_SYSBENCH_BASELINE,)} events/sec`,);

if (peakSysbench <= VPS_SYSBENCH_BASELINE) {
  console.log(
    `  PASS: peak (${String(peakSysbench,)}) <= baseline (${
      String(VPS_SYSBENCH_BASELINE,)
    })`,
  );
}
else {
  console.warn(
    `  WARNING: peak sysbench (${String(peakSysbench,)})`
      + ` exceeds VPS baseline (${String(VPS_SYSBENCH_BASELINE,)}).`
      + ' Add more containers or verify contention is sufficient.',
  );
}

//endregion Sysbench validation

//region Cgroup limit verification

for (let containerIndex = 0; containerIndex < containerResults.length; containerIndex++) {
  const result = containerResults[containerIndex];
  if (result !== undefined) {
    if (!result.limits.cpuAffinityValid) {
      console.warn(
        `  WARNING: Container ${
          String(containerIndex,)
        } CPU affinity unexpected: "${result.limits.cpuAffinity}"`,
      );
    }
    if (!result.limits.memoryValid) {
      console.warn(
        `  WARNING: Container ${
          String(containerIndex,)
        } memory unexpected: "${result.limits.memoryMax}"`,
      );
    }
  }
}

//endregion Cgroup limit verification

//region Benchmark results

console.log('\n=== BENCHMARK RESULTS ===',);

/** Timing categories to aggregate: label prefix maps to display name */
const TIMING_CATEGORIES = ['cold', 'warm', 'source-changed', 'dest-changed',] as const;

for (const category of TIMING_CATEGORIES) {
  const values = collectTimings(containerResults, category,);
  if (values.length > 0)
    console.log(formatTimingSummary(category, values,),);
  const categoryCounters = collectCounters(containerResults, category,);
  if (categoryCounters.length > 0)
    console.log(formatCountersSummary(categoryCounters,),);
}

//endregion Benchmark results

console.log('\n=== COMPLETE ===',);
