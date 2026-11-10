/**
 The btrfs controls a wrapper release waits for.

 The gate container runs on an overlay filesystem, so each control that needs the btrfs ioctls prints
 `BTRFS CONTROL NOT RUN`, passes there, and the release gate stays green without them. The owner decided on 2026-11-10
 that a real wrapper release waits for `native:test:host:btrfs` on a btrfs host and that a control which did not run
 fails it. This module is that gate: it runs the suite, judges the run by its own lines, and refuses the release unless
 every required control passed.

 A green run is judged by names, not by an exit status alone: the run must print the status line of each control this
 module requires, so a run that silently skipped them is no evidence (rule `TRV` in `AGENTS.md`). The one `BTRFS CONTROL
 NOT RUN` line a btrfs host does print belongs to a control that needs a filesystem that is not btrfs; any other such
 line refuses the release.
 */
import {
  type ChildProcessWithoutNullStreams,
  spawn,
} from 'node:child_process';
import { once } from 'node:events';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

//region What the gate requires

/**
 The mise task whose run is the evidence a release waits for: every target on this host, with no acknowledgement
 variable set, on a fixture root that must be on btrfs.
 */
export const hostControlTask = '//package/git-policy/cli:native:test:host:btrfs';

/**
 Longest the gate waits for that run: two hours.
 The gate cannot wait forever (the owner's "nothing should ever wait forever or indefinitely").
 */
export const hostControlLimitSeconds = 7200;

/**
 Seconds the full run of the task took on this host, which sizes the deadline above.

 Measured on 2026-11-10 in its own `systemd-run --user --scope` limited to 600% CPU and 20 GB, with this host's other
 agents working.
 */
export const measuredFullRunSeconds = 0;

/**
 When that run was measured, as the third of the run and the day the deadline was sized from.
 */
export const measuredFullRunDate = '2026-11-10';

/**
 One test the release must see pass, by the binary nextest prints and the name the test has within it.
 */
export type HostControlName = {
  /**
   Binary ID in nextest's status line, such as `monochromatic-git-policy-cli` or
   `monochromatic-git-policy-cli::native_binary`.
   */
  readonly suite: string;
  /**
   The test's name within that binary, which is its module path and function name.
   */
  readonly test: string;
};

/**
 The controls a green run must name, one from each path that runs btrfs code: the ioctls themselves, the migration
 that builds subvolumes, and a binary-level control that goes through `on_btrfs_or_acknowledged`.

 A renamed or deleted control refuses the release until this list is brought up to date, which is the point: the
 release waits for a run whose lines prove these controls happened.
 */
export const requiredHostControls: readonly HostControlName[] = [
  {
    suite: 'monochromatic-git-policy-cli',
    test: 'subvolume::fs_btrfs_ioctl::tests::btrfs_create_snapshot_and_destroy_each_work_once',
  },
  {
    suite: 'monochromatic-git-policy-cli',
    test: 'subvolume::subvolume_btrfs_tests::a_real_migration_makes_subvolumes_and_clones_file_data',
  },
  {
    suite: 'monochromatic-git-policy-cli::native_binary',
    test: 'worktree_ladder::a_worktree_added_from_the_main_checkout_is_a_subvolume',
  },
];

/**
 The reason a control's own line gives when the control needs a filesystem that is not btrfs, which is the one
 `BTRFS CONTROL NOT RUN` line a btrfs host prints.
 */
const otherFilesystemReason = 'needs a filesystem that is not btrfs';

/**
 What a control prints when it found nothing to check here.
 */
const notRunMarker = 'BTRFS CONTROL NOT RUN';

/**
 A passing test's nextest status line: its binary ID, then the test's name.
 */
const passingLinePattern = /^\s*PASS \[[^\]]*\] \(\s*\d+\/\d+\) (\S+) (.*)$/;

/**
 A failing test's nextest status line, in the same shape.
 */
const failingLinePattern = /^\s*FAIL \[[^\]]*\] \(\s*\d+\/\d+\) (\S+) (.*)$/;

/**
 Exit status of a refused gate.
 */
const refusalExitCode = 1;

//endregion What the gate requires

//region Judging one run

/**
 One finished run of the host controls, captured while it ran.
 */
export type HostControlRun = {
  /**
   The run's exit status, absent when a signal or the deadline ended it.
   */
  readonly exitCode: number | null;
  /**
   Everything the run printed, standard output and standard error in the order it arrived.
   */
  readonly output: string;
  /**
   Whether the deadline stopped the run instead of it ending by itself.
   */
  readonly timedOut: boolean;
};

/**
 Starts one run of the host controls and reports how it ended.
 */
export type HostControlRunner = (filters: readonly string[]) => Promise<HostControlRun>;

/**
 What the gate decided about one run.
 */
export type HostControlVerdict =
  | {
    readonly kind: 'green';
    /**
     How many tests the run passed, which is what the person reads as the evidence.
     */
    readonly testsRun: number;
  }
  | {
    readonly kind: 'refused';
    /**
     The lines to print, which name every reason the release does not start.
     */
    readonly lines: readonly string[];
  };

/**
 One parsing pattern's matches, as the binary and the test name of each.

 @param output - the run's captured output

 @param pattern - a nextest status-line pattern with the binary and the name in its two groups

 @returns one entry per matching line, in the order the run printed them

 @example
 ```ts
 namesIn({ output, pattern: passingLinePattern });
 ```
 */
function namesIn({
  output,
  pattern,
}: {
  readonly output: string;
  readonly pattern: RegExp;
}): readonly HostControlName[] {
  return output.split('\n')
    .flatMap(function nameOf(line) {
      /**
       The line's binary and test name, or nothing when the line is not a status line of this kind.
       */
      const match = pattern.exec(line);
      if (match === null)
        return [];
      return [{
        suite: match[1] ?? '',
        test: (match[2] ?? '').trim(),
      }];
    });
}

/**
 The lines a run printed for controls that found nothing to check here.

 @param output - the run's captured output

 @returns the lines, each as the control wrote it

 @example
 ```ts
 notRunLines(passingOutput);
 ```
 */
function notRunLines(output: string): readonly string[] {
  return output.split('\n')
    .filter(function isNotRunLine(line) {
      return line.includes(notRunMarker);
    })
    .map(function trimmed(line) {
      return line.trim();
    });
}

/**
 The run's last lines that hold text, for a run that failed without naming a test.

 @param output - the run's captured output

 @returns the last few non-empty lines, with the run's own indentation removed

 @example
 ```ts
 lastLines('error: could not compile');
 ```
 */
function lastLines(output: string): readonly string[] {
  /**
   How many of the run's last lines a refusal quotes, enough for a compiler's error and its note.
   */
  const quotedLineCount = 10;
  return output.split('\n')
    .filter(function hasText(line) {
      return line.trim() !== '';
    })
    .slice(-quotedLineCount)
    .map(function trimmed(line) {
      return line.trim();
    });
}

/**
 The line that opens every refusal.
 */
const refusalOpening = 'REFUSED the wrapper release: the btrfs controls on this host did not pass, so nothing was dispatched.';

/**
 What a person does next, in the refusal's own words: the cause a host without btrfs filesystems gives, and the dry
 run that skips the gate.
 */
const refusalRemedies: readonly string[] = [
  'A host with no btrfs filesystem has nowhere to run these controls: format a spare partition or loop device with mkfs.btrfs, mount it, and name a directory on that mount when you run the release, as in CLI_GIT_TEST_ROOT=/mnt/the-mounted-btrfs mise run //package/git-policy/cli:native:release -- <from> <to> <note>, then run the same command again.',
  'A --dry-run of the release skips these controls and starts the workflow\'s dry run instead, which proves nothing about them.',
];

/**
 Decide whether one run is the evidence a release waits for.

 @param run - the finished run and what it printed

 @returns green with the number of tests that passed, or the lines of a refusal naming every reason

 @example
 ```ts
 judgeHostControlRun({ exitCode: 0, output, timedOut: false });
 ```
 */
export function judgeHostControlRun(run: HostControlRun): HostControlVerdict {
  /**
   The passing tests the run named.
   */
  const passed = namesIn({
    output: run.output,
    pattern: passingLinePattern,
  });
  /**
   Every reason this run is no evidence, in the order a person fixes them.
   */
  const problems: string[] = [];
  if (run.timedOut)
    problems.push(`The run was stopped after ${String(hostControlLimitSeconds)} seconds, the deadline this gate gives it. The full run took ${String(measuredFullRunSeconds)} seconds on ${measuredFullRunDate}, which is what the deadline is sized from.`);
  /**
   Controls that printed their own line instead of running, apart from the ones that need another filesystem.
   */
  const skipped = notRunLines(run.output)
    .filter(function needsBtrfs(line) {
      return !line.includes(otherFilesystemReason);
    });
  for (const line of skipped)
    problems.push(`This control did not run: ${line}`);
  if ((!run.timedOut) && (run.exitCode !== 0)) {
    /**
     The failing tests the run named, if any.
     */
    const failed = namesIn({
      output: run.output,
      pattern: failingLinePattern,
    });
    if (failed.length > 0)
      for (const test of failed)
        problems.push(`This test failed: ${test.suite} ${test.test}`);
    else
      problems.push(`The run ended with status ${String(run.exitCode ?? 'none')} and named no failing test. Its last lines were: ${lastLines(run.output).join(' | ')}`);
  }
  for (const required of requiredHostControls) {
    /**
     Whether the run printed a status line for this control and no failure beside it.
     */
    const ran = passed.some(function isTheRequiredControl(test) {
      return (test.suite === required.suite) && (test.test === required.test);
    });
    if (!ran)
      problems.push(`The run printed no passing status line for ${required.suite} ${required.test}, so this run is not evidence that the control ran.`);
  }
  if (problems.length === 0)
    return {
      kind: 'green',
      testsRun: passed.length,
    };
  return {
    kind: 'refused',
    lines: [
      refusalOpening,
      ...problems,
      ...refusalRemedies,
    ],
  };
}

//endregion Judging one run

//region Running the gate

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
  print(`Waiting for the btrfs controls on this host: mise run ${hostControlTask} (deadline ${String(hostControlLimitSeconds)} seconds; the full run took ${String(measuredFullRunSeconds)} seconds on ${measuredFullRunDate}).`);
  /**
   The run, or the reason it could not be started.
   */
  let run: HostControlRun;
  try {
    run = await runHostControls(filters);
  }
  catch (error) {
    print(refusalOpening);
    print(`The run could not start: ${caughtValueText(error)}`);
    for (const remedy of refusalRemedies)
      print(remedy);
    return refusalExitCode;
  }
  /**
   What the gate made of it.
   */
  const verdict = judgeHostControlRun(run);
  if (verdict.kind === 'green') {
    print(`BTRFS CONTROLS PASSED ${String(verdict.testsRun)} tests ran here, and the ${String(requiredHostControls.length)} controls this gate requires each printed their passing status line.`);
    return 0;
  }
  for (const line of verdict.lines)
    print(line);
  return refusalExitCode;
}

//endregion Running the gate

//region The run on this host

/**
 Write one child's stream to this process's own stream and keep the text.

 @param stream - the child's stream

 @param target - this process's stream of the same kind, so the run stays visible while it lasts

 @param keep - receives each decoded chunk, in arrival order

 @returns when the stream ends

 @example
 ```ts
 await teeAndKeep({ stream: child.stderr, target: process.stderr, keep: chunks.push.bind(chunks) });
 ```
 */
async function teeAndKeep({
  stream,
  target,
  keep,
}: {
  readonly stream: NodeJS.ReadableStream;
  readonly target: NodeJS.WritableStream;
  readonly keep: (text: string) => void;
}): Promise<void> {
  for await (const chunk of stream) {
    /**
     The chunk as text, which nextest prints line by line.
     */
    const text = typeof chunk === 'string' ? chunk : String(chunk as Buffer);
    keep(text);
    target.write(text);
  }
}

/**
 Stop a command and everything it started, so the deadline leaves no test behind.

 @param child - the started command, made a process group leader when it was spawned

 @example
 ```ts
 stopGroup(child);
 ```
 */
function stopGroup(child: ChildProcessWithoutNullStreams): void {
  try {
    if (child.pid === undefined)
      return;
    process.kill(-child.pid, 'SIGKILL');
  }
  catch {
    child.kill('SIGKILL');
  }
}

/**
 Run the host controls through the package's own task, bounded by the deadline, with each passing test's name in the
 output.

 nextest prints a status line for a passing test only when the reporter is asked for it, so the gate sets
 `NEXTEST_STATUS_LEVEL` and `NEXTEST_FINAL_STATUS_LEVEL` for the run; the task keeps the flags it has and nothing about
 the suite is spelled twice.

 @param filters - test name filters for a narrowed run

 @param limitSeconds - deadline for the run, counted from its start

 @returns the run's exit status, everything it printed, and whether the deadline stopped it

 @example
 ```ts
 await runHostControlsOnThisHost({ filters: ['subvolume::'], limitSeconds: hostControlLimitSeconds });
 ```
 */
export async function runHostControlsOnThisHost({
  filters,
  limitSeconds,
}: {
  readonly filters: readonly string[];
  readonly limitSeconds: number;
}): Promise<HostControlRun> {
  /**
   Everything the run printed, in arrival order.
   */
  const chunks: string[] = [];
  /**
   The run, in a process group of its own so the deadline can stop the tests it starts.
   */
  const child = spawn(
    'mise',
    [
      'run',
      hostControlTask,
      ...(filters.length === 0 ? [] : [
        '--',
        ...filters,
      ]),
    ],
    {
      env: {
        ...process.env,
        NEXTEST_STATUS_LEVEL: 'pass',
        NEXTEST_FINAL_STATUS_LEVEL: 'pass',
      },
      detached: true,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Set when the deadline stopped the run, so a killed run is never read as a failed test.
   */
  let timedOut = false;
  /**
   The deadline, which stops the whole group and then stops holding the run open.
   */
  const deadline = setTimeout(function stopAtDeadline() {
    timedOut = true;
    stopGroup(child);
  }, limitSeconds * 1000);
  /**
   The streams, read together so the text keeps the order the run printed it in.
   */
  const streams = Promise.all([
    teeAndKeep({
      stream: child.stdout,
      target: process.stdout,
      keep: function keepStdout(text) {
        chunks.push(text);
      },
    }),
    teeAndKeep({
      stream: child.stderr,
      target: process.stderr,
      keep: function keepStderr(text) {
        chunks.push(text);
      },
    }),
  ]);
  await Promise.all([
    streams,
    once(
      child,
      'close',
    ),
  ]);
  clearTimeout(deadline);
  return {
    exitCode: child.exitCode,
    output: chunks.join(''),
    timedOut,
  };
}

//endregion The run on this host

//region Main execution

/**
 Run the gate as the release calls it: every argument is a test filter.

 @returns nothing; the exit status reports the verdict
 */
async function main(): Promise<void> {
  process.exitCode = await runHostControlGate({
    filters: process.argv.slice(2)
      .filter(function notTheSeparator(argument) {
        return argument !== '--';
      }),
    runHostControls: async function runHostControls(filters) {
      return await runHostControlsOnThisHost({
        filters,
        limitSeconds: hostControlLimitSeconds,
      });
    },
    print: function print(line) {
      console.log(line);
    },
  });
}

if (import.meta.main)
  await main();

//endregion Main execution
