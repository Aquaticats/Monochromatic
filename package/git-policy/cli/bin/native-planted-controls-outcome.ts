/**
 Verdict of one planted guard removal, read from the output of the suite that ran against the planted source.

 A suite that never started because the planted source did not compile says nothing about the guard, so it gets its
 own verdict instead of passing for a guard that no test noticed.
 */

/**
 What a planted guard removal's run showed.
 `build-failed` carries the compiler's error lines; the other two carry nothing further.
 */
export type PlantedRunVerdict =
  | {
    readonly kind: 'noticed';
  }
    | {
      readonly kind: 'not-noticed';
    }
    | {
      readonly kind: 'build-failed';
      readonly detail: string;
    };

/**
 Most compiler error lines kept in a build failure's detail, so a log full of cascading errors stays readable.
 */
const maxDetailLines = 8;

/**
 Whether a line is one Cargo prints for a build it stopped.

 @param line - one output line, without its newline

 @returns true for `error: could not compile ...`
 */
function isCompileStop(line: string): boolean {
  return line.startsWith('error: could not compile');
}

/**
 Whether a line is a compiler error, which starts at the first column as `error: ` or `error[E0000]: `.
 Cargo's closing `error: command ... exited with code` line only repeats the arguments it ran, so it is not one.

 @param line - one output line, without its newline

 @returns true for an error diagnostic
 */
function isCompilerError(line: string): boolean {
  return line.startsWith('error[E')
    || (line.startsWith('error: ') && (!line.startsWith('error: command ')));
}

/**
 Whether a line is nextest's report of one failed test whose name is `failing`.
 nextest prints `FAIL [ 0.012s] (3/9) <binary id> <test name>`, or `FAIL + LEAK [` when the failed test left a child
 running, or `TIMEOUT [ 300.002s]` when the crate's `slow-timeout` stopped a test that ran too long (nextest counts that
 as a failure, and a removal that makes its control hang has been noticed), so the name is the last token and the line
 holds a time in brackets.

 @param line - one output line, without its newline

 @param failing - test name the planted removal must fail

 @returns true when the line reports that test as failed or stopped for running too long
 */
function reportsFailure({
  line,
  failing,
}: {
  readonly line: string;
  readonly failing: string;
}): boolean {
  /**
   The line without its leading padding, where nextest puts the result word.
   */
  const reported = line.trimStart();
  return (reported.startsWith('FAIL ') || reported.startsWith('TIMEOUT '))
    && line.includes('[')
    && line.trimEnd()
      .endsWith(` ${failing}`);
}

/**
 Decide what a planted removal's suite run showed.

 @param output - both output streams of the run

 @param status - exit status of the run, absent when a signal ended it

 @param failing - test name the planted removal must fail

 @returns `build-failed` when Cargo stopped the build, otherwise `noticed` when the run failed and reported the named
   test as failed, and `not-noticed` in every other case

 @example
 ```ts
 const verdict = judgePlantedRun({ output, status: 100, failing: 'entry::stop_tests::forward_target_naming_this_executable_stops' });
 ```
 */
export function judgePlantedRun({
  output,
  status,
  failing,
}: {
  readonly output: string;
  readonly status?: number;
  readonly failing: string;
}): PlantedRunVerdict {
  /**
   Every output line.
   */
  const lines = output.split('\n');
  if (lines.some(isCompileStop))
    return {
      kind: 'build-failed',
      detail: lines.filter(isCompilerError)
        .slice(
        0,
        maxDetailLines,
        )
        .join('\n'),
    };
  /**
   Whether the run failed and reported the named test among its failures.
   */
  const namedTestFailed = ((status !== undefined) && (status !== 0))
    && lines.some(function isNamedFailure(line) {
      return reportsFailure({
        line,
        failing,
      });
    });
  if (namedTestFailed)
    return { kind: 'noticed', };
  return { kind: 'not-noticed', };
}
