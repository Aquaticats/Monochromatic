/**
 Declared-type campaign and committed-corpus builder.

 `campaign <seed> <size>` draws cases, type-checks them under
 `dist/declared-type/<seed>/`, and writes `report.json` there: every failing
 case's source and diagnostics, plus counts per diagnostic code and entry
 point. Run large campaigns inside the podman caps (see the package README's
 heavy-run rule).

 `corpus` draws the fixed-seed corpus, drops the cases that fail (each
 failure belongs to a pinned class in `./type-known-defect.unit.test.ts` or
 `./type-known-defect-declared.unit.test.ts`; the report records them),
 re-checks the remainder, and writes `./declared-type-soundness.generated.ts`,
 which `lint:types` then checks on every run.

 ```sh
 node src/declared-type-campaign.ts campaign 123 2000
 node src/declared-type-campaign.ts corpus
 ```

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  checkCases,
  emitModule,
  type CaseFailure,
  type RunCase,
} from './declared-type-check.ts';
import { drawCases, } from './declared-type-generate.ts';

/**
 Seed of the committed corpus.
 */
const CORPUS_SEED = 20_260_924;

/**
 Draws for the committed corpus, before failing cases are dropped.
 */
const CORPUS_DRAWS = 500;

/**
 Package root, resolved from this file.
 */
const PACKAGE_ROOT = resolve(
  import.meta.dirname,
  '..',
);

/**
 Count failures per diagnostic code and per entry point.

 @param failures - Failing cases.
 
 @param cases - All cases, for their entry points.

 @returns Code and entry-point tallies.

 @example
 ```ts
 const summary = summarize({ failures, cases, });
 ```
 */
function summarize(
  {
    failures,
    cases,
  }: {
    readonly failures: readonly CaseFailure[];
    readonly cases: readonly RunCase[]
  },
): Readonly<Record<string, number>> {
  /**
   Tally keyed by `code` or `kind:option`.
   */
  const tally = new Map<string, number>();
  for (const failure of failures) {
    /**
     Case behind this failure.
     */
    const drawn = cases[failure.id]
      ?.drawn;
    /**
     Distinct diagnostic codes in this case.
     */
    const codes = new Set(failure.diagnostics
      .map(function codeOf(diagnostic,) {
        return diagnostic.slice(diagnostic.indexOf('error TS',) + 'error '.length,)
          .split(':',)[0]
          ?? 'unknown';
      },),);
    for (const key of [
      ...codes,
      `${drawn?.kind ?? '?'}:${drawn?.option ?? '?'}`,
    ])
      tally.set(
        key,
        (tally.get(key,) ?? 0) + 1,
      );
  }
  return Object.fromEntries([...tally,].toSorted(function byCount(
    left,
    right,
  ) {
    return right[1] - left[1];
  },),);
}

/**
 Run one campaign and write its report.

 @param seed - fast-check seed.
 
 @param size - Number of draws.
 
 @param unions - Whether declared types may contain unions.

 @returns Path of the written report.

 @example
 ```ts
 await runCampaign({ seed: 1, size: 100, unions: true, });
 ```
 */
export async function runCampaign(
  {
    seed,
    size,
    unions,
  }: {
    readonly seed: number;
    readonly size: number;
    readonly unions: boolean
  },
): Promise<string> {
  /**
   Scratch directory of this campaign.
   */
  const dir = join(
    PACKAGE_ROOT,
    'dist',
    'declared-type',
    `${String(seed,)}${unions ? '' : '-no-unions'}`,
  );
  /**
   Drawn and run cases.
   */
  const cases = drawCases({
    seed,
    size,
    unions,
  },);
  /**
   Type-check outcome.
   */
  const {
    failures,
    stray,
  } = await checkCases({
    cases,
    dir,
  },);
  /**
   Report path.
   */
  const reportPath = join(
    dir,
    'report.json',
  );
  await writeFile(
    reportPath,
    `${JSON.stringify(
      {
        cases: cases.length,
        failing: failures.length,
        failures,
        seed,
        stray,
        summary: summarize({
          cases,
          failures,
        },),
      },
      undefined,
      2,
    )}\n`,
  );
  console.log(`seed ${String(seed,)}: ${String(failures.length,)}/${String(cases.length,)} cases fail, ${String(stray.length,)} stray diagnostics; ${reportPath}`,);
  return reportPath;
}

/**
 Build the committed corpus: draw, drop failing cases, re-check, write.

 @throws When the filtered corpus still fails, which means cases interact.

 @example
 ```ts
 await buildCorpus();
 ```
 */
export async function buildCorpus(): Promise<void> {
  /**
   Scratch directory of the corpus build.
   */
  const dir = join(
    PACKAGE_ROOT,
    'dist',
    'declared-type',
    'corpus',
  );
  /**
   Fixed-seed cases.
   */
  const cases = drawCases({
    seed: CORPUS_SEED,
    size: CORPUS_DRAWS,
    unions: true,
  },);
  /**
   Failures of the unfiltered draw.
   */
  const first = await checkCases({
    cases,
    dir,
  },);
  if (first.stray
    .length
    > 0)
    throw new Error(`corpus header or emitter produced diagnostics:\n${first.stray
      .join('\n',)}`,);
  /**
   Ids of failing cases, dropped from the corpus.
   */
  const dropped = new Set(first.failures
    .map(function idOf(failure,) {
      return failure.id;
    },),);
  /**
   Cases that type-check.
   */
  const kept = cases.filter(function passes(
    _case,
    id,
  ) {
    return !dropped.has(id,);
  },);
  /**
   Re-check of the kept cases in isolation.
   */
  const second = await checkCases({
    cases: kept,
    dir: join(
      dir,
      'kept',
    ),
  },);
  if ((second.failures
    .length
    > 0) || (second.stray
      .length
      > 0))
    throw new Error(`filtered corpus still fails in ${String(second.failures
      .length,)} cases`,);
  await mkdir(
    dir,
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      'dropped.json',
    ),
    `${JSON.stringify(
      first.failures,
      undefined,
      2,
    )}\n`,
  );
  /**
   Committed module, with the drop count in its first line.
   */
  const emitted = emitModule({
    cases: kept,
    exportName: 'DECLARED_TYPE_CASES',
  },);
  await writeFile(
    join(
      PACKAGE_ROOT,
      'src',
      'declared-type-soundness.generated.ts',
    ),
    `// Seed ${String(CORPUS_SEED,)}: ${String(kept.length,)} of ${String(cases.length,)} draws kept; ${String(dropped.size,)} failing draws dropped (pinned classes, see dist/declared-type/corpus/dropped.json after a rebuild).\n${emitted.source}`,
  );
  console.log(`corpus: kept ${String(kept.length,)}, dropped ${String(dropped.size,)}`,);
}

if (import.meta.main) {
  /**
   Mode and its arguments.
   */
  const [mode, seedText, sizeText, unionsText,] = process.argv
    .slice(2,);
  if (mode === 'corpus')
    await buildCorpus();
  else if (mode === 'campaign')
    await runCampaign({
      seed: Number(seedText ?? Date.now(),),
      size: Number(sizeText ?? '1000',),
      unions: unionsText !== 'no-unions',
    },);
  else
    throw new Error('usage: node src/declared-type-campaign.ts <campaign <seed> <size> [no-unions] | corpus>',);
}
