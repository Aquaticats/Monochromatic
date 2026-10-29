/**
 Tests for how the rotation sweep reads a suite's output and compares a salt against salt 0 (ledger B701).

 Fixtures are cat-themed invention; the record shapes are the ones the harness of `package/test-harness/core` logs.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  EXIT_WITHOUT_CASE,
  failedCaseNames,
  failureTags,
  newFailures,
  reportLines,
  sweepPassed,
} from './rotation-sweep-report.test-fixture.ts';

/**
 Output of a suite that failed one case of two, as the harness logs it.
 */
const ONE_FAILED = [
  '[info] [2026-10-20T00:00:00.000Z] [nap] [PASS] curls up (2ms)',
  '[error] [2026-10-20T00:00:00.001Z] [nap] [stretches] [FAIL] (3ms) AssertionError: expected [ …(2) ] to include \'purr\'',
  '  continuation line of the cause [FAIL] (not a record)',
  '[error] [2026-10-20T00:00:00.002Z] [nap] [FAIL] (5ms) Error: stretches',
].join('\n',);

await describe({
  name: 'rotation sweep report',
  concurrency: 1,
  children: [
    describe({
      name: failedCaseNames.name,
      concurrency: 1,
      children: [
        it({
          name: 'names the case and leaves the suite that holds it out',
          fn: async () => {
            expect(failedCaseNames({ output: ONE_FAILED, exitCode: 1, },),).toStrictEqual(['[nap] [stretches]',],);
          },
        },),
        it({
          name: 'names every failed case of a suite once',
          fn: async () => {
          /**
           Output with a case failing twice in the log and another failing once.
           */
            const output = [
              '[error] [2026-10-20T00:00:00.001Z] [nap] [stretches] [FAIL] (3ms) Error',
              '[error] [2026-10-20T00:00:00.002Z] [nap] [stretches] [FAIL] (3ms) Error',
              '[error] [2026-10-20T00:00:00.003Z] [nap] [yawns] [FAIL] (3ms) Error',
              '[error] [2026-10-20T00:00:00.004Z] [nap] [FAIL] (5ms) Error: stretches',
            ].join('\n',);
            expect(failedCaseNames({ output, exitCode: 1, },),).toStrictEqual(['[nap] [stretches]', '[nap] [yawns]',],);
          },
        },),
        it({
          name: 'names an exit with no failure record as a case of its own',
          fn: async () => {
            expect(failedCaseNames({ output: 'Error: cannot find module\n', exitCode: 1, },),).toStrictEqual([EXIT_WITHOUT_CASE,],);
          },
        },),
        it({
          name: 'names nothing for a suite that passed',
          fn: async () => {
            expect(failedCaseNames({ output: '[info] [2026-10-20T00:00:00.000Z] [nap] [PASS] curls up (2ms)\n', exitCode: 0, },),)
              .toStrictEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: failureTags.name,
      concurrency: 1,
      children: [
        it({
          name: 'reads error records only, not a line that merely says FAIL',
          fn: async () => {
            expect(failureTags({ output: '[warn] [2026-10-20T00:00:00.000Z] [nap] [FAIL] (3ms)\nplain [FAIL] (3ms)\n', },),)
              .toStrictEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: newFailures.name,
      concurrency: 1,
      children: [
        it({
          name: 'keeps the failures salt 0 did not have, matched by file and case',
          fn: async () => {
          /**
           A case red with no shift.
           */
            const red = { file: 'stage/src/a.unit.test.ts', name: '[nap] [stretches]', };
          /**
           The same tags in another file, which is another case.
           */
            const other = { file: 'stage/src/b.unit.test.ts', name: '[nap] [stretches]', };
            expect(newFailures({ unsalted: [red,], salted: [red, other,], },),).toStrictEqual([other,],);
          },
        },),
      ],
    },),
    describe({
      name: reportLines.name,
      concurrency: 1,
      children: [
        it({
          name: 'names a case failed under several salts once, with every salt, and ends on the verdict',
          fn: async () => {
          /**
           A case that two salts fail.
           */
            const failed = { file: 'stage/src/a.unit.test.ts', name: '[nap] [stretches]', };
            expect(reportLines({
              outcome: {
                results: [
                  { salt: 2, failures: [failed,], flakes: [], milliseconds: 1, },
                  { salt: 3, failures: [failed,], flakes: [], milliseconds: 1, },
                ],
                redAtSaltZero: [],
              },
            },),).toStrictEqual([
              'FAIL under salts 2, 3: stage/src/a.unit.test.ts [nap] [stretches]',
              'rotation sweep: cases failing under a salt that salt 0 passes 1, cases red with no shift 0',
              'rotation sweep: NOT clean',
            ],);
          },
        },),
        it({
          name: 'reports a flake without failing the sweep, and a case red with no shift as failing it',
          fn: async () => {
          /**
           A case that failed once under a salt and passed on the rerun.
           */
            const flaked = { file: 'probe/src/b.unit.test.ts', name: '[purr]', };
          /**
           A case that fails with no shift.
           */
            const red = { file: 'probe/src/c.unit.test.ts', name: '[knead]', };
            expect(reportLines({
              outcome: {
                results: [{ salt: 4, failures: [], flakes: [flaked,], milliseconds: 1, },],
                redAtSaltZero: [red,],
              },
            },),).toStrictEqual([
              'flake under salt 4 (failed once, passed on rerun): probe/src/b.unit.test.ts [purr]',
              'red with no shift (salt 0), which cannot be read against the salts: probe/src/c.unit.test.ts [knead]',
              'rotation sweep: cases failing under a salt that salt 0 passes 0, cases red with no shift 1',
              'rotation sweep: NOT clean',
            ],);
          },
        },),
        it({
          name: 'ends on clean when nothing failed and a flake is the only line',
          fn: async () => {
          /**
           A case that failed once under a salt and passed on the rerun.
           */
            const flaked = { file: 'probe/src/b.unit.test.ts', name: '[purr]', };
          /**
           What the sweep found.
           */
            const outcome = { results: [{ salt: 4, failures: [], flakes: [flaked,], milliseconds: 1, },], redAtSaltZero: [], };
            expect(sweepPassed({ outcome, },),).toBe(true,);
            expect(reportLines({ outcome, },).at(-1,),).toBe('rotation sweep: clean',);
          },
        },),
      ],
    },),
  ],
},);
