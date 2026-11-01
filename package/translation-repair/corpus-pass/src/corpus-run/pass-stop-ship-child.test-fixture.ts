/**
 The child of the case in `pass-stop-signals.unit.test.ts` that holds an entry in flight with a settled slice: it holds a
 runs directory's lock and a run log as a pass does, installs the stop-signal handlers over the stop the pass shares with
 its entries, registers an entry in flight that has settled one slice, says it is ready and waits for a signal.

 When the signal arrives the entry does what `runEntryPipeline` does in its `catch`: it ships its interim page through
 `shipInterimOnStop`, with the stop's signal and its flight, and the handler then flushes, releases the lock and exits.

 Past the shared hang stop it throws, which leaves a code no handler leaves, so a case whose handler never ran fails.

 @module
 */

import { once, } from 'node:events';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { lockRunsDir, } from '@monochromatic-dev/translation-repair-run';
import {
  installRunLog,
  openRunLog,
} from '@monochromatic-dev/translation-repair-typography';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  createPassStop,
  handlingStopSignals,
  shipInterimOnStop,
  startEntryProgress,
} from '../../dist/final/node/index.mjs';
import {
  consolidatedAt,
  CORPUS_SHA,
  DIGEST,
  ENTRY_ID,
  preparation,
  SETTLED_FIRST,
  SOURCE_DOC,
  TARGET_DOC,
  TIP,
} from './pass-entry-stop.test-fixture.ts';

/**
 Runs directory the case made, the first argument.
 */
const [runsDir = '',] = process.argv
  .slice(2,);

/**
 Waits for a signal to be aborted, or for the shared hang stop.

 @param signal - signal to wait on

 @returns Whether it was aborted in time

 @example
 ```ts
 if (await abortedWithin({ signal, },)) ship();
 ```
 */
async function abortedWithin({ signal, }: { readonly signal: AbortSignal; },): Promise<boolean> {
  return await Promise.race([
    aborts({ signal, },),
    hangsOut(),
  ],);
}

/**
 Settles true when a signal aborts.

 @param signal - signal to wait on

 @returns True, once it aborted

 @example
 ```ts
 await aborts({ signal, },);
 ```
 */
async function aborts({ signal, }: { readonly signal: AbortSignal; },): Promise<boolean> {
  await once(
    signal,
    'abort',
  );
  return true;
}

/**
 Settles false when the shared hang stop passes.

 @returns False, once the hang stop passed

 @example
 ```ts
 await hangsOut();
 ```
 */
async function hangsOut(): Promise<boolean> {
  await wait(HANG_STOP_MS,);
  return false;
}

/**
 Holds an entry in flight with one settled slice as a pass does, says it is ready and waits for a stop signal.

 @throws Error when no stop signal reaches the process within the hang stop

 @example
 ```ts
 await holdEntryUntilSignalled();
 ```
 */
async function holdEntryUntilSignalled(): Promise<void> {
  /**
   The lock of the runs directory, as a pass holds it.
   */
  await using lock = await lockRunsDir({ runsDir, },);

  /**
   The run log of the runs directory.
   */
  await using runLog = await openRunLog({ runDir: runsDir, },);

  /**
   The install of that log.
   */
  using _installed = installRunLog({ log: runLog, },);

  /**
   What the handlers and the entry share.
   */
  const stop = createPassStop();

  /**
   The handlers of the stop signals, which release the lock.
   */
  using _signals = handlingStopSignals({
    log: runLog,
    release: async function releaseLock(): Promise<void> {
      await lock[Symbol.asyncDispose]();
    },
    stop,
  },);

  /**
   The entry's place in the stop.
   */
  using flight = stop.begin();

  /**
   Preparation the entry was sliced by.
   */
  const prepared = preparation();

  /**
   What the entry had settled when the signal arrives: one slice.
   */
  const progress = {
    ...startEntryProgress(),
    prepared,
    archiveText: prepared.targetText,
    folded: true as const,
    consolidated: [consolidatedAt({
      sliceIndex: 0,
      text: SETTLED_FIRST,
    },),],
  };

  console.log('ready',);
  /**
   The signal the entry runs under.
   */
  const { signal: stopSignal, } = stop;
  if (await abortedWithin({ signal: stopSignal, },)) {
    // What the pipeline's catch does for an entry the stop ended.
    await shipInterimOnStop({
      error: stopSignal.reason,
      progress,
      entryId: ENTRY_ID,
      sourceText: SOURCE_DOC,
      targetText: TARGET_DOC,
      publishDir: join(
        runsDir,
        'fixed',
      ),
      interimDir: join(
        runsDir,
        'interim',
      ),
      tip: TIP,
      pipelineDigest: DIGEST,
      corpusSha: CORPUS_SHA,
      attempt: 1,
      stopSignal,
      flight,
      l: tagged({ tag: 'pass-stop-ship-child', },),
    },);
  }
  // The handler exits the process once the entry has shipped; reaching the line after this wait means it did not.
  await Promise.race([
    stop.exited,
    wait(HANG_STOP_MS,),
  ],);
  throw new Error('no stop signal reached the child within the hang stop',);
}

await holdEntryUntilSignalled();
