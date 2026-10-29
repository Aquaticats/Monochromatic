import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type CensusGroup,
  type CensusOccurrence,
  flagsOf,
  insideQuote,
  sanitize,
  USABLE_KINDS,
} from './repeat-census-lines.ts';
import { WHITESPACE_RUN, } from './repeat-census-patterns.ts';

//region Repeat census maximal
// The maximal repeats of the census: non-space runs of the census length that
// stand at two or more places, extended while every occurrence agrees.

/**
 Sentinel between lines, so that no run spans two of them.
 */
const LINE_BREAK = String.fromCodePoint(0,);

/**
 Shortest maximal repeat, in non-space characters, as the census measured it.
 */
const MAXIMAL_K = 12;

/**
 Whether every occurrence reads the same wording character at one shift from
 its start, and that character is not a sentinel or the end.

 @param stream - characters of the page with sentinels

 @param positions - start of each occurrence

 @param shift - offset from each start, negative to read before it

 @returns Whether all occurrences agree there

 @example
 ```ts
 const agree = agreesAt({ stream, positions: [0, 14,], shift: 12, },);
 ```
 */
function agreesAt(
  {
    stream,
    positions,
    shift,
  }: {
    readonly stream: string;
    readonly positions: readonly number[];
    readonly shift: number;
  },
): boolean {
  /**
   Character each occurrence reads, the sentinel where the read leaves the stream.
   */
  const reads = positions.map(function readAt(position,): string {
    /**
     Index read for this occurrence.
     */
    const index = position + shift;
    if ((index < 0) || (index >= stream.length))
      return LINE_BREAK;
    return stream.charAt(index,);
  },);
  /**
   Character the first occurrence reads.
   */
  const [first,] = reads;
  return (first !== LINE_BREAK) && reads.every(function same(read,): boolean {
    return read === first;
  },);
}

/**
 Maximal repeated runs of the census length, found over the non-space
 characters of the usable lines.

 @param lines - raw lines

 @param kinds - role of each line

 @returns Groups with their length and occurrences

 @example
 ```ts
 const groups = maximalRepeats({ lines, kinds, },);
 ```
 */
export function maximalRepeats(
  {
    lines,
    kinds,
  }: {
    readonly lines: readonly string[];
    readonly kinds: readonly string[];
  },
): readonly CensusGroup[] {
  /**
   Normalised text of each usable line, empty for the others.
   */
  const norms = lines.map(function normOf(
    line,
    index,
  ): string {
    return USABLE_KINDS.has(nonNullishOrThrow(kinds[index],),)
      ? sanitize({ line, },)
        .replace(
        WHITESPACE_RUN,
        '',
        )
      : '';
  },);

  /**
   Characters of all usable lines, with a sentinel after each line.
   */
  const chars: string[] = [];

  /**
   One-based line each character stands on, zero at a sentinel.
   */
  const lineOf: number[] = [];

  /**
   Column each character stands at in its line, minus one at a sentinel.
   */
  const columnOf: number[] = [];
  for (const [index, norm,] of norms.entries()) {
    for (let column = 0; column < norm.length; column++) {
      chars.push(norm.charAt(column,),);
      lineOf.push(index + 1,);
      columnOf.push(column,);
    }
    chars.push(LINE_BREAK,);
    lineOf.push(0,);
    columnOf.push(-1,);
  }
  /**
   Whole stream of characters, for matching.
   */
  const stream = chars.join('',);

  /**
   Start positions of each run of the census length, by run.
   */
  const starts = new Map<string, number[]>();
  for (let index = 0; (index + MAXIMAL_K) <= stream.length; index++) {
    /**
     Run beginning at this index.
     */
    const run = stream.slice(
      index,
      index + MAXIMAL_K,
    );
    if (run.includes(LINE_BREAK,))
      continue;
    /**
     Positions of this run so far.
     */
    const positions = starts.get(run,) ?? [];
    positions.push(index,);
    starts.set(
      run,
      positions,
    );
  }

  /**
   Keys of the runs already reported.
   */
  const seen = new Set<string>();

  /**
   Groups found.
   */
  const groups: CensusGroup[] = [];
  for (const positions of starts.values()) {
    if (positions.length < 2)
      continue;
    /**
     Characters the occurrences agree on after the census length.
     */
    let right = 0;
    while (agreesAt({
      stream,
      positions,
      shift: MAXIMAL_K + right,
    },))
      right++;
    /**
     Characters the occurrences agree on before the run.
     */
    let left = 0;
    while (agreesAt({
      stream,
      positions,
      shift: -(left + 1),
    },))
      left++;
    /**
     Length of the extended run.
     */
    const length = MAXIMAL_K + left
      + right;
    /**
     Starts of the extended run, in order, keeping those that do not overlap.
     */
    const kept: number[] = [];

    /**
     Index past the run of the last kept start.
     */
    let end = -1;
    for (const start of positions.map(function shiftLeft(position,): number {
      return position - left;
    },)
      .toSorted(function ascending(
        a,
        b,
      ): number {
        return a - b;
      },)) {
      if (start < end)
        continue;
      kept.push(start,);
      end = start + length;
    }
    if (kept.length < 2)
      continue;
    /**
     Key of this run, for de-duplication.
     */
    const key = `${String(length,)}:${kept.join(',',)}`;
    if (seen.has(key,))
      continue;
    seen.add(key,);
    /**
     Occurrences of the run, each with its line, role and quotation.
     */
    const occurrences = kept.map(function occurrenceAt(start,): CensusOccurrence {
      /**
       Line the occurrence starts on.
       */
      const line = nonNullishOrThrow(lineOf[start],);
      return {
        line,
        kind: nonNullishOrThrow(kinds[line - 1],),
        inQuote: insideQuote({
          norm: nonNullishOrThrow(norms[line - 1],),
          column: nonNullishOrThrow(columnOf[start],),
        },),
      };
    },);
    groups.push({
      kind: 'maximal',
      length,
      occurrences,
      flags: flagsOf({ occurrences, },),
    },);
  }
  return groups;
}

//endregion Repeat census maximal
