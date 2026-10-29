import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assistantTextOfLine,
  readActivityWord,
} from '../dist/final/node/statusline.mjs';

/**
 Builds one assistant JSONL entry.
 
 @param blocks - message content blocks
 
 @returns one JSONL line without trailing newline
 
 @example
 ```ts
 assistantLine([{ type: 'text', text: 'Compiling' }]);
 ```
 */
function assistantLine(blocks: readonly unknown[],): string {
  return JSON.stringify({
    type: 'assistant',
    message: { content: blocks, },
  },);
}

/**
 Builds an assistant entry holding one text block.
 
 @param text - prose of the text block
 
 @returns one JSONL line without trailing newline
 
 @example
 ```ts
 assistantTextLine('Compiling');
 ```
 */
function assistantTextLine(text: string,): string {
  return assistantLine([{ type: 'text', text, },],);
}

/**
 Throwaway transcript directory that removes itself at the end of an `await using` scope.
 */
type TempTranscript = {
  /**
   Absolute path to the transcript file.
   */
  readonly path: string;
  /**
   Removes the directory holding the transcript.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Writes lines to a throwaway transcript.
 
 @param lines - JSONL lines in file order
 
 @returns disposable transcript handle
 
 @example
 ```ts
 await using transcript = await makeTranscript(['{}']);
 ```
 */
async function makeTranscript(lines: readonly string[],): Promise<TempTranscript> {
  /**
   Throwaway directory holding the transcript.
   */
  const directory = await mkdtemp(join(
    scratchRoot(),
    'claude-statusline-activity-',
  ),);
  /**
   Transcript path fixture.
   */
  const path = join(
    directory,
    'session.jsonl',
  );
  await writeFile(
    path,
    lines
      .map(function endLine(line,): string {
        return `${line}\n`;
      },)
      .join('',),
  );
  return {
    path,
    [Symbol.asyncDispose]: async function removeDirectory(): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Writes lines to a throwaway transcript and reads its activity word.
 
 @param lines - JSONL lines in file order
 
 @returns activity word from {@link readActivityWord}
 
 @example
 ```ts
 await activityWordOfLines(['{}']);
 ```
 */
async function activityWordOfLines(lines: readonly string[],): Promise<string> {
  await using transcript = await makeTranscript(lines,);
  return await readActivityWord({ transcriptPath: transcript.path, },);
}

await describe({
  name: 'activity',
  children: [
    describe({
      name: assistantTextOfLine.name,
      children: [
        it({
          name: 'joins text blocks and thinking summaries, ignoring signatures and tool_use blocks',
          fn: async function testBlocks(): Promise<void> {
            /**
             Assistant entry mixing prose, a thinking summary, and a tool call.
             */
            const line = assistantLine([
              { type: 'thinking', thinking: 'Pondering', signature: 'Jazzy-nibbling', },
              { type: 'text', text: 'Compiling', },
              { type: 'tool_use', name: 'Running', input: { command: 'Searching', }, },
              { type: 'text', text: 'Testing', },
            ],);
            expect(assistantTextOfLine(line,),).toBe('Pondering\nCompiling\nTesting',);
          },
        },),
        it({
          name: 'ignores non-assistant entries even when they mention the marker',
          fn: async function testNonAssistant(): Promise<void> {
            /**
             User entry whose content mentions the assistant marker.
             */
            const line = JSON.stringify({
              type: 'user',
              message: { content: '"type":"assistant" Compiling', },
            },);
            expect(assistantTextOfLine(line,),).toBe('',);
          },
        },),
        it({
          name: 'returns empty for unparsable lines and string content',
          fn: async function testMalformed(): Promise<void> {
            expect(assistantTextOfLine('{"type":"assistant", cut off',),).toBe('',);
            /**
             Assistant entry whose content is a bare string.
             */
            const stringContentLine = JSON.stringify({
              type: 'assistant',
              message: { content: 'Compiling', },
            },);
            expect(assistantTextOfLine(stringContentLine,),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: readActivityWord.name,
      children: [
        it({
          name: 'returns the last gerund of the newest assistant text',
          fn: async function testNewest(): Promise<void> {
            expect(await activityWordOfLines([
              assistantTextLine('Earlier I was refactoring',),
              assistantTextLine('Now compiling and then testing',),
            ],),).toBe('Testing',);
          },
        },),
        it({
          name: 'ignores gerund-like words in tool results, attachments, and escaped newlines',
          fn: async function testNonProse(): Promise<void> {
            expect(await activityWordOfLines([
              assistantTextLine('Now compiling',),
              JSON.stringify({
                type: 'user',
                message: { content: [{ type: 'tool_result', content: '"Jazzy-nibbling\\\nnothing', },], },
              },),
              JSON.stringify({
                type: 'attachment',
                attachment: { text: 'Wild-nibbling spinner', },
              },),
            ],),).toBe('Compiling',);
          },
        },),
        it({
          name: 'counts a newer thinking summary over older assistant text',
          fn: async function testThinkingSummary(): Promise<void> {
            expect(await activityWordOfLines([
              assistantTextLine('Now compiling',),
              assistantLine([{ type: 'thinking', thinking: 'Cherry-picking the fix', signature: 'Jazzy-nibbling', },],),
            ],),).toBe('Cherry-picking',);
          },
        },),
        it({
          name: 'skips a cut-off first line when the transcript exceeds the tail window',
          fn: async function testCutOff(): Promise<void> {
            expect(await activityWordOfLines([
              assistantTextLine('Compiling',),
              JSON.stringify({ type: 'user', filler: 'x'.repeat(100_000,), },),
            ],),).toBe('',);
          },
        },),
        it({
          name: 'returns empty for a missing transcript',
          fn: async function testMissing(): Promise<void> {
            expect(await readActivityWord({ transcriptPath: join(
              scratchRoot(),
              'claude-statusline-activity-does-not-exist.jsonl',
            ), },),).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
