/**
 Runs one generated program against a real logger built over scripted fake
 sinks while folding the same program into the reference model, so a property
 can compare the two afterwards.

 The fast-check scheduler decides when each scripted outcome reaches the
 logger; the harness drives it one release at a time (`release` steps), lets
 the microtask queue drain, then reads the fake-sink trace to feed the model
 the verify settlements in the order the logger saw them. Init completion is
 observed through `initPromise`, and a `never` verify is folded as such right
 before it.

 Two real timers stay in play: the verify time limit and the flush deadline.
 The harness keeps the verify limit shorter than the deadline and waits for
 init before every flush, so the model's "hung" verdict for a flush depends
 only on `never` outcomes.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  createLogger,
  STARTUP_BUFFER_CAP,
} from '@monochromatic-dev/logger-core';
import type { Scheduler, } from 'fast-check';

import {
  createScriptedSink,
  type ScriptedSink,
  type SinkScript,
  type SinkTraceEvent,
} from './fake-sink.ts';
import {
  emptyModel,
  foldEvent,
  type ModelEvent,
  type ModelState,
} from './model.ts';

//region Program

/**
 One step of a generated program.
 */
export type ProgramOp = 'flush' | 'log' | 'release';

/**
 Verify time limit handed to the logger; shorter than the flush deadline so
 a `never` verify completes init before a flush can time out.
 */
export const VERIFY_TIMEOUT_MS = 25;

/**
 Flush deadline handed to the logger.
 */
export const FLUSH_DEADLINE_MS = 300;

/**
 Slack subtracted from the deadline when classifying a measured flush: a
 flush that took at least this much less than the deadline settled on its
 own; one that took about the deadline or longer hit it.
 */
const DEADLINE_TOLERANCE_MS = 60;

//endregion Program

//region Result

/**
 What the harness observed from the real logger, to compare with the model.
 */
export type Observed = {
  /**
   Per `flush` call, whether it settled clearly inside its deadline.
   */
  readonly flushesWithinDeadline: boolean[];
  /**
   Whether any `log` call threw when the model said it must not, or did not
   throw when the model said it must.
   */
  throwMismatch: boolean;
};

/**
 Harness result: the fakes with their attempts, the folded model, and the
 observations.
 */
export type ProgramRun = {
  readonly fakes: readonly ScriptedSink[];
  readonly model: ModelState;
  readonly observed: Observed;
};

//endregion Result

//region Run

/**
 Runs still executing. A fast-check campaign interrupted by its time limit
 abandons the run in flight, whose logger keeps firing deadline and verify
 timers into whatever `console.warn` is current; a property awaits
 {@link settleRuns} after `assert` so those breadcrumbs cannot leak into the
 next property's count.
 */
const inFlight = new Set<Promise<unknown>>();

/**
 Waits for every run started so far to finish.

 @example
 ```ts
 await assert(asyncProperty(...), run.params);
 await settleRuns();
 ```
 */
export async function settleRuns(): Promise<void> {
  await Promise.allSettled(inFlight,);
}

/**
 Runs `ops` against a fresh logger over `scripts`, folding the model as it
 goes, and tracks the run for {@link settleRuns}.

 @param scheduler - fast-check scheduler deciding outcome order.

 @param scripts - Sink scripts in logger sink order.

 @param ops - Program steps.

 @returns Fakes, model, and observations.

 @example
 ```ts
 const run = await runProgram({ scheduler: s, scripts, ops: ['log', 'release', 'flush'] });
 ```
 */
export async function runProgram(
  {
    scheduler,
    scripts,
    ops,
  }: {
    readonly scheduler: Scheduler;
    readonly scripts: readonly SinkScript[];
    readonly ops: readonly ProgramOp[];
  },
): Promise<ProgramRun> {
  /**
   Untracked run, registered until it settles.
   */
  const running = runProgramUntracked({
    scheduler,
    scripts,
    ops,
  },);
  inFlight.add(running,);
  /**
   Removes the run from the in-flight set once it settles, however it settles.
   */
  using _tracked = {
    [Symbol.dispose](): void {
      inFlight.delete(running,);
    },
  };
  return await running;
}

/**
 Runs `ops` against a fresh logger over `scripts`, folding the model as it
 goes.

 @param scheduler - fast-check scheduler deciding outcome order.

 @param scripts - Sink scripts in logger sink order.

 @param ops - Program steps.

 @returns Fakes, model, and observations.

 @example
 ```ts
 const run = await runProgramUntracked({ scheduler: s, scripts, ops: ['log'] });
 ```
 */
async function runProgramUntracked(
  {
    scheduler,
    scripts,
    ops,
  }: {
    readonly scheduler: Scheduler;
    readonly scripts: readonly SinkScript[];
    readonly ops: readonly ProgramOp[];
  },
): Promise<ProgramRun> {
  /**
   Shared trace every fake sink appends to.
   */
  const trace: SinkTraceEvent[] = [];
  /**
   Fake sinks in logger sink order.
   */
  const fakes = scripts.map(function toFake(
    script,
    index,
  ): ScriptedSink {
    return createScriptedSink({
      index,
      script,
      schedule: function gate<Value,>(
        {
          promise,
          label,
        }: {
          readonly promise: Promise<Value>;
          readonly label: string;
        },
      ): Promise<Value> {
        return scheduler.schedule(
          promise,
          label,
        );
      },
      trace,
    },);
  },);
  /**
   Logger under test and its readiness promise, built over the fake sinks
   with the harness's short verify limit and flush deadline.
   */
  const {
    logger,
    initPromise,
  } = createLogger({
    flushDeadlineMs: FLUSH_DEADLINE_MS,
    sinks: fakes.map(function toSink(fake,) {
      return fake.sink;
    },),
    verifyTimeoutMs: VERIFY_TIMEOUT_MS,
  },);
  /**
   Model state folded alongside the logger.
   */
  const model = emptyModel({ sinkCount: scripts.length, },);
  /**
   Observations from the real logger.
   */
  const observed: Observed = {
    flushesWithinDeadline: [],
    throwMismatch: false,
  };
  /**
   Harness bookkeeping: how much of the trace was folded, which sinks had
   their verify called and settled, and whether init was observed complete.
   */
  const cursor = {
    folded: 0,
    initDone: false,
    initFolded: false,
    logged: 0,
    verifyCalled: new Set<number>(),
    verifySettled: new Set<number>(),
  };

  /**
   Flips the init flag once the logger reports initialization complete.
   */
  async function watchInit(): Promise<void> {
    await initPromise;
    cursor.initDone = true;
  }
  void watchInit();

  /**
   Folds one model event.

   @param event - Event to fold.
   */
  function fold(event: ModelEvent,): void {
    foldEvent({
      state: model,
      scripts,
      cap: STARTUP_BUFFER_CAP,
      event,
    },);
  }

  /**
   Feeds the model every verify settlement the trace gained since the last
   sync, then init completion once observed (folding `never` verifies as
   such first).
   */
  function syncModel(): void {
    for (const event of trace.slice(cursor.folded,)) {
      if (event.hook !== 'verify')
        continue;
      if (event.phase === 'called') {
        cursor.verifyCalled
          .add(event.sink,);
        if (event.outcome === 'throw') {
          cursor.verifySettled
            .add(event.sink,);
          fold({
            kind: 'verify-settled',
            sink: event.sink,
            outcome: 'throw',
          },);
        }
        continue;
      }
      if (event.outcome === 'never')
        continue;
      cursor.verifySettled
        .add(event.sink,);
      fold({
        kind: 'verify-settled',
        sink: event.sink,
        outcome: (event.outcome === 'resolve-true') ? 'resolve-true' : ((event.outcome === 'resolve-false') ? 'resolve-false' : 'reject'),
      },);
    }
    cursor.folded = trace.length;
    if ((!cursor.initDone) || cursor.initFolded)
      return;
    for (const sink of cursor.verifyCalled) {
      if (cursor.verifySettled
        .has(sink,))
        continue;
      fold({
        kind: 'verify-settled',
        sink,
        outcome: 'never',
      },);
    }
    cursor.initFolded = true;
    fold({ kind: 'init-complete', },);
  }

  /**
   Runs one `log` step, checking the throw against the model's expectation.
   */
  function stepLog(): void {
    syncModel();
    /**
     Whether any modeled sink is currently available.
     */
    const anyAvailable = { value: false, };
    for (const sink of model.sinks) {
      if (sink.available)
        anyAvailable.value = true;
    }
    /**
     Whether the model says this call must throw: initialized with no
     available sink.
     */
    const mustThrow = model.initialized && (!anyAvailable.value);
    /**
     Message unique to this call.
     */
    const message = `log #${cursor.logged}`;
    cursor.logged += 1;
    /**
     Whether the call threw.
     */
    const threw = { value: false, };
    try {
      logger.info(message,);
    }
    catch (error: unknown) {
      threw.value = Error.isError(error,);
    }
    if (threw.value !== mustThrow)
      observed.throwMismatch = true;
    if (!threw.value) {
      fold({
        kind: 'log',
        record: {
          level: 'info',
          message,
          timestamp: 0,
        },
      },);
    }
  }

  /**
   Runs one `release` step: lets the scheduler settle one outcome, if any.
   */
  async function stepRelease(): Promise<void> {
    if (scheduler.count() > 0)
      await scheduler.waitNext(1,);
    await wait(0,);
    syncModel();
  }

  /**
   Runs one `flush` step: waits for init so the model's verdict is exact,
   folds the flush, then times the real one while the scheduler releases
   whatever it can.
   */
  async function stepFlush(): Promise<void> {
    await scheduler.waitFor(initPromise,);
    await wait(0,);
    syncModel();
    fold({ kind: 'flush', },);
    /**
     Wall-clock start of the real flush.
     */
    const start = performance.now();
    await scheduler.waitFor(logger.flush(),);
    /**
     Wall-clock duration of the real flush.
     */
    const elapsed = performance.now() - start;
    observed.flushesWithinDeadline
      .push(elapsed < (FLUSH_DEADLINE_MS - DEADLINE_TOLERANCE_MS),);
    await wait(0,);
    syncModel();
  }

  // The program is a sequence by definition: each step observes the state the
  // previous one left, so the awaits must serialize.
  for (const op of ops) {
    if (op === 'log')
      stepLog();
    else if (op === 'release')
      // oxlint-disable-next-line eslint/no-await-in-loop -- program steps are sequential by design
      await stepRelease();
    else
      // oxlint-disable-next-line eslint/no-await-in-loop -- program steps are sequential by design
      await stepFlush();
  }
  await scheduler.waitFor(initPromise,);
  await scheduler.waitIdle();
  await wait(0,);
  syncModel();

  return {
    fakes,
    model,
    observed,
  };
}

//endregion Run
