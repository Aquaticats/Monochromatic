/**
 Tests for the deadline a Claude call is armed with (R1 of the second review of the Claude route). The run hands every
 call 360,000 ms; the Claude cards cap a completion at 32,000 tokens, which at the floor rate of 80 tokens a second and
 a minute for the prompt and the first byte needs 460,000 ms. The cases read the length of the timer the client arms,
 without waiting for it, and NOTHING HERE REACHES A PROVIDER: every transport is an in-memory script and every key is
 invented. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { COMPLETION_CAP, } from '@monochromatic-dev/translation-repair-shared-contract';
import { recordedTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts/recorded-transport.test-fixture.ts';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  createClaudeClient,
  createSyntheticClient,
} from '../dist/final/node/index.mjs';
import { armedDeadlinesDuring, } from './armed-deadlines.test-fixture.ts';
import {
  CLAUDE_CAT_KEY,
  claudeStreamOf,
  PLAIN_USAGE,
} from './claude-client.test-fixture.ts';
import {
  ONE_QUICK_RETRY,
  scripted,
} from './direct-client.test-fixture.ts';

/**
 The deadline the run hands every call, as `RUN_PER_CALL_TIMEOUT_MS` in the run package holds it.
 */
const RUN_DEADLINE_MS = 360_000;

/**
 A caller deadline longer than the Claude one, which the client must not shorten.
 */
const LONGER_DEADLINE_MS = 900_000;

/**
 The Claude deadline the cap, the floor rate and the allowance give: 32,000 tokens at 80 a second is 400 s, and a
 minute more.
 */
const CLAUDE_DEADLINE_MS = 460_000;

/**
 Asks the invented question of one Claude card with a caller deadline and reports the deadlines the client armed.

 @param exchangeTimeoutMs - deadline the caller hands the call
 @param maxTokens - ceiling the caller asks for, absent for the card's own cap

 @returns Delays armed at deadline length

 @example
 ```ts
 const armed = await armedForClaude({ exchangeTimeoutMs: 360_000, },);
 ```
 */
async function armedForClaude(
  {
    exchangeTimeoutMs,
    maxTokens,
  }: {
    readonly exchangeTimeoutMs: number;
    readonly maxTokens?: number;
  },
): Promise<readonly number[]> {
  const { transport, } = scripted({
    replies: [{
      status: 200,
      bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
    },],
  },);
  const client = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);
  const { armed, } = await armedDeadlinesDuring({
    run: async () =>
      await client.chatText({
        modelId: 'claude-sonnet-5-5',
        messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
        signal: new AbortController().signal,
        exchangeTimeoutMs,
        // Conditional spread keeps the knob absent instead of undefined.
        ...((maxTokens === undefined) ? {} : { maxTokens, }),
      },),
  },);
  return armed;
}

await describe({
  name: 'the Claude call deadline',
  // SEQUENTIAL: the fixture wraps the process-wide timer function, so cases must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'ARMS 460,000 MS FOR A CLAUDE CALL THE RUN GAVE 360,000 (R1): the 32,000-token cap at the floor rate of 80 '
        + 'tokens a second plus the allowance for the prompt and the first byte, and not the 360,000 the cap cannot be '
        + 'reached inside below 89 tokens a second',
      fn: async () => {
        const armed = await armedForClaude({ exchangeTimeoutMs: RUN_DEADLINE_MS, },);
        expect(armed,).toEqual([CLAUDE_DEADLINE_MS,],);
        expect(COMPLETION_CAP['claude-sonnet-5-5'],).toBe(32_000,);
      },
    },),

    it({
      name: 'NEVER SHORTENS A LONGER DEADLINE THE CALLER GAVE: the Claude deadline is a floor under the caller\'s',
      fn: async () => {
        expect(await armedForClaude({ exchangeTimeoutMs: LONGER_DEADLINE_MS, },),).toEqual([LONGER_DEADLINE_MS,],);
      },
    },),

    it({
      name: 'SIZES THE DEADLINE FROM THE CEILING THE CALL ASKS FOR: 800 tokens need 70,000 ms, which the run\'s '
        + '360,000 already covers, so the caller\'s stands',
      fn: async () => {
        expect(await armedForClaude({ exchangeTimeoutMs: RUN_DEADLINE_MS, maxTokens: 800, },),)
          .toEqual([RUN_DEADLINE_MS,],);
      },
    },),

    it({
      name: 'KEEPS 360,000 MS FOR A CALL TO ANY OTHER PROVIDER (the control arm of R1): the Synthetic client arms the '
        + 'deadline the run gave it',
      fn: async () => {
        const { transport, } = recordedTransport({
          replies: [{
            status: 200,
            bodyText: 'data: {"choices":[{"delta":{"content":"{\\"spot\\":\\"sunbeam\\"}"}}]}\n\ndata: [DONE]\n\n',
          },],
        },);
        const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
        const { armed, } = await armedDeadlinesDuring({
          run: async () =>
            await client.chatText({
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: new AbortController().signal,
              exchangeTimeoutMs: RUN_DEADLINE_MS,
            },),
        },);
        expect(armed,).toEqual([RUN_DEADLINE_MS,],);
      },
    },),
  ],
},);
