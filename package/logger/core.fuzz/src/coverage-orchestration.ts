/**
 Coverage-driver scenarios over the logger orchestration: every verify
 outcome under the time limit, startup buffering with and without
 overflow, the no-backend throw, every write outcome, every flush-hook
 outcome under the deadline, the lazily built default logger, and tagged
 wrappers. Scripted fake sinks with the identity gate make every scenario
 deterministic.

 @module
 */

import {
  createLogger,
  logger as defaultLogger,
  type Sink,
  STARTUP_BUFFER_CAP,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  createScriptedSink,
  identityGate,
  type SinkScript,
  type SinkTraceEvent,
  type VerifyOutcome,
} from './fake-sink.ts';

//region Fixtures

/**
 Verify limit short enough that a never-answering verify times out fast.
 */
const SHORT_VERIFY_TIMEOUT_MS = 20;

/**
 Flush deadline short enough that a hanging write or hook trips it fast.
 */
const SHORT_FLUSH_DEADLINE_MS = 40;

/**
 Overflow that exercises the plural marker wording.
 */
const PLURAL_OVERFLOW = 3;

/**
 Every verify outcome, one sink each.
 */
const VERIFY_OUTCOMES: readonly VerifyOutcome[] = [
  'resolve-true',
  'resolve-false',
  'reject',
  'throw',
  'never',
];

/**
 Script of a sink that verifies and writes without incident.

 @returns Always-available script.

 @example
 ```ts
 sinkFrom({ index: 0, script: healthy() });
 ```
 */
function healthy(): SinkScript {
  return {
    verify: {
      head: [],
      tail: 'resolve-true',
    },
    write: {
      head: [],
      tail: 'resolve',
    },
  };
}

/**
 Builds one scripted sink whose outcomes settle immediately.

 @param index - Stable identity for the sink.

 @param script - Per-hook outcome scripts.

 @returns Sink adapter.

 @example
 ```ts
 const sink = sinkFrom({ index: 0, script: healthy() });
 ```
 */
function sinkFrom({
  index,
  script,
}: {
  readonly index: number;
  readonly script: SinkScript;
},): Sink {
  /**
   Trace the fake records into; unread by the driver.
   */
  const trace: SinkTraceEvent[] = [];
  return createScriptedSink({
    index,
    script,
    schedule: identityGate,
    trace,
  },)
    .sink;
}

/**
 Builds the sink for one indexed verify outcome; its writes resolve.

 @param entry - Position and verify outcome, as `entries()` yields them.

 @returns Sink adapter.

 @example
 ```ts
 [...VERIFY_OUTCOMES.entries()].map(sinkForVerifyEntry);
 ```
 */
function sinkForVerifyEntry(entry: readonly [
  number,
  VerifyOutcome,
],): Sink {
  /**
   Position and verify outcome.
   */
  const [index, tail,] = entry;
  return sinkFrom({
    index,
    script: {
      verify: {
        head: [],
        tail,
      },
      write: {
        head: [],
        tail: 'resolve',
      },
    },
  },);
}

/**
 Builds the sink for one indexed script.

 @param entry - Position and script, as `entries()` yields them.

 @returns Sink adapter.

 @example
 ```ts
 [...scripts.entries()].map(sinkForScriptEntry);
 ```
 */
function sinkForScriptEntry(entry: readonly [
  number,
  SinkScript,
],): Sink {
  /**
   Position and script.
   */
  const [index, script,] = entry;
  return sinkFrom({
    index,
    script,
  },);
}

/**
 Runs a thunk expected to throw, swallowing the error so the driver keeps
 exercising remaining paths. Re-throws anything that is not an `Error`.

 @param thunk - Operation expected to throw.

 @example
 ```ts
 swallow(function noBackend() { logger.info('x'); });
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
  }
}

//endregion Fixtures

//region Scenarios

/**
 One sink per verify outcome under a short limit: replay to the one that
 verifies, breadcrumbs for the rejecting, throwing, and hanging ones.
 */
async function exerciseVerifyOutcomes(): Promise<void> {
  /**
   Logger over one sink per verify outcome.
   */
  const {
    initPromise,
    logger,
  } = createLogger({
    sinks: [...VERIFY_OUTCOMES.entries(),].map(sinkForVerifyEntry,),
    verifyTimeoutMs: SHORT_VERIFY_TIMEOUT_MS,
  },);
  logger.info('buffered before verification',);
  await initPromise;
  logger.warn('written after verification',);
  await logger.flush();
}

/**
 Overflows the startup buffer by `extra` records before the sink verifies,
 so the oldest are dropped and the marker record is written after init.

 @param extra - Records beyond the cap.
 */
async function exerciseStartupOverflow({ extra, }: { readonly extra: number; },): Promise<void> {
  /**
   Logger over one healthy sink that has not verified yet.
   */
  const {
    initPromise,
    logger,
  } = createLogger({
    sinks: [
      sinkFrom({
        index: 0,
        script: healthy(),
      },),
    ],
  },);
  for (let count = 0; count < (STARTUP_BUFFER_CAP + extra); count += 1)
    logger.debug(`startup ${String(count,)}`,);
  await initPromise;
  await logger.flush();
}

/**
 A logger whose only sink refuses, and one with no sinks: both throw on the
 first log after initialization.
 */
async function exerciseNoBackend(): Promise<void> {
  /**
   Logger whose only sink declines at verify.
   */
  const refused = createLogger({
    sinks: [
      sinkFrom({
        index: 0,
        script: {
          verify: {
            head: [],
            tail: 'resolve-false',
          },
          write: {
            head: [],
            tail: 'resolve',
          },
        },
      },),
    ],
  },);
  await refused.initPromise;
  swallow(function logNowhere(): void {
    refused.logger
      .info('nowhere to go',);
  },);
  await refused.logger
    .flush();
  /**
   Logger with no sinks at all.
   */
  const empty = createLogger({ sinks: [], },);
  await empty.initPromise;
  swallow(function logToNothing(): void {
    empty.logger
      .error('no sinks at all',);
  },);
  await empty.logger
    .flush();
}

/**
 Rejected, thrown, and hanging writes on one sink: the first two leave it
 available, the third trips the flush deadline, and a later flush drains
 normally.
 */
async function exerciseWriteOutcomes(): Promise<void> {
  /**
   Logger over one sink whose writes reject, throw, hang, then settle.
   */
  const {
    initPromise,
    logger,
  } = createLogger({
    flushDeadlineMs: SHORT_FLUSH_DEADLINE_MS,
    sinks: [
      sinkFrom({
        index: 0,
        script: {
          verify: {
            head: [],
            tail: 'resolve-true',
          },
          write: {
            head: [
              'reject',
              'throw',
              'never',
            ],
            tail: 'resolve',
          },
        },
      },),
    ],
  },);
  await initPromise;
  logger.info('rejected write',);
  logger.info('thrown write',);
  logger.info('hanging write',);
  await logger.flush();
  logger.info('settling write',);
  await logger.flush();
}

/**
 Rejecting, throwing, resolving, and absent flush hooks, then a hook that
 never settles under the deadline.
 */
async function exerciseFlushHooks(): Promise<void> {
  /**
   Hook scripts, one sink each; the last sink has no hook.
   */
  const scripts: readonly SinkScript[] = [
    {
      ...healthy(),
      flush: {
        head: ['reject',],
        tail: 'resolve',
      },
    },
    {
      ...healthy(),
      flush: {
        head: ['throw',],
        tail: 'resolve',
      },
    },
    {
      ...healthy(),
      flush: {
        head: [],
        tail: 'resolve',
      },
    },
    healthy(),
  ];
  /**
   Logger over the four hook scripts.
   */
  const mixed = createLogger({ sinks: [...scripts.entries(),].map(sinkForScriptEntry,), },);
  await mixed.initPromise;
  mixed.logger
    .info('before hooks',);
  await mixed.logger
    .flush();
  mixed.logger
    .info('after two retirements',);
  await mixed.logger
    .flush();
  /**
   Logger whose only flush hook never settles.
   */
  const hung = createLogger({
    flushDeadlineMs: SHORT_FLUSH_DEADLINE_MS,
    sinks: [
      sinkFrom({
        index: 0,
        script: {
          ...healthy(),
          flush: {
            head: [],
            tail: 'never',
          },
        },
      },),
    ],
  },);
  await hung.initPromise;
  hung.logger
    .info('hook will hang',);
  await hung.logger
    .flush();
}

/**
 The lazily built default logger at every level, then tagged wrappers over
 it and over an explicit inner logger.
 */
async function exerciseDefaultAndTagged(): Promise<void> {
  defaultLogger.trace('default trace',);
  defaultLogger.debug('default debug',);
  defaultLogger.info('default info',);
  defaultLogger.warn('default warn',);
  defaultLogger.error('default error',);
  defaultLogger.fatal('default fatal',);
  await defaultLogger.flush();
  /**
   Wrapper over the default logger.
   */
  const outer = tagged({ tag: 'coverage', },);
  outer.trace('tagged trace',);
  outer.debug('tagged debug',);
  outer.info('tagged info',);
  outer.warn('tagged warn',);
  outer.error('tagged error',);
  outer.fatal('tagged fatal',);
  /**
   Wrapper over an explicit inner logger.
   */
  const inner = tagged({
    l: outer,
    tag: 'inner',
  },);
  inner.info('nested tags',);
  await inner.flush();
}

//endregion Scenarios

/**
 Runs every orchestration scenario in order.

 @example
 ```ts
 await exerciseOrchestration();
 ```
 */
export async function exerciseOrchestration(): Promise<void> {
  await exerciseVerifyOutcomes();
  await exerciseStartupOverflow({ extra: 1, },);
  await exerciseStartupOverflow({ extra: PLURAL_OVERFLOW, },);
  await exerciseNoBackend();
  await exerciseWriteOutcomes();
  await exerciseFlushHooks();
  await exerciseDefaultAndTagged();
}
