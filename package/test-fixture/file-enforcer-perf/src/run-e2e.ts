/**
 * Orchestrates end-to-end performance benchmarks using hyperfine.
 * Runs four scenarios: cold run, warm run (all unchanged),
 * 1 source changed, and 1 dest changed.
 *
 * Requires hyperfine to be installed (managed via mise).
 */

import {
  mkdir,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';

import spawn from 'nano-spawn';

//region Setup

console.log('[e2e] setting up fixture...',);
await import(resolve(import.meta.dirname, 'setup-fixture.ts',));

/** Absolute path to the perf config */
const CONFIG = resolve(import.meta.dirname, 'perf.config.ts',);
/** Absolute path to the file-enforcer CLI source */
const CLI = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  'file-enforcer',
  'core',
  'src',
  'cli.ts',
);
/** Command string that runs the perf config through the CLI */
const RUN_CONFIG = `node ${CLI} ${CONFIG}`;

/** Fixture root and subdirectories, respects $TMPDIR for sandbox compatibility */
const FIXTURE_DIR = join(scratchRoot(), 'file-enforcer-perf',);
/** Absolute path to the destination subdirectory within the fixture */
const DEST_DIR = `${FIXTURE_DIR}/dest`;

/** Files modified by hyperfine --prepare for changed-file scenarios */
const SOURCE_FILE = `${FIXTURE_DIR}/src/pkg-00/docs/readme.md`;
/** Destination file modified by hyperfine --prepare for the dest-changed scenario */
const DEST_FILE = `${DEST_DIR}/combined-0.md`;

//endregion Setup

//region Hyperfine runner

/**
 * Runs a hyperfine benchmark with the given arguments.
 *
 * @param label - Human-readable scenario name for logging
 *
 * @param args - Arguments passed to hyperfine
 *
 * @throws When hyperfine exits with a non-zero code
 */
async function runHyperfine(label: string, args: readonly string[],): Promise<void> {
  console.log(`\n--- ${label} ---`,);
  await spawn('hyperfine', [...args,], {
    stdout: 'inherit',
    stderr: 'inherit',
  },);
}

//endregion Hyperfine runner

//region Benchmark scenarios

// 1. Cold run: dest dir empty, everything written fresh
await runHyperfine('Cold run (all files written)', [
  '--runs',
  '5',
  '--prepare',
  `rm -rf ${DEST_DIR} && mkdir -p ${DEST_DIR}`,
  RUN_CONFIG,
],);

// 2. Warm run: populate dest first, then re-run (all content-based skips)
console.log('\n[e2e] populating dest for warm runs...',);
await rm(DEST_DIR, { recursive: true, force: true, },);
await mkdir(DEST_DIR, { recursive: true, },);
await spawn('node', [CLI, CONFIG,],);

await runHyperfine('Warm run (all unchanged)', [
  '--warmup',
  '2',
  '--runs',
  '10',
  RUN_CONFIG,
],);

// 3. One source file modified between runs (forces re-read + re-write of affected dests)
await runHyperfine('1 source changed', [
  '--warmup',
  '2',
  '--runs',
  '10',
  '--prepare',
  `echo "# Modified at $(date +%s%N)" >> ${SOURCE_FILE}`,
  RUN_CONFIG,
],);

// 4. One dest file modified between runs (simulates external edit, triggers re-write)
await runHyperfine('1 dest changed', [
  '--warmup',
  '2',
  '--runs',
  '10',
  '--prepare',
  `echo "# Externally modified at $(date +%s%N)" >> ${DEST_FILE}`,
  RUN_CONFIG,
],);

//endregion Benchmark scenarios

console.log('\n[e2e] all benchmarks complete.',);
