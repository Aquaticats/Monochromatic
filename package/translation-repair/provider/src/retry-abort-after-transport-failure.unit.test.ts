/**
 Tests the retry ladder when the caller aborts while it sleeps a backoff after
 a transport failure, where no reply stands to be reported and the failure
 itself is what the ladder surfaces. The cases of a rate-limit reply and of an
 abort tripped by the transport itself are in `transient-retry.unit.test.ts`.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream';
import { exchangeWithRetry, } from '../dist/final/node/index.mjs';

/**
 Retry policy whose first backoff is long enough that sleeping it out would be
 visible against a caller abort: equal jitter sleeps at least half the
 attempt's window, and the first window is the base.
 */
const SLOW_POLICY = {
  limit: 1,
  baseMs: 20_000,
};

await describe({
  name: exchangeWithRetry.name,
  children: [
    it({
      name: 'SURFACES THE TRANSPORT FAILURE WHEN THE CALLER ABORTS DURING THE BACKOFF AFTER IT, not an HTTP '
        + 'error it never received, and asks the transport once',
      fn: async () => {
        /**
         Attempts the transport has seen.
         */
        const calls = { count: 0, };
        /**
         Caller's abort, tripped on the next turn of the event loop.
         */
        const controller = new AbortController();
        /**
         Exactly the failure the transport raised, held so the assertion proves
         the ladder surfaced that object.
         */
        const transportFailure = new Error('connection reset',);
        /**
         Transport dropping the connection, the caller giving up once the
         ladder has begun sleeping its backoff.
         */
        const transport: ModelTransport = async () => {
          calls.count += 1;
          setTimeout(function abortDuringBackoff() {
            controller.abort();
          }, 0,);
          throw transportFailure;
        };

        await expect(
          exchangeWithRetry({
            provider: 'synthetic',
            transport,
            exchange: {
              url: 'https://example.invalid/chat',
              label: 'hf:whiskers',
              method: 'POST',
              headers: { 'content-type': 'application/json', },
              bodyJson: JSON.stringify({ prompt: 'Does the cat purr?', },),
              signal: controller.signal,
            },
            policy: SLOW_POLICY,
          },),
        ).rejects.toBe(transportFailure,);
        expect(calls.count,).toBe(1,);
      },
    },),
  ],
},);
