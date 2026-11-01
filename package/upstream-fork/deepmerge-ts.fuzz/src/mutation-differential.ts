/**
 In-container differential check of mutant bundles against the unmutated
 bundle, run by the `mutation:differential` mise task after
 `mutation:sweep` wrote `/out/mutants/`.

 Each mutant runs the same fixed-seed cases as the baseline in six
 categories (alias graphs through `deepmerge` and `deepmergeInto`, alias
 graphs with option plans through both custom entry points, exotic arguments
 through the four default entry points, and trees with FastUnsafe option
 plans), and each outcome is compared by `./mutation-canon.ts`. A mutant
 that never differs is only "not separated here"; calling it equivalent
 still needs a source argument (see the audit report).

 Two controls run first and must hold, or the task fails: the baseline
 against itself shows no difference, and the control mutant (one the
 sidecar detects) shows one.

 @module
 */


import { appendFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import { outcomeOf, } from './mutation-canon.ts';
import {
  MutationStepError,
  OUT,
} from './mutation-container.ts';
import {
  CASES,
  type Library,
} from './mutation-differential-case.ts';

/**
 Characters of each differing outcome kept in the report.
 */
const OUTCOME_EXCERPT = 600;


/**
 Load one bundle written by the sweep.

 @param id - Mutant id, or `baseline`.

 @returns The bundle's exports.

 @example
 ```ts
 const baseline = await loadBundle('baseline');
 ```
 */
async function loadBundle(id: string,): Promise<Library> {
  /**
   Module namespace of the bundle.
   */
  const loaded: unknown = await import(pathToFileURL(join(
    OUT,
    'mutants',
    `${id}.mjs`,
  ),)
    .href);
  if (((typeof loaded) !== 'object') || (loaded === null))
    throw new MutationStepError(`bundle ${id} did not load as a module`,);
  return loaded;
}

/**
 Result of comparing two bundles: the same on every case, or the first case
 where their outcomes differ.
 */
type Comparison = {
  readonly kind: 'same';
} | {
  readonly kind: 'differs';
  readonly category: string;
  readonly index: number;
  readonly expected: string;
  readonly actual: string;
};

/**
 First case where two bundles leave different outcomes.

 @param left - Reference bundle.

 @param right - Bundle under test.

 @param runs - Cases per category.

 @returns `same`, or the category, case index, and both outcomes of the first difference.

 @example
 ```ts
 firstDifference({ left: baseline, right: mutant, runs: 1000, });
 ```
 */
function firstDifference(
  {
    left,
    right,
    runs,
  }: {
    readonly left: Library;
    readonly right: Library;
    readonly runs: number;
  },
): Comparison {
  for (const differentialCase of CASES) {
    /**
     Reference calls.
     */
    const expectedCalls = differentialCase.calls({
      library: left,
      runs,
    },);
    /**
     Calls under test, drawn fresh from the same seed.
     */
    const actualCalls = differentialCase.calls({
      library: right,
      runs,
    },);
    for (const [index, expectedCall,] of expectedCalls.entries()) {
      /**
       Matching call under test.
       */
      const actualCall = actualCalls[index];
      if (actualCall === undefined)
        throw new MutationStepError(`category ${differentialCase.name} drew fewer cases for the second bundle`,);
      /**
       Reference outcome.
       */
      const expected = outcomeOf(expectedCall,);
      /**
       Outcome under test.
       */
      const actual = outcomeOf(actualCall,);
      if (expected !== actual) {
        return {
          actual: actual.slice(
            0,
            OUTCOME_EXCERPT,
          ),
          category: differentialCase.name,
          expected: expected.slice(
            0,
            OUTCOME_EXCERPT,
          ),
          index,
          kind: 'differs',
        };
      }
    }
  }
  return { kind: 'same', };
}

/**
 Run both controls, then compare every requested mutant with the baseline.

 @param runs - Cases per category.

 @param controlId - Mutant the sidecar detects; it must differ.

 @param ids - Mutants to compare.

 @throws {@link MutationStepError} When a control fails.

 @example
 ```ts
 await runDifferential({ controlId: '34', ids: ['35',], runs: 1000, });
 ```
 */
export async function runDifferential(
  {
    runs,
    controlId,
    ids,
  }: {
    readonly runs: number;
    readonly controlId: string;
    readonly ids: readonly string[];
  },
): Promise<void> {
  /**
   Unmutated bundle.
   */
  const baseline = await loadBundle('baseline',);
  if (firstDifference({
    left: baseline,
    right: baseline,
    runs,
  },)
    .kind
    !== 'same')
    throw new MutationStepError('control failed: the baseline differs from itself, so outcomes are not deterministic',);
  if (firstDifference({
    left: baseline,
    right: await loadBundle(controlId,),
    runs,
  },)
    .kind
    === 'same')
    throw new MutationStepError(`control failed: mutant ${controlId} shows no difference, so the check cannot separate mutants`,);
  console.log(`controls passed (baseline stable, mutant ${controlId} separated) at ${String(runs,)} cases per category`,);
  for (const id of ids) {
    /**
     Bundle for this mutant.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- bundles load one at a time to keep memory flat.
    const mutant = await loadBundle(id,);
    /**
     Comparison with the baseline.
     */
    const comparison = firstDifference({
      left: baseline,
      right: mutant,
      runs,
    },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- one log line per mutant, in request order.
    await appendFile(
      join(
        OUT,
        'differential.jsonl',
      ),
      `${JSON.stringify({
        comparison,
        id,
        runs,
      },)}\n`,
    );
    console.log(`${id}\t${(comparison.kind === 'same') ? 'no difference' : `differs in ${comparison.category} case ${String(comparison.index,)}`}`,);
  }
}

if (import.meta.main) {
  /**
   Cases per category, control mutant, and mutants to compare, from the task.
   */
  const [runsText, controlId, ...ids] = process.argv
    .slice(2,);
  /**
   Cases per category.
   */
  const runs = Number(runsText,);
  if ((!Number.isInteger(runs,)) || (runs <= 0)
    || (controlId === undefined))
    throw new MutationStepError('usage: mutation-differential.ts <runs> <control mutant id> [mutant ids...]',);
  await runDifferential({
    controlId,
    ids,
    runs,
  },);
}
