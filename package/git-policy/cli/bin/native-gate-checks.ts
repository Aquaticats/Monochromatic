/**
 The release gate's two checks, Clippy and the whole test suite, started together and judged together.

 The gate started Clippy only after the suite had passed, so a full Clippy compile waited behind a test run about four
 times its length. Nothing orders the two: each starts one container from the same image, reads nothing the other
 writes, and writes nothing the other reads. Started together, the shorter check finishes inside the longer one, and the
 gate still fails on either check alone (`doc/planning/git-policy-cli-cachyos-test-containers.md`).
 */
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  type CommandOutcome,
  NativeVerificationError,
} from './native-verification-process.ts';

//region What the gate starts

/**
 One gate run's two checks.

 Each member starts its container when it is called and settles when that container has exited, so a caller that calls
 both before awaiting either runs them beside each other.

 @example
 ```ts
 await runGateChecks({ testRun, clippyRun });
 ```
 */
export type GateChecks = {
  /**
   The whole suite, which throws when any test failed or its container could not run.
   */
  readonly testRun: () => Promise<unknown>;
  /**
   Clippy over every target, whose outcome is judged by its exit status so the diagnostics it printed can be quoted.
   */
  readonly clippyRun: () => Promise<CommandOutcome>;
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
 Start the gate's two checks, wait for both, and fail the gate on either.

 Both are started before either is awaited, and both are waited for even when one failed: a container left running
 behind a failure would hold the image and the host's CPUs that the next gate needs.

 @param testRun - the whole suite, started immediately

 @param clippyRun - Clippy over every target, started immediately and judged by its exit status

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
}: GateChecks,): Promise<void> {
  /**
   Both checks, started by being called here and settled in the order they finish.
   */
  const outcome = await Promise.allSettled([
    testRun(),
    clippyRun(),
  ]);
  /**
   The lines that say why the gate refused, empty when both checks passed.
   */
  const failures = failuresOf(outcome,);
  if (failures.length > 0)
    throw new NativeVerificationError(failures.join('\n',),);
  console.log(clippyPassedLine,);
}

//endregion Judging the pair
