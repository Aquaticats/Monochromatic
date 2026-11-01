//region Runs of pieces
// Every short input of a few pieces, for the cases that hold a text reader to a
// property over all of them: a caller starts from the empty run and extends it
// a piece at a time, as far as it means to look.

/**
 Every run of one more piece than the runs it is given.

 @param runs - runs to extend, each by every piece in turn

 @param pieces - what a run is built of

 @returns The longer runs, in the order of the runs and, within one run, of the pieces

 @example
 ```ts
 const twos = longerRuns({ runs: ['a', 'b',], pieces: ['a', 'b',], },); // aa, ab, ba, bb
 ```
 */
export function longerRuns(
  {
    runs,
    pieces,
  }: {
    readonly runs: readonly string[];
    readonly pieces: readonly string[];
  },
): readonly string[] {
  return runs.flatMap(function extended(run,): readonly string[] {
    return pieces.map(function withPiece(piece,): string {
      return `${run}${piece}`;
    },);
  },);
}

//endregion Runs of pieces
