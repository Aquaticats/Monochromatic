import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';

//region Places in the family's source that the check reports and the test names
// The check behind the schema search reports every place it does not understand, and a place it reports is answered
// by reading the place in the search or by naming the place in the test. A place is named by file, shape and position,
// not by line, so an edit above it moves nothing, and each named place must still be reported, so a name never
// outlives the place.

/**
 One place the test names, with the reason it is not read.
 */
export type NamedPlace = {
  /**
   Path of the file under `package/translation-repair`.
   */
  readonly file: string;

  /**
   The shape the check reports.
   */
  readonly shape: BlindSpot['shape'];

  /**
   Where the place sits, as a report gives it.
   */
  readonly position: string;

  /**
   Why the place is not a way for a schema to reach a send unlisted.
   */
  readonly reason: string;
};

/**
 The places in the family's source that the check reports and the test names.
 */
export const NAMED_PLACES: readonly NamedPlace[] = [
  {
    file: 'corpus-pass/src/corpus-run/corpus-pass.ts',
    shape: 'forwarder-outside-a-read-position',
    position: 'Property.value',
    reason: '`settleEntry` is handed to the pass as its `settle` dependency, and `corpus-pass-run.ts` calls '
      + '`input.settle({ client, entry, ... })`. The arguments there are a corpus pair and the settings of the run, '
      + 'not the arguments of a schema, which `settleEntry` makes inside from the pair; the wide reading of a forwarder '
      + 'lists such a function whatever the arguments are.',
  },
];

/**
 Whether a report is the place a name stands for.

 @param spot - report

 @param place - named place

 @returns Whether the file, the shape and the position are the same

 @example
 ```ts
 const isNamed = NAMED_PLACES.some(function names(place,): boolean { return isPlaceOf({ spot, place, },); },);
 ```
 */
export function isPlaceOf(
  {
    spot,
    place,
  }: {
    readonly spot: BlindSpot;
    readonly place: NamedPlace;
  },
): boolean {
  /**
   Whether the report is of the shape the name gives, at the place it gives.
   */
  const isSameShape = (spot.shape === place.shape) && (spot.position === place.position);
  return (spot.file === place.file) && isSameShape;
}

//endregion Places in the family's source that the check reports and the test names
