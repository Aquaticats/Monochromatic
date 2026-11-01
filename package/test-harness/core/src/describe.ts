import {
  logger as defaultLogger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import pLimit from 'p-limit';

import { withTimeout, } from '@monochromatic-dev/async-time-core/ts';
import { formatDuration, } from '@monochromatic-dev/numeric-format-core/ts';
import {
  DEFAULT_CONCURRENCY,
  type DescriptorContext,
  makeDescriptor,
  RUN_WITH_CONTEXT,
  type TestDescriptor,
} from './descriptor.ts';
import { formatFailure, } from '@monochromatic-dev/test-harness-core-diagnostic/ts';
import type { ItResult, } from './it.ts';
import { createVerdictLoggers, } from './verdict.ts';
import { runObservedExecution, } from './execution.ts';

/**
 Result returned by a completed suite, mirroring {@link ItResult}.
 */
export type DescribeResult = {
  /**
   Suite name, returned so parent suites can log the hierarchy.
   */
  readonly name: string;
};

/**
 Single child entry: a lazy {@link TestDescriptor} from nested
 {@link describe} or `it` calls. Execution is dispatched by the parent
 suite, which calls each child's {@link RUN_WITH_CONTEXT} with the
 inherited execution context.
 */
export type DescribeChild = TestDescriptor<DescribeResult | ItResult>;

/**
 Options for a test suite.
 */
export type DescribeOptions = {
  /**
   Child entries from nested {@link describe} or `it` calls.
   Each child resolves with its name on success so the parent
   can log the `child <- parent` relationship.
   
   Children are lazy descriptors and do not execute until the
   parent dispatches them.
   */
  readonly children: readonly DescribeChild[];
  /**
   Maximum number of children running at the same time.
   
   - `1`: sequential execution via `for...of` loop, no `p-limit` overhead
   - `2`..`Number.MAX_SAFE_INTEGER - 1`: bounded concurrency via `p-limit`
   - `Infinity` or `Number.MAX_SAFE_INTEGER`: unbounded concurrency via
     raw `Promise.allSettled` (no `p-limit` overhead)
   
   **Inherited by child describes.** A nested `describe` without its own
   `concurrency` inherits the parent's effective value, so setting
   `concurrency: 1` once at the top sequences all descendants.
   
   Defaults to {@link DEFAULT_CONCURRENCY} (16) at the root.
   */
  readonly concurrency?: number;
  /**
   Logger override. When omitted, a tagged logger derived from
   module-logger's default logger is used.
   */
  readonly l?: Logger;
  /**
   Suite name shown in output and error cause chain.
   Set to empty string to make this level invisible in output;
   the suite still groups and times its children, but adds
   no name segment to the path. Success logs are downgraded
   from `info` to `debug` for empty-name suites.
   Use empty name at the top level since the filename
   already identifies what is being tested.
   */
  readonly name: string;
  /**
   Number of additional times to re-run the entire suite after the first execution.
   Useful for catching flaky suites. `repeats: 2` runs the suite 3 times total.
   Defaults to `0`.
   */
  readonly repeats?: number;
  /**
   Whether to skip execution entirely. When `true` or a reason string,
   the suite logs `[SKIP]` and returns immediately without running children.
   Defaults to `false`.
   */
  readonly skip?: boolean | string;
  /**
   Timeout in milliseconds for the entire suite (all children).
   Powered by `Promise.race`. Children with their own timeout
   should use a smaller value than this.
   */
  readonly timeout?: number;
};

/**
 Executes a suite given options and an inherited context.
 Internal: the public {@link describe} entry point wraps this in
 {@link makeDescriptor} so callers receive a lazy descriptor.
 
 @param opts - suite options
 
 @param ctx - inherited execution context (effective concurrency from parent)
 
 @returns suite result containing the suite name
 
 On failure, emits the wrapped cause inline at `error` level
 (`formatErrorDeep` walks `.cause` and `AggregateError.errors`)
 adjacent to the `[FAIL]` summary, so the log stream alone is
 sufficient for diagnosis. The throw shape is unchanged.
 
 @throws Error wrapping child failures when any child rejects.
   Single failure: `Error(name, { cause: childError })`.
   Multiple failures: `Error(name, { cause: AggregateError([...]) })`.
   Empty name: re-throws the cause directly without wrapping.
   Timeout: re-throws the raw timeout error (no name-wrapping),
   matching the pre-change behavior.
 */
async function runDescribe(
  {
    opts,
    ctx,
  }: {
    readonly ctx: DescriptorContext;
    readonly opts: DescribeOptions;
  },
): Promise<DescribeResult> {
  /**
   Pulls option fields out with their defaults so the body can refer to them without re-reading `opts`.
   */
  const {
    name,
    children,
    concurrency,
    skip = false,
    repeats = 0,
    timeout,
    l: loggerOverride,
  } = opts;
  /**
   Parent logger resolved from the override, the inherited parent, or the default module logger.
   */
  const baseLogger = loggerOverride ?? ctx
    .parentLogger
    ?? defaultLogger;
  /**
   Composed tagged logger for this suite; empty-name suites stay invisible by reusing the parent untagged.
   */
  const l = name === ''
    ? baseLogger
    : tagged({
      tag: name,
      l: baseLogger,
    },);
  /**
   Outcome-tagged loggers composed after this suite's hierarchy tag.
   */
  const {
    fail: failLogger,
    pass: passLogger,
    skip: skipLogger,
  } = createVerdictLoggers({ l, },);

  if (skip !== false) {
    /**
     Optional reason carried in message body while verdict remains in tag.
     */
    const reason = (typeof skip) === 'string' ? `: ${skip}` : '';
    skipLogger.info(`suite${name ? ` "${name}"` : ''}${reason}`,);
    return { name, };
  }

  /**
   Effective concurrency for this suite, inheriting parent when not set.
   */
  const effectiveConcurrency = concurrency ?? ctx
    .effectiveConcurrency;
  /**
   Context passed to each child for inheritance.
   `parentLogger` is this suite's composed tagged logger, so every
   descendant's log line carries the full `[outer] [inner] [test]` tag
   chain and the hierarchy is visible per-line without a separate
   `child <- parent` enumeration.
   */
  const childCtx: DescriptorContext = {
    effectiveConcurrency,
    parentLogger: l,
  };
  /**
   Whether concurrency is effectively unbounded.
   */
  const isUnbounded = effectiveConcurrency >= Number
    .MAX_SAFE_INTEGER;
  /**
   Whether children run one at a time.
   */
  const isSequential = effectiveConcurrency <= 1;

  if (name !== '') {
    /**
     Inline annotation describing this suite's dispatch mode for the `start` debug line.
     */
    const concurrencyLabel = isSequential
      ? ' (sequential)'
      : (isUnbounded
        ? ' (unbounded)'
        : ` (concurrency: ${String(effectiveConcurrency,)})`);
    l.debug(`start${concurrencyLabel}`,);
  }

  /**
   Runs all children sequentially via `for...of`, collecting results in order.
   Each child starts only after the previous one settles.
   
   @returns array of settled results matching `Promise.allSettled` format
   */
  async function runSequential(): Promise<
    PromiseSettledResult<DescribeResult | ItResult>[]
    > {
    /**
     Accumulator matched to `Promise.allSettled` shape so the call site can branch identically across modes.
     */
    const results: PromiseSettledResult<DescribeResult | ItResult>[] = [];

    for (const child of children) {
      try {
        /* oxlint-disable no-await-in-loop -- sequential execution requires awaiting each child before starting the next */
        /**
         Resolved child result captured so it can be wrapped in the fulfilled settled-result shape.
         */
        const value = await child[RUN_WITH_CONTEXT](childCtx,);
        /* oxlint-enable no-await-in-loop */
        results.push({
          status: 'fulfilled',
          value,
        },);
      }
      catch (reason) {
        results.push({
          status: 'rejected',
          reason,
        } as PromiseRejectedResult,);
      }
    }

    return results;
  }

  /**
   Runs one pass of the suite: starts all children, collects results, reports.
   Children emit their own `[PASS]` (at `debug`) and `[FAIL]` (at `error`)
   records during execution, each carrying the full `[outer] [inner] [child]`
   hierarchy via inherited `childCtx.parentLogger`. After all children
   settle, this function emits suite-level enumeration:
   
   - all-success: one `info` record `[PASS] childA, childB, ... (Nms)` listing
     fulfilled children plus suite wall-clock duration
   - mixed-result: one `info` record `[PASS] childA, ...` listing passing
     siblings, followed by `error` record `[FAIL] (Nms)` rollup
   - all-failure: only `error` record `[FAIL] (Nms)` rollup
   - empty suite: one `info` record `[PASS] (Nms)`
   
   On failure, `[FAIL]` rollup is emitted in one outcome-tagged error call
   combined with formatted cause chain (`formatErrorDeep`), so hierarchy
   prefix lands only on summary and continuation lines are untagged.
   Timeout failures emit `[FAIL] timeout (Nms)` with timeout error inline.
   
   Empty-name suites downgrade the info line to `debug` so they stay out of
   default output; the suite still groups and times its children.
   
   @param runLabel - label suffix for repeated runs (empty string for single runs)
   
   @throws Error wrapping child failures when any child rejects
   */
  async function runOnce(runLabel: string,): Promise<void> {
    /**
     Optional repeat label normalized as message prefix after verdict tag.
     */
    const runPrefix = runLabel === '' ? '' : `${runLabel.trim()} `;
    /**
     Selected child dispatcher; assigned once so the subsequent timeout wrapper and await both see the same promise.
     */
    const settleAll: Promise<PromiseSettledResult<DescribeResult | ItResult>[]> =
      isSequential
        ? runSequential()
        : (isUnbounded
          ? Promise.allSettled(children.map(function mapChild(child,) {
            return child[RUN_WITH_CONTEXT](childCtx,);
          },),)
          : (function runLimited(): Promise<
            PromiseSettledResult<DescribeResult | ItResult>[]
            > {
            /**
             p-limit instance owns the in-flight count for this suite so concurrent dispatch stays within the cap.
             */
            const limit = pLimit(effectiveConcurrency,);
            return Promise.allSettled(children.map(function limitChild(child,) {
              return limit(function dispatchChild() {
                return child[RUN_WITH_CONTEXT](childCtx,);
              },);
            },),);
          }()));

    /**
     Settle promise optionally wrapped in `withTimeout` so the timeout failure surfaces through the same await.
     */
    const withTimeoutApplied = timeout !== undefined
      ? withTimeout({
        promise: settleAll,
        ms: timeout,
        label: name || '(root)',
      },)
      : settleAll;

    /**
     Wall-clock start used for the suite-level duration report.
     */
    const startTime = performance.now();
    /**
     Awaits the settle promise while surfacing timeout failures inline.
     Extracted so the binding stays `const`; the catch always re-throws,
     so the resolved array is the only path that reaches assignment.
     
     Timeout bypasses every child-result processing line, so the
     failure has no inline diagnostic surface on its own. Emit one
     `[FAIL]` record carrying summary fused with the formatted
     timeout error before re-throwing; the throw shape is preserved
     (raw timeout error, matching pre-change behavior).
     
     @returns settled results from `Promise.allSettled` or the sequential equivalent
     
     @throws original timeout error, after logging `[FAIL]` record
     */
    async function awaitSettleWithTimeoutLogging(): Promise<
      PromiseSettledResult<DescribeResult | ItResult>[]
      > {
      try {
        return await withTimeoutApplied;
      }
      catch (timeoutError) {
        /**
         Elapsed time at the moment the timeout fired, used in inline failure summary.
         */
        const elapsedMs = performance.now()
          - startTime;
        failLogger.error(await formatFailure({
          summary: `${runPrefix}timeout (${formatDuration(elapsedMs,)})`,
          value: timeoutError,
        },),);
        throw timeoutError;
      }
    }
    /**
     Settled child results awaited via timeout-logging helper so timeout always emits `[FAIL]` before throwing.
     */
    const settled = await awaitSettleWithTimeoutLogging();
    /**
     Elapsed time across the whole settle, used in the suite-level summary line.
     */
    const durationMs = performance.now()
      - startTime;

    /**
     Failure accumulator; rejected child reasons are pushed here for the rollup throw.
     */
    const errors: unknown[] = [];
    /**
     Pass-side accumulator collecting fulfilled child names for the visible info line.
     */
    const passedNames: string[] = [];
    /**
     Empty-name suites are invisible wrappers; downgrade success logs to debug.
     */
    const logPass = name === ''
      ? passLogger.debug
      : passLogger.info;

    for (const result of settled) {
      if (result.status
        === 'fulfilled')
        passedNames.push(result.value
          .name,);
      else
        errors.push(result.reason,);
    }

    /**
     Suite-level info line listing fulfilled children's names plus
     duration. This is the visible-by-default carrier for the
     parent-children mapping now that per-test `[PASS]` is at `debug`:
     a single info line per parent enumerates which children ran
     under it, in array order. Mixed-result suites still emit a
     names list (without duration) so passing siblings remain
     visible alongside error-level `[FAIL]` rollup.
     */
    if (passedNames.length
      > 0) {
      if (errors.length
        === 0) {
        logPass(
          `${passedNames.join(', ',)} ${runPrefix}(${
            formatDuration(durationMs,)
          })`,
        );
      }
      else {
        logPass(passedNames.join(', ',),);
      }
    }

    if (errors.length
      === 0) {
      if (passedNames.length
        === 0)
        logPass(`${runPrefix}(${formatDuration(durationMs,)})`,);
      return;
    }

    /**
     Aggregated cause: single child failure passes through; multiple failures fold into an `AggregateError`.
     */
    const cause = errors.length
      === 1
      ? errors[0]
      : new AggregateError(
        errors,
        `${String(errors.length,)} children failed in suite "${name || '(root)'}"`,
      );

    failLogger.error(await formatFailure({
      summary: `${runPrefix}(${formatDuration(durationMs,)})`,
      value: cause,
    },),);

    if (name === '')
      throw cause;

    throw new Error(
      name,
      { cause, },
    );
  }

  /**
   Total iteration count: one base run plus any explicit repeats.
   */
  const totalRuns = 1 + repeats;

  for (let run = 0; run < totalRuns; run += 1) {
    /**
     Per-iteration label inserted into the suite-level summary so repeat runs can be told apart.
     */
    const runLabel = totalRuns > 1
      ? ` [run ${String(run + 1,)}/${String(totalRuns,)}]`
      : '';
    // oxlint-disable-next-line no-await-in-loop -- sequential suite repetitions must run one at a time
    await runOnce(runLabel,);
  }

  return { name, };
}

/**
 Defines a test suite as a lazy {@link TestDescriptor}.
 Construction is synchronous and side-effect free; execution begins
 when the descriptor is awaited (top-level) or dispatched by a parent
 suite via {@link RUN_WITH_CONTEXT}.
 
 Children run concurrently via `Promise.allSettled` by default,
 capped at `concurrency` (default 16) simultaneous children via `p-limit`.
 Set `concurrency: 1` for sequential execution; nested describes inherit
 the parent's effective value unless they override it.
 
 On failure, the suite emits the full cause chain inline at `error`
 level adjacent to the `[FAIL]` summary, so the log stream alone is
 sufficient for diagnosis. Top-level `try { await describe(...) } catch`
 remains supported; the throw shape is unchanged.
 
 @param opts - suite options
 
 @returns lazy descriptor that resolves with the suite result
 
 @throws Error with child errors as cause when any child fails.
   Single failure: `Error(name, { cause: childError })`.
   Multiple failures: `Error(name, { cause: AggregateError([...]) })`.
   Empty name: re-throws the cause directly without wrapping.
   Timeout: re-throws the raw timeout error.
 
 @example
 ```ts
 await describe({
   name: 'math',
   children: [
     it({ name: 'adds', fn: async () => expect(1 + 1).toBe(2) }),
     it({ name: 'subtracts', fn: async () => expect(2 - 1).toBe(1) }),
   ],
   timeout: 5000,
 });
 ```
 */
export function describe(opts: DescribeOptions,): TestDescriptor<DescribeResult> {
  return makeDescriptor(function runDescribeWithCtx(ctx,) {
    /**
     Explicit or scheduler-provided parent for rejection diagnostics.
     */
    const logger = opts.l ?? ctx.parentLogger;
    return runObservedExecution({
      kind: 'suite',
      name: opts.name,
      ...logger === undefined ? {} : { logger, },
      run: function runObservedSuite(): Promise<DescribeResult> {
        return runDescribe({
          opts,
          ctx,
        },);
      },
    },);
  },);
}
