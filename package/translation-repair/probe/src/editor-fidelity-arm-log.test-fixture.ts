import {
  type ArmLog,
  type EditorCardTally,
  editorDefectOnlyLine,
  editorKindLines,
  type EditorKindTally,
  editorTallyLine,
  guardLine,
  guardTotalLine,
} from '../dist/final/node/index.mjs';

//region Invented editor fidelity arm logs
// Arm logs for the cases of the read step, written by the probe's own line writers so a writer that changes its wording
// fails here before it fails an operator. Cat-themed invention; no corpus content.

/**
 Prefix the logger puts on every line.
 */
const LOG_PREFIX ='[info] [2026-10-23T12:00:00.000Z] [editor-fidelity-probe] ';

/**
 Trials the invented pool holds, which a log's planned line says.
 */
const POOL_TRIALS = 88;

/**
 How many trials of one kind a card fixed cleanly, and how many replies removed the defect and changed more.
 */
type KindFixed = {
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
 One row of an invented arm: the trial it walked, the entry the trial comes from, whether it was a clean fix and how its
 asks ended.
 */
export type RowSpec = {
  /**
   The trial's handle, `<entry>-s0/<kind>`.
   */
  readonly trial: string;

  /**
   The entry the trial comes from.
   */
  readonly entry: string;

  /**
   Whether the row reads as a clean fix.
   */
  readonly fixed: boolean;

  /**
   The stop words the row states, empty for a row that states none.
   */
  readonly stops: string;
};

/**
 The rows of an arm that walks the leading trials of the invented pool: two to an entry, the insertion then the
 alteration, entries named `cat-0`, `cat-1` and so on.

 @param count - how many leading trials

 @param fixed - whether the trial at an index reads as a clean fix

 @param stops - the stop words of the trial at an index, `ok` when omitted

 @returns The rows

 @example
 ```ts
 const rows = poolRows({ count: 40, fixed: function every(index,) { return index % 2 === 0; }, },);
 ```
 */
export function poolRows(
  {
    count,
    fixed,
    stops,
  }: {
    readonly count: number;
    readonly fixed: (index: number,) => boolean;
    readonly stops?: (index: number,) => string;
  },
): readonly RowSpec[] {
  return Array.from(
    { length: count, },
    function rowAt(
      _unused,
      index,
    ): RowSpec {
      /**
       The entry of this trial, two trials to an entry.
       */
      const entry = `cat-${String(Math.floor(index / 2,),)}`;
      return {
        trial: `${entry}-s0/${((index % 2) === 0) ? 'insertion' : 'alteration'}`,
        entry,
        fixed: fixed(index,),
        stops: (stops === undefined) ? 'ok' : stops(index,),
      };
    },
  );
}

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
    missed: counts.trials - counts.fixed
      - counts.defectOnly,
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
 The counts of the rows of one kind.

 @param rows - the rows

 @param kind - the kind of defect

 @returns The trials of the kind and how many read as a clean fix

 @example
 ```ts
 const counts = countsOfKind({ rows, kind: 'insertion', },);
 ```
 */
function countsOfKind({
  rows,
  kind,
}: {
  readonly rows: readonly RowSpec[];
  readonly kind: string
},): KindFixed {
  /**
   The rows of the kind.
   */
  const ofKind = rows.filter(function isKind({ trial, },): boolean {
    return trial.endsWith(`/${kind}`,);
  },);
  return {
    trials: ofKind.length,
    fixed: ofKind.filter(function isFixed({ fixed, },): boolean {
      return fixed;
    },)
      .length,
    defectOnly: 0,
  };
}

/**
 A card's tally from the rows it walked.

 @param cardId - the card

 @param rows - the rows

 @returns The tally, the clean fixes counted for each kind and nothing refused, unusable or defect-only

 @example
 ```ts
 const tally = tallyFromRows({ cardId, rows, },);
 ```
 */
export function tallyFromRows(
  {
    cardId,
    rows,
  }: {
    readonly cardId: EditorCardTally['cardId'];
    readonly rows: readonly RowSpec[];
  },
): EditorCardTally {
  return tallyOf({
    cardId,
    insertion: countsOfKind({
      rows,
      kind: 'insertion',
    },),
    alteration: countsOfKind({
      rows,
      kind: 'alteration',
    },),
    unusable: 0,
    refused: 0,
  },);
}

/**
 The log of one arm: its opening, a row line for each row, the tallies, the trials line, the seat and the guard's total.

 @param arm - the arm's name

 @param tally - the card's tally

 @param stops - for each row, the stop words its line states, empty for a row that states none; ignored when `rows` is
 given

 @param rows - the rows the arm walked, for a log whose trials matter

 @param planned - trials the planned line says, the number of rows when omitted

 @param capped - whether the arm was capped by the spend guard after its rows

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
    rows,
    planned,
    capped = false,
    seat,
    benchSpentUsd,
    guardUsd,
  }: {
    readonly arm: string;
    readonly tally: EditorCardTally;
    readonly stops: readonly string[];
    readonly rows?: readonly RowSpec[];
    readonly planned?: number;
    readonly capped?: boolean;
    readonly seat: string;
    readonly benchSpentUsd: number;
    readonly guardUsd: number;
  },
): ArmLog {
  /**
   The rows the log states: the given ones, else one clean fix for each stop word.
   */
  const walked: readonly RowSpec[] = rows ?? stops.map(function rowOf(
    stopped,
    at,
  ): RowSpec {
    return {
      trial: `tabby-s${String(at,)}/insertion`,
      entry: `cat-${String(at,)}`,
      fixed: true,
      stops: stopped,
    };
  },);
  /**
   Trials the planned line says.
   */
  const plannedTrials = planned ?? walked.length;
  return {
    arm,
    exit: {
      recorded: true,
      code: 0,
    },
    lines: [
      `editors: ${tally.cardId}`,
      guardLine({
        maxUsd: 3,
        benchSpentUsd,
      },),
      `planned: ${String(plannedTrials,)} of ${String(POOL_TRIALS,)} trials`,
      ...walked.map(function rowLine(
        row,
        at,
      ): string {
        return `row ${String(at + 1,)} of ${String(plannedTrials,)}: ${tally.cardId} on ${row.trial}: ${
          row.fixed ? 'clean fix exact, nothing else touched' : 'not fixed, nothing else touched'
        }; entry ${row.entry}${(row.stops === '') ? '' : `; stopped ${row.stops}`}`;
      },),
      ...(capped
        ? [
          `CAPPED at trial ${String(walked.length + 1,)} of ${String(plannedTrials,)}: ${
            String(walked.length,)
          } trials completed; the guard refused`,
        ]
        : []),
      editorTallyLine({ tally, },),
      ...editorKindLines({ tally, },),
      editorDefectOnlyLine({ tally, },),
      `TRIALS completed ${String(walked.length,)} of ${String(plannedTrials,)}`,
      guardTotalLine({ total: {
        usd: guardUsd,
        calls: walked.length,
        atCeiling: 0,
      }, },),
      seat,
    ].map(function prefixed(line,): string {
      return `${LOG_PREFIX}${line}`;
    },),
  };
}

//endregion Invented editor fidelity arm logs
