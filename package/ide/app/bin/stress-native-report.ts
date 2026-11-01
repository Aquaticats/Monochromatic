// The tally of a stress run of the IDE's native tests (bin/stress-native.ts), as pure functions of nextest's output.
// What: reads the status lines of `cargo nextest run --status-level pass --failure-output immediate` logs and counts,
// per test, the passes, the passes after a retry (FLAKY), and the failures, keeping the first failure's output.
// Why: a flake rate is a count of runs, so the report must say how many runs stand behind each number and show the
// first panic message, which is the evidence of the cause. The functions are linear passes over the log lines.

/**
 What a caller reads: the per-test counts and the text to print.
 */
export type Tally = {
  readonly failures: number;
  readonly report: string
};

/**
 Counts of one test over all copies and iterations.
 */
type Counts = {
  pass: number;
  flaky: number;
  fail: number;
  failedAttempts: number;
  first: string | undefined
};

/**
 A nextest status line: the status word, the time, an optional iteration counter, the test counter and the name.
 Retry attempts (`TRY 1 FAIL`) match too, so they are counted apart from final results.
 */
const statusLine = /^\s*(PASS|FAIL|FLAKY \d+\/\d+|TRY \d+ (?:PASS|FAIL)|TIMEOUT|ABORT|SIG[A-Z]+|LEAK|LEAK-FAIL|SLOW|EXECFAIL)\s+\[[^\]]*\]\s+(?:\[[^\]]*\]\s+)?\(([^)]*)\)\s+\S+\s+(\S+)\s*$/;

/**
 A run of more tests than this lists only the tests that failed an attempt; fewer list every test.
 */
const listedCleanTests = 12;

/**
 Lines of the first failure's output that the report keeps.
 */
const keptLines = 30;

/**
 Final status words that are a failure of the test.
 */
const failureWords = new Set([
  'FAIL',
  'TIMEOUT',
  'ABORT',
  'LEAK-FAIL',
  'EXECFAIL'
]);

/**
 Count the final results of every test in some nextest logs.
 
 @param logs - the text of each copy's nextest output.
 
 @returns The number of failing final results and the report text, one block per test, ordered by name.
 */
export function summarize(logs: readonly string[]): Tally {
  const counts = new Map<string, Counts>();
  function entry(test: string): Counts {
    let found = counts.get(test);
    if (found === undefined) {
      found = {
        pass: 0,
        flaky: 0,
        fail: 0,
        failedAttempts: 0,
        first: undefined
      };
      counts.set(
        test,
        found
      );
    }
    return found;
  }
  for (const log of logs) {
    const lines = log.split('\n');
    for (let index = 0; index < lines.length; index += 1) {
      const match = statusLine.exec(lines[index] ?? '');
      if (match === null) {
        continue;
      }
      const words = (match[1] ?? '').split(' ');
      const word = words[0] === 'TRY' ? (words[2] ?? '') : (words[0] ?? '');
      const attempt = words[0] === 'TRY' ? Number(words[1]) : 1;
      // An attempt that a retry follows carries no test counter (`───`); the last attempt of a run does.
      const final = /^\d+\/\d+$/.test((match[2] ?? '').trim());
      const counted = entry(match[3] ?? '');
      const failed = failureWords.has(word) || word.startsWith('SIG');
      if (failed) {
        counted.failedAttempts += 1;
        if (counted.first === undefined) {
          // The failure output follows the status line, up to the next status line or the stress marker.
          // The first failing attempt is kept even when a retry then passes: it is the evidence of the flake.
          const block: string[] = [lines[index] ?? ''];
          for (let next = index + 1; (next < lines.length) && (block.length < keptLines); next += 1) {
            const line = lines[next] ?? '';
            if (statusLine.test(line) || line.startsWith(' Stress test')
              || line.startsWith('────')) {
              break;
            }
            block.push(line);
          }
          counted.first = block.join('\n');
        }
      }
      if (!final) {
        continue;
      }
      if (failed) {
        counted.fail += 1;
      } else if ((word === 'FLAKY') || ((word === 'PASS') && (attempt > 1))) {
        counted.flaky += 1;
      } else if (word === 'PASS') {
        counted.pass += 1;
      }
    }
  }
  let failures = 0;
  const blocks: string[] = [];
  const clean: string[] = [];
  const tests = [...counts.keys()].sort();
  for (const test of tests) {
    const counted = counts.get(test) as Counts;
    failures += counted.fail;
    const runs = counted.pass + counted.flaky
      + counted.fail;
    const head = `${test}: ${counted.fail} failed, ${counted.flaky} passed after a retry (FLAKY), ${counted.pass} passed, of ${runs} runs (${counted.failedAttempts} failed attempts including retries)`;
    if ((counted.failedAttempts === 0) && (tests.length > listedCleanTests)) {
      clean.push(test);
    } else {
      blocks.push(counted.first === undefined ? head : `${head}\nfirst failure:\n${counted.first}`);
    }
  }
  if (clean.length > 0) {
    blocks.push(`${clean.length} other tests had no failed attempt in any run.`);
  }
  if (blocks.length === 0) {
    blocks.push('No test result was found in the logs: the filter selected no test, or nextest did not run.');
    failures += 1;
  }
  return {
    failures,
    report: blocks.join('\n\n')
  };
}
