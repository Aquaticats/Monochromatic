import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { BenchSlice, } from './bench-sample.ts';
import type { ArmOutcome, } from './editor-width-arm.ts';
import { bothOrders, } from './editor-width-contest.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width control
// Whether the panel can see damage at all, asked before the draw spends
// anything.
//
// A null result is only evidence from an instrument shown able to produce a
// positive one. This panel is about to be asked which of two repairs of the
// same passage reads better, a fine judgement; if it cannot first prefer an
// intact passage over the same passage with a whole sentence removed, then
// "the two widths tied" means the panel is blind, not that the widths are
// equal, and an hour of calls would buy an unreadable number.
//
// RUN ON SEVERAL SLICES, not one. A single pair decided by a coin is
// indistinguishable from a panel that works, and this check exists precisely
// to not be fooled by one draw.

/**
 Slices the control is tried on.

 Three rather than one so a single unlucky pair cannot condemn a working
 panel, and rather than ten because this is a gate on spending, not a
 measurement in its own right.
 */
const CONTROL_SLICES = 3;

/**
 Sentence terminators the damage cut looks for, longest first so a full-width
 stop is not mistaken for an ASCII one.
 */
const TERMINATORS = [
  '。',
  '. ',
  '! ',
  '? ',
  // A sentence that ends its line, which a passage written one sentence per
  // line is made of: without these the cut read such a passage as one
  // sentence and the slice left the positive control.
  '.\n',
  '!\n',
  '?\n',
];

/**
 Removes one whole sentence, or reports that there was none to remove.

 DELETION RATHER THAN CORRUPTION is the damage of choice because
 dropped page content is already a named fault the pipeline cares about, so a
 panel that misses it is missing something the corpus rules already say
 matters.

 Exported so the cut can be tested directly. This function decides what the
 positive control is actually asking the panel about, and a version of it that
 quietly returned the passage unchanged, or blank on everything, would turn the
 gate into a formality that passes whatever it is handed.

 @internal

 @param text - passage to damage

 @returns Passage with a sentence gone, or blank when it holds only one

 @example
 ```ts
 const damaged = withoutASentence(text,);
 ```
 */
export function withoutASentence(text: string,): string {
  /**
   Index just past each terminator that appears, as a plain number so no
   accumulator object rides through the scan.
   */
  const ends = TERMINATORS
    .map(function endOf(terminator,): number {
      /**
       Where this terminator first appears.
       */
      const at = text.indexOf(terminator,);

      if (at === (-1))
        return text.length;

      return at + terminator.length;
    },)
    .filter(function leavesSomething(end,): boolean {
      // An end at the very end of the passage means the terminator closed the
      // only sentence, so cutting there leaves nothing to judge. That is a
      // different question from whether the panel notices a deletion, and the
      // caller filters these slices out rather than asking about them.
      return end < text.length;
    },);

  if (ends.length === 0)
    return '';

  return text
    .slice(Math.min(...ends,),)
    .trim();
}

/**
 Presents one text as an arm, so the contest machinery can judge it.

 @param text - passage this arm offers

 @param producers - models credited with it, empty for fixture text nobody
 wrote, which keeps every ballot at full weight

 @returns Arm the contest can seat

 @example
 ```ts
 const arm = asArm({ text, producers: [], },);
 ```
 */
function asArm(
  {
    text,
    producers,
  }: {
    readonly text: string;
    readonly producers: readonly RosterModelId[];
  },
): ArmOutcome {
  return {
    text,
    patch: {
      patchedText: text,
      applied: [],
      rejected: [],
    },
    heard: 0,
    producers,
  };
}

/**
 Asks whether the panel prefers intact text over text missing a sentence.

 @param client - injected model client

 @param slices - drawn sample, of which the first usable few are damaged

 @param judgeModelIds - panel the draw will use

 @param signal - cancellation

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Whether intact text won more of the tried pairs than it lost

 @example
 ```ts
 const held = await widthControlHolds({ client, slices, judgeModelIds, signal, l, stageSettings, },);
 ```
 */
export async function widthControlHolds(
  {
    client,
    slices,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly slices: readonly BenchSlice[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<boolean> {
  /**
   Slices with more than one sentence, which are the only ones a deletion can
   be cut from.
   */
  const usable = slices
    .filter(function damageable(slice,) {
      return withoutASentence(slice.incumbentText,) !== '';
    },)
    .slice(
    0,
    CONTROL_SLICES,
    );

  if (usable.length === 0)
    throw new StatedRefusalError({
      says: 'editor width control refused: no drawn slice holds more than one sentence, so no '
        + 'deletion could be cut and the panel was never asked anything',
    },);

  /**
   Verdict on each pair, gathered so the count is a read over results rather
   than a counter mutated inside the loop.
   */
  const verdicts: string[] = [];

  for (const slice of usable) {
    /**
     Intact passage, seated as the narrow arm.
     */
    const intact = asArm({
      text: slice.incumbentText,
      producers: [],
    },);

    /**
     Same passage with a sentence gone, seated as the wide arm.
     */
    const damaged = asArm({
      text: withoutASentence(slice.incumbentText,),
      producers: [],
    },);

    /* oxlint-disable eslint/no-await-in-loop -- sequential by design: this gate exists to stop a run before it spends, so the pairs must resolve one at a time and be allowed to answer the question early */
    /**
     Which the panel preferred, over both seatings.
     */
    const verdict = await bothOrders({
      client,
      input: {
        entryId: slice.entryId,
        sliceIndex: slice.index,
        sourceText: slice.sourceText,
        targetText: slice.incumbentText,
        issues: [],
        envelopes: [],
        findings: [],
      },
      narrow: intact,
      wide: damaged,
      judgeModelIds,
      signal,
      l,
      stageSettings,
    },);
    /* oxlint-enable eslint/no-await-in-loop */

    verdicts.push(verdict.verdict,);

    console.log(
      `WIDTH control ${slice.entryId} slice ${String(slice.index,)}: ${verdict.verdict}`,
    );
  }

  /**
   Pairs where intact text won both seatings.
   */
  const intactPreferred = verdicts
    .filter(function intactWon(verdict,) {
      return verdict === 'narrow-wins';
    },)
    .length;

  // A MAJORITY RATHER THAN UNANIMITY. One pair may genuinely read better
  // shorter, and demanding a clean sweep would fail a working panel on a
  // passage whose first sentence was redundant.
  return (intactPreferred * 2) > usable.length;
}

//endregion Editor width control
