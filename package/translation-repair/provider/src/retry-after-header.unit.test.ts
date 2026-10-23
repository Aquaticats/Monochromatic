/**
 Tests the `retry-after` header and status 529 on the retry ladder, which the Claude route needs: Anthropic's 429
 and 529 replies name their wait in a `retry-after` header in seconds, and 529 is its overloaded status. The body
 phrase the other providers use is exercised in `retry-after-ms.unit.test.ts`; the ladder as a whole in
 `transient-retry.unit.test.ts`. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  ModelTransport,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  exchangeWithRetry,
  retryAfterHeaderMsOf,
} from '../dist/final/node/index.mjs';

/**
 Retry policy whose reach, the longest stated wait it sleeps in one backoff, is 1.2 s: it covers a one-second
 `retry-after` and not a thirty-second one.
 */
const WAIT_POLICY = {
  limit: 2,
  baseMs: 300,
};

/**
 Reply every ladder in these cases ends on when it succeeds.
 */
const OK_REPLY: TransportReply = {
  status: 200,
  bodyText: 'purr',
};

/**
 Transport answering a script in order, counting its calls.

 @param script - reply for each attempt; the last repeats

 @param calls - counter the cases read

 @returns Transport honoring the script

 @example
 ```ts
 const transport = answering({ script: [OK_REPLY,], calls: { count: 0, }, },);
 ```
 */
function answering(
  {
    script,
    calls,
  }: {
    readonly script: readonly TransportReply[];
    readonly calls: { count: number; };
  },
): ModelTransport {
  return async function transport(): Promise<TransportReply> {
    /**
     Reply for this attempt.
     */
    const reply = script[calls.count] ?? script.at(-1,) ?? OK_REPLY;
    calls.count += 1;
    return reply;
  };
}

/**
 Runs the ladder over a script.

 @param script - reply for each attempt

 @param calls - counter the cases read

 @returns Reply the ladder ended on

 @example
 ```ts
 const reply = await ladderOver({ script, calls, },);
 ```
 */
async function ladderOver(
  {
    script,
    calls,
  }: {
    readonly script: readonly TransportReply[];
    readonly calls: { count: number; };
  },
): Promise<TransportReply> {
  return await exchangeWithRetry({
    provider: 'anthropic',
    transport: answering({
      script,
      calls,
    },),
    exchange: {
      url: 'https://example.invalid/v1/messages',
      label: 'claude-haiku-5-5',
      method: 'POST',
      headers: {},
      bodyJson: '{}',
      signal: new AbortController().signal,
    },
    policy: WAIT_POLICY,
  },);
}

await describe({
  name: 'retry-after header',
  children: [
    it({
      name: 'READS THE HEADER AS SECONDS, whole or with a fraction, and no wait from a missing, empty, negative, hex, '
        + 'exponent or unreadable one',
      fn: async () => {
        expect([
          retryAfterHeaderMsOf({ headers: { 'retry-after': '3', }, },),
          retryAfterHeaderMsOf({ headers: { 'retry-after': '0.5', }, },),
          retryAfterHeaderMsOf({ headers: { 'retry-after': '', }, },),
          retryAfterHeaderMsOf({ headers: { 'retry-after': '-4', }, },),
          retryAfterHeaderMsOf({ headers: { 'retry-after': 'soon', }, },),
          retryAfterHeaderMsOf({ headers: { 'retry-after': '0x10', }, },),
          retryAfterHeaderMsOf({ headers: { 'retry-after': '1e3', }, },),
          retryAfterHeaderMsOf({ headers: { 'x-cat': '3', }, },),
          retryAfterHeaderMsOf({},),
        ],).toEqual([3_000, 500, 0, 0, 0, 0, 0, 0, 0,],);
      },
    },),

    it({
      name: 'RETRIES 529, Anthropic\'s overloaded status, and returns the eventual success',
      fn: async () => {
        /**
         Attempt counter.
         */
        const calls = { count: 0, };
        expect(
          await ladderOver({
            script: [{ status: 529, bodyText: '{"type":"error","error":{"type":"overloaded_error"}}', }, OK_REPLY,],
            calls,
          },),
        ).toStrictEqual(OK_REPLY,);
        expect(calls.count,).toBe(2,);
      },
    },),

    it({
      name: 'WAITS AS LONG AS THE HEADER ASKS before the retry, since a retry sent sooner is refused again',
      fn: async () => {
        /**
         Attempt counter.
         */
        const calls = { count: 0, };
        /**
         When the ladder started.
         */
        const startedAt = performance.now();
        expect(
          await ladderOver({
            script: [
              {
                status: 429,
                bodyText: '{"type":"error","error":{"type":"rate_limit_error"}}',
                headers: { 'retry-after': '1', },
              },
              OK_REPLY,
            ],
            calls,
          },),
        ).toStrictEqual(OK_REPLY,);
        expect(calls.count,).toBe(2,);
        expect(performance.now() - startedAt,).toBeGreaterThanOrEqual(990,);
      },
    },),

    it({
      name: 'ENDS THE LADDER AT ONCE when the header names a wait past its reach, for the router to hold the '
        + 'provider out until then',
      fn: async () => {
        /**
         Attempt counter.
         */
        const calls = { count: 0, };
        /**
         Refusal naming a wait of thirty seconds.
         */
        const refusal: TransportReply = {
          status: 429,
          bodyText: '{"type":"error","error":{"type":"rate_limit_error"}}',
          headers: { 'retry-after': '30', },
        };
        /**
         When the ladder started.
         */
        const startedAt = performance.now();
        expect(
          await ladderOver({
            script: [refusal, OK_REPLY,],
            calls,
          },),
        ).toStrictEqual(refusal,);
        expect(calls.count,).toBe(1,);
        expect(performance.now() - startedAt,).toBeLessThan(5_000,);
      },
    },),
  ],
},);
