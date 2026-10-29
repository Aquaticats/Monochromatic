/**
 Tests for the provider-neutral ladder that turns one raw reply into the
 outcome a caller acts on.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import { readJsonOutcome, } from '../dist/final/node/index.mjs';
import { SEAT_HYPER_VISION, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Model named on every reading, for the log lines only.
 */
const MODEL_ID = SEAT_HYPER_VISION;

/**
 Verdict shape these readings validate against.
 */
type CatVerdict = { readonly verdict: string; };

/**
 Guards parsed model JSON as a verdict.

 @param value - parsed candidate

 @returns Whether value carries a string verdict

 @example
 ```ts
 isCatVerdict({ verdict: 'nap', },);
 ```
 */
function isCatVerdict(value: unknown,): value is CatVerdict {
  return isJsonRecord(value,) && ((typeof value.verdict) === 'string');
}

/**
 Reads one text into an outcome under the verdict guard.

 @param text - answer channel as a provider delivered it

 @param finishReason - why the model stopped, when it said

 @returns Outcome the ladder decided on

 @example
 ```ts
 const outcome = read({ text: '{"verdict":"nap"}', },);
 ```
 */
function read(
  {
    text,
    finishReason,
  }: {
    readonly text: string;
    readonly finishReason?: string;
  },
) {
  return readJsonOutcome({
    modelId: MODEL_ID,
    reply: {
      text,
      // Conditional spread keeps the field absent rather than undefined.
      ...(finishReason === undefined ? {} : { finishReason, }),
    },
    validate: isCatVerdict,
  },);
}

await describe({
  name: readJsonOutcome.name,
  children: [
    it({
      name: 'ACCEPTS plain content the guard admits',
      fn: async () => {
        /** Outcome of a clean answer. */
        const outcome = read({ text: '{"verdict":"nap"}', },);

        expect(outcome.kind,).toBe('ok',);
        expect(
          outcome.kind === 'ok' ? outcome.value : undefined,
        ).toEqual({ verdict: 'nap', },);
      },
    },),

    ...(['max_tokens', 'length',] as const).map(function truncatingCase(reason,) {
      return it({
        name: `REFUSES parsed content stopped by ${reason}`,
        fn: async () => {
          /** Provider text that forms complete JSON despite the ceiling stop. */
          const text = '{"verdict":"nap"}';
          /** Usage that must survive the refusal-before-parse admission. */
          const usage = { prompt_tokens: 13, completion_tokens: 21, };
          /** Outcome of syntactically complete content cut at the provider limit. */
          const outcome = readJsonOutcome({
            modelId: MODEL_ID,
            reply: { text, finishReason: reason, usage, },
            validate: isCatVerdict,
          },);

          expect(outcome.kind,).toBe('schema-mismatch',);
          expect(outcome.rawText,).toBe(text,);
          expect(outcome.usage,).toEqual(usage,);
          expect(
            outcome.kind === 'schema-mismatch' ? outcome.reason : undefined,
          ).toBe('truncated-completion',);
          expect(
            outcome.kind === 'schema-mismatch' ? outcome.detail : undefined,
          ).toBe(
            `provider reported a truncating completion (model stopped with finish_reason=${reason})`,
          );
        },
      },);
    },),

    it({
      name: 'FORWARDS api refusal ahead of the token-limit marker',
      fn: async () => {
        /** Content ignored because an explicit provider refusal is authoritative. */
        const text = '{"verdict":"nap"}';
        /** Outcome carrying both an explicit refusal and a token-limit marker. */
        const outcome = readJsonOutcome({
          modelId: MODEL_ID,
          reply: { text, refusal: 'provider refused this request', finishReason: 'max_tokens', },
          validate: isCatVerdict,
        },);

        expect(outcome.kind,).toBe('refusal-shaped',);
        expect(outcome.rawText,).toBe(text,);
        expect(
          outcome.kind === 'refusal-shaped' ? outcome.marker : '',
        ).toBe('api-refusal-field',);
      },
    },),

    it({
      name: 'FORWARDS the api refusal field ahead of every content heuristic',
      fn: async () => {
        /** Outcome of a reply the provider itself refused. */
        const outcome = readJsonOutcome({
          modelId: MODEL_ID,
          reply: { text: '', refusal: '不能回答。', },
          validate: isCatVerdict,
        },);

        expect(outcome.kind,).toBe('refusal-shaped',);
        expect(
          outcome.kind === 'refusal-shaped' ? outcome.marker : '',
        ).toBe('api-refusal-field',);
        // A refusal with no content falls back to the refusal text itself, so
        // an audit trail is never empty.
        expect(outcome.rawText,).toBe('不能回答。',);
      },
    },),

    it({
      name: 'RECORDS the api refusal text as the raw text when the content shows a reader nothing, as it does for '
        + 'empty content: spaces and a line break, a zero-width space, a Hangul filler',
      fn: async () => {
        expect([
          ' \n ',
          '\u{200B}',
          '\u{3164}',
        ].map(function outcomeOf(text,) {
          return readJsonOutcome({
            modelId: MODEL_ID,
            reply: { text, refusal: '不能回答。', },
            validate: isCatVerdict,
          },);
        },),).toEqual([
          {
            kind: 'refusal-shaped',
            rawText: '不能回答。',
            marker: 'api-refusal-field',
          },
          {
            kind: 'refusal-shaped',
            rawText: '不能回答。',
            marker: 'api-refusal-field',
          },
          {
            kind: 'refusal-shaped',
            rawText: '不能回答。',
            marker: 'api-refusal-field',
          },
        ],);
      },
    },),

    it({
      name: 'ACCEPTS content that quotes refusal-like phrasing but parses',
      fn: async () => {
        // THE ORDER IS THE POINT: the refusal scan runs only after a parse
        // failure, so a valid answer discussing a refusal stays an answer.
        const outcome = read({ text: '{"verdict":"I cannot assist with that"}', },);

        expect(outcome.kind,).toBe('ok',);
      },
    },),

    it({
      name: 'judges the answer rather than the thinking that preceded it',
      fn: async () => {
        /** Outcome of a reply whose deliberation reads like a refusal. */
        const outcome = read({
          text: '<think>I cannot assist with that, or can I? Yes.</think>{"verdict":"nap"}',
        },);

        expect(outcome.kind,).toBe('ok',);
      },
    },),

    it({
      name: 'names truncation inside thinking as its own mismatch',
      fn: async () => {
        /** Outcome of a reply cut off mid-deliberation. */
        const outcome = read({ text: '<think>还在想猫的事情', },);

        expect(outcome.kind,).toBe('schema-mismatch',);
        // It sends a reader to the token ceiling rather than to the prompt.
        expect(
          outcome.kind === 'schema-mismatch'
            ? outcome.detail.includes('maxTokens',)
            : false,
        ).toBe(true,);
      },
    },),

    it({
      name: 'unwraps a fence hidden behind a channel marker',
      fn: async () => {
        // The stripper cannot see a fence while it is looking at the marker,
        // so a single pass loses this answer to the defect just repaired.
        const outcome = read({ text: 'ep|>```json\n{"verdict":"nap"}\n```', },);

        expect(outcome.kind,).toBe('ok',);
      },
    },),

    it({
      name: 'READS PAST an abandoned opening ahead of the object, the seventeenth class, and still holds the '
        + 'guard to what it read',
      fn: async () => {
        const outcome = read({
          text: '{"verdict": "nap{"verdict": "nap", "why": "sun"}',
          finishReason: 'stop',
        },);
        expect(outcome.kind,).toBe('ok',);
        expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({
          verdict: 'nap',
          why: 'sun',
        },);

        /**
         A false start whose object the guard rejects: read, then refused as the guard says.
         */
        const rejected = read({
          text: '{"{"mood": "nap"}',
          finishReason: 'stop',
        },);
        expect(rejected.kind,).toBe('schema-mismatch',);
        expect(
          rejected.kind === 'schema-mismatch' ? rejected.reason : undefined,
        ).toBe('caller-guard-rejected',);
      },
    },),

    it({
      name: 'READS the value a reply opens with when the model trails more text after a complete object '
        + '(ledger P8), rather than refusing the whole answer as unparseable',
      fn: async () => {
        /**
         Answer whose object a sentence follows.
         */
        const text = '{"verdict":"nap"} and that is final.';
        expect(read({ text, },),).toEqual({
          kind: 'ok',
          value: { verdict: 'nap', },
          rawText: text,
        },);
      },
    },),

    it({
      name: 'names why the model stopped when content will not parse',
      fn: async () => {
        // A non-truncating stop reason stays on the unparseable path; the
        // token-limit spellings are refused before parsing instead.
        const outcome = read({
          text: '{"verdict":"na',
          finishReason: 'stop',
        },);

        expect(outcome.kind,).toBe('schema-mismatch',);
        expect(
          outcome.kind === 'schema-mismatch' ? outcome.reason : undefined,
        ).toBe('unparseable-json',);
        // A cut-off reply and a malformed one arrive identically and need
        // opposite remediation.
        expect(
          outcome.kind === 'schema-mismatch'
            ? outcome.detail.includes('finish_reason=stop',)
            : false,
        ).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES parsed content the guard rejects',
      fn: async () => {
        /** Outcome of well-formed JSON of the wrong shape. */
        const outcome = read({ text: '{"verdict":7}', },);

        expect(outcome.kind,).toBe('schema-mismatch',);
        expect(
          outcome.kind === 'schema-mismatch' ? outcome.reason : undefined,
        ).toBe('caller-guard-rejected',);
      },
    },),

    it({
      name: 'carries usage onto every outcome it can produce',
      fn: async () => {
        /** Usage the provider reported. */
        const usage = { prompt_tokens: 9, completion_tokens: 4, };

        /** Every outcome kind, read from the same reported usage. */
        const outcomes = [
          readJsonOutcome({
            modelId: MODEL_ID,
            reply: { text: '{"verdict":"nap"}', usage, },
            validate: isCatVerdict,
          },),
          readJsonOutcome({
            modelId: MODEL_ID,
            reply: { text: '', refusal: '不能回答。', usage, },
            validate: isCatVerdict,
          },),
          readJsonOutcome({
            modelId: MODEL_ID,
            reply: { text: 'not json at all', usage, },
            validate: isCatVerdict,
          },),
        ];

        // Budget observability must not depend on which way a call went.
        for (const outcome of outcomes) {
          expect(outcome.usage,).toEqual(usage,);
        }
      },
    },),
  ],
},);
