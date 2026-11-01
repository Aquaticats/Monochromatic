import {
  CORPUS_EXPANSION,
  isInsertionChunk,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { codePointCount, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Coverage tail
// THE UNTRANSLATED TAIL IS BUDGETED FROM THE PAIRING (owner, 2026-09-19).
// An archive that stops before the source does leaves every source-only
// slice after the pairing's last agreed pair with nothing standing where it
// would go. The whole-page budget read that run against the corpus median
// less whatever the translated part carried, and on an archive that ran long
// (XingZ60: about 3.3 English code points per source code point against the
// corpus median of 2.65) it came up 80 code points short of the footnote
// definitions (XingZ608). The tail's expectation is read off the pairing
// instead: its source at the page's own paired expansion.
//
// THE SECOND SIGNATURE STAYS DETERMINISTIC. The pairing is a roster's work,
// so "nothing after the last pair" is not the model-free reading the
// absence-verdict decision requires. What is model-free is size: a tail
// whose expectation exceeds the last agreed pair's whole rendering cannot
// have been merged into it, which is the one place a tail could hide. Such
// a tail is admitted on that bound; a smaller tail stays with the whole-page
// budget, where it was before.
//
// MEASURED BEFORE BUILDING (2026-09-19, every run that left an artifact and
// a log): a window of one paired neighbour each side, read at the page's own
// ratio, refused real omissions on hakureico, xiept2 and shi_Yumiaoya because
// one paragraph's expansion is noisy by more than a small passage's size.
// The tail is the region the pairing makes exact.

/**
 Source-only slices standing after the pairing's last agreed pair.

 @example
 ```ts
 const tail = readUntranslatedTail({ slices, },);
 ```
 */
export type UntranslatedTail = {
  /**
   Positions of the tail's slices in prepared order; empty when the last
   slice is paired or nothing was ever paired.
   */
  readonly positions: ReadonlySet<number>;

  /**
   Source code points the tail holds.
   */
  readonly sourceCodePoints: number;

  /**
   English code points per source code point over the page's paired slices,
   or the corpus median when nothing was paired.
   */
  readonly expansion: number;

  /**
   English the tail is expected to render into.
   */
  readonly expected: number;

  /**
   English the last agreed pair's rendering holds, the one place the tail
   could have been merged into.
   */
  readonly lastPairTargetCodePoints: number;

  /**
   Whether the tail's expectation exceeds the last agreed pair's whole
   rendering, so it cannot have been merged there and is admitted on that
   bound.
   */
  readonly exceedsLastPair: boolean;
};

/**
 What `findLastIndex` answers when no slice matches.
 */
const NOT_FOUND = -1;

/**
 Running sums over the paired slices.
 */
type PairedSums = {
  /**
   Source code points of every paired slice.
   */
  readonly source: number;

  /**
   Target code points of every paired slice.
   */
  readonly target: number;
};

/**
 Whether a slice's archive side is content.

 @param slice - prepared slice

 @returns True when the archive translated it

 @example
 ```ts
 const lastPaired = slices.findLastIndex(isPaired);
 ```
 */
function isPaired(slice: ChunkPair,): boolean {
  return !isInsertionChunk(slice.target,);
}

/**
 Source code points of a slice.

 @param slice - prepared slice

 @returns Size of its original side

 @example
 ```ts
 const size = sourceSize(slice);
 ```
 */
function sourceSize(slice: ChunkPair,): number {
  /**
   Original side of the slice.
   */
  const { source, } = slice;
  return codePointCount({ text: source.text, },);
}

/**
 Target code points of a slice, none for an insertion anchor.

 @param slice - prepared slice

 @returns Size of its archive side

 @example
 ```ts
 const size = targetSize(slice);
 ```
 */
function targetSize(slice: ChunkPair,): number {
  /**
   Archive side of the slice.
   */
  const { target, } = slice;
  // AN INSERTION SIZES ZERO WITHOUT A GUARD: every caller has already read
  // isPaired, whose contract excludes insertion targets, and
  // makeInsertionChunk keeps the text empty anyway (ledger T8).
  return codePointCount({ text: target.text, },);
}

/**
 Reads the untranslated tail off the pairing.

 @param slices - prepared slices in document order

 @returns Tail positions with their expectation at the page's own expansion

 @example
 ```ts
 const tail = readUntranslatedTail({ slices, },);
 ```
 */
export function readUntranslatedTail(
  { slices, }: { readonly slices: readonly ChunkPair[]; },
): UntranslatedTail {
  /**
   Position of the last slice the archive translated.
   */
  const lastPaired = slices.findLastIndex(isPaired,);
  /**
   Whether the archive translated nothing at all.
   */
  const nothingPaired = lastPaired === NOT_FOUND;
  /**
   Slices after the last agreed pair, none when nothing was paired.
   */
  const tailSlices = nothingPaired
    ? []
    : slices.slice((lastPaired + 1),);
  /**
   Positions of those slices in prepared order.
   */
  const positions = new Set(tailSlices.map(function toPosition(
    _slice,
    offset,
  ): number {
    return lastPaired
      + 1
      + offset;
  },),);
  /**
   Source and target sizes over the paired slices.
   */
  const paired = slices
    .filter(isPaired,)
    .reduce(
    function sumPaired(
      sums: PairedSums,
      slice,
    ): PairedSums {
      return {
        source: sums.source + sourceSize(slice,),
        target: sums.target + targetSize(slice,),
      };
    },
    {
      source: 0,
      target: 0,
    },
    );
  /**
   The page's own expansion, or the corpus median when nothing was paired.
   */
  const expansion = (paired.source === 0)
    ? CORPUS_EXPANSION
    : paired.target / paired.source;
  /**
   Source code points the tail holds.
   */
  const sourceCodePoints = tailSlices
    .map(sourceSize,)
    .reduce(
    function sum(
      total,
      size,
    ): number {
      return total + size;
    },
    0,
    );
  /**
   Last agreed pair, absent when nothing was paired.
   */
  const lastSlice = slices[lastPaired];
  /**
   Size of its rendering.
   */
  const lastPairTargetCodePoints = (lastSlice === undefined)
    ? 0
    : targetSize(lastSlice,);
  /**
   English the tail is expected to render into.
   */
  const expected = sourceCodePoints * expansion;
  return {
    positions,
    sourceCodePoints,
    expansion,
    expected,
    lastPairTargetCodePoints,
    exceedsLastPair: (positions.size > 0) && (expected > lastPairTargetCodePoints),
  };
}

/**
 English the translated part of a page is missing against what its source
 predicts, with the source of a tail admitted on its own bound taken out.

 @param sourceText - whole original page

 @param targetText - whole translation as it stands

 @param tail - untranslated tail read off the pairing

 @returns Code points of English the interior lacks, zero when it lacks none

 @example
 ```ts
 const shortfall = interiorShortfall({ sourceText, targetText, tail, },);
 ```
 */
export function interiorShortfall(
  {
    sourceText,
    targetText,
    tail,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly tail: UntranslatedTail;
  },
): number {
  /**
   Source code points the translated part answers for.
   */
  const interiorSource = codePointCount({ text: sourceText, },)
    - (tail.exceedsLastPair ? tail.sourceCodePoints : 0);
  return Math.max(
    0,
    (interiorSource * CORPUS_EXPANSION) - codePointCount({ text: targetText, },),
  );
}

//endregion Coverage tail
