// The summary a nextest run prints, read as the one question a guard run asks of it.
// What: whether the run reported one test and that test passing.
// Why: a guard run removes one guard from a disposable copy of the package and runs the one test the guard protects; a
// run whose summary says anything else is not the run the guard is about, whatever its exit status. The summary line
// reads `Summary [ 0.039s] 1 test run: 1 passed, 0 skipped`, so both phrases the pattern this replaced looked for are
// present there.

/// The phrase a nextest summary carries when it counted one test.
const ONE_TEST = '1 test';

/// The phrase it carries when it counted one passing test.
const ONE_PASS = '1 passed';

/**
 Whether a nextest run's output reports one test and that test passing.

 What: two substring searches, the answer an unanchored `1 (?:test|passed)` pattern gave to the same question.

 Why: the phrases are what a summary states, so no count is parsed out of them: `11 tests run` holds `1 test` and
 answers true, exactly as the pattern did, and a guard that must tell one test from eleven needs the summary read
 rather than this predicate called.

 @param output - the run's standard output and standard error, joined
 @returns whether the output holds either phrase
 */
export function ranOneTest(output: string): boolean {
  return output.includes(ONE_TEST) || output.includes(ONE_PASS);
}
