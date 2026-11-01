//region Digest group
// Grouping artifact readings by the built output that produced them.
//
// WHY GROUPING IS NOT A DETAIL. `artifact-pool.ts` refuses to pool results
// whose `pipelineDigest` differs, because two builds are two configurations and
// a figure summed across them describes neither. Any report over an archive
// inherits that rule, and the only way to honour it is to never form a
// collection that spans digests in the first place.

/**
 Anything carrying the built output that produced it.
 */
type DigestBearing = {
  /**
   Built output recorded on the artifact this was read from.
   */
  readonly digest: string;
};

/**
 Readings that share one built output.
 */
export type DigestGroup<TReading,> = {
  /**
   Built output all of them record.
   */
  readonly digest: string;

  /**
   Readings under it, in the order they were read.
   */
  readonly readings: readonly TReading[];
};

/**
 Splits readings into one group per built output, largest group first.

 ORDER WITHIN A GROUP IS THE READING ORDER, so a caller that read an archive
 in a stable order gets a stable report.

 @param readings - everything read, in any digest order

 @returns One group per digest, most readings first

 @example
 ```ts
 const groups = groupByDigest({ readings, },);
 ```

 @internal
 */
export function groupByDigest<const TReading extends DigestBearing,>(
  { readings, }: { readonly readings: readonly TReading[]; },
): readonly DigestGroup<TReading>[] {
  /**
   Digests seen, each once, in first-seen order.
   */
  const digests = [
    ...new Set(readings.map(function toDigest(reading,): string {
      return reading.digest;
    },),),
  ];

  return digests
    .map(function toGroup(digest,): DigestGroup<TReading> {
      return {
        digest,
        readings: readings.filter(function under(reading,): boolean {
          return reading.digest === digest;
        },),
      };
    },)
    .toSorted(function byCount(
      left,
      right,
    ): number {
      return right.readings
        .length
        - left.readings
          .length;
    },);
}

//endregion Digest group
