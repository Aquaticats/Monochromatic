/**
 Summarize the type recall results (`./recall-types.ts`) per ledger row:
 whether the control separated the releases, and which sidecar layers show
 diagnostics only against the buggy release. Runs on the host over
 `dist/recall/types/results/*.json`; prints JSON.

 Only files listed in `dist/recall/baseline-files.txt` (the sidecar as it
 stood when the audit started, from `git ls-tree`) and the fresh
 declared-type draw count, so files the audit itself added never inflate
 recall.

 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { attributable, } from './recall-edit.ts';
import { fieldOf, } from './recall-json.ts';
import { TYPE_BUGS, } from './recall-ledger-type.ts';

/**
 Result directory.
 */
const RESULTS = resolve(
  import.meta.dirname,
  '..',
  'dist',
  'recall',
  'types',
  'results',
);

/**
 Sidecar layer a diagnostic's file belongs to.

 @param key - Diagnostic key, `file(line,col): TSnnnn`.

 @returns Layer name.

 @example
 ```ts
 layerOf('src/type-leaf.unit.test.ts(3,1): TS2322'); // 'hand-written expectTypeOf'
 ```
 */
function layerOf(key: string,): string {
  /**
   File part of the key.
   */
  const file = key.split('(',)[0] ?? '';
  if (file.endsWith('fresh/cases.ts',))
    return 'fresh declared-type draw (seed 20260930, 1500 cases)';
  if (file.endsWith('declared-type-soundness.generated.ts',))
    return 'committed declared-type corpus';
  if (file.endsWith('type-soundness.generated.ts',))
    return 'committed inferred-literal corpus';
  if (file.includes('known-defect',))
    return 'type known-defect pins';
  if (file.includes('/type-',))
    return 'hand-written expectTypeOf';
  return 'other sidecar files';
}

/**
 Read one job's outcome, or `missing` when the job never ran.

 @param job - Job name.

 @returns Outcome fields the report needs.

 @example
 ```ts
 await readJob('sidecar-7.1.5');
 ```
 */
async function readJob(job: string,): Promise<{
  readonly state: 'missing' | 'ran' | 'timeout';
  readonly diagnostics: readonly string[];
  readonly ms: number;
}> {
  try {
    /**
     Parsed outcome.
     */
    const parsed: unknown = JSON.parse(await readFile(
      join(
        RESULTS,
        `${job}.json`,
      ),
      'utf8',
    ),);
    /**
     Diagnostics field.
     */
    const diagnostics: unknown = fieldOf({
      key: 'diagnostics',
      value: parsed,
    },);
    return {
      diagnostics: Array.isArray(diagnostics,) ? diagnostics.map(String,) : [],
      ms: Number(fieldOf({
        key: 'ms',
        value: parsed,
      },),),
      state: (fieldOf({
        key: 'timedOut',
        value: parsed,
      },) === true) ? 'timeout' : 'ran',
    };
  } catch (error) {
    if (Error.isError(error,)
      && error.message
        .includes('ENOENT',)) {
      return {
        diagnostics: [],
        ms: 0,
        state: 'missing',
      };
    }
    throw error;
  }
}

if (import.meta.main) {
  /**
   Sidecar files present when the audit started, relative to the package.
   */
  const baseline = new Set((await readFile(
    join(
      RESULTS,
      '..',
      '..',
      'baseline-files.txt',
    ),
    'utf8',
  ))
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },),);
  /**
   One summary per row.
   */
  const rows = await Promise.all(TYPE_BUGS.map(async function summarize(bug,) {
    /**
     Control outcomes.
     */
    const control = {
      buggy: await readJob(`control-${bug.id}-buggy`,),
      fixed: await readJob(`control-${bug.id}-fixed`,),
    };
    /**
     Sidecar outcomes.
     */
    const sidecar = {
      buggy: await readJob(`sidecar-${bug.buggy}`,),
      fixed: await readJob(`sidecar-${bug.fixed}`,),
    };
    /**
     Buggy-only diagnostics.
     */
    const only = attributable({
      buggy: sidecar.buggy
        .diagnostics,
      fixed: sidecar.fixed
        .diagnostics,
    },)
      .filter(function inBaseline(key,) {
        /**
         File part of the key.
         */
        const file = key.split('(',)[0] ?? '';
        return baseline.has(file,) || file.endsWith('fresh/cases.ts',);
      },);
    /**
     Buggy-only diagnostic count per layer.
     */
    const layers = only.reduce<Record<string, number>>(
      function tally(
        acc,
        key,
      ) {
        acc[layerOf(key,)] = (acc[layerOf(key,)] ?? 0) + 1;
        return acc;
      },
      {},
    );
    return {
      buggy: bug.buggy,
      control: (bug.control === '') ? 'none' : (((control.buggy
        .diagnostics
        .length
        > 0) && (control.fixed
          .diagnostics
          .length
          === 0)) ? 'separates' : 'does not separate'),
      fixed: bug.fixed,
      id: bug.id,
      layers,
      sidecar: {
        buggy: `${sidecar.buggy
          .state} ${String(sidecar.buggy
            .diagnostics
            .length,)} in ${String(sidecar.buggy
              .ms,)} ms`,
        fixed: `${sidecar.fixed
          .state} ${String(sidecar.fixed
            .diagnostics
            .length,)} in ${String(sidecar.fixed
              .ms,)} ms`,
      },
    };
  },),);
  console.log(JSON.stringify(
    rows,
    undefined,
    2,
  ),);
}
