import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';
import {
  chmodSync,
  mkdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

/**
 This package's directory, where the built wrapper and the fixture folders live.
 */
const PACKAGE_DIRECTORY = join(import.meta.dirname, '..',);
/**
 Built command-line wrapper under test, the file every `lint:oxlint` task runs.
 */
const ENTRY = join(PACKAGE_DIRECTORY, 'dist', 'final', 'node', 'oxlint-wrapper.mjs',);
/**
 Exit status the wrapper reports when it lets Oxlint's own status decide.
 */
const FINDINGS_EXIT = 1;
/**
 Env variable that tells the stand-in Oxlint which run to imitate.
 */
const STUB_MODE_VARIABLE = 'STUB_MODE';
/**
 What the stand-in Oxlint prints as its summary on a clean run.
 */
const CLEAN_SUMMARY = 'Found 0 warnings and 0 errors.';
/**
 What the stand-in prints as its summary when it produced a diagnostic.
 */
const FINDINGS_SUMMARY = 'Found 0 warnings and 1 errors.';
/**
 Crash report the real TypeScript 7.1 API server printed under Oxlint, minus the frames.
 */
const CRASH_REPORT = 'panic: runtime error: invalid memory address or nil pointer dereference\n'
  + '[signal SIGSEGV: segmentation violation code=0x1 addr=0x8 pc=0xbf31dc]\n'
  + 'goroutine 215 [running]:\n'
  + 'github.com/microsoft/TypeScript/tsc/internal/project/dirty.(*SyncMapEntry[...]).ChangeIf(...)\n';
/**
 Text the wrapper must print when the run's analysis crashed, which names the consequence.
 */
const INCOMPLETE_RUN_NOTE = 'findings are incomplete';

/**
 Stand-in `oxlint` that imitates one run shape, chosen by `STUB_MODE`.

 `clean` and `findings` are ordinary runs. `crash-clean` prints a crash report on standard
 error while reporting a clean summary, which is the shape a crashed analyzer leaves behind.
 `crash-findings` adds a diagnostic to that shape. `half-report` prints a crash header with no
 runtime report beside it, which is not a crash.
 */
const STUB = `#!/bin/sh
case "$${STUB_MODE_VARIABLE}" in
  clean) echo '${CLEAN_SUMMARY}'; exit 0 ;;
  findings) echo '${FINDINGS_SUMMARY}'; exit ${FINDINGS_EXIT} ;;
  crash-clean) printf '%s' '${CRASH_REPORT}' >&2; echo '${CLEAN_SUMMARY}'; exit 0 ;;
  crash-findings) printf '%s' '${CRASH_REPORT}' >&2; echo '${FINDINGS_SUMMARY}'; exit ${FINDINGS_EXIT} ;;
  half-report) echo 'panic: runtime error: not a report on its own' >&2; echo '${CLEAN_SUMMARY}'; exit 0 ;;
esac
echo "unknown STUB_MODE: $${STUB_MODE_VARIABLE}" >&2
exit 64
`;

/**
 One wrapper run's outcome.
 */
type WrapperRun = {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Runs the built wrapper with a stand-in `oxlint` first on `PATH`.

 @param mode - Run shape the stand-in imitates

 @returns Exit status and both captured streams

 @example
 ```ts
 const run = await runWrapper({ mode: 'crash-clean', },);
 ```
 */
async function runWrapper({
  mode,
}: {
  readonly mode: string;
}): Promise<WrapperRun> {
  /**
   Fresh fixture folder, holding this run's stand-in `oxlint`.
   */
  const directory = join(
    PACKAGE_DIRECTORY,
    'dist',
    'temp',
    'test',
    `oxlint-wrapper-${mode}-${Date.now()}-${Math.random().toString(36,).slice(2, 8,)}`,
  );
  mkdirSync(directory, { recursive: true, },);
  try {
    /**
     Stand-in program, which the wrapper finds by name on `PATH`.
     */
    const stubPath = join(directory, 'oxlint',);
    writeFileSync(stubPath, STUB,);
    chmodSync(stubPath, 0o755,);
    return await spawn(process.execPath, [ENTRY,], {
      cwd: directory,
      env: { PATH: `${directory}:/usr/bin:/bin`, [STUB_MODE_VARIABLE]: mode, },
    },).then(
      function finished(result,) {
        return { exitCode: 0, stdout: result.stdout, stderr: result.stderr, };
      },
      function failed(error: SubprocessError,) {
        return { exitCode: error.exitCode ?? (-1), stdout: error.stdout, stderr: error.stderr, };
      },
    );
  }
  finally {
    rmSync(directory, { recursive: true, force: true, },);
  }
}

await describe({
  name: 'task-oxlint analysis integrity',
  children: [
    //region A crashed analysis fails the run

    it({
      name: 'fails a run whose analyzer crashed while reporting a clean summary',
      fn: async () => {
        const run = await runWrapper({ mode: 'crash-clean', },);
        expect(run.exitCode,).toBe(FINDINGS_EXIT,);
        expect(run.stdout,).toContain(CLEAN_SUMMARY,);
        expect(run.stderr,).toContain('[task-oxlint]',);
        expect(run.stderr,).toContain(INCOMPLETE_RUN_NOTE,);
      },
    },),
    it({
      name: 'fails a run whose analyzer crashed beside a reported diagnostic',
      fn: async () => {
        const run = await runWrapper({ mode: 'crash-findings', },);
        expect(run.exitCode,).toBe(FINDINGS_EXIT,);
        expect(run.stdout,).toContain(FINDINGS_SUMMARY,);
        expect(run.stderr,).toContain(INCOMPLETE_RUN_NOTE,);
      },
    },),

    //endregion A crashed analysis fails the run

    //region Runs without a crash keep Oxlint's own status

    it({
      name: 'passes a clean run and says nothing about a crash',
      fn: async () => {
        const run = await runWrapper({ mode: 'clean', },);
        expect(run.exitCode,).toBe(0,);
        expect(run.stdout,).toContain(CLEAN_SUMMARY,);
        expect(run.stderr,).not.toContain(INCOMPLETE_RUN_NOTE,);
      },
    },),
    it({
      name: 'keeps the findings status and says nothing about a crash',
      fn: async () => {
        const run = await runWrapper({ mode: 'findings', },);
        expect(run.exitCode,).toBe(FINDINGS_EXIT,);
        expect(run.stdout,).toContain(FINDINGS_SUMMARY,);
        expect(run.stderr,).not.toContain(INCOMPLETE_RUN_NOTE,);
      },
    },),
    it({
      name: 'passes a run whose standard error quotes a crash header alone',
      fn: async () => {
        const run = await runWrapper({ mode: 'half-report', },);
        expect(run.exitCode,).toBe(0,);
        expect(run.stderr,).not.toContain(INCOMPLETE_RUN_NOTE,);
      },
    },),

    //endregion Runs without a crash keep Oxlint's own status
  ],
},);
