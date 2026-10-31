/**
 Unit tests for the gh child process boundary.
 
 Every case drives a real child through the injected executable so failure
 classification rests on observed Node behavior rather than a scripted stand-in.
 
 @module
 */

import {
  mkdtemp,
  realpath,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createGhCommandRunner,
  type GhCommandOutcome,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Temp directory prefix for the working directory case.
 */
const TEMP_WORKING_DIR_PREFIX = 'gh-process-boundary-';

/**
 Child deadline used by the deadline case, short enough to keep the suite fast.
 */
const SHORT_DEADLINE_MILLISECONDS = 200;

/**
 Child sleep that outlives the short deadline.
 */
const LONG_SLEEP_MILLISECONDS = 5_000;

/**
 Captured output ceiling used by the ceiling case.
 */
const SMALL_OUTPUT_CEILING_BYTES = 1_024;

/**
 Output length that exceeds the small ceiling.
 */
const OVER_CEILING_OUTPUT_CHARACTERS = 100_000;

/**
 Exit code used by the failing child case.
 */
const FAILING_EXIT_CODE = 3;

/**
 Delay before an in-flight cancellation, long enough for the child to start.
 */
const ABORT_DELAY_MILLISECONDS = 60;

/**
 Build one runner driving the current Node executable as the child.
 
 @param deadlineMs - optional child deadline override
 
 @param maxOutputBytes - optional captured output ceiling override
 
 @param cwd - optional working directory override
 
 @returns runner bound to Node script arguments
 */
function nodeRunner(
  {
    deadlineMs,
    maxOutputBytes,
    cwd,
  }: {
    readonly deadlineMs?: number;
    readonly maxOutputBytes?: number;
    readonly cwd?: string;
  } = {},
) {
  return createGhCommandRunner({
    executable: process.execPath,
    ...(deadlineMs === undefined ? {} : { deadlineMs, }),
    ...(maxOutputBytes === undefined ? {} : { maxOutputBytes, }),
    ...(cwd === undefined ? {} : { cwd, }),
  },);
}

/**
 Build one argument vector running a Node script.
 
 @param script - Node script source
 
 @returns argument vector for the injected executable
 */
function nodeArgs(script: string,): readonly string[] {
  return [
    '--input-type=module',
    '--eval',
    script,
  ];
}

/**
 Read one completed run outcome or fail the case.
 
 @param outcome - runner outcome
 
 @returns completed run outcome
 */
function completed(outcome: GhCommandOutcome,): Extract<GhCommandOutcome, { readonly ran: true; }> {
  if (!outcome.ran)
    throw new Error(`expected a completed run, got no-run reason: ${outcome.reason}`,);
  return outcome;
}

/**
 Read one no-run outcome or fail the case.
 
 @param outcome - runner outcome
 
 @returns no-run outcome
 */
function notRun(outcome: GhCommandOutcome,): Extract<GhCommandOutcome, { readonly ran: false; }> {
  if (outcome.ran)
    throw new Error(`expected no run, got exit code ${String(outcome.exitCode,)}`,);
  return outcome;
}

//endregion Fixtures

await describe({
  name: '',
  // One case mutates process.env to prove child environment pins win over ambient values.
  concurrency: 1,
  children: [
    describe({
      name: createGhCommandRunner.name,
      children: [
        //region Completed runs

        it({
          name: 'captures standard output and a zero exit code',
          fn: async () => {
            /**
             Local value for outcome.
             */
            const outcome = completed(await nodeRunner()({
              args: nodeArgs('process.stdout.write("hello gh")',),
            },),);

            expect(outcome.exitCode,).toBe(0,);
            expect(outcome.stdout,).toBe('hello gh',);
            expect(outcome.stdoutIsText,).toBe(true,);
            expect(outcome.stdoutByteLength,).toBe(8,);
            expect(outcome.stderr,).toBe('',);
          },
        },),
        it({
          name: 'captures standard error alongside standard output',
          fn: async () => {
            /**
             Local value for outcome.
             */
            const outcome = completed(await nodeRunner()({
              args: nodeArgs('process.stdout.write("out"); process.stderr.write("err");',),
            },),);

            expect(outcome.stdout,).toBe('out',);
            expect(outcome.stderr,).toBe('err',);
          },
        },),
        it({
          name: 'reports a non-zero exit code with captured standard error',
          fn: async () => {
            /**
             Local value for outcome.
             */
            const outcome = completed(await nodeRunner()({
              args: nodeArgs(`process.stderr.write("boom"); process.exit(${String(FAILING_EXIT_CODE,)});`,),
            },),);

            expect(outcome.ran,).toBe(true,);
            expect(outcome.exitCode,).toBe(FAILING_EXIT_CODE,);
            expect(outcome.stderr,).toBe('boom',);
          },
        },),
        it({
          name: 'reports non-UTF-8 output as lossy with the captured byte count',
          fn: async () => {
            /**
             Local value for outcome.
             */
            const outcome = completed(await nodeRunner()({
              args: nodeArgs('process.stdout.write(Buffer.from([0x77, 0x4f, 0x46, 0x32, 0xff, 0xfe]));',),
            },),);

            expect(outcome.exitCode,).toBe(0,);
            expect(outcome.stdoutIsText,).toBe(false,);
            expect(outcome.stdoutByteLength,).toBe(6,);
            expect(outcome.stdout.includes('\uFFFD',),).toBe(true,);
          },
        },),
        it({
          name: 'reports valid UTF-8 carrying a NUL character as non-text',
          fn: async () => {
            /**
             Local value for outcome.
             */
            const outcome = completed(await nodeRunner()({
              args: nodeArgs(String.raw`process.stdout.write("before\u0000after");`,),
            },),);

            expect(outcome.exitCode,).toBe(0,);
            expect(outcome.stdoutIsText,).toBe(false,);
            expect(outcome.stdoutByteLength,).toBe(12,);
          },
        },),

        //endregion Completed runs

        //region No-run classifications

        it({
          name: 'reports a missing executable without a completed run',
          fn: async () => {
            /**
             Local value for runner.
             */
            const runner = createGhCommandRunner({ executable: 'gh-definitely-not-installed-xyz', },);

            /**
             Local value for outcome.
             */
            const outcome = notRun(await runner({
              args: [
                'repo',
                'view',
                'cli/cli',
              ],
            },),);

            expect(outcome.reason,).toContain('gh-definitely-not-installed-xyz',);
            expect(outcome.reason,).toContain('is missing from PATH',);
            expect(outcome.reason,).toContain('did not spawn',);
          },
        },),
        it({
          name: 'reports an exceeded deadline without a completed run',
          fn: async () => {
            /**
             Local value for runner.
             */
            const runner = nodeRunner({ deadlineMs: SHORT_DEADLINE_MILLISECONDS, },);

            /**
             Local value for outcome.
             */
            const outcome = notRun(await runner({
              args: nodeArgs(`setTimeout(function keepAlive() {}, ${String(LONG_SLEEP_MILLISECONDS,)});`,),
            },),);

            expect(outcome.reason,).toContain(`${String(SHORT_DEADLINE_MILLISECONDS,)}ms deadline`,);
          },
        },),
        it({
          name: 'reports an exceeded output ceiling without a completed run',
          fn: async () => {
            /**
             Local value for runner.
             */
            const runner = nodeRunner({ maxOutputBytes: SMALL_OUTPUT_CEILING_BYTES, },);

            /**
             Local value for outcome.
             */
            const outcome = notRun(await runner({
              args: nodeArgs(`process.stdout.write("x".repeat(${String(OVER_CEILING_OUTPUT_CHARACTERS,)}));`,),
            },),);

            expect(outcome.reason,).toContain(`${String(SMALL_OUTPUT_CEILING_BYTES,)} byte capture ceiling`,);
          },
        },),

        //endregion No-run classifications

        //region Caller contract

        it({
          name: 'throws instead of classifying a caller cancellation',
          fn: async () => {
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            controller.abort();

            /**
             Local value for caught.
             */
            let caught: unknown;
            try {
              await nodeRunner()({
                args: nodeArgs('process.stdout.write("never");',),
                signal: controller.signal,
              },);
            }
            catch (error: unknown) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(Error,);
          },
        },),
        it({
          name: 'throws and stops the child when cancellation arrives mid-run',
          fn: async () => {
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            /**
             Local value for startedAt.
             */
            const startedAt = performance.now();
            /**
             Local value for pending.
             */
            const pending = nodeRunner()({
              args: nodeArgs(`setTimeout(function keepAlive() {}, ${String(LONG_SLEEP_MILLISECONDS,)});`,),
              signal: controller.signal,
            },);
            await delay(ABORT_DELAY_MILLISECONDS,);
            controller.abort();

            /**
             Local value for caught.
             */
            let caught: unknown;
            try {
              await pending;
            }
            catch (error: unknown) {
              caught = error;
            }

            /**
             Local value for elapsed.
             */
            const elapsed = performance.now() - startedAt;
            expect(caught,).toBeInstanceOf(Error,);
            expect(elapsed < LONG_SLEEP_MILLISECONDS,).toBe(true,);
          },
        },),
        it({
          name: 'pins the gh host and output variables over ambient values',
          fn: async () => {
            /**
             Local value for savedHost.
             */
            const savedHost = process.env.GH_HOST;
            /**
             Local value for savedForceTty.
             */
            const savedForceTty = process.env.GH_FORCE_TTY;
            process.env.GH_HOST = 'github.example.invalid';
            process.env.GH_FORCE_TTY = '80';

            /**
             Local value for outcome.
             */
            const outcome = completed(await nodeRunner()({
              args: nodeArgs('process.stdout.write(JSON.stringify([process.env.GH_HOST, process.env.GH_FORCE_TTY, process.env.CLICOLOR_FORCE, process.env.NO_COLOR]));',),
            },),);
            if (savedHost === undefined)
              delete process.env.GH_HOST;
            else
              process.env.GH_HOST = savedHost;
            if (savedForceTty === undefined)
              delete process.env.GH_FORCE_TTY;
            else
              process.env.GH_FORCE_TTY = savedForceTty;

            expect(JSON.parse(outcome.stdout,),).toEqual([
              'github.com',
              '',
              '0',
              '1',
            ],);
          },
        },),
        it({
          name: 'runs the child in the injected working directory',
          fn: async () => {
            /**
             Local value for workingDirectory.
             */
            const workingDirectory = await mkdtemp(join(
              scratchRoot(),
              TEMP_WORKING_DIR_PREFIX,
            ),);

            /**
             Local value for outcome.
             */
            const outcome = completed(await nodeRunner({ cwd: workingDirectory, },)({
              args: nodeArgs('process.stdout.write(process.cwd());',),
            },),);

            expect(outcome.stdout,).toBe(await realpath(workingDirectory,),);
            await rm(
              workingDirectory,
              {
                recursive: true,
                force: true,
              },
            );
          },
        },),

        //endregion Caller contract
      ],
    },),
  ],
},);
