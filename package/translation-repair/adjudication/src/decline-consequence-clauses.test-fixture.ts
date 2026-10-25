/**
 The clauses a judge must be able to read in the sentence that tells it what a decline does over wording that cannot
 ship, written as the meaning each carries rather than as a copy of the constant's words (re-check 3, N1 and N7).

 A copied substring fails on any rewording, including one that keeps every meaning. Each clause here is a list of
 slots, and a slot is the ways one idea can be put. A clause is read when some `;`-separated part of the text carries
 one way of each slot, in slot order, so a rewording that keeps the meaning passes and a lost or reversed clause fails.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

//region Decline clauses
// One entry per thing the sentence must say, in the order a judge meets them.

/**
 One thing the sentence must say.

 @example
 ```ts
 const clause: DeclineClause = { meaning: 'a decline names no candidate', inOrder: [['own decline',], ['no candidate',],], };
 ```
 */
export type DeclineClause = {
  /**
   What the clause tells a judge, as it is named in a failure.
   */
  readonly meaning: string;

  /**
   The slots in the order a judge reads them, each the ways the idea can be put, in lower case.
   */
  readonly inOrder: readonly (readonly string[])[];
};

/**
 What a judge must be told, in the order the sentence says it.

 THE FIRST CLAUSE IS WHAT KEEPS A JUDGE FROM READING ITS DECLINE AS THE WAY TO THE STAY: declining alone does not keep
 the wording in place. The second says the same decline names no candidate, so it adds to no candidate's count. The
 third and fourth are the two outcomes, each in its own `;`-separated part of the sentence: none named by two or more
 judges leaves the wording in place and the page reports it, and one or more named by two or more judges ships the
 preferred of those. The last is the fixed order, the behaviour of `rankedByPreference` in
 `translate-stage/src/translate-runoff-tie.ts`.

 @example
 ```ts
 const unmet = unmetDeclineClauses(sheetText,);
 ```
 */
export const DECLINE_CLAUSES: readonly DeclineClause[] = [
  {
    meaning: 'declining does not by itself keep the wording in place',
    inOrder: [
      ['declin',],
      [
        'does not',
        'will not',
        'is not',
      ],
      [
        'by itself',
        'alone',
        'on its own',
        'quiet',
      ],
      ['keep',],
    ],
  },
  {
    meaning: 'a judge\'s own decline names no candidate',
    inOrder: [
      ['own decline',],
      [
        'no candidate',
        'not name a candidate',
        'names none',
      ],
    ],
  },
  {
    meaning: 'no candidate named by two or more judges: the wording in place stays and the page reports it',
    inOrder: [
      ['no candidate',],
      [
        'named by two or more judges',
        'named by two judges or more',
      ],
      [
        'stays',
        'remains',
      ],
      ['report',],
    ],
  },
  {
    meaning: 'one or more candidates named by two or more judges: the preferred of those ships',
    inOrder: [
      [
        'named by two or more judges',
        'named by two judges or more',
      ],
      ['preferred',],
      [
        'ships',
        'ship',
      ],
    ],
  },
  {
    meaning: 'the preferred ships by a fixed order, the repair lane first, then the translate lane, then the slate',
    inOrder: [
      ['fixed order',],
      ['repair lane',],
      ['translate lane',],
      ['first on the slate',],
    ],
  },
];

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

 @param part - one `;`-separated part of the lower-cased text

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
 The meanings of the clauses a text does not carry, in sentence order.

 @param text - sentence or whole sheet a judge reads

 @returns The meanings that no part of the text satisfies, empty when every clause is read

 @example
 ```ts
 const unmet = unmetDeclineClauses(sheetText,);
 ```
 */
export function unmetDeclineClauses(text: string,): readonly string[] {
  /**
   The text as separate parts, in lower case so a capital at the start of a sentence reads as the same word.
   */
  const parts = text.toLowerCase()
    .split(';',);
  return DECLINE_CLAUSES
    .filter(function isMissing({ inOrder, },): boolean {
      return !parts.some(function carries(part,): boolean {
        return endOfSlots({
          part,
          inOrder,
        },) >= 0;
      },);
    },)
    .map(function toMeaning({ meaning, },): string {
      return meaning;
    },);
}

//endregion Decline clauses
