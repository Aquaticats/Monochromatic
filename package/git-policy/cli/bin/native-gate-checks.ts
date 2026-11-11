/**
 The release gate's two checks, Clippy and the whole test suite, started together and judged together.

 The gate started Clippy only after the suite had passed, so a full Clippy compile waited behind a test run about four
 times its length. Nothing orders the two: each starts one container from the same image, reads nothing the other
 writes, and writes nothing the other reads. Started together, the shorter check finishes inside the longer one, and the
 gate still fails on either check alone (`doc/planning/git-policy-cli-cachyos-test-containers.md`).
 */
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { containerRemovalDeadlineMilliseconds, } from './native-gate-container.ts';
import {
  type CommandOutcome,
  NativeVerificationError,
} from './native-verification-process.ts';

//region What the gate starts

/**
 Milliseconds one check of the gate may take before the gate stops waiting for it, and the bound each check is told so
 it can stop what it started.

 The suite is the gate's longest phase and the gate's own record of its wall time sizes this bound
 (`doc/planning/git-policy-cli-cachyos-test-containers.md`, "Where the release gate's wall time goes"): of the gates of
 2026-11-10 the longest test run took 1,104 s with 7,192 tests at two threads and 479 s at the four threads the gate runs
 now, and Clippy took 195 to 261 s. Three times that slowest measured phase is 3,312 s, so the bound is one hour, the
 same arithmetic as the three hours the btrfs host controls' deadline allows against their own measured run
 (`bin/native-release-host-control-requirements.ts`); it is what a person waits at most for a gate that then reports or
 refuses: a container that never ends refuses the gate instead of holding a release open, because "nothing should ever
 wait forever or indefinitely".
 */
export const gateCheckDeadlineMilliseconds = 3_600_000;

/**
 Milliseconds the grace adds to a removal's own bound, which covers stopping a check's client and the wait for its streams
 to close.
 */
const gateStopSlackMilliseconds = 60_000;

/**
 Milliseconds the gate waits past a check's bound before it answers for a check that never settles.

 A check that reaches its bound stops the client that ran its container and removes that container, and that removal may
 spend its own bound (`containerRemovalDeadlineMilliseconds` in `bin/native-gate-container.ts`) before it is stopped. A
 minute more covers stopping the client and waiting for its streams to close, so the gate answers after the refusal the
 check itself reported and after the container stopped at the bound is gone; only a check that ignores its bound at all
 reaches this one.
 */
export const gateStopGraceMilliseconds: number = containerRemovalDeadlineMilliseconds + gateStopSlackMilliseconds;

/**
 One gate run's two checks.

 Each member starts its container when it is called and settles when that container has exited, so a caller that calls
 both before awaiting either runs them beside each other.

 Each is told the bound the gate waits under, so a check that reaches it can stop the container it started instead of
 leaving it running behind a refusal.

 @example
 ```ts
 await runGateChecks({ testRun, clippyRun });
 ```
 */
export type GateChecks = {
  /**
   The whole suite, which throws when any test failed or its container could not run.

   @param deadlineMilliseconds - the bound the gate waits under, which the check gives its own container
   */
  readonly testRun: (deadlineMilliseconds: number) => Promise<unknown>;
  /**
   Clippy over every target, whose outcome is judged by its exit status so the diagnostics it printed can be quoted.

   @param deadlineMilliseconds - the bound the gate waits under, which the check gives its own container
   */
  readonly clippyRun: (deadlineMilliseconds: number) => Promise<CommandOutcome>;
};

/**
 Both checks as they settled, in the order they were started.
 */
type SettledChecks = readonly [
  PromiseSettledResult<unknown>,
  PromiseSettledResult<CommandOutcome>,
];

/**
 The line a gate prints when Clippy passed, so a green run's log holds evidence that Clippy ran at all.

 A clean Clippy prints nothing, and a check that prints nothing is indistinguishable from a check that was skipped.
 */
export const clippyPassedLine = 'Clippy passed beside the test run.';

//endregion What the gate starts

//region Judging the pair

/**
 What one Clippy run printed, for a refusal that quotes it.

 @param outcome - the finished Clippy run

 @returns its standard error when it wrote one, which is where compiler diagnostics go, otherwise its standard output;
   both trimmed, and empty when it wrote neither
 */
function clippyOutputOf(outcome: CommandOutcome,): string {
  /**
   Standard error as the compiler wrote it, without the trailing newline.
   */
  const stderr = outcome.stderr
    .trim();
  if (stderr !== '')
    return stderr;
  return outcome.stdout
    .trim();
}

/**
 The reasons this pair of checks failed, in the order a person reads them.

 @param outcome - both checks as they settled

 @returns one line per failed check, empty when both passed
 */
function failuresOf(outcome: SettledChecks,): readonly string[] {
  /**
   The settled test run and Clippy run, in the order they were started.
   */
  const [tests, clippy] = outcome;
  /**
   The lines that say why the gate refused.
   */
  const failures: string[] = [];
  if (tests.status === 'rejected')
    failures.push(`the test run failed: ${caughtValueText(tests.reason,)}`);
  if (clippy.status === 'rejected') {
    return [
      ...failures,
      `Clippy failed: ${caughtValueText(clippy.reason,)}`,
    ];
  }
  /**
   The Clippy run that exited, whose status and output are what a refusal quotes.
   */
  const clippyOutcome = clippy.value;
  if (clippyOutcome.status !== 0)
    failures.push(`Clippy failed with status ${String(clippyOutcome.status ?? 'none')}: ${clippyOutputOf(clippyOutcome,)}`);
  return failures;
}

/**
 One check as it settles, whichever way it ends.

 The check is started here, so a check that reaches its own bound reports what it found before the gate's timer answers
 for a check that never ends at all.

 @param start - the check, called here with the bound it runs under

 @param deadlineMilliseconds - the bound the gate waits under

 @returns the check's value or failure, never a rejection of its own
 */
async function startedWithinBound<T>({
  start,
  deadlineMilliseconds,
}: {
  readonly start: (deadlineMilliseconds: number) => Promise<T>;
  readonly deadlineMilliseconds: number;
},): Promise<PromiseSettledResult<T>> {
  try {
    return {
      status: 'fulfilled',
      value: await start(deadlineMilliseconds,),
    };
  }
  catch (reason) {
    return {
      status: 'rejected',
      reason,
    };
  }
}

/**
 One check's settlement, produced by the check itself or by the bound the gate waits under.

 @param start - the check, started here before the gate's own timer

 @param name - what the refusal calls this check

 @param deadlineMilliseconds - the bound the gate waits under

 @param stopGraceMilliseconds - how long past that bound the gate waits for a check that never settles

 @returns the check's value or failure, or the failure naming the bound it never came back inside
 */
async function settledWithinBound<T>({
  start,
  name,
  deadlineMilliseconds,
  stopGraceMilliseconds,
}: {
  readonly start: (deadlineMilliseconds: number) => Promise<T>;
  readonly name: string;
  readonly deadlineMilliseconds: number;
  readonly stopGraceMilliseconds: number;
},): Promise<PromiseSettledResult<T>> {
  /**
   The check as it settles, whichever way it ends.
   */
  const started = startedWithinBound({
    start,
    deadlineMilliseconds,
  },);
  /**
   The gate's own answer, used only when the check has not settled by the bound.
   */
  const pastBound = Promise.withResolvers<PromiseSettledResult<T>>();
  /**
   The timer that answers for a check which never ends.
   */
  const timer = setTimeout(
    function refuseAtBound() {
      pastBound.resolve({
        status: 'rejected',
        reason: new NativeVerificationError(`${name} had still not answered ${String(stopGraceMilliseconds)} ms after the ${String(deadlineMilliseconds)} ms bound every gate check is given, so the gate stopped waiting for it: a check that never ends must not hold a release open, and a check that stops at its own bound removes the container it started.`),
      },);
    },
    deadlineMilliseconds + stopGraceMilliseconds,
  );
  /**
   The check's answer, or the gate's own once the bound passed.
   */
  const outcome = await Promise.race([
    started,
    pastBound.promise,
  ],);
  clearTimeout(timer,);
  return outcome;
}

/**
 Start the gate's two checks, wait for both, and fail the gate on either.

 Both are started before either is awaited, and both are waited for even when one failed: a container left running
 behind a failure would hold the image and the host's CPUs that the next gate needs. Neither can hold the gate open: the
 check that failed is reported beside the one that never answered, and the gate itself gives up at the grace after the
 bound it gave them.

 @param testRun - the whole suite, started immediately

 @param clippyRun - Clippy over every target, started immediately and judged by its exit status

 @param deadlineMilliseconds - the bound the gate waits under, which each check is told so it can stop its container

 @param stopGraceMilliseconds - how long past that bound the gate waits for a check which never settles at all

 @throws NativeVerificationError naming every check that failed, with what each printed

 @example
 ```ts
 await runGateChecks({
   testRun: () => runCommand({ command: 'podman', args: ['run', ...limits, image] }),
   clippyRun: () => runCommand({ command: 'podman', args: [...clippyArguments], capture: true, allowFailure: true }),
 });
 ```
 */
export async function runGateChecks({
  testRun,
  clippyRun,
  deadlineMilliseconds = gateCheckDeadlineMilliseconds,
  stopGraceMilliseconds = gateStopGraceMilliseconds,
}: GateChecks & {
  readonly deadlineMilliseconds?: number;
  readonly stopGraceMilliseconds?: number;
},): Promise<void> {
  /**
   Both checks, started by being called here and settled in the order they finish.
   */
  const outcome = await Promise.all([
    settledWithinBound({
      start: testRun,
      name: 'the test run',
      deadlineMilliseconds,
      stopGraceMilliseconds,
    },),
    settledWithinBound({
      start: clippyRun,
      name: 'Clippy',
      deadlineMilliseconds,
      stopGraceMilliseconds,
    },),
  ],);
  /**
   The lines that say why the gate refused, empty when both checks passed.
   */
  const failures = failuresOf(outcome,);
  if (failures.length > 0)
    throw new NativeVerificationError(failures.join('\n',),);
  console.log(clippyPassedLine,);
}

//endregion Judging the pair
