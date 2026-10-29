/**
 Tests for the rotation sweep's request and package choice (ledger B701), and for the task that runs it:
 the task must build every package the sweep loads, so a package added to the stage's dependents is built before it is swept.

 Fixtures are cat-themed invention; the package case reads this package's own `mise.toml` and the family's manifests.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';

import {
  rotationSweepMain,
  sweepRequestOf,
} from './rotation-sweep.test-fixture.ts';
import { packagesReachingStage, } from './rotation-sweep-packages.test-fixture.ts';
import type {
  FailedCase,
  SweepOutcome,
} from './rotation-sweep-report.test-fixture.ts';

/**
 Header of the task that runs the sweep.
 */
const SWEEP_TASK = '[tasks."test:rotation-sweep"]';

/**
 The command an operator reruns one package at one salt with.
 */
const RERUN = 'mise run //package/translation-repair/source-scan:test:rotation-sweep --';

/**
 A case that two salts fail.
 */
const STRETCHES: FailedCase = { file: 'stage/src/a.unit.test.ts', name: '[nap] [stretches]', };

/**
 A case that fails with no shift.
 */
const KNEADS: FailedCase = { file: 'probe/src/c.unit.test.ts', name: '[knead]', };

/**
 Runs the sweep command over a sweep that found a given outcome, collecting what it wrote.

 @param outcome - what the injected sweep finds, or the error it throws

 @returns Command's result, its report lines and its refusal lines

 @example
 ```ts
 const { result, out, err, } = await commandOver({ outcome: { results: [], redAtSaltZero: [], }, },);
 ```
 */
async function commandOver(
  { outcome, }: { readonly outcome: SweepOutcome | Error; },
): Promise<{
  readonly result: unknown;
  readonly out: readonly string[];
  readonly err: readonly string[];
}> {
  /**
   Lines of the report.
   */
  const out: string[] = [];

  /**
   Lines of a refusal.
   */
  const err: string[] = [];
  return {
    result: await rotationSweepMain({
      args: [],
      sweep: async function found(): Promise<SweepOutcome> {
        if (Error.isError(outcome,))
          throw outcome;
        return outcome;
      },
      write: function takeOut({ line, },): void {
        out.push(line,);
      },
      writeError: function takeErr({ line, },): void {
        err.push(line,);
      },
    },),
    out,
    err,
  };
}

await describe({
  name: 'rotation sweep',
  concurrency: 1,
  children: [
    describe({
      name: sweepRequestOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'runs every salt from 0 to 9 over every package when nothing is asked',
          fn: async () => {
            expect(sweepRequestOf({ args: [], },),).toStrictEqual({ packages: [], salts: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9,], },);
          },
        },),
        it({
          name: 'reads repeated packages and salts, sorted and once each',
          fn: async () => {
            expect(sweepRequestOf({ args: ['--package', 'stage', '--salt', '6', '--package', 'probe', '--salt', '2', '--salt', '6',], },),)
              .toStrictEqual({ packages: ['stage', 'probe',], salts: [2, 6,], },);
          },
        },),
        it({
          name: 'refuses an unknown option, a salt past the largest bench and an option without a value',
          fn: async () => {
            for (const args of [['--nap', 'long',], ['--salt', '10',], ['--salt', 'x',], ['--package',],]) {
              expect(caught(function bad(): void {
                sweepRequestOf({ args, },);
              },),).toBeDefined();
            }
          },
        },),
      ],
    },),
    describe({
      name: rotationSweepMain.name,
      concurrency: 1,
      children: [
        it({
          name: 'REFUSES A SWEEP THAT FOUND A FAILING CASE IN PLAIN WORDS, naming the case, its package, its salts and the '
            + 'command that reruns one salt, and returns the stated-refusal code instead of throwing',
          fn: async () => {
            const { result, out, err, } = await commandOver({
              outcome: {
                results: [
                  { salt: 2, failures: [STRETCHES,], flakes: [], milliseconds: 1, },
                  { salt: 3, failures: [STRETCHES,], flakes: [], milliseconds: 1, },
                ],
                redAtSaltZero: [],
              },
            },);
            expect(result,).toBe(6,);
            expect(out,).toContain('FAIL under salts 2, 3: stage/src/a.unit.test.ts [nap] [stretches]',);
            expect(out.at(-1,),).toBe('rotation sweep: NOT clean',);
            expect(err,).toStrictEqual([
              'rotation sweep: refused, because the sweep ran to the end and found cases to fix; this is a finding, not a fault of the command',
              'case: stage/src/a.unit.test.ts [nap] [stretches]',
              '  package: stage',
              '  fails under salts: 2, 3 (salt 0 passes it)',
              `  rerun one salt: ${RERUN} --package stage --salt 2`,
            ],);
          },
        },),
        it({
          name: 'NAMES A CASE RED WITH NO SHIFT AS A FINDING TOO, with its package and the command that reruns salt 0 alone',
          fn: async () => {
            const { result, err, } = await commandOver({
              outcome: { results: [], redAtSaltZero: [KNEADS,], },
            },);
            expect(result,).toBe(6,);
            expect(err,).toStrictEqual([
              'rotation sweep: refused, because the sweep ran to the end and found cases to fix; this is a finding, not a fault of the command',
              'case: probe/src/c.unit.test.ts [knead]',
              '  package: probe',
              '  fails with no shift (salt 0), which cannot be read against the salts',
              `  rerun salt 0 alone: ${RERUN} --package probe --salt 0`,
            ],);
          },
        },),
        it({
          name: 'PRINTS NO STACK FOR A FINDING: no line of a refusal is a stack frame or an error header',
          fn: async () => {
            const { err, } = await commandOver({
              outcome: { results: [{ salt: 4, failures: [STRETCHES,], flakes: [], milliseconds: 1, },], redAtSaltZero: [KNEADS,], },
            },);
            expect(err.length,).toBeGreaterThan(0,);
            expect(err.filter(function looksLikeStack(line,): boolean {
              return line.trimStart().startsWith('at ',) || line.includes('Error:',);
            },),).toStrictEqual([],);
          },
        },),
        it({
          name: 'LEAVES A CLEAN SWEEP AS IT WAS: the report ends on the clean line, nothing is refused, and the code is 0',
          fn: async () => {
            const { result, out, err, } = await commandOver({
              outcome: { results: [{ salt: 4, failures: [], flakes: [], milliseconds: 1, },], redAtSaltZero: [], },
            },);
            expect(result,).toBe(0,);
            expect(out.at(-1,),).toBe('rotation sweep: clean',);
            expect(err,).toStrictEqual([],);
          },
        },),
        it({
          name: 'REFUSES A BAD OPTION IN PLAIN WORDS, naming the option and the valid forms, returns the stated-refusal '
            + 'code, runs no sweep and prints no stack: an unknown option, a salt past the largest bench, a salt that '
            + 'is no number and an option without a value',
          fn: async () => {
          /**
           Each bad command line and the sentence it is refused with.
           */
            const refusals: readonly (readonly [readonly string[], string])[] = [
              [['--nap', 'long',], 'rotation sweep: refused, --nap is not an option'],
              [['--salt', '10',], 'rotation sweep: refused, --salt takes a whole number from 0 to 9, got 10'],
              [['--salt', 'x',], 'rotation sweep: refused, --salt takes a whole number from 0 to 9, got x'],
              [['--package',], 'rotation sweep: refused, --package needs a value'],
            ];
            for (const [args, sentence,] of refusals) {
            /**
             Lines written, and whether a sweep was started.
             */
              const out: string[] = [];
              const err: string[] = [];
              const started: boolean[] = [];
            /* oxlint-disable no-await-in-loop -- each command line is refused on its own, one after the other */
              const result = await rotationSweepMain({
                args,
                sweep: async function unasked(): Promise<SweepOutcome> {
                  started.push(true,);
                  return { results: [], redAtSaltZero: [], };
                },
                write: function takeOut({ line, },): void {
                  out.push(line,);
                },
                writeError: function takeErr({ line, },): void {
                  err.push(line,);
                },
              },);
            /* oxlint-enable no-await-in-loop */
              expect(result,).toBe(6,);
              expect(started,).toStrictEqual([],);
              expect(out,).toStrictEqual([],);
              expect(err,).toStrictEqual([
                `${sentence}; the options are --package <directory> and --salt <n>, each repeatable`,
              ],);
            }
          },
        },),
        it({
          name: 'LETS A FAULT OF THE MACHINERY SURFACE WITH ITS STACK, an error out of the sweep itself being nobody\'s '
            + 'finding: it is thrown unchanged and nothing is refused',
          fn: async () => {
          /**
           What the machinery broke with.
           */
            const fault = new TypeError('the canary could not read the rotation',);
          /**
           What the command threw, and what it wrote before.
           */
            const thrown = await rejectionOf(async function broken(): Promise<void> {
              await commandOver({ outcome: fault, },);
            },);
            expect(thrown,).toBe(fault,);
            expect(Error.isError(thrown,) ? thrown.stack : '',).toContain('    at ',);
          },
        },),
      ],
    },),
    describe({
      name: packagesReachingStage.name,
      concurrency: 1,
      children: [
        it({
          name: 'holds the stage and its dependents, and leaves out the scans and the packages the stage leans on',
          fn: async () => {
          /**
           Directories chosen.
           */
            const chosen = await packagesReachingStage();
            expect(chosen,).toContain('stage',);
            expect(chosen,).toContain('probe',);
            expect(chosen,).not.toContain('source-scan',);
            expect(chosen,).not.toContain('typography',);
          },
        },),
        it({
          name: 'is built by the task that sweeps it: every chosen package is a dependency of test:rotation-sweep',
          fn: async () => {
          /**
           Text of this package's task file.
           */
            const tasks = await readFile(
              join(
                import.meta.dirname,
                '..',
                '..',
                'mise.toml',
              ),
              'utf8',
            );
          /**
           The sweep task's block, up to the next task.
           */
            const block = tasks.slice(tasks.indexOf(SWEEP_TASK,),).split('\n[tasks.',)[0] ?? '';
          /**
           Packages chosen whose build the task does not name.
           */
            const unbuilt = (await packagesReachingStage()).filter(function lacksBuild(name,): boolean {
              return !block.includes(`"//package/translation-repair/${name}:build:js:node"`,);
            },);
            expect(unbuilt,).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
