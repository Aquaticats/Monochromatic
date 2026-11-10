/**
 Consumer tests for the coverage gate's command line: only `check` and `write`
 are modes, and any other spelling fails with the usage error instead of
 falling through to the `check` path.

 The package's `fuzz:coverage` task runs `src/coverage-report.ts` with `node`,
 so these tests run that same entry point as a subprocess instead of importing
 it (the module reads `process.argv` at import time).

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

//region Subprocess adapter

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Gate script under test, resolved from this file so the test runs whichever
 checkout holds it.
 */
const REPORT_SCRIPT = join(import.meta.dirname, 'coverage-report.ts',);

/**
 Completed gate run: exit status plus both output streams.
 */
type GateRun = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Runs the gate script the way the `fuzz:coverage` task does, reporting a
 non-zero exit instead of throwing.

 @param args - Command-line arguments after the script path.

 @returns Exit status, stdout, and stderr of the run.

 @example
 ```ts
 await runGate(['check', coverageDir, baselinePath]);
 ```
 */
async function runGate(args: readonly string[],): Promise<GateRun> {
  try {
    /**
     Completed run for the accepted-mode paths.
     */
    const done = await promisifyExecFile(execFile,)(
      process.execPath,
      [REPORT_SCRIPT, ...args,],
      { encoding: 'utf8', },
    );
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (error) {
    /**
     Rejected run: `execFile` attaches the exit code and both streams.
     */
    const failure = error as {
      readonly code: number;
      readonly stdout: string;
      readonly stderr: string;
    };
    return {
      status: failure.code,
      stdout: failure.stdout,
      stderr: failure.stderr,
    };
  }
}

//endregion Subprocess adapter

//region Fixtures

/**
 Gate source marker the script projects coverage onto: a reported script URL
 must contain `package/upstream-fork/p-map/src/` to be counted.
 */
const FIXTURE_SOURCE_URL = 'file:///fixture-package/package/upstream-fork/p-map/src/index.ts';

/**
 Package-relative path `FIXTURE_SOURCE_URL` projects to, which is also the only
 baseline key.
 */
const FIXTURE_SOURCE_PATH = 'index.ts';

/**
 Fully executed functions the fixture coverage reports for that one file.
 */
const EXECUTED_FUNCTIONS = 2;

/**
 Baseline above `EXECUTED_FUNCTIONS`, so a `check` run must report a regression.
 */
const REGRESSED_BASELINE_COUNT = EXECUTED_FUNCTIONS + 1;

/**
 Disposable coverage directory plus baseline path the tests drive the gate with.
 */
type GateFixture = {
  readonly coverageDir: string;
  readonly baselinePath: string;
};

/**
 Writes a `NODE_V8_COVERAGE` directory the gate accepts, so every run reaches
 the mode branch instead of failing on an empty projection.

 @param root - Disposable fixture directory.

 @returns Coverage directory to read and baseline path to write.
 */
async function writeCoverageFixture(root: string,): Promise<GateFixture> {
  /**
   Directory the gate reads as `NODE_V8_COVERAGE` output.
   */
  const coverageDir = join(root, 'coverage',);
  await mkdir(coverageDir, { recursive: true, },);
  await writeFile(
    join(coverageDir, 'coverage-1.json',),
    JSON.stringify({
      result: [{
        url: FIXTURE_SOURCE_URL,
        functions: Array.from(
          { length: EXECUTED_FUNCTIONS, },
          function fullyExecuted() {
            return { ranges: [{ count: 1, },], };
          },
        ),
      },],
    },),
  );
  return {
    coverageDir,
    baselinePath: join(root, 'coverage-baseline.json',),
  };
}

/**
 Reads the committed baseline a run left behind.

 @param baselinePath - Baseline path handed to the gate.

 @returns Parsed baseline counts.
 */
async function readBaseline(baselinePath: string,): Promise<unknown> {
  return JSON.parse(await readFile(baselinePath, 'utf8',),) as unknown;
}

//endregion Fixtures

await describe({
  name: 'upstream-fork/p-map.fuzz coverage-report command line',
  timeout: 120_000,
  children: [
    //region Unknown modes

    ...['writ', 'Write', 'checks', '',].map(function mapUnrecognizedMode(mode) {
      return it({
        name: `rejects the unrecognized mode ${JSON.stringify(mode,)} with the usage error`,
        fn: async () => {
          await using fixtureDir = await mkdtempDisposable(join(scratchRoot(), 'p-map-coverage-mode-',),);
          /**
           Coverage and baseline that a `check` run accepts, so a run that
           falls through to the check path exits successfully.
           */
          const fixture = await writeCoverageFixture(fixtureDir.path,);
          await writeFile(
            fixture.baselinePath,
            JSON.stringify({ [FIXTURE_SOURCE_PATH]: EXECUTED_FUNCTIONS, },),
          );
          /**
           Gate run with the mistyped mode.
           */
          const run = await runGate([mode, fixture.coverageDir, fixture.baselinePath,],);
          expect(run.status,).not.toBe(0,);
          expect(run.stderr,).toContain('unknown mode',);
          expect(run.stderr,).toContain('<check|write>',);
          expect(run.stdout,).not.toContain('coverage gate passed',);
        },
      },);
    },),

    it({
      name: 'names the rejected mode spelling in the usage error',
      fn: async () => {
        await using fixtureDir = await mkdtempDisposable(join(scratchRoot(), 'p-map-coverage-named-',),);
        /**
         Coverage and baseline that a `check` run accepts, so only the missing
         usage error can fail this test.
         */
        const fixture = await writeCoverageFixture(fixtureDir.path,);
        await writeFile(
          fixture.baselinePath,
          JSON.stringify({ [FIXTURE_SOURCE_PATH]: EXECUTED_FUNCTIONS, },),
        );
        /**
         Gate run whose mode is a typo of `write`.
         */
        const run = await runGate(['writ', fixture.coverageDir, fixture.baselinePath,],);
        expect(run.stderr,).toContain('unknown mode writ',);
      },
    },),

    it({
      name: 'rejects an unrecognized mode before reading the coverage',
      fn: async () => {
        await using fixtureDir = await mkdtempDisposable(join(scratchRoot(), 'p-map-coverage-before-read-',),);
        /**
         Paths that do not exist, so a run that reads coverage first would report
         a missing coverage directory instead of the mode.
         */
        const missingCoverageDir = join(fixtureDir.path, 'absent-coverage',);
        const missingBaselinePath = join(fixtureDir.path, 'absent-baseline.json',);
        /**
         Gate run whose mode is misspelled and whose paths are absent.
         */
        const run = await runGate(['writ', missingCoverageDir, missingBaselinePath,],);
        expect(run.status,).not.toBe(0,);
        expect(run.stderr,).toContain('unknown mode writ',);
        expect(run.stderr,).not.toContain('ENOENT',);
      },
    },),

    //endregion Unknown modes

    //region Accepted modes

    it({
      name: 'check passes against a baseline the run meets',
      fn: async () => {
        await using fixtureDir = await mkdtempDisposable(join(scratchRoot(), 'p-map-coverage-check-',),);
        /**
         Coverage and baseline both at `EXECUTED_FUNCTIONS`.
         */
        const fixture = await writeCoverageFixture(fixtureDir.path,);
        /**
         Committed baseline the run must meet.
         */
        const baseline = { [FIXTURE_SOURCE_PATH]: EXECUTED_FUNCTIONS, };
        await writeFile(fixture.baselinePath, JSON.stringify(baseline,),);
        /**
         Gate run in check mode.
         */
        const run = await runGate(['check', fixture.coverageDir, fixture.baselinePath,],);
        expect(run.status,).toBe(0,);
        expect(run.stdout,).toContain('coverage gate passed',);
        expect(run.stdout,).toContain(`${String(Object.keys(baseline,)
          .length,)} files at or above baseline`,);
      },
    },),

    it({
      name: 'check fails when the baseline exceeds the measured count',
      fn: async () => {
        await using fixtureDir = await mkdtempDisposable(join(scratchRoot(), 'p-map-coverage-regressed-',),);
        /**
         Coverage at `EXECUTED_FUNCTIONS`, baseline above it.
         */
        const fixture = await writeCoverageFixture(fixtureDir.path,);
        await writeFile(
          fixture.baselinePath,
          JSON.stringify({ [FIXTURE_SOURCE_PATH]: REGRESSED_BASELINE_COUNT, },),
        );
        /**
         Gate run in check mode against the regressed baseline.
         */
        const run = await runGate(['check', fixture.coverageDir, fixture.baselinePath,],);
        expect(run.status,).not.toBe(0,);
        expect(run.stderr,).toContain('coverage regressed',);
        expect(run.stderr,).toContain(`${FIXTURE_SOURCE_PATH}: ${String(EXECUTED_FUNCTIONS,)} < baseline ${String(REGRESSED_BASELINE_COUNT,)}`,);
      },
    },),

    it({
      name: 'write refreezes the baseline from the measured counts',
      fn: async () => {
        await using fixtureDir = await mkdtempDisposable(join(scratchRoot(), 'p-map-coverage-write-',),);
        /**
         Coverage fixture whose stale baseline the run replaces.
         */
        const fixture = await writeCoverageFixture(fixtureDir.path,);
        await writeFile(
          fixture.baselinePath,
          JSON.stringify({ [FIXTURE_SOURCE_PATH]: REGRESSED_BASELINE_COUNT, },),
        );
        /**
         Gate run in write mode.
         */
        const run = await runGate(['write', fixture.coverageDir, fixture.baselinePath,],);
        expect(run.status,).toBe(0,);
        expect(run.stdout,).toContain('coverage baseline written',);
        expect(await readBaseline(fixture.baselinePath,),).toStrictEqual(
          { [FIXTURE_SOURCE_PATH]: EXECUTED_FUNCTIONS, },
        );
      },
    },),

    //endregion Accepted modes
  ],
},);
