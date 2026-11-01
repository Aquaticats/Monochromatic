/**
 Bounded line sink and pair-stack types shared by every difference walk
 module ({@link ./value-walk.ts},
 {@link ./value-walk-collections.ts}).

 Split out so the walk modules share plumbing without importing each
 other.

 @module
 */

//region Difference sink

/**
 Collector bounding how many difference lines render,
 counting the rest so the overflow marker can name them.
 */
export type DifferenceSink = {
  /**
   Lines rendered so far, up to the cap.
   */
  readonly lines: string[];
  /**
   Lines counted past the cap, named by the overflow marker.
   */
  suppressed: number;
  /**
   Cap on rendered lines.
   */
  readonly max: number;
};

/**
 Records one difference line,
 rendering it while under the cap and counting it past the cap so output
 never grows with a pathological input.

 @param sink - collector for this difference computation

 @param line - rendered difference line

 @mutates sink - Appends the line or increments the suppressed count.

 @example
 ```ts
 pushLine({ sink: { lines: [], suppressed: 0, max: 2, }, line: 'only in actual: 1', },);
 ```
 */
export function pushLine({
  sink,
  line,
}: {
  readonly sink: DifferenceSink;
  readonly line: string;
},): void {
  if (sink.lines
    .length
    < sink.max)
    sink.lines
      .push(line,);
  else
    sink.suppressed += 1;
}

//endregion Difference sink

/**
 Compared container pairs currently above this walk,
 ancestor positions only,
 so a self-referential value stops on revisit while a value shared by two
 sibling keys still reports under both.
 */
export type Ancestors = readonly (readonly [
  object,
  unknown
])[];
