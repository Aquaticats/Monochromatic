/**
 Pins what the out-of-memory fixture prints before its deadline passes: one
 allocation line per chunk, reporting mebibytes.

 The fixtures are this package's entire behaviour and the package has no build
 step, so a test spawns `src/<fixture>.ts` under Bun instead of importing a
 built artifact.

 @module
 */

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

//endregion Constants

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
        killSignal: 'SIGKILL',
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
  ],
},);
