/**
 Tests for the Anthropic delta scanner.

 WHAT THIS FILE IS REALLY CHECKING is that a second wire format produces the
 SAME `ChannelDelta` stream the OpenAI-shaped one does, because that identity
 is the whole reason the stream guards were not written twice. If this scanner
 files the answer under `reasoning`, every volume bound and degeneration
 verdict downstream reads a call that thought forever and answered nothing.

 The case that matters most is `input_json_delta`. Under forced tool use the
 model's entire reply arrives as the tool call's arguments, so a scanner that
 treated those fragments as anything but the answer channel would make every
 schema'd call on this transport look silent.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  scanAnthropicDeltas,
  watchForDegeneration,
} from '../dist/final/node/index.mjs';
import { routeDeltaToDetector, } from './delta-channel-routing.test-fixture.ts';
import {
  anthropicBlockDelta,
  anthropicBlockStart,
  anthropicFrameOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';

/**
 Feeds a whole body to a fresh scanner in one chunk.

 @param raw - whole stream body

 @returns Every delta it yielded, in order

 @example
 ```ts
 const deltas = scanAll({ raw, },);
 ```
 */
function scanAll(
  { raw, }: { readonly raw: string; },
): readonly { readonly channel: string; readonly text: string; }[] {
  return scanAnthropicDeltas().feed({ chunk: raw, },);
}

await describe({
  name: scanAnthropicDeltas.name,
  children: [
    it({
      name: 'FORWARDS a text delta as the answer channel, which is the ordinary case every '
        + 'unforced call takes',
      fn: async () => {
        const deltas = scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'text',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'text_delta',
            field: 'text',
            text: 'Biscuit is smug.',
          },),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.channel,).toBe('content',);
        expect(deltas[0]?.text,).toBe('Biscuit is smug.',);
      },
    },),

    it({
      name: 'FORWARDS a thinking delta as the reasoning channel, which this transport DECLARES '
        + 'rather than spelling two ways: the blindness once measured at 47 percent of calls '
        + 'cannot recur through a typed block',
      fn: async () => {
        const deltas = scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'thinking_delta',
            field: 'thinking',
            text: 'Weighing the mug.',
          },),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.channel,).toBe('reasoning',);
        expect(deltas[0]?.text,).toBe('Weighing the mug.',);
      },
    },),

    it({
      name: 'FORWARDS tool-call arguments as the ANSWER channel, which decides whether forced '
        + 'tool use works at all: under it the whole reply is the tool arguments, so filing them '
        + 'anywhere else leaves every schema\'d call on this transport looking silent',
      fn: async () => {
        const deltas = scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'tool_use',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'input_json_delta',
            field: 'partial_json',
            text: '{"mood":"smug"}',
          },),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.channel,).toBe('content',);
        expect(deltas[0]?.text,).toBe('{"mood":"smug"}',);
      },
    },),

    it({
      name: 'FORWARDS plain text inside a THINKING BLOCK as reasoning, because the block that '
        + 'encloses a delta outranks the delta\'s own type: reading only the type would file a '
        + 'model\'s private deliberation as its answer',
      fn: async () => {
        const deltas = scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'text_delta',
            field: 'text',
            text: 'Still weighing.',
          },),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.channel,).toBe('reasoning',);
      },
    },),

    it({
      name: 'FORWARDS a tool argument fragment as CONTENT even inside a thinking block, since a '
        + 'block declaration cannot demote the answer: this is the frame qwen3.8-max sends, which '
        + 'reopens one index as thinking after opening it as a tool call',
      fn: async () => {
        // THE CAPTURED SHAPE, 2026-08-25. Index 1 opens as `tool_use`
        // and is then reopened as `thinking` with no stop between, after which
        // both delta kinds arrive under it. The scanner keeps the later
        // declaration, so this case fails outright unless the answer channel is
        // exempt from the block-type override.
        const deltas = scanAll({
          raw: anthropicBlockStart({
            index: 1,
            type: 'tool_use',
          },) + anthropicBlockStart({
            index: 1,
            type: 'thinking',
          },) + anthropicBlockDelta({
            index: 1,
            deltaType: 'thinking_delta',
            field: 'thinking',
            text: 'Comparing the three.',
          },) + anthropicBlockDelta({
            index: 1,
            deltaType: 'input_json_delta',
            field: 'partial_json',
            text: '{"best": 2',
          },),
        },);

        expect(deltas.map(function channelOf(delta,): string {
          return delta.channel;
        },),)
          .toEqual([
            'reasoning',
            'content',
          ],);
      },
    },),

    it({
      name: 'SEPARATES two open blocks by the type each declared, since a message interleaves '
        + 'thinking and answer blocks and both carry deltas under their own index',
      fn: async () => {
        const deltas = scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },) + anthropicBlockStart({
            index: 1,
            type: 'text',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'thinking_delta',
            field: 'thinking',
            text: 'Hmm.',
          },) + anthropicBlockDelta({
            index: 1,
            deltaType: 'text_delta',
            field: 'text',
            text: 'Ravenous.',
          },),
        },);

        expect(deltas.length,).toBe(2,);
        expect(deltas[0]?.channel,).toBe('reasoning',);
        expect(deltas[1]?.channel,).toBe('content',);
        expect(deltas[1]?.text,).toBe('Ravenous.',);
      },
    },),

    it({
      name: 'REFUSES to emit anything for lifecycle frames, which outnumber delta frames on a '
        + 'short reply and would each read as an empty answer',
      fn: async () => {
        expect(scanAll({
          raw: anthropicFrameOf({ body: { type: 'message_start', }, },)
            + anthropicFrameOf({ body: { type: 'ping', }, },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_stop',
                index: 0,
              },
            },)
            + anthropicFrameOf({ body: { type: 'message_delta', }, },)
            + anthropicFrameOf({ body: { type: 'message_stop', }, },),
        },).length,).toBe(0,);
      },
    },),

    it({
      name: 'REFUSES to emit anything for a delta type it does not read, so a provider adding a '
        + 'new delta kind cannot inject text into either channel unannounced',
      fn: async () => {
        expect(scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'text',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'citations_delta',
            field: 'citation',
            text: 'a mug',
          },),
        },).length,).toBe(0,);
      },
    },),

    it({
      name: 'REFUSES to emit anything for a delta type it does not read inside a THINKING BLOCK, '
        + 'where the block once claimed it as reasoning and its text was read from the field '
        + 'named by the empty string (ledger B91)',
      fn: async () => {
        expect(scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'signature_delta',
            field: '',
            text: 'a pawprint',
          },),
        },).length,).toBe(0,);
      },
    },),

    it({
      name: 'REASSEMBLES a frame split across two chunks, since a chunk boundary lands wherever '
        + 'the network puts it and routinely falls inside a frame',
      fn: async () => {
        /**
         Scanner fed in halves, as a slow connection delivers a body.
         */
        const scanner = scanAnthropicDeltas();

        /**
         Whole body, to be cut at a point inside its delta frame.
         */
        const raw = anthropicBlockStart({
          index: 0,
          type: 'text',
        },) + anthropicBlockDelta({
          index: 0,
          deltaType: 'text_delta',
          field: 'text',
          text: 'Knocked it off.',
        },);

        /**
         Cut point chosen inside the JSON payload rather than on a boundary.
         */
        const cut = raw.length - 12;

        /**
         Everything both halves yielded, in order.
         */
        const deltas = [
          ...scanner.feed({ chunk: raw.slice(
            0,
            cut,
          ), },),
          ...scanner.feed({ chunk: raw.slice(cut,), },),
        ];

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.text,).toBe('Knocked it off.',);
      },
    },),

    it({
      name: 'READS frames whose lines end with a carriage return, which is what the SSE grammar '
        + 'permits and what a proxy may rewrite a body into',
      fn: async () => {
        const deltas = scanAll({
          raw: (anthropicBlockStart({
            index: 0,
            type: 'text',
          },) + anthropicBlockDelta({
            index: 0,
            deltaType: 'text_delta',
            field: 'text',
            text: 'Pleased.',
          },)).split('\n',).join('\r\n',),
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.text,).toBe('Pleased.',);
      },
    },),

    it({
      name: 'COUNTS a payload it cannot read and KEEPS READING, because this runs on every chunk '
        + 'of every call and one malformed frame must not cost the rest of the stream',
      fn: async () => {
        /**
         Scanner fed one broken frame between two sound ones.
         */
        const scanner = scanAnthropicDeltas();

        /**
         Deltas surviving a payload no parser could read.
         */
        const deltas = scanner.feed({
          chunk: `${anthropicBlockStart({
            index: 0,
            type: 'text',
          },)}data: {not json at all\n\n${anthropicBlockDelta({
            index: 0,
            deltaType: 'text_delta',
            field: 'text',
            text: 'Twelve whiskers.',
          },)}`,
        },);

        expect(deltas.length,).toBe(1,);
        expect(deltas[0]?.text,).toBe('Twelve whiskers.',);
        expect(scanner.unreadableFrames(),).toBe(1,);
      },
    },),

    it({
      name: 'COUNTS a payload that parses to a number as unreadable, as the completion reader '
        + 'refuses one: every Anthropic frame is an object',
      fn: async () => {
        /**
         Scanner fed a JSON number where a frame belongs.
         */
        const scanner = scanAnthropicDeltas();
        scanner.feed({ chunk: 'data: 7\n\n', },);
        expect(scanner.unreadableFrames(),).toBe(1,);
      },
    },),

    it({
      name: 'COUNTS a payload that parses to an array as unreadable, as the completion reader '
        + 'refuses one: every Anthropic frame is an object (ledger B92)',
      fn: async () => {
        /**
         Scanner fed a JSON array where a frame belongs.
         */
        const scanner = scanAnthropicDeltas();
        scanner.feed({ chunk: 'data: [1]\n\n', },);
        expect(scanner.unreadableFrames(),).toBe(1,);
      },
    },),

    it({
      name: 'COUNTS a content_block_delta frame whose delta is an array as unreadable, the same '
        + 'refusal a frame with no delta object gets, rather than reading its type as empty and '
        + 'passing it over uncounted (ledger B92)',
      fn: async () => {
        /**
         Scanner fed a delta frame whose delta field is a JSON array.
         */
        const scanner = scanAnthropicDeltas();

        /**
         Deltas that frame yielded, which must be none.
         */
        const deltas = scanner.feed({
          chunk: anthropicBlockStart({
            index: 0,
            type: 'text',
          },) + anthropicFrameOf({
            body: {
              type: 'content_block_delta',
              index: 0,
              delta: [1,],
            },
          },),
        },);

        expect(deltas.length,).toBe(0,);
        expect(scanner.unreadableFrames(),).toBe(1,);
      },
    },),

    it({
      name: 'READS a delta by its own type when its frame names no block, or a block that never '
        + 'opened: without a declaration there is nothing to outrank the type',
      fn: async () => {
        /**
         Text delta with no index, then one at an index no start frame opened,
         after a thinking block at index 0.
         */
        const deltas = scanAll({
          raw: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_delta',
                delta: {
                  type: 'text_delta',
                  text: 'Biscuit ',
                },
              },
            },)
            + anthropicBlockDelta({
              index: 3,
              deltaType: 'text_delta',
              field: 'text',
              text: 'naps.',
            },),
        },);

        expect(deltas,).toEqual([
          {
            channel: 'content',
            text: 'Biscuit ',
          },
          {
            channel: 'content',
            text: 'naps.',
          },
        ],);
      },
    },),

    it({
      name: 'DECLARES nothing for a block start that names no index, carries no block, or names '
        + 'no type, so it cannot claim a later delta, and COUNTS each as unreadable: a start is '
        + 'read for its declaration alone (ledger B93)',
      fn: async () => {
        /**
         Scanner fed three block starts it cannot read, the first declaring
         thinking at no index, then a text delta at index 0.
         */
        const scanner = scanAnthropicDeltas();

        /**
         Deltas those frames yielded.
         */
        const deltas = scanner.feed({
          chunk: anthropicFrameOf({
            body: {
              type: 'content_block_start',
              content_block: { type: 'thinking', },
            },
          },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_start',
                index: 0,
              },
            },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_start',
                index: 0,
                content_block: { type: 7, },
              },
            },)
            + anthropicBlockDelta({
              index: 0,
              deltaType: 'text_delta',
              field: 'text',
              text: 'Smug.',
            },),
        },);

        expect(deltas,).toEqual([{
          channel: 'content',
          text: 'Smug.',
        },],);
        expect(scanner.unreadableFrames(),).toBe(3,);
      },
    },),

    it({
      name: 'FORGETS an earlier declaration when a later start at the same index cannot be read, '
        + 'with no block or no type: the later start supersedes it, as a readable one does, and '
        + 'the block there is now of no known type (ledger B93)',
      fn: async () => {
        for (const unreadableStart of [
          anthropicFrameOf({
            body: {
              type: 'content_block_start',
              index: 0,
            },
          },),
          anthropicFrameOf({
            body: {
              type: 'content_block_start',
              index: 0,
              content_block: { type: 7, },
            },
          },),
        ]) {
          expect(scanAll({
            raw: anthropicBlockStart({
              index: 0,
              type: 'thinking',
            },)
              + unreadableStart
              + anthropicBlockDelta({
                index: 0,
                deltaType: 'text_delta',
                field: 'text',
                text: 'Smug.',
              },),
          },),).toEqual([{
            channel: 'content',
            text: 'Smug.',
          },],);
        }
      },
    },),

    it({
      name: 'EMITS AND COUNTS nothing for a delta whose text is empty, as the first tool-argument '
        + 'fragment the documentation shows is: an empty fragment is read, not unreadable',
      fn: async () => {
        /**
         Scanner fed a tool block's empty opening fragment.
         */
        const scanner = scanAnthropicDeltas();

        /**
         Deltas that fragment yielded, which must be none.
         */
        const deltas = scanner.feed({
          chunk: anthropicBlockStart({
            index: 0,
            type: 'tool_use',
          },)
            + anthropicBlockDelta({
              index: 0,
              deltaType: 'input_json_delta',
              field: 'partial_json',
              text: '',
            },),
        },);

        expect(deltas.length,).toBe(0,);
        expect(scanner.unreadableFrames(),).toBe(0,);
      },
    },),

    it({
      name: 'COUNTS as unreadable a delta frame with no delta, and a delta of a type it reads whose '
        + 'text field is absent or not a string, as the completion reader refuses the first and '
        + 'the answer fragments among the rest (ledger B93)',
      fn: async () => {
        /**
         Scanner fed four delta frames it is meant to read and cannot.
         */
        const scanner = scanAnthropicDeltas();

        /**
         Deltas those frames yielded, which must be none.
         */
        const deltas = scanner.feed({
          chunk: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },)
            + anthropicBlockStart({
              index: 1,
              type: 'text',
            },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_delta',
                index: 1,
              },
            },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_delta',
                index: 1,
                delta: {
                  type: 'text_delta',
                  text: 7,
                },
              },
            },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_delta',
                index: 1,
                delta: { type: 'text_delta', },
              },
            },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_delta',
                index: 0,
                delta: { type: 'thinking_delta', },
              },
            },),
        },);

        expect(deltas.length,).toBe(0,);
        expect(scanner.unreadableFrames(),).toBe(4,);
      },
    },),

    it({
      name: 'COUNTS NOTHING for a delta of a type it does not read, whatever that delta holds: '
        + 'the documentation says new types may be added and are to be passed over',
      fn: async () => {
        /**
         Scanner fed a signature delta and a type no documentation names.
         */
        const scanner = scanAnthropicDeltas();
        scanner.feed({
          chunk: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_delta',
                index: 0,
                delta: {
                  type: 'signature_delta',
                  signature: 7,
                },
              },
            },)
            + anthropicFrameOf({
              body: {
                type: 'content_block_delta',
                index: 0,
                delta: { type: 'purr_delta', },
              },
            },),
        },);
        expect(scanner.unreadableFrames(),).toBe(0,);
      },
    },),

    it({
      name: 'REFUSES to count an empty keep-alive payload as unreadable, since inflating that '
        + 'tally would hide the changed wire format it exists to make visible',
      fn: async () => {
        /**
         Scanner fed the bare `data:` lines servers send to hold a connection.
         */
        const scanner = scanAnthropicDeltas();
        scanner.feed({ chunk: 'data:\n\ndata: \n\n', },);
        expect(scanner.unreadableFrames(),).toBe(0,);
      },
    },),

    it({
      name: 'FORWARDS a thinking runaway into the EXISTING degeneration guard while leaving the '
        + 'answer channel unjudged, which is the claim the whole normalization rests on: a guard '
        + 'built for the other wire format reaches its verdict on this one unchanged',
      fn: async () => {
        /**
         A model that thinks the same thing forever and never answers.
         */
        const frames = anthropicBlockDelta({
          index: 0,
          deltaType: 'thinking_delta',
          field: 'thinking',
          text: 'I will output. ',
        },)
          .repeat(9_000,);

        /**
         Scanner and one detector per channel, wired as the drain will wire them.
         */
        const scanner = scanAnthropicDeltas();
        const thinking = watchForDegeneration();
        const answering = watchForDegeneration();

        scanner.feed({
          chunk: anthropicBlockStart({
            index: 0,
            type: 'thinking',
          },) + frames,
        },).forEach(function route(delta,): void {
          routeDeltaToDetector({ delta, thinking, answering, },);
        },);

        expect(thinking.verdict().kind,).toBe('degenerate',);
        expect(answering.verdict().kind,).toBe('undecided',);
      },
    },),

    it({
      name: 'COUNTS A KEEP-ALIVE PING AS NOTHING, not as an unreadable frame, which a live capture '
        + 'on 2026-08-24 showed this provider sends: twenty of them on an idle stream would '
        + 'otherwise read as twenty frames the scanner could not follow',
      fn: async () => {
        /**
         Scanner under test, fed a stream that is mostly keep-alives.
         */
        const scanner = scanAnthropicDeltas();

        scanner.feed({
          chunk: [
            'data: {"type":"message_start","message":{"usage":{"input_tokens":0,"output_tokens":0}}}',
            ...Array.from(
              { length: 20, },
              function ping(): string {
                return 'data: {"type":"ping"}';
              },
            ),
            'data: {"type":"message_stop"}',
            '',
          ].join('\n\n',),
        },);

        expect(scanner.unreadableFrames(),).toBe(0,);
      },
    },),

    it({
      name: 'COUNTS THE [DONE] SENTINEL AS NOTHING, not as an unreadable frame, which OpenRouter\'s '
        + 'Messages endpoint appends after message_stop under an `event: data` line (captured '
        + '2026-09-03): one such frame on every call would read as a wire format nobody follows',
      fn: async () => {
        /**
         Scanner under test, fed the closing frames as that gateway sends them.
         */
        const scanner = scanAnthropicDeltas();

        /**
         Deltas the closing frames carried, which must be none.
         */
        const deltas = scanner.feed({
          chunk: [
            'event: message_delta',
            'data: {"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},'
              + '"usage":{"input_tokens":574,"output_tokens":306}}',
            'event: message_stop',
            'data: {"type":"message_stop"}',
            'event: data',
            'data: [DONE]',
            '',
          ].join('\n\n',),
        },);

        expect(deltas.length,).toBe(0,);
        expect(scanner.unreadableFrames(),).toBe(0,);
      },
    },),

    it({
      name: 'NAMES NO UPSTREAM, even after a message_start that names one, since the Hyper stream this reads '
        + 'names none and the endpoint that does is used for measurement only (ledger T8)',
      fn: async () => {
        /**
         Scanner under test, fed an opening frame that names an upstream.
         */
        const scanner = scanAnthropicDeltas();

        scanner.feed({
          chunk: 'data: {"type":"message_start","message":{"provider":"Whiskers","usage":{"input_tokens":0}}}\n\n',
        },);

        expect(scanner.servedBy(),).toBe('',);
      },
    },),

    it({
      name: 'READS NO TEXT from a delta whose type is a name every object inherits, where plain-object '
        + 'tables of channels and fields answered with the inherited object or function and read text '
        + 'from the field its string form names (ledger B77)',
      fn: async () => {
        // Each field is the string form of what the type's name inherits:
        // Object.prototype for __proto__, the Object function for constructor.
        expect([
          {
            deltaType: '__proto__',
            field: '[object Object]',
          },
          {
            deltaType: 'constructor',
            field: 'function Object() { [native code] }',
          },
        ].flatMap(function deltasOf({
          deltaType,
          field,
        },) {
          return scanAll({
            raw: anthropicBlockStart({
              index: 0,
              type: 'text',
            },) + anthropicBlockDelta({
              index: 0,
              deltaType,
              field,
              text: 'Mittens naps.',
            },),
          },);
        },),).toStrictEqual([],);
      },
    },),
  ],
},);
