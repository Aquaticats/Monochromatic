import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  type FunctionKey,
  keyOf,
} from './claude-schema-usage.test-fixture.ts';

//region Places in the family's source that the check reports and the test names
// The check behind the schema search reports every place it does not understand, and a place it reports is answered
// by reading the place in the search or by naming the place in the test. A place is named by file, shape, position and,
// for a forwarder held in a place that is no call, the module that declares the forwarder and its name, not by line,
// so an edit above it moves nothing. A name stands for one report, so a second report that agrees with it in every
// part is not named, and each named place must still be reported, so a name never outlives the place. A name that
// stood for a report by file, shape and position alone also stood for a second forwarder handed over beside the first,
// and for the same forwarder handed over again under a second key; a name that held only the name of the forwarder
// also stood for a forwarder of the same name that another module declares and the import was redirected to. A
// forwarder that no module declares, a free name, has no module, so its report holds the bare name, which no key
// equals, and no name stands for it.

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
   The function a report of the place holds, where the shape holds one: the key of the forwarder the place hands over,
   the module that declares it and its name. The type of a key refuses a bare name, which stands for a function no
   module declares and which no name stands for.
   */
  readonly held?: FunctionKey;

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
    held: keyOf({
      file: 'corpus-pass/src/corpus-run/pass-entry-settle.ts',
      name: 'settleEntry',
    },),
    reason: '`settleEntry` is handed to the pass as its `settle` dependency, and `corpus-pass-run.ts` calls '
      + '`input.settle({ client, entry, ... })`. The arguments there are a corpus pair and the settings of the run, '
      + 'not the arguments of a schema, which `settleEntry` makes inside from the pair; the wide reading of a '
      + 'forwarder lists such a function whatever the arguments are.',
  },
];

/**
 Whether a report is the place a name stands for.

 @param spot - report

 @param place - named place

 @returns Whether the file, the shape, the position and the function the report holds are the same

 @example
 ```ts
 const isNamed = NAMED_PLACES.some(function names(place,): boolean { return isPlaceOf({ spot, place, },); },);
 ```
 */
function isPlaceOf(
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
  /**
   Whether the report is in the file the name gives and is of the shape and the position it gives.
   */
  const isSamePlace = (spot.file === place.file) && isSameShape;
  return isSamePlace && (spot.held === place.held);
}

/**
 What the reports and the places left are when the places are matched with the reports one for one.
 */
type Matching = {
  /**
   The reports no place was matched with.
   */
  readonly unnamed: readonly BlindSpot[];

  /**
   The places no report was matched with.
   */
  readonly stale: readonly NamedPlace[];
};

/**
 Matches each report with the first place still free that stands for it, so a place stands for one report.

 @param spots - reports of the check, in order

 @param places - named places

 @returns Reports and places that were not matched

 @example
 ```ts
 const { unnamed, stale, } = matchOf({ spots, places: NAMED_PLACES, },);
 ```
 */
function matchOf(
  {
    spots,
    places,
  }: {
    readonly spots: readonly BlindSpot[];
    readonly places: readonly NamedPlace[];
  },
): Matching {
  return spots.reduce(
    function matchSpot(
      {
        unnamed,
        stale,
      }: Matching,
      spot,
    ): Matching {
      /**
       The first place still free that stands for the report, absent when none does.
       */
      const standing = stale.find(function standsFor(place,): boolean {
        return isPlaceOf({
          spot,
          place,
        },);
      },);
      return (standing === undefined)
        ? {
          unnamed: [
            ...unnamed,
            spot,
          ],
          stale,
        }
        : {
          unnamed,
          stale: stale.toSpliced(
            stale.indexOf(standing,),
            1,
          ),
        };
    },
    {
      unnamed: [],
      stale: places,
    },
  );
}

/**
 The reports that no named place stands for, a place standing for one report.

 @param spots - reports of the check

 @param places - named places

 @returns Reports left, in order

 @example
 ```ts
 const unnamed = spotsNotNamed({ spots, places: NAMED_PLACES, },);
 ```
 */
export function spotsNotNamed(
  {
    spots,
    places,
  }: {
    readonly spots: readonly BlindSpot[];
    readonly places: readonly NamedPlace[];
  },
): readonly BlindSpot[] {
  /**
   The reports no place was matched with.
   */
  const { unnamed, } = matchOf({
    spots,
    places,
  },);
  return unnamed;
}

/**
 The named places that no report stands behind, a report standing behind one place.

 @param spots - reports of the check

 @param places - named places

 @returns Places left, in order

 @example
 ```ts
 const stale = placesWithoutSpot({ spots, places: NAMED_PLACES, },);
 ```
 */
export function placesWithoutSpot(
  {
    spots,
    places,
  }: {
    readonly spots: readonly BlindSpot[];
    readonly places: readonly NamedPlace[];
  },
): readonly NamedPlace[] {
  /**
   The places no report was matched with.
   */
  const { stale, } = matchOf({
    spots,
    places,
  },);
  return stale;
}

//endregion Places in the family's source that the check reports and the test names
