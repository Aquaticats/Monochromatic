/**
 Detect a crashed native analysis child in one Oxlint run's captured standard error.

 Oxlint's type-aware pass runs two kinds of helper process beside itself: `tsgolint`,
 which implements the `typescript/*` rules on the `typescript-go` engine, and the
 TypeScript 7 API server that a JavaScript plugin reaches through `typescript/sync`.
 Both are Go programs, so a crash inside one prints a Go panic report on the
 inheriting standard error and leaves Oxlint itself running with one fewer analyzer.

 Oxlint's own exit status cannot express that: it counts the diagnostics it produced,
 so a run whose semantic analyzer died while nothing else was found exits zero,
 exactly like a clean run. This module supplies the missing evidence, so the wrapper
 can fail the run instead of letting a crashed analyzer look like a clean tree.

 The report is recognized from its own shape rather than from a rule or file name,
 because the crashing analyzer differs per rule family and the panic value differs per
 bug: a crash header (`panic: ...`, or the Go runtime's own `fatal error: ...`) followed
 by the runtime's goroutine dump or its signal line. Ordinary prose that merely
 contains the word `panic` matches neither part.

 @module
 */

//region Types and constants

/**
 Evidence that a Go analysis child of Oxlint died during the run.

 @internal
 */
export type NativeAnalysisCrash = {
  /**
   First line of the crash report, which names the failure as the crashed program printed
   it, such as `panic: runtime error: invalid memory address or nil pointer dereference`.
   */
  readonly headerLine: string;
};

/**
 Heads of a Go runtime crash report.

 Both are written at column zero by the runtime itself: `panic:` for a recovered or
 unrecovered panic value, `fatal error:` for the runtime's own fatal conditions such as
 a concurrent map write, which no `recover` can intercept.

 @internal
 */
const CRASH_HEADER_PREFIXES: readonly string[] = [
  'panic: ',
  'fatal error: ',
];

/**
 Start of the goroutine dump every Go crash report ends with, as in `goroutine 215 [running]:`.
 @internal
 */
const GOROUTINE_DUMP_LINE = /^goroutine \d+ \[/;

/**
 Start of the signal line a Go crash report prints for a fault, as in
 `[signal SIGSEGV: segmentation violation code=0x1 addr=0x8 pc=0xbf31dc]`.
 @internal
 */
const SIGNAL_LINE = /^\[signal SIG/;

//endregion Types and constants

//region Detection

/**
 Finds the crash report a Go analysis child printed, if the run printed one.

 One linear pass over the captured standard error and a single `string | undefined`
 cursor, so an unbounded stream costs one pass and constant stack. A crash header alone is
 not evidence; the report must also carry the goroutine dump or signal line the Go runtime
 writes beside it, which separates a real crash from a message that quotes one.

 @param stderr - Standard error Oxlint captured from the run, child output included

 @returns The crash header, or `undefined` when the run printed no crash report

 @example
 ```ts
 detectNativeAnalysisCrash({ stderr: 'panic: runtime error: ...\ngoroutine 215 [running]:\n', });
 ```
 */
export function detectNativeAnalysisCrash({
  stderr,
}: {
  readonly stderr: string;
},): NativeAnalysisCrash | undefined {
  /**
   Offset of the line being read, advanced past every line so the walk stays linear.
   */
  let cursor = 0;
  /**
   Crash header already read, remembered until the report proves itself a crash.
   */
  let headerLine: string | undefined;
  while (cursor < stderr.length) {
    /**
     Offset just past the current line's newline, or past the end for a final line without one.
     */
    const lineEnd = stderr.indexOf('\n', cursor,);
    /**
     Offset of the current line's terminator, or the end of the string.
     */
    const next = (lineEnd === -1) ? stderr.length : lineEnd;
    /**
     Current line, without its terminator.
     */
    const line = stderr.slice(cursor, next,);
    cursor = next + 1;
    if ((headerLine === undefined) && CRASH_HEADER_PREFIXES.some(function heads(prefix,) {
      return line.startsWith(prefix,);
    },))
      headerLine = line;
    else if ((headerLine !== undefined) && (GOROUTINE_DUMP_LINE.test(line,) || SIGNAL_LINE.test(line,)))
      return { headerLine, };
  }
  return undefined;
}

//endregion Detection
