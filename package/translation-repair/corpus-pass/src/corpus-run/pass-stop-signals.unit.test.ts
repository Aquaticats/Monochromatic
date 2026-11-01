/**
 Tests for what a pass does when it is told to stop (ledger B1113): it records that it stopped and which signal said so,
 flushes its run log, releases the lock of its runs directory and exits with the conventional code, 128 and the
 signal's number.

 THE ORDER IS WHAT MATTERS: the record is made before the flush, the flush before the release, and the exit last, so the
 folder holds the reason and the lock is free for the next pass. The in-process cases hold the order with a stand-in
 for the process's signals; the child cases send a real signal to a child the case started, and to nothing else.

 Every folder is written under a scratch directory. Fixtures are cat-themed invention.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setImmediate, } from 'node:timers/promises';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readInterimEntries,
  readRunLogText,
} from '@monochromatic-dev/translation-repair-glossary';
import { fixedPagePath, } from '@monochromatic-dev/translation-repair-page-check';
import {
  installRunLog,
  openRunLog,
} from '@monochromatic-dev/translation-repair-typography';
import {
  runUntilSignalled,
  type SignalledRun,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  createPassStop,
  handlingStopSignals,
} from '../../dist/final/node/index.mjs';
import {
  ENTRY_ID,
  SETTLED_FIRST,
} from './pass-entry-stop.test-fixture.ts';

/**
 Program the child cases run.
 */
const CHILD_PROGRAM = fileURLToPath(new URL(
  'pass-stop-signals-child.test-fixture.ts',
  import.meta.url,
),);

/**
 Program the child case that holds an entry in flight with a settled slice runs.
 */
const SHIP_CHILD_PROGRAM = fileURLToPath(new URL(
  'pass-stop-ship-child.test-fixture.ts',
  import.meta.url,
),);

/**
 Stand-in for the process's signals: handlers registered by name, and a way to deliver a signal to them.
 */
type FakeSignals = {
  /**
   Registers a handler for a signal.
   */
  readonly on: (
    signal: 'SIGINT' | 'SIGTERM',
    handler: () => void,
  ) => void;

  /**
   Takes a handler off.
   */
  readonly off: (
    signal: 'SIGINT' | 'SIGTERM',
    handler: () => void,
  ) => void;

  /**
   Delivers a signal to every handler registered for it.
   */
  readonly emit: (signal: 'SIGINT' | 'SIGTERM',) => void;

  /**
   How many handlers are registered for a signal.
   */
  readonly count: (signal: 'SIGINT' | 'SIGTERM',) => number;
};

/**
 Makes a stand-in for the process's signals.

 @returns Stand-in that registers, removes and delivers

 @example
 ```ts
 const source = fakeSignals();
 ```
 */
function fakeSignals(): FakeSignals {
  /**
   Handlers registered, by signal.
   */
  const registered = new Map<string, Set<() => void>>();

  /**
   Handlers registered for a signal, made on first use.
   */
  function handlersOf({ signal, }: { readonly signal: string; },): Set<() => void> {
    /**
     The set already made for it.
     */
    const made = registered.get(signal,);
    if (made !== undefined)
      return made;
    /**
     The set made now.
     */
    const fresh = new Set<() => void>();
    registered.set(
      signal,
      fresh,
    );
    return fresh;
  }
  return {
    on: function on(signal, handler,): void {
      handlersOf({ signal, },).add(handler,);
    },
    off: function off(signal, handler,): void {
      handlersOf({ signal, },).delete(handler,);
    },
    emit: function emit(signal,): void {
      for (const handler of handlersOf({ signal, },))
        handler();
    },
    count: function count(signal,): number {
      return handlersOf({ signal, },).size;
    },
  };
}

await describe({
  name: handlingStopSignals.name,
  concurrency: 1,
  children: [
    it({
      name: 'RECORDS THAT THE PASS STOPPED AND ON WHICH SIGNAL, flushes the log, releases the lock and then exits with 128 '
        + 'and the signal\'s number, in that order, for SIGTERM and for SIGINT',
      fn: async (ctx) => {
        ctx.sinon.stub(
          console,
          'error',
        );
        await using scratch = await scratchDir({ prefix: 'pass-stop-signals-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);
        /**
         Stand-in for the process's signals.
         */
        const source = fakeSignals();
        /**
         What the folder held when the lock was released, and the codes exited with.
         */
        const seen: { atRelease: string; readonly exits: number[]; releases: number; } = {
          atRelease: '',
          exits: [],
          releases: 0,
        };
        const exited = Promise.withResolvers<void>();
        using _signals = handlingStopSignals({
          log: runLog,
          release: async function release(): Promise<void> {
            seen.releases += 1;
            seen.atRelease = await readRunLogText({ path: scratch.path, },);
          },
          source,
          exit: function exit(code: number,): void {
            seen.exits.push(code,);
            exited.resolve();
          },
        },);

        source.emit('SIGTERM',);
        // A second signal while the pass is stopping changes nothing.
        source.emit('SIGINT',);
        await exited.promise;

        expect(seen.exits,).toEqual([143,],);
        expect(seen.releases,).toBe(1,);
        expect(seen.atRelease,).toContain('] [corpus-pass] the pass stopped: SIGTERM arrived',);
        expect(seen.atRelease,).not.toContain('SIGINT',);
      },
    },),

    it({
      name: 'EXITS 130 FOR SIGINT, and takes its handlers off when the pass ends, so a later signal is the process\'s own',
      fn: async (ctx) => {
        ctx.sinon.stub(
          console,
          'error',
        );
        await using scratch = await scratchDir({ prefix: 'pass-stop-signals-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);
        const source = fakeSignals();
        const exits: number[] = [];
        const exited = Promise.withResolvers<void>();
        {
          using _signals = handlingStopSignals({
            log: runLog,
            release: async function release(): Promise<void> {},
            source,
            exit: function exit(code: number,): void {
              exits.push(code,);
              exited.resolve();
            },
          },);
          expect([source.count('SIGINT',), source.count('SIGTERM',),],).toEqual([1, 1,],);
          source.emit('SIGINT',);
          await exited.promise;
        }

        expect(exits,).toEqual([130,],);
        expect([source.count('SIGINT',), source.count('SIGTERM',),],).toEqual([0, 0,],);
      },
    },),

    it({
      name: 'ABORTS THE ENTRY IN FLIGHT AND WAITS FOR ITS INTERIM PAGE (B1247): the stop signal aborts the signal the entry '
        + 'runs under, and the flush, the release and the exit come only after the entry says its page is shipped',
      fn: async (ctx) => {
        ctx.sinon.stub(
          console,
          'error',
        );
        await using scratch = await scratchDir({ prefix: 'pass-stop-signals-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);
        const source = fakeSignals();
        const stop = createPassStop();
        /**
         What happened, in order.
         */
        const order: string[] = [];
        stop.signal
          .addEventListener(
            'abort',
            function aborted(): void {
              order.push('aborted',);
            },
          );
        const exited = Promise.withResolvers<void>();
        using flight = stop.begin();
        using _signals = handlingStopSignals({
          log: runLog,
          release: async function release(): Promise<void> {
            order.push('release',);
          },
          stop,
          source,
          exit: function exit(code: number,): void {
            order.push(`exit ${String(code,)}`,);
            exited.resolve();
          },
        },);

        source.emit('SIGTERM',);
        // The entry has not shipped yet, so the handler is still waiting on it.
        await setImmediate();
        expect(order,).toEqual(['aborted',],);
        order.push('shipped',);
        flight.shipped();
        await exited.promise;

        expect(order,).toEqual(['aborted', 'shipped', 'release', 'exit 143',],);
      },
    },),

    it({
      name: 'EXITS WHEN THE WINDOW ENDS (B1247): an entry that never says it shipped holds the stop for the window and no '
        + 'longer, and the run log says the page was not shipped in time',
      fn: async (ctx) => {
        ctx.sinon.stub(
          console,
          'error',
        );
        await using scratch = await scratchDir({ prefix: 'pass-stop-signals-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);
        const source = fakeSignals();
        const stop = createPassStop();
        const exited = Promise.withResolvers<void>();
        const seen: { atRelease: string; readonly exits: number[]; } = {
          atRelease: '',
          exits: [],
        };
        using _flight = stop.begin();
        using _signals = handlingStopSignals({
          log: runLog,
          release: async function release(): Promise<void> {
            seen.atRelease = await readRunLogText({ path: scratch.path, },);
          },
          stop,
          windowMs: 30,
          source,
          exit: function exit(code: number,): void {
            seen.exits.push(code,);
            exited.resolve();
          },
        },);

        source.emit('SIGINT',);
        await exited.promise;

        expect(seen.exits,).toEqual([130,],);
        expect(seen.atRelease,).toContain('the entry in flight did not ship its interim page within 30 ms',);
      },
    },),

    it({
      name: 'EXITS AT ONCE ON A SECOND SIGNAL WHILE IT WAITS (B1247), as B1113 exits: the window is long, the entry never '
        + 'ships, and the second signal ends the wait with the first signal\'s code',
      fn: async (ctx) => {
        ctx.sinon.stub(
          console,
          'error',
        );
        await using scratch = await scratchDir({ prefix: 'pass-stop-signals-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);
        const source = fakeSignals();
        const stop = createPassStop();
        const exited = Promise.withResolvers<void>();
        const seen: { atRelease: string; readonly exits: number[]; } = {
          atRelease: '',
          exits: [],
        };
        using _flight = stop.begin();
        using _signals = handlingStopSignals({
          log: runLog,
          release: async function release(): Promise<void> {
            seen.atRelease = await readRunLogText({ path: scratch.path, },);
          },
          stop,
          windowMs: 600_000,
          source,
          exit: function exit(code: number,): void {
            seen.exits.push(code,);
            exited.resolve();
          },
        },);

        source.emit('SIGTERM',);
        await setImmediate();
        source.emit('SIGINT',);
        await exited.promise;

        expect(seen.exits,).toEqual([143,],);
        expect(seen.atRelease,).toContain('a second signal arrived',);
      },
    },),

    it({
      name: 'DOES NOT WAIT WHEN NO ENTRY IS IN FLIGHT (B1247): a signal between entries, or before the first, exits as '
        + 'B1113 does, with the signal aborted',
      fn: async (ctx) => {
        ctx.sinon.stub(
          console,
          'error',
        );
        await using scratch = await scratchDir({ prefix: 'pass-stop-signals-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);
        const source = fakeSignals();
        const stop = createPassStop();
        const exited = Promise.withResolvers<void>();
        const exits: number[] = [];
        using _signals = handlingStopSignals({
          log: runLog,
          release: async function release(): Promise<void> {},
          stop,
          windowMs: 600_000,
          source,
          exit: function exit(code: number,): void {
            exits.push(code,);
            exited.resolve();
          },
        },);

        source.emit('SIGTERM',);
        await exited.promise;

        expect(exits,).toEqual([143,],);
        expect(stop.signal.aborted,).toBe(true,);
      },
    },),

    it({
      name: 'SHIPS THE INTERIM PAGE OF THE ENTRY IN FLIGHT BEFORE A REAL PASS-SHAPED PROCESS EXITS ON SIGTERM (B1247): '
        + 'the child holds an entry with a settled slice, and after the signal its folder holds the interim page, the '
        + 'record that names the stop signal, and the exit code 143',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-stop-ship-', },);
        const done: SignalledRun = await runUntilSignalled({
          file: process.execPath,
          args: [SHIP_CHILD_PROGRAM, scratch.path,],
          signal: 'SIGTERM',
        },);

        expect([done.code, done.signal, done.stdout.trim(),],).toEqual([143, 'none', 'ready',],);
        /**
         The record the stop left.
         */
        const record = (await readInterimEntries({ interimDir: join(scratch.path, 'interim',), },)).get(ENTRY_ID,);
        expect({
          reason: record?.reason,
          errorClass: record?.errorClass,
          settledSlices: record?.settledSlices,
        },).toEqual({
          reason: 'stop-signal',
          errorClass: 'AbortError',
          settledSlices: 1,
        },);
        expect(await readFile(
          fixedPagePath({
            publishDir: join(scratch.path, 'fixed',),
            entryId: ENTRY_ID,
          },),
          'utf8',
        ),).toBe(`${SETTLED_FIRST}\n## Section two\n\nThe cat has a bowl.\n`,);
        expect(await readRunLogText({ path: scratch.path, },),).toContain(
          'the entry in flight shipped its interim page',
        );
        expect((await readdir(scratch.path,)).includes('pass.lock',),).toBe(false,);
      },
    },),

    it({
      name: 'ENDS A REAL PASS-SHAPED PROCESS ON SIGTERM AND ON SIGINT: it exits 143 and 130, its run folder holds the record '
        + 'naming the signal, and its lock is gone, for a child the case started and signalled itself',
      fn: async () => {
        for (const [signal, code,] of [['SIGTERM', 143,], ['SIGINT', 130,],] as const) {
          // oxlint-disable-next-line no-await-in-loop -- one child after the other, each with its own folder
          await using scratch = await scratchDir({ prefix: 'pass-stop-signals-', },);
          // oxlint-disable-next-line no-await-in-loop -- the case reads this child's end before it starts the next
          const done: SignalledRun = await runUntilSignalled({
            file: process.execPath,
            args: [CHILD_PROGRAM, scratch.path,],
            signal,
          },);

          expect([done.code, done.signal, done.stdout.trim(),],).toEqual([code, 'none', 'ready',],);
          // oxlint-disable-next-line no-await-in-loop -- read after the child ended
          expect(await readRunLogText({ path: scratch.path, },),).toContain(
            `] [corpus-pass] the pass stopped: ${signal} arrived`,
          );
          // oxlint-disable-next-line no-await-in-loop -- read after the child ended
          expect((await readdir(scratch.path,)).includes('pass.lock',),).toBe(false,);
        }
      },
    },),
  ],
},);
