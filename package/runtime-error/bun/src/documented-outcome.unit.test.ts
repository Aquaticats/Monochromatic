/**
 Pins what each fixture does when it runs: the exit code or signal that ends it,
 one marker on the stream it prints, and how long it lives.

 The fixtures are this package's entire behaviour and the package has no build
 step, so a test spawns `src/<fixture>.ts` under Bun instead of importing a
 built artifact.

 @module
 */

import {
  readFileSync,
} from 'node:fs';
import {
  fileURLToPath,
} from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn, {
  SubprocessError,
} from 'nano-spawn';

//region Constants

/**
 Longest a fixture documented to end may run before the test kills it.
 The slowest of them, `uncaught-throw.ts`, took 184 ms on Bun 1.4.4.
 */
const EXIT_DEADLINE_MILLISECONDS = 20_000;

/**
 How long a fixture documented never to end must stay alive.
 One second of `oom.ts` holds about 180 MiB of resident memory, so the bound
 stays short.
 */
const HANG_BOUND_MILLISECONDS = 1_000;

/**
 Directory holding the fixtures, as a URL so a fixture name resolves against it.
 */
const FIXTURE_DIRECTORY_URL = new URL('.', import.meta.url,);

/**
 Directory holding the fixtures, as a path for the child's working directory.
 */
const FIXTURE_DIRECTORY = fileURLToPath(FIXTURE_DIRECTORY_URL,);

/**
 Prefix every allocation line of `oom.ts` starts with.
 */
const ALLOCATION_LINE_PREFIX = 'Allocated ';

/**
 Prefix of the mise.toml line that runs a fixture under Bun.
 */
const TASK_RUN_PREFIX = 'run = "bun src/';

/**
 Signal the test sends a fixture that outlives its deadline.
 */
const STOP_SIGNAL: NodeJS.Signals = 'SIGKILL';

//endregion Constants

//region Documented outcomes

/**
 A fixture whose process exits by itself with this status code.
 */
type ExitsWithCode = { readonly kind: 'exit'; readonly code: number };

/**
 A fixture whose process the runtime terminates with this signal.
 */
type EndsWithSignal = { readonly kind: 'signal'; readonly signal: string };

/**
 A fixture whose process never ends, so the test's own deadline stops it.
 */
type NeverEnds = { readonly kind: 'never' };

/**
 How a fixture's process ends.
 */
type FixtureEnding = ExitsWithCode | EndsWithSignal | NeverEnds;

/**
 What one fixture promises to do, as its TSDoc and its `run:` task description
 say.
 */
type DocumentedOutcome = {
  /** File name under `src/`. */
  readonly fileName: string;
  /** How the process stops when nothing else stops it. */
  readonly ending: FixtureEnding;
  /** Text the fixture must print on standard error, absent when it prints none. */
  readonly stderrMarker?: string;
};

/**
 Every fixture this package ships, with the outcome its documentation promises.
 */
const DOCUMENTED_OUTCOMES: readonly DocumentedOutcome[] = [
  {
    fileName: 'abort.ts',
    ending: { kind: 'signal', signal: 'SIGABRT', },
  },
  {
    fileName: 'infinite-loop.ts',
    ending: { kind: 'never', },
  },
  {
    fileName: 'not-a-function.ts',
    ending: { kind: 'exit', code: 1, },
    stderrMarker: 'TypeError: notAFunction is not a function',
  },
  {
    fileName: 'null-deref.ts',
    ending: { kind: 'exit', code: 1, },
    stderrMarker: "TypeError: null is not an object (evaluating 'value.property')",
  },
  {
    fileName: 'oom.ts',
    ending: { kind: 'never', },
  },
  {
    fileName: 'stack-overflow.ts',
    ending: { kind: 'exit', code: 1, },
    stderrMarker: 'RangeError: Maximum call stack size exceeded',
  },
  {
    fileName: 'uncaught-throw.ts',
    ending: { kind: 'exit', code: 1, },
    stderrMarker: 'error: Intentional uncaught throw',
  },
  {
    fileName: 'unhandled-rejection.ts',
    ending: { kind: 'exit', code: 1, },
    stderrMarker: 'error: Intentional unhandled rejection',
  },
];

//endregion Documented outcomes

//region Helpers

/**
 How one fixture run ended.
 */
type FixtureRun = {
  /** Exit code the process ended with, absent when a signal ended it. */
  readonly exitCode?: number;
  /** Signal that ended the process, absent when it exited on its own. */
  readonly signal?: string;
  /** Everything the fixture printed on standard output. */
  readonly stdout: string;
  /** Everything the fixture printed on standard error. */
  readonly stderr: string;
  /** How long the process lived, in milliseconds. */
  readonly durationMilliseconds: number;
};

/**
 Runs one fixture under Bun until it ends or its deadline passes.

 @param options - fixture to run and how long it may live

 @returns exit code or signal that ended the run, its output, and its lifetime

 @example
 ```ts
 await runFixture({ fileName: 'abort.ts', deadlineMilliseconds: 20_000 });
 ```
 */
async function runFixture(
  {
    fileName,
    deadlineMilliseconds,
  }: {
    readonly fileName: string;
    readonly deadlineMilliseconds: number;
  },
): Promise<FixtureRun> {
  try {
    const result = await spawn(
      'bun',
      [fileURLToPath(new URL(fileName, FIXTURE_DIRECTORY_URL,),),],
      {
        cwd: FIXTURE_DIRECTORY,
        killSignal: STOP_SIGNAL,
        timeout: deadlineMilliseconds,
      },
    );
    return {
      exitCode: 0,
      stdout: result.stdout,
      stderr: result.stderr,
      durationMilliseconds: result.durationMs,
    };
  }
  catch (error) {
    if (!(error instanceof SubprocessError)) {
      throw error;
    }
    return {
      ...(error.exitCode === undefined ? {} : { exitCode: error.exitCode, }),
      ...(error.signalName === undefined ? {} : { signal: error.signalName, }),
      stdout: error.stdout,
      stderr: error.stderr,
      durationMilliseconds: error.durationMs,
    };
  }
}

/**
 Collects the size unit each allocation line of `oom.ts` ends with.

 @param stdout - everything the fixture printed on standard output

 @returns every distinct unit word, in the order first seen

 @example
 ```ts
 allocationUnits('Allocated 256 MiB\nAllocated 512 MiB');
 ```
 */
function allocationUnits(stdout: string,): readonly string[] {
  const unitWords = stdout.split('\n',)
    .filter(function isAllocationLine(line,) {
      return line.startsWith(ALLOCATION_LINE_PREFIX,);
    },)
    .map(function unitWordOf(line,) {
      return line.slice(line.lastIndexOf(' ',) + 1,);
    },);
  return [...new Set(unitWords,),];
}

//endregion Helpers

await describe({
  name: 'documented fixture outcomes',
  children: [
    ...DOCUMENTED_OUTCOMES.map(function toOutcomeTest(outcome,) {
      return it({
        name: `${outcome.fileName} ends as its documentation says`,
        timeout: EXIT_DEADLINE_MILLISECONDS,
        fn: async () => {
          const run = await runFixture({
            fileName: outcome.fileName,
            deadlineMilliseconds: outcome.ending.kind === 'never'
              ? HANG_BOUND_MILLISECONDS
              : EXIT_DEADLINE_MILLISECONDS,
          },);
          if (outcome.ending.kind === 'exit') {
            expect(run.exitCode,).toBe(outcome.ending.code,);
            expect(run.signal,).toBeUndefined();
          }
          if (outcome.ending.kind === 'signal') {
            expect(run.exitCode,).toBeUndefined();
            expect(run.signal,).toBe(outcome.ending.signal,);
          }
          if (outcome.ending.kind === 'never') {
            // Only the deadline's own SIGKILL ends these, and only after the bound.
            expect(run.exitCode,).toBeUndefined();
            expect(run.signal,).toBe(STOP_SIGNAL,);
            expect(run.durationMilliseconds,).toBeGreaterThanOrEqual(
              HANG_BOUND_MILLISECONDS,
            );
          }
          if (outcome.stderrMarker !== undefined) {
            expect(run.stderr,).toContain(outcome.stderrMarker,);
          }
        },
      },);
    },),
    it({
      name: 'oom.ts reports every allocation in mebibytes',
      timeout: EXIT_DEADLINE_MILLISECONDS,
      fn: async () => {
        const run = await runFixture({
          fileName: 'oom.ts',
          deadlineMilliseconds: HANG_BOUND_MILLISECONDS,
        },);
        expect(allocationUnits(run.stdout,),).toEqual(['MiB',],);
      },
    },),
    it({
      name: 'every run task in mise.toml has a documented outcome here',
      fn: async () => {
        const miseToml = readFileSync(
          new URL('../mise.toml', import.meta.url,),
          'utf8',
        );
        for (const outcome of DOCUMENTED_OUTCOMES) {
          expect(miseToml,).toContain(`run = "bun src/${outcome.fileName}"`);
        }
        // One task per fixture, so a fixture added without an outcome fails here.
        const runTaskCount = miseToml.split(TASK_RUN_PREFIX,).length - 1;
        expect(runTaskCount,).toBe(DOCUMENTED_OUTCOMES.length,);
      },
    },),
  ],
},);
