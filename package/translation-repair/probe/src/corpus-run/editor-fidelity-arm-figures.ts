import {
  type ArmLog,
  bodyOf,
  firstStarting,
  isRowBody,
  numberAfter,
  reusedRowFlags,
  wordAfter,
} from './editor-fidelity-arm-facts.ts';

//region Editor fidelity arm figures
// THE FIGURES THE READ STEP PRINTS AND DECIDES BY, read off an arm's log by the markers the probe's writers use: the
// EDITOR lines (pooled and for each kind of defect), the SEAT line (what threw and what was abandoned) and the row lines
// (how the asks of each trial ended). A figure that is not in the log is minus one, never zero, so a log that did not
// state it cannot read as a clean arm.

/**
 The counts of one kind of defect, or of the pool.

 @example
 ```ts
 const counts: KindFigure = { trials: 22, fixed: 18, defectOnly: 2, };
 ```
 */
export type KindFigure = {
  /**
   Trials counted, minus one when the line is absent.
   */
  readonly trials: number;

  /**
   Trials fixed cleanly, minus one when the line is absent.
   */
  readonly fixed: number;

  /**
   Replies that removed the defect and changed something else, minus one when the line is absent.
   */
  readonly defectOnly: number;
};

/**
 How the asks of one row ended, as its line states.

 @example
 ```ts
 const row: RowStops = { row: 2, trial: 'tabby-s1/insertion', stops: ['refused', 'ok',], stated: true, };
 ```
 */
export type RowStops = {
  /**
   The row's number in the log.
   */
  readonly row: number;

  /**
   The trial's handle.
   */
  readonly trial: string;

  /**
   The entry the trial comes from, empty when the line names none.
   */
  readonly entry: string;

  /**
   The pass over the pool the row belongs to, counted from one; one when the line names none.
   */
  readonly pass: number;

  /**
   The kind of defect the handle ends in, empty when it ends in neither.
   */
  readonly kind: '' | 'insertion' | 'alteration';

  /**
   Whether the row's reading is a clean fix, a reading that starts with `clean fix`.
   */
  readonly fixed: boolean;

  /**
   The ends of the asks, in the order asked; empty when the line states none.
   */
  readonly stops: readonly string[];

  /**
   Whether the line states a stop reason at all.
   */
  readonly stated: boolean;
};

/**
 What an arm's log says of how the arm did.

 @example
 ```ts
 const figures = armFiguresOf({ log, },);
 ```
 */
export type ArmFigures = KindFigure & {
  /**
   The card, empty when the log names none.
   */
  readonly card: string;

  /**
   Trials with no usable reply, minus one when the line is absent.
   */
  readonly unusable: number;

  /**
   Trials where the card was heard and proposed no edit, minus one when the line is absent.
   */
  readonly refused: number;

  /**
   The counts of the insertion trials.
   */
  readonly insertion: KindFigure;

  /**
   The counts of the alteration trials.
   */
  readonly alteration: KindFigure;

  /**
   Asks the seat line says threw, minus one when it says nothing.
   */
  readonly threw: number;

  /**
   Attempts the seat line says were abandoned, minus one when it says nothing.
   */
  readonly abandoned: number;

  /**
   Each row line of a row that made a call, in the order written. A row answered by the prompt reuse layer is no trial, so
   it is left out of every count and comparison.
   */
  readonly rows: readonly RowStops[];

  /**
   Row lines left out of `rows` because the reply was a reuse and no call was made.
   */
  readonly reusedRows: number;
};

/**
 Reads the counts of one EDITOR line.

 @param line - line's body, empty when there is none

 @returns Counts, minus one for what the line does not state

 @example
 ```ts
 const counts = kindFigureOf({ line: 'EDITOR claude-cat: clean fix 30 of 40 (exact 30, equivalent 0), defect only 3', },);
 ```
 */
function kindFigureOf({ line, }: { readonly line: string; },): KindFigure {
  return {
    trials: numberAfter({
      text: line,
      marker: ' of ',
    },),
    fixed: numberAfter({
      text: line,
      marker: 'clean fix ',
    },),
    defectOnly: numberAfter({
      text: line,
      marker: 'defect only ',
    },),
  };
}

/**
 Reads the pass one row line names.

 @param body - row line's body

 @returns Pass counted from one, one when the line names none, which is how a row of the first pass reads

 @example
 ```ts
 // yields 2
 const pass = passOf({ body: 'row 90 of 176: card on tabby/insertion: clean fix exact; entry tabby; pass 2; stopped ok', },);
 ```
 */
function passOf({ body, }: { readonly body: string; },): number {
  /**
   The number after the pass marker, minus one when the line names none.
   */
  const named = numberAfter({
    text: body,
    marker: '; pass ',
  },);
  return (named === (-1)) ? 1 : named;
}

/**
 Reads the stops one row line states.

 @param body - row line's body

 @returns Row, its trial, its entry, whether it is a clean fix and its stops

 @example
 ```ts
 const row = rowStopsOf({ body: 'row 2 of 40: card on tabby/insertion: clean fix exact, nothing else touched; entry tabby; stopped ok', },);
 ```
 */
function rowStopsOf({ body, }: { readonly body: string; },): RowStops {
  /**
   Where the stop words begin.
   */
  const marker = '; stopped ';
  /**
   Where the entry begins.
   */
  const entryMarker = '; entry ';
  /**
   What stands before the trial's handle.
   */
  const trialMarker = ' on ';
  /**
   The text after the trial's handle begins.
   */
  const afterOn = body.includes(trialMarker,) ? body.slice(body.indexOf(trialMarker,) + trialMarker.length,) : '';
  /**
   The trial's handle, then the reading and what follows it.
   */
  const [trial = '', ...afterTrial] = afterOn.split(': ',);
  /**
   Where the marker stands, or minus one when the line states no stop words.
   */
  const markerAt = body.lastIndexOf(marker,);
  /**
   Where the entry marker stands, or minus one when the line names no entry.
   */
  const entryAt = body.lastIndexOf(entryMarker,);
  /**
   The text after the entry marker, up to the next semicolon.
   */
  const entry = (entryAt === (-1))
    ? ''
    : (body.slice(entryAt + entryMarker.length,)
      .split(';',)[0]
      ?? '');
  /**
   The kind the handle ends in.
   */
  const kindWord = trial.slice(trial.lastIndexOf('/',) + 1,);
  return {
    row: numberAfter({
      text: body,
      marker: 'row ',
    },),
    trial,
    entry,
    pass: passOf({ body, },),
    kind: (kindWord === 'insertion') || (kindWord === 'alteration') ? kindWord : '',
    fixed: afterTrial.join(': ',)
      .startsWith('clean fix',),
    stops: (markerAt === (-1)) ? [] : body.slice(markerAt + marker.length,)
      .split('+',),
    stated: markerAt !== (-1),
  };
}

/**
 Reads the figures of an arm's log.

 @param log - arm

 @returns Figures

 @example
 ```ts
 const figures = armFiguresOf({ log, },);
 ```
 */
export function armFiguresOf({ log, }: { readonly log: ArmLog; },): ArmFigures {
  /**
   The log's bodies.
   */
  const bodies = log.lines
    .map(function body(line,): string {
      return bodyOf({ line, },);
    },);
  /**
   The card the arm seated.
   */
  const card = wordAfter({
    text: firstStarting({
      bodies,
      marker: 'editors: ',
    },),
    marker: 'editors: ',
  },);
  /**
   The pooled line.
   */
  const pooled = firstStarting({
    bodies,
    marker: `EDITOR ${card}: clean fix `,
  },);
  /**
   The seat line.
   */
  const seat = firstStarting({
    bodies,
    marker: `SEAT ${card} `,
  },);
  /**
   The bodies of the row lines, in the order written.
   */
  const rowBodies = bodies.filter(function isRow(body,): boolean {
    return isRowBody({ body, },);
  },);
  /**
   Which of them made no call.
   */
  const reused = reusedRowFlags({ bodies, },);
  return {
    ...kindFigureOf({ line: pooled, },),
    card,
    unusable: numberAfter({
      text: pooled,
      marker: 'unusable ',
    },),
    refused: numberAfter({
      text: pooled,
      marker: 'refused ',
    },),
    insertion: kindFigureOf({ line: firstStarting({
      bodies,
      marker: `EDITOR ${card} insertion: clean fix `,
    },), },),
    alteration: kindFigureOf({ line: firstStarting({
      bodies,
      marker: `EDITOR ${card} alteration: clean fix `,
    },), },),
    threw: numberAfter({
      text: seat,
      marker: 'threw=',
    },),
    abandoned: numberAfter({
      text: seat,
      marker: 'abandoned=',
    },),
    rows: rowBodies
      .filter(function madeACall(
        _body,
        at,
      ): boolean {
        return reused[at] !== true;
      },)
      .map(function stopsOf(body,): RowStops {
        return rowStopsOf({ body, },);
      },),
    reusedRows: reused.filter(function isReused(flag,): boolean {
      return flag;
    },)
      .length,
  };
}

//endregion Editor fidelity arm figures
