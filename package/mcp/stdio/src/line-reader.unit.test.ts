import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readLines, } from '@monochromatic-dev/mcp-stdio';

//region helpers: create ReadableStream from string content

/**
 Creates a ReadableStream from a string, encoding it as a single UTF-8 chunk.
 
 @param content - Raw string to stream.
 @returns ReadableStream yielding a single Uint8Array chunk.
 */
function streamFromString(content: string,): ReadableStream<Uint8Array> {
  const encoder = new TextEncoder();
  return new ReadableStream({
    start(controller: ReadableStreamDefaultController<Uint8Array>,) {
      controller.enqueue(encoder.encode(content,),);
      controller.close();
    },
  },);
}

/**
 Creates a ReadableStream from multiple string chunks, each encoded separately.
 Simulates data arriving in pieces across chunk boundaries.
 
 @param chunks - Strings to enqueue as separate chunks.
 @returns ReadableStream yielding one Uint8Array per chunk.
 */
function streamFromChunks(chunks: readonly string[],): ReadableStream<Uint8Array> {
  const encoder = new TextEncoder();
  return new ReadableStream({
    start(controller: ReadableStreamDefaultController<Uint8Array>,) {
      for (const chunk of chunks)
        controller.enqueue(encoder.encode(chunk,),);
      controller.close();
    },
  },);
}

/**
 Collects all lines from a readLines async generator into an array.
 
 @param stream - Input byte stream.
 @returns Array of yielded lines.
 */
async function collectLines(
  stream: ReadableStream<Uint8Array>,
): Promise<readonly string[]> {
  const lines: string[] = [];
  for await (const line of readLines(stream,))
    lines.push(line,);
  return lines;
}

//endregion helpers

//region readLines: async generator yielding newline-delimited lines

await describe({
  name: readLines.name,
  children: [
    it({
      name: 'yields single line from newline-terminated input',
      fn: async () => {
        const lines = await collectLines(streamFromString('hello\n',),);
        expect(lines,).toEqual(['hello',],);
      },
    },),
    it({
      name: 'yields multiple lines from single chunk',
      fn: async () => {
        const lines = await collectLines(streamFromString('line1\nline2\nline3\n',),);
        expect(lines,).toEqual(['line1', 'line2', 'line3',],);
      },
    },),
    it({
      name: 'handles line split across chunk boundaries',
      fn: async () => {
        const lines = await collectLines(streamFromChunks(['hel', 'lo\nwor', 'ld\n',],),);
        expect(lines,).toEqual(['hello', 'world',],);
      },
    },),
    it({
      name: 'yields empty array for empty stream',
      fn: async () => {
        const lines = await collectLines(streamFromString('',),);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'flushes trailing content without final newline',
      fn: async () => {
        const lines = await collectLines(streamFromString('no-newline',),);
        expect(lines,).toEqual(['no-newline',],);
      },
    },),
    it({
      name: 'handles empty lines between content',
      fn: async () => {
        const lines = await collectLines(streamFromString('a\n\nb\n',),);
        expect(lines,).toEqual(['a', '', 'b',],);
      },
    },),
    it({
      name: 'handles multiple consecutive newlines',
      fn: async () => {
        const lines = await collectLines(streamFromString('\n\n\n',),);
        expect(lines,).toEqual(['', '', '',],);
      },
    },),
    it({
      name: 'handles single newline',
      fn: async () => {
        const lines = await collectLines(streamFromString('\n',),);
        expect(lines,).toEqual(['',],);
      },
    },),
    it({
      name: 'handles chunk ending exactly at newline',
      fn: async () => {
        const lines = await collectLines(streamFromChunks(['line1\n', 'line2\n',],),);
        expect(lines,).toEqual(['line1', 'line2',],);
      },
    },),
    it({
      name: 'handles JSON-RPC style messages',
      fn: async () => {
        const message1 = '{"jsonrpc":"2.0","id":1,"method":"server/discover"}';
        const message2 = '{"jsonrpc":"2.0","method":"notifications/cancelled"}';
        const lines = await collectLines(
          streamFromString(`${message1}\n${message2}\n`,),
        );
        expect(lines,).toEqual([message1, message2,],);
      },
    },),
    it({
      name: 'stitches a multi-byte character split across a chunk boundary',
      fn: async () => {
        /** UTF-8 bytes of a snowman, split so neither chunk decodes on its own. */
        const snowmanBytes = new TextEncoder().encode('☃',);
        const stream = new ReadableStream<Uint8Array>({
          start(controller: ReadableStreamDefaultController<Uint8Array>,) {
            controller.enqueue(snowmanBytes.slice(0, 1,),);
            controller.enqueue(snowmanBytes.slice(1,),);
            controller.enqueue(new TextEncoder().encode('\n',),);
            controller.close();
          },
        },);
        const lines = await collectLines(stream,);
        expect(lines,).toEqual(['☃',],);
      },
    },),
    it({
      name: 'flushes a multi-byte character truncated at end of stream',
      fn: async () => {
        /** Stream ends mid-character, so the decoder must flush a replacement character. */
        const snowmanBytes = new TextEncoder().encode('☃',);
        const stream = new ReadableStream<Uint8Array>({
          start(controller: ReadableStreamDefaultController<Uint8Array>,) {
            controller.enqueue(snowmanBytes.slice(0, 2,),);
            controller.close();
          },
        },);
        const lines = await collectLines(stream,);
        expect(lines,).toEqual(['�',],);
      },
    },),
  ],
},);

//endregion readLines
