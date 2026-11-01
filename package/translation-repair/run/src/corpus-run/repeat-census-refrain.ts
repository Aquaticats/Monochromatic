import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  CLAUSE_BOUNDARY,
  REFRAIN_CLAUSE_MINIMUM,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

import {
  type CensusGroup,
  type CensusOccurrence,
  flagsOf,
  insideQuote,
  type RefrainSetting,
  sanitize,
  USABLE_KINDS,
} from './repeat-census-lines.ts';
import { WHITESPACE_RUN, } from './repeat-census-patterns.ts';

//region Repeat census refrain
// The refrain side of the census: clauses cut from the usable lines, and the
// frames that two distinct clauses share under one setting. Like the design's
// census it counts UTF-16 units, and it reads none of the package's finders, so
// that its numbers stay the design's own.

/**
 Clause of a usable line, with the place it stands.
 */
export type CensusClause = {
  /**
   One-based line the clause stands on.
   */
  readonly line: number;

  /**
   Role of that line.
   */
  readonly kind: string;

  /**
   Whether the clause begins inside an open quotation.
   */
  readonly inQuote: boolean;

  /**
   Wording of the clause, with whitespace removed.
   */
  readonly text: string;
};

/**
 Clauses of the usable lines: runs of wording between the census's clause
 boundaries, of at least six characters.

 @param lines - raw lines

 @param kinds - role of each line

 @returns Clauses with their line, role, column and text

 @example
 ```ts
 const clauses = censusClausesOf({ lines, kinds, },);
 ```
 */
export function censusClausesOf(
  {
    lines,
    kinds,
  }: {
    readonly lines: readonly string[];
    readonly kinds: readonly string[];
  },
): readonly CensusClause[] {
  /**
   Clauses found, in page order.
   */
  const clauses: CensusClause[] = [];
  for (const [index, line,] of lines.entries()) {
    /**
     Role of this line.
     */
    const kind = nonNullishOrThrow(kinds[index],);
    if (!USABLE_KINDS.has(kind,))
      continue;
    /**
     Line with markup blanked and whitespace removed.
     */
    const norm = sanitize({ line, },)
      .replace(
      WHITESPACE_RUN,
      '',
      );

    /**
     Start of the clause being read.
     */
    let start = 0;
    for (let column = 0; column <= norm.length; column++) {
      /**
       Character ending the clause here, or the end of the line.
       */
      const edge = (column === norm.length) ? '' : norm.charAt(column,);
      if ((edge === '') || CLAUSE_BOUNDARY.includes(edge,)) {
        if ((column - start) >= REFRAIN_CLAUSE_MINIMUM) {
          clauses.push({
            line: index + 1,
            kind,
            inQuote: insideQuote({
              norm,
              column: start,
            },),
            text: norm.slice(
              start,
              column,
            ),
          },);
        }
        start = column + 1;
      }
    }
  }
  return clauses;
}

/**
 Length of the common prefix of two strings, in UTF-16 units.

 @param a - first string

 @param b - second string

 @returns Count of equal leading units

 @example
 ```ts
 const shared = sharedPrefixLength({ a: '猫咪', b: '猫狗', },);
 ```
 */
function sharedPrefixLength({
  a,
  b,
}: {
  readonly a: string;
  readonly b: string
},): number {
  /**
   Count of leading units equal so far.
   */
  let length = 0;
  while ((length < a.length) && (length < b.length)
    && (a.charAt(length,) === b.charAt(length,)))
    length++;
  return length;
}

/**
 Length of the common suffix of two strings, in UTF-16 units.

 @param a - first string

 @param b - second string

 @returns Count of equal trailing units

 @example
 ```ts
 const shared = sharedSuffixLength({ a: '胖猫咪吧', b: '瘦猫咪吧', },);
 ```
 */
function sharedSuffixLength({
  a,
  b,
}: {
  readonly a: string;
  readonly b: string
},): number {
  /**
   Count of trailing units equal so far.
   */
  let length = 0;
  while (
    (length < a.length)
      && (length < b.length)
      && (a.charAt(a.length - 1
        - length,) === b.charAt(b.length - 1
          - length,))
  )
    length++;
  return length;
}

/**
 Refrain frames of the clauses under one setting: frames held by at least two
 distinct clauses.

 @param clauses - clauses of the page

 @param setting - thresholds to apply

 @returns Frames with the clauses carrying each

 @example
 ```ts
 const frames = refrainFrames({ clauses, setting: REFRAIN_SETTINGS.main, },);
 ```
 */
function refrainFrames(
  {
    clauses,
    setting,
  }: {
    readonly clauses: readonly CensusClause[];
    readonly setting: RefrainSetting;
  },
): readonly {
  readonly prefixLength: number;
  readonly suffixLength: number;
  readonly members: readonly number[];
}[] {
  /**
   Clause indexes by their leading characters.
   */
  const buckets = new Map<string, number[]>();
  for (const [index, clause,] of clauses.entries()) {
    if (clause.text
      .length
      < setting.minimumClause)
      continue;
    /**
     Leading characters the clause is bucketed under.
     */
    const lead = clause.text
      .slice(
      0,
      setting.prefixMinimum,
      );
    /**
     Clauses already bucketed here.
     */
    const bucket = buckets.get(lead,) ?? [];
    bucket.push(index,);
    buckets.set(
      lead,
      bucket,
    );
  }

  /**
   Clause indexes carried by each frame.
   */
  const frames = new Map<string, {
    readonly prefixLength: number;
    readonly suffixLength: number;
    readonly members: Set<number>;
  }>();
  for (const bucket of buckets.values()) {
    for (const [x, left,] of bucket.entries()) {
      for (const right of bucket.slice(x + 1,)) {
        /**
         First clause of the pair.
         */
        const a = nonNullishOrThrow(clauses[left],)
          .text;

        /**
         Second clause of the pair.
         */
        const b = nonNullishOrThrow(clauses[right],)
          .text;
        if (a === b)
          continue;
        /**
         Shared prefix of the pair.
         */
        const prefixLength = sharedPrefixLength({
          a,
          b,
        },);

        /**
         Shared suffix of the pair, capped so each slot keeps one character.
         */
        const suffixLength = Math.min(
          sharedSuffixLength({
            a,
            b,
          },),
          Math.min(
            a.length,
            b.length,
          ) - 1
            - prefixLength,
        );
        if ((prefixLength < setting.prefixMinimum) || (suffixLength < setting.suffixMinimum))
          continue;
        /**
         Slot of the first clause.
         */
        const slotA = a.length - prefixLength
          - suffixLength;

        /**
         Slot of the second clause.
         */
        const slotB = b.length - prefixLength
          - suffixLength;
        if ((slotA < 1) || (slotB < 1)
          || (slotA > setting.maximumSlot)
          || (slotB > setting.maximumSlot))
          continue;
        /**
         Key of the frame: its prefix and suffix text.
         */
        const key = `${String(prefixLength,)}/${String(suffixLength,)}/${a.slice(
          0,
          prefixLength,
        )}|${
          a.slice(a.length - suffixLength,)
        }`;
        /**
         Frame record, created by the first pair that forms it.
         */
        const frame = frames.get(key,) ?? {
          prefixLength,
          suffixLength,
          members: new Set<number>(),
        };
        frame.members
          .add(left,);
        frame.members
          .add(right,);
        frames.set(
          key,
          frame,
        );
      }
    }
  }
  return [...frames.values(),]
    .filter(function heldByTwo(frame,): boolean {
      return frame.members
        .size
        >= 2;
    },)
    .map(function sorted(frame,) {
      return {
        prefixLength: frame.prefixLength,
        suffixLength: frame.suffixLength,
        members: [...frame.members,].toSorted(function ascending(
          a,
          b,
        ): number {
          return a - b;
        },),
      };
    },);
}

/**
 Refrain groups of a page under one setting, with their flags.

 @param clauses - clauses of the page

 @param setting - thresholds to apply

 @returns Refrain groups

 @example
 ```ts
 const groups = refrainGroups({ clauses, setting: REFRAIN_SETTINGS.main, },);
 ```
 */
export function refrainGroups(
  {
    clauses,
    setting,
  }: {
    readonly clauses: readonly CensusClause[];
    readonly setting: RefrainSetting;
  },
): readonly CensusGroup[] {
  return refrainFrames({
    clauses,
    setting,
  },)
    .map(function asGroup(frame,): CensusGroup {
    /**
     Occurrences of the frame's clauses.
     */
      const occurrences = frame.members
        .map(function occurrenceOfClause(index,): CensusOccurrence {
      /**
       Clause the occurrence is.
       */
          const clause = nonNullishOrThrow(clauses[index],);
          return {
            line: clause.line,
            kind: clause.kind,
            inQuote: clause.inQuote,
          };
        },);
      return {
        kind: 'refrain',
        length: 0,
        occurrences,
        flags: flagsOf({ occurrences, },),
      };
    },);
}

//endregion Repeat census refrain
