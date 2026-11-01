import { readingsCorroborate, } from './reading-corroboration.ts';

//region Reading cluster
// MEASURED ON XINGZ601 (2026-09-17): five readers were seated, three read the
// score screenshot, and the corroboration compared only the first two, a rule
// written when the vision sub-roster was exactly two models. Those two stood
// at overlap 0.297, just under the line; the third reading stood at 0.586
// and 0.326 against them. The entry stopped at the pictures phase after seven
// minutes on a picture three readers agreed about. Corroboration is now
// taken over every pair: a reading vouched for by any other reading may be
// used, and the picture is corroborated when at least one pair agrees.

/**
 One transcript a reader produced, as much of it as clustering needs.
 */
type Transcript = {
  readonly text: string;
};

/**
 Two transcripts and how far they agree.

 @example
 ```ts
 const pair: TranscriptPair<Transcript> = { left, right, leftIndex: 0, rightIndex: 2, overlap: 0.42, corroborated: true, };
 ```
 */
export type TranscriptPair<ReadingT extends Transcript,> = {
  /**
   Earlier of the two in roster order.
   */
  readonly left: ReadingT;

  /**
   Later of the two.
   */
  readonly right: ReadingT;

  /**
   Position of the earlier reading in roster order.
   */
  readonly leftIndex: number;

  /**
   Position of the later reading.
   */
  readonly rightIndex: number;

  /**
   Share of the smaller reading's trigrams the larger carried.
   */
  readonly overlap: number;

  /**
   Whether the two describe the same picture by the corroboration threshold.
   */
  readonly corroborated: boolean;
};

/**
 Which readings vouch for each other.

 @example
 ```ts
 const cluster: ReadingCluster<Transcript> = { vouched: [], closest, };
 ```
 */
export type ReadingCluster<ReadingT extends Transcript,> = {
  /**
   Readings corroborated by at least one other reading, in roster order;
   fewer than two when no pair agrees.
   */
  readonly vouched: readonly ReadingT[];

  /**
   Pair that agreed most, whatever side of the threshold it fell on, so a
   refusal can name how close the readers came.
   */
  readonly closest: TranscriptPair<ReadingT>;
};

/**
 Raised when clustering is asked about fewer than two readings, which have
 no pair to compare.

 @example
 ```ts
 throw new ReadingClusterError('one reading has no pair',);
 ```
 */
export class ReadingClusterError extends Error {
  /**
   Distinguishes this from other errors after serialization.
   */
  public override readonly name = 'ReadingClusterError';
}

/**
 Corroborates readings over every pair rather than the first two.

 @param readings - what each reader transcribed, in roster order, two at least

 @returns Readings some other reading vouches for, and the closest pair

 @throws {@link ReadingClusterError} when handed fewer than two readings

 @example
 ```ts
 const cluster = clusterReadings({ readings, },);
 ```
 */
export function clusterReadings<ReadingT extends Transcript,>(
  { readings, }: { readonly readings: readonly ReadingT[]; },
): ReadingCluster<ReadingT> {
  /**
   Every pair in roster order, each with its agreement measured once.
   */
  const pairs: readonly TranscriptPair<ReadingT>[] = readings.flatMap(function pairsFrom(
    left,
    leftIndex,
  ): readonly TranscriptPair<ReadingT>[] {
    return readings.slice(leftIndex + 1,)
      .map(function pairWith(
        right,
        offset,
      ): TranscriptPair<ReadingT> {
        /**
         Whether these two describe the same picture.
         */
        const verdict = readingsCorroborate({
          left: left.text,
          right: right.text,
        },);
        return {
          left,
          right,
          leftIndex,
          rightIndex: leftIndex
            + 1
            + offset,
          overlap: verdict.overlap,
          corroborated: verdict.kind === 'corroborated',
        };
      },);
  },);
  /**
   First pair, whose absence means there was nothing to compare.
   */
  const first = pairs.at(0,);
  if (first === undefined)
    throw new ReadingClusterError(`${String(readings.length,)} reading(s) have no pair to compare`,);

  /**
   Positions of every reading some pair corroborated.
   */
  const vouchedIndices = new Set(pairs
    .filter(function agreed(pair,): boolean {
      return pair.corroborated;
    },)
    .flatMap(function bothSides(pair,): readonly number[] {
      return [
        pair.leftIndex,
        pair.rightIndex,
      ];
    },),);
  return {
    vouched: readings.filter(function isVouched(
      _reading,
      index,
    ): boolean {
      return vouchedIndices.has(index,);
    },),
    closest: pairs.reduce(
      function closerPair(
        best: TranscriptPair<ReadingT>,
        pair: TranscriptPair<ReadingT>,
      ): TranscriptPair<ReadingT> {
        return (pair.overlap > best.overlap) ? pair : best;
      },
      first,
    ),
  };
}

//endregion Reading cluster
