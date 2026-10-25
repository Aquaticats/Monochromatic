import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { earlierStopTexts, } from './turn-history.ts';

/**
 Transcript line for a turn the user typed, which closes the window.
 */
const HUMAN = JSON.stringify({ type: 'user', origin: { kind: 'human', }, message: { content: 'go', }, },);

/**
 Transcript line for an assistant record.

 @param blocks - content blocks of the record

 @param extra - fields merged into the record, such as `isSidechain`

 @returns one JSONL line

 @example
 ```ts
 assistant({ blocks: [{ type: 'text', text: 'Done.' }] });
 ```
 */
function assistant(
  {
    blocks,
    extra = {},
  }: {
    readonly blocks: readonly Readonly<Record<string, unknown>>[];
    readonly extra?: Readonly<Record<string, unknown>>;
  },
): string {
  return JSON.stringify({ type: 'assistant', message: { content: blocks, }, ...extra, },);
}

/**
 Transcript line for an assistant stop with one text block.

 @param text - the stop's prose

 @returns one JSONL line

 @example
 ```ts
 stop('Waiting for the build.');
 ```
 */
function stop(text: string,): string {
  return assistant({ blocks: [{ type: 'text', text, },], },);
}

/**
 Transcript line for the feedback record Claude Code writes when the hook blocks.
 */
const FEEDBACK = JSON.stringify({ type: 'user', message: { content: 'Stop hook feedback:\nreminder', }, },);

await describe({
  name: earlierStopTexts.name,
  children: [
    it({
      name: 'returns the earlier stops of this turn oldest first, without the final message',
      fn: async () => {
        expect(earlierStopTexts({
          transcriptLines: [HUMAN, stop('first stop',), FEEDBACK, stop('second stop',), FEEDBACK, stop('final stop',),],
          finalMessage: 'final stop',
        },),).toEqual(['first stop', 'second stop',],);
      },
    },),
    it({
      name: 'keeps the newest stop when it is not the final message',
      fn: async () => {
        expect(earlierStopTexts({
          transcriptLines: [HUMAN, stop('first stop',),],
          finalMessage: 'a message the transcript has not caught up with',
        },),).toEqual(['first stop',],);
      },
    },),
    it({
      name: 'stops at the last human turn',
      fn: async () => {
        expect(earlierStopTexts({
          transcriptLines: [HUMAN, stop('old turn',), HUMAN, stop('this turn',), stop('final',),],
          finalMessage: 'final',
        },),).toEqual(['this turn',],);
      },
    },),
    it({
      name: 'ignores an assistant record that issued a tool call',
      fn: async () => {
        expect(earlierStopTexts({
          transcriptLines: [
            HUMAN,
            assistant({ blocks: [{ type: 'text', text: 'running it', }, { type: 'tool_use', name: 'Bash', },], },),
            stop('final',),
          ],
          finalMessage: 'final',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'ignores a subagent record and a record without text',
      fn: async () => {
        expect(earlierStopTexts({
          transcriptLines: [
            HUMAN,
            assistant({ blocks: [{ type: 'text', text: 'subagent says', },], extra: { isSidechain: true, }, },),
            assistant({ blocks: [{ type: 'thinking', },], },),
            stop('final',),
          ],
          finalMessage: 'final',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'joins the text blocks of one stop',
      fn: async () => {
        expect(earlierStopTexts({
          transcriptLines: [
            HUMAN,
            assistant({ blocks: [{ type: 'text', text: 'part one', }, { type: 'text', text: 'part two', },], },),
            stop('final',),
          ],
          finalMessage: 'final',
        },),).toEqual(['part one\npart two',],);
      },
    },),
    it({
      name: 'does not take a human turn quoted inside a tool result for the boundary',
      fn: async () => {
        /**
         Tool result that prints a human-origin marker, which must not end the scan.
         */
        const quoted = JSON.stringify({
          type: 'user',
          origin: { kind: 'human', },
          toolUseResult: { task: {}, },
          message: { content: [{ type: 'tool_result', content: '"kind":"human"', },], },
        },);

        expect(earlierStopTexts({
          transcriptLines: [HUMAN, stop('first stop',), quoted, stop('final',),],
          finalMessage: 'final',
        },),).toEqual(['first stop',],);
      },
    },),
    it({
      name: 'skips a truncated line and returns nothing for an empty transcript',
      fn: async () => {
        expect(earlierStopTexts({
          transcriptLines: [HUMAN, stop('first stop',), '{"type":"assistant","message":{"content":[{"type":"te',],
          finalMessage: 'final',
        },),).toEqual(['first stop',],);
        expect(earlierStopTexts({ transcriptLines: [], finalMessage: 'final', },),).toEqual([],);
      },
    },),
  ],
},);
