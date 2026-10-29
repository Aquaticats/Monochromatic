/**
 Tests for reporting the coverage census's refusals instead of crashing out of a CLI.

 THESE CASES LIVE APART FROM `cli-refusal.unit.test.ts` because the names they
 report, `readBaselineFile` and `SourceMapFileError`, belong to a package above
 the one that owns `reportingRefusals`. A test file may import only from its own
 package and from the packages that package depends on, so a case that needs the
 census's reader or class cannot sit in the file of the package below it. The
 baseline case moved whole, and the table-driven case every operator's own
 mistake gets was split by its list of classes.

 BOTH SWAPS ARE DISPOSABLE, and the suite runs at `concurrency: 1` for the
 same reason as in `cli-refusal.unit.test.ts`. `process.exitCode` and
 `console.error` are process-wide, and `describe` runs children concurrently by
 default, so cases that swapped them would race. There is exactly one process
 here: the runner spawns `node` once per test FILE, so nothing outside this file
 is touched, and nothing inside it may overlap.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportingRefusals, } from '@monochromatic-dev/translation-repair-run';
import {
  readBaselineFile,
  SourceMapFileError,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  BARE_ARGV,
  caseName,
  collectingErrors,
  holdingExitCode,
  NO_KEYS,
  REFUSED_AS_STATED,
  reportsMistake,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/cli-refusal-cases.test-fixture.ts';

//region Coverage census refusal tests

/**
 Refusals an operator's own mistake raises that belong to the coverage census.
 */
const CENSUS_MISTAKES: readonly Error[] = [
  new SourceMapFileError({
    path: 'dist/final/node/index.mjs.map',
    says: 'it has no mappings',
  },),
];

await describe({
  name: reportingRefusals.name,
  children: [
    it({
      name: 'REPEATS a baseline the operator named wrongly in its own words, at the stated-refusal code and with no '
        + 'fault line or frames, where the census command reads it',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);
        await using scratch = await scratchDir({ prefix: 'cli-refusal-baseline-', },);
        /**
         A path nothing stands at, as a baseline named with a typo would be.
         */
        const path = join(
          scratch.path,
          'purr.json',
        );

        await reportingRefusals({
          what: 'coverage-census',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            await readBaselineFile({ path, },);
          },
        },);

        expect(process.exitCode,).toBe(REFUSED_AS_STATED,);
        expect(printed.lines,).toEqual([
          `coverage-census: baseline ${path} does not read as a census this command wrote: it could not be read `
            + '(ENOENT)',
        ],);
      },
    },),
    ...CENSUS_MISTAKES.map(function operatorMistakeCase(refusal,) {
      return it({
        name: caseName({ refusal, },),
        fn: () => reportsMistake({ refusal, },),
      },);
    },),
  ],
  concurrency: 1,
},);

//endregion Coverage census refusal tests
