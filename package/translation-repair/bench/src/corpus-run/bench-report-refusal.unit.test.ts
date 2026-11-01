/**
 Tests for reporting the bench report's refusal instead of crashing out of a CLI.

 THESE CASES LIVE APART FROM `cli-refusal.unit.test.ts` because the class they
 report, `BenchReportError`, belongs to a package above the one that owns
 `reportingRefusals`. A test file may import only from its own package and from
 the packages that package depends on, so a case that needs the bench report's
 class cannot sit in the file of the package below it. The case is the same
 one the other refusals get, and only its list of classes is split.

 BOTH SWAPS ARE DISPOSABLE, and the suite runs at `concurrency: 1` for the
 same reason as in `cli-refusal.unit.test.ts`. `process.exitCode` and
 `console.error` are process-wide, and `describe` runs children concurrently by
 default, so cases that swapped them would race. There is exactly one process
 here: the runner spawns `node` once per test FILE, so nothing outside this file
 is touched, and nothing inside it may overlap.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportingRefusals, } from '@monochromatic-dev/translation-repair-run';
import { BenchReportError, } from '../../dist/final/node/index.mjs';
import {
  caseName,
  reportsMistake,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/cli-refusal-cases.test-fixture.ts';

//region Bench report refusal tests

/**
 Refusals an operator's own mistake raises that belong to the bench report.
 */
const BENCH_MISTAKES: readonly Error[] = [
  new BenchReportError({ seats: 1, },),
];

await describe({
  name: reportingRefusals.name,
  children: BENCH_MISTAKES.map(function operatorMistakeCase(refusal,) {
    return it({
      name: caseName({ refusal, },),
      fn: () => reportsMistake({ refusal, },),
    },);
  },),
  concurrency: 1,
},);

//endregion Bench report refusal tests
