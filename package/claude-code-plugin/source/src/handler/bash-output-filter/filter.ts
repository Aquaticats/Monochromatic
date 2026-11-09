/**
 Filter pipeline for Bash tool output.
 
 `runFilter` is the entry-point function called by the per-plugin filter shim.
 It reads stdin, applies all transformations, and writes filtered output to
 stdout. On any error, it falls through and writes the original input
 unchanged so tool output is never lost.
 
 @module
 */

import { text, } from 'node:stream/consumers';
import {
  collapseCwdPaths,
  collapseHomePaths,
  collapseRepeatedChars,
  flushRepeated,
  shouldStripLine,
  truncateLine,
} from './filter-transforms.ts';

/**
 Sentinel returned when stdin cannot be read. A unique symbol keeps unavailable
 input distinct from a legitimately empty Bash output stream.
 */
const STDIN_UNAVAILABLE: unique symbol = Symbol('bash-output-filter/stdin-stream-could-not-be-read',);

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
 Reads stdin to EOF, returning an explicit sentinel when the stream fails.
 
 @returns stdin text, or {@link STDIN_UNAVAILABLE} when unavailable
 */
async function readStdin(): Promise<string | typeof STDIN_UNAVAILABLE> {
  try {
    return await text(process.stdin,);
  }
  catch (_error: unknown) {
    return STDIN_UNAVAILABLE;
  }
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
   Full stdin payload from the Bash tool, awaited to EOF before transforms run.
   */
  const input = await readStdin();
  if (input === STDIN_UNAVAILABLE)
    return;

  try {
    /**
     Filtered text written back to stdout in the happy path.
     */
    const filtered = filterOutput(input,);
    process.stdout
      .write(filtered,);
  }
  catch (_error: unknown) {
    process.stdout
      .write(input,);
  }
}

export {
  filterOutput,
  runFilter,
};
