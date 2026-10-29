/**
 Tests for asking again when a reader's reply was cut by the completion limit
 before it wrote a single character of an answer.

 WHY THIS EXISTS: XingZ6016 (2026-10-21) stopped for want of a second
 reading because `glm-5.3-flash` on NeuralWatt reasoned at the provider's
 default effort until the completion cap cut it: `text: ""`, `finishReason:
 "length"`, 18316 completion tokens, 0 content characters. An empty reply was
 never asked again, only a refusal was, so the reader's one reading was lost.
 A cut reasoning run is a property of the roll and of the serving stack, which
 is the case a second ask on another provider that serves the model can win.

 WHAT THESE PIN: only an empty reply that the length limit ended is asked
 once more; it is asked on another provider where one is wet and on the same
 provider otherwise; a second cut stays a failed reading; an empty reply with
 no cut reason is still asked once, as before.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type ChatTextReply,
  type ChatTextRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { readPastRefusal, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Reader used by every case.
 */
const READER: RosterModelId = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 What a reader returns when it reads.
 */
const READING = '走失猫咪 Mittens，虎斑，请电 555 0134。';

/**
 Reply the completion cap ended while the model was still reasoning.
 */
const CUT: ChatTextReply = {
  text: '',
  finishReason: 'length',
  servedBy: 'neuralwatt',
};

/**
 What a recorded ask reads as when it carried no `otherThan` hint.
 */
const NO_HINT = 'no hint';

/**
 One scripted step: a reply, or the router's refusal to serve it elsewhere.
 */
type Step = ChatTextReply | 'nowhere-else';

/**
 Client that answers from a script and records what each ask carried.

 @param steps - what each ask gets, in order; the last one repeats

 @returns Client, the `otherThan` hint of each ask (or the no-hint word), and each ask's prompt

 @example
 ```ts
 const { client, hints, } = scriptedClient({ steps: [CUT, { text: READING, },], },);
 ```
 */
function scriptedClient(
  { steps, }: { readonly steps: readonly Step[]; },
): {
  readonly client: ModelClient;
  readonly hints: string[];
  readonly prompts: string[];
} {
  /**
   The `otherThan` hint of each ask.
   */
  const hints: string[] = [];
  /**
   Prompt text of each ask.
   */
  const prompts: string[] = [];
  return {
    hints,
    prompts,
    client: {
      chatText: async (request,) => {
        /**
         Request as this package's own contract describes it.
         */
        const asked = request as ChatTextRequest;
        hints.push(asked.otherThan ?? NO_HINT,);
        prompts.push(asked.messages
          .map(function text(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',),);

        /**
         Step for this ask, holding at the last scripted one.
         */
        const step = steps[hints.length - 1] ?? steps.at(-1,) ?? { text: '', };
        await Promise.resolve();
        if (step === 'nowhere-else') {
          throw new NoProviderForModelError({
            modelId: asked.modelId,
            reason: 'no provider other than neuralwatt serves it with budget',
          },);
        }
        return step;
      },
      chatJson: async () => {
        throw new Error('chatJson unused by the reading stage',);
      },
      quotas: async () => {
        throw new Error('quotas unused by the reading stage',);
      },
    },
  };
}

/**
 Logger keeping every line it is given.

 @param said - where the lines go

 @returns Logger writing to `said`

 @example
 ```ts
 const l = keepingLogger({ said, },);
 ```
 */
function keepingLogger({ said, }: { readonly said: string[]; },): Logger {
  /**
   Keeps one line.

   @param message - line written
   */
  const keep = (message: string,): void => {
    said.push(message,);
  };
  return {
    debug: keep,
    error: keep,
    fatal: keep,
    flush: async () => undefined,
    info: keep,
    trace: keep,
    warn: keep,
  };
}

/**
 Reads the picture once, past whatever the script gives.

 @param client - scripted client
 @param l - logger

 @returns Reading

 @example
 ```ts
 const reading = await readOnce({ client, l, },);
 ```
 */
async function readOnce(
  {
    client,
    l,
  }: {
    readonly client: ModelClient;
    readonly l: Logger;
  },
): ReturnType<typeof readPastRefusal> {
  return await readPastRefusal({
    client,
    modelId: READER,
    bytes: new Uint8Array(64,),
    assetName: 'noticeboard.webp',
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
  },);
}

await describe({
  name: 'reading past a reply the completion limit cut',
  children: [
    it({
      name: 'ASKS AGAIN ON ANOTHER PROVIDER AFTER AN EMPTY REPLY THE LENGTH LIMIT CUT, with a prompt of its '
        + 'own so the uniqueness wrapper buys it, and returns the reading that follows',
      fn: async () => {
        const { client, hints, prompts, } = scriptedClient({ steps: [CUT, { text: READING, servedBy: 'openrouter', },], },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(reading.kind,).toBe('read',);
        expect(hints,).toEqual([NO_HINT, 'neuralwatt',],);
        expect(new Set(prompts,).size,).toBe(2,);
        expect(prompts[1],).toContain('Small-text pass',);
      },
    },),

    it({
      name: 'ASKS THE SAME PROVIDER WHEN NO OTHER ONE IS WET, with no hint on the third ask, because the '
        + 'router refuses a hinted ask that has nowhere else to go',
      fn: async () => {
        const { client, hints, } = scriptedClient({ steps: [CUT, 'nowhere-else', { text: READING, },], },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(reading.kind,).toBe('read',);
        expect(hints,).toEqual([NO_HINT, 'neuralwatt', NO_HINT,],);
      },
    },),

    it({
      name: 'LEAVES A SECOND CUT AS A FAILED READING, so a reader that reasons to the limit twice costs '
        + 'two asks and not an unbounded number',
      fn: async () => {
        const { client, hints, } = scriptedClient({ steps: [CUT,], },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(hints.length,).toBe(2,);
        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('empty-reply',);
      },
    },),

    it({
      name: 'LOGS THE RE-ASK AND ITS REASON, naming the picture, the provider that was cut and the finish reason',
      fn: async () => {
        /**
         Every line written.
         */
        const said: string[] = [];
        const { client, } = scriptedClient({ steps: [CUT, { text: READING, },], },);
        await readOnce({ client, l: keepingLogger({ said, },), },);

        expect(said.some(function names(line,): boolean {
          return line.includes('noticeboard.webp',) && line.includes('finish reason length',)
            && line.includes('neuralwatt',) && line.includes('asking again',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'STILL ASKS ONCE FOR AN EMPTY REPLY THAT NO LENGTH LIMIT CUT, since only the cut run has '
        + 'been measured to change on a second ask',
      fn: async () => {
        const { client, hints, } = scriptedClient({
          steps: [{ text: '', finishReason: 'stop', servedBy: 'neuralwatt', },],
        },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(hints.length,).toBe(1,);
        expect(reading.kind,).toBe('unavailable',);
      },
    },),

    it({
      name: 'ASKS WITHOUT A HINT WHEN THE CUT REPLY NAMED NO PROVIDER, as a client that routes nowhere gives',
      fn: async () => {
        const { client, hints, } = scriptedClient({
          steps: [{ text: '', finishReason: 'length', }, { text: READING, },],
        },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(reading.kind,).toBe('read',);
        expect(hints,).toEqual([NO_HINT, NO_HINT,],);
      },
    },),
  ],
},);
