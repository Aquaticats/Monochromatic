/**
 Runs one sidecar test file against one deepmerge-ts bundle for the
 historical-recall run (`./recall-runtime.ts`), keeping the whole output so
 failing tests can be named, and optionally as one campaign round (seed and
 run count through `./fuzz-budget.ts`'s environment variables).

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';

import { failingTests, } from './recall-edit.ts';

/**
 Per-file limit for the bounded layer; a bundle that makes a file hang counts
 as failing it.
 */
const BOUNDED_TIMEOUT_MS = 180_000;

/**
 Per-file limit for one campaign round of a property file.
 */
const ROUND_TIMEOUT_MS = 900_000;

/**
 Which layer a file runs in: the fixed-seed bounded layer, or one campaign
 round with its own seed and run count per property.
 */
export type Layer = {
  readonly kind: 'bounded';
} | {
  readonly kind: 'round';
  readonly seed: number;
  readonly numRuns: number;
};

/**
 Failures of one file against one bundle.
 */
export type FileRun = {
  readonly file: string;
  readonly exit: string;
  readonly failures: readonly string[];
};

/**
 Run one file and name its failing tests; a nonzero exit without a named
 failure is recorded as the synthetic failure `<exit>`.

 @param file - Sidecar test file, relative to the package root.

 @param target - Bundle `DEEPMERGE_FUZZ_TARGET` points at.

 @param layer - Bounded layer or campaign round.

 @returns Exit description and failing test names.

 @example
 ```ts
 await runFile({ file: 'src/model.unit.test.ts', layer: { kind: 'bounded', }, target: '/out/bundles/baseline.mjs', });
 ```
 */
export async function runFile(
  {
    file,
    target,
    layer,
  }: {
    readonly file: string;
    readonly target: string;
    readonly layer: Layer;
  },
): Promise<FileRun> {
  /**
   Environment for the child: the target, plus the round when one is given.
   */
  const env = (layer.kind === 'bounded')
    ? {
      ...process.env,
      DEEPMERGE_FUZZ_TARGET: target,
    }
    : {
      ...process.env,
      DEEPMERGE_FUZZ_NUM_RUNS: String(layer.numRuns,),
      DEEPMERGE_FUZZ_SEED: String(layer.seed,),
      DEEPMERGE_FUZZ_TARGET: target,
    };
  /**
   Child running the file.
   */
  const child = spawn(
    'node',
    [file,],
    {
      env,
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
   Kill timer for a hung file.
   */
  const timer = setTimeout(
    function killHung() {
      child.kill('SIGKILL',);
    },
    (layer.kind === 'bounded') ? BOUNDED_TIMEOUT_MS : ROUND_TIMEOUT_MS,
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
  /**
   Exit description.
   */
  const exit = ((typeof code) === 'number') ? `code ${String(code,)}` : `signal ${String(signal,)}`;
  /**
   Named failures from the output.
   */
  const named = failingTests(Buffer.concat(chunks,)
    .toString('utf8',),);
  return {
    exit,
    failures: ((code !== 0) && (named.length === 0)) ? [`<${exit}>`,] : named,
    file,
  };
}

/**
 Run files against one bundle, two at a time to match the container's CPUs.

 @param files - Sidecar test files.

 @param target - Bundle under test.

 @param layer - Bounded layer or campaign round.

 @returns Runs in file order.

 @example
 ```ts
 await runFiles({ files, layer: { kind: 'bounded', }, target, });
 ```
 */
export async function runFiles(
  {
    files,
    target,
    layer,
  }: {
    readonly files: readonly string[];
    readonly target: string;
    readonly layer: Layer;
  },
): Promise<readonly FileRun[]> {
  /**
   Files not yet claimed by a worker.
   */
  const queue = [...files,];
  /**
   Each worker's runs.
   */
  const perWorker = await Promise.all([
    0,
    1,
  ].map(async function worker() {
    /**
     Runs this worker produced.
     */
    const runs: FileRun[] = [];
    for (let file = queue.shift(); file !== undefined; file = queue.shift()) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- each worker runs its files one at a time; two workers bound concurrency.
      runs.push(await runFile({
        file,
        layer,
        target,
      },),);
    }
    return runs;
  },),);
  return perWorker.flat()
    .toSorted(function byFile(
      left,
      right,
    ) {
      return left.file
        .localeCompare(right.file,);
    },);
}
