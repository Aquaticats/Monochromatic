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
 Answer for a run that printed no crash report.

 A sentinel rather than a nullish value: absence is one of the answers this module
 reports, and the repository models absence without nullish unions.

 
 @internal
 */
export const NO_NATIVE_ANALYSIS_CRASH: unique symbol = Symbol('No native analysis crash in this run',);

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

 Both are written at column zero by the runtime itself: `panic:` for a panic value,
 `fatal error:` for the runtime's own fatal conditions such as a concurrent map write,
 which no `recover` can intercept.

 
 @internal
 */
const CRASH_HEADER_PREFIXES: readonly string[] = [
  'panic: ',
  'fatal error: ',
];

/**
 Text every goroutine dump line starts with, as in `goroutine 215 [running]:`.
 
 @internal
 */
const GOROUTINE_PREFIX = 'goroutine ';

/**
 Text that must follow a goroutine number on a dump line, which no ordinary sentence does.
 
 @internal
 */
const GOROUTINE_STATE_OPEN = ' [';

/**
 Text every signal line starts with, as in `[signal SIGSEGV: segmentation violation ...]`.
 
 @internal
 */
const SIGNAL_PREFIX = '[signal SIG';

/**
 The separator the walk splits captured output on.
 
 @internal
 */
const LINE_BREAK = '\n';

/**
 Answer from an offset search that found nothing.
 
 @internal
 */
const NOT_FOUND = -1;

/**
 Lowest and highest characters of an ASCII decimal digit, for scanning a goroutine number
 without a regular expression.
 
 @internal
 */
const DIGIT_LOW = '0';
/**
 Highest character of an ASCII decimal digit, paired with {@link DIGIT_LOW}.
 
 @internal
 */
const DIGIT_HIGH = '9';

//endregion Types and constants

//region Line shapes

/**
 Reports whether the text is one or more ASCII decimal digits and nothing else.

 @param text - Candidate goroutine number

 @returns whether every character is a decimal digit

 @example
 ```ts
 expect(isAsciiDigits('215')).toBe(true);
 ```
 */
function isAsciiDigits(text: string,): boolean {
  if (text.length === 0)
    return false;
  /**
   Characters of the candidate, so the digit check reads one of them at a time.
   */
  const characters = Array.from(text,);
  return characters.every(function isDigit(character,) {
    return (character >= DIGIT_LOW) && (character <= DIGIT_HIGH);
  },);
}

/**
 Reports whether the line heads a goroutine dump, as `goroutine 215 [running]:` does.

 @param line - One line of captured output

 @returns whether the line has the dump's shape
 */
function isGoroutineDumpLine(line: string,): boolean {
  if (!line.startsWith(GOROUTINE_PREFIX,))
    return false;
  /**
   Line without the `goroutine ` prefix.
   */
  const rest = line.slice(GOROUTINE_PREFIX.length,);
  /**
   Offset of the state bracket after the goroutine number.
   */
  const bracket = rest.indexOf(GOROUTINE_STATE_OPEN,);
  if (bracket <= 0)
    return false;
  /**
   Text before the state bracket, which must be the goroutine number.
   */
  const identifier = rest.slice(
    0,
    bracket,
  );
  return isAsciiDigits(identifier,);
}

/**
 Reports whether the line is the signal line a faulted Go program prints.

 @param line - One line of captured output

 @returns whether the line starts with the signal marker
 */
function isSignalLine(line: string,): boolean {
  return line.startsWith(SIGNAL_PREFIX,);
}

//endregion Line shapes

//region Detection

/**
 Finds the crash report a Go analysis child printed, if the run printed one.

 One linear pass over the captured standard error and two cursors that only move forward,
 so an unbounded stream costs one pass and constant stack. A crash header alone is not
 evidence; the report must also carry the goroutine dump or signal line the Go runtime
 writes beside it, which separates a real crash from a message that quotes one.

 @param stderr - Standard error Oxlint captured from the run, child output included

 @returns The crash header, or {@link NO_NATIVE_ANALYSIS_CRASH} when no report was printed

 @example
 ```ts
 detectNativeAnalysisCrash({ stderr: 'panic: runtime error: ...\ngoroutine 215 [running]:\n', });
 ```
 */
export function detectNativeAnalysisCrash({
  stderr,
}: {
  readonly stderr: string;
},): NativeAnalysisCrash | typeof NO_NATIVE_ANALYSIS_CRASH {
  return (function scan(): NativeAnalysisCrash | typeof NO_NATIVE_ANALYSIS_CRASH {
    /**
     Offset of the line being read, advanced past every line so the walk stays linear.
     */
    let cursor = 0;
    /**
     Crash header already read, kept until the report proves itself a crash.
     */
    let headerLine: string | typeof NO_NATIVE_ANALYSIS_CRASH = NO_NATIVE_ANALYSIS_CRASH;
    while (cursor < stderr.length) {
      /**
       Offset of the current line's terminator, or the end of the string.
       */
      const lineEnd = stderr.indexOf(
        LINE_BREAK,
        cursor,
      );
      /**
       Offset of the current line's terminator, or the end of the string when it has none.
       */
      const next = (lineEnd === NOT_FOUND) ? stderr.length : lineEnd;
      /**
       Current line, without its terminator.
       */
      const line = stderr.slice(
        cursor,
        next,
      );
      cursor = next + 1;
      if (headerLine === NO_NATIVE_ANALYSIS_CRASH) {
        /**
         Prefix that names the line as a crash header, when one does.
         */
        const headerPrefix = CRASH_HEADER_PREFIXES.find(function matches(prefix,) {
          return line.startsWith(prefix,);
        },);
        if (headerPrefix !== undefined)
          headerLine = line;
        continue;
      }
      if (isGoroutineDumpLine(line,) || isSignalLine(line,))
        return { headerLine, };
    }
    return NO_NATIVE_ANALYSIS_CRASH;
  })();
}

//endregion Detection
