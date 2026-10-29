/**
 Tests for the seat tally: counting at the client seam and the closing report.

 THE COUNTS ARE THE EVIDENCE ONE CALIBRATION LACKED. Five of ten seats failed every
 call of a four-slice calibration and the command exited 0, because quorum was
 met on the nose by the other five and nothing read the whole run. The tally
 counts every settled call against its seat, and the report says which seats
 were asked and produced nothing usable, in the closing lines of every
 command. The cases in this file pin what "usable" means on each surface, that the
 wrapped client is otherwise untouched, and the exact lines a reader greps.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
  DecisionReply,
  DecisionRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  createSeatTally,
  RUN_SEATS,
  seatReportLines,
  seatTallyClient,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Fixtures

/**
 Single user message reused across cases.
 */
const MESSAGES = [
  {
    role: 'user' as const,
    content: '猫猫的翻译对吗？',
  },
];

/**
 Signal reused across cases; nothing here aborts.
 */
const SIGNAL = new AbortController().signal;

/**
 Verdict shape the JSON cases validate against.
 */
type CatVerdict = { readonly verdict: string; };

/**
 Guards parsed JSON as a verdict.

 @param value - parsed candidate

 @returns Whether it carries a string verdict

 @example
 ```ts
 const ok = isCatVerdict({ verdict: 'purr', },);
 ```
 */
function isCatVerdict(value: unknown,): value is CatVerdict {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('verdict' in value)
    && ((typeof value.verdict) === 'string');
}

/**
 Failure the throwing fixture raises, kept identical so a case can assert the
 wrapper rethrew the very same value.
 */
const FAILURE = new Error('the cat unplugged the router',);

/**
 Builds an inner client that answers every text call with `text`, answers
 every JSON call with the parse of `text` run through the request's guard,
 and throws `FAILURE` on every call instead when `failing` is set.

 @param text - what the fixture says

 @param failing - whether every call throws instead

 @returns Client with the full surface

 @example
 ```ts
 const inner = innerClient({ text: '{"verdict":"purr"}', },);
 ```
 */
function innerClient(
  {
    text,
    failing = false,
  }: {
    readonly text: string;
    readonly failing?: boolean;
  },
): ModelClient {
  return {
    async chatText(): Promise<ChatTextReply> {
      if (failing)
        throw FAILURE;
      return { text, };
    },
    async chatJson<ValueT,>(
      request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      if (failing)
        throw FAILURE;

      /**
       What the fixed text parses to.
       */
      const parsed: unknown = JSON.parse(text,);
      if (request.validate(parsed,))
        return {
          kind: 'ok',
          value: parsed,
          rawText: text,
        };
      return {
        kind: 'refusal-shaped',
        rawText: text,
        marker: 'cannot',
      };
    },
    async quotas(): Promise<never> {
      throw new Error('quotas is never asked here',);
    },
  };
}

/**
 Body of the refusal Bedrock gave the TianqiChen66626 pass on every ask.
 */
const EXPIRED_TOKEN_BODY = '{"error":{"code":"invalid_api_key","message":"The security token included in the request '
  + 'has expired.","param":null,"type":"permission_denied_error"}}';

/**
 Builds an inner client whose every call throws the provider status failure given.

 @param provider - provider that served, and refused, every call

 @param status - HTTP status the provider answered with

 @returns Client with the full surface, failing every call

 @example
 ```ts
 const inner = refusingClient({ provider: 'bedrock', status: 401, },);
 ```
 */
function refusingClient(
  {
    provider,
    status,
  }: {
    readonly provider: ProviderName;
    readonly status: number;
  },
): ModelClient {
  /**
   Failure every call raises.
   */
  const failure = new ProviderHttpError({ provider, status, bodyText: EXPIRED_TOKEN_BODY, },);
  return {
    async chatText(): Promise<ChatTextReply> {
      throw failure;
    },
    async chatJson<ValueT,>(): Promise<ChatJsonOutcome<ValueT>> {
      throw failure;
    },
    async quotas(): Promise<never> {
      throw new Error('quotas is never asked here',);
    },
  };
}

/**
 Words the router gives for a seat whose every provider is out of budget, as
 the zheermao20 log read them for `glm-5.3`.
 */
const ROUTER_REASON = 'every provider serving this model is out of budget, seat unreachable';

/**
 Builds an inner client that behaves as the provider router does for a seat no
 wet provider serves: every call on every surface throws the router's own
 `NoProviderForModelError` before any request is sent.

 @returns Client with the full surface, refusing every call as the router does

 @example
 ```ts
 const inner = routerRefusingClient();
 ```
 */
function routerRefusingClient(): ModelClient {
  /**
   Builds the refusal the router raises for the seat a call names.

   @param modelId - seat the call names

   @returns Router's refusal

   @example
   ```ts
   throw routerRefusal({ modelId: SEAT_HYPER_ONLY, },);
   ```
   */
  function routerRefusal({ modelId, }: { readonly modelId: RosterModelId; },): NoProviderForModelError {
    return new NoProviderForModelError({ modelId, reason: ROUTER_REASON, },);
  }
  return {
    async chatText(request: ForeignBorrowed<ChatTextRequest>,): Promise<ChatTextReply> {
      throw routerRefusal({ modelId: request.modelId, },);
    },
    async chatJson<ValueT,>(
      request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      throw routerRefusal({ modelId: request.modelId, },);
    },
    async decide(request: ForeignBorrowed<DecisionRequest>,): Promise<DecisionReply> {
      throw routerRefusal({ modelId: request.modelId, },);
    },
    async quotas(): Promise<never> {
      throw new Error('quotas is never asked here',);
    },
  };
}

/**
 Runs one call and hands back what it threw, or `undefined` when it resolved.

 @param call - call to run

 @returns Thrown value, or `undefined`

 @example
 ```ts
 const thrown = await thrownBy(function ask() { return client.chatText(request,); },);
 ```
 */
async function thrownBy(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

/**
 Makes one text call to a seat and hands back what it threw, or `undefined`
 when it resolved.

 @param client - client under test

 @param modelId - seat the call names

 @param signal - signal the call is made with

 @returns Thrown value, or `undefined`

 @example
 ```ts
 const thrown = await thrownByText({ client, modelId: SEAT_HYPER_VISION, signal: SIGNAL, },);
 ```
 */
async function thrownByText(
  {
    client,
    modelId,
    signal,
  }: {
    readonly client: ModelClient;
    readonly modelId: RosterModelId;
    readonly signal: AbortSignal;
  },
): Promise<unknown> {
  return await thrownBy(function ask(): Promise<unknown> {
    return client.chatText({ modelId, messages: MESSAGES, signal, },);
  },);
}

/**
 Makes four calls to the seat, alternating the text and JSON surfaces twice,
 and hands back what each one threw, in order.

 @param client - client under test

 @param signal - signal every call is made with

 @returns What each call threw

 @example
 ```ts
 const thrown = await askFourTimes({ client, signal: SIGNAL, },);
 ```
 */
async function askFourTimes(
  {
    client,
    signal,
  }: {
    readonly client: ModelClient;
    readonly signal: AbortSignal;
  },
): Promise<unknown[]> {
  /**
   One text call and one JSON call, what each threw in call order.

   @returns What the two calls threw
   */
  async function askTextAndJson(): Promise<unknown[]> {
    return await Promise.all([
      thrownBy(function askText(): Promise<unknown> {
        return client.chatText({ modelId: SEAT_HYPER_VISION, messages: MESSAGES, signal, },);
      },),
      thrownBy(function askJson(): Promise<unknown> {
        return client.chatJson({
          modelId: SEAT_HYPER_VISION,
          messages: MESSAGES,
          signal,
          validate: isCatVerdict,
        },);
      },),
    ],);
  }
  /**
   What the first pair threw.
   */
  const first = await askTextAndJson();
  /**
   What the second pair threw.
   */
  const second = await askTextAndJson();
  return [...first, ...second,];
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: createSeatTally.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS asked, usable, unusable, and threw per seat, in first-asked order',
          fn: async () => {
            /** Tally under test. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'unusable', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'usable', },);

            expect(tally.counts(),).toStrictEqual([
              {
                modelId: SEAT_HYPER_TEXT_BEDROCK,
                asked: 3,
                usable: 1,
                unusable: 1,
                threw: 1,
                silent: 0,
                abandoned: 0,
                unreachable: 0,
              },
              {
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                asked: 1,
                usable: 1,
                unusable: 0,
                threw: 0,
                silent: 0,
                abandoned: 0,
                unreachable: 0,
              },
            ],);
          },
        },),

        it({
          name: 'NAMES as dark only a seat asked at least once that never produced a usable answer, '
            + 'so a seat that merely lost some rounds is not reported',
          fn: async () => {
            /** Tally with one dark seat, one mixed seat, and one clean seat. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'unusable', },);
            tally.record({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, outcome: 'unusable', },);
            tally.record({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, outcome: 'usable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);

            expect(tally.dark().map(function toId(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_VISION,],);
          },
        },),

        it({
          name: 'FORGETS every seat on reset, so a new command in the same process starts from nothing',
          fn: async () => {
            /** Tally under test. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.reset();

            expect(tally.counts(),).toStrictEqual([],);
            expect(tally.dark(),).toStrictEqual([],);
          },
        },),

        it({
          name: 'NAMES as withheld only a seat asked at least the given number of times that threw on every '
            + 'call, never a seat that answered once or was asked too few times to tell',
          fn: async () => {
            /** Tally with one withheld seat and three seats that are not. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'unusable', },);

            expect(tally.withheld({ minAsks: 4, },).map(function toId(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_VISION,],);
          },
        },),

        it({
          name: 'COUNTS an abandoned call beside the settled ones without adding it to asked, so the withhold test '
            + 'that compares threw with asked never reads a cut-off call as a throw or as an answer',
          fn: async () => {
            /** Tally under test. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'abandoned', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'abandoned', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'abandoned', },);

            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              asked: 2,
              usable: 2,
              unusable: 0,
              threw: 0,
              silent: 0,
              abandoned: 3,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'DOES NOT NAME as dark a seat whose every call was abandoned, since a seat nobody waited for has '
            + 'not shown that it cannot answer, and does not withhold a seat that threw on every settled call '
            + 'while abandoned calls sit beside them',
          fn: async () => {
            /** Tally with one seat only abandoned and one that threw on each settled ask. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'abandoned', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'abandoned', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'threw', },);

            expect(tally.dark().map(function toId(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_VISION,],);
            expect(tally.withheld({ minAsks: 4, },).map(function toId(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_VISION,],);
          },
        },),

        it({
          name: 'COUNTS an unreachable call beside the settled ones without adding it to asked, so a seat the router '
            + 'only refused is neither dark nor withheld however often it was refused',
          fn: async () => {
            /** Tally of a seat the router refused four times and a seat that answered once. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);

            expect(tally.counts(),).toStrictEqual([
              {
                modelId: SEAT_HYPER_ONLY,
                asked: 0,
                usable: 0,
                unusable: 0,
                threw: 0,
                silent: 0,
                abandoned: 0,
                unreachable: 4,
              },
              {
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                asked: 1,
                usable: 1,
                unusable: 0,
                threw: 0,
                silent: 0,
                abandoned: 0,
                unreachable: 0,
              },
            ],);
            expect(tally.withheld({ minAsks: 1, },),).toStrictEqual([],);
            expect(tally.dark(),).toStrictEqual([],);
          },
        },),

        it({
          name: 'SHARES one run-wide tally, which every client the factory builds counts into',
          fn: async () => {
            RUN_SEATS.reset();
            RUN_SEATS.record({ modelId: SEAT_HYPER_VISION, outcome: 'usable', },);

            expect(RUN_SEATS.counts().length,).toBe(1,);

            RUN_SEATS.reset();

            expect(RUN_SEATS.counts().length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: seatTallyClient.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS a text reply as usable and hands it back untouched',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test. */
            const client = seatTallyClient({
              inner: innerClient({ text: '喵。', },),
              tally,
              minAsks: 4,
            },);

            /** Reply as the caller sees it. */
            const reply = await client.chatText({
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              messages: MESSAGES,
              signal: SIGNAL,
            },);

            expect(reply.text,).toBe('喵。',);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              asked: 1,
              usable: 1,
              unusable: 0,
              threw: 0,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'COUNTS an ok JSON outcome as usable and hands it back untouched',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test. */
            const client = seatTallyClient({
              inner: innerClient({ text: '{"verdict":"purr"}', },),
              tally,
              minAsks: 4,
            },);

            /** Outcome as the caller sees it. */
            const outcome = await client.chatJson({
              modelId: SEAT_HYPER_VISION,
              messages: MESSAGES,
              signal: SIGNAL,
              validate: isCatVerdict,
            },);

            expect(outcome.kind,).toBe('ok',);
            expect((outcome.kind === 'ok') ? outcome.value.verdict : '',).toBe('purr',);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_VISION,
              asked: 1,
              usable: 1,
              unusable: 0,
              threw: 0,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'COUNTS a JSON outcome the guard rejected as unusable, since an answer nothing can read '
            + 'is not a voice',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test, answering JSON of the wrong shape. */
            const client = seatTallyClient({
              inner: innerClient({ text: '{"nap":"spot"}', },),
              tally,
              minAsks: 4,
            },);

            /** Outcome as the caller sees it. */
            const outcome = await client.chatJson({
              modelId: SEAT_HYPER_VISION,
              messages: MESSAGES,
              signal: SIGNAL,
              validate: isCatVerdict,
            },);

            expect(outcome.kind,).toBe('refusal-shaped',);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_VISION,
              asked: 1,
              usable: 0,
              unusable: 1,
              threw: 0,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'COUNTS a throw as threw on both surfaces and rethrows the very same value',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test, failing every call. */
            const client = seatTallyClient({
              inner: innerClient({ text: '', failing: true, },),
              tally,
              minAsks: 4,
            },);

            /** What the text surface threw. */
            let fromText: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_TEXT_BEDROCK,
                messages: MESSAGES,
                signal: SIGNAL,
              },);
            }
            catch (error) {
              fromText = error;
            }

            /** What the JSON surface threw. */
            let fromJson: unknown;
            try {
              await client.chatJson({
                modelId: SEAT_HYPER_TEXT_BEDROCK,
                messages: MESSAGES,
                signal: SIGNAL,
                validate: isCatVerdict,
              },);
            }
            catch (error) {
              fromJson = error;
            }

            expect(fromText,).toBe(FAILURE,);
            expect(fromJson,).toBe(FAILURE,);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              asked: 2,
              usable: 0,
              unusable: 0,
              threw: 2,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
            expect(tally.dark().length,).toBe(1,);
          },
        },),

        it({
          name: 'REFUSES a withheld seat on every surface before any call reaches the inner client, naming it '
            + 'unreachable, and leaves its counts as they were',
          fn: async () => {
            /** Tally where one seat threw on each of its four asks. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            /** Client under test, whose inner client would answer every call. */
            const client = seatTallyClient({
              inner: innerClient({ text: '{"verdict":"purr"}', },),
              tally,
              minAsks: 4,
            },);

            /** What the text surface threw. */
            let fromText: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_TEXT_BEDROCK,
                messages: MESSAGES,
                signal: SIGNAL,
              },);
            }
            catch (error) {
              fromText = error;
            }

            /** What the JSON surface threw. */
            let fromJson: unknown;
            try {
              await client.chatJson({
                modelId: SEAT_HYPER_TEXT_BEDROCK,
                messages: MESSAGES,
                signal: SIGNAL,
                validate: isCatVerdict,
              },);
            }
            catch (error) {
              fromJson = error;
            }

            expect(String(fromText,),).toBe(
              'NoProviderForModelError: no provider can take gemma-4-26b-a4b-it: the run withheld it after it '
                + 'threw on all 4 asks and does not ask it again',
            );
            expect(String(fromJson,),).toBe(String(fromText,),);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              asked: 4,
              usable: 0,
              unusable: 0,
              threw: 4,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'COUNTS four calls a caller aborted and that then threw as abandoned and nothing else, so a seat cut off '
            + 'after quorum shows in the census but is neither counted as asked nor withheld',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test, failing every call it is asked. */
            const client = seatTallyClient({
              inner: innerClient({ text: '', failing: true, },),
              tally,
              minAsks: 4,
            },);
            /** Signal the caller aborted before any call was made. */
            const aborted = new AbortController();
            aborted.abort();

            /** What each of the four calls threw, in order. */
            const thrown = await askFourTimes({
              client,
              signal: aborted.signal,
            },);

            expect(thrown,).toStrictEqual([FAILURE, FAILURE, FAILURE, FAILURE,],);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_VISION,
              asked: 0,
              usable: 0,
              unusable: 0,
              threw: 0,
              silent: 0,
              abandoned: 4,
              unreachable: 0,
            },],);
            expect(tally.withheld({ minAsks: 4, },),).toStrictEqual([],);
            expect(tally.dark(),).toStrictEqual([],);
          },
        },),

        it({
          name: 'COUNTS a call the caller aborted as abandoned whichever surface it went to, and a call that '
            + 'settled with an answer after the abort as usable, since the answer arrived',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Signal the caller aborted before any call was made. */
            const aborted = new AbortController();
            aborted.abort();
            /** Client whose every call throws. */
            const failing = seatTallyClient({
              inner: innerClient({ text: '', failing: true, },),
              tally,
              minAsks: 4,
            },);
            /** Client whose every call answers although the caller left. */
            const answering = seatTallyClient({
              inner: innerClient({ text: '喵。', },),
              tally,
              minAsks: 4,
            },);

            await thrownByText({ client: failing, modelId: SEAT_HYPER_VISION, signal: aborted.signal, },);
            await thrownBy(function askJson(): Promise<unknown> {
              return failing.chatJson({
                modelId: SEAT_HYPER_VISION,
                messages: MESSAGES,
                signal: aborted.signal,
                validate: isCatVerdict,
              },);
            },);
            await answering.chatText({ modelId: SEAT_HYPER_VISION, messages: MESSAGES, signal: aborted.signal, },);

            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_VISION,
              asked: 1,
              usable: 1,
              unusable: 0,
              threw: 0,
              silent: 0,
              abandoned: 2,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'WITHHOLDS a seat whose four calls all threw with the caller still waiting on each one',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test, failing every call it is asked. */
            const client = seatTallyClient({
              inner: innerClient({ text: '', failing: true, },),
              tally,
              minAsks: 4,
            },);

            /** What each of the four calls threw, in order. */
            const thrown = await askFourTimes({
              client,
              signal: SIGNAL,
            },);

            expect(thrown,).toStrictEqual([FAILURE, FAILURE, FAILURE, FAILURE,],);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_VISION,
              asked: 4,
              usable: 0,
              unusable: 0,
              threw: 4,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
            expect(tally.withheld({ minAsks: 4, },).map(function toId(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_VISION,],);
          },
        },),

        it({
          name: 'COUNTS A REJECTED CREDENTIAL (HTTP 401) AS ANY OTHER THROW and withholds the seat after its failed asks: '
            + 'the provider is held out beneath this wrapper, in the router, which throws only when no provider '
            + 'that serves the seat is left',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test, whose provider refuses the credential on every call. */
            const client = seatTallyClient({
              inner: refusingClient({ provider: 'bedrock', status: 401, },),
              tally,
              minAsks: 2,
            },);

            await thrownByText({ client, modelId: SEAT_HYPER_TEXT_BEDROCK, signal: SIGNAL, },);
            await thrownByText({ client, modelId: SEAT_HYPER_TEXT_BEDROCK, signal: SIGNAL, },);
            /** What the third call threw, with the seat withheld. */
            const third = await thrownByText({ client, modelId: SEAT_HYPER_TEXT_BEDROCK, signal: SIGNAL, },);

            expect(String(third,),).toBe(
              'NoProviderForModelError: no provider can take gemma-4-26b-a4b-it: the run withheld it after it '
                + 'threw on all 2 asks and does not ask it again',
            );
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              asked: 2,
              usable: 0,
              unusable: 0,
              threw: 2,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'COUNTS THE ROUTER\'S OWN REFUSAL AS UNREACHABLE on every surface, neither asked nor threw, rethrows '
            + 'the very same refusal, and never withholds the seat however often the router refuses it (zheermao20: '
            + '"glm-5.3 asked=4 threw=4" for four refusals and no request)',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client under test, over a router that refuses every call. */
            const client = seatTallyClient({
              inner: routerRefusingClient(),
              tally,
              minAsks: 4,
            },);
            /** Request the typed exchange is made with, cast past what the tally never reads. */
            const decision = {
              modelId: SEAT_HYPER_ONLY,
              signal: SIGNAL,
            } as unknown as DecisionRequest;

            /** What five text calls threw, the fifth made after four refusals. */
            const fromText = [
              await thrownByText({ client, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },),
              await thrownByText({ client, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },),
              await thrownByText({ client, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },),
              await thrownByText({ client, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },),
              await thrownByText({ client, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },),
            ];
            /** What the JSON surface threw. */
            const fromJson = await thrownBy(function askJson(): Promise<unknown> {
              return client.chatJson({
                modelId: SEAT_HYPER_ONLY,
                messages: MESSAGES,
                signal: SIGNAL,
                validate: isCatVerdict,
              },);
            },);
            /** What the typed exchange threw. */
            const fromDecide = await thrownBy(function askDecide(): Promise<unknown> {
              return client.decide?.(decision,) ?? Promise.resolve();
            },);

            expect(
              [...fromText, fromJson, fromDecide,].map(String,),
            ).toStrictEqual(
              Array.from({ length: 7, }, function sameRefusal(): string {
                return `NoProviderForModelError: no provider can take glm-5.3: ${ROUTER_REASON}`;
              },),
            );
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_ONLY,
              asked: 0,
              usable: 0,
              unusable: 0,
              threw: 0,
              silent: 0,
              abandoned: 0,
              unreachable: 7,
            },],);
            expect(tally.withheld({ minAsks: 4, },),).toStrictEqual([],);
            expect(tally.dark(),).toStrictEqual([],);
          },
        },),

        it({
          name: 'STILL COUNTS ANOTHER KIND OF FAILURE AS THREW and withholds the seat after minAsks asks that all '
            + 'threw, router refusals beside them leaving asked and threw as they were, and the tally\'s own '
            + 'withholding refusal counting as nothing at all',
          fn: async () => {
            /** Tally both clients count into. */
            const tally = createSeatTally();
            /** Client over an inner that fails every call in its own way. */
            const failing = seatTallyClient({
              inner: innerClient({ text: '', failing: true, },),
              tally,
              minAsks: 4,
            },);
            /** Client over a router that refuses every call. */
            const refused = seatTallyClient({
              inner: routerRefusingClient(),
              tally,
              minAsks: 4,
            },);

            await thrownByText({ client: refused, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },);
            await thrownByText({ client: failing, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },);
            await thrownByText({ client: failing, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },);
            await thrownByText({ client: refused, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },);
            await thrownByText({ client: failing, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },);
            await thrownByText({ client: failing, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },);
            /** What the next call threw, with the seat withheld. */
            const withheldCall = await thrownByText({ client: refused, modelId: SEAT_HYPER_ONLY, signal: SIGNAL, },);

            expect(String(withheldCall,),).toBe(
              'NoProviderForModelError: no provider can take glm-5.3: the run withheld it after it threw on all '
                + '4 asks and does not ask it again',
            );
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_ONLY,
              asked: 4,
              usable: 0,
              unusable: 0,
              threw: 4,
              silent: 0,
              abandoned: 0,
              unreachable: 2,
            },],);
            expect(tally.withheld({ minAsks: 4, },).map(function toId(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_ONLY,],);
          },
        },),

        it({
          name: 'COUNTS A REFUSAL THE ROUTER GAVE AFTER THE CALLER LEFT AS UNREACHABLE, since the refusal is a fact '
            + 'about the seat that no request was sent for, while any other throw after the abort stays abandoned',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Signal the caller aborted before any call was made. */
            const aborted = new AbortController();
            aborted.abort();
            /** Client over a router that refuses every call. */
            const refused = seatTallyClient({
              inner: routerRefusingClient(),
              tally,
              minAsks: 4,
            },);
            /** Client over an inner that fails every call in its own way. */
            const failing = seatTallyClient({
              inner: innerClient({ text: '', failing: true, },),
              tally,
              minAsks: 4,
            },);

            await thrownByText({ client: refused, modelId: SEAT_HYPER_ONLY, signal: aborted.signal, },);
            await thrownByText({ client: failing, modelId: SEAT_HYPER_ONLY, signal: aborted.signal, },);

            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_ONLY,
              asked: 0,
              usable: 0,
              unusable: 0,
              threw: 0,
              silent: 0,
              abandoned: 1,
              unreachable: 1,
            },],);
          },
        },),

        it({
          name: 'FORWARDS quotas as the very same function, since the meter is not a seat',
          fn: async () => {
            /** Inner client whose meter identity is checked. */
            const inner = innerClient({ text: '喵。', },);
            /** Client under test. */
            const client = seatTallyClient({
              inner,
              tally: createSeatTally(),
              minAsks: 4,
            },);

            expect(client.quotas,).toBe(inner.quotas,);
          },
        },),

        it({
          name: 'COUNTS THE TYPED DECISION EXCHANGE AS A SEAT ASKED, a reply usable and a throw rethrown, and offers it '
            + 'only where the wrapped client does (ledger T8)',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Reply the answering decision seat gives. */
            const reply = { answers: ['windowsill',], } as unknown as DecisionReply;
            /** Decision request naming one seat, cast past what the tally never reads. */
            const request = {
              modelId: SEAT_HYPER_VISION,
              signal: SIGNAL,
            } as unknown as DecisionRequest;
            /** Client under test, whose decision seat answers. */
            const answering = seatTallyClient({
              inner: {
                ...innerClient({ text: '喵。', },),
                decide: async function decide(): Promise<DecisionReply> {
                  return reply;
                },
              },
              tally,
              minAsks: 4,
            },);
            /** Client under test, whose decision seat throws. */
            const failing = seatTallyClient({
              inner: {
                ...innerClient({ text: '喵。', },),
                decide: async function decide(): Promise<never> {
                  throw FAILURE;
                },
              },
              tally,
              minAsks: 4,
            },);

            expect(await answering.decide?.(request,),).toBe(reply,);
            await expect(failing.decide?.(request,),).rejects.toBe(FAILURE,);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_VISION,
              asked: 2,
              usable: 1,
              unusable: 0,
              threw: 1,
              silent: 0,
              abandoned: 0,
              unreachable: 0,
            },],);
            expect(seatTallyClient({
              inner: innerClient({ text: '喵。', },),
              tally,
              minAsks: 4,
            },).decide,).toBeUndefined();
          },
        },),
      ],
    },),

    describe({
      name: seatReportLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS nothing when no seat was asked, so a command that never built a client says nothing extra',
          fn: async () => {
            expect(seatReportLines({ tally: createSeatTally(), },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'PRINTS one SEAT line per seat and no dark line when every seat answered at least once',
          fn: async () => {
            /** Tally in which every seat produced something usable. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'unusable', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'usable', },);

            expect(seatReportLines({ tally, },),).toStrictEqual([
              'SEAT hf:openai/gpt-oss-120b asked=1 usable=1 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0',
              'SEAT minimax-m3 asked=2 usable=1 unusable=1 threw=0 silent=0 abandoned=0 unreachable=0',
            ],);
          },
        },),

        it({
          name: 'PRINTS THE ABANDONED COUNT on the SEAT line, so a seat whose replies were cut at the straggler '
            + 'deadline does not read as one that answered every ask',
          fn: async () => {
            /** Tally of a seat that answered twice and was cut off three times. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'abandoned', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'abandoned', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'abandoned', },);

            expect(seatReportLines({ tally, },),).toStrictEqual([
              'SEAT hf:openai/gpt-oss-120b asked=2 usable=2 unusable=0 threw=0 silent=0 abandoned=3 unreachable=0',
            ],);
          },
        },),

        it({
          name: 'PRINTS THE UNREACHABLE COUNT on the SEAT line, and gives a seat only the router refused its own line '
            + 'with asked=0 that is not named dark, since it was never asked (zheermao20, glm-5.3)',
          fn: async () => {
            /** Tally of a seat the router refused four times and a seat that answered. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);

            expect(seatReportLines({ tally, },),).toStrictEqual([
              'SEAT glm-5.3 asked=0 usable=0 unusable=0 threw=0 silent=0 abandoned=0 unreachable=4',
              'SEAT hf:openai/gpt-oss-120b asked=1 usable=1 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0',
            ],);
          },
        },),

        it({
          name: 'COUNTS ONLY THE SEATS ASKED in the SEATS DARK line, so a seat the router only refused is not one of '
            + 'the seats said to have produced nothing',
          fn: async () => {
            /** Tally with one dark seat, one seat that answered and one the router only refused. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_ONLY, outcome: 'unreachable', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);

            /** Lines as a reader sees them. */
            const lines = seatReportLines({ tally, },);

            expect(lines.length,).toBe(4,);
            expect(lines[3],).toContain(
              'SEATS DARK: 1 of 2 seats asked produced nothing usable this run: gemma-4-26b-a4b-it (asked 1, '
                + 'unusable 0, threw 1). ',
            );
          },
        },),

        it({
          name: 'APPENDS the SEATS DARK line naming only the dark seats with the counts that make them dark',
          fn: async () => {
            /** Tally with two dark seats among three. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'unusable', },);

            /** Lines as a reader sees them. */
            const lines = seatReportLines({ tally, },);

            expect(lines.length,).toBe(4,);
            expect(lines[3],).toBe(
              'SEATS DARK: 2 of 3 seats asked produced nothing usable this run: '
                + 'gemma-4-26b-a4b-it (asked 2, unusable 0, threw 2); minimax-m3 (asked 1, unusable 1, threw 0). '
                + 'A seat that fails every call is a provider that errors on it or a model that answers '
                + 'nothing readable; the run log names which, and writes each failed call\'s error text on a '
                + 'warn line "<stage> <model>: <error>, voice lost". A seat the router refused before any '
                + 'request was sent is on its SEAT line as unreachable=N and is not counted here. Do not read '
                + 'this run as a comparison of the roster.',
            );
          },
        },),
      ],
    },),
  ],
},);
