/**
 Tests for the stream bound of class one hundred forty-eight: the bound cuts
 a call with a reason the router can read through any cause chain, and the
 caller's own abort still wins. Cat-themed invention throughout.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StreamCutShortError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  armStreamBound,
  isStreamBoundCut,
  StreamBoundError,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 A bound short enough for a test to wait out.
 */
const SHORT_BOUND_MS = 20;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: armStreamBound.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CUTS the call at the bound with a StreamBoundError naming the model and the bound',
          fn: async () => {
            using bound = armStreamBound({
              signal: new AbortController().signal,
              boundMs: SHORT_BOUND_MS,
              label: 'cats/whisker-1',
            },);
            await wait(SHORT_BOUND_MS * 2,);
            expect(bound.callSignal.aborted,).toBe(true,);
            expect(bound.callSignal.reason instanceof StreamBoundError,).toBe(true,);
            expect((bound.callSignal.reason as StreamBoundError).boundMs,).toBe(SHORT_BOUND_MS,);
          },
        },),
        it({
          name: 'FORWARDS the caller\'s abort with its own reason, which is not a bound cut',
          fn: async () => {
            const caller = new AbortController();
            using bound = armStreamBound({
              signal: caller.signal,
              boundMs: HANG_STOP_MS,
              label: 'cats/whisker-1',
            },);
            caller.abort(new Error('the cat left the room',),);
            expect(bound.callSignal.aborted,).toBe(true,);
            expect(isStreamBoundCut({ error: bound.callSignal.reason, },),).toBe(false,);
          },
        },),
        it({
          name: 'FORWARDS AN ALREADY-ABORTED CALLER SIGNAL IMMEDIATELY, without waiting on the '
            + '\'abort\' event: a retry ladder can hand this a signal that aborted before the next '
            + 'attempt was even armed',
          fn: async () => {
            const caller = new AbortController();
            caller.abort(new Error('the cat left before the call',),);
            using bound = armStreamBound({
              signal: caller.signal,
              boundMs: HANG_STOP_MS,
              label: 'cats/whisker-1',
            },);
            expect(bound.callSignal.aborted,).toBe(true,);
            expect(isStreamBoundCut({ error: bound.callSignal.reason, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isStreamBoundCut.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a bound cut through the stream-cut wrapper the drain throws, and nothing else as one',
          fn: async () => {
            const cut = new StreamCutShortError({
              label: 'cats/whisker-1',
              partialText: '',
              progress: {
                firstByteMs: 15_000,
                firstContentMs: 15_000,
                maxGapMs: 15_000,
                chars: 30,
                elapsedMs: SHORT_BOUND_MS,
              },
              cause: new StreamBoundError({
                label: 'cats/whisker-1',
                boundMs: SHORT_BOUND_MS,
              },),
            },);
            expect(isStreamBoundCut({ error: cut, },),).toBe(true,);
            expect(isStreamBoundCut({ error: new Error('the bowl is empty',), },),).toBe(false,);
            expect(isStreamBoundCut({ error: 'not an error', },),).toBe(false,);
          },
        },),
        it({
          name: 'ENDS on a cause chain that loops back on itself',
          fn: async () => {
            const first = new Error('first nap',);
            const second = new Error('second nap', { cause: first, },);
            first.cause = second;
            expect(isStreamBoundCut({ error: first, },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
