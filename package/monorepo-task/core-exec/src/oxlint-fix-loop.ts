/**
 Iterate `oxlint --fix` until the codebase reaches a fixpoint.
 
 oxlint applies at most one fix per overlapping byte region per pass and does
 not re-lint within a single invocation, so two rules whose fixes touch
 overlapping spans (or a fix that introduces a fresh fixable violation) need
 multiple `--fix` runs to fully converge. ESLint loops up to ten times for the
 same reason; oxlint tracks the gap at oxc#16118 but has not shipped
 fix-iteration. Full analysis:
 {@link file://../../../../doc/troubleshooting/oxlint-multi-fix-convergence.md}.
 
 Detecting convergence is the hard part. `oxlint --fix` reports neither through
 its exit code nor its stdout whether a pass actually changed any file: a pass
 that applies fixes and leaves nothing unfixable exits zero with
 `Found 0 ... errors.`, identical to a genuine no-op pass, even when its fixes
 introduced a fresh fixable violation that the next pass will rewrite. oxlint's
 JSON output carries no fix-applied count either. The only signal that tracks
 the file is a plain (no-fix) lint of the post-fix state: it reports every
 remaining violation, fixable or not, so its output moves while the file moves
 and settles exactly when the file stops changing.
 
 {@link fixUntilStable} therefore runs two oxlint invocations per pass: one
 `--fix` to apply fixes, then one plain lint as the convergence oracle. That
 doubles oxlint work versus a single pass, which is the cost of correctness
 given oxlint exposes no fix-applied signal; the documented baseline already
 accepts running `--fix` twice. Both runners are injected so the loop is
 unit-testable without spawning oxlint.
 
 @module
 */

import {
  extractRuleName,
  NO_RULE,
  stripAnsi,
} from './oxlint-augment.ts';

//region Types and constants

/**
 Maximum `oxlint --fix` passes before giving up on convergence.
 
 Matches the cap the stylistic convergence test already uses
 (`oxlint-stylistic.unit.test.ts`). ESLint's analogous limit is ten; eight is
 generous for the overlapping-fix and fix-induced-violation chains this
 workspace produces and bounds worst-case work if two rules' fixes oscillate.
 
 @internal
 */
export const MAX_AUTOFIX_PASSES = 8;

/**
 Normalized result of one `oxlint` invocation.
 
 Collapses nano-spawn's success path and its thrown `SubprocessError` shape
 into one object so the loop reads both uniformly. `exitCode` is absent on
 signal termination; `executionError` is set only when oxlint could not be
 spawned at all (e.g. binary missing), never for ordinary diagnostics exits.
 
 @internal
 */
export type OxlintRunResult = {
  /**
   Captured standard output (diagnostics plus the `Found N ...` summary).
   */
  readonly stdout: string;
  /**
   Captured standard error.
   */
  readonly stderr: string;
  /**
   Process exit code; absent when terminated by a signal.
   */
  readonly exitCode?: number;
  /**
   Terminating signal name when the process was killed; absent otherwise.
   */
  readonly signalName?: string;
  /**
   Message set only when oxlint itself could not be executed; absent for
   normal runs including non-zero diagnostics exits.
   */
  readonly executionError?: string;
};

/**
 Why {@link fixUntilStable} stopped looping.
 
 - `clean`: the oracle lint reported zero diagnostics, so no violation remains
   and no further `--fix` could change the file.
 - `stable`: the oracle's normalized output stopped changing while diagnostics
   still remain (unfixable remainder; the file has reached a fixpoint).
 - `cycle`: the oracle returned to an earlier (non-immediate) state, so two or
   more autofixes are flipping a file back and forth and it will never settle.
 - `execution-error`: an oxlint run failed to execute or was signal-terminated.
 - `cap`: {@link MAX_AUTOFIX_PASSES} reached while the oracle kept changing
   without repeating a prior state (an unusually deep but progressing fix chain).
 */
export type FixLoopStopReason =
  | 'clean'
    | 'stable'
    | 'cycle'
    | 'execution-error'
    | 'cap';

/**
 Outcome of the fix loop: the final oracle run plus how and when it stopped.
 */
export type FixLoopOutcome = {
  /**
   Final oracle (no-fix lint) result, forwarded to the wrapper's output stage
   so the user sees the true remaining diagnostics after all fixes; on an
   execution failure this is instead the failed run.
   */
  readonly result: OxlintRunResult;
  /**
   Number of `--fix` passes performed (each pass also runs one oracle lint).
   */
  readonly passes: number;
  /**
   Terminating condition; `cap` signals non-convergence worth warning about.
   */
  readonly stopReason: FixLoopStopReason;
};

//endregion Types and constants

//region Convergence

/**
 Literal prefix of oxlint's per-run timing footer.
 */
const TIMING_PREFIX = 'Finished in ';

/**
 Canonicalizes oxlint stdout into an order- and timing-invariant form.
 
 Two sources of run-to-run noise would otherwise defeat the convergence
 comparison even when the linted files are unchanged:
 
 - The trailing `Finished in 247ms on 1 file with 111 rules using 16 threads.`
   footer; its duration differs every run.
 - Diagnostic block order. Across a multi-file, multi-threaded run oxlint emits
   the same set of blocks in a non-deterministic order (verified: two identical
   no-fix lints of a 2531-file tree produced the same 41419 lines shuffled).
 
 So the timing line is dropped, the remaining output is split into
 blank-line-separated diagnostic blocks (oxlint never puts a blank line inside
 a block; a shown blank source line keeps its ` N | ` prefix), and the blocks
 are sorted. The result changes only when the set of diagnostics changes, which
 is exactly the fixpoint signal. ANSI is stripped defensively via {@link stripAnsi}
 even though piped oxlint emits none.
 
 @param stdout - raw captured stdout from one `oxlint` run
 
 @returns timing-free, block-sorted canonical form for equality comparison
 
 @example
 ```ts
 normalizeForConvergence(runA.stdout) === normalizeForConvergence(runB.stdout);
 // true when A and B report the same diagnostics in any order
 ```
 
 @internal
 */
export function normalizeForConvergence(stdout: string,): string {
  return stdout
    .split('\n',)
    .filter(function keepNonTimingLine(line,): boolean {
      return !stripAnsi(line,)
        .trimStart()
        .startsWith(TIMING_PREFIX,);
    },)
    .join('\n',)
    .split('\n\n',)
    .map(function trimBlock(block,): string {
      return block.trim();
    },)
    .filter(function isNonEmptyBlock(block,): boolean {
      return block.length > 0;
    },)
    .toSorted(function compareBlocks(
      first,
      second,
    ): number {
      if (first < second)
        return -1;
      if (first > second)
        return 1;
      return 0;
    },)
    .join('\n\n',);
}

/**
 Whether a run failed to execute rather than merely reporting diagnostics.
 
 @param result - one normalized run result
 
 @returns whether oxlint could not run or was signal-terminated
 */
function isExecutionFailure(result: OxlintRunResult,): boolean {
  return (result.executionError !== undefined)
    || ((result.signalName !== undefined) && (result.signalName !== ''));
}

/**
 Whether an oracle lint reported any diagnostic at all.
 
 A file with zero diagnostics is a guaranteed fixpoint: `--fix` has nothing to
 change. This drives the early `clean` stop and is sound across severities,
 unlike oxlint's exit code, which is zero whenever no errors remain even if a
 fixable warning is still pending. Reuses {@link extractRuleName} so the same
 `x rule(...)` or `! rule(...)` parsing used by output augmentation decides what
 counts as a diagnostic block opener; summary, context, and blank lines do not.
 
 @param stdout - raw oracle stdout
 
 @returns whether at least one diagnostic header is present
 */
function hasDiagnostics(stdout: string,): boolean {
  return stdout
    .split('\n',)
    .some(function isDiagnosticHeader(line,): boolean {
      return extractRuleName(line,) !== NO_RULE;
    },);
}

//endregion Convergence

//region Loop

/**
 Options for {@link fixUntilStable}.
 */
export type FixUntilStableOptions = {
  /**
   Runs `oxlint --fix` once with identical arguments, mutating files in place,
   and returns its normalized result.
   */
  readonly runFix: () => Promise<OxlintRunResult>;
  /**
   Runs a plain `oxlint` lint (no fix flags) over the same target, used as the
   convergence oracle: its output reflects the current on-disk file state.
   */
  readonly runLint: () => Promise<OxlintRunResult>;
  /**
   Hard cap on `--fix` passes; the loop returns the last oracle once reached.
   */
  readonly maxPasses: number;
};

/**
 Runs `oxlint --fix` repeatedly until its effect on the codebase stabilizes.
 
 Each pass applies fixes ({@link FixUntilStableOptions.runFix}) and then
 re-lints the result without fixing ({@link FixUntilStableOptions.runLint}) to
 read the true file state. The loop stops as soon as that oracle lint reports
 zero diagnostics per {@link hasDiagnostics} (nothing left for any fix to change),
 or its {@link normalizeForConvergence}-normalized output matches the previous
 pass's oracle (a fixpoint with unfixable diagnostics remaining), or it matches
 a non-adjacent earlier pass (two fixes oscillating a file back and forth, which
 would never settle), or an execution failure occurs, or the cap is hit.
 The returned final oracle result flows through the wrapper's augmentation and
 exit-preserving output stage; the `--fix` passes exist only to apply fixes.
 
 The oracle is required because `oxlint --fix` reveals file changes through
 neither its exit code nor its stdout: a pass can exit zero with unchanged
 output yet still rewrite the file (e.g. a fix that introduces a fresh fixable
 violation). A plain lint reports that violation as a diagnostic, so the oracle
 moves with the file.
 
 @param runFix - applies `oxlint --fix` once (same arguments every call)
 
 @param runLint - plain oracle lint over the same target
 
 @param maxPasses - hard cap on `--fix` passes (at least one)
 
 @returns final oracle result plus pass count and stop reason
 
 @throws RangeError when `maxPasses` is below one, so the loop cannot run
 
 @example
 ```ts
 const outcome = await fixUntilStable({
   runFix: () => runOxlint(['--fix', '--format=default']),
   runLint: () => runOxlint(['--format=default']),
   maxPasses: MAX_AUTOFIX_PASSES,
 });
 ```
 
 @internal
 */
export async function fixUntilStable(
  {
    runFix,
    runLint,
    maxPasses,
  }: FixUntilStableOptions,
): Promise<FixLoopOutcome> {
  if (maxPasses < 1)
    throw new RangeError(`maxPasses must be at least 1, received ${maxPasses}`,);

  /**
   Every normalized oracle state seen so far; a repeat means a `--fix` cycle.
   
   Holds prior passes' states, not the immediately-previous one, so a
   non-adjacent repeat distinguishes an oscillation from a true fixpoint.
   */
  const seen = new Set<string>();

  /* oxlint-disable no-restricted-syntax/no-function-root-let -- multi-pass fix cursor must remember the prior pass's normalized oracle output across side-effecting --fix iterations */
  /**
   Previous pass's normalized oracle output; `''` before the first pass.
   
   `''` is a safe "no previous" sentinel: the stability comparison runs only
   after {@link hasDiagnostics} confirmed a diagnostic header survived
   normalization, so a real oracle reaching the comparison is never empty and
   never collides with the sentinel.
   */
  let previousOracle = '';
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  for (
    let pass = 0;
    pass < maxPasses;
    pass += 1
  ) {
    /* oxlint-disable eslint/no-await-in-loop -- each pass must mutate files on disk before the oracle re-lints them */
    /**
     This pass's `--fix` result; an execution failure short-circuits the loop.
     */
    const fixResult = await runFix();
    /* oxlint-enable eslint/no-await-in-loop */
    if (isExecutionFailure(fixResult,))
      return {
        result: fixResult,
        passes: pass + 1,
        stopReason: 'execution-error',
      };

    /* oxlint-disable eslint/no-await-in-loop -- oracle must observe this pass's on-disk fixes before the next pass */
    /**
     Plain re-lint of the post-fix file; its diagnostics drive convergence.
     */
    const oracle = await runLint();
    /* oxlint-enable eslint/no-await-in-loop */
    if (isExecutionFailure(oracle,))
      return {
        result: oracle,
        passes: pass + 1,
        stopReason: 'execution-error',
      };

    if (!hasDiagnostics(oracle.stdout,))
      return {
        result: oracle,
        passes: pass + 1,
        stopReason: 'clean',
      };

    /**
     Oracle output with the volatile timing line removed; equality with the
     previous pass's oracle means the `--fix` between them changed nothing.
     */
    const normalized = normalizeForConvergence(oracle.stdout,);

    if (normalized === previousOracle)
      return {
        result: oracle,
        passes: pass + 1,
        stopReason: 'stable',
      };

    if (seen.has(normalized,))
      return {
        result: oracle,
        passes: pass + 1,
        stopReason: 'cycle',
      };

    if (pass === (maxPasses - 1))
      return {
        result: oracle,
        passes: pass + 1,
        stopReason: 'cap',
      };

    seen.add(normalized,);
    previousOracle = normalized;
  }

  throw new Error('fixUntilStable loop exited without returning; unreachable for maxPasses >= 1',);
}

//endregion Loop
