/**
 Tests for the Claude client, the first-party route on the owner's own Claude key.

 NOTHING HERE REACHES A PROVIDER: every transport is an in-memory script, and every key is invented. The body the
 client builds is tested in `claude-body.unit.test.ts`; these cases pin what a call does around it. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  COMPLETION_CAP,
  resetRunSpend,
  type RosterModelId,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  infoLinesDuring,
  warnLinesDuring,
} from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  exchangeFailureLogText,
  ProviderHttpError,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  CLAUDE_MODELS,
  CLAUDE_PACE_WINDOW_MS,
  CLAUDE_REQUESTS_PER_MINUTE,
  ClaudeModelNotServedError,
  ClaudeSchemaKeywordError,
  createClaudeClient,
  createProviderBudgets,
  statedWaitMsOf,
} from '../dist/final/node/index.mjs';
import {
  askCat,
  errorChainText,
  firstBodyOf,
  ONE_QUICK_RETRY,
  scripted,
} from './direct-client.test-fixture.ts';
import { anthropicFrameOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';
import {
  CLAUDE_CAT_KEY,
  claudeStreamOf,
  headedReply,
  PLAIN_USAGE,
} from './claude-client.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

await describe({
  name: 'the Claude client',
  // SEQUENTIAL: the run spend meter is process-wide, so cases that read its change must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'ANSWERS from the text blocks of a streamed Messages reply, dropping the thinking, and posts it to the '
        + 'messages endpoint with the key in x-api-key, the API version header and the anthropic wire format',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
          },],
        },);

        const reply = await askCat({
          client: createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },),
          modelId: 'claude-haiku-5-5',
        },);

        expect(reply.text,).toBe('{"spot":"sunbeam"}',);
        expect(reply.finishReason,).toBe('end_turn',);
        expect(reply.usage,).toEqual({ prompt_tokens: 100, completion_tokens: 10, total_tokens: 110, },);
        expect(exchanges.length,).toBe(1,);
        const [exchange,] = exchanges;
        expect(exchange?.url,).toBe('https://api.anthropic.com/v1/messages',);
        expect(exchange?.method,).toBe('POST',);
        expect(exchange?.headers['x-api-key'],).toBe(CLAUDE_CAT_KEY,);
        expect(exchange?.headers['anthropic-version'],).toBe('2023-06-01',);
        expect(exchange?.headers.Authorization,).toBeUndefined();
        expect(exchange?.wireFormat,).toBe('anthropic',);
        expect(firstBodyOf({ exchanges, },).stream,).toBe(true,);
      },
    },),

    it({
      name: 'DROPS the provider\'s words from a 401, which may echo a fragment of the key, and still fails as a 401 '
        + 'after one request',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{
            status: 401,
            bodyText: '{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key ****Qx4d"}}',
          },],
        },);

        const failure: unknown = await rejectionOf(
          () =>
            askCat({
              client: createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },),
              modelId: 'claude-haiku-5-5',
            },),
        );

        expect(failure,).toBeInstanceOf(ProviderHttpError,);
        expect((failure as ProviderHttpError).status,).toBe(401,);
        expect((failure as ProviderHttpError).provider,).toBe('anthropic',);
        expect(exchangeFailureLogText({ error: failure, },),).not.toContain('Qx4d',);
        expect(errorChainText({ error: failure, },),).not.toContain('Qx4d',);
        expect(exchanges.length,).toBe(1,);
      },
    },),
    it({
      name: 'REFUSES EVERY MODEL BUT THE EIGHT CLAUDE CARDS before any request is built, by a named error: the key lists '
        + 'Opus, Fable and older models, so a roster model of another provider and an Opus id both stop at the client',
      fn: async () => {
        await Promise.all([
          'hf:moonshotai/Kimi-K3',
          'deepseek-v4.1-flash',
          'claude-opus-5-5',
          'claude-sonnet-5',
          'claude-haiku-4-5',
        ].map(async function refuseOtherModel(modelId,) {
          const { transport, exchanges, } = scripted({
            replies: [{
              status: 200,
              bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
            },],
          },);

          const failure: unknown = await rejectionOf(
            () =>
              askCat({
                client: createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },),
                modelId: modelId as RosterModelId,
              },),
          );

          expect((failure as Error).name,).toBe('ClaudeModelNotServedError',);
          expect((failure as Error).message,).toContain(modelId,);
          expect((failure as Error).message,).not.toContain(CLAUDE_CAT_KEY,);
          expect(exchanges.length,).toBe(0,);
        },),);
      },
    },),
    it({
      name: 'ANSWERS A REFUSAL AS A REFUSAL OUTCOME AND NOT AS AN OUTAGE: stop_reason refusal is a 200, so the seat has '
        + 'no answer for that call, the partial text is discarded, the category is logged, and nothing is retried or '
        + 'thrown for the router to hold the provider out for',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({
              text: 'The cat is on the',
              stopReason: 'refusal',
              stopDetails: { type: 'refusal', category: 'cyber', explanation: 'declined', },
              usage: PLAIN_USAGE,
            }),
          },],
        },);

        const { result: reply, warned, } = await warnLinesDuring({
          run: async () =>
            await askCat({
              client: createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },),
              modelId: 'claude-haiku-5-5',
            },),
        },);

        expect(reply.text,).toBe('',);
        expect(reply.finishReason,).toBe('refusal',);
        expect(reply.refusal,).toContain('cyber',);
        expect(reply.usage,).toEqual({ prompt_tokens: 100, completion_tokens: 10, total_tokens: 110, },);
        expect(exchanges.length,).toBe(1,);
        expect(warned.some(function namesCategory(line,): boolean {
          return line.includes('claude-haiku-5-5',) && line.includes('refusal',) && line.includes('cyber',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'READS A REFUSAL WITH NO CATEGORY as a refusal too, the category null being a permanent valid state, and '
        + 'hands it to the caller as the refusal-shaped outcome',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({
              text: '',
              stopReason: 'refusal',
              stopDetails: { type: 'refusal', category: null, explanation: null, },
              usage: PLAIN_USAGE,
            }),
          },],
        },);

        const outcome = await createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },)
          .chatJson({
            modelId: 'claude-sonnet-5-5',
            messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
            signal: new AbortController().signal,
            validate: function isSpot(value: unknown,): value is { readonly spot: string; } {
              return (typeof value) === 'object';
            },
          },);

        expect(outcome.kind,).toBe('refusal-shaped',);
        expect(exchanges.length,).toBe(1,);
      },
    },),
    it({
      name: 'WRITES ONE SPEND LINE PER CALL IN USD FROM THE FOUR COUNTS, named by the card and not by the wire model: '
        + '100 plain, 50 written and 850 read prompt tokens and 10 completion tokens on a Haiku effort card are a '
        + '1,000-token prompt that cost 0.00002975, with 850 cached, and the run meter takes the same figure',
      fn: async () => {
        resetRunSpend();
        const { transport, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({
              text: '{"spot":"sunbeam"}',
              stopReason: 'end_turn',
              usage: { input: 100, cacheWrite: 50, cacheRead: 850, output: 10, },
            }),
          },],
        },);

        const { logged, } = await infoLinesDuring({
          run: async () =>
            await askCat({
              client: createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },),
              modelId: 'claude-haiku-5-5-effort-low',
            },),
        },);

        /**
         The one SPEND line the call wrote.
         */
        const spendLines = logged.filter(function isSpend(line,): boolean {
          return line.includes('SPEND ',);
        },);
        expect(spendLines.length,).toBe(1,);
        const [line = '',] = spendLines;
        expect(line,).toContain('SPEND provider=anthropic model=claude-haiku-5-5-effort-low prompt=1000 completion=10 cost=',);
        expect(line,).toContain(' cached=850',);
        expect(Number((line.split(' cost=',)[1] ?? '').split(' ',)[0],),).toBeCloseTo(0.00002975, 10,);
        expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.00002975, 10,);
      },
    },),

    it({
      name: 'WRITES THE SPEND LINE OF A REFUSAL TOO, since a mid-stream decline bills the tokens it spent',
      fn: async () => {
        resetRunSpend();
        const { transport, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({
              text: 'The cat is on the',
              stopReason: 'refusal',
              stopDetails: { type: 'refusal', category: 'bio', explanation: 'declined', },
              usage: { input: 1_000, cacheWrite: 0, cacheRead: 0, output: 100, },
            }),
          },],
        },);

        const { logged, } = await infoLinesDuring({
          run: async () =>
            await warnLinesDuring({
              run: async () =>
                await askCat({
                  client: createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },),
                  modelId: 'claude-sonnet-5-5',
                },),
            },),
        },);

        expect(logged.some(function spendOfRefusal(line,): boolean {
          return line.includes('SPEND provider=anthropic model=claude-sonnet-5-5 prompt=1000 completion=100 cost=',);
        },),).toBe(true,);
        // 1,000 plain tokens at 2 and 100 completion tokens at 10 USD a million.
        expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.003, 10,);
      },
    },),
    it({
      name: 'LOGS THE RATE-LIMIT HEADERS ONCE, on the first reply of the run, and asks the transport for them and for '
        + 'retry-after on every exchange: the key lists 1000 requests, 2,000,000 input and 400,000 output tokens a '
        + 'minute, and a run should read its own limits off the wire rather than off a probe',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [headedReply({
            bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
            headers: {
              'anthropic-ratelimit-requests-limit': '1000',
              'anthropic-ratelimit-requests-remaining': '999',
              'anthropic-ratelimit-input-tokens-limit': '2000000',
              'anthropic-ratelimit-output-tokens-limit': '400000',
            },
          },),],
        },);
        const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);

        const { logged, } = await infoLinesDuring({
          run: async () => {
            await askCat({ client, modelId: 'claude-haiku-5-5', },);
            await askCat({ client, modelId: 'claude-haiku-5-5', },);
          },
        },);

        /**
         Lines that carry the headers.
         */
        const limitLines = logged.filter(function carriesLimits(line,): boolean {
          return line.includes('anthropic-ratelimit-',);
        },);
        expect(limitLines.length,).toBe(1,);
        const [limitLine = '',] = limitLines;
        expect(limitLine,).toContain('anthropic-ratelimit-requests-limit=1000',);
        expect(limitLine,).toContain('anthropic-ratelimit-requests-remaining=999',);
        expect(limitLine,).toContain('anthropic-ratelimit-input-tokens-limit=2000000',);
        expect(limitLine,).toContain('anthropic-ratelimit-output-tokens-limit=400000',);
        expect(limitLine,).not.toContain(CLAUDE_CAT_KEY,);
        for (const exchange of exchanges)
          expect(exchange.captureHeaders,).toEqual(['retry-after', 'anthropic-ratelimit-*',],);
      },
    },),

    it({
      name: 'PACES EVERY ATTEMPT THROUGH A REQUEST WINDOW of 1000 starts a minute, the key\'s stated rate: Haiku on the '
        + 'wide panel and the checker bench fans out hundreds of calls a run, and a window with no place left reads '
        + 'as a wait the router can see',
      fn: async () => {
        expect(CLAUDE_REQUESTS_PER_MINUTE,).toBe(1_000,);
        expect(CLAUDE_PACE_WINDOW_MS,).toBe(60_000,);
        const { transport, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
          },],
        },);
        const client = createClaudeClient({
          apiKey: CLAUDE_CAT_KEY,
          transport,
          retryPolicy: ONE_QUICK_RETRY,
          requestsPerMinute: 2,
        },);

        expect(client.pace.waitMs(),).toBe(0,);
        await askCat({ client, modelId: 'claude-haiku-5-5', },);
        expect(client.pace.waitMs(),).toBe(0,);
        await askCat({ client, modelId: 'claude-sonnet-5-5', },);
        expect(client.pace.waitMs(),).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'MARKS THE CLIENT\'S METER DRY FOR THE RUN after a billing error, in both shapes Anthropic has used (402 '
        + 'billing_error, and 400 with a "credit balance is too low" message), though a later call settles; a client '
        + 'that met none reads wet',
      fn: async () => {
        /**
         Signal every meter read carries.
         */
        const { signal, } = new AbortController();
        /**
         Bodies that mean the balance is empty, each with its status.
         */
        const billingReplies = [
          { status: 402, bodyText: '{"type":"error","error":{"type":"billing_error","message":"billing problem"}}', },
          {
            status: 400,
            bodyText: '{"type":"error","error":{"type":"invalid_request_error","message":"Your credit balance is too '
              + 'low to access the Anthropic API."}}',
          },
        ];
        await Promise.all(billingReplies.map(async function meetBilling(billing,) {
          const { transport, } = scripted({
            replies: [billing, {
              status: 200,
              bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
            },],
          },);
          const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);
          expect((await client.spend({ signal, },)).billingRefused,).toBe(false,);
          const error = await rejectionOf(() => askCat({ client, modelId: 'claude-haiku-5-5', },),);
          expect(error,).toBeInstanceOf(ProviderHttpError,);
          expect((await client.spend({ signal, },)).billingRefused,).toBe(true,);
          await askCat({ client, modelId: 'claude-haiku-5-5', },);
          expect((await client.spend({ signal, },)).billingRefused,).toBe(true,);
        },),);
      },
    },),
    it({
      name: 'CARRIES A retry-after PAST THE LADDER\'S REACH ONTO THE FAILURE, so the budget layer holds Anthropic '
        + 'out for the 3,600 s the header names and not for the 60 s default: the wait the router reads is the larger '
        + 'of the body\'s and the header\'s',
      fn: async () => {
        /**
         Rate-limit reply naming an hour in its header, and nothing in its body.
         */
        const refusal: TransportReply = {
          status: 429,
          bodyText: '{"type":"error","error":{"type":"rate_limit_error","message":"slow down"}}',
          headers: { 'retry-after': '3600', },
        };
        const { transport, exchanges, } = scripted({ replies: [refusal,], },);
        const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);

        const failure: unknown = await rejectionOf(() => askCat({ client, modelId: 'claude-haiku-5-5', },),);

        expect(failure,).toBeInstanceOf(ProviderHttpError,);
        expect(exchanges.length,).toBe(1,);
        expect(statedWaitMsOf({ error: failure, },),).toBe(3_600_000,);
        /**
         Clock the budgets read.
         */
        const clock = { now: 10_000, };
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          anthropic: client,
          freshForMs: 1_000,
          now: () => clock.now,
        },);
        await budgets.markRefused({
          provider: 'anthropic',
          signal: new AbortController().signal,
          statedWaitMs: statedWaitMsOf({ error: failure, },),
        },);
        expect(budgets.holds().anthropic,).toBe(3_600_000,);
      },
    },),

    it({
      name: 'NEVER ARMS A TIMER WITH A HEADER-STATED WAIT PAST THE TIMER\'S RANGE (R7): a retry-after of '
        + '99,999,999,999 seconds ends the ladder on its first reply instead of sleeping or firing at once (a delay '
        + 'over 2,147,483,647 ms is clamped to 1 ms by Node), and the hold the budgets keep is the whole stated wait, '
        + 'read by timestamp',
      fn: async () => {
        /**
         Rate-limit reply naming a wait no timer can hold, and nothing in its body.
         */
        const refusal: TransportReply = {
          status: 429,
          bodyText: '{"type":"error","error":{"type":"rate_limit_error","message":"slow down"}}',
          headers: { 'retry-after': '99999999999', },
        };
        const { transport, exchanges, } = scripted({ replies: [refusal,], },);
        const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);

        const failure: unknown = await rejectionOf(() => askCat({ client, modelId: 'claude-haiku-5-5', },),);

        expect(failure,).toBeInstanceOf(ProviderHttpError,);
        // One exchange: a clamped timer would have fired at once and retried against the scripted refusal.
        expect(exchanges.length,).toBe(1,);
        expect(statedWaitMsOf({ error: failure, },),).toBe(99_999_999_999_000,);
        /**
         Clock the budgets read.
         */
        const clock = { now: 10_000, };
        // A BOUND PAST THE STATED WAIT, so the budgets keep the whole of it and the case reads it by timestamp.
        const budgets = createProviderBudgets({
          maxHoldMs: Number.MAX_SAFE_INTEGER,
          anthropic: client,
          freshForMs: 1_000,
          now: () => clock.now,
        },);
        await budgets.markRefused({
          provider: 'anthropic',
          signal: new AbortController().signal,
          statedWaitMs: statedWaitMsOf({ error: failure, },),
        },);
        expect(budgets.holds().anthropic,).toBe(99_999_999_999_000,);
      },
    },),

    it({
      name: 'RAISES THE DRY FLAG FOR A billing_error EVENT UNDER A SUCCESS STATUS, which the retry ladder returns '
        + 'unretried as a thrown 402: the meter reads dry after it as after a plain HTTP 402',
      fn: async () => {
        /**
         Signal every meter read carries.
         */
        const { signal, } = new AbortController();
        const { transport, } = scripted({
          replies: [{
            status: 200,
            bodyText: anthropicFrameOf({
              body: { type: 'error', error: { type: 'billing_error', message: 'billing problem', }, },
            },),
          },],
        },);
        const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);
        expect((await client.spend({ signal, },)).billingRefused,).toBe(false,);

        const failure: unknown = await rejectionOf(() => askCat({ client, modelId: 'claude-haiku-5-5', },),);

        expect(failure,).toBeInstanceOf(ProviderHttpError,);
        expect((failure as ProviderHttpError).status,).toBe(402,);
        expect((await client.spend({ signal, },)).billingRefused,).toBe(true,);
      },
    },),

    it({
      name: 'HONOURS A 429 THE HEADER ASKS ONE SECOND FOR end to end through the client: the retry waits the '
        + 'second and the call answers',
      fn: async () => {
        /**
         The first reply with the header the transport hands back when the exchange asks for it.
         */
        const waiting: TransportReply = {
          status: 429,
          bodyText: '{"type":"error","error":{"type":"rate_limit_error"}}',
          headers: { 'retry-after': '1', },
        };
        const { transport, exchanges, } = scripted({
          replies: [
            waiting,
            { status: 200, bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }), },
          ],
        },);
        /**
         When the call started.
         */
        const startedAt = performance.now();

        const reply = await askCat({
          client: createClaudeClient({
            apiKey: CLAUDE_CAT_KEY,
            transport,
            retryPolicy: { limit: 2, baseMs: 300, },
          },),
          modelId: 'claude-haiku-5-5',
        },);

        expect(reply.text,).toBe('{"spot":"sunbeam"}',);
        expect(exchanges.length,).toBe(2,);
        expect(performance.now() - startedAt,).toBeGreaterThanOrEqual(990,);
      },
    },),

    it({
      name: 'RETRIES A 529 OVERLOADED REPLY through the client and answers from the second attempt',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [
            { status: 529, bodyText: '{"type":"error","error":{"type":"overloaded_error","message":"busy"}}', },
            { status: 200, bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }), },
          ],
        },);

        const reply = await askCat({
          client: createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },),
          modelId: 'claude-sonnet-5-5',
        },);

        expect(reply.text,).toBe('{"spot":"sunbeam"}',);
        expect(exchanges.length,).toBe(2,);
      },
    },),

    it({
      name: 'READS A max_tokens STOP AFTER A THINKING BLOCK AS A TRUNCATED VOICE: empty text, finish reason '
        + 'max_tokens, the spent tokens on a SPEND line, and the truncated outcome for the caller, with no retry',
      fn: async () => {
        resetRunSpend();
        const { transport, exchanges, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({
              text: '',
              stopReason: 'max_tokens',
              usage: { input: 100, cacheWrite: 0, cacheRead: 0, output: COMPLETION_CAP['claude-sonnet-5-5'], },
            }),
          },],
        },);
        const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);

        const { result: outcome, logged, } = await infoLinesDuring({
          run: async () =>
            await client.chatJson({
              modelId: 'claude-sonnet-5-5',
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: new AbortController().signal,
              validate: function isSpot(value: unknown,): value is { readonly spot: string; } {
                return (typeof value) === 'object';
              },
            },),
        },);

        expect(outcome.kind,).toBe('schema-mismatch',);
        expect((outcome as { readonly reason?: string; }).reason,).toBe('truncated-completion',);
        expect(exchanges.length,).toBe(1,);
        expect(logged.filter(function isSpend(line,): boolean {
          return line.includes('SPEND ',) && line.includes(`completion=${String(COMPLETION_CAP['claude-sonnet-5-5'],)}`,);
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'STOPS A CARD EDITED PAST THE TYPES, the lock\'s third line: a wire model outside the two granted and an '
        + 'effort outside the three both refuse the call at the client, before a request is built',
      fn: async () => {
        /**
         The Haiku row of the catalog, typed loosely so the case can edit it past what the types allow.
         */
        const row = CLAUDE_MODELS['claude-haiku-5-5'] as { wireModel: string; effort?: string; };
        /**
         What the row holds.
         */
        const original = { ...row, };
        /**
         Puts the row back when the case ends, however it ends.
         */
        using restoreRow = {
          [Symbol.dispose](): void {
            row.wireModel = original.wireModel;
            if (original.effort === undefined)
              delete row.effort;
            else
              row.effort = original.effort;
          },
        };
        const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: '', },], },);
        const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);
        row.wireModel = 'claude-opus-5-5';
        const wrongModel: unknown = await rejectionOf(() => askCat({ client, modelId: 'claude-haiku-5-5', },),);
        row.wireModel = original.wireModel;
        row.effort = 'xhigh';
        const wrongEffort: unknown = await rejectionOf(() => askCat({ client, modelId: 'claude-haiku-5-5', },),);
        expect(wrongModel,).toBeInstanceOf(ClaudeModelNotServedError,);
        expect((wrongModel as Error).message,).toContain('claude-opus-5-5',);
        expect(wrongEffort,).toBeInstanceOf(ClaudeModelNotServedError,);
        expect((wrongEffort as Error).message,).toContain('effort xhigh',);
        expect(exchanges.length,).toBe(0,);
      },
    },),

    it({
      name: 'REFUSES A SCHEMA THAT WRITES oneOf WITH ClaudeSchemaKeywordError AND MAKES NO EXCHANGE (F4 of the '
        + 're-check of the second fix round): the refusal leaves chatText as the named class, so the caller sees a '
        + 'programming fault and not an HTTP 400',
      fn: async () => {
        const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: '', },], },);
        const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);
        const refusal: unknown = await rejectionOf(() =>
          client.chatText({
            modelId: 'claude-haiku-5-5',
            messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
            signal: new AbortController().signal,
            responseFormat: {
              type: 'json_schema',
              json_schema: {
                name: 'cat_spot',
                strict: true,
                schema: {
                  type: 'object',
                  properties: { spot: { oneOf: [{ type: 'string', }, { type: 'integer', },], }, },
                  required: ['spot',],
                  additionalProperties: false,
                },
              },
            },
          },),
        );
        expect(refusal,).toBeInstanceOf(ClaudeSchemaKeywordError,);
        expect((refusal as Error).message,).toContain('"oneOf"',);
        expect(exchanges.length,).toBe(0,);
      },
    },),
  ],
},);
