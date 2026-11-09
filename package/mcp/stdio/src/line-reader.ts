// Async iterator that yields complete newline-delimited lines from any async byte source.
// MCP stdio transport requires messages delimited by newlines with no embedded newlines.

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { diagnosticLogger, } from './diagnostic-logger.ts';

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site.
 */
const lineReaderLogger = tagged({
  l: diagnosticLogger,
  tag: 'line-reader',
},);

/**
 Yields complete lines from a byte stream, buffering partial reads across chunk boundaries.
 Each yielded string has the trailing newline stripped.
 
 Accepts any `AsyncIterable<Uint8Array>`: works with `ReadableStream`, `process.stdin`,
 and Node `Readable` streams without conversion or type casts.
 
 @param stream - Async iterable of byte chunks to consume.
 
 @returns Async generator yielding individual newline-delimited lines without the trailing newline character.
 
 @example
 ```ts
 for await (const line of readLines(process.stdin)) {
   handleLine(line);
 }
 ```
 */
export async function* readLines(
  stream: AsyncIterable<Uint8Array>,
): AsyncGenerator<string> {
  /**
   Reused decoder so multi-byte UTF-8 sequences split across chunks are stitched correctly via `stream: true`.
   */
  const decoder = new TextDecoder();
  /**
   Accumulates partial line data between chunk boundaries.
   
   Declared with `let` because each chunk appends new text and each yield slices off
   the emitted prefix; both operations require reassignment of the binding.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- parser cursor: buffer accumulates across stream chunks and is sliced after each yielded line, so reassignment is fundamental to the generator's contract
  let buffer = '';

  for await (const chunk of stream) {
    buffer += decoder.decode(
      chunk,
      { stream: true, },
    );

    /**
     Position of the next newline within `buffer`, or `-1` when no further complete line exists.
     
     Updated inside the loop as each line is sliced off and emitted, so a single chunk
     containing several newline-delimited messages yields each in turn.
     */
    let newlineIndex = buffer.indexOf('\n',);
    while (newlineIndex !== (-1)) {
      yield buffer.slice(
        0,
        newlineIndex,
      );
      buffer = buffer.slice(newlineIndex + 1,);
      newlineIndex = buffer.indexOf('\n',);
    }
  }

  // Flush the decoder itself: without a final zero-length decode, a multi-byte UTF-8
  // sequence split across the stream's last chunk boundary stays buffered inside the
  // decoder and never reaches the caller.
  buffer += decoder.decode();

  // Flush remaining data after stream closes without a trailing newline.
  // Not expected in normal MCP usage (clients send newline-terminated messages),
  // but logged so protocol issues are visible during debugging.
  if (buffer.length
    > 0) {
    tagged({
      tag: readLines.name,
      l: lineReaderLogger,
    },)
      .debug(`flushing trailing buffer without newline terminator: ${buffer}`,);
    yield buffer;
  }
}
