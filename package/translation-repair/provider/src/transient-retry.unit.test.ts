/**
 Tests for the transport-level transient retry.

 `exchangeWithRetry` had no test. It sits under every model call in the
 pipeline, so its two failure modes are both expensive: retrying something
 permanent burns the flat-rate provider's capacity on a guaranteed rejection,
 and NOT retrying something transient throws away a voice the ensemble needed,
 which shows up much later as a thinner quorum rather than as an error.

 The abort case gets the most attention. A caller abort during backoff must
 stop the loop rather than burn the remaining attempts, and it must surface
 the failure that actually happened rather than a generic one.

 The two guard errors, degeneration and overrun, are both ends this system
 chose rather than weather. They ride one predicate instead of two separate
 class checks, so both get a case here: a check that named only one of them
 would pass this whole suite while the ladder re-bought every overrun.

 Every case uses a tiny backoff base so the suite stays fast; the delay
 arithmetic is jittered and is not what these assert.

 Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ModelTransport,
  StreamCutShortError,
  StreamDegenerateError,
  StreamOverrunError,
  ProviderHttpError,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  requireWholeAnthropicMessage,
  retryAfterMsOf,
} from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { quotingFailure, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';

/**
 Retry policy with a backoff small enough to keep the suite quick.
 */
const FAST_POLICY = {
  limit: 2,
  baseMs: 1,
};

/**
 Retry policy whose reach, the longest stated wait it sleeps in one backoff
 (1.2 s: its whole span rounded up to the next doubling), covers the
 one-second wait Hyper's hourly refusal names, so the ladder sleeps that wait
 instead of ending on it.
 */
const WAIT_POLICY = {
  limit: 2,
  baseMs: 300,
};

/**
 Retry policy whose first backoff is long enough that sleeping it out is
 visible against a caller abort.
 */
const SLOW_POLICY = {
  limit: 1,
  baseMs: 20_000,
};

/**
 Shortest first backoff `SLOW_POLICY` grants: equal jitter sleeps at least
 half the attempt's window, and the first window is the base.
 */
const SHORTEST_FIRST_BACKOFF_MS = SLOW_POLICY.baseMs / 2;

/**
 Builds the exchange every case sends.

 @param signal - caller abort handle

 @returns Exchange the transport receives

 @example
 ```ts
 const exchange = exchangeWith({ signal: new AbortController().signal, },);
 ```
 */
function exchangeWith({ signal, }: { readonly signal: AbortSignal; },) {
  return {
    url: 'https://example.invalid/chat',
    label: 'hf:whiskers',
    method: 'POST' as const,
    headers: { 'content-type': 'application/json', },
    bodyJson: JSON.stringify({ prompt: 'Does the cat purr?', },),
    signal,
  };
}

/**
 Transport replaying a scripted list of replies and failures, recording how
 many times it was called.

 @param script - one entry per expected attempt; an Error is thrown, a reply
 is returned

 @param calls - shared counter the cases assert on

 @returns Transport honoring the script

 @example
 ```ts
 const transport = scriptedTransport({ script: [okReply,], calls, },);
 ```
 */
function scriptedTransport(
  {
    script,
    calls,
  }: {
    readonly script: readonly (TransportReply | Error)[];
    readonly calls: { count: number; };
  },
): ModelTransport {
  return async function transport() {
    /**
     Entry for this attempt; the last entry repeats once the script runs out.
     */
    const entry = script[calls.count] ?? script.at(-1,);
    calls.count += 1;
    if (Error.isError(entry,))
      throw entry;
    if (entry === undefined)
      throw new Error('scripted transport ran out of entries',);
    return entry;
  };
}

/**
 Successful reply used wherever the content does not matter.
 */
const OK_REPLY: TransportReply = {
  status: 200,
  bodyText: '{"purr":"loud"}',
};

/**
 Reply a stream answered 200 with and stopped before its terminator.
 */
const TRUNCATED_REPLY: TransportReply = {
  status: 200,
  bodyText: 'data: {"purr":',
};

/**
 Refuses a success reply without its terminator, as every client's check does.

 @param reply - one attempt's reply

 @throws Error when the reply stops before `[DONE]`

 @example
 ```ts
 requireDone(reply,);
 ```
 */
function requireDone(reply: TransportReply,): void {
  if ((reply.status === 200) && (!reply.bodyText.includes('[DONE]',)))
    throw new Error('stream ended without its [DONE] terminator',);
}

/**
 Builds a success-status Messages body that ends in one error event and then
 its terminator.

 @param errorType - type the error event names

 @returns Reply a stream answered 200 with

 @example
 ```ts
 const reply = errorEventReply({ errorType: 'invalid_request_error', },);
 ```
 */
function errorEventReply({ errorType, }: { readonly errorType: string; },): TransportReply {
  return {
    status: 200,
    bodyText: `data: ${JSON.stringify({ type: 'error', error: { type: errorType, message: 'Napping', }, },)}\n\n`
      + `data: ${JSON.stringify({ type: 'message_stop', },)}\n\n`,
  };
}

/**
 Runs the ladder over a stream that answers 200 with one error event and its
 terminator, every attempt, checking each reply as a Messages client does.

 @param errorType - type the error event names

 @param calls - shared counter of attempts made

 @returns What the ladder threw, or that it returned

 @example
 ```ts
 const outcome = await ladderOutcome({ errorType: 'overloaded_error', calls, },);
 ```
 */
async function ladderOutcome(
  {
    errorType,
    calls,
  }: {
    readonly errorType: string;
    readonly calls: { count: number; };
  },
): Promise<unknown> {
  try {
    await exchangeWithRetry({
      provider: 'synthetic',
      transport: scriptedTransport({
        script: [errorEventReply({ errorType, },),],
        calls,
      },),
      exchange: exchangeWith({ signal: new AbortController().signal, },),
      policy: FAST_POLICY,
      verify: function verify(reply,): void {
        requireWholeAnthropicMessage({ provider: 'hyper', bodyText: reply.bodyText, },);
      },
    },);
    return 'returned';
  }
  catch (error) {
    return error;
  }
}

/**
 Successful reply carrying its terminator.
 */
const WHOLE_REPLY: TransportReply = {
  status: 200,
  bodyText: 'data: {"purr":"loud"}\n\ndata: [DONE]\n\n',
};

/**
 Runs one ladder over a script, recording the attempts it reported abandoned.

 @param script - one entry per attempt

 @returns Raw characters each reported attempt delivered, and whether the ladder threw

 @example
 ```ts
 const { reported, threw, } = await abandonedAttempts({ script: [TRUNCATED_REPLY, WHOLE_REPLY,], },);
 ```
 */
async function abandonedAttempts(
  { script, }: { readonly script: readonly (TransportReply | Error)[]; },
): Promise<{ readonly reported: readonly number[]; readonly threw: boolean; }> {
  /**
   Raw characters of each attempt the ladder reported.
   */
  const reported: number[] = [];
  try {
    await exchangeWithRetry({
      provider: 'synthetic',
      transport: scriptedTransport({
        script,
        calls: { count: 0, },
      },),
      exchange: exchangeWith({ signal: new AbortController().signal, },),
      policy: FAST_POLICY,
      verify: requireDone,
      onAbandonedAttempt: async function record({ deliveredChars, },): Promise<void> {
        reported.push(deliveredChars,);
      },
    },);
    return {
      reported,
      threw: false,
    };
  }
  catch (error) {
    // The outcome is the assertion; the error itself is not.
    void error;
    return {
      reported,
      threw: true,
    };
  }
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: exchangeWithRetry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns a success on the first attempt without retrying, so a '
            + 'healthy call costs exactly one request',
          fn: async () => {
            /**
             Attempt counter for this case.
             */
            const calls = { count: 0, };

            expect(
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [OK_REPLY,],
                  calls,
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
              },),
            ).toStrictEqual(OK_REPLY,);
            expect(calls.count,).toBe(1,);
          },
        },),

        it({
          name: 'RETURNS a non-retryable status immediately rather than retrying '
            + 'it, because burning the retry budget on a permanent rejection costs '
            + 'provider capacity the run needs elsewhere and cannot succeed',
          fn: async () => {
            // Concurrent because each status gets its own transport and counter,
            // so nothing here is a sequence.
            await Promise.all([
              400,
              401,
              404,
              422,
            ].map(async function expectNoRetry(status,) {
              /**
               Attempt counter for this status.
               */
              const calls = { count: 0, };

              /**
               Reply carrying the non-retryable status.
               */
              const reply: TransportReply = {
                status,
                bodyText: 'no',
              };

              expect(
                await exchangeWithRetry({
                  provider: 'synthetic',
                  transport: scriptedTransport({
                    script: [reply,],
                    calls,
                  },),
                  exchange: exchangeWith({ signal: new AbortController().signal, },),
                  policy: FAST_POLICY,
                },),
              ).toStrictEqual(reply,);
              expect(calls.count,).toBe(1,);
            },),);
          },
        },),

        it({
          name: 'RETRIES every transient status and returns the eventual success, '
            + 'since these are exactly the statuses a flat-rate provider emits '
            + 'under load and giving up on them loses a voice the ensemble needed',
          fn: async () => {
            // Concurrent for the same reason: one transport and counter per status.
            await Promise.all([
              408,
              429,
              500,
              502,
              503,
              504,
            ].map(async function expectRetry(status,) {
              /**
               Attempt counter for this status.
               */
              const calls = { count: 0, };

              expect(
                await exchangeWithRetry({
                  provider: 'synthetic',
                  transport: scriptedTransport({
                    script: [
                      {
                        status,
                        bodyText: 'busy',
                      },
                      OK_REPLY,
                    ],
                    calls,
                  },),
                  exchange: exchangeWith({ signal: new AbortController().signal, },),
                  policy: FAST_POLICY,
                },),
              ).toStrictEqual(OK_REPLY,);
              expect(calls.count,).toBe(2,);
            },),);
          },
        },),

        it({
          name: 'ENDS THE LADDER AT ONCE when the refusal names a wait past the ladder\'s reach, since '
            + 'Hyper\'s daily limit says "try again in 14m40s" and no retry inside the ladder would live to '
            + 'see it (Huasheng, 2026-09-07: 2,693 refused attempts over 2h53m)',
          fn: async () => {
            /**
             Attempt counter.
             */
            const calls = { count: 0, };

            /**
             Refusal naming a wait of fourteen minutes and forty seconds.
             */
            const dailyRefusal = {
              status: 429,
              bodyText: '{"type":"error","error":{"type":"rate_limit_error","message":'
                + '"You\'ve hit your daily rate limit. Please try again in 14m40s."}}',
            };

            /**
             When the exchange started.
             */
            const startedAt = performance.now();
            expect(
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [
                    dailyRefusal,
                    OK_REPLY,
                  ],
                  calls,
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
              },),
            ).toStrictEqual(dailyRefusal,);
            expect(calls.count,).toBe(1,);
            // The reply came back without the ladder sleeping the named wait.
            expect(performance.now() - startedAt,).toBeLessThan(retryAfterMsOf({ bodyText: dailyRefusal.bodyText, },),);
          },
        },),

        it({
          name: 'WAITS as long as a refusal asks before retrying, since Hyper\'s 429 body names its own '
            + 'wait ("try again in 1s") and a retry sent sooner is refused again',
          fn: async () => {
            expect(retryAfterMsOf({ bodyText: 'Please try again in 3s.', },),).toBe(3_000,);
            expect(retryAfterMsOf({ bodyText: 'try again in s', },),).toBe(0,);
            expect(retryAfterMsOf({ bodyText: 'try again in 12 minutes', },),).toBe(720_000,);
            expect(retryAfterMsOf({ bodyText: 'busy', },),).toBe(0,);
            // The daily wording writes hours, minutes and seconds in one run.
            expect(retryAfterMsOf({ bodyText: 'Please try again in 14m40s.', },),).toBe(880_000,);
            expect(retryAfterMsOf({ bodyText: 'Please try again in 2h25m18s.', },),).toBe(8_718_000,);
            expect(retryAfterMsOf({ bodyText: 'try again in 2h', },),).toBe(7_200_000,);

            /**
             Attempt counter.
             */
            const calls = { count: 0, };
            /**
             When the exchange started.
             */
            const startedAt = performance.now();
            expect(
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [
                    {
                      status: 429,
                      bodyText: '{"type":"error","error":{"type":"rate_limit_error","message":'
                        + '"You\'ve hit your hourly rate limit. Please try again in 1s."}}',
                    },
                    OK_REPLY,
                  ],
                  calls,
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: WAIT_POLICY,
              },),
            ).toStrictEqual(OK_REPLY,);
            expect(calls.count,).toBe(2,);
            // WAIT_POLICY backs off in hundreds of milliseconds; only the body's
            // wait can hold the retry for a full second.
            expect(performance.now() - startedAt,).toBeGreaterThanOrEqual(1_000,);
          },
        },),

        it({
          name: 'returns the last retryable reply once attempts exhaust rather '
            + 'than throwing, so the caller sees the status the provider actually '
            + 'gave and can record it instead of guessing',
          fn: async () => {
            /**
             Attempt counter across the exhausted budget.
             */
            const calls = { count: 0, };

            /**
             Reply the provider keeps giving.
             */
            const busy: TransportReply = {
              status: 503,
              bodyText: 'still busy',
            };

            expect(
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [busy,],
                  calls,
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
              },),
            ).toStrictEqual(busy,);
            // One initial attempt plus the policy's retries.
            expect(calls.count,).toBe(FAST_POLICY.limit + 1,);
          },
        },),

        it({
          name: 'retries a THROWN transport failure and succeeds later, since a '
            + 'dropped connection is the commonest transient failure and never '
            + 'arrives as a status at all',
          fn: async () => {
            /**
             Attempt counter for this case.
             */
            const calls = { count: 0, };

            expect(
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [
                    new Error('connection reset',),
                    OK_REPLY,
                  ],
                  calls,
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
              },),
            ).toStrictEqual(OK_REPLY,);
            expect(calls.count,).toBe(2,);
          },
        },),

        it({
          name: 'rethrows the ORIGINAL transport failure once retries exhaust, not '
            + 'a wrapper, so the cause survives to whoever reads the log',
          fn: async () => {
            /**
             Attempt counter across the exhausted budget.
             */
            const calls = { count: 0, };

            /**
             Exactly the failure the transport raised, held so the assertion can
             prove THAT object came back. A wrapper quoting it reads identically
             to a matcher that only checks wording, and a wrapper is what this
             case exists to rule out.
             */
            const transportFailure = new Error('connection reset',);

            await expect(
              exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [transportFailure,],
                  calls,
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
              },),
            ).rejects.toBe(transportFailure,);
            expect(calls.count,).toBe(FAST_POLICY.limit + 1,);
          },
        },),

        it({
          name: 'STOPS RETRYING once the caller aborts during backoff, rather than '
            + 'spending the remaining attempts on calls that are guaranteed to '
            + 'fail: the abort is what the user asked for and the budget belongs '
            + 'to the rest of the run',
          fn: async () => {
            /**
             Attempt counter, so a stopped loop is visible as a call count.
             */
            const calls = { count: 0, };

            /**
             Caller abort tripped as soon as the first attempt fails.
             */
            const controller = new AbortController();

            /**
             Transport that aborts the caller while failing transiently.
             */
            const transport: ModelTransport = async () => {
              calls.count += 1;
              controller.abort();
              return {
                status: 503,
                bodyText: 'busy',
              };
            };

            await expect(
              exchangeWithRetry({
                provider: 'synthetic',
                transport,
                exchange: exchangeWith({ signal: controller.signal, },),
                policy: {
                  limit: 5,
                  baseMs: 1,
                },
              },),
            ).rejects.toThrow(ProviderHttpError,);
            // One attempt, then the abort stops the loop instead of five more.
            expect(calls.count,).toBe(1,);
          },
        },),

        it({
          name: 'CARRIES THE PROVIDER THE CALLER NAMED on the failure it raises when an abort lands during the '
            + 'backoff, since the ladder cannot tell on its own whose reply it was retrying',
          fn: async () => {
            /**
             Caller abort tripped as soon as the first attempt fails.
             */
            const controller = new AbortController();

            /**
             Transport that aborts the caller while failing transiently.
             */
            const transport: ModelTransport = async () => {
              controller.abort();
              return {
                status: 503,
                bodyText: 'busy',
              };
            };

            /**
             What the stopped ladder raised for a call Bedrock was serving.
             */
            let raised: unknown;
            try {
              await exchangeWithRetry({
                provider: 'bedrock',
                transport,
                exchange: exchangeWith({ signal: controller.signal, },),
                policy: {
                  limit: 5,
                  baseMs: 1,
                },
              },);
            } catch (error) {
              raised = error;
            }

            expect(raised instanceof ProviderHttpError,).toBe(true,);
            expect(
              raised instanceof ProviderHttpError
                ? raised.provider
                : undefined,
            ).toBe('bedrock',);
          },
        },),

        it({
          name: 'ENDS THE BACKOFF THE MOMENT THE CALLER ABORTS, rather than sleeping it out while the call holds '
            + 'its seat (ledger P13: a backoff of up to 16 s outlived the abort)',
          fn: async () => {
            /**
             Attempt counter, so a stopped loop is visible as a call count.
             */
            const calls = { count: 0, };

            /**
             Caller abort tripped while the first backoff sleeps.
             */
            const controller = new AbortController();

            /**
             Transport failing transiently and aborting the caller on the next
             turn of the event loop, once the ladder has begun its backoff.
             */
            const transport: ModelTransport = async () => {
              calls.count += 1;
              setTimeout(function abortDuringBackoff() {
                controller.abort();
              }, 0,);
              return {
                status: 503,
                bodyText: 'busy',
              };
            };

            /**
             When the ladder began.
             */
            const startedMs = performance.now();
            await expect(
              exchangeWithRetry({
                provider: 'synthetic',
                transport,
                exchange: exchangeWith({ signal: controller.signal, },),
                policy: SLOW_POLICY,
              },),
            ).rejects.toThrow(ProviderHttpError,);
            expect({
              calls: calls.count,
              outlivedTheAbort: (performance.now() - startedMs) >= SHORTEST_FIRST_BACKOFF_MS,
            },).toEqual({
              calls: 1,
              outlivedTheAbort: false,
            },);
          },
        },),

        it({
          name: 'surfaces the STATUS that was failing when the caller aborted, so '
            + 'an abort during a rate-limit storm is still recognizable as a rate '
            + 'limit rather than as a bare cancellation',
          fn: async () => {
            /**
             Caller abort tripped during the first backoff.
             */
            const controller = new AbortController();

            /**
             Transport reporting rate limiting, then aborting the caller.
             */
            const transport: ModelTransport = async () => {
              controller.abort();
              return {
                status: 429,
                bodyText: 'slow down',
              };
            };

            await expect(
              exchangeWithRetry({
                provider: 'synthetic',
                transport,
                exchange: exchangeWith({ signal: controller.signal, },),
                policy: FAST_POLICY,
              },),
            ).rejects.toThrow(ProviderHttpError,);
          },
        },),

        it({
          name: 'surfaces the THROWN failure when the caller aborts after a '
            + 'transport drop, rather than replacing it with an HTTP error it '
            + 'never received',
          fn: async () => {
            /**
             Caller abort tripped during the first backoff.
             */
            const controller = new AbortController();

            /**
             Exactly the failure the transport raised, held so the assertion can
             prove the retry layer surfaced THAT object rather than the HTTP error
             the case name says it never received.
             */
            const transportFailure = new Error('connection reset',);

            /**
             Transport dropping the connection, then aborting the caller.
             */
            const transport: ModelTransport = async () => {
              controller.abort();
              throw transportFailure;
            };

            await expect(
              exchangeWithRetry({
                provider: 'synthetic',
                transport,
                exchange: exchangeWith({ signal: controller.signal, },),
                policy: FAST_POLICY,
              },),
            ).rejects.toBe(transportFailure,);
          },
        },),

        it({
          name: 'defaults to the shipped policy when none is given, so a caller '
            + 'that omits it gets the tuned budget rather than no retries at all',
          fn: async () => {
            /**
             Attempt counter under the default policy.
             */
            const calls = { count: 0, };

            expect(
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [
                    new Error('connection reset',),
                    OK_REPLY,
                  ],
                  calls,
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
              },),
            ).toStrictEqual(OK_REPLY,);
            expect(calls.count,).toBe(2,);
            expect(DEFAULT_RETRY_POLICY.limit,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'ENDS THE LADDER AT ONCE on an error event naming a request the provider refuses, and RETRIES one '
            + 'naming a failure that may pass, as the same refusals are met over plain HTTP',
          fn: async () => {
            /**
             Attempt counter for each refusal.
             */
            const permanentCalls = { count: 0, };
            const transientCalls = { count: 0, };

            /**
             What the ladder threw for the refused request.
             */
            const refused = await ladderOutcome({
              errorType: 'invalid_request_error',
              calls: permanentCalls,
            },);
            await ladderOutcome({
              errorType: 'overloaded_error',
              calls: transientCalls,
            },);

            expect(
              [
                permanentCalls.count,
                transientCalls.count,
                (refused instanceof ProviderHttpError) ? refused.status : refused,
              ],
            ).toEqual([
              1,
              FAST_POLICY.limit + 1,
              400,
            ],);
          },
        },),

        it({
          name: 'REFUSES TO RE-DISPATCH A CALL THIS SYSTEM ENDED ON PURPOSE, because a model that has '
            + 'begun repeating itself will repeat itself again. Treating a runaway as weather turns one '
            + 'of them into one per attempt the ladder grants, which multiplies the exact cost the '
            + 'degeneration guard exists to avoid',
          fn: async () => {
            /**
             Attempt counter, which is the whole assertion: the error's identity
             would look right even if the transport had been called five times.
             */
            const calls = { count: 0, };

            /**
             What the drain throws once it has cancelled a runaway reader. The
             caller's signal is NOT aborted on this path, because the termination
             is ours rather than the caller's steering, so nothing else in the
             retry loop marks it as permanent.
             */
            const runaway = new StreamDegenerateError({
              label: 'hf:whiskers',
              channel: 'reasoning',
              distinctRatio: 0.0037,
              charsSeen: 131_475,
              rawChars: 131_475,
            },);

            /**
             What the call did, as a value, so the assertion reads as an
             expectation rather than as control flow.
             */
            const raised = await (async function attempt(): Promise<unknown> {
              try {
                await exchangeWithRetry({
                  provider: 'synthetic',
                  transport: scriptedTransport({
                    script: [runaway,],
                    calls,
                  },),
                  exchange: exchangeWith({ signal: new AbortController().signal, },),
                  policy: FAST_POLICY,
                },);
                return undefined;
              }
              catch (error) {
                return error;
              }
            })();

            expect(raised,).toBeInstanceOf(StreamDegenerateError,);
            expect(calls.count,).toBe(1,);
          },
        },),

        it({
          name: 'REFUSES TO RE-DISPATCH A CALL THIS SYSTEM ENDED FOR VOLUME, because the ladder cannot '
            + 'tell one guard error from the other by class. A bound that stops a runaway at thirty-two '
            + 'thousand characters, then grants the ladder four more attempts at it, costs more than '
            + 'the unbounded call it replaced',
          fn: async () => {
            /**
             Attempt counter, which is the whole assertion: the error's identity
             arrives unchanged whether the ladder re-dispatched or not, so only
             the count separates a guard that holds from one that does not.
             */
            const calls = { count: 0, };

            /**
             What the drain throws once it has cancelled a reader that crossed the
             content bound. The caller's signal is NOT aborted on this path, the
             termination being ours rather than the caller's steering, so nothing
             else in the retry loop marks it permanent.
             */
            const overrun = new StreamOverrunError({
              label: 'hf:whiskers',
              channel: 'content',
              charsSeen: 32_000,
              rawChars: 32_000,
              cap: 32_000,
            },);

            /**
             What the call did, as a value, so the assertion reads as an
             expectation rather than as control flow.
             */
            const raised = await (async function attempt(): Promise<unknown> {
              try {
                await exchangeWithRetry({
                  provider: 'synthetic',
                  transport: scriptedTransport({
                    script: [overrun,],
                    calls,
                  },),
                  exchange: exchangeWith({ signal: new AbortController().signal, },),
                  policy: FAST_POLICY,
                },);
                return undefined;
              }
              catch (error) {
                return error;
              }
            })();

            expect(raised,).toBeInstanceOf(StreamOverrunError,);
            expect(calls.count,).toBe(1,);
          },
        },),
        it({
          name: 'READS A STATED WAIT in any case, in milliseconds before minutes, in words and with a fraction, '
            + 'and names none for a unit it does not know (ledger E11)',
          fn: async () => {
            expect([
              retryAfterMsOf({ bodyText: 'Upstream rate limited. Try again in 60s.', },),
              retryAfterMsOf({ bodyText: 'try again in 500ms', },),
              retryAfterMsOf({ bodyText: 'Please try again in 1 minute 30 seconds.', },),
              retryAfterMsOf({ bodyText: 'try again in 1.5s', },),
              retryAfterMsOf({ bodyText: 'TRY AGAIN IN 2 HOURS', },),
              retryAfterMsOf({ bodyText: 'try again in 3 fortnights', },),
            ],).toEqual([
              60_000,
              500,
              90_000,
              1_500,
              7_200_000,
              0,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'every billed attempt is reported (ledger P1)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS AN ATTEMPT THE CHECK REFUSED AND THE LADDER RETRIED, with the raw characters its body '
            + 'carried: the endpoint accepted it and billed what it streamed, and 3,864 such retries across the '
            + 'logs left no spend line',
          fn: async () => {
            expect(await abandonedAttempts({ script: [TRUNCATED_REPLY, WHOLE_REPLY,], },),).toEqual({
              reported: [TRUNCATED_REPLY.bodyText.length,],
              threw: false,
            },);
          },
        },),
        it({
          name: 'REPORTS A CUT STREAM THE LADDER RETRIED, with what it had delivered',
          fn: async () => {
            /**
             Stream cut after some text.
             */
            const cut = new StreamCutShortError({
              label: 'hf:whiskers',
              partialText: 'data: {"purr":"lo',
              progress: {
                firstByteMs: 10,
                firstContentMs: 10,
                maxGapMs: 5,
                elapsedMs: 40,
                chars: 17,
              },
              cause: new Error('reset',),
            },);
            expect(await abandonedAttempts({ script: [cut, WHOLE_REPLY,], },),).toEqual({
              reported: [cut.partialText.length,],
              threw: false,
            },);
          },
        },),
        it({
          name: 'REPORTS AN ATTEMPT THE LADDER ENDS WITHOUT RETRYING, which leaves by another path than the retries',
          fn: async () => {
            /**
             Overrun this system chose to end.
             */
            const overrun = new StreamOverrunError({
              label: 'hf:whiskers',
              channel: 'content',
              charsSeen: 900,
              cap: 800,
              rawChars: 81_000,
            },);
            expect(await abandonedAttempts({ script: [overrun,], },),).toEqual({
              reported: [overrun.rawChars,],
              threw: true,
            },);
          },
        },),
        it({
          name: 'REPORTS NOTHING FOR A FAILURE THAT DELIVERED NOTHING, since no endpoint billed it',
          fn: async () => {
            expect(await abandonedAttempts({ script: [new Error('connection reset',), WHOLE_REPLY,], },),).toEqual({
              reported: [],
              threw: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'every attempt is told to the caller before it is sent',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TELLS THE CALLER BEFORE EVERY ATTEMPT, the first and each retry, and before the transport is reached: '
            + 'two cut streams and a whole one are three tellings, each ahead of its request',
          fn: async () => {
            /**
             What happened, in order: the caller told, the transport reached.
             */
            const order: string[] = [];
            /**
             The scripted transport the recording one answers through.
             */
            const answering = scriptedTransport({
              script: [TRUNCATED_REPLY, TRUNCATED_REPLY, WHOLE_REPLY,],
              calls: { count: 0, },
            },);
            await exchangeWithRetry({
              provider: 'synthetic',
              transport: async function recording(request,) {
                order.push('send',);
                return await answering(request,);
              },
              exchange: exchangeWith({ signal: new AbortController().signal, },),
              policy: FAST_POLICY,
              verify: requireDone,
              beforeAttempt: async function told(): Promise<void> {
                order.push('told',);
              },
            },);
            expect(order,).toStrictEqual(['told', 'send', 'told', 'send', 'told', 'send',],);
          },
        },),
        it({
          name: 'ENDS THE EXCHANGE AT THE ATTEMPT THE CALLER REFUSES: what the caller throws before the third attempt '
            + 'escapes untouched, is not retried and is not reported as an abandoned attempt, and the third request '
            + 'is never sent',
          fn: async () => {
            /**
             What the caller throws to refuse the third attempt.
             */
            const refusal = new Error('the third attempt would pass the amount',);
            /**
             Attempts the caller was told of.
             */
            const told = { count: 0, };
            /**
             Attempts that reached the transport.
             */
            const calls = { count: 0, };
            /**
             Attempts reported abandoned.
             */
            const reported: number[] = [];
            /**
             What the exchange threw.
             */
            const raised = await rejectionOf(async () =>
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({ script: [TRUNCATED_REPLY,], calls, },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
                verify: requireDone,
                beforeAttempt: async function refusesThird(): Promise<void> {
                  told.count += 1;
                  if (told.count === 3)
                    throw refusal;
                },
                onAbandonedAttempt: async function record({ deliveredChars, },): Promise<void> {
                  reported.push(deliveredChars,);
                },
              },),
            );
            expect(raised,).toBe(refusal,);
            expect(told.count,).toBe(3,);
            expect(calls.count,).toBe(2,);
            expect(reported,).toStrictEqual([TRUNCATED_REPLY.bodyText.length, TRUNCATED_REPLY.bodyText.length,],);
          },
        },),
      ],
    },),

    describe({
      name: 'the retry line names the call it retries (ledger P12)',
      // ONE AT A TIME: its case diverts the process-wide `console.warn` across an
      // await (ledger B79).
      concurrency: 1,
      children: [
        it({
          name: 'NAMES THE MODEL on the retry line: 3,864 "stream ended without its [DONE] terminator ... retrying" '
            + 'lines across the logs named no model or provider, so none could be attributed',
          fn: async () => {
            /**
             Lines `console.warn` received.
             */
            const lines: string[] = [];
            /**
             `console.warn` as it was, put back once the ladder returns.
             */
            const warned = console.warn;
            console.warn = (...parts: readonly unknown[]) => {
              lines.push(parts.map(String,)
                .join(' ',),);
            };
            {
              await using restore = {
                [Symbol.asyncDispose]: async () => {
                  console.warn = warned;
                },
              };
              await exchangeWithRetry({
                provider: 'synthetic',
                transport: scriptedTransport({
                  script: [new Error('connection reset',), OK_REPLY,],
                  calls: { count: 0, },
                },),
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
              },);
              void restore;
            }
            /**
             The label every attempt of this exchange carries.
             */
            const { label, } = exchangeWith({ signal: new AbortController().signal, },);
            expect(lines.some(function namesIt(line,): boolean {
              return line.includes('retrying',) && line.includes(label,);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES A THROWN TRANSPORT FAILURE BY CLASS on the retry line and never by its message, which an '
            + 'unsendable header quotes',
          fn: async () => {
            const { warned, } = await warnLinesDuring({
              run: async () =>
                exchangeWithRetry({
                  provider: 'synthetic',
                  transport: scriptedTransport({
                    script: [quotingFailure(), OK_REPLY,],
                    calls: { count: 0, },
                  },),
                  exchange: exchangeWith({ signal: new AbortController().signal, },),
                  policy: FAST_POLICY,
                },),
            },);
            expect(warned.map(function untilBackoff(line,): string {
              return line.split('; retrying in',)[0] ?? '';
            },),).toEqual(['[translation-repair] [exchangeWithRetry] hf:whiskers: transport failure: refused by TypeError',],);
          },
        },),
        it({
          name: 'NAMES A THROWN PROVIDER STATUS FAILURE BY CLASS AND STATUS on the retry line, then the provider\'s '
            + 'own words labelled as its own',
          fn: async () => {
            const { warned, } = await warnLinesDuring({
              run: async () =>
                exchangeWithRetry({
                  provider: 'synthetic',
                  transport: scriptedTransport({
                    script: [new ProviderHttpError({ provider: 'synthetic', status: 503, bodyText: 'the cat is napping', },), OK_REPLY,],
                    calls: { count: 0, },
                  },),
                  exchange: exchangeWith({ signal: new AbortController().signal, },),
                  policy: FAST_POLICY,
                },),
            },);
            expect(warned.map(function untilBackoff(line,): string {
              return line.split('; retrying in',)[0] ?? '';
            },),).toEqual([
              '[translation-repair] [exchangeWithRetry] hf:whiskers: transport failure: refused by '
                + 'ProviderHttpError from synthetic with HTTP 503 (the provider said: "the cat is napping")',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: `${exchangeWithRetry.name} thrown non-Errors`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRAPS a thrown value that is no Error into one, so the retry outcome always carries a failure '
            + 'the caller can read',
          fn: async () => {
            /**
             Exchange over a transport that throws a bare string.
             */
            const refusal = await rejectionOf(async function exhausted(): Promise<unknown> {
              return await exchangeWithRetry({
                provider: 'synthetic',
                transport: async function transport(): Promise<TransportReply> {
                  // A thrown value that is no Error, as a hostile transport might.
                  // Typed `unknown` so the throw carries no literal the lint reads
                  // as a non-Error shape.
                  const thrownValue: unknown = 'the cat knocked the cable';
                  throw thrownValue;
                },
                exchange: exchangeWith({ signal: new AbortController().signal, },),
                policy: FAST_POLICY,
              },);
            },);
            // THE CLASS FIRST: the message and the cause are read off an Error, so a
            // string rejected unwrapped must fail here.
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe('Error: the transport threw a value that is not an Error',);
            expect(Error.isError(refusal,) ? refusal.cause : undefined,).toBe('the cat knocked the cable',);
          },
        },),
      ],
    },),
  ],
},);
