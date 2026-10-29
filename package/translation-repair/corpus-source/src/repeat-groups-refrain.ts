import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  BREAK,
  CLAUSE_BOUNDARY,
  REFRAIN_CLAUSE_MINIMUM,
  REFRAIN_PREFIX_MINIMUM,
  REFRAIN_SLOT_MAXIMUM,
  REFRAIN_SLOT_MINIMUM,
  REFRAIN_SUFFIX_MINIMUM,
} from './repeat-groups-bounds.ts';
import {
  quotedCountsOf,
  touchesQuotation,
} from './repeat-groups-quotation.ts';
import type {
  Clause,
  WordingStream,
} from './repeat-groups-types.ts';

//region Repeat groups refrain
// The refrain finders: the clauses cut from the wording stream, and the frames
// two clauses share.

/**
 Clauses of the wording stream: runs of wording between clause boundaries,
 breaks and the end, of at least the clause minimum.

 @param wording - wording stream of the page

 @param stream - characters of that stream, as one string per character

 @returns Clauses in page order

 @example
 ```ts
 const clauses = clausesOf({ wording, stream, },);
 ```
 */
export function clausesOf(
  {
    wording,
    stream,
  }: {
    readonly wording: WordingStream;
    readonly stream: readonly string[];
  },
): readonly Clause[] {
  /**
   Clauses found, in page order.
   */
  const clauses: Clause[] = [];

  /**
   Quoted characters before each place in the stream. Every quotation mark is a clause boundary,
   so no clause holds one and the depth is the same at every character of a clause: no case can
   tell this reading from one at the clause's first character, and it is the one every finder shares.
   */
  const quotedCounts = quotedCountsOf({ wording, },);

  /**
   Stream index where the clause being read begins.
   */
  let start = 0;
  for (let index = 0; index <= stream.length; index++) {
    /**
     Character ending the clause here, or the end of the stream.
     */
    const edge = (index === stream.length) ? BREAK : nonNullishOrThrow(stream[index],);
    if ((edge !== BREAK) && (!CLAUSE_BOUNDARY.includes(edge,)))
      continue;
    if ((index - start) >= REFRAIN_CLAUSE_MINIMUM) {
      /**
       Characters of the clause, one code point each.
       */
      const points = stream.slice(
        start,
        index,
      );
      clauses.push({
        start,
        end: index,
        points,
        text: points.join('',),
        quoted: touchesQuotation({
          counts: quotedCounts,
          start,
          end: index,
        },),
      },);
    }
    start = index + 1;
  }
  return clauses;
}

/**
 Length of the common prefix of two character lists.

 @param a - first list, one code point per element

 @param b - second list, one code point per element

 @returns Count of equal leading characters

 @example
 ```ts
 const shared = commonPrefixLength({ a: [...'猫咪',], b: [...'猫狗',], },);
 ```
 */
function commonPrefixLength({
  a,
  b,
}: {
  readonly a: readonly string[];
  readonly b: readonly string[]
},): number {
  /**
   Count of leading characters equal so far.
   */
  let length = 0;
  while ((length < a.length) && (length < b.length)
    && (a[length] === b[length]))
    length++;
  return length;
}

/**
 Length of the common suffix of two character lists.

 @param a - first list, one code point per element

 @param b - second list, one code point per element

 @returns Count of equal trailing characters

 @example
 ```ts
 const shared = commonSuffixLength({ a: [...'胖猫咪吧',], b: [...'瘦猫咪吧',], },);
 ```
 */
function commonSuffixLength({
  a,
  b,
}: {
  readonly a: readonly string[];
  readonly b: readonly string[]
},): number {
  /**
   Count of trailing characters equal so far.
   */
  let length = 0;
  while (
    (length < a.length)
      && (length < b.length)
      && (a[a.length - 1
        - length] === b[b.length - 1
          - length])
  )
    length++;
  return length;
}

/**
 Refrain frames: clauses that share a prefix and a suffix and differ only in a
 slot of the bounded length, grouped by frame, with the frames that hold at
 least two distinct clauses.

 COST: clauses are bucketed by their first four and last three elements, which
 two clauses of one frame must share, and every pair inside a bucket is
 compared. The work grows with the square of the largest bucket, so a page whose
 clauses nearly all open and close alike is the slow case, and prose is not.

 @param clauses - clauses of the page in page order

 @returns Frames with the clauses carrying each, the clauses of a frame in page order
 and the frames in no stated order

 @example
 ```ts
 const frames = refrainFramesOf({ clauses, },);
 ```
 */
export function refrainFramesOf(
  {
    clauses,
  }: {
    readonly clauses: readonly Clause[];
  },
): readonly {
  readonly prefix: string;
  readonly suffix: string;
  readonly prefixLength: number;
  readonly suffixLength: number;
  readonly members: readonly Clause[];
}[] {
  /**
   Clauses that share their first prefix-minimum characters and their last
   suffix-minimum characters, which two clauses of one frame must.
   */
  const buckets = new Map<string, Clause[]>();
  for (const clause of clauses) {
    /**
     Leading characters of the clause.
     */
    const head = clause.points
      .slice(
      0,
      REFRAIN_PREFIX_MINIMUM,
      )
      .join('',);

    /**
     Trailing characters of the clause.
     */
    const tail = clause.points
      .slice(-REFRAIN_SUFFIX_MINIMUM,)
      .join('',);

    /**
     Leading characters, a break, then trailing characters: the key the clause is bucketed under.
     No clause holds a break, so the key reads one way.
     */
    const lead = `${head}${BREAK}${tail}`;
    /**
     Clauses already in this bucket.
     */
    const bucket = buckets.get(lead,) ?? [];
    bucket.push(clause,);
    buckets.set(
      lead,
      bucket,
    );
  }

  /**
   Clauses carried by each frame, keyed by prefix length, prefix and suffix.
   */
  const frames = new Map<string, {
    readonly prefix: string;
    readonly suffix: string;
    readonly prefixLength: number;
    readonly suffixLength: number;
    readonly members: Set<Clause>;
  }>();
  for (const bucket of buckets.values()) {
    for (const [x, left,] of bucket.entries()) {
      for (const right of bucket.slice(x + 1,)) {
        if (left.text === right.text)
          continue;
        /**
         Shared prefix of the pair, at least the prefix minimum.
         */
        const prefixLength = commonPrefixLength({
          a: left.points,
          b: right.points,
        },);
        /**
         Shared suffix of the pair, capped so each slot keeps one character.
         */
        const suffixLength = Math.min(
          commonSuffixLength({
            a: left.points,
            b: right.points,
          },),
          Math.min(
            left.points
              .length,
            right.points
              .length,
          ) - 1
            - prefixLength,
        );
        if ((prefixLength < REFRAIN_PREFIX_MINIMUM) || (suffixLength < REFRAIN_SUFFIX_MINIMUM))
          continue;
        /**
         Slot length of the first clause.
         */
        const leftSlot = left.points
          .length
          - prefixLength
          - suffixLength;
        /**
         Slot length of the second clause.
         */
        const rightSlot = right.points
          .length
          - prefixLength
          - suffixLength;
        if (
          (leftSlot < REFRAIN_SLOT_MINIMUM)
            || (rightSlot < REFRAIN_SLOT_MINIMUM)
            || (leftSlot > REFRAIN_SLOT_MAXIMUM)
            || (rightSlot > REFRAIN_SLOT_MAXIMUM)
        )
          continue;
        /**
         Prefix the pair shares.
         */
        const prefix = left.points
          .slice(
          0,
          prefixLength,
          )
          .join('',);
        /**
         Suffix the pair shares.
         */
        const suffix = left.points
          .slice(left.points
            .length
            - suffixLength,)
          .join('',);
        /**
         Key of this frame, unambiguous because the prefix length leads it.
         */
        const key = `${String(prefixLength,)}:${prefix}:${suffix}`;
        /**
         Frame record, created on the first pair that forms it.
         */
        const frame = frames.get(key,) ?? {
          prefix,
          suffix,
          prefixLength,
          suffixLength,
          members: new Set<Clause>(),
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
    .filter(function holdsTwo(frame,): boolean {
      return frame.members
        .size
        >= 2;
    },)
    .map(function inPageOrder(frame,) {
      return {
        prefix: frame.prefix,
        suffix: frame.suffix,
        prefixLength: frame.prefixLength,
        suffixLength: frame.suffixLength,
        members: [...frame.members,].toSorted(function byStart(
          a,
          b,
        ): number {
          return a.start - b.start;
        },),
      };
    },);
}

//endregion Repeat groups refrain
