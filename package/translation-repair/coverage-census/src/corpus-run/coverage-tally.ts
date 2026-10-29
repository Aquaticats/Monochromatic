import {
  compareCodePoints,
  textsInCodePointOrder,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  BundleScript,
  CoverageRange,
} from './coverage-file.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Coverage tally
// LEDGER T8: WHICH BUNDLE CODE NO TEST PROCESS RAN, over every file
// `NODE_V8_COVERAGE` wrote. V8 reports, per process and per script, each
// function as a tree of ranges whose innermost count is the count of the code
// under it, and it DROPS A NESTED RANGE WHOSE COUNT EQUALS ITS PARENT'S
// (`MergeNestedRanges` in V8's `src/debug/debug-coverage.cc`). A range one
// process lists as cold can therefore have run in a process that does not
// list it: summing the listed ranges alone called 1,061 of 2,371 blocks cold
// that ran (ledger T8, 2026-09-29).
//
// THE TALLY CUTS each bundle at every range boundary any process reported,
// gives each piece the count of its innermost enclosing range in each process,
// and sums over processes. That needs every boundary before the first piece is
// counted, so the coverage files are read twice: once through
// `noteBoundaries`, then once through `paint`.
//
// MUTABLE INSIDE, BY DESIGN. A full unit suite writes about 8 GB of coverage
// over some 860 processes; the tally folds each process into per-bundle typed
// arrays as it is read, rather than holding every process's ranges at once.
// The state lives in the factory's closure, and callers see only the frozen
// methods.
//
// FUNCTIONS ARE KEYED BY BOTH ENDS. A function that opens its chunk starts at
// offset 0, as the chunk's own top-level script function does, and keying by
// the start alone merged the two: the function took the script's empty name
// and the script's count (all five "unmatched" exports of 2026-09-28).

/**
 A tally asked to note a boundary after painting began, or handed a bundle or
 a range the first reading never reported.

 @example
 ```ts
 throw new CoverageTallyError({ says: 'a boundary was noted after painting began', },);
 ```
 */
export class CoverageTallyError extends Error {
  /**
   Declares this message safe to forward: it names bundles and offsets this
   process read out of its own build.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal.

   @param says - what the two readings disagreed on

   @example
   ```ts
   new CoverageTallyError({ says: 'a boundary was noted after painting began', },);
   ```
   */
  constructor({ says, }: { readonly says: string; },) {
    super(`the coverage tally cannot count: ${says}; both readings of the coverage directory must see the same files, in order`,);
    this.name = 'CoverageTallyError';
  }
}

/**
 One bundle cut at every boundary any process reported, with each piece's
 count summed over the processes painted so far.
 */
type BundlePieces = {
  /**
   Every boundary, ascending; piece `i` runs from `offsets[i]` to `offsets[i + 1]`.
   */
  readonly offsets: readonly number[];

  /**
   Position of each boundary in `offsets`.
   */
  readonly indexOf: ReadonlyMap<number, number>;

  /**
   Summed count per piece.
   */
  readonly total: Float64Array;

  /**
   1 where some process had a range over the piece.
   */
  readonly covered: Uint8Array;
};

/**
 Where a function sits and what V8 called it.

 @example
 ```ts
 const site: FunctionSite = { bundle: 'index.mjs', start: 10, end: 20, name: 'nap', };
 ```
 */
export type FunctionSite = {
  /**
   Bundle holding it.
   */
  readonly bundle: string;

  /**
   Its first offset.
   */
  readonly start: number;

  /**
   One past its last offset.
   */
  readonly end: number;

  /**
   Name V8 gave it, empty for an anonymous function.
   */
  readonly name: string;
};

/**
 A stretch of bundle code no process ran: a whole function no process called,
 or code inside a function that ran.

 @example
 ```ts
 const stretch: ColdStretch = { bundle: 'index.mjs', start: 10, end: 20, shape: { kind: 'block', }, };
 ```
 */
export type ColdStretch = {
  /**
   Bundle holding it.
   */
  readonly bundle: string;

  /**
   Its first offset.
   */
  readonly start: number;

  /**
   One past its last offset.
   */
  readonly end: number;

  /**
   A whole uncalled function with its name, or code inside a function.
   */
  readonly shape: { readonly kind: 'block'; } | {
    readonly kind: 'function';
    readonly name: string;
  };
};

/**
 A function no process called.

 @example
 ```ts
 const uncalled: UncalledFunction = { bundle: 'index.mjs', start: 10, end: 20, name: 'nap', nested: false, };
 ```
 */
export type UncalledFunction = FunctionSite & {
  /**
   Whether it sits inside another function no process called, so a test
   reaching the outer one is where the work starts.
   */
  readonly nested: boolean;
};

/**
 Counts which bundle code the unit suite ran, over every process's coverage
 file, read twice.

 @example
 ```ts
 const tally = createCoverageTally();
 ```
 */
export type CoverageTally = {
  /**
   Notes every boundary one process reported; the first reading.
   */
  readonly noteBoundaries: (input: { readonly scripts: readonly BundleScript[]; },) => void;

  /**
   Adds one process's counts; the second reading.
   */
  readonly paint: (input: { readonly scripts: readonly BundleScript[]; },) => void;

  /**
   Every stretch of loaded bundle code no process ran.
   */
  readonly coldStretches: () => readonly ColdStretch[];

  /**
   Every function no process called.
   */
  readonly uncalledFunctions: () => readonly UncalledFunction[];

  /**
   Bundles some process loaded, sorted.
   */
  readonly loadedBundles: () => readonly string[];
};

/**
 Key of one function in one bundle, by both ends.

 @param site - function's place

 @returns Key unique within the tally

 @example
 ```ts
 functionKey({ bundle: 'index.mjs', start: 0, end: 40, },); // 'index.mjs|0|40'
 ```
 */
function functionKey(site: Pick<FunctionSite, 'bundle' | 'end' | 'start'>,): string {
  return `${site.bundle}|${String(site.start,)}|${String(site.end,)}`;
}

/**
 Whether one piece is code some process loaded and none ran.

 @param pieces - bundle's pieces after every process painted

 @param index - piece asked about

 @returns True for a cold piece

 @example
 ```ts
 isColdPiece({ pieces, index: 3, },);
 ```
 */
function isColdPiece(
  {
    pieces,
    index,
  }: {
    readonly pieces: BundlePieces;
    readonly index: number;
  },
): boolean {
  return (pieces.covered[index] === 1) && (pieces.total[index] === 0);
}

/**
 Position of a boundary among a bundle's pieces.

 @param bundle - bundle name, for the refusal

 @param pieces - its pieces

 @param offset - boundary a process reported

 @returns Index of that boundary

 @throws CoverageTallyError where the first reading never noted the boundary

 @example
 ```ts
 boundaryIndex({ bundle: 'index.mjs', pieces, offset: 40, },);
 ```
 */
function boundaryIndex(
  {
    bundle,
    pieces,
    offset,
  }: {
    readonly bundle: string;
    readonly pieces: BundlePieces;
    readonly offset: number;
  },
): number {
  /**
   Its position.
   */
  const index = pieces.indexOf
    .get(offset,);
  if (index === undefined)
    throw new CoverageTallyError({ says: `${bundle} offset ${String(offset,)} was not in the first reading`, },);
  return index;
}

/**
 Last piece of the cold run that starts at `from`.

 @param pieces - bundle's pieces after every process painted

 @param from - first piece of the run, itself cold

 @returns Index of the run's last piece

 @example
 ```ts
 lastColdPiece({ pieces, from: 3, },);
 ```
 */
function lastColdPiece(
  {
    pieces,
    from,
  }: {
    readonly pieces: BundlePieces;
    readonly from: number;
  },
): number {
  /**
   Pieces there are: `offsets` holds one more boundary than that.
   */
  const count = pieces.offsets
    .length
    - 1;
  for (let piece = from; (piece + 1) < count; piece += 1) {
    if (!isColdPiece({
      pieces,
      index: piece + 1,
    },))
      return piece;
  }
  return count - 1;
}

/**
 Merges one bundle's cold pieces into stretches.

 @param bundle - bundle name

 @param pieces - its pieces after every process painted

 @param calls - summed call count per function key

 @param sites - function per key

 @returns Its cold stretches in bundle order

 @example
 ```ts
 const stretches = stretchesOf({ bundle, pieces, calls, sites, },);
 ```
 */
function stretchesOf(
  {
    bundle,
    pieces,
    calls,
    sites,
  }: {
    readonly bundle: string;
    readonly pieces: BundlePieces;
    readonly calls: ReadonlyMap<string, number>;
    readonly sites: ReadonlyMap<string, FunctionSite>;
  },
): readonly ColdStretch[] {
  /**
   First piece of each cold run: a cold piece with no cold piece before it.
   The last boundary opens no piece, so it is left out.
   */
  const firsts = pieces.offsets
    .slice(
    0,
    -1,
    )
    .flatMap(function runStart(
      _offset,
      index,
    ): readonly number[] {
      /**
       Whether this piece is cold.
       */
      const cold = isColdPiece({
        pieces,
        index,
      },);
      /**
       Whether the piece before it is, where there is one.
       */
      const afterCold = (index > 0) && isColdPiece({
        pieces,
        index: index - 1,
      },);
      return (cold && (!afterCold)) ? [index,] : [];
    },);
  return firsts.map(function stretchFrom(first,): ColdStretch {
    /**
     Its first offset.
     */
    const start = nonNullishOrThrow(pieces.offsets[first],);
    /**
     One past its last character: the boundary after the run's last piece,
     which every piece has.
     */
    const end = nonNullishOrThrow(pieces.offsets[lastColdPiece({
      pieces,
      from: first,
    },) + 1],);
    /**
     The function with exactly these ends, when it is one no process called.
     */
    const key = functionKey({
      bundle,
      start,
      end,
    },);
    /**
     That function.
     */
    const whole = (calls.get(key,) === 0) ? sites.get(key,) : undefined;
    return {
      bundle,
      start,
      end,
      shape: (whole === undefined) ? { kind: 'block', } : {
        kind: 'function',
        name: whole.name,
      },
    };
  },);
}

/**
 Marks each uncalled function sitting inside another.

 @param sites - uncalled functions sorted by bundle, then start, enclosing ones first

 @returns Same, each marked

 @example
 ```ts
 const marked = markNested({ sites, },);
 ```
 */
function markNested({ sites, }: { readonly sites: readonly FunctionSite[]; },): readonly UncalledFunction[] {
  /**
   Outermost uncalled functions met so far; the last one is open.
   */
  const outers: FunctionSite[] = [];
  return sites.map(function marked(site,): UncalledFunction {
    /**
     The open outer function.
     */
    const outer = outers.at(-1,);
    /**
     Whether it holds this one.
     */
    const nested = (outer !== undefined) && (outer.bundle === site.bundle)
      && (site.end <= outer.end);
    if (!nested)
      outers.push(site,);
    return {
      ...site,
      nested,
    };
  },);
}

/**
 Makes an empty tally.

 @returns Its methods, frozen; the counts live in their closure

 @throws CoverageTallyError from `noteBoundaries` once painting has begun, and
 from `paint` where a bundle or boundary was missing from the first reading

 @example
 ```ts
 const tally = createCoverageTally();
 for (const scripts of readings) tally.noteBoundaries({ scripts, },);
 for (const scripts of readings) tally.paint({ scripts, },);
 const cold = tally.coldStretches();
 ```
 */
export function createCoverageTally(): CoverageTally {
  /**
   Every boundary noted per bundle during the first reading.
   */
  const boundaries = new Map<string, Set<number>>();
  /**
   Pieces per bundle, cut when painting begins.
   */
  const pieces = new Map<string, BundlePieces>();
  /**
   Summed call count per function key.
   */
  const calls = new Map<string, number>();
  /**
   Each function by key.
   */
  const sites = new Map<string, FunctionSite>();

  /**
   Cuts every bundle at its noted boundaries, once.
   */
  function cut(): void {
    if (pieces.size > 0)
      return;
    for (const [bundle, held,] of boundaries) {
      /**
       Its boundaries ascending.
       */
      const offsets = [...held,].toSorted(function ascending(
        left,
        right,
      ): number {
        return left - right;
      },);
      pieces.set(
        bundle,
        {
          offsets,
          indexOf: new Map(offsets.map(function located(
            offset,
            index,
          ) {
            return [
              offset,
              index,
            ] as const;
          },),),
          total: new Float64Array(offsets.length,),
          covered: new Uint8Array(offsets.length,),
        },
      );
    }
  }

  return Object.freeze({
    noteBoundaries: function noteBoundaries({ scripts, }: { readonly scripts: readonly BundleScript[]; },): void {
      if (pieces.size > 0)
        throw new CoverageTallyError({ says: 'a boundary was noted after painting began', },);
      for (const script of scripts) {
        /**
         Boundaries of this bundle so far.
         */
        const held = boundaries.get(script.bundle,) ?? new Set<number>();
        for (const fn of script.functions) {
          for (const range of fn.ranges) {
            held.add(range.startOffset,);
            held.add(range.endOffset,);
          }
        }
        boundaries.set(
          script.bundle,
          held,
        );
      }
    },

    paint: function paint({ scripts, }: { readonly scripts: readonly BundleScript[]; },): void {
      cut();
      for (const script of scripts) {
        /**
         The bundle's pieces.
         */
        const bundlePieces = pieces.get(script.bundle,);
        if (bundlePieces === undefined)
          throw new CoverageTallyError({ says: `${script.bundle} was not in the first reading`, },);
        /**
         This process's count per piece.
         */
        const painted = new Float64Array(bundlePieces.offsets
          .length,);
        /**
         Its ranges, each enclosing range before anything inside it.
         */
        const ranges = script.functions
          .flatMap(function rangesOf(fn,): readonly CoverageRange[] {
            return fn.ranges;
          },)
          .toSorted(function outerFirst(
            left,
            right,
          ): number {
            return (left.startOffset - right.startOffset) || (right.endOffset - left.endOffset);
          },);
        for (const range of ranges) {
          /**
           First piece under the range.
           */
          const from = boundaryIndex({
            bundle: script.bundle,
            pieces: bundlePieces,
            offset: range.startOffset,
          },);
          /**
           Piece just past it.
           */
          const to = boundaryIndex({
            bundle: script.bundle,
            pieces: bundlePieces,
            offset: range.endOffset,
          },);
          painted.fill(
            range.count,
            from,
            to,
          );
          bundlePieces.covered
            .fill(
            1,
            from,
            to,
            );
        }
        for (const [piece, count,] of painted.entries())
          bundlePieces.total[piece] = nonNullishOrThrow(bundlePieces.total[piece],) + count;
        for (const fn of script.functions) {
          /**
           Its whole extent; `bundleScriptsOf` admits no function without one.
           */
          const [whole,] = fn.ranges;
          if (whole === undefined)
            throw new CoverageTallyError({ says: `${script.bundle} reports a function with no extent`, },);
          /**
           Where it sits.
           */
          const site: FunctionSite = {
            bundle: script.bundle,
            start: whole.startOffset,
            end: whole.endOffset,
            name: fn.functionName,
          };
          /**
           Its key.
           */
          const key = functionKey(site,);
          sites.set(
            key,
            site,
          );
          calls.set(
            key,
            (calls.get(key,) ?? 0) + whole.count,
          );
        }
      }
    },

    coldStretches: function coldStretches(): readonly ColdStretch[] {
      return [...pieces,]
        .toSorted(function byBundle(
          [left,],
          [right,],
        ): number {
          return compareCodePoints({
            left,
            right,
          },);
        },)
        .flatMap(function stretchesOfBundle([bundle, bundlePieces,],): readonly ColdStretch[] {
          return stretchesOf({
            bundle,
            pieces: bundlePieces,
            calls,
            sites,
          },);
        },);
    },

    uncalledFunctions: function uncalledFunctions(): readonly UncalledFunction[] {
      return markNested({
        sites: [...sites,]
          .filter(function uncalled([key,]: ReadonlyEntry<string, FunctionSite>,): boolean {
            return calls.get(key,) === 0;
          },)
          .map(function siteOf([, site,]: ReadonlyEntry<string, FunctionSite>,): FunctionSite {
            return site;
          },)
          .toSorted(function enclosingFirst(
            left,
            right,
          ): number {
            return compareCodePoints({
              left: left.bundle,
              right: right.bundle,
            },)
              || (left.start - right.start)
              || (right.end - left.end);
          },),
      },);
    },

    loadedBundles: function loadedBundles(): readonly string[] {
      return textsInCodePointOrder({ texts: [...boundaries.keys(),], },);
    },
  },);
}

//endregion Coverage tally
