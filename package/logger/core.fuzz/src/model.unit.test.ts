import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { LogRecord, } from '@monochromatic-dev/logger-core';

import type { SinkScript, } from './fake-sink.ts';
import {
  foldTrace,
  type ModelEvent,
} from './model.ts';

/**
 Builds a record with a distinct message.

 @param message - Message text.

 @returns Info record.
 */
function record(message: string,): LogRecord {
  return {
    level: 'info',
    message,
    timestamp: 0,
  };
}

/**
 Sink whose every hook succeeds and which has a flush hook.
 */
const GOOD: SinkScript = {
  flush: {
    head: [],
    tail: 'resolve',
  },
  verify: {
    head: [],
    tail: 'resolve-true',
  },
  write: {
    head: [],
    tail: 'resolve',
  },
};

/**
 Sink whose verify rejects.
 */
const BROKEN: SinkScript = {
  verify: {
    head: [],
    tail: 'reject',
  },
  write: GOOD.write,
};

/**
 Startup buffer cap used by every fold in this file.
 */
const CAP = 3;

/**
 Maps a sink model's attempts to their messages.

 @param messages - Records to map.

 @returns Message list.
 */
function texts(messages: readonly LogRecord[],): string[] {
  return messages.map(function toMessage(entry,) {
    return entry.message;
  },);
}

await describe({
  name: 'reference model',
  children: [
    it({
      name: 'a record logged before verify is replayed once, one logged after is written immediately',
      fn: async () => {
        const state = foldTrace({
          scripts: [GOOD, GOOD,],
          cap: CAP,
          events: [
            { kind: 'log', record: record('early',), },
            { kind: 'verify-settled', sink: 0, outcome: 'resolve-true', },
            { kind: 'log', record: record('mid',), },
            { kind: 'verify-settled', sink: 1, outcome: 'resolve-true', },
            { kind: 'init-complete', },
            { kind: 'log', record: record('late',), },
          ],
        },);
        expect(texts(state.sinks[0]?.attempts ?? [],),)
          .toEqual([
            'early',
            'mid',
            'late',
          ],);
        expect(texts(state.sinks[1]?.attempts ?? [],),)
          .toEqual([
            'early',
            'mid',
            'late',
          ],);
        expect(state.breadcrumbs,)
          .toBe(0,);
        expect(state.markerDue,)
          .toBe(false,);
      },
    },),

    it({
      name: 'a failed verify gets nothing and costs one breadcrumb',
      fn: async () => {
        const state = foldTrace({
          scripts: [BROKEN, GOOD,],
          cap: CAP,
          events: [
            { kind: 'log', record: record('early',), },
            { kind: 'verify-settled', sink: 0, outcome: 'reject', },
            { kind: 'verify-settled', sink: 1, outcome: 'resolve-true', },
            { kind: 'init-complete', },
          ],
        },);
        expect(state.sinks[0]?.attempts,)
          .toHaveLength(0,);
        expect(state.sinks[0]?.available,)
          .toBe(false,);
        expect(texts(state.sinks[1]?.attempts ?? [],),)
          .toEqual(['early',],);
        expect(state.breadcrumbs,)
          .toBe(1,);
      },
    },),

    it({
      name: 'overflowing the startup buffer drops the oldest and makes the marker due',
      fn: async () => {
        const state = foldTrace({
          scripts: [GOOD,],
          cap: CAP,
          events: [
            { kind: 'log', record: record('a',), },
            { kind: 'log', record: record('b',), },
            { kind: 'log', record: record('c',), },
            { kind: 'log', record: record('d',), },
            { kind: 'verify-settled', sink: 0, outcome: 'resolve-true', },
            { kind: 'init-complete', },
          ],
        },);
        expect(state.dropped,)
          .toBe(1,);
        expect(state.markerDue,)
          .toBe(true,);
        expect(texts(state.sinks[0]?.attempts ?? [],),)
          .toEqual([
            'b',
            'c',
            'd',
            `1 startup record dropped before a backend verified (buffer cap ${CAP})`,
          ],);
      },
    },),

    it({
      name: 'write outcomes are read by call index: a rejecting first write costs a breadcrumb',
      fn: async () => {
        const state = foldTrace({
          scripts: [{
            ...GOOD,
            write: {
              head: ['reject',],
              tail: 'resolve',
            },
          },],
          cap: CAP,
          events: [
            { kind: 'verify-settled', sink: 0, outcome: 'resolve-true', },
            { kind: 'init-complete', },
            { kind: 'log', record: record('first',), },
            { kind: 'log', record: record('second',), },
          ],
        },);
        expect(state.sinks[0]?.attempts,)
          .toHaveLength(2,);
        expect(state.breadcrumbs,)
          .toBe(1,);
      },
    },),

    it({
      name: 'a hanging write makes flush miss its deadline without reaching the hooks; a later rejecting hook retires the sink',
      fn: async () => {
        /**
         Sink whose first write hangs and whose flush hook rejects on its second call.
         */
        const script: SinkScript = {
          flush: {
            head: [
              'resolve',
              'reject',
            ],
            tail: 'resolve',
          },
          verify: GOOD.verify,
          write: {
            head: ['never',],
            tail: 'resolve',
          },
        };
        const events: ModelEvent[] = [
          { kind: 'verify-settled', sink: 0, outcome: 'resolve-true', },
          { kind: 'init-complete', },
          { kind: 'log', record: record('stuck',), },
          // Hangs on the drain: no hook call, deadline breadcrumb, writes abandoned.
          { kind: 'flush', },
          // Drain empty: first hook call resolves.
          { kind: 'flush', },
          { kind: 'log', record: record('still available',), },
          // Second hook call rejects: retire plus breadcrumb.
          { kind: 'flush', },
          { kind: 'log', record: record('after retire',), },
        ];
        const state = foldTrace({
          scripts: [script,],
          cap: CAP,
          events,
        },);
        expect(state.flushesWithinDeadline,)
          .toEqual([
            false,
            true,
            true,
          ],);
        expect(state.sinks[0]?.available,)
          .toBe(false,);
        expect(texts(state.sinks[0]?.attempts ?? [],),)
          .toEqual([
            'stuck',
            'still available',
          ],);
        // One deadline breadcrumb, one flush-hook rejection breadcrumb.
        expect(state.breadcrumbs,)
          .toBe(2,);
      },
    },),
  ],
},);
