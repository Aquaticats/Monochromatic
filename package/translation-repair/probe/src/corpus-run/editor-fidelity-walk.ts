import { writeFile, } from 'node:fs/promises';

import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { mapOverlapped, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { EditorFidelityTrial, } from '../editor-fidelity-trials.ts';
import type {
  EditorFidelityRow,
  EditorPair,
} from './editor-fidelity-plan.ts';
import { pairName, } from './editor-fidelity-probe-read.ts';
import { cappedLine, } from './editor-fidelity-progress-lines.ts';
import { fidelityPartialWarning, } from './judge-fidelity-probe-plan.ts';

//region Editor fidelity walk
// THE WALK OVER THE PLANNED PAIRS, one at a time. The spend guard ends an arm by aborting the arm's own signal with its
// refusal as the reason; the walk turns that one ending into a result, the rows that finished and the guard's sentence, so
// an arm the guard capped is read as an arm that walked fewer trials and not as a failed one. Any other throw stays one.

/**
 One ask of the walk: a trial in one pass over the pool.

 @example
 ```ts
 const ask: PassAsk = { trial, pass: 1, };
 ```
 */
export type PassAsk = {
  /**
   The trial asked.
   */
  readonly trial: EditorFidelityTrial;

  /**
   Which pass over the pool the ask belongs to, from zero.
   */
  readonly pass: number;
};

/**
 The asks of an arm that walks the pool pass by pass: every trial in the pool's order, then every trial again, and so on, so
 that the leading asks of an arm the spend guard capped are the first pass whole and then the start of the second.

 @param trials - pool's trials in the one seeded order every arm walks

 @param passes - how many times the pool is walked

 @returns Asks, first pass's first

 @example
 ```ts
 const asks = passAsksOf({ trials, passes: 2, },);
 ```
 */
export function passAsksOf(
  {
    trials,
    passes,
  }: {
    readonly trials: readonly EditorFidelityTrial[];
    readonly passes: number;
  },
): readonly PassAsk[] {
  return Array.from(
    { length: passes, },
    function passAt(
      _unused,
      pass,
    ): readonly PassAsk[] {
      return trials.map(function askOf(trial,): PassAsk {
        return {
          trial,
          pass,
        };
      },);
    },
  )
    .flat();
}

/**
 Each card's asks together, the first card's before the second's.

 @param cardIds - cards seated, in the order named

 @param planned - asks every card walks

 @returns Pairs, in the order they are asked; a pair of a later pass names the pass

 @example
 ```ts
 const pairs = pairsOf({ cardIds, planned, },);
 ```
 */
export function pairsOf(
  {
    cardIds,
    planned,
  }: {
    readonly cardIds: readonly RosterModelId[];
    readonly planned: readonly PassAsk[];
  },
): readonly EditorPair[] {
  return cardIds.flatMap(function cardPairs(cardId,): readonly EditorPair[] {
    return planned.map(function pairOf(
      {
        trial,
        pass,
      },
    ): EditorPair {
      return {
        cardId,
        trial,
        // The member stays absent for the first pass, so a pair of one pass is what it always was.
        ...((pass === 0) ? {} : { pass, }),
      };
    },);
  },);
}

/**
 The leading asks of the passes over the pool an arm walks, warning in the judge probe's words when they leave asks out.

 @param trials - pool's trials in the one seeded order every arm walks

 @param passes - how many times the pool is walked

 @param walked - how many leading asks the arm walks

 @param log - logger the warning goes to

 @returns Leading asks, all of them when the pool and its passes hold no more than `walked`

 @example
 ```ts
 const planned = plannedAsksOf({ trials, passes: 2, walked: 120, log, },);
 ```
 */
export function plannedAsksOf(
  {
    trials,
    passes,
    walked,
    log,
  }: {
    readonly trials: readonly EditorFidelityTrial[];
    readonly passes: number;
    readonly walked: number;
    readonly log: Logger;
  },
): readonly PassAsk[] {
  /**
   Every ask of the passes, the first pass whole before the second begins.
   */
  const everyAsk = passAsksOf({
    trials,
    passes,
  },);
  /**
   The leading asks.
   */
  const planned = everyAsk.slice(
    0,
    walked,
  );
  if (planned.length < everyAsk.length) {
    log.warn(fidelityPartialWarning({
      planned: planned.length,
      total: everyAsk.length,
    },),);
  }
  return planned;
}

/**
 Writes a JSON document to a file that must not exist yet, readable by its owner alone.

 @param path - file

 @param value - what the file holds

 @throws {@link Error} when the file exists or cannot be written

 @example
 ```ts
 await writeJsonOnce({ path: join(runDir, 'plan.json',), value: plan, },);
 ```
 */
export async function writeJsonOnce({
  path,
  value,
}: {
  readonly path: string;
  readonly value: unknown
},): Promise<void> {
  await writeFile(
    path,
    JSON.stringify(
      value,
      undefined,
      2,
    ),
    {
      mode: 0o600,
      flag: 'wx',
    },
  );
}

/**
 Walks the pairs one at a time and keeps every row that finished, so an arm the spend guard caps returns the rows it has.

 @param pairs - planned pairs, in the order walked

 @param controller - arm's own, which the guard aborts with its refusal as the reason

 @param log - logger the walk writes its progress to

 @param oneRow - asks one pair and returns its row

 @returns Rows that finished, in order; when the guard capped the arm, a CAPPED line has been logged

 @throws {@link Error} whatever an ask threw that is not the guard's refusal

 @example
 ```ts
 const rows = await walkPairs({ pairs, controller, log, oneRow, },);
 ```
 */
export async function walkPairs(
  {
    pairs,
    controller,
    log,
    oneRow,
  }: {
    readonly pairs: readonly EditorPair[];
    readonly controller: AbortController;
    readonly log: Logger;
    readonly oneRow: (
      input: {
        readonly pair: EditorPair;
        readonly position: number
      },
    ) => Promise<EditorFidelityRow>;
  },
): Promise<readonly EditorFidelityRow[]> {
  /**
   Rows in the order they finished, kept here because the overlapped map discards what it holds when an item throws.
   */
  const finished: EditorFidelityRow[] = [];
  try {
    await mapOverlapped({
      items: pairs,
      overlap: 1,
      // The spend guard aborts this signal when it refuses a call, and nothing else does;
      // one row runs at a time, so no failure is left unthrown to be written.
      signal: controller.signal,
      l: log,
      nameOf: pairName,
      oneItem: async function runPair({
        item: pair,
        position,
      },): Promise<EditorFidelityRow> {
        /**
         The pair's row.
         */
        const row = await oneRow({
          pair,
          position,
        },);
        finished.push(row,);
        return row;
      },
    },);
  }
  catch (error) {
    /**
     Why the arm's signal aborted, if it did.
     */
    const reason: unknown = controller.signal
      .reason;
    if (!(controller.signal
      .aborted
      && (reason instanceof StatedRefusalError)))
      throw error;
    log.debug(`the walk stopped on the spend guard's refusal: ${refusalText({ error, },)}`,);
    // THE GUARD'S REFUSAL ENDS THE ARM AS CAPPED, NOT AS FAILED: the rows it finished are a result, and the read step takes
    // the trials the later arms walk from how many were completed.
    log.warn(cappedLine({
      completed: finished.length,
      planned: pairs.length,
      said: refusalText({ error: reason, },),
    },),);
  }
  return finished;
}

//endregion Editor fidelity walk
