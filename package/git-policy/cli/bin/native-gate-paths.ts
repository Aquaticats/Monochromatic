/**
 Where a release gate runs and where it writes its log: the gate worktree, the log folder, and the environment
 variables that move either of them.

 The gate worktree is a worktree of this repository, so a bare worktree name is looked up under the repository's own
 `.claude/worktrees` folder, and an absolute path is taken as given. The log folder is scratch, which belongs under the
 home directory and never in the system temporary directory, whose backing device is unknowable (`AGENTS.md`, rule
 `ATH`); every recorded gate log of this repository is in `${HOME}/temp/agent/gate`.
 */
import { stat } from 'node:fs/promises';
import {
  isAbsolute,
  join,
  resolve,
} from 'node:path';

import { NativeGateError } from './native-gate-error.ts';

/**
 Milliseconds a git step of a gate command may take.

 One bound covers the three steps: `git rev-parse` answered in 11 ms and `git status --short` of a worktree of this
 repository took 8.9 s, both measured 2026-11-10, and the checkout of the gate worktree had this same bound before it was
 promoted here. A step still running after ten minutes is one that will not answer.
 */
export const gitDeadlineMilliseconds = 600_000;

/**
 The program every git step of a gate command runs, which is the wrapper's `git` on `PATH` (`AGENTS.md`, rule `LFF`).
 */
export const gitProgram = 'git';

/**
 The two variables a git step is given so it never asks for a password.

 A checkout that asked for one would hold the command open with no terminal to answer it, and these commands run under a
 coordinator with no one at the keyboard (`AGENTS.md`, the rule that nothing waits forever).
 */
export const promptlessGitSettings: Readonly<Record<string, string>> = {
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 Path segments, from the repository root, of the folder every agent worktree of this repository lives in.
 */
export const gateWorktreesFolderSegments = [
  '.claude',
  'worktrees',
] as const;

/**
 The worktree the wrapper's release gate runs in when the caller names none.

 It is the coordinator's own gate worktree: `native:gate:main-tip` moves this worktree to the tip of `main` and starts a
 gate there, which is how a wrapper release gets its evidence (`doc/handover/cli-git-rust-implementation.md`, "Gates").
 */
export const defaultGateWorktreeName = 'gate-wrapper-next-alpha';

/**
 Path segments, from the home directory, of the folder the gate logs are written to.
 */
export const gateLogFolderSegments = [
  'temp',
  'agent',
  'gate',
] as const;

/**
 Environment variable that names the checkout the gate worktree belongs to, read when `--repository` is absent.
 */
export const repositoryEnvironmentVariable = 'GIT_POLICY_GATE_REPOSITORY';

/**
 Environment variable that names the folder gate logs are written to, read when `--log-directory` is absent.
 */
export const logDirectoryEnvironmentVariable = 'GIT_POLICY_GATE_LOG_DIRECTORY';

/**
 Environment variable that names the worktree the gate runs in, read by `native:gate:main-tip` when `--worktree` is
 absent.
 */
export const gateWorktreeEnvironmentVariable = 'GIT_POLICY_GATE_WORKTREE';

/**
 The part of a process environment that moves where a gate runs or where it writes; `process.env` fits it.
 */
export type GatePathEnvironment = {
  readonly GIT_POLICY_GATE_REPOSITORY?: string;
  readonly GIT_POLICY_GATE_LOG_DIRECTORY?: string;
  readonly GIT_POLICY_GATE_WORKTREE?: string;
};

/**
 The folder the gate logs are written to when nothing names another.

 @param home - the user's home directory, from which the scratch folder is derived

 @returns the absolute path of the log folder

 @example
 ```ts
 defaultGateLogDirectory({ home: '/home/a' });
 ```
 */
export function defaultGateLogDirectory({ home }: { readonly home: string; }): string {
  return join(
    home,
    ...gateLogFolderSegments,
  );
}

/**
 The worktree a gate runs in.

 A bare name is a worktree of the repository, and an absolute path is taken as given, so a caller can point a gate at a
 worktree outside this repository's own folder.

 @param repository - absolute path of the checkout the worktrees folder belongs to

 @param worktree - worktree name under the worktrees folder, or an absolute path

 @returns the absolute path of the worktree

 @example
 ```ts
 gateWorktreePath({ repository: '/repo', worktree: 'gate-wrapper-next-alpha' });
 ```
 */
export function gateWorktreePath({
  repository,
  worktree,
}: {
  readonly repository: string;
  readonly worktree: string;
}): string {
  if (isAbsolute(worktree))
    return resolve(worktree);
  return join(
    resolve(repository),
    ...gateWorktreesFolderSegments,
    worktree,
  );
}

/**
 The log file of one gate.

 @param logDirectory - folder the logs are written to

 @param unitName - the gate unit's name, which the log file is named after

 @returns the absolute path of the gate's log file

 @example
 ```ts
 gateLogPath({ logDirectory: '/l', unitName: 'gpc-gate-a-20261110T180203-0500' });
 ```
 */
export function gateLogPath({
  logDirectory,
  unitName,
}: {
  readonly logDirectory: string;
  readonly unitName: string;
}): string {
  return join(
    resolve(logDirectory),
    `${unitName}.log`,
  );
}

/**
 The name that marks a folder as a checkout root: the main checkout of a repository holds it as a folder, and a worktree
 holds a file that points at the main checkout's own git directory.
 */
export const checkoutMarkerName = '.git';

/**
 What one path is, which the search for a checkout and the check for a worktree both need.

 'missing' is the kernel saying the path is not there; 'folder' and 'other' are what `stat` found.
 */
export type PathKind =
  | 'missing'
    | 'folder'
    | 'other';

/**
 Tells what one path is, without failing when it is not there.

 @param path - the path to look at

 @returns whether it is missing, a folder, or something else

 @throws NativeGateError when the path is there and cannot be looked at, which is neither of the three answers

 @example
 ```ts
 await pathKindOf({ path: '/repo/.git' });
 ```
 */
export async function pathKindOf({ path }: { readonly path: string; }): Promise<PathKind> {
  try {
    /**
     What the path is.
     */
    const entry = await stat(path,);
    return entry.isDirectory() ? 'folder' : 'other';
  }
  catch (error) {
    if (isMissingPathError({ error, }))
      return 'missing';
    throw new NativeGateError(
      `the path ${path} could not be looked at, so what it is stays unknown: ${String(error)}`,
      { cause: error, }
    );
  }
}

/**
 Tells whether a caught failure is the kernel saying a path is not there.

 @param error - the caught value

 @returns true for `ENOENT`, a path that is absent, and `ENOTDIR`, a parent that is not a folder

 @example
 ```ts
 isMissingPathError({ error: new Error('nope') });
 ```
 */
export function isMissingPathError({ error }: { readonly error: unknown; }): boolean {
  /**
   The code the kernel answered with, when the value carries one.
   */
  const code: unknown = ((typeof error) === 'object') && (error !== null) ? Reflect.get(
    error,
    'code',
  ) : undefined;
  return (code === 'ENOENT') || (code === 'ENOTDIR');
}

/**
 What the search for a checkout found.
 */
export type CheckoutSearch =
  | {
    /**
     Marks a checkout the search found.
     */
    readonly kind: 'found';
    /**
     Absolute path of that checkout.
     */
    readonly checkout: string
  }
    | {
    /**
     Marks a folder tree that holds no checkout.
     */
      readonly kind: 'none'
    };

/**
 The main checkout a folder belongs to: the nearest folder above its own whose `.git` is a folder.

 A worktree's `.git` is a file that points at the main checkout, so this skip the worktree and answers the checkout the
 gate worktree lives in. The walk reads the folder tree rather than asking git, because this repository's `git` refuses a
 subcommand run outside the root of a repository, and a gate command starts in the package folder rather than at the root.

 @param from - the folder to start at

 @returns the checkout the search found, or that it found none

 @example
 ```ts
 await repositoryHoldingCheckout({ from: '/repo/package/git-policy/cli' });
 ```
 */
export async function repositoryHoldingCheckout({ from }: { readonly from: string; }): Promise<CheckoutSearch> {
  /**
   The folder being looked at.
   */
  const folder = resolve(from);
  if ((await pathKindOf({ path: join(
    folder,
    checkoutMarkerName
  ), })) === 'folder')
    return {
      kind: 'found',
      checkout: folder,
    };
  /**
   The folder above this one, which is this one at the root of the filesystem.
   */
  const above = resolve(
    folder,
    '..',
  );
  if (above === folder)
    return { kind: 'none', };
  return await repositoryHoldingCheckout({ from: above, });
}

/**
 The checkout the gate worktree belongs to, from the option, the environment, or the folder tree.

 `--repository` on the command line wins over {@link repositoryEnvironmentVariable}, and when neither names a checkout the
 answer is the nearest folder above the caller whose `.git` is a folder, so a gate started from any worktree of a checkout
 finds the checkout the gate worktree lives in. A caller outside every checkout, such as a folder of scratch scripts, has
 to name one.

 @param environment - the environment the command runs with

 @param named - the checkout the command line named, which wins over the environment

 @param from - the folder the search starts at, which defaults to the caller's own

 @returns the absolute path of the checkout

 @throws NativeGateError when nothing names a checkout and no folder above the caller is one

 @example
 ```ts
 await gateRepositoryOf({ environment: process.env });
 ```
 */
export async function gateRepositoryOf({
  environment,
  named,
  from = process.cwd(),
}: {
  readonly environment: GatePathEnvironment;
  readonly named?: string;
  readonly from?: string;
}): Promise<string> {
  /**
   The checkout the caller named, when it named one.
   */
  const fromOption = named ?? environment[repositoryEnvironmentVariable];
  if ((fromOption !== undefined) && (fromOption !== ''))
    return resolve(fromOption);
  /**
   The checkout the folder tree names, when a folder above the caller is one.
   */
  const found = await repositoryHoldingCheckout({ from, });
  if (found.kind === 'found')
    return found.checkout;
  throw new NativeGateError(`no folder above ${resolve(from)} holds a ${checkoutMarkerName} folder, so the checkout a gate worktree belongs to is unknown; pass --repository <path> or set ${repositoryEnvironmentVariable}.`);
}

/**
 The folder a gate's log is written to, from the environment or from the home directory.

 @param environment - the environment the command runs with

 @param home - the user's home directory, which the scratch folder is derived from

 @param named - the folder the command line named, which wins over the environment

 @returns the absolute path of the log folder

 @example
 ```ts
 gateLogDirectoryOf({ environment: {}, home: '/home/a' });
 ```
 */
export function gateLogDirectoryOf({
  environment,
  home,
  named,
}: {
  readonly environment: GatePathEnvironment;
  readonly home: string;
  readonly named?: string;
}): string {
  /**
   The folder the caller named, when it named one.
   */
  const fromOption = named ?? environment[logDirectoryEnvironmentVariable];
  if ((fromOption !== undefined) && (fromOption !== ''))
    return resolve(fromOption);
  return defaultGateLogDirectory({ home, });
}

/**
 The worktree a gate runs in when the command line names none.

 @param environment - the environment the command runs with

 @returns the worktree name, which resolves under the checkout's worktrees folder

 @example
 ```ts
 gateWorktreeNameOf({ environment: {} });
 ```
 */
export function gateWorktreeNameOf({ environment }: { readonly environment: GatePathEnvironment; }): string {
  /**
   The worktree the environment names, when it names one.
   */
  const named = environment[gateWorktreeEnvironmentVariable];
  return ((named === undefined) || (named === '')) ? defaultGateWorktreeName : named;
}
