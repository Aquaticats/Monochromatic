/**
 Tests for asking again when a reader's reply was cut by the completion limit
 before it wrote a single character of an answer.

 WHY THIS EXISTS: XingZ6016 (2026-10-21) stopped for want of a second
 reading because `glm-5.3-flash` on NeuralWatt reasoned at the provider's
 default effort until the completion cap cut it: `text: ""`, `finishReason:
 "length"`, 18316 completion tokens, 0 content characters. An empty reply was
 never asked again, only a refusal was, so the reader's one reading was lost.

 WHAT THEY PIN SINCE 2026-10-30 (ledger B1074, B1075, B1076): a cut empty reply is a reasoning runaway, and the
 owner's answer "Everywhere (Recommended)" sends its re-ask to another model, not to the same weights on another
 route. The other model is chosen in the order of the cap census's runaway shares ("Lowest runaway share"), among the
 models that read pictures, that their cards do not hold out of the reader role ("Honour holds") and that are not
 readers of the same picture (the bench), and it is asked the picture as the first ask asked it, with no change of
 wording ("Plain question"). Where no model can take it, the reader itself is asked as before: on another provider
 where one is wet, on the same provider otherwise. ONCE, not up to the refusal limit: a second cut stays a failed
 reading.

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
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Cut-empty reading tests

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
 Readers of a picture other than the one that was cut, in the order the census gives them: the models that read
 pictures and that no hold keeps out of the reader role.
 */
const READING_ALTERNATES: readonly RosterModelId[] = [
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
];

/**
 One scripted step: a reply, or the router's refusal for want of a provider.
 */
type Step = ChatTextReply | 'no-provider';

/**
 One ask the scripted client saw.
 */
type SeenAsk = {
  /**
   Model the ask was addressed to.
   */
  readonly modelId: RosterModelId;

  /**
   The `otherThan` hint of the ask, or the no-hint word.
   */
  readonly hint: string;

  /**
   Prompt text of the ask.
   */
  readonly prompt: string;
};

/**
 Client that answers from a script and records what each ask carried.

 @param scripts - steps each model gets, in order, the last one repeating; a model not named reads the picture

 @returns Client and the asks it saw

 @example
 ```ts
 const { client, seen, } = scriptedClient({ scripts: { [READER]: [CUT,], }, },);
 ```
 */
function scriptedClient(
  { scripts, }: { readonly scripts: Readonly<Record<string, readonly Step[]>>; },
): {
  readonly client: ModelClient;
  readonly seen: SeenAsk[];
} {
  /**
   Asks the client saw, in order.
   */
  const seen: SeenAsk[] = [];
  return {
    seen,
    client: {
      chatText: async (request,) => {
        /**
         Request as this package's own contract describes it.
         */
        const asked = request as ChatTextRequest;

        /**
         Asks this model had before this one.
         */
        const before = seen.filter(function sameModel(ask,): boolean {
          return ask.modelId === asked.modelId;
        },).length;
        seen.push({
          modelId: asked.modelId,
          hint: asked.otherThan ?? NO_HINT,
          prompt: asked.messages
            .map(function text(message,): string {
              return messageText({ message, },);
            },)
            .join('\n',),
        },);

        /**
         Step for this ask, holding at the last scripted one.
         */
        const steps = scripts[asked.modelId] ?? [{ text: READING, servedBy: 'hyper', },];
        const step = steps[before] ?? steps.at(-1,) ?? { text: '', };
        await Promise.resolve();
        if (step === 'no-provider') {
          throw new NoProviderForModelError({
            modelId: asked.modelId,
            reason: 'no provider with budget serves it',
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

 @param benchModelIds - readers of the same picture, when the case names them

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
    benchModelIds,
  }: {
    readonly client: ModelClient;
    readonly l: Logger;
    readonly benchModelIds?: readonly RosterModelId[];
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
    ...((benchModelIds === undefined) ? {} : { benchModelIds, }),
  },);
}

/**
 Scripts that give every model other than the reader no provider.

 @returns Script for each model that reads pictures and is not held out of the reader role

 @example
 ```ts
 const scripts = { ...nobodyElse(), [READER]: [CUT, READING_STEP,], };
 ```
 */
function nobodyElse(): Readonly<Record<string, readonly Step[]>> {
  return Object.fromEntries(READING_ALTERNATES.map(function refused(modelId,): readonly [string, readonly Step[]] {
    return [modelId, ['no-provider',],];
  },),);
}

await describe({
  name: 'reading past a reply the completion limit cut',
  concurrency: 1,
  children: [
    it({
      name: 'ASKS ANOTHER MODEL AFTER AN EMPTY REPLY THE LENGTH LIMIT CUT, the one with the lowest share of runaways that '
        + 'reads pictures, with the same prompt and no provider hint, and returns the reading that follows',
      fn: async () => {
        const { client, seen, } = scriptedClient({ scripts: { [READER]: [CUT,], }, },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(reading.kind,).toBe('read',);
        expect(seen.map(function ask(asked,) {
          return [asked.modelId, asked.hint,];
        },),).toEqual([[READER, NO_HINT,], [SEAT_HYPER_TEXT_BEDROCK, NO_HINT,],],);
        // THE PLAIN QUESTION: the picture is asked exactly as the first ask asked it.
        expect(new Set(seen.map(function promptOf(asked,) {
          return asked.prompt;
        },),).size,).toBe(1,);
      },
    },),

    it({
      name: 'MOVES ON TO THE NEXT MODEL when one has no provider with budget, and passes over every model that reads no '
        + 'picture, the culled model and the ones the reader role holds out',
      fn: async () => {
        const { client, seen, } = scriptedClient({
          scripts: { [READER]: [CUT,], [SEAT_HYPER_TEXT_BEDROCK]: ['no-provider',], },
        },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(reading.kind,).toBe('read',);
        expect(seen.map(function modelOf(asked,) {
          return asked.modelId;
        },),).toEqual([READER, SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_VISION_UNSEATED,],);

        const everyone = scriptedClient({ scripts: { [READER]: [CUT,], ...nobodyElse(), }, },);
        await readOnce({ client: everyone.client, l: tagged({ tag: 'cut-empty-test', },), },);
        /**
         Models the second case asked.
         */
        const asked = new Set(everyone.seen.map(function modelOf(ask,) {
          return ask.modelId;
        },),);
        for (const passedOver of [
          SEAT_BEDROCK_ONLY_TEXT,
          SEAT_OPENROUTER_ONLY,
          SEAT_HYPER_ONLY,
          SEAT_HYPER_OPENROUTER_UNMEASURED,
          SEAT_OPENROUTER_ONLY_CHECKER,
        ]) {
          expect(asked.has(passedOver,),).toBe(false,);
        }
      },
    },),

    it({
      name: 'LEAVES OUT THE READERS OF THE SAME PICTURE, so an answer recorded as the cut reader\'s reading is never a '
        + 'second reading by a model that also reads the picture on its own seat',
      fn: async () => {
        const { client, seen, } = scriptedClient({ scripts: { [READER]: [CUT,], }, },);
        await readOnce({
          client,
          l: tagged({ tag: 'cut-empty-test', },),
          benchModelIds: [READER, SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_VISION_UNSEATED,],
        },);

        expect(seen.map(function modelOf(asked,) {
          return asked.modelId;
        },),).toEqual([READER, SEAT_HYPER_VISION,],);
      },
    },),

    it({
      name: 'ASKS THE READER ITSELF ON ANOTHER PROVIDER WHEN NO OTHER MODEL CAN TAKE IT, with the next reading '
        + 'responsibility, saying so in the log, and on the same provider when no other one is wet',
      fn: async () => {
        /**
         Every line written.
         */
        const said: string[] = [];
        const { client, seen, } = scriptedClient({
          scripts: { ...nobodyElse(), [READER]: [CUT, 'no-provider', { text: READING, },], },
        },);
        const reading = await readOnce({ client, l: keepingLogger({ said, },), },);

        expect(reading.kind,).toBe('read',);
        expect(seen.filter(function isReader(asked,): boolean {
          return asked.modelId === READER;
        },).map(function hintOf(asked,) {
          return asked.hint;
        },),).toEqual([NO_HINT, 'neuralwatt', NO_HINT,],);
        // THE FIRST ASK AND THE OTHER MODELS' ASKS SHARE A PROMPT; THE FALL BACK TAKES THE NEXT RESPONSIBILITY.
        expect(seen.at(-1,)?.prompt,).toContain('Small-text pass',);
        expect(seen[0]?.prompt,).not.toContain('Small-text pass',);
        expect(said.some(function names(line,): boolean {
          return line.includes('no other model could take the re-ask',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'FALLS BACK TO THE READER ITSELF WHEN THE BENCH HOLDS EVERY MODEL THAT COULD TAKE IT: nothing else is asked',
      fn: async () => {
        const { client, seen, } = scriptedClient({ scripts: { [READER]: [CUT, { text: READING, },], }, },);
        const reading = await readOnce({
          client,
          l: tagged({ tag: 'cut-empty-test', },),
          benchModelIds: [READER, ...READING_ALTERNATES,],
        },);

        expect(reading.kind,).toBe('read',);
        expect(seen.map(function ask(asked,) {
          return [asked.modelId, asked.hint,];
        },),).toEqual([[READER, NO_HINT,], [READER, 'neuralwatt',],],);
      },
    },),

    it({
      name: 'LEAVES A SECOND CUT AS A FAILED READING, so a reader that reasons to the limit and the model asked in its '
        + 'place cost two asks and not an unbounded number',
      fn: async () => {
        const { client, seen, } = scriptedClient({
          scripts: { [READER]: [CUT,], [SEAT_HYPER_TEXT_BEDROCK]: [CUT,], },
        },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(seen.length,).toBe(2,);
        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('empty-reply',);
      },
    },),

    it({
      name: 'LOGS THE RE-ASK AND ITS REASON, naming the picture, the provider that was cut, the finish reason, the model '
        + 'chosen and each model left out with why',
      fn: async () => {
        /**
         Every line written.
         */
        const said: string[] = [];
        const { client, } = scriptedClient({ scripts: { [READER]: [CUT,], }, },);
        await readOnce({
          client,
          l: keepingLogger({ said, },),
          benchModelIds: [READER, SEAT_HYPER_VISION,],
        },);

        expect(said.some(function names(line,): boolean {
          return line.includes('noticeboard.webp',) && line.includes('finish reason length on neuralwatt',)
            && line.includes(`re-asking ${SEAT_HYPER_TEXT_BEDROCK} instead, plain question`,)
            && line.includes(`${SEAT_HYPER_VISION} (seated on the bench of this question)`,)
            && line.includes(`${SEAT_HYPER_OPENROUTER_UNMEASURED} (held out of the reader role by reader-unmeasured)`,);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'STILL ASKS ONCE FOR AN EMPTY REPLY THAT NO LENGTH LIMIT CUT, since only the cut run has '
        + 'been measured to change on a second ask',
      fn: async () => {
        const { client, seen, } = scriptedClient({
          scripts: { [READER]: [{ text: '', finishReason: 'stop', servedBy: 'neuralwatt', },], },
        },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(seen.length,).toBe(1,);
        expect(reading.kind,).toBe('unavailable',);
      },
    },),

    it({
      name: 'ASKS THE READER WITHOUT A HINT WHEN THE CUT REPLY NAMED NO PROVIDER, as a client that routes nowhere gives, '
        + 'once no other model can take it',
      fn: async () => {
        const { client, seen, } = scriptedClient({
          scripts: { ...nobodyElse(), [READER]: [{ text: '', finishReason: 'length', }, { text: READING, },], },
        },);
        const reading = await readOnce({ client, l: tagged({ tag: 'cut-empty-test', },), },);

        expect(reading.kind,).toBe('read',);
        expect(seen.map(function hintOf(asked,) {
          return asked.hint;
        },).every(function isNoHint(hint,): boolean {
          return hint === NO_HINT;
        },),).toBe(true,);
        expect(seen.at(-1,)?.modelId,).toBe(READER,);
      },
    },),
  ],
},);

//endregion Cut-empty reading tests
