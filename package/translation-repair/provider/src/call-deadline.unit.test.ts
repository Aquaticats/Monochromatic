/**
 Tests for the per-call deadline handle:
 expiry aborts with a labeled timeout, caller aborts always win and
 keep their reason, and disposal defuses both the timer and the
 caller-abort listener.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { armCallDeadline, } from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Deadline short enough to expire inside a test.
 */
const SHORT_DEADLINE_MS = 20;

/**
 Comfortable margin past the short deadline.
 */
const PAST_DEADLINE_MS = SHORT_DEADLINE_MS * 3;

await describe({
  name: armCallDeadline.name,
  children: [
    it({
      name: 'aborts the call with a labeled timeout at its deadline',
      fn: async () => {
        const caller = new AbortController();
        using deadline = armCallDeadline({
          signal: caller.signal,
          timeoutMs: SHORT_DEADLINE_MS,
          label: 'cat-model',
        },);
        await wait(PAST_DEADLINE_MS,);
        expect(deadline.callSignal.aborted,).toBe(true,);
        expect(String(deadline.callSignal.reason,),).toContain(
          `Timeout: cat-model exceeded its ${String(SHORT_DEADLINE_MS,)}ms deadline`,
        );
      },
    },),
    it({
      name: 'forwards a caller abort, keeping the caller reason',
      fn: async () => {
        const caller = new AbortController();
        using deadline = armCallDeadline({
          signal: caller.signal,
          timeoutMs: HANG_STOP_MS,
          label: 'cat-model',
        },);
        caller.abort(new Error('user steered away',),);
        expect(deadline.callSignal.aborted,).toBe(true,);
        expect(String(deadline.callSignal.reason,),).toContain('user steered away',);
      },
    },),
    it({
      name: 'aborts immediately for a caller already aborted at arming',
      fn: async () => {
        const caller = new AbortController();
        caller.abort(new Error('aborted before arming',),);
        using deadline = armCallDeadline({
          signal: caller.signal,
          timeoutMs: HANG_STOP_MS,
          label: 'cat-model',
        },);
        expect(deadline.callSignal.aborted,).toBe(true,);
        expect(String(deadline.callSignal.reason,),).toContain('aborted before arming',);
      },
    },),
    it({
      name: 'disposal defuses the timer and detaches the abort listener',
      fn: async () => {
        const caller = new AbortController();

        /**
         Signal kept alive past disposal to observe non-abort.
         */
        const { callSignal, } = (function armAndDispose() {
          using deadline = armCallDeadline({
            signal: caller.signal,
            timeoutMs: SHORT_DEADLINE_MS,
            label: 'cat-model',
          },);
          return { callSignal: deadline.callSignal, };
        })();

        await wait(PAST_DEADLINE_MS,);
        caller.abort(new Error('after disposal',),);
        expect(callSignal.aborted,).toBe(false,);
      },
    },),
  ],
},);
