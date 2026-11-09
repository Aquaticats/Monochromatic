/**
 Filter pipeline for Bash tool output.
 
 `runFilter` is the entry-point function called by the per-plugin filter shim.
 It reads stdin, applies all transformations, and writes filtered output to
 stdout.
 
 Which failures keep the tool output: a failed stdin read still forwards every
 byte that reached the filter before the stream broke, and a failed
 transformation writes the input back unchanged. Both report on stderr, which
 carries no hook protocol. Losing a command's output is worse than showing it
 unfiltered.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  collapseCwdPaths,
  collapseHomePaths,
  collapseRepeatedChars,
  flushRepeated,
  shouldStripLine,
  truncateLine,
} from './filter-transforms.ts';

/**
 Module logger for the filter pipeline.
 */
const loggerForFilter = tagged({ tag: 'bash-output-filter', },);

/**
 What one stdin read produced: every byte received, and the failure that stopped
 it early when there was one.
 */
type StdinRead = {
  /**
   Text of every byte that reached the filter, empty when the stream failed before any.
   */
  readonly text: string;

  /**
   Why the read stopped before the end of the stream; absent when stdin was read whole.
   */
  readonly failure?: Error;
};

/**
 Applies all filter transformations to raw tool output.
 
 @param input - raw stdout/stderr text from the Bash tool
 
 @returns filtered text with boilerplate, long lines, duplicates, and trailing
   whitespace removed
 
 @example
 ```ts
 filterOutput('hello\nhello\nhello\nworld'); // 'hello (x3 repeated lines)\nworld'
 ```
 */
function filterOutput(input: string,): string {
  return collapseLines(input.split('\n',),)
    .join('\n',);
}

/**
 Applies per-line transformations and collapses consecutive duplicates,
 returning the resulting lines.
 
 Helper-function shape: encapsulates the mutable state machine
 (`prevLine`, `repeatCount`) needed to detect and collapse runs of
 identical lines without leaking those bindings to callers.
 
 @param lines - raw lines from the input, before transformations
 
 @returns processed lines with duplicates collapsed via {@link flushRepeated}
 
 @example
 ```ts
 const out = collapseLines(['hello', 'hello', 'world']);
 ```
 */
function collapseLines(lines: readonly string[],): string[] {
  /**
   Output buffer that receives each line (or `(xN)` marker) once duplicates are flushed.
   */
  const result: string[] = [];
  /**
   Last line emitted, compared against the next line to detect runs of duplicates.
   */
  let prevLine = '';
  /**
   Number of consecutive occurrences of `prevLine` so far in the current run.
   */
  let repeatCount = 0;

  for (const rawLine of lines) {
    /**
     Line with trailing whitespace removed before pattern checks and pipeline transforms.
     */
    const trimmed = rawLine.trimEnd();

    if (shouldStripLine(trimmed,))
      continue;

    /**
     Trimmed line after runs of identical characters are collapsed.
     */
    const collapsed = collapseRepeatedChars(trimmed,);
    /**
     Same line with absolute paths under the cwd rewritten as relative paths.
     */
    const relative = collapseCwdPaths(collapsed,);
    /**
     Same line with paths under the user's home rewritten as `~/...`.
     */
    const shortened = collapseHomePaths(relative,);
    /**
     Fully-processed line, possibly truncated when too long to display.
     */
    const processed = truncateLine(shortened,);

    if ((processed === prevLine) && (repeatCount > 0))
      repeatCount++;
    else {
      result.push(
        ...flushRepeated({
          line: prevLine,
          count: repeatCount,
        },),
      );
      prevLine = processed;
      repeatCount = 1;
    }
  }

  result.push(
    ...flushRepeated({
      line: prevLine,
      count: repeatCount,
    },),
  );

  return result;
}

/**
 Reads stdin to EOF, keeping whatever arrived when the stream fails part-way.
 
 A single `text(process.stdin)` call discards its accumulator when the stream
 errors, which would throw away a large command's output over one transient
 read failure. Chunks are collected here instead, so the failure can be
 reported without losing the bytes that did arrive. Encoding happens once over
 the joined bytes, so a multi-byte character split across two chunks still
 decodes correctly.
 
 @param stream - readable stream carrying the tool output
 
 @returns every byte received and, when the read failed, why
 
 @example
 ```ts
 const { text, failure } = await readStdin(process.stdin);
 ```
 */
async function readStdin(stream: NodeJS.ReadableStream,): Promise<StdinRead> {
  /**
   Raw chunks received so far, joined and decoded in one pass once the stream ends.
   */
  const chunks: Buffer[] = [];

  try {
    for await (const chunk of stream) {
      chunks.push(Buffer.isBuffer(chunk,)
        ? chunk
        : Buffer.from(chunk,));
    }
  }
  catch (error: unknown) {
    return {
      failure: Error.isError(error,) ? error : new Error(String(error,),),
      text: Buffer.concat(chunks,)
        .toString('utf8',),
    };
  }
  return { text: Buffer.concat(chunks,)
    .toString('utf8',) };
}

/**
 Entry-point for the filter script. Reads stdin to EOF via
 {@link readStdin}, applies the filter pipeline via
 {@link filterOutput}, and writes the result to stdout. On transform
 failure, writes the already-read stdin content unchanged; losing
 output is worse than failing to filter.
 
 @example
 ```ts
 await runFilter();
 ```
 */
async function runFilter(): Promise<void> {
  /**
   Every byte the Bash tool sent, plus why the read stopped early when it did.
   */
  const stdin = await readStdin(process.stdin,);

  if (stdin.failure !== undefined) {
    /**
     Read failure and the byte count that did reach the filter, both quoted in
     the diagnostic.
     */
    const { failure, } = stdin;
    /**
     Bytes forwarded unfiltered.
     */
    const forwardedBytes = Buffer.byteLength(stdin.text,);
    // stderr is safe here: stdout carries the tool output this filter exists to keep.
    process.stderr
      .write(`bash-output-filter: stdin read failed after ${forwardedBytes} bytes, forwarding them unfiltered: ${failure.message}\n`,);
  }

  try {
    /**
     Filtered text written back to stdout in the happy path.
     */
    const filtered = filterOutput(stdin.text,);
    process.stdout
      .write(filtered,);
  }
  catch (error: unknown) {
    // The unfiltered text still reaches stdout; the diagnostic says why it was
    // not filtered, so a permanently failing transform is visible.
    loggerForFilter.warn(`filtering failed, writing the input unchanged: ${String(error,)}`,);
    process.stdout
      .write(stdin.text,);
  }
}

export type { StdinRead, };

export {
  filterOutput,
  readStdin,
  runFilter,
};
