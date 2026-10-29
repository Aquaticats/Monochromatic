import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Nested single line readings
// THE READINGS OF ONE LINE of straight single marks (`nested-single-quotes.ts`). A quoted two-digit number inside a quotation
// (`'the '90s'`) can be read two ways, as an elision whose closing mark closes the quotation around it or as a quotation of its
// own, and which reading is right depends on the marks that follow on the line. A reading is therefore a value: the opening marks
// that wait for a partner, the innermost last, and the pairs found. The scan keeps every reading that can still win to the end of
// the line, one for each number of waiting marks (of readings that wait with the same number of marks the future is the same, so
// the one with the most pairs is kept), and the reading with the most pairs wins, a tie keeping the elision.

/**
 A pair of straight single marks found on a line.
 */
export type PairedMarks = {
  /**
   Offset of the opening mark.
   */
  readonly open: number;

  /**
   Offset of the closing mark.
   */
  readonly close: number;
};

/**
 One reading of a line up to the mark under the scan: the marks that wait, and the pairs found.
 */
export type LineState = {
  /**
   Offsets of the opening marks that wait for their partners, the innermost last.
   */
  readonly waiting: readonly number[];

  /**
   The pairs found; each pair leaves two marks fewer unpaired.
   */
  readonly paired: readonly PairedMarks[];
};

/**
 How a straight single mark reads by its neighbours.
 */
export type MarkShape = 'opening' | 'closing' | 'neither';

/**
 The reading of a line before any mark.
 */
export const EMPTY_LINE: LineState = {
  waiting: [],
  paired: [],
};

/**
 How many pairs a line reading found.

 @param state - reading

 @returns Number of pairs

 @example
 ```ts
 const count = pairCountOf({ state: EMPTY_LINE, },); // 0
 ```
 */
function pairCountOf({ state, }: { readonly state: LineState; },): number {
  /**
   The pairs the reading found.
   */
  const { paired, } = state;
  return paired.length;
}

/**
 A line reading after the closing mark at an offset pairs with the innermost opening mark that waits.

 @param state - reading, whose innermost opening mark waits

 @param close - offset of the closing mark

 @returns Reading with the pair found and opening mark no longer waiting

 @throws The error of `nonNullishOrThrow` when no opening mark waits

 @example
 ```ts
 const next = closedBy({ state: { waiting: [5,], paired: [], }, close: 10, },); // pairs 5 with 10
 ```
 */
function closedBy(
  {
    state,
    close,
  }: {
    readonly state: LineState;
    readonly close: number;
  },
): LineState {
  /**
   The marks that wait and the pairs found.
   */
  const {
    waiting,
    paired,
  } = state;
  /**
   The innermost opening mark that waits.
   */
  const open = nonNullishOrThrow(waiting.at(-1,),);
  return {
    waiting: waiting.slice(
      0,
      -1,
    ),
    paired: [
      ...paired,
      {
        open,
        close,
      },
    ],
  };
}

/**
 A line reading after the straight single mark at an offset, read by its shape.

 @param state - reading before the mark

 @param at - offset of the mark

 @param shape - how the mark reads by its neighbours

 @param elision - whether the mark starts a listed elision (`'em`, `'tis`)

 @returns Reading after the mark: an opening mark waits (a listed elision never joins the opening marks that wait, since
 inside a quotation it is an apostrophe), and a closing mark pairs with the innermost opening mark that waits

 @example
 ```ts
 const next = withMark({ state: EMPTY_LINE, at: 5, shape: 'opening', elision: false, },);
 ```
 */
export function withMark(
  {
    state,
    at,
    shape,
    elision,
  }: {
    readonly state: LineState;
    readonly at: number;
    readonly shape: MarkShape;
    readonly elision: boolean;
  },
): LineState {
  /**
   The marks that wait.
   */
  const { waiting, } = state;
  if ((shape === 'opening') && ((waiting.length === 0) || (!elision))) {
    return {
      ...state,
      waiting: [
        ...waiting,
        at,
      ],
    };
  }
  if ((shape === 'closing') && (waiting.length > 0)) {
    return closedBy({
      state,
      close: at,
    },);
  }
  return state;
}

/**
 The line readings with one for each number of waiting marks: of the readings that wait with the same number of marks the future
 is the same, so the one with the most pairs is kept, and of equal ones the first, which keeps an elision.

 @param states - readings, reading that keeps an elision first

 @returns Kept readings in the order of their first appearance

 @example
 ```ts
 const kept = distinctDepths({ states: [EMPTY_LINE, EMPTY_LINE,], },); // one reading
 ```
 */
export function distinctDepths({ states, }: { readonly states: readonly LineState[]; },): readonly LineState[] {
  /**
   The best reading of each number of waiting marks, mutated only inside this function.
   */
  const best = new Map<number, LineState>();
  for (const state of states) {
    /**
     How many marks wait in this reading.
     */
    const { length: depth, } = state.waiting;
    /**
     The reading kept for this number of waiting marks so far.
     */
    const known = best.get(depth,);
    if ((known === undefined) || (pairCountOf({ state, },) > pairCountOf({ state: known, },))) {
      best.set(
        depth,
        state,
      );
    }
  }
  return Array.from(best.values(),);
}

/**
 The line readings after a quoted two-digit number (`'42'`, `'90s'`) and the mark that closes it.

 @param states - readings before the number, reading that keeps an elision first

 @param open - offset of the number's opening mark

 @param close - offset of the mark that closes it

 @param closeShape - how the closing mark reads by its neighbours

 @returns For each reading with a mark waiting, two: the mark an elision, so that the closing mark closes the quotation around
 it, and the number a quotation of its own; for a reading with none waiting the number, which the mark closes in either reading

 @example
 ```ts
 const next = withQuotedNumber({ states: [EMPTY_LINE,], open: 5, close: 8, closeShape: 'closing', },);
 ```
 */
export function withQuotedNumber(
  {
    states,
    open,
    close,
    closeShape,
  }: {
    readonly states: readonly LineState[];
    readonly open: number;
    readonly close: number;
    readonly closeShape: MarkShape;
  },
): readonly LineState[] {
  return distinctDepths({
    states: states.flatMap(function readings(state,): readonly LineState[] {
      /**
       The marks that wait and the pairs found.
       */
      const {
        waiting,
        paired,
      } = state;
      /**
       The reading where the number is a quotation of its own.
       */
      const quotation: LineState = {
        ...state,
        paired: [
          ...paired,
          {
            open,
            close,
          },
        ],
      };
      if (waiting.length === 0)
        return [quotation,];
      return [
        (closeShape === 'closing')
          ? closedBy({
            state,
            close,
          },)
          : state,
        quotation,
      ];
    },),
  },);
}

/**
 The pairs of the best reading of a line: the reading with the most pairs, and of equal ones the first, which keeps an elision.

 @param states - readings of the line, at least one

 @returns Pairs of the winning reading

 @example
 ```ts
 const pairs = bestPairsOf({ states: [EMPTY_LINE,], },); // none
 ```
 */
export function bestPairsOf({ states, }: { readonly states: readonly LineState[]; },): readonly PairedMarks[] {
  /**
   The reading with the most pairs, the first of equal ones.
   */
  const best = states.reduce(function fewer(
    kept,
    state,
  ): LineState {
    return (pairCountOf({ state, },) > pairCountOf({ state: kept, },)) ? state : kept;
  },);
  return best.paired;
}

//endregion Nested single line readings
