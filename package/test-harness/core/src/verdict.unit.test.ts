/**
 Tests outcome tags emitted by test and suite runners.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';
import {
  createLogger,
  type Logger,
  type LogRecord,
  type Sink,
} from '@monochromatic-dev/logger-core/ts';

/**
 Runner outcomes encoded by dedicated logger tags.
 
 @example
 ```ts
 const verdict: Verdict = 'PASS';
 ```
 */
type Verdict = 'FAIL' | 'PASS' | 'SKIP';

/**
 Capturing logger plus records received by its only sink.
 
 @example
 ```ts
 const capture = await createCapture();
 capture.logger.info('message');
 ```
 */
type Capture = {
  readonly logger: Logger;
  readonly records: LogRecord[];
};

/**
 Builds logger whose sink retains every record for assertions.
 
 @returns initialized logger and mutable record collection owned by sink
 
 @example
 ```ts
 const capture = await createCapture();
 await capture.logger.flush();
 ```
 */
async function createCapture(): Promise<Capture> {
  /** Records retained in sink arrival order. */
  const records: LogRecord[] = [];
  /** Sink exposing logger records without console or file formatting. */
  const sink: Sink = {
    verify: function verify(): Promise<boolean> {
      return Promise.resolve(true,);
    },
    write: function write(record: LogRecord,): Promise<void> {
      records.push(record,);
      return Promise.resolve();
    },
  };
  /** Logger and initialization boundary built over capturing sink. */
  const {
    initPromise,
    logger,
  } = createLogger({ sinks: [sink,], },);
  await initPromise;
  return {
    logger,
    records,
  };
}

/**
 Selects records carrying exact bracketed outcome token.
 
 @param records - captured logger records to inspect
 
 @param verdict - outcome token required in message
 
 @returns records containing dedicated verdict tag
 
 @example
 ```ts
 const failed = recordsForVerdict({ records, verdict: 'FAIL', });
 ```
 */
function recordsForVerdict({
  records,
  verdict,
}: {
  readonly records: readonly LogRecord[];
  readonly verdict: Verdict;
},): readonly LogRecord[] {
  /** Bracketed logger tag boundary used by console and JSONL consumers. */
  const tag = `[${verdict}]`;
  return records.filter(function hasVerdict(record,) {
    return record.message.includes(tag,);
  },);
}

/**
 Finds first captured record whose message includes fragment.
 
 @param records - captured records to search
 
 @param fragment - exact message fragment required
 
 @returns first matching record
 
 @throws Error when no record contains fragment
 
 @example
 ```ts
 const record = recordContaining({ records, fragment: '[FAIL]', });
 ```
 */
function recordContaining({
  records,
  fragment,
}: {
  readonly fragment: string;
  readonly records: readonly LogRecord[];
},): LogRecord {
  /** First record whose message contains requested fragment. */
  const record = records.find(function includesFragment(candidate,) {
    return candidate.message.includes(fragment,);
  },);
  if (record === undefined)
    throw new Error(`No captured log record contains: ${fragment}`,);
  return record;
}

await describe({
  name: 'verdict tags',
  children: [
    it({
      name: 'separate test outcomes from names and diagnostics containing verdict words',
      fn: async () => {
        /** Capture for direct test-level outcomes. */
        const capture = await createCapture();

        await it({
          name: 'name contains PASS FAIL PASSAGE FAILURE',
          l: capture.logger,
          fn: async () => {},
        },);

        /** Failure descriptor whose diagnostic repeats every collision word. */
        const failed = it({
          name: 'diagnostic collision',
          l: capture.logger,
          fn: async () => {
            throw new Error('diagnostic contains PASS FAIL PASSAGE FAILURE',);
          },
        },);
        /** Caught failure proving inner test ran without failing outer harness test. */
        let caught: unknown;
        try {
          await failed;
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);

        await it({
          name: 'skipped collision PASSAGE FAILURE',
          l: capture.logger,
          skip: 'reason contains PASS FAIL PASSAGE FAILURE',
          fn: async () => {},
        },);
        await capture.logger.flush();

        /** Passing verdict records selected independently from bare words. */
        const passes = recordsForVerdict({
          records: capture.records,
          verdict: 'PASS',
        },);
        /** Failing verdict records selected independently from diagnostics. */
        const failures = recordsForVerdict({
          records: capture.records,
          verdict: 'FAIL',
        },);
        /** Skipped verdict records selected independently from reason text. */
        const skips = recordsForVerdict({
          records: capture.records,
          verdict: 'SKIP',
        },);

        expect(passes,).toHaveLength(1,);
        expect(failures,).toHaveLength(1,);
        expect(skips,).toHaveLength(1,);
        expect(passes[0]?.level,).toBe('debug',);
        expect(failures[0]?.level,).toBe('error',);
        expect(skips[0]?.level,).toBe('info',);
        expect(passes[0]?.message,).toContain(
          '[name contains PASS FAIL PASSAGE FAILURE] [PASS] (',
        );
        expect(failures[0]?.message,).toContain(
          '[diagnostic collision] [FAIL] (',
        );
        expect(failures[0]?.message,).toContain(
          'diagnostic contains PASS FAIL PASSAGE FAILURE',
        );
        expect(skips[0]?.message,).toContain(
          '[skipped collision PASSAGE FAILURE] [SKIP] reason contains PASS FAIL PASSAGE FAILURE',
        );
      },
    },),

    it({
      name: 'tags test and suite records while preserving hierarchy',
      fn: async () => {
        /** Capture for nested suite outcomes. */
        const capture = await createCapture();
        /** Mixed suite proving fulfilled-child summary and failure rollup carry separate verdicts. */
        const mixedSuite = describe({
          name: 'suite contains PASSAGE FAILURE',
          l: capture.logger,
          concurrency: 1,
          children: [
            it({
              name: 'child contains PASS',
              fn: async () => {},
            },),
            it({
              name: 'child contains FAILURE',
              fn: async () => {
                throw new Error('suite diagnostic contains PASS FAIL PASSAGE FAILURE',);
              },
            },),
          ],
        },);
        /** Caught mixed-suite failure retained for behavior assertion. */
        let caught: unknown;
        try {
          await mixedSuite;
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        await capture.logger.flush();

        /** Test pass plus fulfilled-child summary. */
        const passes = recordsForVerdict({
          records: capture.records,
          verdict: 'PASS',
        },);
        /** Test failure plus suite failure rollup. */
        const failures = recordsForVerdict({
          records: capture.records,
          verdict: 'FAIL',
        },);

        expect(passes,).toHaveLength(2,);
        expect(failures,).toHaveLength(2,);
        expect(passes[0]?.level,).toBe('debug',);
        expect(passes[1]?.level,).toBe('info',);
        expect(failures[0]?.level,).toBe('error',);
        expect(failures[1]?.level,).toBe('error',);
        expect(passes[0]?.message,).toContain(
          '[suite contains PASSAGE FAILURE] [child contains PASS] [PASS] (',
        );
        expect(passes[1]?.message,).toBe(
          '[suite contains PASSAGE FAILURE] [PASS] child contains PASS',
        );
        expect(failures[0]?.message,).toContain(
          '[suite contains PASSAGE FAILURE] [child contains FAILURE] [FAIL] (',
        );
        expect(failures[1]?.message,).toContain(
          '[suite contains PASSAGE FAILURE] [FAIL] (',
        );
      },
    },),

    it({
      name: 'tags expected failures, repeats, timeouts, and boolean skips',
      fn: async () => {
        /** Number of additional repeated test runs. */
        const REPEATS = 1;
        /** Delay that exceeds timeout boundary. */
        const DELAY_MS = 20;
        /** Timeout that fires before delayed test resolves. */
        const TIMEOUT_MS = 1;
        /** Capture for remaining test verdict branches. */
        const capture = await createCapture();

        await it({
          name: 'expected failure',
          l: capture.logger,
          fails: 'reason contains PASS FAIL PASSAGE FAILURE',
          fn: async () => {
            throw new Error('expected throw',);
          },
        },);

        /** Expected-failure descriptor that unexpectedly resolves. */
        const unexpectedSuccess = it({
          name: 'unexpected success',
          l: capture.logger,
          fails: true,
          fn: async () => {},
        },);
        /** Caught unexpected-success failure. */
        let unexpectedCaught: unknown;
        try {
          await unexpectedSuccess;
        }
        catch (error: unknown) {
          unexpectedCaught = error;
        }
        expect(unexpectedCaught,).toBeInstanceOf(Error,);

        await it({
          name: 'repeated pass',
          l: capture.logger,
          repeats: REPEATS,
          fn: async () => {},
        },);

        /** Descriptor whose timeout becomes failure verdict. */
        const timed = it({
          name: 'timed test',
          l: capture.logger,
          timeout: TIMEOUT_MS,
          fn: async () => {
            await wait(DELAY_MS,);
          },
        },);
        /** Caught timeout failure. */
        let timeoutCaught: unknown;
        try {
          await timed;
        }
        catch (error: unknown) {
          timeoutCaught = error;
        }
        expect(timeoutCaught,).toBeInstanceOf(Error,);

        await it({
          name: 'boolean skip',
          l: capture.logger,
          skip: true,
          fn: async () => {},
        },);
        await capture.logger.flush();

        /** Expected-failure pass verdict. */
        const expectedFailure = recordContaining({
          records: capture.records,
          fragment: '[expected failure] [PASS] threw as expected (reason contains PASS FAIL PASSAGE FAILURE) (',
        },);
        /** Unexpected-success failure verdict. */
        const unexpectedFailure = recordContaining({
          records: capture.records,
          fragment: '[unexpected success] [FAIL] expected to throw but passed (',
        },);
        /** First repeated pass verdict. */
        const firstRepeat = recordContaining({
          records: capture.records,
          fragment: '[repeated pass] [PASS] [run 1/2] (',
        },);
        /** Second repeated pass verdict. */
        const secondRepeat = recordContaining({
          records: capture.records,
          fragment: '[repeated pass] [PASS] [run 2/2] (',
        },);
        /** Timeout failure verdict. */
        const timeoutFailure = recordContaining({
          records: capture.records,
          fragment: '[timed test] [FAIL] (',
        },);
        /** Boolean-skip verdict. */
        const booleanSkip = recordContaining({
          records: capture.records,
          fragment: '[boolean skip] [SKIP] (no reason)',
        },);

        expect(expectedFailure.level,).toBe('debug',);
        expect(unexpectedFailure.level,).toBe('error',);
        expect(firstRepeat.level,).toBe('debug',);
        expect(secondRepeat.level,).toBe('debug',);
        expect(timeoutFailure.level,).toBe('error',);
        expect(timeoutFailure.message,).toContain('Timed out after 1ms',);
        expect(booleanSkip.level,).toBe('info',);
      },
    },),

    it({
      name: 'tags all-failure, repeated, and timed suite branches',
      fn: async () => {
        /** Number of additional repeated suite runs. */
        const REPEATS = 1;
        /** Delay that exceeds suite timeout boundary. */
        const DELAY_MS = 20;
        /** Suite timeout that fires before delayed child resolves. */
        const TIMEOUT_MS = 1;
        /** Capture for all-failure suite path. */
        const allFailureCapture = await createCapture();
        /** Suite with no fulfilled child. */
        const allFailure = describe({
          name: 'all failure suite',
          l: allFailureCapture.logger,
          children: [
            it({
              name: 'only failure',
              fn: async () => {
                throw new Error('only child failed',);
              },
            },),
          ],
        },);
        /** Caught all-failure suite error. */
        let allFailureCaught: unknown;
        try {
          await allFailure;
        }
        catch (error: unknown) {
          allFailureCaught = error;
        }
        expect(allFailureCaught,).toBeInstanceOf(Error,);
        await allFailureCapture.logger.flush();

        expect(recordsForVerdict({
          records: allFailureCapture.records,
          verdict: 'PASS',
        },),).toHaveLength(0,);
        expect(recordsForVerdict({
          records: allFailureCapture.records,
          verdict: 'FAIL',
        },),).toHaveLength(2,);
        expect(recordContaining({
          records: allFailureCapture.records,
          fragment: '[all failure suite] [FAIL] (',
        },).level,).toBe('error',);

        /** Capture for repeated empty-suite verdicts. */
        const repeatCapture = await createCapture();
        await describe({
          name: 'repeated empty suite',
          l: repeatCapture.logger,
          repeats: REPEATS,
          children: [],
        },);
        await repeatCapture.logger.flush();
        expect(recordContaining({
          records: repeatCapture.records,
          fragment: '[repeated empty suite] [PASS] [run 1/2] (',
        },).level,).toBe('info',);
        expect(recordContaining({
          records: repeatCapture.records,
          fragment: '[repeated empty suite] [PASS] [run 2/2] (',
        },).level,).toBe('info',);

        /** Capture for suite-timeout verdict. */
        const timeoutCapture = await createCapture();
        /** Suite that times out while child remains in flight. */
        const timedSuite = describe({
          name: 'timed suite',
          l: timeoutCapture.logger,
          timeout: TIMEOUT_MS,
          children: [
            it({
              name: 'delayed child',
              fn: async () => {
                await wait(DELAY_MS,);
              },
            },),
          ],
        },);
        /** Caught suite timeout. */
        let timeoutCaught: unknown;
        try {
          await timedSuite;
        }
        catch (error: unknown) {
          timeoutCaught = error;
        }
        expect(timeoutCaught,).toBeInstanceOf(Error,);
        await wait(DELAY_MS,);
        await timeoutCapture.logger.flush();

        /** Suite-timeout failure verdict. */
        const timeoutFailure = recordContaining({
          records: timeoutCapture.records,
          fragment: '[timed suite] [FAIL] timeout (',
        },);
        expect(timeoutFailure.level,).toBe('error',);
        expect(timeoutFailure.message,).toContain('Timed out after 1ms',);
      },
    },),

    it({
      name: 'tags skipped and empty suites',
      fn: async () => {
        /** Capture for suite paths with no child execution. */
        const capture = await createCapture();

        await describe({
          name: 'skipped suite PASSAGE FAILURE',
          l: capture.logger,
          skip: 'blocked by PASS FAIL diagnostic',
          children: [],
        },);
        await describe({
          name: 'boolean skipped suite',
          l: capture.logger,
          skip: true,
          children: [],
        },);
        await describe({
          name: 'empty suite PASSAGE FAILURE',
          l: capture.logger,
          children: [],
        },);
        await capture.logger.flush();

        /** Empty-suite pass verdict. */
        const passes = recordsForVerdict({
          records: capture.records,
          verdict: 'PASS',
        },);
        /** Skipped-suite verdict. */
        const skips = recordsForVerdict({
          records: capture.records,
          verdict: 'SKIP',
        },);

        expect(passes,).toHaveLength(1,);
        expect(skips,).toHaveLength(2,);
        expect(passes[0]?.level,).toBe('info',);
        expect(skips[0]?.level,).toBe('info',);
        expect(skips[1]?.level,).toBe('info',);
        expect(passes[0]?.message,).toContain(
          '[empty suite PASSAGE FAILURE] [PASS] (',
        );
        expect(skips[0]?.message,).toBe(
          '[skipped suite PASSAGE FAILURE] [SKIP] suite "skipped suite PASSAGE FAILURE": blocked by PASS FAIL diagnostic',
        );
        expect(skips[1]?.message,).toBe(
          '[boolean skipped suite] [SKIP] suite "boolean skipped suite"',
        );
      },
    },),
  ],
},);
