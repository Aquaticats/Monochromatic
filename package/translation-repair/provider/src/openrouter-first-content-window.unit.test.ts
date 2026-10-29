/**
 Tests that the OpenRouter client hands the transport the first-content window
 of the card it serves, and that the one card naming a window names a defensible
 one.

 WHY: in `XingZ6017` twenty-six rounds waited 250 to 360 seconds on an
 `inception/mercury-2.5` stream (two rounds on two in a row) that sent `: OPENROUTER PROCESSING` comment
 lines and no text until the gateway ended it with HTTP 502 at about 301 s
 (ledger B809). A round whose quorum is every live seat cannot start its grace,
 so only the stream guard can end such a call, and its shared window is 180 s,
 set against first bytes of providers that send no comment lines (ledger B804).
 Across 24 pass logs the 4,636 Mercury streams that delivered text ended inside
 56,772 ms, and no Mercury stream that sent only comment lines for a minute or
 more ever delivered text afterwards.

 WHAT THESE PIN: the Mercury exchange carries the card's window, an exchange of
 a card that names none carries none, and the card's window sits at least half
 again over the longest Mercury stream that delivered text and under the shared
 window.

 Fixtures are cat-themed invention. No call is sent.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createOpenRouterClient, } from '../dist/final/node/index.mjs';
import { STREAM_FIRST_CONTENT_MS, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { MODEL_CARDS, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { TransportExchange, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  keepingTransport,
  settled,
} from './recording-transport.test-fixture.ts';
import {
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Longest elapsed time of a Mercury stream that delivered text, over the pass
 logs `CuspariaKLSY16` to `XingZ6017` (24 logs, 4,636 streams), read on
 2026-10-22 (UTC). The time to the first text is no longer than this.
 */
const LONGEST_MERCURY_TEXT_STREAM_MS = 56_772;

/**
 Messages of the one call each case asks for.
 */
const MESSAGES = [{ role: 'user', content: 'Where does the cat nap?', },] as const;

/**
 Exchange the client hands its transport for one call to a seat.

 @param modelId - seat asked

 @returns Exchange the transport was handed

 @throws {@link Error} when the client sent nothing

 @example
 ```ts
 const exchange = await exchangeFor({ modelId: SEAT_OPENROUTER_ONLY, },);
 ```
 */
async function exchangeFor(
  { modelId, }: { readonly modelId: typeof SEAT_OPENROUTER_ONLY | typeof SEAT_OPENROUTER_ONLY_CHECKER; },
): Promise<TransportExchange> {
  /**
   Exchanges the transport was handed.
   */
  const seen: TransportExchange[] = [];
  /**
   Client over a transport that keeps what it is handed and refuses it.
   */
  const client = createOpenRouterClient({
    apiKey: 'whiskers',
    transport: keepingTransport({ seen, },),
    retryPolicy: { limit: 0, baseMs: 1, },
  },);
  await settled(client.chatText({
    modelId,
    messages: MESSAGES,
    signal: new AbortController().signal,
  },),);
  const [exchange,] = seen;
  if (exchange === undefined)
    throw new Error('the client handed its transport no exchange',);
  return exchange;
}

await describe({
  name: 'the first-content window the OpenRouter client hands its transport',
  concurrency: 1,
  children: [
    it({
      name: 'HANDS THE TRANSPORT THE WINDOW OF THE MERCURY CARD',
      fn: async () => {
        const exchange = await exchangeFor({ modelId: SEAT_OPENROUTER_ONLY, },);
        expect(exchange.firstContentMs,).toBe(MODEL_CARDS[SEAT_OPENROUTER_ONLY].openrouter?.firstContentMs,);
        expect(exchange.firstContentMs,).toBeDefined();
      },
    },),

    it({
      name: 'HANDS A CARD THAT NAMES NO WINDOW NONE, so the shared window stays its own',
      fn: async () => {
        const exchange = await exchangeFor({ modelId: SEAT_OPENROUTER_ONLY_CHECKER, },);
        expect('firstContentMs' in exchange,).toBe(false,);
      },
    },),

    it({
      name: 'KEEPS THE MERCURY WINDOW HALF AGAIN OVER THE LONGEST STREAM THAT DELIVERED TEXT, AND UNDER THE SHARED WINDOW',
      fn: async () => {
        await Promise.resolve();
        const window = MODEL_CARDS[SEAT_OPENROUTER_ONLY].openrouter?.firstContentMs;
        expect(window,).toBeDefined();
        expect(window as number,).toBeGreaterThanOrEqual(LONGEST_MERCURY_TEXT_STREAM_MS * 1.5,);
        expect(window as number,).toBeLessThan(STREAM_FIRST_CONTENT_MS,);
      },
    },),
  ],
},);
