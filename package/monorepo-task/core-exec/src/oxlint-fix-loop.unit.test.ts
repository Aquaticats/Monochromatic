import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fixUntilStable,
  MAX_AUTOFIX_PASSES,
  normalizeForConvergence,
  type OxlintRunResult,
} from '../dist/final/node/testing.mjs';

/**
 Builds a normalized run result; omitted optional fields stay absent.
 
 @param overrides - run fields to set; `stdout`/`stderr` default to empty,
   `exitCode`/`signalName`/`executionError` are omitted when not given
 
 @returns full {@link OxlintRunResult}
 
 @example
 ```ts
 makeResult({ exitCode: 0 });
 ```
 */
function makeResult(
  {
    stdout = '',
    stderr = '',
    exitCode,
    signalName,
    executionError,
  }: {
    readonly stdout?: string;
    readonly stderr?: string;
    readonly exitCode?: number;
    readonly signalName?: string;
    readonly executionError?: string;
  },
): OxlintRunResult {
  return {
    stdout,
    stderr,
    ...((exitCode === undefined) ? {} : { exitCode, }),
    ...((signalName === undefined) ? {} : { signalName, }),
    ...((executionError === undefined) ? {} : { executionError, }),
  };
}

/**
 Renders a diagnostics body plus a timing footer with a given duration.
 
 The duration is the only volatile part of real oxlint output, so two calls
 differing only in `ms` must normalize equal.
 
 @param body - stable diagnostics-and-summary text
 
 @param ms - timing footer duration in milliseconds
 
 @returns combined stdout as oxlint would print it
 
 @example
 ```ts
 withTiming({ body: 'Found 0 warnings and 1 error.', ms: 5 });
 ```
 */
function withTiming(
  {
    body,
    ms,
  }: {
    readonly body: string;
    readonly ms: number;
  },
): string {
  return `${body}\nFinished in ${ms}ms on 1 file with 111 rules using 16 threads.`;
}

/**
 Wraps a queue of results into a runner the loop can drain one call at a time.
 
 The queue is mutated by `shift`, so a test asserts `queue.length` afterward to
 pin the exact call count; a call past the queue's end throws, proving the loop
 never over-runs.
 
 @param queue - results returned in order, one per call
 
 @returns runner suitable for {@link fixUntilStable}'s `runFix`/`runLint`
 
 @example
 ```ts
 const fixes = [makeResult({ exitCode: 0 })];
 await fixUntilStable({ runFix: makeScriptedRun(fixes), runLint: makeScriptedRun([clean]), maxPasses: 8 });
 ```
 */
function makeScriptedRun(
  queue: OxlintRunResult[],
): () => Promise<OxlintRunResult> {
  return function scriptedRun(): Promise<OxlintRunResult> {
    /**
     Next scripted result, or `undefined` once the queue is drained.
     */
    const next = queue.shift();
    if (next === undefined)
      throw new Error('scripted run queue exhausted: fixUntilStable called a runner more times than expected',);
    return Promise.resolve(next,);
  };
}

/** Stable diagnostics body used to exercise the timing-line normalization. */
const oneErrorBody = [
  '',
  '  x stylistic(chain-per-line): Put each step in this chain on its own line.',
  '    ,-[file.ts:1:11]',
  '  1 | const r = a + b * c.d.e;',
  '    :           ^^^^^^^^^^^^^',
  '    `----',
  '',
  'Found 0 warnings and 1 error.',
].join('\n',);

/**
 Builds a dirty oracle output carrying a real diagnostic header.
 
 The loop's clean stop keys off {@link hasDiagnostics}, which scans for an
 `x rule(...)` block opener, so a stand-in for "violations remain" must include
 one; a bare `Found N errors.` summary line is not a diagnostic header.
 
 @param label - distinguishes otherwise-identical bodies across passes
 
 @returns oracle stdout with one diagnostic block and a summary
 
 @example
 ```ts
 dirtyOracle('A');
 ```
 */
function dirtyOracle(label: string,): string {
  return [
    '',
    `  x stylistic(chain-per-line): remaining boundary ${label}.`,
    '    ,-[file.ts:1:11]',
    '  1 | const r = a + b * c.d.e;',
    '    :           ^^^^^^^^^^^^^',
    '    `----',
    '',
    'Found 0 warnings and 1 error.',
  ].join('\n',);
}

/** Dirty oracle output, variant A. */
const oracleA = dirtyOracle('A',);
/** Dirty oracle output, variant B. */
const oracleB = dirtyOracle('B',);
/** Dirty oracle output, variant C. */
const oracleC = dirtyOracle('C',);
/** Clean oracle output: summary only, no diagnostic header. */
const oracleClean = 'Found 0 warnings and 0 errors.';

/**
 Builds a single diagnostic block (header plus source frame) for a label.
 
 @param label - distinguishes block content and target path
 
 @returns one diagnostic block, no summary or footer
 
 @example
 ```ts
 blockFor('A');
 ```
 */
function blockFor(label: string,): string {
  return [
    `  x stylistic(chain-per-line): boundary ${label}.`,
    `    ,-[${label}.ts:1:1]`,
    '  1 | code',
    '    `----',
  ].join('\n',);
}

/**
 Assembles two diagnostic blocks, a summary, and a timing footer.
 
 Models oxlint's multi-file output, whose block order and footer duration both
 vary run to run.
 
 @param first - first diagnostic block
 
 @param second - second diagnostic block
 
 @param ms - timing footer duration
 
 @returns combined stdout for two diagnostics
 
 @example
 ```ts
 twoBlockOutput({ first: blockFor('A'), second: blockFor('B'), ms: 5 });
 ```
 */
function twoBlockOutput(
  {
    first,
    second,
    ms,
  }: {
    readonly first: string;
    readonly second: string;
    readonly ms: number;
  },
): string {
  return [
    '',
    first,
    '',
    second,
    '',
    'Found 0 warnings and 2 errors.',
    `Finished in ${ms}ms on 2 files with 111 rules using 16 threads.`,
  ].join('\n',);
}

await describe({
  name: '',
  children: [
    describe({
      name: normalizeForConvergence.name,
      children: [
        it({
          name: 'drops the volatile Finished-in timing footer',
          fn: async () => {
            /** One run's stdout normalized; the timing footer must be gone. */
            const normalized = normalizeForConvergence(withTiming({ body: oneErrorBody, ms: 247, },),);
            expect(normalized.includes('Finished in',),)
              .toBe(false,);
          },
        },),
        it({
          name: 'is invariant to the timing footer duration',
          fn: async () => {
            /** Fast run's normalized stdout. */
            const fast = normalizeForConvergence(withTiming({ body: oneErrorBody, ms: 100, },),);
            /** Slow run's normalized stdout; identical body, slower timing footer. */
            const slow = normalizeForConvergence(withTiming({ body: oneErrorBody, ms: 999, },),);
            expect(fast,)
              .toBe(slow,);
          },
        },),
        it({
          name: 'is invariant to diagnostic block order',
          fn: async () => {
            // The repo-scale failure: a multi-file run emits the same blocks in
            // non-deterministic order, so order must not affect the canonical form.
            /** Blocks emitted A-then-B. */
            const forward = normalizeForConvergence(
              twoBlockOutput({ first: blockFor('A',), second: blockFor('B',), ms: 5, },),
            );
            /** Same blocks emitted B-then-A with a different footer. */
            const reversed = normalizeForConvergence(
              twoBlockOutput({ first: blockFor('B',), second: blockFor('A',), ms: 9, },),
            );
            expect(forward,)
              .toBe(reversed,);
          },
        },),
        it({
          name: 'distinguishes a changed diagnostic set',
          fn: async () => {
            /** Output with blocks A and B. */
            const ab = normalizeForConvergence(
              twoBlockOutput({ first: blockFor('A',), second: blockFor('B',), ms: 5, },),
            );
            /** Output with blocks A and C; C differs, so the canonical form must differ. */
            const ac = normalizeForConvergence(
              twoBlockOutput({ first: blockFor('A',), second: blockFor('C',), ms: 5, },),
            );
            expect(ab === ac,)
              .toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: fixUntilStable.name,
      children: [
        it({
          name: 'stops after one pass when the oracle is clean',
          fn: async () => {
            const fixes = [
              makeResult({ exitCode: 0, },),
              makeResult({ exitCode: 0, },),
            ];
            const lints = [
              makeResult({ stdout: oracleClean, exitCode: 0, },),
              makeResult({ stdout: oracleClean, exitCode: 0, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(1,);
            expect(outcome.stopReason,)
              .toBe('clean',);
            expect(fixes.length,)
              .toBe(1,);
            expect(lints.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'keeps looping while the fix pass exits zero but the oracle stays dirty',
          fn: async () => {
            // The exact shape that fooled an exit-code-only stop condition:
            // every --fix pass exits 0, yet each still rewrites the file, so the
            // oracle reports a (different) remaining violation until pass 3.
            const fixes = [
              makeResult({ exitCode: 0, },),
              makeResult({ exitCode: 0, },),
              makeResult({ exitCode: 0, },),
            ];
            const lints = [
              makeResult({ stdout: oracleA, exitCode: 1, },),
              makeResult({ stdout: oracleB, exitCode: 1, },),
              makeResult({ stdout: oracleClean, exitCode: 0, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(3,);
            expect(outcome.stopReason,)
              .toBe('clean',);
            expect(fixes.length,)
              .toBe(0,);
            expect(lints.length,)
              .toBe(0,);
          },
        },),
        it({
          name: 'loops past a deferred-fix pass until the oracle is clean',
          fn: async () => {
            const fixes = [
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 0, },),
            ];
            const lints = [
              makeResult({ stdout: oracleA, exitCode: 1, },),
              makeResult({ stdout: oracleClean, exitCode: 0, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(2,);
            expect(outcome.stopReason,)
              .toBe('clean',);
          },
        },),
        it({
          name: 'stops when an unfixable remainder makes the oracle stable',
          fn: async () => {
            const fixes = [
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
            ];
            const lints = [
              makeResult({ stdout: oracleA, exitCode: 1, },),
              makeResult({ stdout: oracleB, exitCode: 1, },),
              makeResult({ stdout: oracleB, exitCode: 1, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(3,);
            expect(outcome.stopReason,)
              .toBe('stable',);
          },
        },),
        it({
          name: 'treats a timing-only oracle difference as a fixpoint',
          fn: async () => {
            const fixes = [
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
            ];
            const lints = [
              makeResult({ stdout: withTiming({ body: oneErrorBody, ms: 188, },), exitCode: 1, },),
              makeResult({ stdout: withTiming({ body: oneErrorBody, ms: 251, },), exitCode: 1, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(2,);
            expect(outcome.stopReason,)
              .toBe('stable',);
          },
        },),
        it({
          name: 'treats a reordered oracle as stable (repo-scale ordering noise)',
          fn: async () => {
            const fixes = [
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
            ];
            const lints = [
              makeResult({
                stdout: twoBlockOutput({ first: blockFor('A',), second: blockFor('B',), ms: 5, },),
                exitCode: 1,
              },),
              makeResult({
                stdout: twoBlockOutput({ first: blockFor('B',), second: blockFor('A',), ms: 9, },),
                exitCode: 1,
              },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(2,);
            expect(outcome.stopReason,)
              .toBe('stable',);
          },
        },),
        it({
          name: 'stops at the cap when the oracle keeps changing',
          fn: async () => {
            const fixes = [
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
            ];
            const lints = [
              makeResult({ stdout: oracleA, exitCode: 1, },),
              makeResult({ stdout: oracleB, exitCode: 1, },),
              makeResult({ stdout: oracleC, exitCode: 1, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: 3,
            },);
            expect(outcome.passes,)
              .toBe(3,);
            expect(outcome.stopReason,)
              .toBe('cap',);
            expect(fixes.length,)
              .toBe(0,);
            expect(lints.length,)
              .toBe(0,);
          },
        },),
        it({
          name: 'detects a two-state oscillation and stops with cycle',
          fn: async () => {
            // The real-repo failure: --fix flips a file A -> B -> A -> ...,
            // so the oracle never matches the immediately-previous pass but
            // revisits an earlier state. Detect that instead of running to cap.
            const fixes = [
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
              makeResult({ exitCode: 1, },),
            ];
            const lints = [
              makeResult({ stdout: oracleA, exitCode: 1, },),
              makeResult({ stdout: oracleB, exitCode: 1, },),
              makeResult({ stdout: oracleA, exitCode: 1, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(3,);
            expect(outcome.stopReason,)
              .toBe('cycle',);
          },
        },),
        it({
          name: 'returns immediately when the fix pass cannot execute oxlint',
          fn: async () => {
            const fixes = [
              makeResult({ executionError: 'spawn oxlint ENOENT', },),
              makeResult({ exitCode: 0, },),
            ];
            const lints = [
              makeResult({ stdout: oracleClean, exitCode: 0, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(1,);
            expect(outcome.stopReason,)
              .toBe('execution-error',);
            // oracle never runs when the fix pass already failed
            expect(lints.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'returns immediately when the oracle lint cannot execute oxlint',
          fn: async () => {
            const fixes = [
              makeResult({ exitCode: 1, },),
            ];
            const lints = [
              makeResult({ executionError: 'spawn oxlint ENOENT', },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(1,);
            expect(outcome.stopReason,)
              .toBe('execution-error',);
          },
        },),
        it({
          name: 'returns immediately on signal termination of a fix pass',
          fn: async () => {
            const fixes = [
              makeResult({ signalName: 'SIGTERM', },),
            ];
            const lints = [
              makeResult({ stdout: oracleClean, exitCode: 0, },),
            ];
            const outcome = await fixUntilStable({
              runFix: makeScriptedRun(fixes,),
              runLint: makeScriptedRun(lints,),
              maxPasses: MAX_AUTOFIX_PASSES,
            },);
            expect(outcome.passes,)
              .toBe(1,);
            expect(outcome.stopReason,)
              .toBe('execution-error',);
            expect(lints.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'rejects a maxPasses below one',
          fn: async () => {
            await expect(
              fixUntilStable({
                runFix: makeScriptedRun([],),
                runLint: makeScriptedRun([],),
                maxPasses: 0,
              },),
            )
              .rejects
              .toThrow(RangeError,);
          },
        },),
      ],
    },),
  ],
},);
