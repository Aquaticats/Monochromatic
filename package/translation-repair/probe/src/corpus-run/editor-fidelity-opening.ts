import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { FidelityReferenceSpec, } from '../fidelity-reference-model.ts';
import type { EditorFidelityTrial, } from '../editor-fidelity-trials.ts';
import {
  ceilingLine,
  type EditorCeiling,
} from './editor-fidelity-ceiling.ts';
import { editorDecisionRuleLine, } from './editor-fidelity-plan.ts';
import { poolLine, } from './editor-fidelity-pool-line.ts';
import { guardLine, } from './editor-fidelity-spend-guard.ts';

//region Editor fidelity opening lines
// THE LINES AN ARM WRITES BEFORE IT ASKS ANYTHING, in the order the read step finds them: the manifest, the pool, the
// ceiling of each card, the spend guard and the decision rule. They are one list so that a run that stops before its
// first call, which is what a cap of zero does, writes the very lines a run that goes on to ask would write.

/**
 Digest of the pinned reference specifications a run was asked for, the one figure every arm of a bench must share.

 @param specs - pinned reference specifications the request selected

 @returns Digest of their JSON

 @example
 ```ts
 const digest = manifestDigest({ specs, },);
 ```
 */
export function manifestDigest({ specs, }: { readonly specs: readonly FidelityReferenceSpec[]; },): string {
  return hashContent({ content: JSON.stringify(specs,), },);
}

/**
 Says how many specifications the run was asked for and their digest.

 @param specs - pinned reference specifications the request selected

 @returns Line, without a newline

 @example
 ```ts
 log.info(manifestLine({ specs, },),);
 ```
 */
export function manifestLine({ specs, }: { readonly specs: readonly FidelityReferenceSpec[]; },): string {
  return `manifest: ${String(specs.length,)} ${
    wordForCount({
      count: specs.length,
      one: 'specification',
      many: 'specifications',
    },)
  }, digest ${manifestDigest({ specs, },)}`;
}

/**
 The lines an arm writes before it asks anything.

 @param specs - pinned reference specifications the request selected

 @param trials - trials the run built, before the cap

 @param ceilings - most each card can cost over the trials priced

 @param maxUsd - amount from `--max-usd`

 @param benchSpentUsd - amount from `--bench-spent-usd`

 @returns Manifest, pool, ceiling, guard and rule lines, in that order, without newlines

 @example
 ```ts
 for (const said of openingLines({ specs, trials, ceilings, maxUsd: 3, benchSpentUsd: 0, },))
   log.info(said,);
 ```
 */
export function openingLines(
  {
    specs,
    trials,
    ceilings,
    maxUsd,
    benchSpentUsd,
  }: {
    readonly specs: readonly FidelityReferenceSpec[];
    readonly trials: readonly EditorFidelityTrial[];
    readonly ceilings: readonly EditorCeiling[];
    readonly maxUsd: number;
    readonly benchSpentUsd: number;
  },
): readonly string[] {
  return [
    manifestLine({ specs, },),
    poolLine({ trials, },),
    ...ceilings.map(function lineOf(ceiling,): string {
      return ceilingLine({ ceiling, },);
    },),
    guardLine({
      maxUsd,
      benchSpentUsd,
    },),
    editorDecisionRuleLine(),
  ];
}

//endregion Editor fidelity opening lines
