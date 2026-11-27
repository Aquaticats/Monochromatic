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
  SCOPE_NOT_READ,
  type ScopeAccountingRead,
  checkScopeCgroupPath,
  checkScopePollMilliseconds,
  checkScopeSlice,
  readScopeAccounting,
  watchScopeAccounting,
} from './native-check-scope-accounting.ts';
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
 own answer. The slice is named rather than left to systemd's default, so the path this runner reads the scope's
 counters from (`bin/native-check-scope-accounting.ts`) is the path the scope is put in; measured on this host on
 2026-11-27, a scope that named no slice landed in `app.slice`, which is what {@link checkScopeSlice} states.

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
    `--slice=${checkScopeSlice}`,
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
 How one command in its own scope ended, and what that scope counted while it ran.
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
  /**
   How long the scope ran the command, from the moment `systemd-run` was started to the moment it answered.
   */
  readonly elapsedMilliseconds: number;
  /**
   The last figures read from the scope while it ran, or {@link SCOPE_NOT_READ} when its control group was never there to
   read.
   */
  readonly accounting: ScopeAccountingRead;
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
 Runs one command in its own capped scope and reports how it ended and what that scope counted, ending the scope when its
 deadline passes.

 The scope's counters are read while it runs (`bin/native-check-scope-accounting.ts`), because a scope takes its control
 group with it: what a caller reads after the command ended is the last sample taken before that. A check refused a fork
 or killed for memory therefore answers with the reading that names it, and `bin/native-ts-checks.ts` says so beside the
 check's own result.

 @param name - the scope's name

 @param caps - its caps

 @param program - the command to run

 @param args - its arguments

 @param deadlineMilliseconds - how long it may run before its scope is ended

 @param workingDirectory - where it runs, which defaults to this process's directory

 @param environment - its environment, which defaults to this process's

 @param scopeProgram - the `systemd-run` to start, which a test replaces with a stand-in

 @param stopProgram - the `systemctl` that ends the scope, which a test replaces with a stand-in

 @param scopeRoot - the cgroup hierarchy's root, which a test points at a fixture tree

 @param pollMilliseconds - how long between two reads of the scope, which a test shortens

 @returns its status, its output, whether the deadline ended it, how long it ran and what its scope counted

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
  scopeRoot,
  pollMilliseconds = checkScopePollMilliseconds,
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
  readonly scopeRoot?: string;
  readonly pollMilliseconds?: number;
}): Promise<ScopeOutcome> {
  /**
   The control group this check's scope takes, which its counters are read from while it runs.
   */
  const cgroupPath = checkScopeCgroupPath({
    name,
    ...(scopeRoot === undefined ? {} : { root: scopeRoot, }),
  });
  await using watch = watchScopeAccounting({
    cgroupPath,
    intervalMilliseconds: pollMilliseconds,
  });
  /**
   When the scope was asked for, which is what the check's wall time is counted from.
   */
  const startedAt = performance.now();
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
  /**
   How long the scope held the command, which a check held at its CPU quota is reported by.
   */
  const elapsedMilliseconds = performance.now() - startedAt;
  if (outcome.timedOut) {
    // The client is gone but the command it started is still in the scope, so the scope is ended by the name it carries,
    // before anything else waits, which is what keeps a check that ignores its deadline from outliving it.
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
  }
  await watch.stop();
  // A scope still exists for a moment after its command ends, so this last look keeps the final counters when it is
  // there and leaves the last sample taken while it ran when it is not.
  /**
   What the scope's control group answered once its command had ended.
   */
  const lastLook = await readScopeAccounting({ cgroupPath, });
  /**
   The reading this outcome carries: the last look when the scope was still there, and the last sample taken while it
   ran otherwise.
   */
  const accounting = lastLook === SCOPE_NOT_READ ? watch.lastRead() : lastLook;
  return {
    status: outcome.status,
    stdout: outcome.stdout,
    stderr: outcome.stderr,
    timedOut: outcome.timedOut,
    elapsedMilliseconds,
    accounting,
  };
}

/**
 Exit status of a run whose checks did not all pass, kept apart from the exit status of a refusal so a coordinator tells a
 failing check from a command line it typed wrongly.
 */
export const checkFailedExitCode = 2;
