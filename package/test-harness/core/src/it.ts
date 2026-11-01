import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { formatDuration, } from '@monochromatic-dev/numeric-format-core/ts';
import {
  type DescriptorContext,
  makeDescriptor,
  type TestDescriptor,
} from './descriptor.ts';
import {
  createScopedExpect,
  type ScopedExpect,
} from '@monochromatic-dev/test-harness-core-expect/ts';
import { formatFailure, } from '@monochromatic-dev/test-harness-core-diagnostic/ts';
import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core-sandbox/ts';
import { runItAttempt, } from './it-attempt.ts';
import { createVerdictLoggers, } from './verdict.ts';
import { runObservedExecution, } from './execution.ts';

/**
 Context passed to each test function.
 Contains a scoped `expect` with assertion counting
 and a sinon sandbox that auto-restores after the test.
 */
export type TestContext = {
  /**
   Scoped expect with assertion tracking. Supports `expect.assertions(n)` and `expect.hasAssertions()`.
   */
  readonly expect: ScopedExpect;
  /**
   Sinon sandbox for stubs, spies, and fake timers. Auto-restores after the test completes.
   */
  readonly sinon: DisposableSandbox;
};

/**
 Options for a single test case.
 */
export type ItOptions = {
  /**
   Whether the test is expected to throw. When `true` or a reason string, a throwing test receives `[PASS]` and a passing test receives `[FAIL]`. Defaults to `false`.
   */
  readonly fails?: boolean | string;
  /**
   Async function that performs assertions and throws on failure.
   Receives a {@link TestContext} with a scoped `expect` for assertion counting.
   The global `expect` still works but does not support `expect.assertions(n)`.
   */
  readonly fn: (ctx: TestContext,) => Promise<void>;
  /**
   Logger to use for pass/fail output. Provided by the parent describe.
   */
  readonly l?: Logger;
  /**
   Human-readable test name, shown in output and error messages.
   */
  readonly name: string;
  /**
   Number of additional times to re-run the test after the first execution. Useful for catching flaky tests. Defaults to `0`.
   */
  readonly repeats?: number;
  /**
   Whether to skip execution entirely. When `true` or a reason string, the test logs `[SKIP]` and returns immediately. Defaults to `false`.
   */
  readonly skip?: boolean | string;
  /**
   Timeout in milliseconds. Must be less than any parent describe timeout.
   */
  readonly timeout?: number;
};

/**
 Result returned by a completed test case.
 */
export type ItResult = {
  /**
   Test name, returned so parent suites can log the hierarchy.
   */
  readonly name: string;
};

/**
 Executes a single test case. Internal: the public {@link it} entry
 point wraps this in {@link makeDescriptor} so callers receive a lazy
 descriptor.
 
 Logs `[PASS] (Nms)` at `debug` on success, so per-test output stays
 out of default verbosity. The parent `describe` surfaces the
 fulfilled child names in a single `info` line, preserving the
 parent-children mapping without one info line per test. `[FAIL]` at
 `error` is always visible; `[SKIP]` at `info`.
 On failure, also emits the caught error inline at `error` level
 (message, stack, `.cause` chain, `AggregateError.errors`) adjacent
 to the `[FAIL]` summary, so the log stream alone is sufficient for
 diagnosis without depending on the runtime's unhandled-rejection
 printer. The throw shape is unchanged.
 Throws `Error(name, { cause })` on failure or timeout, propagating
 the original error as the cause.
 
 @param opts - test options
 
 @param descriptorCtx - inherited execution context. Carries the
   parent suite's composed tagged logger, which this test wraps with
   its own name so the resulting tag chain reads root-first.
 
 @returns test result containing the test name
 
 @throws Error wrapping the original failure with the test name and cause chain
 */
async function runIt(
  {
    opts,
    descriptorCtx,
  }: {
    readonly descriptorCtx: DescriptorContext;
    readonly opts: ItOptions;
  },
): Promise<ItResult> {
  /**
   Pulls out individual fields with their defaults so the body can refer to them without re-reading the option object.
   */
  const {
    name,
    fn,
    timeout,
    skip = false,
    repeats = 0,
    fails = false,
    l: explicitLogger,
  } = opts;
  /**
   Parent logger comes from either an explicit `opts.l` (rare, used
   when callers build their own logger pipeline) or the parent suite's
   composed tagged logger threaded through `descriptorCtx.parentLogger`.
   Wrapping the parent with this test's name puts the test tag rightmost
   so verdict composition reads root-first: `[outer] [inner] [test-name] [PASS]`.
   */
  const baseLogger = explicitLogger ?? descriptorCtx
    .parentLogger;
  /**
   Hierarchy-tagged logger wrapped by outcome-specific loggers for verdict records.
   */
  const l = baseLogger !== undefined
    ? tagged({
      tag: name,
      l: baseLogger,
    },)
    : tagged({ tag: name, },);
  /**
   Outcome-tagged loggers composed after this test's hierarchy tag.
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
    const reason = (typeof skip) === 'string' ? skip : '(no reason)';
    skipLogger.info(reason,);
    return { name, };
  }

  /**
   Scoped expect plus its tracker so per-run assertion counts can be checked after each iteration.
   */
  const [scopedExpect, tracker,] = createScopedExpect();
  /**
   Total iteration count: one base run plus any explicit repeats.
   */
  const totalRuns = 1 + repeats;

  /**
   Spreads `timeout` into the attempt call only when set, so exactOptional never receives an explicit `undefined`.
   */
  const timeoutArg = timeout !== undefined ? { timeout, } : {};

  for (let run = 0; run < totalRuns; run += 1) {
    /**
     Per-iteration label inserted in log messages so repeat runs can be told apart.
     */
    const runLabel = totalRuns > 1
      ? ` [run ${String(run + 1,)}/${String(totalRuns,)}]`
      : '';
    /**
     Optional repeat label normalized as message prefix after verdict tag.
     */
    const runPrefix = runLabel === '' ? '' : `${runLabel.trim()} `;
    /**
     Tracks whether the test body threw so post-run logic can branch on outcome.
     */
    let threw = false;
    /**
     Captured throwable so failure formatting and rethrow can use the original cause.
     */
    let caughtError: unknown = undefined;
    /**
     Start timestamp for this iteration so duration can be reported in verdict output.
     */
    const runStart = performance.now();

    tracker.count = 0;

    try {
      // oxlint-disable-next-line no-await-in-loop -- sequential test repetitions must run one at a time
      await runItAttempt({
        fn,
        expect: scopedExpect,
        name,
        l,
        ...timeoutArg,
      },);
    }
    catch (error) {
      threw = true;
      caughtError = error;
    }

    /**
     Elapsed time for this iteration, formatted into the result log line.
     */
    const durationMs = performance.now()
      - runStart;

    /**
     Inline annotation appended to verdict body when `fails` was set as a string.
     */
    const failsReason = (typeof fails) === 'string' ? ` (${fails})` : '';

    if (fails !== false) {
      if (threw) {
        passLogger.debug(
          `${runPrefix}threw as expected${failsReason} (${
            formatDuration(durationMs,)
          })`,
        );
        continue;
      }

      /**
       Synthetic cause attached to the rethrow when a `fails`-marked test unexpectedly passes.
       */
      const failsCause = new Error('Expected test to throw but it passed',);
      // oxlint-disable-next-line no-await-in-loop -- formatFailure is async; await is required before the throw on the next line, and only one loop iteration runs on this path
      failLogger.error(await formatFailure({
        summary: `${runPrefix}expected to throw but passed${failsReason} (${
          formatDuration(durationMs,)
        })`,
        value: failsCause,
      },),);
      throw new Error(
        name,
        { cause: failsCause, },
      );
    }

    if (threw) {
      // oxlint-disable-next-line no-await-in-loop -- formatFailure is async; await is required before the throw on the next line, and only one loop iteration runs on this path
      failLogger.error(await formatFailure({
        summary: `${runPrefix}(${formatDuration(durationMs,)})`,
        value: caughtError,
      },),);
      throw new Error(
        name,
        { cause: caughtError, },
      );
    }

    //region Assertion count verification
    if ((tracker.expected
      !== undefined) && (tracker.count
        !== tracker
          .expected)) {
      /**
       Synthetic cause naming the assertion-count mismatch so the failure surface mirrors a regular throw.
       */
      const assertionCause = new Error(
        `Expected ${String(tracker.expected,)} assertions, but ${
          String(tracker.count,)
        } were called`,
      );
      // oxlint-disable-next-line no-await-in-loop -- formatFailure is async; await is required before the throw on the next line, and only one loop iteration runs on this path
      failLogger.error(await formatFailure({
        summary: `${runPrefix}expected ${String(tracker.expected,)} assertions but ${
          String(tracker.count,)
        } were called (${formatDuration(durationMs,)})`,
        value: assertionCause,
      },),);
      throw new Error(
        name,
        { cause: assertionCause, },
      );
    }

    if (tracker.requiresAtLeastOne
      && (tracker.count
        === 0)) {
      /**
       Synthetic cause used when `expect.hasAssertions()` was declared but no assertion ran.
       */
      const noAssertionsCause = new Error(
        'Expected at least one assertion to be called',
      );
      // oxlint-disable-next-line no-await-in-loop -- formatFailure is async; await is required before the throw on the next line, and only one loop iteration runs on this path
      failLogger.error(await formatFailure({
        summary:
          `${runPrefix}expected at least one assertion but none were called (${
            formatDuration(durationMs,)
          })`,
        value: noAssertionsCause,
      },),);
      throw new Error(
        name,
        { cause: noAssertionsCause, },
      );
    }
    //endregion Assertion count verification

    passLogger.debug(`${runPrefix}(${formatDuration(durationMs,)})`,);
  }

  return { name, };
}

/**
 Defines a single test case as a lazy {@link TestDescriptor}.
 Construction is synchronous and side-effect free; execution begins
 when the descriptor is awaited or dispatched by a parent suite.
 
 @param opts - test options
 
 @returns lazy descriptor that resolves with the test result
 
 @throws Error wrapping the original failure with the test name and cause chain
 
 @example
 ```ts
 await it({
   name: 'adds two numbers',
   fn: async () => {
     expect(add(1, 2)).toBe(3);
   },
 });
 ```
 */
export function it(opts: ItOptions,): TestDescriptor<ItResult> {
  return makeDescriptor(function runItWithCtx(ctx,) {
    /**
     Explicit or scheduler-provided parent for rejection diagnostics.
     */
    const logger = opts.l ?? ctx.parentLogger;
    return runObservedExecution({
      kind: 'test',
      name: opts.name,
      ...logger === undefined ? {} : { logger, },
      run: function runObservedTest(): Promise<ItResult> {
        return runIt({
          opts,
          descriptorCtx: ctx,
        },);
      },
    },);
  },);
}
