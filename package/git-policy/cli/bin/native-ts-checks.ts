#!/usr/bin/env node
/**
 Run the TypeScript checks of the packages a change touched, each in its own capped systemd scope, all at once, and print
 one line per check with its status and, when it failed, the lines that say why.

 This is what `native:ts-checks` runs, and it folds the coordinator's scratch script `~/temp/agent/gate/ts-checks.ts` into
 the package. That script named two packages, the wrapper and the mutation runner, because it was written for one
 integration branch; this command names the checks of this package by default and takes any other task name as a
 positional, so the same run serves any change.

 The bounds the scratch script had are kept and stated: each check runs in a scope of its own with a CPU quota, a memory
 cap, a task limit and a disk weight, and every check now has a deadline, which the scratch script did not have.
 */
import {
  taskRunnerProgram,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/constants.ts';

import {
  cpuQuotaOption,
  deadlineMillisecondsOption,
  gateOptions,
  ioWeightOption,
  memoryMaxOption,
  numberOption,
  tasksMaxOption,
  textOption,
} from './native-gate-arguments.ts';
import {
  type ScopeCaps,
  checkFailedExitCode,
  checkScopeName,
  checkedScopeCaps,
  defaultCheckScopeCaps,
  runInCappedScope,
  type ScopeOutcome,
} from './native-capped-scope-command.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  printLine,
  runGateMain,
} from './native-gate-main.ts';
import {
  gitDeadlineMilliseconds,
  gitProgram,
  promptlessGitSettings,
} from './native-gate-paths.ts';
import { runGateCommand } from './native-gate-run.ts';
import { localStamp } from './native-gate-unit-name.ts';

/**
 The checks this package runs when the caller names none: its own unit tests and its own type checks, which are the two
 tasks a change to `bin/` or `src/` has to pass before it is committed.
 */
export const recordedCheckTasks: readonly string[] = [
  '//package/git-policy/cli:test:unit',
  '//package/git-policy/cli:lint:types',
];

/**
 Seconds one check is given when the caller names none.

 The TypeScript unit run of this package and its type check are the two this command starts by default; one full run of
 both on this host on 2026-11-11 took 91 seconds (13:46:48 to 13:48:19, `mise run //package/git-policy/cli:native:ts-checks`),
 so fifteen minutes is ten times that and a check still running then is one that will not answer. `--deadline-seconds`
 overrides it.
 */
export const defaultCheckDeadlineSeconds = 900;

/**
 How many lines of a failing check's own failure lines are kept, which is the bound the scratch script had.
 */
const failureLineLimit = 12;

/**
 How many of a failing check's last lines are kept, the tail that names what a check was doing.
 */
const tailLineLimit = 12;

/**
 Characters a scope label keeps: lowercase letters and digits, with hyphens between the words.
 */
const labelCharacters = 'abcdefghijklmnopqrstuvwxyz0123456789';

/**
 Longest label a scope name carries, which leaves the prefix and the stamp room in a unit name systemd reads.

 A task name longer than this is cut, which keeps two long task names of one package apart by their own words rather than
 by a counter.
 */
const labelLimit = 48;

/**
 The lines that say why a check did not pass, as the scratch script recognised them: a failure, a type error, a test
 harness failure, or a line saying a test did not pass.
 */
const failureLineMarkers: readonly string[] = [
  'FAIL',
  'error TS',
  '✖',
  'not ok',
];

/**
 Option that names the checkout the checks run in.
 */
const worktreeOption = '--worktree';

/**
 Option that names how long one check may run.
 */
const deadlineOption = '--deadline-seconds';

/**
 How the command is called, printed by every refusal that reads the command line.
 */
const usage = 'usage: native:ts-checks [--worktree <path>] [--deadline-seconds <count>] [--unit-cpu-quota <value>] [--unit-memory-max <value>] [--unit-tasks-max <count>] [--unit-io-weight <weight>] [<task>...]';

/**
 One character of a label: what it is, or a hyphen when the task name spells it another way.

 @param character - the character to read

 @returns the character, lowercased, or a hyphen

 @example
 ```ts
 labelCharacter({ character: '/' });
 ```
 */
function labelCharacter({ character }: { readonly character: string; }): string {
  /**
   The character in the case a unit name is written in.
   */
  const lowered = character.toLowerCase();
  return labelCharacters.includes(lowered) ? lowered : '-';
}

/**
 The label one task's scope carries: the task name in lowercase with every other character a hyphen, which is what a
 systemd unit name holds.

 @param task - the task as it is written on the command line

 @returns the label, never empty and never longer than {@link labelLimit}

 @example
 ```ts
 scopeLabelOfTask({ task: '//package/git-policy/cli:test:unit' });
 ```
 */
export function scopeLabelOfTask({ task }: { readonly task: string; }): string {
  /**
   The label's characters, with a run of unusable ones written once.
   */
  const characters: string[] = [];
  for (const character of task) {
    /**
     What this character is in a label.
     */
    const written = labelCharacter({ character, });
    // A separator is written only between two words, so no label starts or ends with one.
    if ((written === '-') && ((characters.length === 0) || (characters.at(-1) === '-')))
      continue;
    characters.push(written);
  }
  while (characters.at(-1) === '-')
    characters.pop();
  /**
   The label, cut to the length a unit name leaves for it.
   */
  const label = characters.slice(
    0,
    labelLimit
  )
    .join('');
  if (label === '')
    throw new NativeGateError(`the task "${task}" holds nothing a scope name can be made of, so the check cannot be named.`);
  return label;
}

/**
 One check this run starts: the task as the command line named it, and the label its scope carries.

 The label is what {@link scopeLabelOfTask} makes of the task, and a second task that writes the same label gets its
 position appended: two task names longer than the label limit, or two spellings that differ only in case, would
 otherwise ask systemd for the same scope and the second start would be refused.
 */
export type NamedCheck = {
  /**
   The task as the command line named it.
   */
  readonly task: string;
  /**
   The label its scope carries, unique among the checks of one run.
   */
  readonly label: string;
};

/**
 Pairs every task with the label its scope carries, made unique within the run.

 @param tasks - the tasks the caller named, or this package's own

 @returns one entry per task, in the order the caller named them

 @example
 ```ts
 namedChecks({ tasks: ['//package/a:b', '//package/a:b'] });
 ```
 */
export function namedChecks({ tasks }: { readonly tasks: readonly string[]; }): readonly NamedCheck[] {
  /**
   How often each label has been used so far, which is what a repeat is numbered from.
   */
  const used = new Map<string, number>();
  return tasks.map(function named(task) {
    /**
     The label the task's own words make.
     */
    const label = scopeLabelOfTask({ task, });
    /**
     How often this label has been used, this task included.
     */
    const count = (used.get(label) ?? 0) + 1;
    used.set(
      label,
      count
    );
    return {
      task,
      label: count === 1 ? label : `${label}-${String(count)}`,
    };
  });
}

/**
 The lines of a check's output that say why it did not pass.

 @param output - everything the check printed, on both streams

 @returns the failure lines and then the tail, at most twice {@link failureLineLimit} lines

 @example
 ```ts
 failureLinesOf({ output: 'error TS2322: Type string is not assignable\n' });
 ```
 */
export function failureLinesOf({ output }: { readonly output: string; }): readonly string[] {
  /**
   The output's lines.
   */
  const lines = output.split('\n');
  /**
   The lines that name a failure.
   */
  const failures = lines.filter(function namesFailure(line) {
    return failureLineMarkers.some(function carries(marker) {
      return line.includes(marker);
    });
  })
    .slice(
    0,
    failureLineLimit
    );
  return [
    ...failures,
    ...lines.slice(-tailLineLimit),
  ];
}

/**
 One check this command runs and what came of it.
 */
type CheckRun = {
  /**
   The task as the command line named it.
   */
  readonly task: string;
  /**
   The scope the check ran in.
   */
  readonly scope: string;
  /**
   How it ended.
   */
  readonly outcome: ScopeOutcome;
};

/**
 The lines one check's result prints.

 @param check - the check and its outcome

 @returns the status line, and for a check that did not pass the lines that say why

 @example
 ```ts
 resultLinesOf({ check });
 ```
 */
export function resultLinesOf({ check }: { readonly check: CheckRun; }): readonly string[] {
  if (check.outcome
    .timedOut)
    return [
      `${check.task}: stopped at its deadline, so the scope ${check.scope} was ended`,
      ...failureLinesOf({ output: `${check.outcome
        .stdout}\n${check.outcome
          .stderr}`, }),
    ];
  if (check.outcome
    .status
    === 0)
    return [`${check.task}: exit 0`];
  return [
    `${check.task}: exit ${String(check.outcome
      .status)}`,
    ...failureLinesOf({ output: `${check.outcome
      .stdout}\n${check.outcome
        .stderr}`, }),
  ];
}

/**
 Reads the caps the caller asked for, check by check, so a refused value names its own option.

 @param options - the parsed command line

 @returns the caps, checked against what systemd can read

 @throws NativeGateError when a cap is not a value systemd reads

 @example
 ```ts
 checkCapsFromOptions({ options });
 ```
 */
function checkCapsFromOptions({ options }: { readonly options: ReturnType<typeof gateOptions>; }): ScopeCaps {
  return checkedScopeCaps({
    caps: {
      unitCpuQuota: textOption({
        options,
        name: cpuQuotaOption,
        fallback: defaultCheckScopeCaps.unitCpuQuota,
      }),
      unitMemoryMax: textOption({
        options,
        name: memoryMaxOption,
        fallback: defaultCheckScopeCaps.unitMemoryMax,
      }),
      unitTasksMax: numberOption({
        options,
        name: tasksMaxOption,
        fallback: defaultCheckScopeCaps.unitTasksMax,
      }),
      unitIoWeight: numberOption({
        options,
        name: ioWeightOption,
        fallback: defaultCheckScopeCaps.unitIoWeight,
      }),
    },
  });
}

/**
 The commit the checkout the checks run in points at, or empty text when it has none.

 A throwaway checkout of a test is not a commit, and neither is a folder outside every checkout; the answer is printed
 beside the checks so a log says what was checked when the folder name does not.

 @param worktree - the checkout the checks run in

 @returns the short commit, or empty text

 @example
 ```ts
 await commitOf({ worktree: '/repo/.claude/worktrees/a' });
 ```
 */
async function commitOf({ worktree }: { readonly worktree: string; }): Promise<string> {
  /**
   What `git rev-parse` answered.
   */
  const answered = await runGateCommand({
    program: gitProgram,
    args: [
      '-C',
      worktree,
      'rev-parse',
      '--short=10',
      'HEAD',
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: {
      ...process.env,
      ...promptlessGitSettings,
    },
  });
  return answered.status === 0 ? answered.stdout
    .trim() : '';
}

/**
 Run every check at once and print one line per check, in the order the caller named them.

 @returns zero when every check passed, {@link checkFailedExitCode} when one did not or ran past its deadline

 @throws NativeGateError when the command line, a cap or the checkout cannot start a check

 @example
 ```ts
 process.exitCode = await runGateMain({ main });
 ```
 */
async function main(): Promise<number> {
  /**
   The command line, whose positionals are the tasks to run in place of this package's own.
   */
  const options = gateOptions({
    argv: process.argv
      .slice(2),
    knownOptions: [
      worktreeOption,
      deadlineOption,
      cpuQuotaOption,
      memoryMaxOption,
      tasksMaxOption,
      ioWeightOption,
    ],
  });
  /**
   The checkout the checks run in.
   */
  const worktree = textOption({
    options,
    name: worktreeOption,
    fallback: process.cwd(),
  });
  /**
   How long one check may run.
   */
  const deadlineMilliseconds = deadlineMillisecondsOption({
    options,
    name: deadlineOption,
    fallbackSeconds: defaultCheckDeadlineSeconds,
  });
  /**
   The caps every check's scope carries.
   */
  const caps = checkCapsFromOptions({ options, });
  /**
   The tasks to run, which are this package's own when the caller names none.
   */
  const tasks = (options.positionals
    .length
    === 0) ? recordedCheckTasks : options.positionals;
  /**
   Each task with the label its scope carries, which no two checks of one run share.
   */
  const named = namedChecks({ tasks, });
  /**
   The moment every scope name is stamped with, so the checks of one run are one run.
   */
  const moment = new Date();
  /**
   Every check's task, scope and outcome, in the order the caller named them.
   */
  const checks = await Promise.all(named.map(async function checkOf(one): Promise<CheckRun> {
    /**
     The scope this check runs in.
     */
    const scope = checkScopeName({
      label: one.label,
      stamp: localStamp({ moment, }),
    });
    return {
      task: one.task,
      scope,
      outcome: await runInCappedScope({
        name: scope,
        caps,
        program: taskRunnerProgram,
        args: [
          'run',
          one.task,
        ],
        deadlineMilliseconds,
        workingDirectory: worktree,
      }),
    };
  }));
  /**
   The commit the checkout points at, printed so a log says what was checked.
   */
  const commit = await commitOf({ worktree, });
  printLine(`${worktree}${commit === '' ? '' : ` at ${commit}`}`);
  for (const check of checks)
    for (const line of resultLinesOf({ check, }))
      printLine(line);
  return checks.every(function passed(check) {
    return (check.outcome
      .status
      === 0) && (!check.outcome
        .timedOut);
  })
    ? 0
    : checkFailedExitCode;
}

if (import.meta.main)
  process.exitCode = await runGateMain({ main, });
