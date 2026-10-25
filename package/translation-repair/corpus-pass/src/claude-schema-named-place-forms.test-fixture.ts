import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import type { NamedPlace, } from './claude-schema-named-places.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region What a name of a place stands for
// A place the test names is one report of the check, found by file, shape, position and the function the report holds
// (the module that declares it and its name), and a name stands for one report. A report that differs from the name in
// any one of the four is not named, a second report that agrees with the name in all four is not named either, and a
// name that no report is left for is stale. The line of a report is no part of the name, so an edit above the place
// moves nothing. The cases here hold the comparison with reports of their own, so a change that stops comparing one
// part, or lets one name stand for two reports, fails a case; `claude-schema-named-flow-forms.test-fixture.ts` holds it
// with the reports the check makes over imaginary files.

/**
 The function the report and the name of every case hold, as the file that declares it and its name make the key.
 */
const HELD = keyOf({
  file: 'corpus-pass/src/stage.ts',
  name: 'mk',
},);

/**
 A report of a forwarder held as the value of a property, which the cases change one part of at a time.
 */
const SPOT: BlindSpot = {
  file: 'corpus-pass/src/a.ts',
  line: 10,
  shape: 'forwarder-outside-a-read-position',
  position: 'Property.value',
  held: HELD,
};

/**
 The name of the place that `SPOT` stands for.
 */
const PLACE: NamedPlace = {
  file: 'corpus-pass/src/a.ts',
  shape: 'forwarder-outside-a-read-position',
  position: 'Property.value',
  held: HELD,
  reason: 'a case',
};

/**
 The name of a place in another file that `SPOT` is no report of.
 */
const PLACE_IN_OTHER_FILE: NamedPlace = {
  ...PLACE,
  file: 'corpus-pass/src/b.ts',
};

/**
 The name of a place whose report holds no function: the same file and position as `PLACE`, and no function.
 */
const PLACE_WITHOUT_FUNCTION: NamedPlace = {
  file: PLACE.file,
  shape: PLACE.shape,
  position: PLACE.position,
  reason: PLACE.reason,
};

/**
 One case of the comparison between the reports and the names.
 */
export type NamedPlaceCase = {
  /**
   What the case holds.
   */
  readonly name: string;

  /**
   The reports of the check.
   */
  readonly spots: readonly BlindSpot[];

  /**
   The named places.
   */
  readonly places: readonly NamedPlace[];

  /**
   The reports no name stands for, as the comparison returns them.
   */
  readonly unnamed: readonly BlindSpot[];

  /**
   The names no report is left for, as the comparison returns them.
   */
  readonly stale: readonly NamedPlace[];
};

/**
 Cases of the comparison between reports and names.
 */
export const NAMED_PLACE_CASES: readonly NamedPlaceCase[] = [
  {
    name: 'NAMES THE REPORT THAT AGREES WITH THE NAME IN FILE, SHAPE, POSITION AND THE FUNCTION IT HOLDS',
    spots: [SPOT,],
    places: [PLACE,],
    unnamed: [],
    stale: [],
  },
  {
    name: 'KEEPS THE NAME WHEN THE REPORT MOVES TO ANOTHER LINE, since the line is no part of the name',
    spots: [
      {
        ...SPOT,
        line: 99,
      },
    ],
    places: [PLACE,],
    unnamed: [],
    stale: [],
  },
  {
    name: 'DOES NOT NAME A REPORT IN ANOTHER FILE, and leaves the name stale',
    spots: [
      {
        ...SPOT,
        file: 'corpus-pass/src/b.ts',
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        ...SPOT,
        file: 'corpus-pass/src/b.ts',
      },
    ],
    stale: [PLACE,],
  },
  {
    name: 'DOES NOT NAME A REPORT OF ANOTHER SHAPE, and leaves the name stale',
    spots: [
      {
        ...SPOT,
        shape: 'builder-outside-a-read-position',
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        ...SPOT,
        shape: 'builder-outside-a-read-position',
      },
    ],
    stale: [PLACE,],
  },
  {
    name: 'DOES NOT NAME A REPORT AT ANOTHER POSITION, and leaves the name stale',
    spots: [
      {
        ...SPOT,
        position: 'ArrayExpression.elements',
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        ...SPOT,
        position: 'ArrayExpression.elements',
      },
    ],
    stale: [PLACE,],
  },
  {
    name: 'DOES NOT NAME A REPORT OF ANOTHER FUNCTION, and leaves the name stale',
    spots: [
      {
        ...SPOT,
        held: keyOf({
          file: 'corpus-pass/src/stage.ts',
          name: 'other',
        },),
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        ...SPOT,
        held: keyOf({
          file: 'corpus-pass/src/stage.ts',
          name: 'other',
        },),
      },
    ],
    stale: [PLACE,],
  },
  {
    name: 'DOES NOT NAME A REPORT OF A FUNCTION OF THE SAME NAME THAT ANOTHER MODULE DECLARES, and leaves the name '
      + 'stale',
    spots: [
      {
        ...SPOT,
        held: keyOf({
          file: 'corpus-pass/src/other-stage.ts',
          name: 'mk',
        },),
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        ...SPOT,
        held: keyOf({
          file: 'corpus-pass/src/other-stage.ts',
          name: 'mk',
        },),
      },
    ],
    stale: [PLACE,],
  },
  {
    name: 'DOES NOT NAME A REPORT OF A FREE NAME THAT NO MODULE DECLARES, whatever name it has, and leaves the name '
      + 'stale',
    spots: [
      {
        ...SPOT,
        held: 'mk',
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        ...SPOT,
        held: 'mk',
      },
    ],
    stale: [PLACE,],
  },
  {
    name: 'DOES NOT NAME A REPORT THAT HOLDS A FUNCTION BY A NAME THAT GIVES NONE, and leaves the name stale',
    spots: [SPOT,],
    places: [PLACE_WITHOUT_FUNCTION,],
    unnamed: [SPOT,],
    stale: [PLACE_WITHOUT_FUNCTION,],
  },
  {
    name: 'NAMES A REPORT THAT HOLDS NO FUNCTION BY A NAME THAT GIVES NONE',
    spots: [
      {
        file: SPOT.file,
        line: SPOT.line,
        shape: 'builder-outside-a-read-position',
        position: SPOT.position,
      },
    ],
    places: [
      {
        file: PLACE.file,
        shape: 'builder-outside-a-read-position',
        position: PLACE.position,
        reason: PLACE.reason,
      },
    ],
    unnamed: [],
    stale: [],
  },
  {
    name: 'DOES NOT NAME A REPORT THAT HOLDS NO FUNCTION BY A NAME THAT GIVES ONE, and leaves the name stale',
    spots: [
      {
        file: PLACE.file,
        line: SPOT.line,
        shape: PLACE.shape,
        position: PLACE.position,
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        file: PLACE.file,
        line: SPOT.line,
        shape: PLACE.shape,
        position: PLACE.position,
      },
    ],
    stale: [PLACE,],
  },
  {
    name: 'MATCHES A REPORT WITH THE NAME THAT STANDS FOR IT AND NOT WITH THE FIRST NAME OF THE LIST',
    spots: [SPOT,],
    places: [
      PLACE_IN_OTHER_FILE,
      PLACE,
    ],
    unnamed: [],
    stale: [PLACE_IN_OTHER_FILE,],
  },
  {
    name: 'NAMES ONE REPORT AND LEAVES A SECOND REPORT THAT AGREES WITH THE NAME IN ALL FOUR PARTS',
    spots: [
      SPOT,
      {
        ...SPOT,
        line: 11,
      },
    ],
    places: [PLACE,],
    unnamed: [
      {
        ...SPOT,
        line: 11,
      },
    ],
    stale: [],
  },
  {
    name: 'LEAVES A NAME STALE WHEN TWO NAMES STAND FOR ONE REPORT',
    spots: [SPOT,],
    places: [
      PLACE,
      PLACE,
    ],
    unnamed: [],
    stale: [PLACE,],
  },
];

//endregion What a name of a place stands for
