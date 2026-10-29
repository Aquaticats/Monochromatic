#!/usr/bin/env node

/**
 Container-side entrypoint for one shard.
 
 Reads the shard manifest, prepares the work tree, establishes a green
 baseline (tests pass, types clean, tsbuildinfo warmed), runs the
 mutant loop, and writes the shard report. A red baseline reports every
 mutant as unrun so the host treats the shard as an infra failure
 rather than blaming mutants.
 
 @example
 ```bash
 node /baked/package/mutation-test/cli/src/container/main.ts
 ```
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { isRecord, } from '../is-record.ts';
import {
  MANIFEST_MOUNT,
  REPORT_MOUNT,
  WORK_MOUNT,
} from '../mounts.ts';
import { runBuildStep, } from './build-step.ts';
import { runMutantLoop, } from './mutant-loop.ts';
import {
  runTests,
  type TestRunOutcome,
} from './test-run.ts';
import {
  tsgoCheck,
  type TsgoOutcome,
} from './tsgo-check.ts';
import { prepareWorkTree, } from './worktree.ts';
import {
  SHARD_SCHEMA_VERSION,
  type ShardManifest,
  type ShardReport,
} from '../shard-schema.ts';

/**
 Module logger for the container entrypoint.
 */
const l = tagged({ tag: 'mutation-test-container', },);

/**
 Manifest file name inside the manifest mount.
 */
export const MANIFEST_FILE_NAME = 'shard-manifest.json';

/**
 Report file name inside the report mount.
 */
export const REPORT_FILE_NAME = 'shard-report.json';

/**
 Generous baseline timeout: the first unmutated run compiles caches and
 warms tsbuildinfo, so per-mutant limits do not apply to it.
 */
const BASELINE_TIMEOUT_MS = 600_000;

/**
 Returns whether a parsed JSON value has the shard manifest shape.
 
 Checks the discriminating fields (version, id, package, arrays); the
 host is the only writer, so field-level validation stays shallow.
 
 @param value - Parsed JSON value.
 
 @returns Whether value is a usable manifest.
 
 @example
 ```ts
 isShardManifest(JSON.parse(raw));
 ```
 */
function isShardManifest(value: unknown,): value is ShardManifest {
  if (!isRecord(value,))
    return false;

  /**
   Record view over the candidate manifest.
   */
  const record = value;
  return (record.schemaVersion === SHARD_SCHEMA_VERSION)
    && ((typeof record.shardId) === 'string')
    && ((typeof record.packagePath) === 'string')
    && Array.isArray(record.mutants,)
    && Array.isArray(record.tests,)
    && ((typeof record.timeoutFloorMs) === 'number')
    && ((typeof record.timeoutFactor) === 'number');
}

/**
 Reads and validates the mounted shard manifest.
 
 @returns Parsed manifest.
 
 @throws Error when the manifest is missing or has a wrong shape.
 
 @example
 ```ts
 const manifest = await readManifest();
 ```
 */
export async function readManifest(): Promise<ShardManifest> {
  /**
   Raw manifest JSON text from the mount.
   */
  const raw = await readFile(
    join(
      MANIFEST_MOUNT,
      MANIFEST_FILE_NAME,
    ),
    'utf8',
  );
  /**
   Parsed manifest before shape validation.
   */
  const parsed: unknown = JSON.parse(raw,);

  if (!isShardManifest(parsed,))
    throw new Error(
      `manifest at ${MANIFEST_MOUNT}/${MANIFEST_FILE_NAME} has unsupported shape or version; expected schema ${String(SHARD_SCHEMA_VERSION,)}`,
    );

  return parsed;
}

/**
 Writes the shard report to the report mount.
 
 @param report - Completed shard report.
 
 @mutates report - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 await writeReport(report);
 ```
 */
async function writeReport(report: ShardReport,): Promise<void> {
  /**
   Report destination path on the report mount.
   */
  const reportPath = join(
    REPORT_MOUNT,
    REPORT_FILE_NAME,
  );
  await mkdir(
    dirname(reportPath,),
    { recursive: true, },
  );
  await writeFile(
    reportPath,
    `${JSON.stringify(
      report,
      null,
      2,
    )}\n`,
    'utf8',
  );
}

/**
 Executes the full container-side shard flow.
 
 @example
 ```ts
 await runShard();
 ```
 */
export async function runShard(): Promise<void> {
  /**
   Logger scoped to the shard run.
   */
  const rl = tagged({
    tag: runShard.name,
    l,
  },);
  /**
   Mounted shard manifest.
   */
  const manifest = await readManifest();
  rl.info(`shard ${manifest.shardId}: ${String(manifest.mutants
    .length,)} mutants`,);

  await prepareWorkTree();

  /**
   Target package working directory inside the work tree.
   */
  const packageCwd = join(
    WORK_MOUNT,
    manifest.packagePath,
  );
  /**
   Baseline build materialising dist output and declarations before
   the type gate and the output-importing tests.
   */
  const baselineBuild = await runBuildStep({ packageCwd, },);
  /**
   Baseline type check, also warming the incremental build info.
   */
  const baselineTsgo: TsgoOutcome = baselineBuild.clean
    ? await tsgoCheck({ cwd: packageCwd, },)
    : {
      clean: false,
      durationMs: 0,
      detail: baselineBuild.detail,
    };
  /**
   Baseline test run over the selected tests.
   */
  const baselineTests: TestRunOutcome = await runTests({
    cwd: packageCwd,
    tests: manifest.tests,
    timeoutMs: BASELINE_TIMEOUT_MS,
  },);
  /**
   Whether the unmutated package passes its own gates.
   */
  const green = baselineBuild.clean
    && baselineTsgo.clean
    && (baselineTests.kind === 'passed');

  if (!green) {
    rl.error(
      `red baseline: tsgo clean=${String(baselineTsgo.clean,)}, tests=${baselineTests.kind}`,
    );
    await writeReport({
      schemaVersion: SHARD_SCHEMA_VERSION,
      shardId: manifest.shardId,
      baseline: {
        green: false,
        testsMs: baselineTests.durationMs,
        tsgoMs: baselineTsgo.durationMs,
        detail: `${baselineTsgo.detail} ${baselineTests.detail}`.trim(),
      },
      results: [],
      unrun: manifest.mutants
        .map(function toId(mutant,): string {
          return mutant.id;
        },),
      anomaly: 'red baseline: unmutated package failed its own gates',
    },);
    return;
  }

  /**
   Mutant loop output for this shard.
   */
  const loop = await runMutantLoop({
    packageCwd,
    manifest,
    baselineTestsMs: baselineTests.durationMs,
  },);

  await writeReport({
    schemaVersion: SHARD_SCHEMA_VERSION,
    shardId: manifest.shardId,
    baseline: {
      green: true,
      testsMs: baselineTests.durationMs,
      tsgoMs: baselineTsgo.durationMs,
      detail: '',
    },
    results: loop.results,
    unrun: loop.unrun,
    anomaly: loop.anomaly,
  },);
  rl.info(
    `shard ${manifest.shardId} done: ${String(loop.results
      .length,)} results, ${String(loop.unrun
        .length,)} unrun`,
  );
}

/**
 Whether this module is running as the process entrypoint.
 */
const isDirectEntrypoint = (process.argv[1] !== undefined)
  && (import.meta.url
    === pathToFileURL(process.argv[1],)
      .href);

if (isDirectEntrypoint)
  await runShard();
