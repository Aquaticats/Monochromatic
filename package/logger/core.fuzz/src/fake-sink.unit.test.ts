import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { LogRecord, } from '@monochromatic-dev/logger-core';

import {
  createScriptedSink,
  formatSinkScript,
  identityGate,
  outcomeAt,
  type SinkScript,
  type SinkTraceEvent,
} from './fake-sink.ts';

/**
 Record used by every write in this file.
 */
const RECORD: LogRecord = {
  level: 'info',
  message: 'hello',
  timestamp: 1,
};

/**
 Script whose first write rejects and whose flush hook always hangs.
 */
const RECOVERING: SinkScript = {
  flush: {
    head: [],
    tail: 'never',
  },
  verify: {
    head: [],
    tail: 'resolve-true',
  },
  write: {
    head: ['reject',],
    tail: 'resolve',
  },
};

/**
 Awaits a promise and reports whether it rejected, so a scripted rejection
 can be asserted without `.catch`.

 @param promise - Promise to settle.

 @returns Whether it rejected.
 */
async function rejected({ promise, }: { readonly promise: Promise<unknown>; },): Promise<boolean> {
  try {
    await promise;
    return false;
  }
  catch (error: unknown) {
    return Error.isError(error,);
  }
}

await describe({
  name: 'scripted fake sink',
  children: [
    it({
      name: 'outcomeAt reads the head in order and then the tail forever',
      fn: async () => {
        expect(outcomeAt({
          script: RECOVERING.write,
          callIndex: 0,
        },),)
          .toBe('reject',);
        expect(outcomeAt({
          script: RECOVERING.write,
          callIndex: 1,
        },),)
          .toBe('resolve',);
        expect(outcomeAt({
          script: RECOVERING.write,
          callIndex: 40,
        },),)
          .toBe('resolve',);
      },
    },),

    it({
      name: 'formats a script as one readable line naming the sink',
      fn: async () => {
        expect(formatSinkScript({
          index: 2,
          script: RECOVERING,
        },),)
          .toBe('sink 2: verify [resolve-true*] write [reject, resolve*] flush [never*]',);
        expect(formatSinkScript({
          index: 0,
          script: {
            verify: RECOVERING.verify,
            write: RECOVERING.write,
          },
        },),)
          .toBe('sink 0: verify [resolve-true*] write [reject, resolve*] flush absent',);
      },
    },),

    it({
      name: 'write follows the script per call and tracks attempts and deliveries',
      fn: async () => {
        const trace: SinkTraceEvent[] = [];
        const fake = createScriptedSink({
          index: 0,
          script: RECOVERING,
          schedule: identityGate,
          trace,
        },);
        expect(await rejected({ promise: fake.sink.write(RECORD,), },),)
          .toBe(true,);
        await fake.sink.write(RECORD,);
        expect(fake.attempts,)
          .toHaveLength(2,);
        expect(fake.delivered,)
          .toHaveLength(1,);
        expect(trace.filter(function isWrite(event,) {
          return event.hook === 'write';
        },).map(function toPhase(event,) {
          return `${event.phase}:${event.callIndex}:${event.outcome}`;
        },),)
          .toEqual([
            'called:0:reject',
            'settled:0:reject',
            'called:1:resolve',
            'settled:1:resolve',
          ],);
      },
    },),

    it({
      name: 'verify resolves the scripted boolean and a never outcome stays pending',
      fn: async () => {
        const trace: SinkTraceEvent[] = [];
        const fake = createScriptedSink({
          index: 1,
          script: RECOVERING,
          schedule: identityGate,
          trace,
        },);
        expect(await fake.sink.verify(),)
          .toBe(true,);
        /**
         Race between the never-settling flush hook and a settled marker.
         */
        const winner = await Promise.race([
          fake.sink.flush?.(),
          Promise.resolve('marker',),
        ],);
        expect(winner,)
          .toBe('marker',);
        expect(trace.filter(function isFlush(event,) {
          return event.hook === 'flush';
        },),)
          .toHaveLength(1,);
      },
    },),

    it({
      name: 'a throw outcome throws synchronously and leaves no settled event',
      fn: async () => {
        const trace: SinkTraceEvent[] = [];
        const fake = createScriptedSink({
          index: 0,
          script: {
            verify: {
              head: ['throw',],
              tail: 'resolve-true',
            },
            write: RECOVERING.write,
          },
          schedule: identityGate,
          trace,
        },);
        expect(function callVerify() {
          return fake.sink.verify();
        },)
          .toThrow('scripted verify throw',);
        expect(trace,)
          .toHaveLength(1,);
        expect(fake.sink.flush,)
          .toBeUndefined();
      },
    },),

    it({
      name: 'the schedule gate decides when a settleable outcome reaches the caller',
      fn: async () => {
        /**
         Gate that parks every outcome until released.
         */
        const held = Promise.withResolvers<void>();
        /**
         Outcomes the gate has captured, in order.
         */
        const labels: string[] = [];
        const trace: SinkTraceEvent[] = [];
        const fake = createScriptedSink({
          index: 3,
          script: RECOVERING,
          schedule: async function gate<Value,>(
            {
              promise,
              label,
            }: {
              readonly promise: Promise<Value>;
              readonly label: string;
            },
          ): Promise<Value> {
            labels.push(label,);
            await held.promise;
            return await promise;
          },
          trace,
        },);
        /**
         Verify call parked behind the gate.
         */
        const verifying = fake.sink.verify();
        expect(labels,)
          .toEqual(['sink 3 verify #0',],);
        expect(trace.map(function toPhase(event,) {
          return event.phase;
        },),)
          .toEqual(['called',],);
        held.resolve();
        expect(await verifying,)
          .toBe(true,);
        expect(trace.map(function toPhase(event,) {
          return event.phase;
        },),)
          .toEqual([
            'called',
            'settled',
          ],);
      },
    },),
  ],
},);
