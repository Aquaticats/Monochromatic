/**
 The reader of the clauses a judge must be able to read in the sentence that tells it what a decline does over wording
 that cannot ship, written as the meaning each carries rather than as a copy of the constant's words (re-check 3, N1 and
 N7), for each of the three rounds that sentence is told in (re-check 4, F1). The clauses are in
 `decline-clause.test-fixture.ts` and `decline-clause-rounds.test-fixture.ts`.

 A copied substring fails on any rewording, including one that keeps every meaning. Each clause is a list of slots, and
 a slot is the ways one idea can be put. A clause is read when one part of the text, the text cut at every `;`, `:` and
 full stop, carries one way of each slot in slot order and carries none of the clause's refused phrases. What this
 refuses: a clause lost; the fixed order with its lanes reversed; two outcomes swapped, since the stay reads only in a
 part that says no candidate was named by two or more judges and the ship only in a part that says no such thing; and
 a decline said to keep the wording, since the lead clause refuses `keeps`; and an outcome put negatively, since the
 outcome clauses refuse `never`, `does not`, `will not`, `cannot` and `no longer` (re-check 5, N-a). A round also
 refuses the promises of the others: the first ask may not promise a ship or a stay, and a challenge may not say a
 decline settles nothing and asks again. A caller can also name the shape of the declined round, and the sentence must
 then say what is true of that shape: that the panel is asked again over the leaders where some candidates are left
 behind, and that it asks no such question where none is (`decline-clause-shapes.test-fixture.ts`; re-check 6, M-B).
 What it does not read is a meaning no clause names or a negation not listed.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type {
  DeclineClause,
  DeclineMode,
} from './decline-clause.test-fixture.ts';
import { MODES, } from './decline-clause-rounds.test-fixture.ts';
import {
  type RunoffShape,
  shapeClauseOf,
} from './decline-clause-shapes.test-fixture.ts';

export type { DeclineMode, } from './decline-clause.test-fixture.ts';
export type { RunoffShape, } from './decline-clause-shapes.test-fixture.ts';

//region Decline clauses
// The reader: where a text is cut, whether a part says a clause, and what a round's sentence fails.

/**
 Where one way of putting an idea stands in a part of a text.

 @example
 ```ts
 const located: Located = { at: 12, end: 24, };
 ```
 */
type Located = {
  /**
   Index of the way in the part, -1 when the part does not carry it.
   */
  readonly at: number;

  /**
   Index just past the way.
   */
  readonly end: number;
};

/**
 Where the slots of one clause end in one part of a text, read in order.

 @param part - one part of the lower-cased text, cut at every break of {@link PART_BREAKS}

 @param inOrder - slots to find, each after the one before

 @returns Index just past the last slot found, or -1 when a slot is missing after its predecessor

 @example
 ```ts
 const end = endOfSlots({ part: 'your own decline names no candidate', inOrder: [['own decline',], ['no candidate',],], },); // 35
 ```
 */
function endOfSlots(
  {
    part,
    inOrder,
  }: {
    readonly part: string;
    readonly inOrder: readonly (readonly string[])[];
  },
): number {
  return inOrder.reduce(
    function advance(
      from,
      ways,
    ): number {
      if (from < 0)
        return -1;
      /**
       The ways this slot is put that the part carries after the previous slot.
       */
      const found = ways
        .map(function locate(way,): Located {
          /**
           Where this way stands in the part, -1 when it does not.
           */
          const at = part.indexOf(
            way,
            from,
          );
          return {
            at,
            end: at + way.length,
          };
        },)
        .filter(function isPresent({ at, },): boolean {
          return at >= 0;
        },);
      if (found.length === 0)
        return -1;
      /**
       The way that stands first, so a later slot is searched after it.
       */
      const first = found.reduce(function earliest(
        best,
        one,
      ): Located {
        return (one.at < best.at) ? one : best;
      },);
      return first.end;
    },
    0,
  );
}

/**
 Whether one part of a text reads as a clause: it carries every slot in order and none of the clause's refused phrases.

 @param part - one part of the lower-cased text, cut at every break of {@link PART_BREAKS}

 @param clause - clause being read

 @returns Whether the part says this clause

 @example
 ```ts
 const says = partSays({ part: 'your own decline names no candidate', clause: OWN_DECLINE_NAMES_NONE, },);
 ```
 */
function partSays(
  {
    part,
    clause,
  }: {
    readonly part: string;
    readonly clause: DeclineClause;
  },
): boolean {
  return (endOfSlots({
    part,
    inOrder: clause.inOrder,
  },) >= 0)
    && (!clause.forbidden
      .some(function isCarried(refused,): boolean {
        return part.includes(refused,);
      },));
}

/**
 Where a text is cut into parts: a semicolon and a colon, so the lead of the sentence and each outcome are parts of
 their own, and a full stop, so the words a sheet puts after the sentence are not read with its last outcome.
 */
const PART_BREAKS: readonly string[] = [
  ';',
  ':',
  '.',
];

/**
 The text as separate parts, in lower case so a capital at the start of a sentence reads as the same word, cut at
 every break of {@link PART_BREAKS}.

 @param text - sentence or whole sheet a judge reads

 @returns Parts, in reading order

 @example
 ```ts
 const parts = partsOf({ text: 'Declining is real; it counts', },); // ['declining is real', ' it counts']
 ```
 */
function partsOf({ text, }: { readonly text: string; },): readonly string[] {
  return PART_BREAKS.reduce(
    function cutAt(
      pieces: readonly string[],
      breakAt,
    ): readonly string[] {
      return pieces.flatMap(function cut(piece,): readonly string[] {
        return piece.split(breakAt,);
      },);
    },
    [text.toLowerCase(),],
  );
}

/**
 Whether some part of the text says the clause.

 @param parts - text as {@link partsOf} cuts it

 @param clause - clause being read

 @returns Whether any part says it

 @example
 ```ts
 const said = isSaid({ parts, clause: NOT_A_KEEP, },);
 ```
 */
function isSaid(
  {
    parts,
    clause,
  }: {
    readonly parts: readonly string[];
    readonly clause: DeclineClause;
  },
): boolean {
  return parts.some(function carries(part,): boolean {
    return partSays({
      part,
      clause,
    },);
  },);
}

/**
 What a round's sentence fails: the clauses it must say and does not, then the promises it must not make and does.

 @param text - sentence or whole sheet a judge reads

 @param mode - round the sentence is told in

 @param shape - shape of the declined round, when the caller knows it, whose clause the sentence must also say

 @returns Meanings of the clauses no part says, then `promises: ` and the meaning of each refused clause some part
 says; empty when the sentence is right for its round

 @example
 ```ts
 const failures = unmetClausesOf({ text: sheetText, mode: 'asked-again', },);
 const forShape = unmetClausesOf({ text: sheetText, mode: 'ships', shape: 'slate-of-two', },);
 ```
 */
export function unmetClausesOf(
  {
    text,
    mode,
    shape,
  }: {
    readonly text: string;
    readonly mode: DeclineMode;
    readonly shape?: RunoffShape;
  },
): readonly string[] {
  /**
   The text as separate parts.
   */
  const parts = partsOf({ text, },);
  /**
   What this round must say and must not promise.
   */
  const {
    required: general,
    refused,
  } = MODES[mode];
  /**
   What this round must say, with the clause of the shape when one is named.
   */
  const required = (shape === undefined)
    ? general
    : [
      ...general,
      shapeClauseOf({
        shape,
        mode,
      },),
    ];
  return [
    ...required
      .filter(function isMissing(clause,): boolean {
        return !isSaid({
          parts,
          clause,
        },);
      },)
      .map(function toMeaning({ meaning, },): string {
        return meaning;
      },),
    ...refused
      .filter(function isPromised(clause,): boolean {
        return isSaid({
          parts,
          clause,
        },);
      },)
      .map(function toPromise({ meaning, },): string {
        return `promises: ${meaning}`;
      },),
  ];
}

/**
 The meanings of the clauses a text does not carry for a challenge whose caller ships past a decline, in sentence
 order.

 @param text - sentence or whole sheet a judge reads

 @returns Failures of {@link unmetClausesOf} in the `ships` round, empty when every clause is read

 @example
 ```ts
 const unmet = unmetDeclineClauses(sheetText,);
 ```
 */
export function unmetDeclineClauses(text: string,): readonly string[] {
  return unmetClausesOf({
    text,
    mode: 'ships',
  },);
}

//endregion Decline clauses
