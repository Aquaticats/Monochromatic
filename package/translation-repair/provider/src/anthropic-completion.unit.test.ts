/**
 Tests for the Anthropic completion extractor.

 THE CASE THAT DECIDES EVERYTHING is the tool call. On this provider a schema'd
 answer arrives entirely as `input_json_delta` fragments of a tool's arguments
 and the model emits no prose at all, so an extractor that read only text
 would return the empty string for every successful call and every stage would
 record a lost voice.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MalformedCompletionError, } from '@monochromatic-dev/translation-repair-provider-stream';
import { extractAnthropicCompletion, } from '../dist/final/node/index.mjs';

/**
 Builds one event line as the wire sends it.

 @param body - frame payload, which carries its own `type`

 @returns Frame, newline-terminated

 @example
 ```ts
 const raw = frameOf({ body: { type: 'message_stop', }, },);
 ```
 */
function frameOf(
  { body, }: { readonly body: Readonly<Record<string, unknown>>; },
): string {
  return `data: ${JSON.stringify(body,)}\n\n`;
}

/**
 Opening frame, whose usage sits nested inside `message`.

 @param inputTokens - prompt tokens the provider reports

 @returns Frame ready to feed the extractor

 @example
 ```ts
 const raw = startOf({ inputTokens: 41, },);
 ```
 */
function startOf(
  { inputTokens, }: { readonly inputTokens: number; },
): string {
  return frameOf({
    body: {
      type: 'message_start',
      message: {
        id: 'msg_tabby',
        role: 'assistant',
        stop_reason: null,
        usage: {
          input_tokens: inputTokens,
          output_tokens: 0,
        },
      },
    },
  },);
}

/**
 Closing pair: the stop reason and usage, then the terminator.

 @param stopReason - why the model stopped

 @param outputTokens - completion tokens the provider reports

 @returns Frames ready to feed the extractor

 @example
 ```ts
 const raw = endOf({ stopReason: 'tool_use', outputTokens: 12, },);
 ```
 */
function endOf(
  {
    stopReason,
    outputTokens,
  }: {
    readonly stopReason: string;
    readonly outputTokens: number;
  },
): string {
  return frameOf({
    body: {
      type: 'message_delta',
      delta: { stop_reason: stopReason, },
      usage: { output_tokens: outputTokens, },
    },
  },) + frameOf({ body: { type: 'message_stop', }, },);
}

/**
 One delta frame of a given kind.

 @param deltaType - kind of delta

 @param field - field the text rides in

 @param text - text the frame carries

 @returns Frame ready to feed the extractor

 @example
 ```ts
 const raw = deltaOf({ deltaType: 'text_delta', field: 'text', text: 'Biscuit', },);
 ```
 */
function deltaOf(
  {
    deltaType,
    field,
    text,
  }: {
    readonly deltaType: string;
    readonly field: string;
    readonly text: string;
  },
): string {
  return frameOf({
    body: {
      type: 'content_block_delta',
      index: 0,
      delta: {
        type: deltaType,
        [field]: text,
      },
    },
  },);
}

/**
 Attempts to extract a completion from a data frame that is not JSON, since a
 frame nobody can read means the answer assembled from the rest is missing an
 unknown piece.

 @throws MalformedCompletionError naming the Anthropic Messages contract, so
 the failure points at the right provider rather than the OpenAI-compatible one

 @example
 ```ts
 expect(refuseUnreadableFrame,).toThrow(MalformedCompletionError,);
 ```
 */
function refuseUnreadableFrame(): void {
  extractAnthropicCompletion({
    provider: 'hyper',
    bodyText: `${startOf({ inputTokens: 8, },)}data: {not json at all\n\n${endOf({
      stopReason: 'end_turn',
      outputTokens: 1,
    },)}`,
  },);
}

await describe({
  name: extractAnthropicCompletion.name,
  children: [
    it({
      name: 'RETURNS the tool arguments as the answer, which is the whole reply on a schema\'d '
        + 'call: a model asked for a tool emits no prose, so reading only text would report every '
        + 'successful call as a lost voice',
      fn: async () => {
        /**
         Tool arguments arriving in fragments, as the wire splits them.
         */
        const extracted = extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: startOf({ inputTokens: 41, },)
            + deltaOf({
              deltaType: 'input_json_delta',
              field: 'partial_json',
              text: '{"mood":',
            },)
            + deltaOf({
              deltaType: 'input_json_delta',
              field: 'partial_json',
              text: '"smug"}',
            },)
            + endOf({
              stopReason: 'tool_use',
              outputTokens: 12,
            },),
        },);

        expect(extracted.text,).toBe('{"mood":"smug"}',);
        expect(extracted.finishReason,).toBe('tool_use',);
        expect(extracted.usage?.prompt_tokens,).toBe(41,);
        expect(extracted.usage?.completion_tokens,).toBe(12,);
        expect(extracted.usage?.total_tokens,).toBe(53,);
      },
    },),

    it({
      name: 'RETURNS prose when the model answered in text rather than a tool, since both shapes '
        + 'are used by this pipeline',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: startOf({ inputTokens: 8, },)
            + deltaOf({
              deltaType: 'text_delta',
              field: 'text',
              text: 'Biscuit is smug.',
            },)
            + endOf({
              stopReason: 'end_turn',
              outputTokens: 5,
            },),
        },).text,).toBe('Biscuit is smug.',);
      },
    },),

    it({
      name: 'KEEPS the tool arguments as the whole answer when the model also wrote prose before '
        + 'calling the tool, since gluing the two lost the voice on a schema\'d call',
      fn: async () => {
        /**
         Stream with a text block before the tool block, which `tool_choice:
         auto` permits.
         */
        const bodyText = startOf({ inputTokens: 12, },)
          + deltaOf({
            deltaType: 'text_delta',
            field: 'text',
            text: 'Here is my verdict: ',
          },)
          + deltaOf({
            deltaType: 'input_json_delta',
            field: 'partial_json',
            text: '{"verdict":',
          },)
          + deltaOf({
            deltaType: 'input_json_delta',
            field: 'partial_json',
            text: '"pass"}',
          },)
          + endOf({
            stopReason: 'tool_use',
            outputTokens: 9,
          },);
        expect(extractAnthropicCompletion({ provider: 'hyper', bodyText, },).text,).toBe('{"verdict":"pass"}',);
      },
    },),
    it({
      name: 'RETURNS the prose when the tool arguments show a reader nothing, as it does when no tool was called: '
        + 'a tool call carrying only spaces, a zero-width space and a Hangul filler carried no answer',
      fn: async () => {
        /**
         Stream answering in prose beside a tool call whose arguments are padding.
         */
        const bodyText = startOf({ inputTokens: 12, },)
          + deltaOf({
            deltaType: 'text_delta',
            field: 'text',
            text: '{"verdict":"pass"}',
          },)
          + deltaOf({
            deltaType: 'input_json_delta',
            field: 'partial_json',
            text: ' \u{200B}',
          },)
          + deltaOf({
            deltaType: 'input_json_delta',
            field: 'partial_json',
            text: '\u{3164}\n',
          },)
          + endOf({
            stopReason: 'tool_use',
            outputTokens: 9,
          },);
        expect(extractAnthropicCompletion({ provider: 'hyper', bodyText, },),).toEqual({
          text: '{"verdict":"pass"}',
          finishReason: 'tool_use',
          usage: {
            prompt_tokens: 12,
            completion_tokens: 9,
            total_tokens: 21,
          },
        },);
      },
    },),
    it({
      name: 'DISCARDS the thinking channel, which is the model\'s private working and would '
        + 'corrupt the answer a validator reads if it were concatenated in',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: startOf({ inputTokens: 8, },)
            + deltaOf({
              deltaType: 'thinking_delta',
              field: 'thinking',
              text: 'Weighing the mug carefully.',
            },)
            + deltaOf({
              deltaType: 'input_json_delta',
              field: 'partial_json',
              text: '{"mood":"aloof"}',
            },)
            + endOf({
              stopReason: 'tool_use',
              outputTokens: 9,
            },),
        },).text,).toBe('{"mood":"aloof"}',);
      },
    },),

    it({
      name: 'REFUSES a stream that never sent message_stop, because returning its prefix would '
        + 'hand a validator half a JSON object and get a cut connection reported as a schema '
        + 'mismatch, sending a reader to the prompt instead of to the transport',
      fn: async () => {
        expect(function truncated() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: startOf({ inputTokens: 8, },)
              + deltaOf({
                deltaType: 'input_json_delta',
                field: 'partial_json',
                text: '{"mood":"rav',
              },),
          },);
        },).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'REFUSES a stream whose only "message_stop" is a value inside another frame, since the '
        + 'terminator is a frame of that type: a tool named that way, on a stream cut before it '
        + 'ended, read as whole and handed a validator half a JSON object (ledger B87)',
      fn: async () => {
        expect(function namedOnly() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: startOf({ inputTokens: 8, },)
              + frameOf({
                body: {
                  type: 'content_block_start',
                  index: 0,
                  content_block: {
                    type: 'tool_use',
                    name: 'message_stop',
                    input: {},
                  },
                },
              },)
              + deltaOf({
                deltaType: 'input_json_delta',
                field: 'partial_json',
                text: '{"mood":"rav',
              },),
          },);
        },).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'NAMES THE ERROR an error event reported, rather than calling the stream cut off: the '
        + 'provider said why it stopped, and a refusal naming a dropped connection sends a reader '
        + 'to the network instead (ledger B87)',
      fn: async () => {
        /**
         Stream the provider ended with an error event, as the streaming
         documentation shows one.
         */
        const bodyText = startOf({ inputTokens: 8, },)
          + deltaOf({
            deltaType: 'text_delta',
            field: 'text',
            text: 'Biscuit',
          },)
          + frameOf({
            body: {
              type: 'error',
              error: {
                type: 'overloaded_error',
                message: 'Overloaded',
              },
            },
          },);
        expect(function errored() {
          extractAnthropicCompletion({ provider: 'hyper', bodyText, },);
        },).toThrow(MalformedCompletionError,);
        expect(function errored() {
          extractAnthropicCompletion({ provider: 'hyper', bodyText, },);
        },).toThrow('overloaded_error',);
      },
    },),

    it({
      name: 'REFUSES an answer past an error event even when message_stop follows it, since what '
        + 'the stream carried before the provider reported a failure is not an answer it vouched '
        + 'for (ledger B87)',
      fn: async () => {
        expect(function erroredThenStopped() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: startOf({ inputTokens: 8, },)
              + deltaOf({
                deltaType: 'text_delta',
                field: 'text',
                text: 'Biscuit',
              },)
              + frameOf({
                body: {
                  type: 'error',
                  error: {
                    type: 'api_error',
                    message: 'Internal',
                  },
                },
              },)
              + endOf({
                stopReason: 'end_turn',
                outputTokens: 1,
              },),
          },);
        },).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'REFUSES a content_block_delta frame that carries no delta, since the documentation '
        + 'gives every such frame one, and folding past it returned the answer with a piece '
        + 'missing and nothing said (ledger B89)',
      fn: async () => {
        expect(function deltaless() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: startOf({ inputTokens: 8, },)
              + deltaOf({
                deltaType: 'text_delta',
                field: 'text',
                text: 'Biscuit ',
              },)
              + frameOf({
                body: {
                  type: 'content_block_delta',
                  index: 0,
                },
              },)
              + deltaOf({
                deltaType: 'text_delta',
                field: 'text',
                text: 'is smug.',
              },)
              + endOf({
                stopReason: 'end_turn',
                outputTokens: 3,
              },),
          },);
        },).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'REFUSES a content_block_delta frame whose delta is an array, the same refusal a frame '
        + 'with no delta object gets, rather than reading its type as empty and silently folding '
        + 'nothing for it (ledger B92)',
      fn: async () => {
        expect(function arrayDelta() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: startOf({ inputTokens: 8, },)
              + deltaOf({
                deltaType: 'text_delta',
                field: 'text',
                text: 'Biscuit ',
              },)
              + frameOf({
                body: {
                  type: 'content_block_delta',
                  index: 0,
                  delta: [1,],
                },
              },)
              + endOf({
                stopReason: 'end_turn',
                outputTokens: 3,
              },),
          },);
        },).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'REFUSES a text_delta whose text is not a string, since reading it as empty dropped '
        + 'words from a prose answer that then shipped as whole (ledger B89)',
      fn: async () => {
        expect(function textless() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: startOf({ inputTokens: 8, },)
              + frameOf({
                body: {
                  type: 'content_block_delta',
                  index: 0,
                  delta: {
                    type: 'text_delta',
                    text: 7,
                  },
                },
              },)
              + endOf({
                stopReason: 'end_turn',
                outputTokens: 1,
              },),
          },);
        },).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'REFUSES an input_json_delta with no partial_json, since reading it as empty handed '
        + 'the validator tool arguments with a hole in them (ledger B89)',
      fn: async () => {
        expect(function argumentless() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: startOf({ inputTokens: 8, },)
              + deltaOf({
                deltaType: 'input_json_delta',
                field: 'partial_json',
                text: '{"mood":',
              },)
              + frameOf({
                body: {
                  type: 'content_block_delta',
                  index: 0,
                  delta: { type: 'input_json_delta', },
                },
              },)
              + endOf({
                stopReason: 'tool_use',
                outputTokens: 2,
              },),
          },);
        },).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'PASSES OVER a delta naming no type, or a type this reader does not fold, since either '
        + 'may be one the provider added later and neither carries the answer',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: startOf({ inputTokens: 8, },)
            + deltaOf({
              deltaType: 'text_delta',
              field: 'text',
              text: 'Biscuit is smug.',
            },)
            + frameOf({
              body: {
                type: 'content_block_delta',
                index: 0,
                delta: { text: 'stray', },
              },
            },)
            + deltaOf({
              deltaType: 'citations_delta',
              field: 'citation',
              text: 'a whisker',
            },)
            + endOf({
              stopReason: 'end_turn',
              outputTokens: 4,
            },),
        },).text,).toBe('Biscuit is smug.',);
      },
    },),

    it({
      name: 'REFUSES a payload that is not JSON rather than skipping it, since a frame nobody can '
        + 'read means the answer assembled from the rest is missing an unknown piece',
      fn: async () => {
        expect(refuseUnreadableFrame,).toThrow(MalformedCompletionError,);
      },
    },),

    it({
      name: 'NAMES THE ANTHROPIC MESSAGES CONTRACT when the fold refuses a frame, not the '
        + 'OpenAI-compatible one (ledger B90)',
      fn: async () => {
        expect(refuseUnreadableFrame,).toThrow('Anthropic Messages contract',);
      },
    },),

    it({
      name: 'REPORTS no usage at all rather than zeros when the provider sent none, so a reader '
        + 'can tell a call that cost nothing from one whose cost went unreported',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: deltaOf({
            deltaType: 'text_delta',
            field: 'text',
            text: 'Twelve whiskers.',
          },) + frameOf({ body: { type: 'message_stop', }, },),
        },).usage,).toBe(undefined,);
      },
    },),

    it({
      name: 'COUNTS CACHED PROMPT TOKENS as prompt tokens, since Anthropic\'s input_tokens counts only '
        + 'what follows the last cache breakpoint: the prompt the model read is that plus the tokens '
        + 'read from and written to the cache, and dropping them priced a cached call as a short one '
        + '(ledger B88)',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: frameOf({
            body: {
              type: 'message_start',
              message: {
                id: 'msg_tabby',
                role: 'assistant',
                stop_reason: null,
                usage: {
                  input_tokens: 40,
                  cache_creation_input_tokens: 20,
                  cache_read_input_tokens: 300,
                  output_tokens: 1,
                },
              },
            },
          },)
            + deltaOf({
              deltaType: 'text_delta',
              field: 'text',
              text: 'Biscuit is smug.',
            },)
            + endOf({
              stopReason: 'end_turn',
              outputTokens: 12,
            },),
        },).usage,).toEqual({
          prompt_tokens: 360,
          completion_tokens: 12,
          total_tokens: 372,
        },);
      },
    },),

    it({
      name: 'REPORTS no finish reason rather than guessing one, since a completion that stopped '
        + 'early is indistinguishable from a malformed one without it',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: deltaOf({
            deltaType: 'text_delta',
            field: 'text',
            text: 'Pleased.',
          },) + frameOf({ body: { type: 'message_stop', }, },),
        },).finishReason,).toBe(undefined,);
      },
    },),

    it({
      name: 'FOLDS NOTHING from a message_start with no message or a message with no usage, or '
        + 'from a message_delta with no delta, while keeping the counts and answer the rest carry: '
        + 'the stop reason is reported by its absence',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: frameOf({ body: { type: 'message_start', }, },)
            + frameOf({
              body: {
                type: 'message_start',
                message: { id: 'msg_tabby', },
              },
            },)
            + deltaOf({
              deltaType: 'text_delta',
              field: 'text',
              text: 'Pleased.',
            },)
            + frameOf({
              body: {
                type: 'message_delta',
                usage: {
                  input_tokens: 8,
                  output_tokens: 2,
                },
              },
            },)
            + frameOf({ body: { type: 'message_stop', }, },),
        },),).toEqual({
          text: 'Pleased.',
          usage: {
            prompt_tokens: 8,
            completion_tokens: 2,
            total_tokens: 10,
          },
        },);
      },
    },),

    it({
      name: 'REPORTS no usage rather than a zero when the stream reported its completion count and '
        + 'never its prompt count, or the reverse: a zero would read as a measured count, and the '
        + 'OpenAI-compatible reader reports usage only when both are numbers (ledger B94)',
      fn: async () => {
        /**
         Bodies each reporting one of the two counts, never the other.
         */
        const halfReported = [
          frameOf({
            body: {
              type: 'message_delta',
              delta: { stop_reason: 'end_turn', },
              usage: { output_tokens: 2, },
            },
          },),
          frameOf({
            body: {
              type: 'message_start',
              message: {
                id: 'msg_tabby',
                usage: { input_tokens: 8, },
              },
            },
          },),
          frameOf({
            body: {
              type: 'message_start',
              message: {
                id: 'msg_tabby',
                usage: {
                  input_tokens: 8,
                  cache_read_input_tokens: 300,
                },
              },
            },
          },),
        ];
        for (const counts of halfReported) {
          expect(extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: counts
              + deltaOf({
                deltaType: 'text_delta',
                field: 'text',
                text: 'Pleased.',
              },)
              + frameOf({ body: { type: 'message_stop', }, },),
          },).usage,).toBe(undefined,);
        }
      },
    },),
    it({
      name: 'REFUSES a payload that parses to a number, since every Anthropic frame is an object '
        + 'and the answer assembled from the rest would be missing a piece',
      fn: async () => {
        expect(function numbered() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: `${startOf({ inputTokens: 8, },)}data: 7\n\n${endOf({
              stopReason: 'end_turn',
              outputTokens: 1,
            },)}`,
          },);
        },).toThrow('stream event is not a JSON object',);
      },
    },),

    it({
      name: 'REFUSES a payload that parses to an array, since every Anthropic frame is an object and '
        + 'reading one as a frame named no type and was silently passed over rather than refused '
        + '(ledger B92)',
      fn: async () => {
        expect(function arrayed() {
          extractAnthropicCompletion({
            provider: 'hyper',
            bodyText: `${startOf({ inputTokens: 8, },)}data: [1]\n\n${endOf({
              stopReason: 'end_turn',
              outputTokens: 1,
            },)}`,
          },);
        },).toThrow('stream event is not a JSON object',);
      },
    },),

    it({
      name: 'READS THE EXACT STREAM THIS PROVIDER SENT ON 2026-08-24, keep-alive ping and all, '
        + 'which is the only case here taken off the wire rather than written by hand',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: [
            'data: {"type":"message_start","message":{"id":"msg_1","type":"message","role":"assistant",'
              + '"content":[],"model":"gemma-4-26b-a4b-it","stop_reason":null,"stop_sequence":null,'
              + '"usage":{"input_tokens":0,"output_tokens":0}}}',
            'data: {"type":"ping"}',
            'data: {"type":"content_block_start","index":0,"content_block":{"type":"tool_use",'
              + '"id":"toolu_1","name":"whisker_report","input":{}}}',
            // BUILT RATHER THAN QUOTED, and only this frame. Its payload is JSON
            // inside a JSON string, so writing it literally means escaping every
            // quote twice and reading it means unescaping twice. The bytes are
            // the same either way.
            `data: ${JSON.stringify({
              type: 'content_block_delta',
              index: 0,
              delta: {
                type: 'input_json_delta',
                partial_json: '{"toebeans": 4}',
              },
            },)}`,
            'data: {"type":"content_block_stop","index":0}',
            'data: {"type":"message_delta","delta":{"stop_reason":"tool_use","stop_sequence":null},'
              + '"usage":{"input_tokens":102,"output_tokens":15}}',
            'data: {"type":"message_stop"}',
            '',
          ].join('\n\n',),
        },),).toEqual({
          text: '{"toebeans": 4}',
          finishReason: 'tool_use',
          usage: {
            prompt_tokens: 102,
            completion_tokens: 15,
            total_tokens: 117,
          },
        },);
      },
    },),

    it({
      name: 'SKIPS THE [DONE] SENTINEL OPENROUTER APPENDS AFTER message_stop, taken off the wire '
        + 'on 2026-09-03 (deepseek/deepseek-v4-flash-0731 via DigitalOcean): the answer arrives '
        + 'in several input_json_delta pieces, usage and cost ride on message_delta, and the '
        + 'stop reason is end_turn rather than tool_use, none of which may cost the answer',
      fn: async () => {
        expect(extractAnthropicCompletion({
          provider: 'hyper',
          bodyText: [
            'event: message_start',
            'data: {"type":"message_start","message":{"id":"gen-1788450198-boHX6M5376fQSYqjwNP4",'
              + '"type":"message","role":"assistant","container":null,"content":[],'
              + '"model":"deepseek/deepseek-v4-flash-0731","stop_reason":null,"stop_details":null,'
              + '"stop_sequence":null,"usage":{"input_tokens":0,"output_tokens":0,'
              + '"output_tokens_details":null,"cache_creation_input_tokens":null,'
              + '"cache_read_input_tokens":null,"cache_creation":null,"inference_geo":null,'
              + '"server_tool_use":null,"service_tier":null,"speed":"standard"},'
              + '"provider":"DigitalOcean"}}',
            'event: content_block_start',
            'data: {"type":"content_block_start","index":0,"content_block":{"type":"thinking",'
              + '"thinking":"","signature":""}}',
            'event: content_block_delta',
            'data: {"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta",'
              + '"thinking":"The user wants"}}',
            'event: content_block_stop',
            'data: {"type":"content_block_stop","index":0}',
            'event: content_block_start',
            'data: {"type":"content_block_start","index":1,"content_block":{"type":"tool_use",'
              + '"id":"chatcmpl-tool-ad5286db98ea77ad","caller":{"type":"direct"},"name":"verdict",'
              + '"input":{}}}',
            // BUILT RATHER THAN QUOTED, as the 2026-08-24 case explains: JSON
            // inside a JSON string. Three pieces, as the gateway split them.
            ...[
              '{"translation": "She',
              ' left.", "ambiguous": true, ',
              '"reason": "Context decides."}',
            ].map(function pieceOf(partial: string,): string {
              return `event: content_block_delta\ndata: ${JSON.stringify({
                type: 'content_block_delta',
                index: 1,
                delta: {
                  type: 'input_json_delta',
                  partial_json: partial,
                },
              },)}`;
            },),
            'event: content_block_stop',
            'data: {"type":"content_block_stop","index":1}',
            'event: message_delta',
            'data: {"type":"message_delta","delta":{"container":null,"stop_details":null,'
              + '"stop_reason":"end_turn","stop_sequence":null},"usage":{"input_tokens":574,'
              + '"output_tokens":306,"output_tokens_details":{"thinking_tokens":191},'
              + '"cache_creation_input_tokens":null,"cache_read_input_tokens":128,'
              + '"cache_creation":null,"server_tool_use":null,"service_tier":null,'
              + '"speed":"standard","cost":0.000126255304,"is_byok":false,'
              + '"cost_details":{"upstream_inference_cost":0.000126255304,'
              + '"upstream_inference_prompt_cost":0.000049143304,'
              + '"upstream_inference_completions_cost":0.000077112}},"context_management":null}',
            'event: message_stop',
            'data: {"type":"message_stop"}',
            'event: data',
            'data: [DONE]',
            '',
          ].join('\n\n',),
        },),).toEqual({
          text: '{"translation": "She left.", "ambiguous": true, "reason": "Context decides."}',
          finishReason: 'end_turn',
          // 574 FRESH PLUS 128 READ FROM THE CACHE (ledger B88). This case
          // expected 574 before, which wrote the dropped cache read into the
          // capture's reading. Whether this gateway's input_tokens already
          // counts the cached ones was not settled: today's endpoint prices
          // matched neither reading of the capture's own cost.
          usage: {
            prompt_tokens: 702,
            completion_tokens: 306,
            total_tokens: 1_008,
          },
        },);
      },
    },),

  ],
},);
