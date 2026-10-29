/**
 Scheduled stateful property over the logger orchestration.

 A random program of `log`, `release`, and `flush` steps runs against a
 logger built over one to four scripted fake sinks while a fast-check
 scheduler chooses the order in which every verify, write, and flush outcome
 reaches the logger. Afterwards the real sinks and the reference model must
 agree on: the exact records each sink was handed, in order (exactly-once
 delivery, startup replay, dropout on failed verify); which of them the sink
 delivered (write resilience); whether each flush settled inside its deadline
 (flush totality); how many breadcrumbs were written; and whether a log call
 threw for want of a backend.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  array,
  assert,
  asyncProperty,
  constantFrom,
  scheduler,
} from 'fast-check';

import {
  availableSinkScript,
  sinkScript,
} from './fake-sink-arbitrary.ts';
import {
  formatSinkScript,
  outcomeAt,
  type ScriptedSink,
  type SinkScript,
} from './fake-sink.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import {
  type ProgramOp,
  type ProgramRun,
  runProgram,
  settleRuns,
} from './harness.ts';

//region Arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const run = fuzzRunPlan();

/**
 Longest program; flush steps are expensive when they hit the deadline, so
 they are rarer than log and release steps.
 */
const MAX_OPS = 8;

/**
 Program steps, weighted toward logging and releasing.
 */
const program = array(
  constantFrom<ProgramOp>(
    'log',
    'log',
    'log',
    'release',
    'release',
    'flush',
  ),
  { maxLength: MAX_OPS, },
);

/**
 Sink lists of one to four scripted sinks.
 */
const sinkLists = array(
  sinkScript(),
  {
    minLength: 1,
    maxLength: 4,
  },
);

//endregion Arbitraries

//region Checks

/**
 Maps records to their messages.

 @param records - Records to map.

 @returns Messages in order.
 */
function messages(records: readonly { readonly message: string; }[],): string[] {
  return records.map(function toMessage(record,) {
    return record.message;
  },);
}

/**
 Records the model expects a sink to have delivered: its attempts filtered
 by the scripted write outcome at each call index.

 @param attempts - Expected attempts in order.

 @param script - Sink script.

 @returns Expected delivered messages.
 */
function expectedDelivered(
  {
    attempts,
    script,
  }: {
    readonly attempts: readonly { readonly message: string; }[];
    readonly script: SinkScript;
  },
): string[] {
  return attempts
    .filter(function resolved(
      _record,
      callIndex,
    ) {
      return outcomeAt({
        script: script.write,
        callIndex,
      },) === 'resolve';
    },)
    .map(function toMessage(record,) {
      return record.message;
    },);
}

/**
 Asserts every contract the model states about one finished run.

 @param finished - Harness result.

 @param scripts - Sink scripts, for the counterexample text.

 @param breadcrumbs - Messages of every `console.warn` call the logger made
 during the run, so a count mismatch names the extra or missing breadcrumb.
 */
function checkRun(
  {
    finished,
    scripts,
    breadcrumbs,
  }: {
    readonly finished: ProgramRun;
    readonly scripts: readonly SinkScript[];
    readonly breadcrumbs: readonly string[];
  },
): void {
  /**
   Script lines for the failure message.
   */
  const described = scripts.map(function describeScript(
    script,
    index,
  ) {
    return formatSinkScript({
      index,
      script,
    },);
  },).join('\n',);
  finished.fakes.forEach(function checkSink(
    fake: ScriptedSink,
    index,
  ) {
    /**
     Model of this sink.
     */
    const expected = finished.model.sinks[index];
    if (expected === undefined)
      throw new Error(`model lost sink ${index}\n${described}`,);
    checked({
      context: `attempts of sink ${index}\n${described}`,
      check: function attemptsAgree() {
        expect(messages(fake.attempts,),)
          .toEqual(messages(expected.attempts,),);
      },
    },);
    checked({
      context: `deliveries of sink ${index}\n${described}`,
      check: function deliveriesAgree() {
        expect(messages(fake.delivered,),)
          .toEqual(expectedDelivered({
            attempts: expected.attempts,
            script: fake.script,
          },),);
      },
    },);
  },);
  checked({
    context: `flush deadline verdicts\n${described}`,
    check: function flushesAgree() {
      expect(finished.observed.flushesWithinDeadline,)
        .toEqual(finished.model.flushesWithinDeadline,);
    },
  },);
  checked({
    context: `breadcrumbs\n${described}\n${breadcrumbs.join('\n',)}`,
    check: function breadcrumbsAgree() {
      expect(breadcrumbs.length,)
        .toBe(finished.model.breadcrumbs,);
    },
  },);
  checked({
    context: `no-backend throw agreement\n${described}`,
    check: function throwsAgree() {
      expect(finished.observed.throwMismatch,)
        .toBe(false,);
    },
  },);
}

/**
 Runs one assertion and, on failure, rethrows with the counterexample
 context so the shrunk report names the sink scripts.

 @param context - Text describing what was compared and the scripts.

 @param check - Assertion to run.

 @throws Error carrying the original assertion error as its cause.
 */
function checked(
  {
    context,
    check,
  }: {
    readonly context: string;
    readonly check: () => void;
  },
): void {
  try {
    check();
  }
  catch (error: unknown) {
    throw new Error(
      context,
      { cause: error, },
    );
  }
}

//endregion Checks

await describe({
  name: 'scheduled orchestration model',
  // Each property stubs the shared console.warn, so they run one at a time.
  concurrency: 1,
  children: [
    it({
      name: 'the real sinks and the model agree on attempts, deliveries, flush deadlines, breadcrumbs, and throws',
      timeout: run.timeout,
      fn: async ({ sinon, },) => {
        const warn = sinon.stub(
          console,
          'warn',
        );
        await assert(
          asyncProperty(
            scheduler(),
            sinkLists,
            program,
            async function agrees(
              s,
              scripts,
              ops,
            ) {
              warn.resetHistory();
              const finished = await runProgram({
                scheduler: s,
                scripts,
                ops,
              },);
              checkRun({
                finished,
                scripts,
                breadcrumbs: warn.getCalls().map(function toMessage(call,) {
                  return String(call.args[0],);
                },),
              },);
            },
          ),
          run.params,
        );
        // A campaign interrupted by its time limit abandons the run in flight;
        // let it finish so its late breadcrumbs stay out of the next property.
        await settleRuns();
      },
    },),

    it({
      name: 'over always-available sinks every record reaches every sink exactly once regardless of order',
      timeout: run.timeout,
      fn: async ({ sinon, },) => {
        const warn = sinon.stub(
          console,
          'warn',
        );
        await assert(
          asyncProperty(
            scheduler(),
            array(
              availableSinkScript(),
              {
                minLength: 1,
                maxLength: 3,
              },
            ),
            program,
            async function exactlyOnce(
              s,
              scripts,
              ops,
            ) {
              warn.resetHistory();
              const finished = await runProgram({
                scheduler: s,
                scripts,
                ops,
              },);
              /**
               Messages logged without throwing, in order.
               */
              const logged = finished.model.sinks[0]?.attempts ?? [];
              finished.fakes.forEach(function everySink(fake,) {
                expect(messages(fake.attempts,),)
                  .toEqual(messages(logged,),);
                expect(new Set(messages(fake.attempts,),).size,)
                  .toBe(fake.attempts.length,);
              },);
              checkRun({
                finished,
                scripts,
                breadcrumbs: warn.getCalls().map(function toMessage(call,) {
                  return String(call.args[0],);
                },),
              },);
            },
          ),
          run.params,
        );
        await settleRuns();
      },
    },),
  ],
},);
