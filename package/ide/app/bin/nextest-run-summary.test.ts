// Checks of the nextest summary reader the guard scripts ask whether their run was the one they meant.
// What: each expected value is what the unanchored `1 (?:test|passed)` pattern the predicate replaced returned for the
// same output.
// Why: a guard run removes one guard from a disposable copy and accepts its run only when the summary says one test
// passed, so a predicate that answers yes to the wrong run accepts a guard that never ran.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { ranOneTest } from './nextest-run-summary.ts';

/// A nextest summary line, as the run prints one.
const summary = (counts: string): string => '     Summary [   0.039s] ' + counts + '\n';

test(
  'reads one passing test from the summary a run prints',
  () => {
    assert.equal(
      ranOneTest('    Starting 1 test across 1 binary\n        PASS [   0.039s] ide_app::guards::the_guard\n--------\n'
        + summary('1 test run: 1 passed, 0 skipped')),
      true
    );
  }
);

test(
  'reads either phrase the pattern reached',
  () => {
    assert.equal(ranOneTest('1 passed'), true);
    assert.equal(ranOneTest(summary('1 test run: 0 skipped')), true);
  }
);

test(
  'answers yes to a run of more than one test that holds the phrase',
  () => {
    assert.equal(ranOneTest(summary('11 tests run: 11 passed, 0 skipped')), true);
  }
);

test(
  'answers no to a run of several tests that holds neither phrase',
  () => {
    assert.equal(ranOneTest(summary('2 tests run: 2 passed, 0 skipped')), false);
    assert.equal(ranOneTest('10 passed'), false);
  }
);

test(
  'answers no to output that holds no result',
  () => {
    assert.equal(ranOneTest('error[E0308]: mismatched types'), false);
    assert.equal(ranOneTest('no result at all'), false);
    assert.equal(ranOneTest(''), false);
  }
);
