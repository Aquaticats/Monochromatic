/**
 The btrfs controls a release that is not an alpha waits for: judgement of one run, the gate around it, and the entry
 point a release calls. An alpha release never runs it (the owner's standing rule of 2026-11-11, "make the release
 pipeline avoid running any extra work for alphas"), so the task stays the one a person can repeat on its own.

 A green run is judged by names, not by an exit status alone: the run must print the status line of each control the
 requirements name, so a run that silently skipped them is no evidence (rule `TRV` in `AGENTS.md`). The one `BTRFS
 CONTROL NOT RUN` line a btrfs host does print belongs to a control that needs a filesystem that is not btrfs; any other
 such line refuses the release.

 A failing test that is not one of the required controls is printed as a notice and does not refuse: this gate judges the
 btrfs controls, the container release gate judges the whole suite, and the owner set on 2026-11-10 "Standing rule.
 Alphas don't even have to resolve all flakes." after a run on this busy host failed nine tests outside the controls and
 held a release whose code the container gate had passed.
 */
import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import {
  type HostControlName,
  hostControlLimitSeconds,
  hostControlTask,
  lastLines,
  measuredFullRunDate,
  measuredFullRunSeconds,
  namesIn,
  needsAnotherFilesystem,
  notRunLines,
  passingTag,
  failingTag,
  requiredHostControls,
} from './native-release-host-control-requirements.ts';
import {
  type HostControlRun,
  type HostControlRunner,
  runHostControlsOnThisHost,
} from './native-release-host-control-run.ts';

//region Judging one run

/**
 What the gate decided about one run: how many tests passed, or the lines of a refusal naming every reason it is no
 evidence.
 */
export type HostControlVerdict =
  | {
    readonly kind: 'green';
    readonly testsRun: number;
    /**
     One line per failing test outside the required controls, printed before the pass and never refusing it.
     */
    readonly notices: readonly string[]
  }
    | {
      readonly kind: 'refused';
      readonly lines: readonly string[]
    };

/**
 Exit status of a refused gate.
 */
const refusalExitCode = 1;

/**
 The line that opens every refusal.
 */
const refusalOpening = 'REFUSED the wrapper release: the btrfs controls on this host did not pass, so nothing was dispatched.';

/**
 What a person does next, in the refusal's own words: the cause a host without btrfs filesystems gives, and the dry run
 that skips the gate.
 */
const refusalRemedies: readonly string[] = [
  'A host with no btrfs filesystem has nowhere to run these controls: format a spare partition or loop device with mkfs.btrfs, mount it, and name a directory on that mount when you run the release, as in CLI_GIT_TEST_ROOT=/mnt/the-mounted-btrfs mise run //package/git-policy/cli:native:release -- <from> <to> <note>, then run the same command again.',
  'A --dry-run of the release skips these controls and starts the workflow\'s dry run instead, which proves nothing about them.',
];

/**
 Why a failure outside the required controls does not refuse, in the owner's words, printed once above those notices.
 */
const outsideFailuresNote = 'These tests failed outside the btrfs controls this gate requires, so they do not hold the release: the container release gate judges the whole suite, and the owner set on 2026-11-10 "Standing rule. Alphas don\'t even have to resolve all flakes."';

/**
 Whether a named test is one of the controls the release waits for.

 @param test - a test a status line named

 @returns true for a required control
 */
function isRequiredControl(test: HostControlName): boolean {
  return requiredHostControls.some(function isThisControl(required) {
    return (required.suite === test.suite) && (required.test === test.test);
  });
}

/**
 The failing tests a run named, each once, though nextest prints a failing line again in its summary.

 @param run - the finished run and what it printed

 @returns one entry per distinct failing test
 */
function distinctFailures(run: HostControlRun): readonly HostControlName[] {
  /**
   The failing tests keyed by binary and name, keeping the first line of each.
   */
  const byKey = new Map(namesIn({
    output: run.output,
    tag: failingTag,
  })
    .map(function keyed(test) {
      return [
        `${test.suite} ${test.test}`,
        test
      ] as const;
    }));
  return [...byKey.values()];
}

/**
 The reasons a run that failed says nothing about the controls.

 A failing test outside the required controls is no such reason; {@link noticesOf} names those.

 @param run - the finished run and what it printed

 @returns one line per reason, empty when the run ended cleanly or failed only outside the required controls
 */
function failureReasons(run: HostControlRun): readonly string[] {
  if (run.timedOut)
    return [`The run was stopped after ${String(hostControlLimitSeconds)} seconds, the deadline this gate gives it. The full run took ${String(measuredFullRunSeconds)} seconds on ${measuredFullRunDate}, which is what the deadline is sized from.`];
  if (run.ending
    .kind
    === 'stopped')
    return [`The run was ended by ${run.ending
      .signal} before it finished, so it says nothing about the controls.`];
  if (run.ending
    .status
    === 0)
    return [];
  /**
   The failing tests the run named, if any.
   */
  const failed = distinctFailures(run);
  if (failed.length > 0)
    return failed.filter(isRequiredControl)
      .map(function namesTheControl(test) {
        return `This required control failed: ${test.suite} ${test.test}`;
      });
  return [`The run ended with status ${String(run.ending
    .status)} and named no failing test. Its last lines were: ${lastLines(run.output)
      .join(' | ')}`];
}

/**
 The failing tests outside the required controls, which a green run prints before its pass.

 @param run - the finished run and what it printed

 @returns the note and one line per such test, empty when there are none
 */
function noticesOf(run: HostControlRun): readonly string[] {
  /**
   Failing tests that are not controls the release waits for.
   */
  const outside = distinctFailures(run)
    .filter(function isNotRequired(test) {
      return !isRequiredControl(test);
    });
  if (outside.length === 0)
    return [];
  return [
    outsideFailuresNote,
    ...outside.map(function namesTheTest(test) {
      return `This test failed: ${test.suite} ${test.test}`;
    }),
  ];
}

/**
 Every reason one run is no evidence, in the order a person fixes them.

 @param run - the finished run and what it printed

 @returns one line per reason, empty when the run is evidence
 */
function problemsOf(run: HostControlRun): readonly string[] {
  /**
   The passing tests the run named.
   */
  const passed = namesIn({
    output: run.output,
    tag: passingTag,
  });
  /**
   The controls that printed their own line instead of running, apart from the ones that need another filesystem.
   */
  const skipped = notRunLines(run.output)
    .filter(function needsBtrfs(line) {
      return !needsAnotherFilesystem(line);
    })
    .map(function quotesTheLine(line) {
      return `This control did not run: ${line}`;
    });
  return [
    ...failureReasons(run),
    ...skipped,
    ...requiredHostControls.flatMap(function missingReason(required) {
      /**
       Whether the run printed a passing status line for this control.
       */
      const ran = passed.some(function isTheRequiredControl(test) {
        return (test.suite === required.suite) && (test.test === required.test);
      });
      if (ran)
        return [];
      return [`The run printed no passing status line for ${required.suite} ${required.test}, so this run is not evidence that the control ran.`];
    }),
  ];
}

/**
 How many tests the passing status lines name, counted once each.

 nextest prints a passing test's status line twice in a run it reports at the end: once when the test finishes, and again
 with the summary. Counting lines would report a full run of 5993 tests as the 11960 lines of its log.

 @param passed - one entry per passing status line

 @returns the number of distinct tests

 @example
 ```ts
 distinctTestCount([{ suite: 'a', test: 'b' }, { suite: 'a', test: 'b' }]);
 ```
 */
function distinctTestCount(passed: readonly HostControlName[]): number {
  return new Set(passed.map(function keyOf(test) {
    return `${test.suite} ${test.test}`;
  })).size;
}

/**
 Decide whether one run is the evidence a release waits for.

 @param run - the finished run and what it printed

 @returns green with the number of tests that passed, or the lines of a refusal naming every reason

 @example
 ```ts
 judgeHostControlRun({ ending: { kind: 'exited', status: 0 }, output, timedOut: false });
 ```
 */
export function judgeHostControlRun(run: HostControlRun): HostControlVerdict {
  /**
   Every reason this run is no evidence.
   */
  const problems = problemsOf(run);
  if (problems.length > 0) {
    return {
      kind: 'refused',
      lines: [
        refusalOpening,
        ...problems,
        ...refusalRemedies,
      ],
    };
  }
  return {
    kind: 'green',
    testsRun: distinctTestCount(
      namesIn({
        output: run.output,
        tag: passingTag,
      }),
    ),
    notices: noticesOf(run),
  };
}

//endregion Judging one run

//region Running the gate

/**
 The line that says what the release waits for and what that run costs.

 @returns the line, which names the task, the deadline and the run the deadline is sized from
 */
function waitingLine(): string {
  return `Waiting for the btrfs controls on this host: mise run ${hostControlTask} (deadline ${String(hostControlLimitSeconds)} seconds; the full run took ${String(measuredFullRunSeconds)} seconds on ${measuredFullRunDate}).`;
}

/**
 Print what the gate decided about one run and answer the exit status it earns.

 @param run - the finished run and what it printed

 @param print - prints one line

 @returns zero when every required control passed, the refusal status otherwise
 */
function reportVerdict({
  run,
  print,
}: {
  readonly run: HostControlRun;
  readonly print: (line: string) => void;
}): number {
  /**
   What the gate made of the run.
   */
  const verdict = judgeHostControlRun(run);
  if (verdict.kind === 'green') {
    for (const notice of verdict.notices)
      print(notice);
    print(`BTRFS CONTROLS PASSED ${String(verdict.testsRun)} tests ran here, and the ${String(requiredHostControls.length)} controls this gate requires each printed their passing status line.`);
    return 0;
  }
  for (const line of verdict.lines)
    print(line);
  return refusalExitCode;
}

/**
 Say what the gate waits for, run the controls, and print the verdict.

 @param filters - test name filters, which a person iterating narrows the run with and a release never gives

 @param runHostControls - starts one run and reports how it ended

 @param print - prints one line

 @returns zero when every required control passed, the refusal status otherwise

 @example
 ```ts
 await runHostControlGate({ filters: [], runHostControls, print: console.log });
 ```
 */
export async function runHostControlGate({
  filters,
  runHostControls,
  print,
}: {
  readonly filters: readonly string[];
  readonly runHostControls: HostControlRunner;
  readonly print: (line: string) => void;
}): Promise<number> {
  print(waitingLine());
  try {
    return reportVerdict({
      run: await runHostControls(filters),
      print,
    });
  }
  catch (error) {
    print(refusalOpening);
    print(`The run could not start: ${caughtValueText(error)}`);
    for (const remedy of refusalRemedies)
      print(remedy);
    return refusalExitCode;
  }
}

//endregion Running the gate

//region Main execution

/**
 Print one line of the gate's own output, which is the product here and not a log record.

 @param line - the line to print
 */
function printLine(line: string): void {
  console.log(line);
}

/**
 Start the run on this host with the deadline the requirements ship.

 @param filters - test name filters for a narrowed run

 @returns how the run ended and what it printed
 */
function runHostControlsFromThisHost(filters: readonly string[]): Promise<HostControlRun> {
  return runHostControlsOnThisHost({
    filters,
    limitSeconds: hostControlLimitSeconds,
  });
}

/**
 Run the gate as the release calls it: every argument is a test filter, and the exit status reports the verdict.
 */
async function main(): Promise<void> {
  process.exitCode = await runHostControlGate({
    filters: process.argv
      .slice(2)
      .filter(function notTheSeparator(argument) {
        return argument !== '--';
      }),
    runHostControls: runHostControlsFromThisHost,
    print: printLine,
  });
}

if (import.meta.main)
  await main();

//endregion Main execution
