/**
 Unit tests for semver range extraction and precedence comparison.
 
 Pins semver clause 11 ordering, which decides whether a catalog floor is
 rewritten: a comparison that answers "installed is greater" when it is not
 lowers the floor, and one that answers "not greater" when it is misses the
 tightening. Numeric prerelease identifiers (`rc.9` versus `rc.29`) and partial
 floors (`4.0`) are the shapes a whole-string or `NaN` comparison gets wrong.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isStrictlyGreater,
  NOT_A_RANGE,
  type ParsedRange,
  parseRange,
} from './version-parse.ts';

//region Fixtures

/**
 One catalog floor against one installed version, with the precedence answer
 semver clause 11 gives.
 */
type PrecedenceCase = {
  /**
   What the pair demonstrates, used as the test name.
   */
  readonly name: string;
  /**
   Version taken from the catalog `>=` range.
   */
  readonly cataloged: string;
  /**
   Version read from the install layout.
   */
  readonly installed: string;
  /**
   Whether the installed version is strictly newer, so the floor is raised.
   */
  readonly expected: boolean;
};

/**
 Precedence pairs covering each clause of the semver ordering.
 */
const PRECEDENCE_CASES: readonly PrecedenceCase[] = [
  {
    name: 'raises the floor for a greater minor release',
    cataloged: '1.2.0',
    installed: '1.3.0',
    expected: true,
  },
  {
    name: 'leaves an identical version alone',
    cataloged: '1.2.0',
    installed: '1.2.0',
    expected: false,
  },
  {
    name: 'leaves the floor alone when the installed version is older',
    cataloged: '1.3.0',
    installed: '1.2.0',
    expected: false,
  },
  {
    name: 'compares the patch of an older release against a newer prerelease',
    cataloged: '1.2.3',
    installed: '1.2.4-0',
    expected: true,
  },
  {
    name: 'compares dot-separated prerelease identifiers one by one',
    cataloged: '7.0.0-dev.1',
    installed: '7.0.0-dev.2',
    expected: true,
  },
  {
    name: 'does not raise the floor when the installed prerelease identifier is numerically lower',
    cataloged: '2.0.1-rc.29',
    installed: '2.0.1-rc.9',
    expected: false,
  },
  {
    name: 'raises the floor when the installed prerelease identifier is numerically higher',
    cataloged: '2.0.1-rc.9',
    installed: '2.0.1-rc.29',
    expected: true,
  },
  {
    name: 'compares the workspace catalog prerelease identifier numerically',
    cataloged: '3.0.0-2',
    installed: '3.0.0-10',
    expected: true,
  },
  {
    name: 'does not lower the floor from a higher prerelease identifier',
    cataloged: '3.0.0-10',
    installed: '3.0.0-2',
    expected: false,
  },
  {
    name: 'ranks a numeric identifier below an alphanumeric one',
    cataloged: '1.0.0-1',
    installed: '1.0.0-alpha',
    expected: true,
  },
  {
    name: 'does not lift an alphanumeric identifier above a numeric one',
    cataloged: '1.0.0-alpha',
    installed: '1.0.0-1',
    expected: false,
  },
  {
    name: 'ranks a longer identifier set above its own prefix',
    cataloged: '1.0.0-alpha',
    installed: '1.0.0-alpha.1',
    expected: true,
  },
  {
    name: 'does not rank an identifier prefix above the longer set',
    cataloged: '1.0.0-alpha.1',
    installed: '1.0.0-alpha',
    expected: false,
  },
  {
    name: 'orders alphanumeric identifiers in ASCII order',
    cataloged: '1.0.0-beta',
    installed: '1.0.0-rc',
    expected: true,
  },
  {
    name: 'ranks a release above a prerelease of the same core',
    cataloged: '1.0.0-rc.1',
    installed: '1.0.0',
    expected: true,
  },
  {
    name: 'does not rank a prerelease above the release of the same core',
    cataloged: '1.0.0',
    installed: '1.0.0-rc.1',
    expected: false,
  },
  {
    name: 'reads a partial floor as the full core with zero fillers',
    cataloged: '4.0',
    installed: '4.0.2',
    expected: true,
  },
  {
    name: 'leaves a partial floor alone when the zero-filled core matches',
    cataloged: '4.0',
    installed: '4.0.0',
    expected: false,
  },
  {
    name: 'reads a major-only floor as a zero-filled full core',
    cataloged: '4',
    installed: '4.1.0',
    expected: true,
  },
  {
    name: 'ignores build metadata on both sides',
    cataloged: '1.0.0+build.1',
    installed: '1.0.0+build.2',
    expected: false,
  },
  {
    name: 'ignores build metadata the floor does not carry',
    cataloged: '1.0.0',
    installed: '1.0.0+build.7',
    expected: false,
  },
  {
    name: 'compares the prerelease while ignoring each side build metadata',
    cataloged: '1.0.0-rc.1+build.1',
    installed: '1.0.0-rc.2+build.0',
    expected: true,
  },
  {
    name: 'compares zero-padded numeric identifiers by value',
    cataloged: '3.0.0-02',
    installed: '3.0.0-10',
    expected: true,
  },
  {
    name: 'leaves a floor with unparseable version text alone',
    cataloged: 'latest',
    installed: '1.0.0',
    expected: false,
  },
  {
    name: 'leaves the floor alone when the installed version is unparseable',
    cataloged: '1.0.0',
    installed: 'not-a-version',
    expected: false,
  },
  {
    name: 'leaves the floor alone when the catalog text carries a fourth core segment',
    cataloged: '1.2.3.4',
    installed: '1.2.5',
    expected: false,
  },
  {
    name: 'leaves the floor alone when the catalog text carries an empty core segment',
    cataloged: '1.2.',
    installed: '1.2.5',
    expected: false,
  },
];

/**
 One catalog value that is a `>=` range and the range it yields.
 */
type ParsedRangeCase = {
  /**
   What the parser case demonstrates, used as the test name.
   */
  readonly name: string;
  /**
   Raw catalog value fed to {@link parseRange}.
   */
  readonly value: string;
  /**
   Prefix and version the value parses into.
   */
  readonly expected: ParsedRange;
};

/**
 Catalog values that are `>=` ranges.
 */
const PARSED_RANGE_CASES: readonly ParsedRangeCase[] = [
  {
    name: 'extracts a plain floor',
    value: '>=1.2.3',
    expected: {
      prefix: '',
      version: '1.2.3',
    },
  },
  {
    name: 'keeps the npm alias prefix before the floor',
    value: 'npm:@jsr/zod__zod@>=4.1.8',
    expected: {
      prefix: 'npm:@jsr/zod__zod@',
      version: '4.1.8',
    },
  },
];

/**
 One catalog value that is not a `>=` range.
 */
type NonRangeCase = {
  /**
   What the rejection case demonstrates, used as the test name.
   */
  readonly name: string;
  /**
   Raw catalog value fed to {@link parseRange}.
   */
  readonly value: string;
};

/**
 Catalog values the caller must treat as opaque.
 */
const NON_RANGE_CASES: readonly NonRangeCase[] = [
  {
    name: 'rejects a wildcard',
    value: '*',
  },
  {
    name: 'rejects a caret range',
    value: '^1.2.3',
  },
  {
    name: 'rejects an exact version',
    value: '1.2.3',
  },
  {
    name: 'rejects a bare floor token',
    value: '>=',
  },
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: parseRange.name,
      children: [
        ...PARSED_RANGE_CASES.map(function mapParsedRangeCase(rangeCase,) {
          return it({
            name: rangeCase.name,
            fn: async () => {
              expect(parseRange(rangeCase.value,),).toEqual(rangeCase.expected,);
            },
          },);
        },),
        ...NON_RANGE_CASES.map(function mapNonRangeCase(rangeCase,) {
          return it({
            name: rangeCase.name,
            fn: async () => {
              expect(parseRange(rangeCase.value,),).toBe(NOT_A_RANGE,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: isStrictlyGreater.name,
      children: PRECEDENCE_CASES.map(function mapPrecedenceCase(precedenceCase,) {
        return it({
          name: precedenceCase.name,
          fn: async () => {
            expect(isStrictlyGreater({
              cataloged: precedenceCase.cataloged,
              installed: precedenceCase.installed,
            },),).toBe(precedenceCase.expected,);
          },
        },);
      },),
    },),
  ],
},);
