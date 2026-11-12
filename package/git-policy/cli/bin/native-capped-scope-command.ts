/**
 One command under its own capped systemd user scope, with a deadline that really ends it.

 This is what the package's check runner needs and what the coordinator's scratch script did with a bare `systemd-run
 --user --scope`: the work of a check is heavy, so the control group bounds its threads, memory, tasks and disk weight
 (`AGENTS.md`, rules `RXI` and `RXS`), and nothing here may wait forever, so a command still running at its deadline is
 killed rather than waited on.

 A scope started with `--scope` runs the command as a child of `systemd-run`, so killing that client leaves the command
 running under systemd. The scope therefore carries a name of its own and the deadline ends it by that name, which was
 measured on this host on 2026-11-11: `systemctl --user kill --signal=SIGKILL --kill-whom=all <name>.scope` turned an
 active `sleep 40` scope inactive, and `stop` on an unloaded scope answered "Unit ... not loaded." with status 5.
 */
import { randomUUID } from 'node:crypto';

import { systemdRunProgram } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/constants.ts';

import {
  maximumUnitIoWeight,
  textCapProblem,
  wholeCapProblem,
} from './native-gate-caps.ts';
import { NativeGateError } from './native-gate-error.ts';
import { runGateCommand } from './native-gate-run.ts';

/**
 Prefix of every scope name a check carries, so `systemctl --user list-units 'gpc-check-*'` finds what is left of a run.
 */
export const checkScopeNamePrefix = 'gpc-check';

/**
 The program that ends a scope by name, which killing the client alone does not do.
 */
export const systemctlProgram = 'systemctl';

/**
 Milliseconds a stop of one scope is given, which was milliseconds on this host and is bounded so a busy manager cannot
 hold the runner.
 */
const scopeStopDeadlineMilliseconds = 60_000;

/**
 The caps of one check's scope: what its control group may use.

 The container's process limit is absent on purpose: a check starts no container, so its own `TasksMax` is the bound.
 */
export type ScopeCaps = {
  /**
   `CPUQuota` of the scope, as systemd reads it.
   */
  readonly unitCpuQuota: string;
  /**
   `MemoryMax` of the scope, as systemd reads it.
   */
  readonly unitMemoryMax: string;
  /**
   `TasksMax` of the scope, in tasks, which a control group counts as threads and processes together.
   */
  readonly unitTasksMax: number;
  /**
   `IOWeight` of the scope, from one to ten.
   */
  readonly unitIoWeight: number;
};

/**
 The caps every recorded run of the coordinator's TypeScript check script used, kept so a log of an earlier run and one of
 the next are comparable.

 The first three are that script's own values (`CPUQuota=200%`, `MemoryMax=4G`, `TasksMax=400`). Its disk weight was
 thirty, which is above the ten a per-command control group may carry (rule `RXS`), so it is one here.
 */
export const defaultCheckScopeCaps: ScopeCaps = {
  unitCpuQuota: '200%',
  unitMemoryMax: '4G',
  unitTasksMax: 400,
  unitIoWeight: 1,
};

/**
 Why one scope's caps cannot be used, one line each, empty when they can.

 Checked before anything starts, because a cap systemd cannot read fails the start with a message about a property rather
 than about the check.

 @param caps - the caps to check

 @returns one line per refusal, in the order the options are listed

 @example
 ```ts
 scopeCapsProblems({ caps: defaultCheckScopeCaps });
 ```
 */
export function scopeCapsProblems({ caps }: { readonly caps: ScopeCaps; }): readonly string[] {
  return [
    ...textCapProblem({
      value: caps.unitCpuQuota,
      name: '--unit-cpu-quota',
    }),
    ...textCapProblem({
      value: caps.unitMemoryMax,
      name: '--unit-memory-max',
    }),
    ...wholeCapProblem({
      value: caps.unitTasksMax,
      name: '--unit-tasks-max',
    }),
    ...(caps.unitIoWeight > maximumUnitIoWeight
      ? [`--unit-io-weight ${String(caps.unitIoWeight)} is above ${String(maximumUnitIoWeight)}, the highest disk weight a per-command control group may carry (rule RXS).`]
      : wholeCapProblem({
        value: caps.unitIoWeight,
        name: '--unit-io-weight',
      })),
  ];
}

/**
 Refuse caps systemd could not read, before anything starts.

 @param caps - the caps the caller asked for

 @returns the caps, unchanged

 @throws NativeGateError when a cap is not a value systemd reads, naming every unusable one

 @example
 ```ts
 const caps = checkedScopeCaps({ caps: defaultCheckScopeCaps });
 ```
 */
export function checkedScopeCaps({ caps }: { readonly caps: ScopeCaps; }): ScopeCaps {
  /**
   Every reason the caps are unusable.
   */
  const problems = scopeCapsProblems({ caps, });
  if (problems.length > 0)
    throw new NativeGateError(`the check scope's caps cannot be used: ${problems.join('\n')}`);
  return caps;
}

/**
 The identity one run gives its scope names, which keeps them apart from another run's.

 A stamp alone is second-precise, so two runs of one check started in the same second asked the manager for the same unit
 name; while the first scope was active the manager refused the second run's start, which ended that check with status 2
 without running it. This token is a UUID without hyphens, 122 random bits: two runs drawing the same token is not a
 realistic event, which is the same reason the planted-controls launcher names its units by a launch token
 (`native-planted-parallel.ts`).

 @returns the token a run puts in every scope name it makes

 @example
 ```ts
 runTokenOfScopes();
 ```
 */
export function runTokenOfScopes(): string {
  return randomUUID()
    .replaceAll(
    '-',
    '',
    );
}

/**
 The name of one check's scope.

 @param label - what the check covers, in lowercase letters, digits and hyphens

 @param stamp - the moment the run started, from `localStamp`

 @param runToken - the identity of the run the scope belongs to, from {@link runTokenOfScopes}

 @returns the scope's name, without the `.scope` suffix

 @example
 ```ts
 checkScopeName({ label: 'git-policy-cli-test-unit', stamp: '20261111T133700-0500', runToken: 'a1b2c3d4' });
 ```
 */
export function checkScopeName({
  label,
  stamp,
  runToken,
}: {
  readonly label: string;
  readonly stamp: string;
  readonly runToken: string;
}): string {
  return `${checkScopeNamePrefix}-${label}-${stamp}-${runToken}`;
}

/**
 The arguments after the `systemd-run` program name that start one capped scope.

 `--quiet` keeps systemd's own "Running scope as unit" line out of the check's output, which a person reads as the check's
 own answer.

 @param name - the scope's name

 @param caps - its caps

 @param program - the command to run

 @param args - its arguments, each kept whole so no shell reads one

 @returns the complete argument array

 @example
 ```ts
 scopeArguments({ name: 'gpc-check-a-20261111T133700-0500', caps: defaultCheckScopeCaps, program: 'mise', args: ['run', 'x'] });
 ```
 */
export function scopeArguments({
  name,
  caps,
  program,
  args,
}: {
  readonly name: string;
  readonly caps: ScopeCaps;
  readonly program: string;
  readonly args: readonly string[];
}): readonly string[] {
  return [
    '--user',
    '--scope',
    '--quiet',
    `--unit=${name}`,
    `--property=CPUQuota=${caps.unitCpuQuota}`,
    `--property=MemoryMax=${caps.unitMemoryMax}`,
    `--property=TasksMax=${String(caps.unitTasksMax)}`,
    `--property=IOWeight=${String(caps.unitIoWeight)}`,
    '--',
    program,
    ...args,
  ];
}

/**
 How one command in its own scope ended.
 */
export type ScopeOutcome = {
  /**
   Status the command exited with, or minus one when it never ran to an exit.
   */
  readonly status: number;
  /**
   What it printed on standard output.
   */
  readonly stdout: string;
  /**
   What it printed on standard error.
   */
  readonly stderr: string;
  /**
   Whether its deadline ended it instead of it ending by itself.
   */
  readonly timedOut: boolean;
};

/**
 What one attempt to end a scope found.
 */
export type ScopeStop = {
  /**
   Whether nothing of the scope is left: the manager ended it, or it knew no such unit, which is a scope already over.
   */
  readonly ended: boolean;
  /**
   What the manager answered, which a stop that did nothing carries into the refusal.
   */
  readonly detail: string;
};

/**
 The exit status `systemctl` uses for a unit it does not know, which is a scope that is already over and not a failure.
 */
const unitNotLoadedStatus = 5;

/**
 Ends one scope and everything in it by the name it was started with.

 A scope the manager no longer knows is already over, which is not a failure: the client is asked about and nothing waits
 on the answer.

 @param name - the scope's name, without its `.scope` suffix

 @param program - the `systemctl` to run, which a test replaces with a stand-in

 @param environment - its environment, which defaults to this process's

 @returns whether nothing of the scope is left, and what the manager answered

 @example
 ```ts
 await endCappedScope({ name: 'gpc-check-a-20261111T133700-0500' });
 ```
 */
export async function endCappedScope({
  name,
  program = systemctlProgram,
  environment = process.env,
}: {
  readonly name: string;
  readonly program?: string;
  readonly environment?: NodeJS.ProcessEnv;
}): Promise<ScopeStop> {
  /**
   What the manager answered.
   */
  const ended = await runGateCommand({
    program,
    args: [
      '--user',
      'kill',
      '--signal=SIGKILL',
      '--kill-whom=all',
      `${name}.scope`,
    ],
    deadlineMilliseconds: scopeStopDeadlineMilliseconds,
    environment,
  });
  /**
   What the manager printed, which a stop that did nothing carries.
   */
  const printed = `${ended.stderr}\n${ended.stdout}`.trim();
  return {
    ended: (ended.status === 0) || (ended.status === unitNotLoadedStatus),
    detail: ended.timedOut
      ? `${program} --user kill did not answer within ${String(scopeStopDeadlineMilliseconds)} ms`
      : printed,
  };
}

/**
 Runs one command in its own capped scope and reports how it ended, ending the scope when its deadline passes.

 @param name - the scope's name

 @param caps - its caps

 @param program - the command to run

 @param args - its arguments

 @param deadlineMilliseconds - how long it may run before its scope is ended

 @param workingDirectory - where it runs, which defaults to this process's directory

 @param environment - its environment, which defaults to this process's

 @param scopeProgram - the `systemd-run` to start, which a test replaces with a stand-in

 @param stopProgram - the `systemctl` that ends the scope, which a test replaces with a stand-in

 @returns its status, its output and whether the deadline ended it

 @example
 ```ts
 await runInCappedScope({ name, caps: defaultCheckScopeCaps, program: 'mise', args: ['run', 'x'], deadlineMilliseconds: 60_000 });
 ```
 */
export async function runInCappedScope({
  name,
  caps,
  program,
  args,
  deadlineMilliseconds,
  workingDirectory = process.cwd(),
  environment = process.env,
  scopeProgram = systemdRunProgram,
  stopProgram = systemctlProgram,
}: {
  readonly name: string;
  readonly caps: ScopeCaps;
  readonly program: string;
  readonly args: readonly string[];
  readonly deadlineMilliseconds: number;
  readonly workingDirectory?: string;
  readonly environment?: NodeJS.ProcessEnv;
  readonly scopeProgram?: string;
  readonly stopProgram?: string;
}): Promise<ScopeOutcome> {
  /**
   What the scope's command printed and how it ended.
   */
  const outcome = await runGateCommand({
    program: scopeProgram,
    args: scopeArguments({
      name,
      caps,
      program,
      args,
    }),
    deadlineMilliseconds,
    environment,
    workingDirectory,
  });
  if (!outcome.timedOut)
    return {
      status: outcome.status,
      stdout: outcome.stdout,
      stderr: outcome.stderr,
      timedOut: false,
    };
  // The client is gone but the command it started is still in the scope, so the scope is ended by the name it carries.
  /**
   What ending the scope found.
   */
  const stop = await endCappedScope({
    name,
    program: stopProgram,
    environment,
  });
  if (!stop.ended)
    throw new NativeGateError(`the check ${name} ran past its deadline and its scope could not be ended, so what it started may still run: ${stop.detail === '' ? 'the manager printed nothing' : stop.detail}`);
  return {
    status: outcome.status,
    stdout: outcome.stdout,
    stderr: outcome.stderr,
    timedOut: true,
  };
}

/**
 Exit status of a run whose checks did not all pass, kept apart from the exit status of a refusal so a coordinator tells a
 failing check from a command line it typed wrongly.
 */
export const checkFailedExitCode = 2;
