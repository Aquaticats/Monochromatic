/**
 * Runs inside the podman container under CPU contention from sibling
 * containers pinned to the same core via taskset. Validates that CPU
 * affinity and memory limits are enforced, creates the benchmark fixture,
 * and times multiple perf.config.ts executions under natural scheduling
 * contention.
 *
 * Outputs a JSON summary to stdout for parsing by run-constrained.ts.
 * All diagnostic messages go to stderr.
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';

import spawn from 'nano-spawn';

import {
  type CountersSnapshot,
  loadCounters,
  measureRegion,
  probeCounters,
} from './container-counters.ts';

/** Pattern to extract CPU affinity from /proc/self/status */
const CPU_AFFINITY_PATTERN = /Cpus_allowed_list:\s+(.+)/;

/** Pattern to extract events per second from sysbench output */
const SYSBENCH_EVENTS_PATTERN = /events per second:\s+([\d.]+)/;

//region Resource limit validation

/**
 * Reads the Cpus_allowed_list from /proc/self/status to verify that
 * taskset successfully pinned this process to the expected core.
 *
 * @returns CPU affinity string (e.g. "0" for single core, "0-15" for all)
 */
async function readCpuAffinity(): Promise<string> {
  const status = await readFile('/proc/self/status', 'utf8',);
  const match = status.match(CPU_AFFINITY_PATTERN,);
  return match !== null && match[1] !== undefined ? match[1].trim() : 'unknown';
}

/** Current process CPU affinity read from /proc/self/status */
const cpuAffinity = await readCpuAffinity();

/**
 * Cgroup memory limit, defaults to 'unknown' when cgroup reads fail outside a container.
 * let needed: cgroup reads may fail outside a container.
 */
let memoryMax = 'unknown';
try {
  memoryMax = (await readFile('/sys/fs/cgroup/memory.max', 'utf8',)).trim();
}
catch {
  // Not in a cgroup v2 environment
}

console.error(`[container] CPU affinity (Cpus_allowed_list): ${cpuAffinity}`,);
console.error(`[container] Memory limit (memory.max): ${memoryMax}`,);

/**
 * Expected CPU affinity: pinned to a single core (e.g., "0") by taskset.
 * All containers share this core via EEVDF scheduling rather than
 * CFS bandwidth throttling, avoiding artificial sawtooth latency.
 */
const EXPECTED_CPU_AFFINITY = '0';
/** Expected memory limit in bytes (1 GiB) for the container */
const EXPECTED_MEMORY_BYTES = '1073741824';

/** Whether CPU affinity matches the expected single-core pinning */
const cpuAffinityValid = cpuAffinity === EXPECTED_CPU_AFFINITY;
/** Whether memory limit matches the expected 1 GiB container limit */
const memoryValid = memoryMax === EXPECTED_MEMORY_BYTES;

if (!cpuAffinityValid) {
  console.error(
    `[container] WARNING: CPU affinity unexpected. Got "${cpuAffinity}", expected "${EXPECTED_CPU_AFFINITY}"`,
  );
}
if (!memoryValid) {
  console.error(
    `[container] WARNING: Memory limit unexpected. Got "${memoryMax}", expected "${EXPECTED_MEMORY_BYTES}"`,
  );
}

//endregion Resource limit validation

//region Sysbench CPU baseline comparison

console.error('[container] running sysbench cpu...',);
/** Sysbench stdout output for parsing events per second */
const sysbenchStdout = (await spawn('sysbench', ['cpu', '--threads=1', 'run',],)).stdout;

/** Regex match result for sysbench events per second */
const sysbenchMatch = sysbenchStdout.match(SYSBENCH_EVENTS_PATTERN,);
/** Parsed sysbench events per second, or -1 if parsing failed */
const sysbenchEventsPerSec = sysbenchMatch !== null && sysbenchMatch[1] !== undefined
  ? Number.parseFloat(sysbenchMatch[1],)
  : -1;
console.error(`[container] sysbench: ${sysbenchEventsPerSec.toFixed(1,)} events/sec`,);

//endregion Sysbench CPU baseline comparison

//region Fixture setup

console.error('[container] creating fixture...',);
await import(resolve(import.meta.dirname, 'setup-fixture.ts',));

//endregion Fixture setup

//region Hardware counter setup

/** Loaded counters module, or null when the native addon fails to load. */
const countersModule = await loadCounters();

/**
 * Counters module when the host grants perf access, else null. Best-effort:
 * rootless podman at perf_event_paranoid >= 1 without CAP_PERFMON probes false,
 * and every region is timed without counters.
 */
const counters = countersModule !== null && probeCounters(countersModule,)
  ? countersModule
  : null;

console.error(
  `[container] hardware counters: ${counters !== null ? 'enabled' : 'unavailable (timing only)'}`,
);

//endregion Hardware counter setup

//region Timed config runs

/** Absolute path to the benchmark configuration file */
const CONFIG_PATH = resolve(import.meta.dirname, 'perf.config.ts',);
/** Absolute path to the file-enforcer CLI source */
const CLI_PATH = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  'file-enforcer',
  'core',
  'src',
  'cli.ts',
);

/**
 * Runs the benchmark configuration through the public CLI.
 *
 * @example
 * ```ts
 * await runConfig();
 * ```
 */
async function runConfig(): Promise<void> {
  await spawn('node', [CLI_PATH, CONFIG_PATH,],);
}

/** Timing entry for one config execution */
type TimingEntry = {
  readonly label: string;
  readonly ms: number;
  readonly counters: CountersSnapshot | null;
};

/**
 * Mutable array: benchmark results accumulated sequentially because each
 * run must complete before the next starts (some modify files between runs).
 */
const timings: TimingEntry[] = [];

// Cold run; all destination files written fresh
/** Cold-run timing and counters; all destination files written fresh. */
const coldResult = await measureRegion({
  counters,
  run: async function runCold() {
    await runConfig();
  },
},);
timings.push({ label: 'cold', ms: coldResult.ms, counters: coldResult.counters, },);
console.error(`[container] cold run: ${coldResult.ms.toFixed(1,)}ms`,);

// Warm runs: content unchanged, all writes skipped.
// 10 iterations provide enough samples per container; with N containers
// running simultaneously, the aggregate dataset has N*10 warm data points.
/** Number of warm run iterations per container */
const WARM_RUN_COUNT = 10;
// Sequential execution required: each warm run must complete before the
// next to measure individual run timing accurately.
for (let warmIndex = 0; warmIndex < WARM_RUN_COUNT; warmIndex++) {
  // oxlint-disable-next-line no-await-in-loop -- sequential benchmark timing required
  const warmResult = await measureRegion({
    counters,
    run: async function runWarm() {
      await runConfig();
    },
  },);
  timings.push({
    label: `warm-${String(warmIndex,)}`,
    ms: warmResult.ms,
    counters: warmResult.counters,
  },);
  console.error(`[container] warm run ${String(warmIndex,)}: ${warmResult.ms.toFixed(1,)}ms`,);
}

// 1 source changed: modify one source file, invalidate its cache entry, re-run.
// This mirrors what watch mode does: it knows exactly which file changed.
/** Absolute path to the fixture root directory */
const fixtureDir = join(scratchRoot(), 'file-enforcer-perf',);
/** Path to the source file modified for the source-changed benchmark */
const sourceFile = join(fixtureDir, 'src', 'pkg-00', 'docs', 'readme.md',);
/** Original content of the source file before modification */
const sourceContent = await readFile(sourceFile, 'utf8',);
await writeFile(sourceFile, `${sourceContent}\n# Modified for benchmark`,);
/** Source-changed timing and counters after modifying one source file. */
const srcChangedResult = await measureRegion({
  counters,
  run: async function runSrcChanged() {
    await runConfig();
  },
},);
timings.push({
  label: 'source-changed',
  ms: srcChangedResult.ms,
  counters: srcChangedResult.counters,
},);
console.error(`[container] 1 source changed: ${srcChangedResult.ms.toFixed(1,)}ms`,);

// 1 dest changed: modify one dest file externally, invalidate its cache entry, re-run.
// Simulates watch mode detecting an external edit to a managed destination.
/** Path to the destination file modified for the dest-changed benchmark */
const destFile = join(fixtureDir, 'dest', 'combined-0.md',);
/** Original content of the destination file before modification */
const destContent = await readFile(destFile, 'utf8',);
await writeFile(destFile, `${destContent}\n# Externally modified`,);
/** Dest-changed timing and counters after an external edit to one dest file. */
const destChangedResult = await measureRegion({
  counters,
  run: async function runDestChanged() {
    await runConfig();
  },
},);
timings.push({
  label: 'dest-changed',
  ms: destChangedResult.ms,
  counters: destChangedResult.counters,
},);
console.error(`[container] 1 dest changed: ${destChangedResult.ms.toFixed(1,)}ms`,);

//endregion Timed config runs

//region Output JSON results

/**
 * Rounds to one decimal place.
 *
 * @param value - Number to round
 *
 * @returns Rounded value
 */
function round1(value: number,): number {
  return Math.round(value * 10,) / 10;
}

/**
 * Rounds a timing entry's ms value to one decimal place; counter values are
 * already rounded by the snapshot mapper and pass through unchanged.
 *
 * @param entry - Timing entry with label, ms, and counters
 *
 * @returns New timing entry with rounded ms
 */
function roundTimingEntry(
  entry: TimingEntry,
): { label: string; ms: number; counters: CountersSnapshot | null; } {
  return { label: entry.label, ms: round1(entry.ms,), counters: entry.counters, };
}

/** Structured benchmark results for JSON output */
const results = {
  limits: { cpuAffinity, memoryMax, cpuAffinityValid: cpuAffinityValid, memoryValid, },
  sysbench: { eventsPerSec: round1(sysbenchEventsPerSec,), },
  timings: timings.map(function roundEntry(entry,) {
    return roundTimingEntry(entry,);
  },),
};

// JSON to stdout for parsing by run-constrained.ts
console.log(JSON.stringify(results,),);

//endregion Output JSON results
