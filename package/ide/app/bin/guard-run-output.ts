// Whether one guard run reported a single passing test.
// What: the answer a guard-removal script needs before it accepts a removal it made: the run with the guard in place
// exited zero, printed the test name it expects, and reported the single test as passing.
// Why: nine guards scripts share this reading of the run's combined output, and the pattern that used to answer it is
// unnecessary: `1 test` or `1 passed` anywhere in the output is one substring scan each. The reading stays exactly as
// loose as the pattern was, including that `11 tests` carries `1 test`.
// Callers: inspect-watch-guards.ts, inspect-theme-guards.ts, inspect-source-guards.ts.

// The counters node:test prints for a run in which one test passed. The pattern matched either spelling anywhere in
// the output, which is what `String#includes` does here.
const ONE_TEST_PASSED_MARKERS = ['1 test', '1 passed'];

/**
 Whether a run's combined standard output and standard error report one passing test.

 @param output - the run's standard output and standard error, concatenated

 @returns whether the output holds one of the passing counters anywhere in it
 */
export function reportsOneTestPassed({ output }: { readonly output: string }): boolean {
  return ONE_TEST_PASSED_MARKERS.some(function reports(marker) { return output.includes(marker); });
}
