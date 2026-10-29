/**
 * Runs CPU (serial + parallel), memory, sysbench, and IO benchmarks,
 * reads cgroup limits. Outputs structured JSON to stdout for comparison
 * between host and container. Diagnostic messages go to stderr.
 *
 * Usage:
 *   Host:      node validate-resources.ts
 *   Container: podman run ... node validate-resources.ts
 */

import { readFile, } from 'node:fs/promises';

import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

import {
  round1,
  runIoBenchmark,
  runMemoryBenchmark,
  runParallelCpuBenchmark,
  runSerialCpuBenchmark,
  runSysbench,
} from './validate-benchmarks.ts';

//region Cgroup limit detection

/**
 * Cgroup CPU quota, defaults to 'max 100000' when reads fail.
 * let needed: cgroup reads may fail on host or non-cgroup-v2 systems.
 */
let cpuMax = 'max 100000';
try {
  cpuMax = (await readFile('/sys/fs/cgroup/cpu.max', 'utf8',)).trim();
}
catch {
  // Not in a cgroup v2 container or no read access
}

/**
 * Cgroup memory limit, defaults to 'max' when reads fail.
 * let needed: same fallback reason as cpuMax.
 */
let memoryMax = 'max';
try {
  memoryMax = (await readFile('/sys/fs/cgroup/memory.max', 'utf8',)).trim();
}
catch {
  // Not in a cgroup v2 container or no read access
}

//endregion Cgroup limit detection

//region Sysbench CPU benchmark

console.error('[validate] running sysbench cpu...',);
/** Sysbench events per second, or -1 if sysbench is not installed */
const sysbenchEventsPerSec = await runSysbench();
console.error(
  `[validate] sysbench: ${String(sysbenchEventsPerSec.toFixed(1,),)} events/sec`,
);

//endregion Sysbench CPU benchmark

//region Serial CPU benchmark: SHA-256 hashing, single-threaded

/** Number of SHA-256 hashes to compute sequentially */
const SERIAL_HASH_COUNT = 200_000;
console.error(`[validate] serial CPU: ${String(SERIAL_HASH_COUNT,)} SHA-256 hashes...`,);

/** Serial CPU benchmark result containing elapsed time and iteration count */
const serialResult = runSerialCpuBenchmark(SERIAL_HASH_COUNT,);
console.error(`[validate] serial CPU: ${serialResult.ms.toFixed(1,)}ms`,);

//endregion Serial CPU benchmark

//region Parallel CPU benchmark: 8 workers each computing hashes

/**
 * Number of parallel worker processes to spawn.
 * On a 16-core host, these all run truly parallel.
 * In a container with --cpus=0.3, they share 0.3 of a core equivalent.
 */
const WORKER_COUNT = 8;
/** SHA-256 hashes per worker */
const HASHES_PER_WORKER = 50_000;

console.error(
  `[validate] parallel CPU: ${String(WORKER_COUNT,)} workers * ${
    String(HASHES_PER_WORKER,)
  } hashes...`,
);

/** Elapsed wall-clock time for parallel CPU benchmark in milliseconds */
const parallelMs = await runParallelCpuBenchmark(WORKER_COUNT, HASHES_PER_WORKER,);
console.error(`[validate] parallel CPU: ${parallelMs.toFixed(1,)}ms`,);

//endregion Parallel CPU benchmark

//region Memory benchmark: allocate and fill 256 MB

/** Allocation size in megabytes */
const ALLOC_MB = 256;
console.error(`[validate] memory: allocating ${String(ALLOC_MB,)}MB...`,);

/** Memory benchmark result containing elapsed time and sink byte */
const memResult = runMemoryBenchmark(ALLOC_MB,);
console.error(`[validate] memory: ${memResult.ms.toFixed(1,)}ms`,);

//endregion Memory benchmark

//region IO benchmark: write + read small files to detect throttling

/**
 * Number of small files to write and read back for IO benchmarking.
 * Tests random IO patterns similar to file-enforcer workload.
 */
const IO_FILE_COUNT = 50;

console.error(`[validate] IO: ${String(IO_FILE_COUNT,)} file write/read cycles...`,);

/** IO benchmark result containing elapsed time and file count */
const ioResult = await runIoBenchmark(IO_FILE_COUNT,);
console.error(
  `[validate] IO: ${ioResult.ms.toFixed(1,)}ms (${
    (IO_FILE_COUNT / ioResult.ms * MS_PER_SECOND)
      .toFixed(0,)
  } files/sec)`,
);

//endregion IO benchmark

//region Output structured JSON

/** Prevent JIT dead code elimination of benchmark results */
const _sink = memResult.sinkByte;

/** Structured benchmark results aggregated for JSON output */
const result = {
  limits: {
    cpuMax,
    memoryMax,
  },
  sysbench: {
    eventsPerSec: round1(sysbenchEventsPerSec,),
  },
  serial: {
    iterations: SERIAL_HASH_COUNT,
    ms: round1(serialResult.ms,),
    hashesPerSec: Math.round(SERIAL_HASH_COUNT / serialResult.ms * MS_PER_SECOND,),
  },
  parallel: {
    workers: WORKER_COUNT,
    hashesPerWorker: HASHES_PER_WORKER,
    totalHashes: WORKER_COUNT * HASHES_PER_WORKER,
    ms: round1(parallelMs,),
    hashesPerSec: Math.round(
      (WORKER_COUNT * HASHES_PER_WORKER) / parallelMs * MS_PER_SECOND,
    ),
  },
  memory: {
    allocatedMB: ALLOC_MB,
    ms: round1(memResult.ms,),
    mbPerSec: Math.round(ALLOC_MB / memResult.ms * MS_PER_SECOND,),
  },
  io: {
    fileCount: IO_FILE_COUNT,
    ms: round1(ioResult.ms,),
    filesPerSec: Math.round(IO_FILE_COUNT / ioResult.ms * MS_PER_SECOND,),
  },
};

// JSON to stdout (parseable), diagnostics already went to stderr
console.log(JSON.stringify(result,),);

//endregion Output structured JSON
