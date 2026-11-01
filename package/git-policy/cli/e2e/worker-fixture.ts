/**
 Scenario actors:
 worktree writes,
 commit attempts through the wrapper,
 and auxiliary wrapper commands.

 @module
 */

import {
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import type {
  AttemptMode,
  AttemptRecord,
  CapturedPath,
  WorkloadLedger,
} from './ledger-fixture.ts';
import {
  type ProcessOutcome,
  type RunningProcess,
  runBytes,
  startProcess,
} from './process-fixture.ts';
import {
  realGit,
  type ScenarioRepository,
} from './repository-fixture.ts';

//region Types

/**
 Shared scenario handles.
 */
export type ScenarioActors = Readonly<{
  /**
   Scenario repository.
   */
  repository: ScenarioRepository;
  /**
   Scenario ledger.
   */
  ledger: WorkloadLedger;
}>;

/**
 Started commit attempt.
 */
export type StartedAttempt = Readonly<{
  /**
   Message token.
   */
  token: string;
  /**
   Running process group.
   */
  running: RunningProcess;
  /**
   Kills the process group and records the attempt as deliberately killed.
   */
  kill: () => void;
  /**
   Settles with the recorded attempt.
   */
  finished: Promise<AttemptRecord>;
}>;

/**
 Attempt fields known before its process settles.
 */
export type PendingAttempt = Omit<AttemptRecord, 'killed' | 'outcome' | 'startedAt'>;

//endregion Types

//region Worktree

/**
 Writes or removes one worktree path and records the harness's intent.

 @param repository - scenario repository

 @param ledger - scenario ledger

 @param path - repository path

 @param bytes - new bytes, absent to remove

 @example
 ```ts
 await writeWorktree({ ...actors, path: 'a.txt', bytes: Buffer.from('a\n') });
 ```
 */
export async function writeWorktree({
  repository,
  ledger,
  path,
  bytes,
}: ScenarioActors & Readonly<{
  path: string;
  bytes?: Buffer;
}>,): Promise<void> {
  /**
   Absolute path.
   */
  const absolute = join(
    repository.worktree,
    path,
  );
  if (bytes === undefined)
    await rm(
      absolute,
      { force: true, },
    );
  else {
    await mkdir(
      dirname(absolute,),
      { recursive: true, },
    );
    await writeFile(
      absolute,
      bytes,
    );
  }
  ledger.recordWorktree({
    path,
    ...(bytes === undefined ? {} : { bytes, }),
  },);
}

/**
 Reads worktree bytes of a path.

 @param repository - scenario repository

 @param path - repository path

 @returns captured path, without bytes when missing

 @example
 ```ts
 await readWorktree({ repository, path: 'a.txt' });
 ```
 */
export async function readWorktree({
  repository,
  path,
}: Readonly<{
  repository: ScenarioRepository;
  path: string;
}>,): Promise<CapturedPath> {
  try {
    return {
      path,
      bytes: await readFile(join(
        repository.worktree,
        path,
      ),),
    };
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return { path, };
    throw error;
  }
}

/**
 Reads the real index's stage-0 bytes of a path.

 @param repository - scenario repository

 @param path - repository path

 @returns captured path, without bytes when the index has no entry

 @example
 ```ts
 await readIndexEntry({ repository, path: 'a.txt' });
 ```
 */
export async function readIndexEntry({
  repository,
  path,
}: Readonly<{
  repository: ScenarioRepository;
  path: string;
}>,): Promise<CapturedPath> {
  /**
   `<mode> <oid> <stage>\t<path>` record, empty when untracked.
   */
  const listed = await realGit({
    repository,
    args: [
      'ls-files',
      '--stage',
      '-z',
      '--',
      path,
    ],
  },);
  /**
   Mode and staged blob ID fields.
   */
  const [, oid,] = listed.split(' ',);
  if ((listed === '') || (oid === undefined))
    return { path, };
  return {
    path,
    bytes: await runBytes({
      command: repository.realGit,
      args: [
        'cat-file',
        'blob',
        oid,
      ],
      cwd: repository.worktree,
      env: repository.realEnv,
    },),
  };
}

//endregion Worktree

//region Commands

/**
 Builds the message token for an attempt label.

 @param label - unique label

 @returns token embedded in the commit message

 @example
 ```ts
 attemptToken('w1'); // => 'E2E-TOKEN-w1'
 ```
 */
export function attemptToken(label: string,): string {
  return `E2E-TOKEN-${label}`;
}

/**
 Wraps a running process into a started attempt that records itself on settlement.

 @param ledger - scenario ledger

 @param pending - attempt fields known at start

 @param running - running process

 @returns started attempt

 @example
 ```ts
 trackAttempt({ ledger, pending, running });
 ```
 */
export function trackAttempt({
  ledger,
  pending,
  running,
}: Readonly<{
  ledger: WorkloadLedger;
  pending: PendingAttempt;
  running: RunningProcess;
}>,): StartedAttempt {
  /**
   Kill flag recorded with the attempt.
   */
  const state = { killed: false, };
  /**
   Start time, read before the process runs.
   */
  const startedAt = performance.now();
  return {
    token: pending.token,
    running,
    kill(): void {
      // A process that already settled keeps its real outcome; only an in-flight group counts as killed.
      if (running.isSettled())
        return;
      state.killed = true;
      running.killGroup();
    },
    finished: (async function record(): Promise<AttemptRecord> {
      /**
       Recorded attempt.
       */
      const attempt: AttemptRecord = {
        ...pending,
        startedAt,
        outcome: await running.outcome,
        killed: state.killed,
      };
      ledger.addAttempt(attempt,);
      return attempt;
    })(),
  };
}

/**
 Starts a commit attempt through the wrapper and records it on settlement.

 @param repository - scenario repository

 @param ledger - scenario ledger

 @param label - unique label

 @param paths - explicit paths, or the index paths an index commit is expected to take

 @param mode - selection mode

 @param env - extra wrapper environment, such as a test-only phase marker

 @param extraArgs - `commit` options before the selection, such as `-e` for a held editor

 @returns started attempt

 @example
 ```ts
 const attempt = await startAttempt({ ...actors, label: 'w1', paths: ['a.txt'], mode: 'explicit' });
 await attempt.finished;
 ```
 */
export async function startAttempt({
  repository,
  ledger,
  label,
  paths,
  mode,
  env = {},
  extraArgs = [],
}: ScenarioActors & Readonly<{
  label: string;
  paths: readonly string[];
  mode: Exclude<AttemptMode, 'foreign'>;
  env?: Readonly<Record<string, string>>;
  extraArgs?: readonly string[];
}>,): Promise<StartedAttempt> {
  /**
   Message token.
   */
  const token = attemptToken(label,);
  /**
   Bytes at invocation:
   worktree for explicit and amend commits,
   real index for index commits.
   */
  const captured = await Promise.all(paths.map(async function capture(path,) {
    return mode === 'index' ? await readIndexEntry({
      repository,
      path,
    },) : await readWorktree({
      repository,
      path,
    },);
  },),);
  /**
   Mode-specific arguments.
   */
  const modeArgs = {
    explicit: [
      '--',
      ...paths,
    ],
    amend: [
      '--amend',
      '--',
      ...paths,
    ],
    index: ['--no-only',],
  }[mode];
  /**
   `HEAD` read immediately before start.
   */
  const headBefore = (await realGit({
    repository,
    args: [
      'rev-parse',
      'HEAD',
    ],
  },)).trim();
  return trackAttempt({
    ledger,
    pending: {
      label,
      token,
      mode,
      selectedPaths: paths,
      captured,
      headBefore,
      expectedBranch: repository.branch,
      // Auto-push never forces, so an amended pushed commit cannot reach the remote.
      requiresRemote: mode !== 'amend',
    },
    running: startProcess({
      command: 'git',
      args: [
        'commit',
        '--quiet',
        '--message',
        `e2e ${label} [${token}]`,
        ...extraArgs,
        ...modeArgs,
      ],
      cwd: repository.worktree,
      env: {
        ...repository.wrapperEnv,
        ...env,
        E2E_TOKEN: token,
      },
    },),
  },);
}

/**
 Runs one wrapper command to settlement and records it.

 @param repository - scenario repository

 @param ledger - scenario ledger

 @param label - unique label

 @param args - Git arguments

 @param mustSucceed - whether the accepted design requires exit 0

 @returns settled outcome

 @example
 ```ts
 await runWrapper({ ...actors, label: 'add b', args: ['add', 'b.txt'], mustSucceed: true });
 ```
 */
export async function runWrapper({
  repository,
  ledger,
  label,
  args,
  mustSucceed,
}: ScenarioActors & Readonly<{
  label: string;
  args: readonly string[];
  mustSucceed: boolean;
}>,): Promise<ProcessOutcome> {
  /**
   Settled wrapper.
   */
  const outcome = await startProcess({
    command: 'git',
    args,
    cwd: repository.worktree,
    env: repository.wrapperEnv,
  },)
    .outcome;
  ledger.addAuxiliary({
    label,
    outcome,
    mustSucceed,
    wrapper: true,
  },);
  return outcome;
}

//endregion Commands
