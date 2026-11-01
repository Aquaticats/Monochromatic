//region Slice cache contract
// The shape a lane's resumable per-slice cache has, independent of what a lane
// stores in it.
//
// Generic because two lanes now cache slices and their values have nothing in
// common: a repair outcome carries issues, regions and checker verdicts, a
// translate record carries a slate, ballots and an alignment assessment. One
// union-valued map plus a cast after lookup is how a repair outcome ends up
// resumed into a translate run, which no reader downstream could detect: the
// artifact would record a single pipeline over internally mixed slices.
//
// Injected like the client, so a driver's result stays a function of its inputs
// and its resumed values. `persist` is a write-through side effect that never
// feeds back into the result.

/**
 Cross-run cache making a large document resumable.

 A run aborted at the hard cap resumes from the last settled slice instead of
 recomputing from scratch.

 @example
 ```ts
 const cache: SliceCache<string> = {
   resumed: new Map(),
   persist: async ({ key, serialized, },) => writeSliceFile({ key, serialized, },),
 };
 ```
 */
export type SliceCache<ValueT,> = {
  /**
   Values settled on an earlier run, keyed by slice hash; a hit skips every
   model call for that slice.
   */
  readonly resumed: ReadonlyMap<string, ValueT>;

  /**
   Persists one freshly settled slice under its hash key before the next slice
   starts, so an abort leaves settled slices recoverable.

   The lane owns serialization; the store writes exactly these bytes and
   parses them back into {@link SliceCache.resumed} next run.
   */
  readonly persist: (
    input: {
      readonly key: string;
      readonly serialized: string;
    },
  ) => Promise<void>;
};

//endregion Slice cache contract
