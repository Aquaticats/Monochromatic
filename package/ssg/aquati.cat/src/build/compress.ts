/**
 Static-asset compression stage entry: zstd-compress dist/ via node:zlib across
 worker_threads.
 
 Replaces the former `zstd ... -r --adapt dist` CLI task. The build runs under
 Node; Node's async zstdCompress is pathological here,
 so each worker calls synchronous zstdCompressSync. The main thread snapshots
 the candidate set before spawning any worker, so no worker observes a `.zst`
 being created mid-run; existing `.zst` files are excluded from candidates, and
 symlinks are skipped via lstat so recursion can't escape dist/.
 
 This file is the self-referential worker entry: each worker re-runs it via
 `new Worker(new URL(import.meta.url))` and takes the worker branch below. The
 pure logic (heuristic, distribution, compression) lives in `./compress-lib.ts`.
 
 Run via `mise run build:compress` or `node src/build/compress.ts`.
 
 @see doc/decision/zstd-cli-to-node-zlib.md for the engine, level, and threading evidence.
 */
import { once, } from 'node:events';
import {
  lstat,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  isMainThread,
  parentPort,
  Worker,
  workerData,
} from 'node:worker_threads';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';

import {
  addTallies,
  compressBucket,
  distribute,
  EMPTY_TALLY,
  resolveWorkerCount,
  type Tally,
} from './compress-lib.ts';
import { DIST, } from './write-page.ts';

//region Worker lifecycle

/**
 Spawns one worker for a bucket and resolves with its tally.
 
 The worker re-runs this same file (`new Worker(new URL(import.meta.url))`),
 taking the worker branch. Resolution waits for the worker's `message`; an
 `error` event rejects via `once`'s built-in handling, and a non-zero `exit`
 (which can happen without an `error`) aborts the wait so it can't hang. The
 exit listener is registered before the `once` wait so neither event is missed.
 
 @param files - path bucket this worker compresses
 
 @returns worker's compression tally
 
 @example
 ```ts
 const tally = await runWorker({ files: ['dist/index.html',], },);
 ```
 */
async function runWorker(
  { files, }: { readonly files: readonly string[]; },
): Promise<Tally> {
  /**
   Worker re-executing this module's worker branch with the assigned bucket.
   */
  const worker = new Worker(
    new URL(import.meta.url,),
    { workerData: { files, }, },
  );
  /**
   Aborts the `message` wait when the worker exits non-zero without a message.
   */
  const exitGuard = new AbortController();
  worker.once(
    'exit',
    function onExit(code,) {
      if (code !== 0)
        exitGuard.abort(new Error(`compress worker exited with code ${code}`,),);
    },
  );
  /* oxlint-disable typescript/no-unsafe-type-assertion -- node:events once() is typed Promise<any[]>; widen through unknown then assert the worker's single Tally message payload */
  /**
   Worker's posted message arguments; exactly one {@link Tally} for a `message` event.
   */
  const messages = await once(
    worker,
    'message',
    { signal: exitGuard.signal, },
  ) as unknown as readonly Tally[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return nonNullishOrThrow(messages[0],);
}

//endregion Worker lifecycle

//region Entry: main thread orchestration, worker branch dispatch

if (isMainThread) {
  /**
   Tagged logger module, imported dynamically so worker threads (which take the
   else branch) never pay its import-time sink auto-discovery.
   */
  const {
    logger,
    tagged,
  } = await import('@monochromatic-dev/logger-core');
  /**
   Tagged logger for the compression stage.
   */
  const l = tagged({
    tag: 'compress',
    l: logger,
  },);

  /**
   Every entry under dist/, joined to a full path. `.zst` companions are
   filtered out before the (async) stat so no worker is spawned for them.
   */
  const distPaths = (await readdir(
    DIST,
    {
      recursive: true,
      encoding: 'utf8',
    },
  ))
    .map(function toDistPath(entry,) {
      return join(
        DIST,
        entry,
      );
    },)
    .filter(function notZstCompanion(entryPath,) {
      return !entryPath
        .toLowerCase()
        .endsWith('.zst',);
    },);
  /**
   Observational regular-file classification.
   */
  type MarkedPath = Readonly<{
    entryPath: string;
    isRegularFile: boolean;
  }>;
  /**
   Each non-`.zst` path paired with whether it is a regular file, resolved
   concurrently. `lstat` (not `stat`) keeps symlinks out so recursion can't
   escape dist/.
   */
  const markedPaths = await Promise.all(
    distPaths.map(async function markRegularFile(entryPath,): Promise<MarkedPath> {
      return {
        entryPath,
        isRegularFile: (await lstat(entryPath,))
          .isFile(),
      };
    },),
  );
  /**
   Snapshot of compressible candidates: regular files under dist/, excluding
   existing `.zst` companions and symlinks, taken before any worker starts.
   */
  const candidates = markedPaths
    .filter(function keepRegularFiles(marked,) {
      return marked.isRegularFile;
    },)
    .map(function toEntryPath(marked,) {
      return marked.entryPath;
    },);

  if (candidates.length === 0) {
    l.info(`compressed 0, skipped 0 (no candidate files in ${DIST})`,);
  }
  else {
    /**
     Worker count for this run, capped by parallelism and the candidate count.
     */
    const workers = await resolveWorkerCount({ fileCount: candidates.length, },);
    /**
     Per-worker tallies collected once every worker resolves.
     */
    const tallies = await Promise.all(
      distribute({
        files: candidates,
        workers,
      },)
        .map(function spawnBucket(files,) {
          return runWorker({ files, },);
        },),
    );
    /**
     Combined tally across every worker.
     */
    const total = tallies.reduce(
      function fold(
        left,
        right,
      ) {
        return addTallies({
          left,
          right,
        },);
      },
      EMPTY_TALLY,
    );
    l.info(
      `compressed ${total.written}, skipped ${total.skipped}, saved ${total.savedBytes} bytes across ${workers} workers`,
    );
  }
}
else {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- workerData is the structured-clone payload set by runWorker's Worker({ workerData: { files } }); node types it any */
  /**
   Bucket of file paths assigned to this worker by the main thread.
   */
  const { files, } = workerData as { readonly files: readonly string[]; };
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  nonNullishOrThrow(parentPort,)
    // oxlint-disable-next-line unicorn/require-post-message-target-origin -- parentPort is a node:worker_threads MessagePort; its postMessage takes (value, transferList?), not a browser Window targetOrigin
    .postMessage(compressBucket({ files, },),);
}

//endregion Entry: main thread orchestration, worker branch dispatch
