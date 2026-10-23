/**
 Tests for the guarded single exchange every pipeline stage runs through.

 `attemptStageCall` had no test, and it decides the one thing the ensemble
 design depends on: which failures become a missing voice and which propagate.
 The ensemble tolerates missing panelists and critics, so turning a failure
 into an absent voice is correct. Turning a CALLER ABORT into an absent voice
 would not be: the fan-out would carry on answering after the user asked it to
 stop, and quorum would be reached from voices nobody wanted.

 So the cases in this file separate those two, and check that a lost voice is always
 logged rather than silently swallowed.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  EveryProviderDryError,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { attemptStageCall, } from '../dist/final/node/index.mjs';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import { warningRecordingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { quotingFailure, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isPurrReply,
  PURR_FORMAT,
  type PurrReply,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 Logger for the exchanges under test.
 */
const l = tagged({ tag: 'stage-call-test', },);

/**
 Model the exchanges address.
 */
const MODEL_ID = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Client answering with one scripted outcome, or throwing one scripted error.

 @param outcome - outcome to return, when the client answers

 @param thrown - error to throw instead, when the client fails

 @returns Client honoring exactly that script

 @example
 ```ts
 const client = scriptedClient({ outcome: { kind: 'refusal-shaped', rawText: '', detail: '', }, },);
 ```
 */
function scriptedClient(
  {
    outcome,
    thrown,
  }: {
    readonly outcome?: unknown;
    readonly thrown?: Error;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (thrown !== undefined)
        throw thrown;
      if (outcome === undefined) {
        /**
         Payload for the default success path.
         */
        const scripted: unknown = { purr: 'loud', };
        if (!request.validate(scripted,))
          throw new Error('scripted payload failed the guard',);
        return {
          kind: 'ok',
          value: scripted,
          rawText: JSON.stringify(scripted,),
        };
      }
      return outcome as ChatJsonOutcome<ValueT>;
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Runs one exchange against a scripted client.

 @param client - scripted client

 @param signal - caller abort handle

 @returns Voice as data

 @example
 ```ts
 const voice = await callWith({ client, signal: new AbortController().signal, },);
 ```
 */
async function callWith(
  {
    client,
    signal,
    logger = l,
  }: {
    readonly client: ModelClient;
    readonly signal: AbortSignal;
    readonly logger?: typeof l;
  },
) {
  return await attemptStageCall({
    client,
    modelId: MODEL_ID,
    messages: [
      {
        role: 'user',
        content: 'Does the cat purr?',
      },
    ],
    signal,
    exchangeTimeoutMs: 1_000,
    responseFormat: PURR_FORMAT,
    validate: isPurrReply,
    stage: 'purr-check',
    l: logger,
  },);
}

await describe({
  name: attemptStageCall.name,
  children: [
    it({
      name: 'returns the validated reply as a heard voice',
      fn: async () => {
        /**
         Voice from a clean exchange.
         */
        const voice = await callWith({
          client: scriptedClient({},),
          signal: new AbortController().signal,
        },);

        expect(voice.heard,).toBe(true,);
        expect(voice.heard ? voice.value.purr : '',).toBe('loud',);
      },
    },),

    it({
      name: 'turns every NON-OK outcome into an absent voice rather than '
        + 'throwing, because the ensemble is built to reach a verdict without '
        + 'every panelist and one model refusing must not take the stage down',
      fn: async () => {
        /**
         Voices from each scripted non-ok outcome; concurrent because the
         scripted clients share no state and nothing here is a sequence.
         */
        const voices = await Promise.all([
          {
            kind: 'refusal-shaped',
            rawText: 'I cannot help with that.',
            marker: 'i-cannot',
          },
          {
            kind: 'schema-mismatch',
            rawText: '{}',
            detail: 'guard rejected',
          },
        ].map(async function toVoice(outcome,) {
          return await callWith({
            client: scriptedClient({ outcome, },),
            signal: new AbortController().signal,
          },);
        },),);

        for (const voice of voices)
          expect(voice.heard,).toBe(false,);
      },
    },),

    it({
      name: 'NAMES THE SUB-KIND when a schema mismatch loses a voice, because '
        + 'schema-mismatch covers truncated thinking, unparseable content and '
        + 'a rejected guard, which need three different fixes and read '
        + 'identically without it',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({
            outcome: {
              kind: 'schema-mismatch',
              rawText: '|>{"purr":"loud"}',
              detail: 'content is not valid JSON: Unexpected token',
            },
          },),
          signal: new AbortController().signal,
          logger,
        },);

        expect(warnings,).toHaveLength(1,);
        expect(warnings[0],).toContain('content is not valid JSON',);
      },
    },),

    it({
      name: 'carries the OPENING of the model text, which is where a parse '
        + 'failure is diagnosable: the Kimi-K3 outage was a two-character '
        + 'channel marker that explained 507 mismatches in one pass',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({
            outcome: {
              kind: 'schema-mismatch',
              rawText: '|>{"purr":"loud"}',
              detail: 'content is not valid JSON: Unexpected token',
            },
          },),
          signal: new AbortController().signal,
          logger,
        },);

        expect(warnings[0],).toContain('|>',);
      },
    },),

    it({
      name: 'NAMES BOTH FAILURES WHEN THE RE-ASK ON ANOTHER PROVIDER FAILED TOO, the first answer\'s and the re-ask\'s, '
        + 'so a voice lost to a label written in a form no guard accepted is not blamed on the first reply alone '
        + '(the run of TianqiChen66625 logged only the first)',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({
            outcome: {
              kind: 'schema-mismatch',
              rawText: '{"purr":"Occurrence 3"}',
              detail: 'content parsed as JSON but failed the caller schema guard',
              reaskFailure: {
                servedBy: 'hyper',
                kind: 'refusal-shaped',
                rawText: 'I cannot help with that',
                cause: 'refusal-opening',
              },
            },
          },),
          signal: new AbortController().signal,
          logger,
        },);

        expect(warnings,).toHaveLength(1,);
        expect(warnings[0],).toContain('content parsed as JSON but failed the caller schema guard',);
        expect(warnings[0],).toContain('Occurrence 3',);
        expect(warnings[0],).toContain('the re-ask on hyper failed too: refusal-shaped (refusal-opening)',);
        expect(warnings[0],).toContain('I cannot help with that',);
      },
    },),

    it({
      name: 'BOUNDS THE OPENING OF THE RE-ASK\'S REPLY AS IT BOUNDS THE FIRST\'S, so a re-ask reply of three hundred characters '
        + 'shows its first hundred and twenty and a line break in it shows as a space',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({
            outcome: {
              kind: 'schema-mismatch',
              rawText: '{"purr":"Occurrence 3"}',
              detail: 'content parsed as JSON but failed the caller schema guard',
              reaskFailure: {
                servedBy: 'hyper',
                kind: 'schema-mismatch',
                rawText: `a\nb${'q'.repeat(300,)}`,
                cause: 'content is not valid JSON: Unexpected token',
              },
            },
          },),
          signal: new AbortController().signal,
          logger,
        },);

        /**
         Warning the lost voice recorded.
         */
        const warning = warnings[0] ?? '';
        expect(warning,).toContain(`raw="a b${'q'.repeat(117,)}"`,);
        expect(warning.includes('\n',),).toBe(false,);
      },
    },),

    it({
      name: 'TRUNCATES a long reply and flattens its line breaks, so one lost '
        + 'voice cannot bury the rest of a run log',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({
            outcome: {
              kind: 'schema-mismatch',
              rawText: 'purr\n'.repeat(200,),
              detail: 'content is not valid JSON: Unexpected token',
            },
          },),
          signal: new AbortController().signal,
          logger,
        },);

        /**
         Warning the lost voice recorded.
         */
        const warning = warnings[0] ?? '';
        expect(warning.length,).toBeLessThan(300,);
        expect(warning.includes('\n',),).toBe(false,);
      },
    },),

    it({
      name: 'names the refusal MARKER when a refusal-shaped reply loses a '
        + 'voice, since a refusal and a parse failure call for different '
        + 'responses and both read as a bare lost voice otherwise',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({
            outcome: {
              kind: 'refusal-shaped',
              rawText: 'I cannot help with that.',
              marker: 'i-cannot',
            },
          },),
          signal: new AbortController().signal,
          logger,
        },);

        expect(warnings[0],).toContain('i-cannot',);
      },
    },),

    it({
      name: 'turns a thrown transport failure into an absent voice too, since '
        + 'the client has already run its own transport retries by the time '
        + 'this helper sees the error',
      fn: async () => {
        /**
         Voice from a client that threw.
         */
        const voice = await callWith({
          client: scriptedClient({ thrown: new Error('connection reset',), },),
          signal: new AbortController().signal,
        },);

        expect(voice.heard,).toBe(false,);
      },
    },),

    it({
      name: 'LOGS A THROWN TRANSPORT FAILURE BY CLASS WHEN IT IS NOT A MARKED CLASS, because a runtime '
        + 'rejection for an unsendable header quotes the header value',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({ thrown: quotingFailure(), },),
          signal: new AbortController().signal,
          logger,
        },);

        expect(warnings,).toEqual([`purr-check ${MODEL_ID}: refused by TypeError, voice lost`,],);
      },
    },),

    it({
      name: 'LOGS A PROVIDER STATUS FAILURE BY CLASS AND STATUS, then the provider\'s own words labelled as its own '
        + 'and kept to one line',
      fn: async () => {
        const warnings: string[] = [];
        const logger = warningRecordingLogger({ base: l, warnings, },);
        await callWith({
          client: scriptedClient({
            thrown: new ProviderHttpError({
              provider: 'synthetic',
              status: 400,
              bodyText: 'the cat\nrefused "this" passage',
            },),
          },),
          signal: new AbortController().signal,
          logger,
        },);

        /**
         The words as one quoted line, the line break written as an escape.
         */
        const quoted = String.raw`"the cat\nrefused \"this\" passage"`;
        expect(warnings,).toEqual([
          `purr-check ${MODEL_ID}: refused by ProviderHttpError from synthetic with HTTP 400 (the provider said: ${quoted}), voice lost`,
        ],);
      },
    },),

    it({
      name: 'RETHROWS when the caller aborted, which is the case that must not '
        + 'degrade to a lost voice: swallowing it would let a fan-out keep '
        + 'answering after the user asked it to stop, and quorum could then be '
        + 'reached from voices nobody wanted',
      fn: async () => {
        /**
         Abort handle already tripped when the exchange fails.
         */
        const controller = new AbortController();
        controller.abort();

        /**
         Exactly the failure the transport raised, held so the assertion can prove
         THAT object propagated rather than a wrapper quoting its words.
         */
        const abortFailure = new Error('aborted',);

        await expect(
          callWith({
            client: scriptedClient({ thrown: abortFailure, },),
            signal: controller.signal,
          },),
        ).rejects.toBe(abortFailure,);
      },
    },),

    it({
      name: 'decides by the SIGNAL rather than by the error text, so a failure '
        + 'that merely mentions abortion is still a lost voice while any error '
        + 'under a tripped signal propagates',
      fn: async () => {
        /**
         Voice from an abort-sounding failure with no abort in fact.
         */
        const voice = await callWith({
          client: scriptedClient({ thrown: new Error('AbortError: upstream said so',), },),
          signal: new AbortController().signal,
        },);

        expect(voice.heard,).toBe(false,);

        /**
         Abort handle tripped while the failure says nothing about aborting.
         */
        const controller = new AbortController();
        controller.abort();

        /**
         Failure saying nothing about aborting, held by identity because the claim
         under test is that the SIGNAL decides: a wrapper mentioning these words
         would satisfy a wording assertion while breaking the contract.
         */
        const unrelatedFailure = new Error('connection reset',);

        await expect(
          callWith({
            client: scriptedClient({ thrown: unrelatedFailure, },),
            signal: controller.signal,
          },),
        ).rejects.toBe(unrelatedFailure,);
      },
    },),

    it({
      name: 'RETHROWS an every-provider-dry refusal although nobody aborted, since the owner ruled it ends the run: '
        + 'a lost voice here let the run limp on for 77 s before the bench floor stopped it (XingZ6018, 2026-10-22)',
      fn: async () => {
        /**
         The refusal the router raised, held so the assertion proves THAT object propagated.
         */
        const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);

        await expect(
          callWith({
            client: scriptedClient({ thrown: dry, },),
            signal: new AbortController().signal,
          },),
        ).rejects.toBe(dry,);
      },
    },),

    it({
      name: 'CONTAINS A MODEL-DARK REFUSAL as an unreachable voice although nobody aborted, since a model whose own '
        + 'providers are all out of budget is a lost seat and the run goes on, and the log names the dry provider',
      fn: async () => {
        const warnings: string[] = [];
        const voice = await callWith({
          client: scriptedClient({ thrown: modelDarkRefusal({ modelId: MODEL_ID, },), },),
          signal: new AbortController().signal,
          logger: warningRecordingLogger({ base: l, warnings, },),
        },);

        expect(voice,).toMatchObject({ heard: false, answered: false, unreachable: true, },);
        expect(warnings.length,).toBe(1,);
        expect(warnings[0],).toContain('every provider serving this model is out of budget (hyper dry)',);
        expect(warnings[0],).toContain('seat unreachable',);
      },
    },),

    it({
      name: 'passes the caller\'s signal and deadline through to the client '
        + 'untouched, so the abort the stage honors is the one the caller '
        + 'holds rather than a derived handle it cannot trip',
      fn: async () => {
        /**
         Requests the client received, held in a list because a binding
         assigned only inside a callback narrows to never at the read.
         */
        const seen: ChatJsonRequest<PurrReply>[] = [];

        /**
         Caller's own abort handle.
         */
        const controller = new AbortController();

        /**
         Client recording the request it was handed.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            seen.push(request as unknown as ChatJsonRequest<PurrReply>,);

            /**
             Payload satisfying the guard.
             */
            const scripted: unknown = { purr: 'loud', };
            if (!request.validate(scripted,))
              throw new Error('scripted payload failed the guard',);
            return {
              kind: 'ok',
              value: scripted,
              rawText: JSON.stringify(scripted,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };

        await callWith({
          client,
          signal: controller.signal,
        },);

        expect(seen[0]?.signal,).toBe(controller.signal,);
        expect(seen[0]?.exchangeTimeoutMs,).toBe(1_000,);
        expect(seen[0]?.modelId,).toBe(MODEL_ID,);
      },
    },),
  ],
},);
