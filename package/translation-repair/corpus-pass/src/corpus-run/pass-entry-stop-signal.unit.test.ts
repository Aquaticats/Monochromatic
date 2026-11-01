/**
 Tests for how the shipper of a stopped entry answers a stop signal (B1247): the reason of the record names the signal
 and not the entry's hard ceiling, and the flight the entry registered is told once the shipper is done, however it
 ended.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readInterimEntries, } from '@monochromatic-dev/translation-repair-glossary';
import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import {
  createPassStop,
  shipInterimOnStop,
} from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  dirsOf,
  DIGEST,
  ENTRY_ID,
  pageOf,
  slicedProgress,
  SOURCE_DOC,
  TARGET_DOC,
} from './pass-entry-stop.test-fixture.ts';

/**
 What the pipeline's stages throw when the entry's signal aborts.
 */
const ABORT = new DOMException(
  'the signal was aborted',
  'AbortError',
);

/**
 Ships over a scratch tree with a stop and a flight, the arguments every case shares.

 @param dirs - scratch tree

 @param error - what stopped the entry

 @param stop - stop the pass shares with its entries, whose signal the shipper is told where `signalled`

 @param signalled - whether the shipper is told the stop signal, as the pipeline tells it

 @returns Whether the flight was told its page was shipped

 @example
 ```ts
 const told = await shipUnderStop({ dirs, error: ABORT, stop, signalled: true, },);
 ```
 */
async function shipUnderStop(
  {
    dirs,
    error,
    stop,
    signalled,
  }: {
    readonly dirs: ReturnType<typeof dirsOf>;
    readonly error: unknown;
    readonly stop: ReturnType<typeof createPassStop>;
    readonly signalled: boolean;
  },
): Promise<boolean> {
  /**
   Whether the shipper has told the flight.
   */
  const told: { current: boolean; } = { current: false, };
  await shipInterimOnStop({
    error,
    progress: slicedProgress(),
    entryId: ENTRY_ID,
    sourceText: SOURCE_DOC,
    targetText: TARGET_DOC,
    publishDir: dirs.publishDir,
    interimDir: dirs.interimDir,
    tip: 'a'.repeat(40,),
    pipelineDigest: DIGEST,
    corpusSha: 'b'.repeat(40,),
    attempt: 1,
    ...(signalled ? { stopSignal: stop.signal, } : {}),
    flight: {
      shipped: function shipped(): void {
        told.current = true;
      },
      [Symbol.dispose](): void {},
    },
    l: capturingLogger({ messages: [], },),
  },);
  return told.current;
}

await describe({
  name: 'the shipper of a stopped entry under a stop signal',
  children: [
    it({
      name: 'NAMES THE STOP SIGNAL AS THE REASON when the pass\'s stop is aborted, keeping the class AbortError the stages '
        + 'threw, and ships the page',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-stop-signal-', },);
        const dirs = dirsOf({ root: root.path, },);
        const stop = createPassStop();
        stop.request();
        await shipUnderStop({
          dirs,
          error: ABORT,
          stop,
          signalled: true,
        },);
        /**
         The record the stop left.
         */
        const record = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,);
        expect({ reason: record?.reason, errorClass: record?.errorClass, },).toEqual({
          reason: 'stop-signal',
          errorClass: 'AbortError',
        },);
        expect(await pageOf({ dirs, },),).toBe(TARGET_DOC,);
      },
    },),

    it({
      name: 'KEEPS THE REASON entry-error FOR THE SAME ABORT WHEN NO STOP WAS REQUESTED, the control: the hard ceiling of an '
        + 'entry aborts its signal too, and a record that called it a stop signal would blame the operator',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-stop-signal-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipUnderStop({
          dirs,
          error: ABORT,
          stop: createPassStop(),
          signalled: true,
        },);
        expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,)?.reason,)
          .toBe('entry-error',);
      },
    },),

    it({
      name: 'KEEPS THE REASON dry-stop FOR A RUN-ENDING REFUSAL THAT MET A REQUESTED STOP, since the refusal ends the run '
        + 'whatever else was asked of it',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-stop-signal-', },);
        const dirs = dirsOf({ root: root.path, },);
        const stop = createPassStop();
        stop.request();
        await shipUnderStop({
          dirs,
          error: new EveryProviderDryError({ measured: 'the meters of the cases read dry', },),
          stop,
          signalled: true,
        },);
        expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,)?.reason,)
          .toBe('dry-stop',);
      },
    },),

    it({
      name: 'TELLS THE FLIGHT ITS PAGE IS SHIPPED when the page was written, so the signal handler stops waiting',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-stop-signal-', },);
        const dirs = dirsOf({ root: root.path, },);
        const stop = createPassStop();
        stop.request();
        expect(await shipUnderStop({
          dirs,
          error: ABORT,
          stop,
          signalled: true,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'TELLS THE FLIGHT EVEN WHEN THE PAGE COULD NOT BE SHIPPED, since the handler must not wait out its window for '
        + 'a page that will never come',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-stop-signal-', },);
        const dirs = dirsOf({ root: root.path, },);
        // A published tree that is a file, so no page can be made under it.
        await writeFile(
          dirs.publishDir,
          'not a directory',
        );
        const stop = createPassStop();
        stop.request();
        expect(await shipUnderStop({
          dirs,
          error: ABORT,
          stop,
          signalled: true,
        },),).toBe(true,);
      },
    },),
  ],
},);
