import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  sinks,
  type LogRecord,
} from '@monochromatic-dev/logger-core';

/**
 Sink factories under test, read from the built artifact's `sinks` namespace.
 */
const {
  createConsoleSink,
} = sinks;

/**
 Awaits two microtask hops so any pending `queueMicrotask(flushBuffer)`
 has definitely fired and the buffer is drained before the next assertion.
 One hop would often be enough, but two is cheap insurance against timing
 skew from the harness itself.
 */
async function waitForFlush(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

/**
 Builds a `LogRecord` with a fixed timestamp so the formatted output
 is stable across test runs.
 
 @param level - Severity level.
 
 @param message - Message body.
 
 @returns A complete `LogRecord`.
 */
function record(
  {
    level,
    message,
  }: {
    readonly level: LogRecord['level'];
    readonly message: string;
  },
): LogRecord {
  return {
    level,
    message,
    timestamp: 0,
  };
}

/**
 Appends `--verbose` to `process.argv` and removes it again when the returned
 value goes out of `using` scope, so a verbose-detection test cannot leak the
 flag into the sibling test that asserts the silenced default.
 
 @returns Disposable that restores `process.argv` on scope exit.
 */
function withVerboseArgv(): Disposable {
  process.argv
    .push('--verbose',);
  return {
    [Symbol.dispose](): void {
      // Reassign to a flag-free copy rather than splice by index: the test
      // environment carries no `--verbose` of its own, so filtering restores
      // the original vector and dodges the conflicting index-check lint rules.
      process.argv = process.argv
        .filter(function keepNonVerbose(arg,) {
          return arg !== '--verbose';
        },);
    },
  };
}

await describe({
  name: 'console sink (microtask-batched)',
  // Sequential because every test spies a global (`console.*`); concurrent
  // runs would clobber each other's spies. Sink state is now per-instance
  // (each test builds its own `createConsoleSink()`), so no reset hook or
  // shared-buffer coordination is needed.
  concurrency: 1,
  children: [
    it({
      name: 'verify resolves true in a test environment',
      fn: async () => {
        const sink = createConsoleSink();
        expect(await sink.verify(),)
          .toBe(true,);
      },
    },),

    it({
      name: 'single write defers until next microtask',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'info',
        );

        void sink.write(record({
          level: 'info',
          message: 'deferred',
        },),);
        expect(spy.callCount,)
          .toBe(0,);

        await waitForFlush();
        expect(spy.callCount,)
          .toBe(1,);
      },
    },),

    it({
      // Regression: MONOCHROMATIC_WARN=false drops warn records so machine-protocol consumers
      // (such as the toml-edit conformance codec) keep their output clean.
      name: 'MONOCHROMATIC_WARN=false suppresses warn records',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_WARN = 'false';
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'warn',
        );

        void sink.write(record({
          level: 'warn',
          message: 'hushed',
        },),);

        await waitForFlush();
        expect(spy.callCount,)
          .toBe(0,);
        Reflect.deleteProperty(
          process.env,
          'MONOCHROMATIC_WARN',
        );
      },
    },),

    it({
      name: 'warn records emit when MONOCHROMATIC_WARN is unset',
      fn: async ({ sinon, },) => {
        Reflect.deleteProperty(
          process.env,
          'MONOCHROMATIC_WARN',
        );
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'warn',
        );

        void sink.write(record({
          level: 'warn',
          message: 'audible',
        },),);

        await waitForFlush();
        expect(spy.callCount,)
          .toBe(1,);
      },
    },),

    it({
      name: 'contiguous same-level runs collapse to one console call',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'info',
        );

        void sink.write(record({
          level: 'info',
          message: 'first',
        },),);
        void sink.write(record({
          level: 'info',
          message: 'second',
        },),);

        await waitForFlush();
        expect(spy.callCount,)
          .toBe(1,);
        const emitted = spy.firstCall.args[0] as string;
        expect(emitted.split('\n',).length,)
          .toBe(2,);
        expect(emitted,)
          .toContain('first',);
        expect(emitted,)
          .toContain('second',);
      },
    },),

    it({
      name: 'debug writes to process stderr when process stderr is available',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const stderrSpy = sinon.stub(
          process.stderr,
          'write',
        );
        const consoleDebugSpy = sinon.stub(
          console,
          'debug',
        );

        void sink.write(record({
          level: 'debug',
          message: 'shown',
        },),);
        await waitForFlush();

        expect(stderrSpy.callCount,)
          .toBe(1,);
        expect(consoleDebugSpy.callCount,)
          .toBe(0,);
        const emitted = stderrSpy.firstCall.args[0] as string;
        expect(emitted,)
          .toBe('[debug] [1970-01-01T00:00:00.000Z] shown\n',);
      },
    },),

    it({
      name: 'level transitions split into separate console calls',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const stderrSpy = sinon.stub(
          process.stderr,
          'write',
        );
        const consoleDebugSpy = sinon.stub(
          console,
          'debug',
        );
        const warnSpy = sinon.stub(
          console,
          'warn',
        );

        void sink.write(record({
          level: 'debug',
          message: 'd1',
        },),);
        void sink.write(record({
          level: 'debug',
          message: 'd2',
        },),);
        void sink.write(record({
          level: 'warn',
          message: 'w1',
        },),);
        void sink.write(record({
          level: 'debug',
          message: 'd3',
        },),);

        await waitForFlush();
        expect(stderrSpy.callCount,)
          .toBe(2,);
        expect(consoleDebugSpy.callCount,)
          .toBe(0,);
        expect(warnSpy.callCount,)
          .toBe(1,);
        const firstDebug = stderrSpy.firstCall.args[0] as string;
        const secondDebug = stderrSpy.secondCall.args[0] as string;
        expect(firstDebug.split('\n',).length,)
          .toBe(3,);
        expect(secondDebug.split('\n',).length,)
          .toBe(2,);
      },
    },),

    it({
      name: 'formats each record as [level] [iso] message',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'info',
        );

        void sink.write(record({
          level: 'info',
          message: 'hi',
        },),);
        await waitForFlush();

        const emitted = spy.firstCall.args[0] as string;
        expect(emitted,)
          .toBe('[info] [1970-01-01T00:00:00.000Z] hi',);
      },
    },),

    it({
      name: 'neutralizes terminal control characters before they reach console',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'info',
        );

        void sink.write(record({
          level: 'info',
          message: 'title:\u001B]0;PWNED\u0007 clear:\u001B[2J\n\tkept',
        },),);
        await waitForFlush();

        const emitted = spy.firstCall.args[0] as string;
        expect(emitted,)
          .toBe('[info] [1970-01-01T00:00:00.000Z] title:\\u001B]0;PWNED\\u0007 clear:\\u001B[2J\n\tkept',);
        expect(emitted,)
          .not
          .toContain('\u001B',);
      },
    },),

    it({
      name: 'each level routes to its mapped output channel',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        // Stub (not spy) console.trace: Node's Console.prototype.trace
        // internally calls this.error(stack), so a spied trace would delegate
        // into the spied console.error and inflate the error count by one.
        // The stub still records the routing call (callCount) without that
        // delegation, isolating the sink's level-to-channel mapping from Node
        // console internals.
        const traceSpy = sinon.stub(
          console,
          'trace',
        );
        const stderrSpy = sinon.stub(
          process.stderr,
          'write',
        );
        const consoleDebugSpy = sinon.stub(
          console,
          'debug',
        );
        const infoSpy = sinon.stub(
          console,
          'info',
        );
        const warnSpy = sinon.stub(
          console,
          'warn',
        );
        const errorSpy = sinon.stub(
          console,
          'error',
        );

        void sink.write(record({
          level: 'trace',
          message: 't',
        },),);
        await waitForFlush();
        void sink.write(record({
          level: 'debug',
          message: 'd',
        },),);
        await waitForFlush();
        void sink.write(record({
          level: 'info',
          message: 'i',
        },),);
        await waitForFlush();
        void sink.write(record({
          level: 'warn',
          message: 'w',
        },),);
        await waitForFlush();
        void sink.write(record({
          level: 'error',
          message: 'e',
        },),);
        await waitForFlush();
        void sink.write(record({
          level: 'fatal',
          message: 'f',
        },),);
        await waitForFlush();

        // Assert the whole count map at once: a future drift reports which
        // channel's count changed (e.g. {error: 3} vs {error: 2}) instead of a
        // bare "expected 3 to equal 2". error is 2 because console.error backs
        // both 'error' and 'fatal' (fatal shares the error method). Keys are
        // alphabetized to satisfy oxlint sort-keys.
        expect({
          debugConsole: consoleDebugSpy.callCount,
          debugStderr: stderrSpy.callCount,
          error: errorSpy.callCount,
          info: infoSpy.callCount,
          trace: traceSpy.callCount,
          warn: warnSpy.callCount,
        },)
          .toEqual({
            debugConsole: 0,
            debugStderr: 1,
            error: 2,
            info: 1,
            trace: 1,
            warn: 1,
          },);
      },
    },),

    it({
      name: 'silent gating drops debug when verbose is off',
      fn: async ({ sinon, },) => {
        delete process.env.MONOCHROMATIC_VERBOSE;
        // A fresh sink recomputes verbose on its first write, so clearing
        // MONOCHROMATIC_VERBOSE here yields verbose=false (process.argv has no --verbose in a
        // normal test run, and 'window' is not in globalThis under Node).
        const sink = createConsoleSink();
        const stderrSpy = sinon.stub(
          process.stderr,
          'write',
        );
        const traceSpy = sinon.spy(
          console,
          'trace',
        );

        void sink.write(record({
          level: 'debug',
          message: 'hidden',
        },),);
        void sink.write(record({
          level: 'trace',
          message: 'hidden',
        },),);

        await waitForFlush();
        expect(stderrSpy.callCount,)
          .toBe(0,);
        expect(traceSpy.callCount,)
          .toBe(0,);
      },
    },),

    it({
      name: '--verbose in process.argv enables debug output without MONOCHROMATIC_VERBOSE',
      fn: async ({ sinon, },) => {
        delete process.env.MONOCHROMATIC_VERBOSE;
        // A fresh sink recomputes verbose on its first write; with MONOCHROMATIC_VERBOSE unset
        // and no browser `window`, the --verbose argv flag is the sole trigger.
        using _restoreArgv = withVerboseArgv();
        const sink = createConsoleSink();
        const spy = sinon.stub(
          process.stderr,
          'write',
        );

        void sink.write(record({
          level: 'debug',
          message: 'shown',
        },),);
        await waitForFlush();

        expect(spy.callCount,)
          .toBe(1,);
      },
    },),

    it({
      name: 'cross-microtask writes produce separate console calls',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'info',
        );

        void sink.write(record({
          level: 'info',
          message: 'first',
        },),);
        await waitForFlush();
        void sink.write(record({
          level: 'info',
          message: 'second',
        },),);
        await waitForFlush();

        expect(spy.callCount,)
          .toBe(2,);
      },
    },),

    it({
      name: 'flush() drains synchronously before the await resolves',
      fn: async ({ sinon, },) => {
        process.env.MONOCHROMATIC_VERBOSE = 'true';
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'info',
        );

        void sink.write(record({
          level: 'info',
          message: 'force-drain',
        },),);
        expect(spy.callCount,)
          .toBe(0,);

        await sink.flush?.();
        expect(spy.callCount,)
          .toBe(1,);
      },
    },),

    it({
      name: 'flush() on empty buffer resolves without emitting',
      fn: async ({ sinon, },) => {
        const sink = createConsoleSink();
        const spy = sinon.spy(
          console,
          'info',
        );

        await sink.flush?.();
        expect(spy.callCount,)
          .toBe(0,);
      },
    },),
  ],
},);
