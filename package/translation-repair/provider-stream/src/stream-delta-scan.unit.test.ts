/**
 Tests for the stream delta scanner.

 The case that matters is the REASONING CHANNEL. A model can degenerate
 entirely inside its thinking, repeating one sentence forever while emitting
 no answer at all, and a scanner that read only `content` would hand the
 detector an empty string. That reads as a short reply rather than a runaway
 one, so the worst case would be the one case nothing caught.

 The rest is wire robustness. This runs inside the drain loop for every chunk
 of every call, so anything the provider sends that it cannot read must leave
 a working stream working.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  scanStreamDeltas,
  watchForDegeneration,
} from '../dist/final/node/index.mjs';
import { routeDeltaToDetector, } from './delta-channel-routing.test-fixture.ts';

/**
 Builds one server-sent event frame carrying text on one channel.

 @param channel - which channel the text arrives on

 @param text - text the frame carries

 @returns Frame, newline-terminated as the wire sends it

 @example
 ```ts
 const raw = frameOf({ channel: 'reasoning', text: 'I will output. ', },);
 ```
 */
function frameOf(
  {
    channel,
    text,
  }: {
    readonly channel: 'content' | 'reasoning';
    readonly text: string;
  },
): string {
  /**
   Delta object, whose field name is what distinguishes the channels.
   */
  const delta = (channel === 'content') ? { content: text, } : { reasoning_content: text, };

  return `data: ${
    JSON.stringify({
      id: 'chatcmpl-tabby',
      object: 'chat.completion.chunk',
      choices: [{
        index: 0,
        delta,
        finish_reason: null,
      },],
    },)
  }\n\n`;
}

/**
 Feeds a whole raw stream through a scanner, splitting it at awkward
 boundaries so no frame arrives whole.

 @param raw - whole stream body

 @returns Every delta it yielded, and how many frames it could not read

 @example
 ```ts
 const { deltas, unreadable, } = scanAll({ raw, },);
 ```
 */
function scanAll({ raw, }: { readonly raw: string; },): {
  readonly deltas: readonly { readonly channel: string; readonly text: string; }[];
  readonly unreadable: number;
} {
  /**
   Scanner under test.
   */
  const scanner = scanStreamDeltas();

  /**
   Chunk width, a prime so boundaries land inside frames rather than between.
   */
  const width = 7;

  /**
   Every delta the scan produced.
   */
  const deltas = Array.from(
    { length: Math.ceil(raw.length / width,), },
    function piece(
      _unused,
      at,
    ): readonly { readonly channel: string; readonly text: string; }[] {
      return scanner.feed({
        chunk: raw.slice(
          at * width,
          (at + 1) * width,
        ),
      },);
    },
  ).flat();

  return {
    deltas,
    unreadable: scanner.unreadableFrames(),
  };
}

/**
 Builds one frame whose delta carries exactly the fields given, so a test can
 spell the thinking channel the way a particular model spells it.

 SEPARATE FROM `frameOf` because that one encodes a choice this helper exists
 to vary. Folding the spelling into it would make every existing case depend
 on a parameter none of them cares about.

 @param delta - delta fields exactly as the wire carries them

 @returns Frame, newline-terminated as the wire sends it

 @example
 ```ts
 const raw = frameSpelled({ delta: { reasoning: 'The cat considers. ', }, },);
 ```
 */
function frameSpelled({ delta, }: { readonly delta: Readonly<Record<string, string>>; },): string {
  return `data: ${
    JSON.stringify({
      id: 'chatcmpl-tabby',
      object: 'chat.completion.chunk',
      choices: [{
        index: 0,
        delta,
        finish_reason: null,
      },],
    },)
  }\n\n`;
}

await describe({
  name: scanStreamDeltas.name,
  children: [
    it({
      name: 'SEPARATES the reasoning channel from the answer channel, which is what lets a '
        + 'degeneration verdict name where it happened',
      fn: async () => {
        const { deltas, } = scanAll({
          raw: frameOf({
            channel: 'reasoning',
            text: 'The cat is grey. ',
          },) + frameOf({
            channel: 'content',
            text: 'A grey cat naps. ',
          },),
        },);

        expect(deltas.length,).toBe(2,);
        expect(deltas[0]?.channel,).toBe('reasoning',);
        expect(deltas[0]?.text,).toBe('The cat is grey. ',);
        expect(deltas[1]?.channel,).toBe('content',);
      },
    },),

    it({
      name: 'NAMES THE UPSTREAM a many-endpoint gateway puts on its frames, settled by the first '
        + 'frame that names one and readable mid-stream, so a cut call is attributed from what had '
        + 'already arrived; a wire whose frames name none reads as empty',
      fn: async () => {
        /**
         Scanner fed a gateway stream, frame by frame.
         */
        const scanner = scanStreamDeltas();
        expect(scanner.servedBy(),).toBe('',);
        scanner.feed({
          chunk: `data: ${JSON.stringify({
            id: 'gen-tabby',
            provider: 'ModelRun',
            choices: [{ index: 0, delta: { reasoning: 'The cat', }, finish_reason: null, },],
          },)}\n\n`,
        },);
        expect(scanner.servedBy(),).toBe('ModelRun',);
        scanner.feed({
          chunk: `data: ${JSON.stringify({
            id: 'gen-tabby',
            provider: 'Parasail',
            choices: [{ index: 0, delta: { content: ' naps.', }, finish_reason: null, },],
          },)}\n\n`,
        },);
        expect(scanner.servedBy(),).toBe('ModelRun',);

        /**
         Scanner fed a single-upstream provider's stream, which names none.
         */
        const unnamed = scanStreamDeltas();
        unnamed.feed({
          chunk: frameOf({
            channel: 'content',
            text: 'A grey cat naps. ',
          },),
        },);
        expect(unnamed.servedBy(),).toBe('',);
      },
    },),

    it({
      name: 'REASSEMBLES frames split across chunks, since a chunk boundary lands wherever the '
        + 'network puts it and routinely falls inside a frame',
      fn: async () => {
        /**
         Twenty frames, delivered in pieces far smaller than one frame.
         */
        const raw = Array.from(
          { length: 20, },
          function one(
            _unused,
            at,
          ): string {
            return frameOf({
              channel: 'content',
              text: `piece ${String(at,)} `,
            },);
          },
        ).join('',);

        const { deltas, } = scanAll({ raw, },);
        expect(deltas.length,).toBe(20,);
        expect(deltas[19]?.text,).toBe('piece 19 ',);
      },
    },),

    it({
      name: 'SKIPS what carries no generated text: keep-alive comments, the done marker, a '
        + 'usage-only closing frame, and a frame carrying only a finish reason',
      fn: async () => {
        const { deltas, unreadable, } = scanAll({
          raw: `: keep-alive\n\n${
            frameOf({
              channel: 'content',
              text: 'A cat. ',
            },)
          }data: {"choices":[{"index":0,"finish_reason":"stop"}]}\n\n`
            + `data: {"object":"chat.completion.chunk","usage":{"completion_tokens":20}}\n\n`
            + `data: [DONE]\n\n`,
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.text,).toBe('A cat. ',);
        expect(unreadable,).toBe(0,);
      },
    },),

    it({
      name: 'READS THE FORM WITH NO SPACE AFTER THE COLON, which is valid and which a sender may '
        + 'legitimately choose: spelling the prefix with the space would skip those lines as though '
        + 'they were comments, and skip them silently, since only payload lines are ever counted as '
        + 'unreadable',
      fn: async () => {
        /**
         One frame, written tight.
         */
        const tight = `data:${
          JSON.stringify({
            choices: [{
              index: 0,
              delta: { content: 'A cat. ', },
              finish_reason: null,
            },],
          },)
        }\n\n`;

        const {
          deltas,
          unreadable,
        } = scanAll({ raw: tight, },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.text,).toBe('A cat. ',);
        expect(unreadable,).toBe(0,);

        // And the done marker in the same tight form, which must not read as a
        // frame nobody could parse.
        expect(scanAll({ raw: 'data:[DONE]\n\n', },).unreadable,).toBe(0,);
      },
    },),

    it({
      name: 'COUNTS a frame it cannot read instead of throwing, because this runs on every chunk '
        + 'of every call and one unreadable frame must not end a stream that is working',
      fn: async () => {
        const { deltas, unreadable, } = scanAll({
          raw: `data: {not json at all\n\n${
            frameOf({
              channel: 'content',
              text: 'still working ',
            },)
          }`,
        },);

        expect(unreadable,).toBe(1,);
        expect(deltas.length,).toBe(1,);
      },
    },),

    it({
      name: 'COUNTS A FRAME THAT PARSES TO SOMETHING OTHER THAN AN OBJECT AS UNREADABLE, as the Anthropic '
        + 'scanner does, and reads neither a delta nor an upstream name from it, while the frame after it '
        + 'still reads',
      fn: async () => {
        /**
         Scanner fed an array frame, then a number, then a working frame, the
         array first so no upstream name is settled before it.
         */
        const scanner = scanStreamDeltas();
        const deltas = scanner.feed({
          chunk: `data: [1,2,3]\n\ndata: 7\n\n${
            frameOf({
              channel: 'content',
              text: 'still purring ',
            },)
          }`,
        },);

        expect(deltas.map(function textOf({ text, },): string {
          return text;
        },),).toEqual(['still purring ',],);
        expect(scanner.servedBy(),).toBe('',);
        expect(scanner.unreadableFrames(),).toBe(2,);
      },
    },),

    it({
      name: 'READS A FRAME WHOSE FIRST CHOICE IS NOT AN OBJECT as carrying no delta, rather than '
        + 'reading past the array or primitive entry a provider could send in its place',
      fn: async () => {
        const scanner = scanStreamDeltas();
        const deltas = scanner.feed({ chunk: 'data: {"choices":[null]}\n\n', },);

        expect(deltas.length,).toBe(0,);
        expect(scanner.unreadableFrames(),).toBe(0,);
      },
    },),

    it({
      name: 'CATCHES A THINKING-TRACE RUNAWAY while leaving the answer channel unjudged, which is '
        + 'the whole reason both channels are scanned: a model repeating one sentence forever '
        + 'inside its reasoning emits no answer, so an answer-only scan would see an empty string '
        + 'and read the worst case as the shortest reply',
      fn: async () => {
        /**
         A model that thinks the same thing forever and never answers.
         */
        const frames = Array.from(
          { length: 9_000, },
          function think(): string {
            return frameOf({
              channel: 'reasoning',
              text: 'I will output. ',
            },);
          },
        ).join('',);

        /**
         The same stream with its closing marker, which must not be read as a
         frame nobody could parse.
         */
        const raw = `${frames}data: [DONE]\n\n`;

        /**
         Scanner and one detector per channel, wired as the drain will wire them.
         */
        const scanner = scanStreamDeltas();
        const thinking = watchForDegeneration();
        const answering = watchForDegeneration();

        scanner.feed({ chunk: raw, },).forEach(function route(delta,): void {
          routeDeltaToDetector({ delta, thinking, answering, },);
        },);

        expect(thinking.verdict().kind,).toBe('degenerate',);
        expect(answering.verdict().kind,).toBe('undecided',);
      },
    },),

    it({
      name: 'READS the thinking channel when a model spells it `reasoning`, which is how '
        + 'GLM-4.7-Flash and Nemotron send every thinking token they produce: reading only '
        + '`reasoning_content` handed the detector an empty string for those two models, and the '
        + 'unreadable tally could not show it because such frames parse perfectly',
      fn: async () => {
        const { deltas, unreadable, } = scanAll({
          raw: frameSpelled({ delta: { reasoning: 'The cat considers the jar. ', }, },),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.channel,).toBe('reasoning',);
        expect(deltas[0]?.text,).toBe('The cat considers the jar. ',);
        expect(unreadable,).toBe(0,);
      },
    },),

    it({
      name: 'COUNTS ONCE when a frame carries both spellings, so a provider that begins sending '
        + 'them as aliases does not double every thinking character and push the detector toward '
        + 'a verdict on volume the model never produced',
      fn: async () => {
        const { deltas, } = scanAll({
          raw: frameSpelled({
            delta: {
              reasoning_content: 'The cat considers. ',
              reasoning: 'The cat considers. ',
            },
          },),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.text,).toBe('The cat considers. ',);
      },
    },),

    it({
      name: 'FALLS THROUGH to `reasoning` when `reasoning_content` is present but empty, since '
        + 'first-present rather than first-non-empty would read such a frame as carrying no '
        + 'thinking at all',
      fn: async () => {
        const { deltas, } = scanAll({
          raw: frameSpelled({
            delta: {
              reasoning_content: '',
              reasoning: 'The cat naps anyway. ',
            },
          },),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.channel,).toBe('reasoning',);
        expect(deltas[0]?.text,).toBe('The cat naps anyway. ',);
      },
    },),

    it({
      name: 'CATCHES A THINKING-TRACE RUNAWAY SPELLED `reasoning` TOO, which is the case that '
        + 'was live and unguarded: the same loop spelled `reasoning_content` was caught, and this '
        + 'one produced no text for the detector to read at all, so the model was neither silent '
        + 'nor repetitive and ran to its deadline',
      fn: async () => {
        /**
         A model that thinks the same thing forever, spelling its channel the
         way the two affected models spell it.
         */
        const frames = Array.from(
          { length: 9_000, },
          function think(): string {
            return frameSpelled({ delta: { reasoning: 'I will output. ', }, },);
          },
        ).join('',);

        /**
         Scanner and one detector per channel, wired as the drain wires them.
         */
        const scanner = scanStreamDeltas();
        const thinking = watchForDegeneration();
        const answering = watchForDegeneration();

        scanner.feed({ chunk: `${frames}data: [DONE]\n\n`, },).forEach(function route(delta,): void {
          routeDeltaToDetector({ delta, thinking, answering, },);
        },);

        expect(thinking.verdict().kind,).toBe('degenerate',);
        expect(answering.verdict().kind,).toBe('undecided',);
      },
    },),
  ],
},);
