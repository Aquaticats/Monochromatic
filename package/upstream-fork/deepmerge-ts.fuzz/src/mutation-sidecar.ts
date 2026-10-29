/**
 Runs the sidecar's unit files against one deepmerge-ts bundle through
 `DEEPMERGE_FUZZ_TARGET`, for `./mutation-sweep.ts`. A file that times out
 counts as failing, as Stryker counts timeouts as detections.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { glob, } from 'node:fs/promises';

/**
 Per-file limit; a mutant that makes a file loop forever counts as detected.
 */
const FILE_TIMEOUT_MS = 120_000;

/**
 Sidecar files run at once, matching the container's two CPUs.
 */
const WORKERS = 2;

/**
 Characters of a failing file's output kept in the sweep log.
 */
const OUTPUT_TAIL = 4_000;

/**
 Outcome of one sidecar file against one bundle.
 */
export type FileOutcome = {
  readonly file: string;
  readonly passed: boolean;
  readonly exit: string;
  readonly tail: string;
};

/**
 Sidecar unit test files, sorted so runs are comparable.

 @returns Paths relative to the package root (the working directory).

 @example
 ```ts
 const files = await sidecarFiles();
 ```
 */
export async function sidecarFiles(): Promise<readonly string[]> {
  return (await Array.fromAsync(glob('src/*.unit.test.ts',),)).toSorted();
}

/**
 Run one sidecar file against a bundle.

 @param file - Sidecar test file.

 @param target - Bundle `DEEPMERGE_FUZZ_TARGET` points at.

 @returns Whether it passed, how it exited, and the end of its output when it failed.

 @example
 ```ts
 await runSidecarFile({ file: 'src/model.unit.test.ts', target: '/out/mutants/7.mjs', });
 ```
 */
async function runSidecarFile(
  {
    file,
    target,
  }: {
    readonly file: string;
    readonly target: string;
  },
): Promise<FileOutcome> {
  /**
   Child running the file.
   */
  const child = spawn(
    'node',
    [file,],
    {
      env: {
        ...process.env,
        DEEPMERGE_FUZZ_TARGET: target,
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Output chunks, stdout and stderr interleaved.
   */
  const chunks: Buffer[] = [];
  child.stdout
    .on(
    'data',
    function collectStdout(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  child.stderr
    .on(
    'data',
    function collectStderr(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  /**
   Kill timer for a file the mutant made hang.
   */
  const timer = setTimeout(
    function killHung() {
      child.kill('SIGKILL',);
    },
    FILE_TIMEOUT_MS,
  );
  /**
   Exit code and signal once both streams closed.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  clearTimeout(timer,);
  /**
   Exit code (`null` when a signal ended the file) and signal name.
   */
  const [
    code,
    signal,
  ] = closed;
  return {
    exit: ((typeof code) === 'number') ? `code ${String(code,)}` : `signal ${String(signal,)}`,
    file,
    passed: code === 0,
    tail: (code === 0) ? '' : Buffer.concat(chunks,)
      .toString('utf8',)
      .slice(-OUTPUT_TAIL,),
  };
}

/**
 Run every sidecar file against one bundle, `WORKERS` at a time.

 @param files - Sidecar test files.

 @param target - Bundle under test.

 @returns Outcomes sorted by file.

 @example
 ```ts
 await runSidecar({ files, target: '/out/mutants/baseline.mjs', });
 ```
 */
export async function runSidecar(
  {
    files,
    target,
  }: {
    readonly files: readonly string[];
    readonly target: string;
  },
): Promise<readonly FileOutcome[]> {
  /**
   Files not yet claimed by a worker.
   */
  const queue = [...files,];
  /**
   Each worker's outcomes.
   */
  const perWorker = await Promise.all(Array.from(
    { length: WORKERS, },
    async function worker() {
    /**
     Outcomes this worker produced.
     */
      const outcomes: FileOutcome[] = [];
      for (let file = queue.shift(); file !== undefined; file = queue.shift()) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- each worker runs its files one at a time; WORKERS bounds concurrency.
        outcomes.push(await runSidecarFile({
          file,
          target,
        },),);
      }
      return outcomes;
    },
  ),);
  return perWorker.flat()
    .toSorted(function byFile(
      left,
      right,
    ) {
      return left.file
        .localeCompare(right.file,);
    },);
}
