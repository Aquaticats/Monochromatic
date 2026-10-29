/**
 Tests that the route a probe builds hands the Claude client the caller's admit for each attempt.

 A probe that holds an arm to an amount has to look again before every attempt the Claude client makes inside one call,
 and the client is built three layers down (`runClientFrom`, `configureProviders`, `createClaudeClient`), so the admit
 has to travel that route without changing any request. Every transport here is an in-memory script and every key is
 invented, so nothing reaches a provider.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { anthropicFrameOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';
import {
  CLAUDE_CAT_KEY,
  claudeStreamOf,
  PLAIN_USAGE,
} from '@monochromatic-dev/translation-repair-provider/ts/claude-client.test-fixture.ts';
import { scripted, } from '@monochromatic-dev/translation-repair-provider/ts/direct-client.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import { withFreshRunSeats, } from '@monochromatic-dev/translation-repair-stage/ts/corpus-run/fresh-run-seats.test-fixture.ts';
import {
  CLAUDE_KEY_VARIABLE,
  configureProviders,
  runClientFrom,
} from '../../dist/final/node/index.mjs';

/**
 The card every case asks.
 */
const CARD = 'claude-sonnet-5-5-effort-low';

/**
 A whole stream with its terminator frame cut away, which is what a dropped connection leaves.
 */
const CUT = claudeStreamOf({ text: '{"spot":"sun', stopReason: 'end_turn', usage: PLAIN_USAGE, })
  .replace(anthropicFrameOf({ body: { type: 'message_stop', }, },), '',);

/**
 A whole stream that answers.
 */
const WHOLE = claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, });

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: configureProviders.name,
      concurrency: 1,
      children: [
        it({
          name: 'HANDS THE CLAUDE CLIENT THE CALLER\'S ADMIT: the one attempt of a whole reply is told before its request '
            + 'is sent, with the card it is for',
          fn: async () => {
            const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: WHOLE, },], },);
            /**
             What the caller was told, with the requests sent when it was told.
             */
            const asks: { readonly label: string; readonly sent: number; }[] = [];
            const providers = configureProviders({
              maxHoldMs: TEST_MAX_HOLD_MS,
              env: { [CLAUDE_KEY_VARIABLE]: CLAUDE_CAT_KEY, },
              transport,
              off: [],
              admitClaudeAttempt: function noteAsk({ label, },): void {
                asks.push({ label, sent: exchanges.length, },);
              },
            },);
            await providers.anthropic?.chatText({
              modelId: CARD,
              messages: [{ role: 'user', content: 'Where does the cat nap?', },],
              signal: new AbortController().signal,
            },);
            expect(asks,).toStrictEqual([{ label: CARD, sent: 0, },],);
            expect(exchanges.length,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: runClientFrom.name,
      concurrency: 1,
      children: [
        it({
          name: 'TELLS THE CALLER OF EVERY ATTEMPT A ROUTED CALL MAKES: a cut stream and a whole one are two tellings, '
            + 'one ahead of each request, and the call still answers',
          fn: async () => {
            using _fresh = withFreshRunSeats();
            const { transport, exchanges, } = scripted({
              replies: [{ status: 200, bodyText: CUT, }, { status: 200, bodyText: WHOLE, },],
            },);
            /**
             What the caller was told, with the requests sent when it was told.
             */
            const asks: { readonly label: string; readonly sent: number; }[] = [];
            const client = runClientFrom({
              env: { [CLAUDE_KEY_VARIABLE]: CLAUDE_CAT_KEY, },
              transport,
              maxHoldMs: TEST_MAX_HOLD_MS,
              off: [],
              admitClaudeAttempt: function noteAsk({ label, },): void {
                asks.push({ label, sent: exchanges.length, },);
              },
            },);
            const outcome = await client.chatJson({
              modelId: CARD,
              messages: [{ role: 'user', content: 'Where does the cat nap?', },],
              signal: new AbortController().signal,
              validate: (value: unknown,): value is { readonly spot: string; } => (typeof value) === 'object',
            },);
            expect(outcome.kind,).toBe('ok',);
            expect(asks,).toStrictEqual([{ label: CARD, sent: 0, }, { label: CARD, sent: 1, },],);
          },
        },),
      ],
    },),
  ],
},);
