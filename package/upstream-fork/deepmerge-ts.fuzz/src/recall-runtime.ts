/**
 In-container historical-recall run for runtime bugs
 (`doc/audit/deepmerge-ts-recall-2026-09-24.md`), in the image built by
 `../container/mutation.Containerfile` with upstream trees at `/trees`.

 For each row of `./recall-ledger-runtime.ts` (or the ids given): build its
 buggy and fixed bundles (`./recall-bundle.ts`), require its control to be
 `true` on the buggy bundle and `false` on the fixed one, run every sidecar
 unit file against both, and keep failures only the buggy bundle shows. When
 no specifying file (see `isPinningFile`) detects the bug, bounded campaign
 rounds of every property file follow. One JSON line per row goes to
 `/out/runtime.jsonl`.

 @module
 */

import {
  appendFile,
  glob,
  mkdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { isPinningFile, } from './mutation-mutant.ts';
import {
  BASELINE_TREE,
  buildPair,
  bundle,
  controlOn,
  OUT,
  RecallRunError,
  TREES,
} from './recall-bundle.ts';
import { attributable, } from './recall-edit.ts';
import type { RuntimeBug, } from './recall-ledger.ts';
import { RUNTIME_BUGS, } from './recall-ledger-runtime.ts';
import {
  type FileRun,
  type Layer,
  runFiles,
} from './recall-runner.ts';

/**
 First seed of the bounded campaign rounds; any fixed value keeps runs reproducible.
 */
const FIRST_CAMPAIGN_SEED = 11;

/**
 Seeds of the bounded campaign rounds run when the bounded layer misses.
 */
const CAMPAIGN_SEEDS: readonly number[] = Array.from(
  { length: 5, },
  function seedAt(
    _unused: unknown,
    index: number,
  ) {
    return FIRST_CAMPAIGN_SEED + index;
  },
);

/**
 Runs per property in each bounded campaign round.
 */
const CAMPAIGN_RUNS = 2_000;

/**
 Runs already made, keyed by bundle and layer, so the shared baseline bundle
 runs once per layer.
 */
const runCache = new Map<string, readonly FileRun[]>();

/**
 Run files against a bundle once per layer.

 @param files - Sidecar files.

 @param target - Bundle path.

 @param layer - Bounded layer or campaign round.

 @returns Runs in file order.

 @example
 ```ts
 await cachedRuns({ files, layer: { kind: 'bounded', }, target, });
 ```
 */
async function cachedRuns(
  {
    files,
    target,
    layer,
  }: {
    readonly files: readonly string[];
    readonly target: string;
    readonly layer: Layer;
  },
): Promise<readonly FileRun[]> {
  /**
   Cache key.
   */
  const key = JSON.stringify([
    target,
    layer,
    files,
  ],);
  /**
   Earlier runs, when present.
   */
  const cached = runCache.get(key,);
  if (cached !== undefined)
    return cached;
  /**
   Fresh runs.
   */
  const runs = await runFiles({
    files,
    layer,
    target,
  },);
  runCache.set(
    key,
    runs,
  );
  return runs;
}

/**
 Failures per file that only the buggy bundle shows.

 @param buggy - Runs against the buggy bundle.

 @param fixed - Runs against the fixed bundle, same files and order.

 @returns Files with at least one buggy-only failure.

 @example
 ```ts
 detections({ buggy, fixed, });
 ```
 */
function detections(
  {
    buggy,
    fixed,
  }: {
    readonly buggy: readonly FileRun[];
    readonly fixed: readonly FileRun[];
  },
): readonly {
  readonly file: string;
  readonly failures: readonly string[];
}[] {
  return buggy.flatMap(function perFile(run,) {
    /**
     Failures of the same file on the fixed bundle.
     */
    const fixedRun = fixed.find(function sameFile(other,) {
      return other.file === run.file;
    },);
    /**
     Buggy-only failures.
     */
    const failures = attributable({
      buggy: run.failures,
      fixed: fixedRun?.failures ?? [],
    },);
    return (failures.length === 0) ? [] : [{
      failures,
      file: run.file,
    },];
  },);
}

/**
 Run one row and append its record.

 @param bug - Ledger row.

 @throws {@link RecallRunError} When the control does not separate the bundles.

 @example
 ```ts
 await recallOne(RUNTIME_BUGS[0]);
 ```
 */
async function recallOne(bug: RuntimeBug,): Promise<void> {
  /**
   Buggy and fixed bundles.
   */
  const pair = await buildPair(bug,);
  /**
   Control on each bundle.
   */
  const control = {
    buggy: await controlOn({
      bug,
      path: pair.buggy,
    },),
    fixed: await controlOn({
      bug,
      path: pair.fixed,
    },),
  };
  if ((control.buggy !== 'true') || (control.fixed !== 'false'))
    throw new RecallRunError(`control for ${bug.id} does not separate the bundles: ${JSON.stringify(control,)}`,);
  /**
   Every sidecar unit file.
   */
  const files = (await Array.fromAsync(glob('src/*.unit.test.ts',),)).toSorted();
  /**
   Bounded-layer runs against the fixed bundle; any failure here is recorded,
   since the attribution below cannot see a bug both bundles share.
   */
  const fixedRuns = await cachedRuns({
    files,
    layer: { kind: 'bounded', },
    target: pair.fixed,
  },);
  /**
   Bounded-layer detections.
   */
  const bounded = detections({
    buggy: await cachedRuns({
      files,
      layer: { kind: 'bounded', },
      target: pair.buggy,
    },),
    fixed: fixedRuns,
  },);
  /**
   Whether a file that specifies behaviour, not one pinning it, detected the bug.
   */
  const specified = bounded.some(function specifies(entry,) {
    return !isPinningFile(entry.file,);
  },);
  /**
   Property files for the campaign rounds.
   */
  const propertyFiles = files.filter(function isProperty(file,) {
    return file.endsWith('.property.unit.test.ts',);
  },);
  /**
   Campaign detections per seed, run only when the bounded layer missed.
   */
  const campaign: {
    readonly seed: number;
    readonly detections: ReturnType<typeof detections>;
  }[] = [];
  for (const seed of (specified ? [] : CAMPAIGN_SEEDS)) {
    /**
     Round layer.
     */
    const layer: Layer = {
      kind: 'round',
      numRuns: CAMPAIGN_RUNS,
      seed,
    };
    campaign.push({
      detections: detections({
        // oxlint-disable-next-line eslint/no-await-in-loop -- rounds run one at a time; each already uses both CPUs.
        buggy: await cachedRuns({
          files: propertyFiles,
          layer,
          target: pair.buggy,
        },),
        // oxlint-disable-next-line eslint/no-await-in-loop -- rounds run one at a time; each already uses both CPUs.
        fixed: await cachedRuns({
          files: propertyFiles,
          layer,
          target: pair.fixed,
        },),
      },),
      seed,
    },);
  }
  /**
   Record for this row.
   */
  const record = {
    bounded,
    campaign,
    commit: bug.commit,
    control,
    fixedFailing: fixedRuns
      .filter(function failed(run,) {
        return run.failures
          .length
          > 0;
      },)
      .map(function fileOf(run,) {
        return run.file;
      },),
    id: bug.id,
    specified,
    version: bug.version,
  };
  await appendFile(
    join(
      OUT,
      'runtime.jsonl',
    ),
    `${JSON.stringify(record,)}\n`,
  );
  console.log(`${bug.id}\tbounded ${String(bounded.length,)} file(s)\tspecified ${String(specified,)}\tcampaign ${campaign
    .map(function countOf(entry,) {
      return `${String(entry.seed,)}:${String(entry.detections
        .length,)}`;
    },)
    .join(' ',)}`,);
}

if (import.meta.main) {
  /**
   Row ids to run; empty runs every row.
   */
  const ids = process.argv
    .slice(2,);
  await mkdir(
    join(
      OUT,
      'bundles',
    ),
    { recursive: true, },
  );
  await bundle({
    outfile: join(
      OUT,
      'bundles',
      'baseline.mjs',
    ),
    srcDir: join(
      TREES,
      BASELINE_TREE,
      'src',
    ),
  },);
  /**
   Rows selected by id.
   */
  const selected = RUNTIME_BUGS.filter(function wanted(bug,) {
    return (ids.length === 0) || ids.includes(bug.id,);
  },);
  for (const bug of selected) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- rows run one at a time; each already uses both CPUs.
    await recallOne(bug,);
  }
}
