/**
 Exercises provider-supplied rate data, including the NeuralWatt overage exception.
 @module
 */

import { syntheticWeeklyWindow, } from './synthetic-quota-fixture.ts';
import { createAssistantMessageEventStream, isRetryableAssistantError, lazyStream, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension, { headerReset, QUOTA_READ_INTERVAL_MS, } from '../dist/final/node/index.mjs';
import {
  assistant,
  collect,
  finish,
  fixture,
  immediate,
  MEMBER_KEY,
  PHYSICAL_MODELS,
  poolModel,
  rateLimitRefusal,
  RATE_LIMIT_STATUS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';
import { labelOf, REVIEWER_TARGETS, WRITER_TARGETS, } from './member-order-fixture.ts';

/**
 The recorded NeuralWatt overage refusal, verbatim: 795 characters including the adapter's own
 `429: ` prefix, recorded with its run folder and date in `doc/captures.md`.
 */
const OVERAGE_REFUSAL: string = `429: {"type":"rate_limit_error","code":"concurrent_budget_exceeded","message":"Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. Wait for a request to complete and retry. Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews. Turn auto-renew back on to restore your account level's limits: https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off","retry_strategy":{"type":"concurrent_drain","suggested_initial_delay_s":1,"max_delay_s":30,"backoff":"exponential","backoff_base":2,"jitter":true},"retryable":true,"context":{"budget":6,"in_flight":6,"model":"deepseek-ai/DeepSeek-V4.1-Flash","limit_type":"concurrent","tpm_window_tokens":0,"tpm_limit":5242880,"floor":5,"burst":6,"burst_shed":false}}`;

/** Empty flex lane; a positive standard ceiling must not let it admit any requests. */
const NO_FLEX_REQUESTS = 0;

/** A quota window in the fixture clock, not a plugin cooldown. */
const WINDOW_MS = 60_000;

/** Header reset duration in seconds, matching the fixture clock window. */
const WINDOW_SECONDS = WINDOW_MS / 1_000;

/**
 Stand-in published flex ceiling for the NeuralWatt member; the real value is read from `/v1/rate_limits`
 in production, and 20 is chosen only to sit above the recorded overage cap.
 */
const PUBLISHED_CAP = 20;

/** Concurrent slots the recorded overage refusal reports, `6/6 slots in use`. */
const OVERAGE_CAP = 6;

/**
 Reviewer's NeuralWatt member, the flex lane of the two NeuralWatt members the pools ask,
 with the full catalog record the stand-in registry returns.
 */
const NEURALWATT_FLEX = REVIEWER_TARGETS.at(-1,);
if (NEURALWATT_FLEX === undefined)
  throw new Error('Expected the reviewer pool to end with its NeuralWatt member',);

/**
 Delay the recorded overage refusal names in `retry_strategy.suggested_initial_delay_s`, one second.
 */
const RECORDED_DELAY_MS = 1_000;

/**
 Seconds a well-formed compound reset names, `1m30s` in the duration case.
 */
const COMPOUND_SECONDS = 90;

/**
 Share of a quota window a depleted reading reports.
 */
const DEPLETED_PERCENT = 0;

/**
 Share of a quota window a full reading reports.
 */
const FULL_PERCENT = 100;

/**
 Longest delay a refusal body may set, the largest `max_delay_s` the provider's documented retry
 strategies name (https://docs.neuralwatt.com/guides/rate-limits.md, "Uncached TPM Limits").
 */
const BODY_DELAY_CAP_MS = 60_000;

/**
 Delay the capped case's body names, an hour, far past that bound.
 */
const BODY_DELAY_HOUR_SECONDS = 3_600;

/**
 One concurrency slot, the stand-in ceiling a single open stream fills.
 */
const SINGLE_SLOT = 1;

/**
 Writer head, the NeuralWatt standard lane that asks the same endpoint for its own ceiling.
 */
const [NEURALWATT_STANDARD,] = WRITER_TARGETS;

/**
 Read the URL from each fetch input form without stringifying an arbitrary object.

 @param input - quota fetch destination in the fixture

 @returns URL to check against the quota contract
 */
function quotaUrl(input: Parameters<typeof globalThis.fetch>[0],): string {
  if ((typeof input) === 'string')
    return input;
  if (input instanceof URL)
    return input.href;
  return input.url;
}

/**
 Stand-in body for `GET /v1/rate_limits?model=…`, using the field names that endpoint documents:
 `models[].flex_floor` and `models[].effective_concurrency`
 (https://docs.neuralwatt.com/guides/rate-limits.md, "Checking your current limit").
 */
function rateLimitsReading({
  flex,
  effective = PUBLISHED_CAP,
}: {
  readonly flex: number;
  readonly effective?: number;
},): Response {
  return Response.json({ models: [{ effective_concurrency: effective, flex_floor: flex, },], },);
}

/**
 Stand-in body for `GET /v1/quota`, using the field names that endpoint documents:
 `subscription.in_overage`, `subscription.auto_renew` and `subscription.current_period_end`
 (https://docs.neuralwatt.com/api/quota).
 */
function quotaReading({
  inOverage,
  autoRenew = false,
  periodEnd,
}: {
  readonly inOverage: boolean;
  readonly autoRenew?: boolean;
  readonly periodEnd?: number;
},): Response {
  return Response.json({
    subscription: {
      in_overage: inOverage,
      auto_renew: autoRenew,
      ...(periodEnd === undefined ? {} : { current_period_end: new Date(periodEnd,).toISOString(), }),
    },
  },);
}

/**
 Attempts a pool makes when one member has answered or refuses: the first one it dispatches.
 */
const FIRST_ATTEMPT = 1;

/**
 Attempts a pool makes when a second member is asked as well.
 */
const SECOND_ATTEMPT = 2;
await describe({
  name: 'provider quota evidence',
  children: [
    ...[false, true,].map(function retryAfterFormat(asDate,) {
      return it({
        name: `uses ${asDate ? 'HTTP-date' : 'numeric'} 429 Retry-After and resumes at the reported reset`,
        fn: async function retryWindow(ctx,) {
          const clock = { now: Date.now(), };
          ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
          const deadline = new Date(clock.now + WINDOW_MS,);
          const retryAfter = asDate ? deadline.toUTCString() : String(WINDOW_SECONDS,);
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, options: member, }) {
              if (calls.length !== FIRST_ATTEMPT)
                return immediate({ model, },);
              // Stand-in for a shape no installed adapter emits: each adapter calls `onResponse`
              // after a successful response alone, so a real 429 never reaches the plugin here.
              return lazyStream(model, async function refused() {
                await member?.onResponse?.({
                  status: RATE_LIMIT_STATUS,
                  headers: { 'Retry-After': retryAfter, },
                }, model,);
                return immediate({ model, errorMessage: rateLimitRefusal({ model, },), },);
              },);
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          const beforeReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          clock.now = asDate ? Date.parse(retryAfter,) : deadline.getTime();
          const afterReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          /**
           Physical member each dispatch answered from, as `provider/id`.
           */
          const answered = [first, beforeReset, afterReset,].map(function member(result,) {
            return `${result.result.provider}/${result.result.model}`;
          },);
          expect(answered,).toEqual([
            `${WRITER_TARGETS[1].provider}/${WRITER_TARGETS[1].id}`,
            `${WRITER_TARGETS[1].provider}/${WRITER_TARGETS[1].id}`,
            `${WRITER_TARGETS[0].provider}/${WRITER_TARGETS[0].id}`,
          ],);
        },
      },);
    },),
    it({
      name: 'uses the later compound OpenAI reset when both zero-remaining headers are seen on success',
      // Neither pool has an OpenAI member since 2026-11-19, so this provider branch is checked
      // against the parser directly rather than through a dispatch.
      fn: async function openAiRemaining() {
        /**
         Observation time for a successful response carrying depleted windows.
         */
        const now = Date.parse('2026-11-19T00:00:00Z',);
        /**
         Later of the two depleted windows, the request-count one.
         */
        const reset = headerReset({
          provider: 'openai',
          headers: new Headers({
            'x-ratelimit-remaining-tokens': '0',
            'x-ratelimit-reset-tokens': `${WINDOW_SECONDS / 2}s`,
            'x-ratelimit-remaining-requests': '0',
            'x-ratelimit-reset-requests': '1m0s',
          },),
          now,
          status: 200,
        },);
        expect(reset - now,).toBe(WINDOW_MS,);
      },
    },),
    it({
      name: 'reads Synthetic on first use and at reset, not on every request',
      fn: async function syntheticWindow(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const end = clock.now + WINDOW_MS;
        const endpoints: string[] = [];
        const { provider, calls, options, } = await fixture({
          extension,
          models: PHYSICAL_MODELS.filter(model => (model.provider === 'synthetic') || (model.provider === 'opencode-go'),),
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(input, init,) {
            // Assert credentials inside the test without printing or returning them.
            if (new Headers(init?.headers,).get('authorization',) !== `Bearer ${MEMBER_KEY}`)
              throw new Error('Incorrect stand-in bearer',);
            if (init?.redirect !== 'error')
              throw new Error('Quota fetch permits redirects',);
            endpoints.push(quotaUrl(input,),);
            return syntheticWeeklyWindow({
              percentRemaining: clock.now < end ? DEPLETED_PERCENT : FULL_PERCENT,
              nextRegenAt: new Date(end,).toISOString(),
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        clock.now = end;
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(endpoints,).toEqual(Array.from({ length: 2, }, () => 'https://api.synthetic.new/v2/quotas',),);
        expect(calls.map(call => call.model.provider,),).toEqual(['opencode-go', 'opencode-go', 'synthetic', 'synthetic',],);
      },
    },),
    it({
      name: 'admits the member when the limits lookup returns no single row',
      fn: async function noSingleRow() {
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(input,) {
            return quotaUrl(input,).includes('/v1/rate_limits',)
              ? Response.json({
                models: [
                  { flex_floor: NO_FLEX_REQUESTS, },
                  { flex_floor: NO_FLEX_REQUESTS, },
                ],
              },)
              : Response.json({},);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(NEURALWATT_FLEX,),],);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'ignores a malformed compound reset duration',
      fn: async function malformedDuration() {
        /**
         Observation time both calls share.
         */
        const now = Date.parse('2026-11-19T00:00:00Z',);
        /**
         Malformed durations the endpoint's reset field can carry.
         */
        const malformed = [
          'six minutes',
          '6w',
          '1.',
          '1m30',
        ] as const;
        for (const value of malformed) {
          expect(headerReset({
            provider: 'openai',
            headers: new Headers({
              'x-ratelimit-remaining-requests': '0',
              'x-ratelimit-reset-requests': value,
            },),
            now,
            status: 200,
          // A malformed duration leaves the deadline at the observation time, which holds nothing.
          },),).toBe(now,);
        }
        expect(headerReset({
          provider: 'openai',
          headers: new Headers({
            'x-ratelimit-remaining-requests': '0',
            'x-ratelimit-reset-requests': '1m30s',
          },),
          now,
          status: 200,
        },),).toBe(now + (COMPOUND_SECONDS * 1_000),);
      },
    },),
    it({
      name: 'does not send a credential to a quota endpoint after a registry origin override',
      fn: async function credentialsStayOnOrigin() {
        const probe = { invoked: false, };
        const { provider, options, } = await fixture({
          extension,
          models: PHYSICAL_MODELS.filter(model => model.provider === 'synthetic',),
          resolvedBaseUrl: 'https://stand-in-proxy.example/v2',
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            probe.invoked = true;
            return Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.provider,).toBe('synthetic',);
        expect(probe.invoked,).toBe(false,);
      },
    },),
    it({
      name: 'holds the member for the delay the refusal body names',
      fn: async function bodyDelayHold(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) {
            return calls.length === FIRST_ATTEMPT
              ? immediate({ model, errorMessage: OVERAGE_REFUSAL, },)
              : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.errorMessage,).toContain('concurrent_budget_exceeded',);
        const held = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(held.result.errorMessage,).toContain(`${labelOf(NEURALWATT_FLEX,)} is held until`,);
        expect(calls.length,).toBe(FIRST_ATTEMPT,);
        expect(held.result.errorMessage,).toContain(new Date(clock.now + RECORDED_DELAY_MS,).toISOString(),);
      },
    },),
    it({
      name: 'caps the delay a refusal body names',
      fn: async function cappedBodyDelay(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        /**
         Refusal body naming an hour, far past any delay the provider's strategies document.
         */
        const hourBody = `429: ${JSON.stringify({ retry_strategy: { suggested_initial_delay_s: BODY_DELAY_HOUR_SECONDS, }, },)}`;
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) {
            return calls.length === FIRST_ATTEMPT
              ? immediate({ model, errorMessage: hourBody, },)
              : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.stopReason,).toBe('error',);
        const held = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(held.result.errorMessage,).toContain(new Date(clock.now + BODY_DELAY_CAP_MS,).toISOString(),);
        expect(held.result.errorMessage,).not.toContain(new Date(clock.now + (BODY_DELAY_HOUR_SECONDS * 1_000),).toISOString(),);
        expect(calls.length,).toBe(FIRST_ATTEMPT,);
      },
    },),
    it({
      name: 'reads the standard lane’s own ceiling, not the flex floor, for the writer head',
      fn: async function standardLaneCeiling() {
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_STANDARD, NEURALWATT_FLEX,],
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(input,) {
            return quotaUrl(input,).includes('/v1/rate_limits',)
              ? Response.json({
                models: [
                  { effective_concurrency: PUBLISHED_CAP, flex_floor: NO_FLEX_REQUESTS, },
                ],
              },)
              : Response.json({},);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(NEURALWATT_STANDARD,),],);
        expect(result.provider,).toBe(NEURALWATT_STANDARD.provider,);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'reports the named NeuralWatt member and a published flex floor of zero without retrying',
      fn: async function zeroFlexFloor() {
        const redirects: RequestRedirect[] = [];
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(input, init,) {
            if (init?.redirect !== undefined)
              redirects.push(init.redirect,);
            return quotaUrl(input,).includes('/v1/rate_limits',)
              ? rateLimitsReading({ flex: NO_FLEX_REQUESTS, },)
              : Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls,).toEqual([],);
        expect(redirects,).toEqual(['error', 'error',],);
        expect(result.errorMessage,).toContain(`${NEURALWATT_FLEX.provider}/${NEURALWATT_FLEX.id} has a published flex floor of zero`,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    it({
      name: 'admits only the reported flex ceiling, not the standard concurrency ceiling',
      fn: async function flexCeiling(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const physical = NEURALWATT_FLEX;
        const reading = { flex: 1, };
        const ready = Promise.withResolvers<void>();
        const held = createAssistantMessageEventStream();
        const { provider, calls, options, } = await fixture({
          extension,
          models: [physical,],
          play({ model, }) {
            if (calls.length !== FIRST_ATTEMPT)
              return immediate({ model, },);
            ready.resolve();
            return held;
          },
          async fetchQuota(input,) {
            return quotaUrl(input,).includes('/v1/rate_limits',)
              ? rateLimitsReading({ flex: reading.flex, },)
              : Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const pending = provider.streamSimple(model, TRANSCRIPT, options,);
        {
          await using cleanup: AsyncDisposable = {
            async [Symbol.asyncDispose](): Promise<void> {
              finish({ stream: held, message: assistant({ model: physical, },), },);
              await collect(pending,);
            },
          };
          await ready.promise;
          const denied = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(denied.result.errorMessage,).toContain(`${physical.provider}/${physical.id} is at its local concurrency ceiling (1 of 1 in use)`,);
          expect(isRetryableAssistantError(denied.result,),).toBe(false,);
          expect(calls.length,).toBe(FIRST_ATTEMPT,);
          reading.flex += 1;
          // The endpoint may be read again only after its documented interval.
          clock.now += QUOTA_READ_INTERVAL_MS;
          const afterIncrease = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(afterIncrease.result.stopReason,).toBe('stop',);
        }
        const again = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(again.result.provider,).toBe('neuralwatt',);
      },
    },),
    it({
      name: `a NeuralWatt overage 429 overrides ${PUBLISHED_CAP} with ${OVERAGE_CAP} until a renewal learned on reentry`,
      fn: async function overageCap(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const end = clock.now + WINDOW_MS;
        const endpoints: string[] = [];
        const ready = Promise.withResolvers<void>();
        const held: (() => void)[] = [];
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) {
            if (calls.length === FIRST_ATTEMPT)
              return immediate({ model, errorMessage: OVERAGE_REFUSAL, },);
            if (clock.now >= end)
              return immediate({ model, },);
            const stream = createAssistantMessageEventStream();
            held.push(() => finish({ stream, message: assistant({ model, },), },),);
            if (held.length === OVERAGE_CAP)
              ready.resolve();
            // Deliberately complete any unexpected extra stream so a mutant fails without hanging.
            if (held.length > OVERAGE_CAP)
              finish({ stream, message: assistant({ model, },), },);
            return stream;
          },
          async fetchQuota(input,) {
            const url = new URL(quotaUrl(input,),);
            endpoints.push(url.pathname,);
            if (url.pathname === '/v1/rate_limits') {
              if (url.searchParams.get('model',) !== 'deepseek-v4.1-flash-flex')
                throw new Error('The quota query did not name the flex model',);
              return rateLimitsReading({ flex: PUBLISHED_CAP, },);
            }
            return quotaReading({
              inOverage: true,
              ...(calls.length === 0 ? {} : { periodEnd: end, }),
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.errorMessage,).toContain('overage with auto-renew off',);
        // The refusal body names a one-second hold, which the member must observe before the cap runs.
        clock.now += RECORDED_DELAY_MS;
        const inFlight = Array.from({ length: OVERAGE_CAP, }, () => provider.streamSimple(model, TRANSCRIPT, options,),);
        await using cleanup: AsyncDisposable = {
          async [Symbol.asyncDispose](): Promise<void> {
            for (const release of held)
              release();
            await Promise.all(inFlight.map(function consume(stream,) {
              return collect(stream,);
            },),);
          },
        };
        await ready.promise;
        {
          const limited = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(limited.result.errorMessage,).toContain(`local concurrency ceiling (${OVERAGE_CAP} of ${OVERAGE_CAP} in use)`,);
          expect(isRetryableAssistantError(limited.result,),).toBe(false,);
          expect(calls.length,).toBe(1 + OVERAGE_CAP,);
          clock.now = end;
          const renewed = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(renewed.result.provider,).toBe('neuralwatt',);
          expect(renewed.result.stopReason,).toBe('stop',);
          expect(endpoints,).toEqual([
            '/v1/rate_limits', '/v1/quota', '/v1/rate_limits', '/v1/quota', '/v1/rate_limits', '/v1/quota',
          ],);
        }
      },
    },),
    it({
      name: 'a quota read that confirms the overage cleared removes the observed cap',
      fn: async function clearedOverage(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        /**
         Hand-written stand-in for the recorded refusal class with one slot, so the case needs one open
         stream; `OVERAGE_REFUSAL` is the recorded body it models, whose cap is six.
         */
        const singleSlot = rateLimitRefusal({
          model: NEURALWATT_FLEX,
          message: 'Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: 1/1 slots in use. '
            + 'Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews.',
        },);
        /**
         Subscription state the quota endpoint reports, flipped once the exception clears.
         */
        const subscription = { inOverage: true, };
        const ready = Promise.withResolvers<void>();
        const held = createAssistantMessageEventStream();
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) {
            if (calls.length === 1)
              return immediate({ model, errorMessage: singleSlot, },);
            if (calls.length === 2) {
              ready.resolve();
              return held;
            }
            return immediate({ model, },);
          },
          async fetchQuota(input,) {
            return quotaUrl(input,).includes('/v1/rate_limits',)
              ? rateLimitsReading({ flex: PUBLISHED_CAP, },)
              : quotaReading({ inOverage: subscription.inOverage, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.errorMessage,).toContain('overage with auto-renew off',);
        const open = provider.streamSimple(model, TRANSCRIPT, options,);
        {
          await using cleanup: AsyncDisposable = {
            async [Symbol.asyncDispose](): Promise<void> {
              finish({ stream: held, message: assistant({ model: NEURALWATT_FLEX, },), },);
              await collect(open,);
            },
          };
          await ready.promise;
          const ceiling = SINGLE_SLOT;
          const denied = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(denied.result.errorMessage,).toContain(`local concurrency ceiling (${ceiling} of ${ceiling} in use)`,);
          subscription.inOverage = false;
          // The endpoint may be read again only after its documented interval.
          clock.now += QUOTA_READ_INTERVAL_MS;
          const cleared = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(cleared.result.stopReason,).toBe('stop',);
          expect(cleared.result.provider,).toBe('neuralwatt',);
        }
      },
    },),
  ],
},);
