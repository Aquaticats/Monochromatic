/**
 Tests for the question the model health command asks one model, and for the
 lines it logs about the answer.

 THE PROBE EXISTS TO TELL TWO FAULTS APART: a model that answered badly and a
 model that could not be asked. This file pins the first half, the lines an
 answer of every kind leaves, and that a call which fails is not answered here
 but handed to the caller to report.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  claudeModelClientOver,
  claudeStreamOf,
  PLAIN_USAGE,
} from '@monochromatic-dev/translation-repair-provider/ts/claude-client.test-fixture.ts';
import {
  transportLandingAfter,
} from '@monochromatic-dev/translation-repair-provider/ts/delayed-transport.test-fixture.ts';
import {
  walkOnFakeClock,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/fake-clock-walk.test-fixture.ts';
import { askModelHealth, } from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  modelReplyScriptedClient,
  type ScriptedModelReply,
} from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/model-reply-scripted-client.test-fixture.ts';

/**
 Model every case asks.
 */
const MODEL = 'hf:Qwen/Qwen3.8-27B' as const;

/**
 Timeout every case hands the probe, which it must pass on to the call.
 */
const TIMEOUT_MS = 4_321;

/**
 Units the raw reply opening is cut to.
 */
const PREVIEW_UNITS = 300;

/**
 Text of a reply in the shape asked for.
 */
const HEALTHY_TEXT = '{"count":2,"first":"Mittens"}';

/**
 Reply in the shape asked for, as the scripted client answers it.
 */
const HEALTHY: ScriptedModelReply = {
  kind: 'ok',
  text: HEALTHY_TEXT,
};

/**
 Asks the probe about the scripted model.

 @param reply - what the model is scripted to do

 @returns Lines logged and requests the client saw

 @example
 ```ts
 const { lines, requests, } = await askWith({ reply: HEALTHY, },);
 ```
 */
async function askWith(
  { reply, }: { readonly reply: ScriptedModelReply; },
): Promise<{
  readonly lines: readonly string[];
  readonly requests: ReturnType<typeof modelReplyScriptedClient>['requests'];
}> {
  const { client, requests, } = modelReplyScriptedClient({ replies: new Map([[MODEL, reply,],],), },);
  const { logger, lines, } = capturingLoggerPair();
  await askModelHealth({
    client,
    modelId: MODEL,
    timeoutMs: TIMEOUT_MS,
    l: logger,
  },);
  return {
    lines,
    requests,
  };
}

/**
 The per-call figure the run hands every call, which a caller-side cut would use.
 */
const RUN_PER_CALL_MS = 360_000;

/**
 The Claude deadline the cap, the floor rate and the allowance give.
 */
const CLAUDE_DEADLINE_MS = 460_000;

/**
 Longest the fake clock is ticked for a reply that is meant to be heard: past the Claude deadline and a minute more.
 */
const LONGEST_TICKING_MS = 520_000;

/**
 Asks the probe about a Claude seat whose reply lands after a delay, on a fake clock, ticking the given steps.

 @param landMs - delay before the scripted reply lands

 @param ticks - fake milliseconds to advance, one step after another

 @param waitForEnd - whether to wait for the probe to end once the ticking is done, which a probe meant to outlast the
 ticking does not

 @param sinon - case's own sandbox

 @returns Lines logged, the reasons exchanges were cut for, the cuts counted after each tick, whether the probe ended by
 itself, and what the probe threw

 @example
 ```ts
 const asked = await askClaudeLandingAfter({
   landMs: 400_000,
   ticks: [LONGEST_TICKING_MS,],
   waitForEnd: true,
   sinon: ctx.sinon,
 },);
 ```
 */
async function askClaudeLandingAfter(
  {
    landMs,
    ticks,
    waitForEnd,
    sinon,
  }: {
    readonly landMs: number;
    readonly ticks: readonly number[];
    readonly waitForEnd: boolean;
    readonly sinon: Parameters<typeof walkOnFakeClock>[0]['sinon'];
  },
): Promise<{
  readonly lines: readonly string[];
  readonly cut: readonly unknown[];
  readonly cutsAfterEachTick: readonly number[];
  readonly endedByItself: boolean;
  readonly thrown: readonly unknown[];
}> {
  const { transport, firstCall, cut, release, } = transportLandingAfter({
    landMs,
    reply: {
      status: 200,
      bodyText: claudeStreamOf({ text: HEALTHY_TEXT, stopReason: 'end_turn', usage: PLAIN_USAGE, },),
    },
  },);
  const client = claudeModelClientOver({ transport, },);
  const { logger, lines, } = capturingLoggerPair();

  /**
   What the probe threw, in a list of none or one.
   */
  const thrown: unknown[] = [];
  const walk = await walkOnFakeClock({
    sinon,
    walkOnce: async function askOnce(): Promise<void> {
      try {
        await askModelHealth({
          client,
          modelId: 'claude-haiku-5-5',
          timeoutMs: RUN_PER_CALL_MS,
          l: logger,
        },);
      }
      catch (error) {
        thrown.push(error,);
      }
    },
    firstCall,
    release,
    ticks,
    cutCount: () => cut.length,
    waitForEnd,
  },);
  return {
    lines,
    cut,
    cutsAfterEachTick: walk.cutsAfterEachTick,
    endedByItself: walk.endedByItself,
    thrown,
  };
}

await describe({
  name: askModelHealth.name,
  // SEQUENTIAL: one case spies on the process-wide signal factory.
  concurrency: 1,
  children: [
    it({
      name: 'ARMS NO CALLER-SIDE TIMEOUT SIGNAL (F1 of the re-check of the second fix round): the client arms the '
        + 'deadline '
        + 'from `exchangeTimeoutMs`, and a signal that aborts at the same figure would cut a Claude call before the '
        + 'Claude deadline',
      fn: async (ctx) => {
        const timeoutSignal = ctx.sinon.spy(
          AbortSignal,
          'timeout',
        );
        const { requests, } = await askWith({ reply: HEALTHY, },);

        expect(requests[0]?.exchangeTimeoutMs,).toBe(TIMEOUT_MS,);
        expect(timeoutSignal.callCount,).toBe(0,);
      },
    },),
    it({
      name: 'HEARS A CLAUDE REPLY THAT LANDS AT 400,000 MS: the probe hands '
        + 'the client no signal that aborts before the Claude deadline of 460,000 ms, whether it is built from '
        + 'AbortSignal.timeout, from setTimeout or from AbortSignal.any',
      fn: async (ctx) => {
        const asked = await askClaudeLandingAfter({
          landMs: 400_000,
          ticks: [LONGEST_TICKING_MS,],
          waitForEnd: true,
          sinon: ctx.sinon,
        },);

        expect(asked.thrown,).toEqual([],);
        expect(asked.cut,).toEqual([],);
        expect(asked.endedByItself,).toBe(true,);
        expect(asked.lines[0],).toBe('claude-haiku-5-5: ok',);
      },
    },),
    it({
      name: 'CUTS A CLAUDE REPLY THAT LANDS AT 500,000 MS AT 460,000 MS, not before (the positive control): the '
        + 'rig shows the client\'s own deadline firing, so the heard reply at 400,000 ms is a measurement',
      fn: async (ctx) => {
        const asked = await askClaudeLandingAfter({
          landMs: 500_000,
          ticks: [CLAUDE_DEADLINE_MS - 1, 1,],
          waitForEnd: false,
          sinon: ctx.sinon,
        },);

        expect(asked.cutsAfterEachTick,).toEqual([0, 1,],);
      },
    },),
    it({
      name: 'LOGS the outcome kind and the whole raw reply for an answer in the shape asked for',
      fn: async () => {
        const { lines, } = await askWith({ reply: HEALTHY, },);

        expect(lines,).toEqual([
          `${MODEL}: ok`,
          `${MODEL}: raw reply opening (at most 300 UTF-16 units): "{\\"count\\":2,\\"first\\":\\"Mittens\\"}"`,
        ],);
      },
    },),
    it({
      name: 'ASKS the model it was given the cat question, in the schema and with the timeout it was handed',
      fn: async () => {
        const { requests, } = await askWith({ reply: HEALTHY, },);

        expect(requests.length,).toBe(1,);
        expect(requests[0]?.modelId,).toBe(MODEL,);
        expect(requests[0]?.exchangeTimeoutMs,).toBe(TIMEOUT_MS,);
        expect(requests[0]?.messages,).toEqual([
          {
            role: 'user',
            content: 'Reply with JSON matching the schema: how many cats are named in this sentence, '
              + 'and what is the first one called? "Mittens and Tabby sat on the windowsill."',
          },
        ],);
        expect(requests[0]?.responseFormat,).toEqual({
          type: 'json_schema',
          json_schema: {
            name: 'cat_count',
            strict: true,
            schema: {
              type: 'object',
              additionalProperties: false,
              required: [
                'count',
                'first',
              ],
              properties: {
                count: { type: 'number', },
                first: { type: 'string', },
              },
            },
          },
        },);
      },
    },),
    it({
      name: 'ADMITS a reply carrying both fields whatever their values, and refuses one missing a field or not a record',
      fn: async () => {
        const { requests, } = await askWith({ reply: HEALTHY, },);
        const validate = requests[0]?.validate;

        expect(validate?.({
          count: 2,
          first: 'Mittens',
        },),).toBe(true,);
        expect(validate?.({
          count: 'two',
          first: 7,
        },),).toBe(true,);
        expect(validate?.({ count: 2, },),).toBe(false,);
        expect(validate?.({ first: 'Mittens', },),).toBe(false,);
        expect(validate?.(['count', 'first',],),).toBe(false,);
        expect(validate?.(null,),).toBe(false,);
        expect(validate?.('Mittens',),).toBe(false,);
      },
    },),
    it({
      name: 'LOGS the detail beside the kind of a reply the client found unreadable, then its raw opening',
      fn: async () => {
        const { lines, } = await askWith({
          reply: {
            kind: 'schema-mismatch',
            rawText: 'Here you go: {"count":2}',
            detail: 'reply is not JSON',
          },
        },);

        expect(lines,).toEqual([
          `${MODEL}: schema-mismatch -- reply is not JSON`,
          `${MODEL}: raw reply opening (at most 300 UTF-16 units): "Here you go: {\\"count\\":2}"`,
        ],);
      },
    },),
    it({
      name: 'LOGS the kind alone, with no detail, and the raw opening for a reply that reads as a refusal',
      fn: async () => {
        const { lines, } = await askWith({
          reply: {
            kind: 'refusal-shaped',
            rawText: 'The cat declines.',
          },
        },);

        expect(lines,).toEqual([
          `${MODEL}: refusal-shaped`,
          `${MODEL}: raw reply opening (at most 300 UTF-16 units): "The cat declines."`,
        ],);
      },
    },),
    it({
      name: 'CUTS a raw reply longer than the preview to its first 300 units',
      fn: async () => {
        const { lines, } = await askWith({
          reply: {
            kind: 'schema-mismatch',
            rawText: `${'a'.repeat(PREVIEW_UNITS,)}${'b'.repeat(PREVIEW_UNITS,)}`,
            detail: 'reply is not JSON',
          },
        },);

        expect(lines[1],).toBe(
          `${MODEL}: raw reply opening (at most 300 UTF-16 units): "${'a'.repeat(PREVIEW_UNITS,)}"`,
        );
      },
    },),
    it({
      name: 'LEAVES a raw reply of exactly 300 units whole',
      fn: async () => {
        const { lines, } = await askWith({
          reply: {
            kind: 'schema-mismatch',
            rawText: 'a'.repeat(PREVIEW_UNITS,),
            detail: 'reply is not JSON',
          },
        },);

        expect(lines[1],).toBe(
          `${MODEL}: raw reply opening (at most 300 UTF-16 units): "${'a'.repeat(PREVIEW_UNITS,)}"`,
        );
      },
    },),
    it({
      name: 'CUTS before a cat emoji whose two halves the 300-unit limit would separate',
      fn: async () => {
        const { lines, } = await askWith({
          reply: {
            kind: 'schema-mismatch',
            rawText: `${'a'.repeat(PREVIEW_UNITS - 1,)}\u{1F431}tail`,
            detail: 'reply is not JSON',
          },
        },);

        expect(lines[1],).toBe(
          `${MODEL}: raw reply opening (at most 300 UTF-16 units): "${'a'.repeat(PREVIEW_UNITS - 1,)}"`,
        );
      },
    },),
    it({
      name: 'REJECTS with the very value the call threw and logs nothing, leaving the report to the caller',
      fn: async () => {
        const failure = new TypeError('the cat flap is shut',);
        const { client, } = modelReplyScriptedClient({
          replies: new Map([[
            MODEL,
            {
              kind: 'throws',
              error: failure,
            },
          ],],),
        },);
        const { logger, lines, } = capturingLoggerPair();

        const refusal = await rejectionOf(async function askThrowingModel(): Promise<void> {
          await askModelHealth({
            client,
            modelId: MODEL,
            timeoutMs: TIMEOUT_MS,
            l: logger,
          },);
        },);

        expect(refusal,).toBe(failure,);
        expect(lines,).toEqual([],);
      },
    },),
  ],
},);
