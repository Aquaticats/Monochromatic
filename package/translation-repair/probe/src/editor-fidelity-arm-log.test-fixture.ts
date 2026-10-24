import type { ArmLog, } from './corpus-run/editor-fidelity-arm-facts.ts';
import {
  type EditorCardTally,
  editorDefectOnlyLine,
  editorKindLines,
  type EditorKindTally,
  editorTallyLine,
} from './corpus-run/editor-fidelity-plan.ts';
import {
  guardLine,
  guardTotalLine,
} from './corpus-run/editor-fidelity-spend-guard.ts';

//region Invented editor fidelity arm logs
// Arm logs for the cases of the read step, written by the probe's own line writers so a writer that changes its wording
// fails here before it fails an operator. Cat-themed invention; no corpus content.

/**
 Prefix the logger puts on every line.
 */
export const LOG_PREFIX = '[info] [2026-10-23T12:00:00.000Z] [editor-fidelity-probe] ';

/**
 How many trials of one kind a card fixed cleanly, and how many replies removed the defect and changed more.
 */
export type KindFixed = {
  /**
   Trials of the kind.
   */
  readonly trials: number;

  /**
   Trials fixed cleanly.
   */
  readonly fixed: number;

  /**
   Trials where the defect was removed and something else changed.
   */
  readonly defectOnly: number;
};

/**
 The counts of one kind written as a kind tally.

 @param counts - the kind's counts

 @returns The tally, every clean fix counted as exact

 @example
 ```ts
 const tally = kindTallyOf({ counts: { trials: 22, fixed: 18, defectOnly: 2, }, },);
 ```
 */
function kindTallyOf({ counts, }: { readonly counts: KindFixed; }): EditorKindTally {
  return {
    trials: counts.trials,
    exact: counts.fixed,
    equivalent: 0,
    defectOnly: counts.defectOnly,
    collateral: 0,
    missed: counts.trials - counts.fixed - counts.defectOnly,
    refused: 0,
    unusable: 0,
  };
}

/**
 A card's tally from the counts of its two kinds.

 @param cardId - the card
 @param insertion - counts of the insertion trials
 @param alteration - counts of the alteration trials
 @param unusable - trials with no usable reply
 @param refused - trials where the card was heard and proposed no edit

 @returns The tally

 @example
 ```ts
 const tally = tallyOf({ cardId, insertion, alteration, unusable: 1, refused: 2, },);
 ```
 */
export function tallyOf(
  {
    cardId,
    insertion,
    alteration,
    unusable,
    refused,
  }: {
    readonly cardId: EditorCardTally['cardId'];
    readonly insertion: KindFixed;
    readonly alteration: KindFixed;
    readonly unusable: number;
    readonly refused: number;
  },
): EditorCardTally {
  return {
    cardId,
    trials: insertion.trials + alteration.trials,
    exact: insertion.fixed + alteration.fixed,
    equivalent: 0,
    defectOnly: insertion.defectOnly + alteration.defectOnly,
    collateral: 0,
    missed: 0,
    refused,
    unusable,
    fixed: insertion.fixed + alteration.fixed,
    byKind: {
      insertion: kindTallyOf({ counts: insertion, },),
      alteration: kindTallyOf({ counts: alteration, },),
    },
  };
}

/**
 The log of one arm: its opening, a row line for each stop given, the tallies, the seat and the guard's total.

 @param arm - the arm's name
 @param tally - the card's tally
 @param stops - for each row, the stop words its line states, absent for a row that states none
 @param seat - the SEAT line
 @param benchSpentUsd - the spend of the earlier arms the guard line states
 @param guardUsd - the total the guard line states

 @returns The arm, with a recorded exit of zero

 @example
 ```ts
 const log = armLogOf({ arm: 'low', tally, stops: ['ok', 'refused+ok',], seat, benchSpentUsd: 0, guardUsd: 0.5, },);
 ```
 */
export function armLogOf(
  {
    arm,
    tally,
    stops,
    seat,
    benchSpentUsd,
    guardUsd,
  }: {
    readonly arm: string;
    readonly tally: EditorCardTally;
    readonly stops: readonly (string | undefined)[];
    readonly seat: string;
    readonly benchSpentUsd: number;
    readonly guardUsd: number;
  },
): ArmLog {
  return {
    arm,
    exit: {
      recorded: true,
      code: 0,
    },
    lines: [
      `editors: ${tally.cardId}`,
      guardLine({ maxUsd: 3, benchSpentUsd, },),
      ...stops.map(function rowLine(
        stopped,
        at,
      ): string {
        return `row ${String(at + 1,)} of ${String(stops.length,)}: ${tally.cardId} on tabby-s${String(at,)}/insertion: `
          + `clean fix exact, nothing else touched${(stopped === undefined) ? '' : `; stopped ${stopped}`}`;
      },),
      editorTallyLine({ tally, },),
      ...editorKindLines({ tally, },),
      editorDefectOnlyLine({ tally, },),
      guardTotalLine({ total: { usd: guardUsd, calls: stops.length, atCeiling: 0, }, },),
      seat,
    ].map(function prefixed(line,): string {
      return `${LOG_PREFIX}${line}`;
    },),
  };
}

//endregion Invented editor fidelity arm logs
