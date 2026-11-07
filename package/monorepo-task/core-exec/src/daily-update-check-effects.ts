/**
 The real effects of the daily update's package checks: the task listing, the checks themselves, file texts at a commit,
 `gh`, the list of issues kept for the next run, and the clock.

 @module
 */

import spawn from 'nano-spawn';
import { spawn as spawnChild, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  CARGO_BUILD_JOBS_PER_CHECK,
  GH_CALL_TIMEOUT_MS,
  TASK_LISTING_TIMEOUT_MS,
} from './daily-update-check-bounds.ts';
import { runCheckBounded, } from './daily-update-check-command.ts';
import type {
  CheckEffects,
  GhRequest,
  GhResult,
  RenderedIssue,
} from './daily-update-package-check-types.ts';

/**
 Repository-relative path of the file that keeps the issues an earlier run could not publish.
 */
export const PENDING_ISSUES_PATH: string = join(
  '.cache',
  'toolchain-daily-update-pending-issues.json',
);

/**
 Environment every check runs with: no colors, so the first error is plain text, and a bound on Cargo's own compile
 fan-out so checks that run side by side do not each start a compiler per core.
 */
export const CHECK_ENVIRONMENT: Readonly<Record<string, string>> = {
  NO_COLOR: '1',
  CARGO_TERM_COLOR: 'never',
  CARGO_BUILD_JOBS: String(CARGO_BUILD_JOBS_PER_CHECK,),
};

/**
 Environment `gh` runs with: it reads its own credentials, never asks a question, and never looks for an update.
 `GIT_ASKPASS` is set and empty and `GIT_TERMINAL_PROMPT` is zero so no Git credential prompt can appear either.
 */
export const GH_ENVIRONMENT: Readonly<Record<string, string>> = {
  GH_PROMPT_DISABLED: '1',
  GH_NO_UPDATE_NOTIFIER: '1',
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 Whether a value is an object with a task name.

 @param task - a value from the parsed task list

 @returns true when it has a string `name`

 @example
 ```ts
 hasTaskName({ name: '//package/a/b:build', },); // true
 ```
 */
export function hasTaskName(task: unknown,): task is { readonly name: string; } {
  return ((typeof task) === 'object') && (task !== null)
    && ('name' in task)
    && ((typeof task.name) === 'string');
}

/**
 Names every task of every package.

 @param repository - absolute repository path

 @returns task names such as `//package/a/b:lint:clippy`

 @throws SubprocessError when `mise tasks ls` fails or runs past its bound

 @throws Error when its output is not a JSON list

 @example
 ```ts
 const names = await listTaskNames('/var/home/user/Monochromatic',);
 ```
 */
export async function listTaskNames(repository: string,): Promise<readonly string[]> {
  /**
   What `mise tasks ls --all --json` printed.
   */
  const { stdout, } = await spawn(
    'mise',
    [
      'tasks',
      'ls',
      '--all',
      '--json',
    ],
    {
      cwd: repository,
      timeout: TASK_LISTING_TIMEOUT_MS,
    },
  );
  /**
   The parsed list.
   */
  const tasks: unknown = JSON.parse(stdout,);
  if (!Array.isArray(tasks,))
    throw new Error('mise tasks ls --all --json printed something other than a JSON list',);
  return tasks.filter(hasTaskName,)
    .map(function nameOf(task,) {
      return task.name;
    },);
}

/**
 Runs `gh` with a time bound, sending a text on standard input.

 @param request - arguments and standard input

 @param directory - working directory, the repository, where `gh` finds the repository it works on

 @returns what `gh` answered

 @throws Error when `gh` cannot start

 @example
 ```ts
 const answer = await runGh({ request: { args: ['issue', 'list',], stdin: '', }, directory: '/r', },);
 ```
 */
export async function runGh({
  request,
  directory,
}: {
  readonly request: GhRequest;
  readonly directory: string;
},): Promise<GhResult> {
  /**
   The `gh` process.
   */
  const child = spawnChild(
    'gh',
    [...request.args,],
    {
      cwd: directory,
      env: {
        ...process.env,
        ...GH_ENVIRONMENT,
      },
      stdio: [
        'pipe',
        'pipe',
        'pipe',
      ],
      timeout: GH_CALL_TIMEOUT_MS,
      killSignal: 'SIGKILL',
    },
  );
  /**
   Output collected so far.
   */
  const collected = {
    stdout: '',
    stderr: '',
  };
  child.stdout
    .on(
    'data',
    function keepOut(chunk: Buffer,) {
      collected.stdout += chunk.toString('utf8',);
    },
    );
  child.stderr
    .on(
    'data',
    function keepErr(chunk: Buffer,) {
      collected.stderr += chunk.toString('utf8',);
    },
    );
  // An error on standard input (gh exited before reading it all) is reported by the exit status; the text is kept.
  child.stdin
    .on(
    'error',
    function keepInputError(error,) {
      collected.stderr += `standard input: ${String(error,)}\n`;
    },
    );
  child.stdin
    .end(request.stdin,);
  await once(
    child,
    'close',
  );
  return {
    exitCode: child.exitCode ?? (-1),
    stdout: collected.stdout,
    stderr: child.signalCode === null
      ? collected.stderr
      : `${collected.stderr}gh was stopped by ${child.signalCode} after its time bound`,
  };
}

/**
 Whether a value has every part of a rendered issue.

 @param value - a value from the parsed file

 @returns true when it is a rendered issue

 @example
 ```ts
 isRenderedIssue({ marker: 'm', title: 't', body: 'b', comment: 'c', labels: [], },); // true
 ```
 */
export function isRenderedIssue(value: unknown,): value is RenderedIssue {
  return ((typeof value) === 'object') && (value !== null)
    && ('marker' in value)
    && ((typeof value.marker) === 'string')
    && ('title' in value)
    && ((typeof value.title) === 'string')
    && ('body' in value)
    && ((typeof value.body) === 'string')
    && ('comment' in value)
    && ((typeof value.comment) === 'string')
    && ('labels' in value)
    && Array.isArray(value.labels,);
}

/**
 Reads the issues an earlier run kept.

 @param path - absolute path of the file

 @returns the issues, none when the file does not exist or is not a list

 @example
 ```ts
 const issues = await readPending('/r/.cache/toolchain-daily-update-pending-issues.json',);
 ```
 */
export async function readPending(path: string,): Promise<readonly RenderedIssue[]> {
  try {
    /**
     The parsed file.
     */
    const parsed: unknown = JSON.parse(await readFile(
      path,
      'utf8',
    ),);
    return Array.isArray(parsed,) ? parsed.filter(isRenderedIssue,) : [];
  }
  catch (error) {
    // A missing file is the normal case: no issue is waiting. Any other failure is logged and treated the same.
    if (!((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ENOENT'))) {
      process.stderr
        .write(`daily-update: the pending issues file could not be read, so none is retried: ${String(error,)}\n`,);
    }
    return [];
  }
}

/**
 Milliseconds on a clock that never goes backwards.

 @returns the time

 @example
 ```ts
 const startedAt = clockNow();
 ```
 */
export function clockNow(): number {
  return performance.now();
}

/**
 Reads the text of a repository file at a commit.

 @param repository - absolute repository path

 @param commit - commit

 @param path - repository-relative path

 @returns the text; the empty string when the commit has no such file

 @example
 ```ts
 const lock = await fileAtCommit({ repository: '/r', commit: 'abc', path: 'toolchain.lock.json', },);
 ```
 */
export async function fileAtCommit({
  repository,
  commit,
  path,
}: {
  readonly repository: string;
  readonly commit: string;
  readonly path: string;
},): Promise<string> {
  try {
    return (await spawn(
      'git',
      [
        '-C',
        repository,
        'show',
        `${commit}:${path}`,
      ],
    )).stdout;
  }
  catch (error) {
    // The file does not exist at that commit, which is a normal answer for a path a commit never held.
    process.stderr
      .write(`daily-update: ${path} is not readable at ${commit}: ${String(error,)}\n`,);
    return '';
  }
}

/**
 Builds the real effects of the checks for a repository.

 @param repository - absolute repository path

 @returns the effects

 @example
 ```ts
 const effects = realCheckEffects('/var/home/user/Monochromatic',);
 ```
 */
export function realCheckEffects(repository: string,): CheckEffects {
  /**
   Absolute path of the pending issues file.
   */
  const pendingPath = join(
    repository,
    PENDING_ISSUES_PATH,
  );
  return {
    listCheckTaskNames: function listChecks() {
      return listTaskNames(repository,);
    },
    runCheck: function runCheck(check,) {
      return runCheckBounded({
        repository,
        check,
        environment: CHECK_ENVIRONMENT,
      },);
    },
    fileAt: function fileAt({
      commit,
      path,
    },) {
      return fileAtCommit({
        repository,
        commit,
        path,
      },);
    },
    gh: function gh(request,) {
      return runGh({
        request,
        directory: repository,
      },);
    },
    readPendingIssues: function readPendingIssues() {
      return readPending(pendingPath,);
    },
    writePendingIssues: async function writePendingIssues(issues,) {
      await mkdir(
        dirname(pendingPath,),
        { recursive: true, },
      );
      await writeFile(
        pendingPath,
        `${JSON.stringify(
          issues,
          undefined,
          2,
        )}\n`,
      );
    },
    now: clockNow,
  };
}
