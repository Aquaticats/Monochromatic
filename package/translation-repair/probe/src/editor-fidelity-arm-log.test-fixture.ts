import {
  type ArmLog,
  cappedLine,
  type EditorCardTally,
  editorDefectOnlyLine,
  editorKindLines,
  type EditorKindTally,
  editorTallyLine,
  guardTotalLine,
  plannedLine,
  trialsLine,
} from '../dist/final/node/index.mjs';
import {
  openingOf,
  POOL_TRIALS,
  type RowSpec,
  rowLineFor,
} from './editor-fidelity-arm-log-lines.test-fixture.ts';

export {
  poolStubs,
  POOL_TRIALS,
  type RowSpec,
  rowLineFor,
  stubTrial,
} from './editor-fidelity-arm-log-lines.test-fixture.ts';

//region Invented editor fidelity arm logs
// Arm logs for the cases of the read step, written by the probe's own line writers so a writer that changes its wording
// fails here before it fails an operator. Cat-themed invention; no corpus content.

/**
 Prefix the logger puts on every line.
 */
const LOG_PREFIX ='[info] [2026-10-23T12:00:00.000Z] [editor-fidelity-probe] ';

/**
 The digest the invented PROMPT-REUSE lines name: sixty-four hexadecimal characters, as a prompt digest is.
 */
const REUSED_DIGEST = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';

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
 The rows put into a later pass over the pool, each with the pass its line names.

 @param rows - rows of a pass

 @param pass - pass, counted from one

 @returns Same rows, each in that pass

 @example
 ```ts
 const second = inPass({ rows: poolRows({ count: 44, fixed, },), pass: 2, },);
 ```
 */
export function inPass(
  {
    rows,
    pass,
  }: {
    readonly rows: readonly RowSpec[];
    readonly pass: number;
  },
): readonly RowSpec[] {
  return rows.map(function passed(row,): RowSpec {
    return {
      ...row,
      pass,
    };
  },);
}

/**
 The rows of an arm that walks the leading trials of the invented pool: two to an entry, the insertion then the
 alteration, entries named `cat-0`, `cat-1` and so on.

 @param count - how many leading trials

 @param fixed - whether the trial at an index reads as a clean fix

 @param stops - stop words of the trial at an index, `ok` when omitted

 @returns Rows

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

 @param counts - kind's counts

 @returns Tally, every clean fix counted as exact

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

 @param cardId - card

 @param insertion - counts of the insertion trials

 @param alteration - counts of the alteration trials

 @param unusable - trials with no usable reply

 @param refused - trials where the card was heard and proposed no edit

 @returns Tally

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

 @param rows - rows

 @param kind - kind of defect

 @returns Trials of the kind and how many read as a clean fix

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

 @param cardId - card

 @param rows - rows

 @returns Tally, clean fixes counted for each kind and nothing refused, unusable or defect-only

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

 @param arm - arm's name

 @param tally - card's tally

 @param stops - for each row, the stop words its line states, empty for a row that states none; ignored when `rows` is
 given

 @param rows - rows the arm walked, for a log whose trials matter

 @param planned - trials the planned line says, the number of rows when omitted

 @param passes - times the arm was asked to walk the pool, the latest pass a row names when omitted

 @param capped - whether the arm was capped by the spend guard after its rows

 @param seat - SEAT line

 @param benchSpentUsd - spend of the earlier arms the guard line states

 @param guardUsd - total the guard line states

 @returns Arm, with a recorded exit of zero

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
    passes: passesAsked,
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
    readonly passes?: number;
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
  /**
   Times the arm walks the pool: the number asked, else the latest pass a row names, one when none names a later pass.
   */
  const passes = passesAsked ?? Math.max(
    1,
    ...walked.map(function passOf({ pass = 1, },): number {
      return pass;
    },),
  );
  return {
    arm,
    exit: {
      recorded: true,
      code: 0,
    },
    lines: [
      `editors: ${tally.cardId}`,
      ...openingOf({
        cardId: tally.cardId,
        plannedAsks: plannedTrials,
        maxUsd: 3,
        benchSpentUsd,
      },),
      plannedLine({
        planned: plannedTrials,
        total: POOL_TRIALS * passes,
        passes,
      },),
      // The SPEND line is written by hand: its writer also moves the run's meter, which a case must not touch.
      `[info] [2026-10-23T12:00:01.000Z] [translation-repair] [reportSpend] SPEND provider=anthropic model=${tally.cardId} prompt=9000 completion=1500 cost=${String(guardUsd,)}`,
      ...walked.flatMap(function rowLines(
        row,
        at,
      ): readonly string[] {
        return [
          // The line is written by hand, as the SPEND line is: its writer is the stage's logger, which a case must not call.
          ...((row.reused === true)
            ? [`PROMPT-REUSE source=memory model=${tally.cardId} digest=${REUSED_DIGEST}`,]
            : []),
          rowLineFor({
            cardId: tally.cardId,
            row,
            position: at,
            total: plannedTrials,
          },),
        ];
      },),
      ...(capped
        ? [
          cappedLine({
            completed: walked.length,
            planned: plannedTrials,
            said: 'the guard refused',
          },),
        ]
        : []),
      editorTallyLine({ tally, },),
      ...editorKindLines({ tally, },),
      editorDefectOnlyLine({ tally, },),
      trialsLine({
        completed: walked.length,
        planned: plannedTrials,
      },),
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
