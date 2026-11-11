#!/usr/bin/env node
/**
 Move the release gate's worktree to the tip of `main` and start a gate there.

 This is what `native:gate:main-tip` runs, and it is the order a wrapper release is gated in: the worktree is detached at
 the commit `main` points at now, then a gate is started on it with a label that names that commit, so the log says what
 it tested even after `main` moved on. A worktree holding uncommitted work is not moved: the status is printed and the
 command exits with a status of its own, because a coordinator's copy of the main checkout is never another session's to
 throw away (`AGENTS.md`, rule `EC1`).
 */
import { homedir } from 'node:os';

import {
  gateCapsFromOptions,
  gateCapsOptionNames,
  gateOptions,
} from './native-gate-arguments.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  gateRefusalExitCode,
  printLine,
  runGateMain,
} from './native-gate-main.ts';
import {
  gateLogDirectoryOf,
  gateRepositoryOf,
  gateWorktreeNameOf,
  gateWorktreePath,
  gitDeadlineMilliseconds,
  gitProgram,
  promptlessGitSettings,
} from './native-gate-paths.ts';
import { runGateCommandOrThrow } from './native-gate-run.ts';
import { startGateUnit } from './native-gate-start.ts';
import { gateLabelOnMainTip } from './native-gate-unit-name.ts';

/**
 Option that names the checkout the gate worktree belongs to.
 */
const repositoryOption = '--repository';

/**
 Option that names the worktree the gate runs in.
 */
const worktreeOption = '--worktree';

/**
 Option that names the folder the gate's log is written to.
 */
const logDirectoryOption = '--log-directory';

/**
 Characters of `git rev-parse --short=<digits>` the label and the printed line name, so a log names a commit a person can
 find and no more.
 */
const shortShaDigits = 10;

/**
 Exit status of a worktree that holds uncommitted work and was therefore not moved.
 */
export const worktreeNotCleanExitCode = 2;

/**
 Characters of the worktree status a refusal prints, which is the status of a tree that could not be moved.
 */
const printedStatusCharacters = 800;

/**
 How the command is called, printed by every refusal that reads the command line.
 */
const usage = 'usage: native:gate:main-tip [--repository <path>] [--worktree <name>] [--log-directory <path>] [--unit-cpu-quota <value>] [--unit-memory-max <value>] [--unit-tasks-max <count>] [--unit-io-weight <weight>] [--container-pids-limit <count>]';

/**
 Move the gate worktree to the tip of `main`, then start a gate there.

 @returns zero once a gate is queued at `main`'s tip, {@link worktreeNotCleanExitCode} when the worktree was not moved,
   and {@link gateRefusalExitCode} when the gate could not be started

 @throws NativeGateError when the command line, git or the caps cannot move the worktree or start the gate

 @example
 ```ts
 process.exitCode = await runGateMain({ main });
 ```
 */
async function main(): Promise<number> {
  /**
   The command line, whose options all take a value and which takes no positionals.
   */
  const options = gateOptions({
    argv: process.argv
      .slice(2),
    knownOptions: [
      repositoryOption,
      worktreeOption,
      logDirectoryOption,
      ...gateCapsOptionNames,
    ],
  });
  if (options.positionals
    .length
    > 0)
    throw new NativeGateError(`native:gate:main-tip takes no positional arguments, and "${options.positionals
      .join(' ')}" is one. ${usage}`);
  /**
   The checkout the command line named, which wins over the environment.
   */
  const namedRepository = options.values
    .get(repositoryOption);
  /**
   The log folder the command line named, which wins over the environment.
   */
  const namedLogDirectory = options.values
    .get(logDirectoryOption);
  /**
   The checkout the gate worktree belongs to, and the worktree itself.
   */
  const repository = await gateRepositoryOf({
    environment: process.env,
    ...(namedRepository === undefined ? {} : { named: namedRepository, }),
  });
  /**
   The worktree the gate runs in, which the option, the environment or the recorded default names.
   */
  const worktree = options.values
    .get(worktreeOption)
    ?? gateWorktreeNameOf({ environment: process.env, });
  /**
   The worktree's absolute path, which the git steps and the unit all name.
   */
  const worktreePath = gateWorktreePath({
    repository,
    worktree,
  });
  /**
   The environment git runs with, which asks for no password.
   */
  const gitEnvironment = {
    ...process.env,
    ...promptlessGitSettings,
  };
  /**
   The commit `main` points at.
   */
  const sha = (await runGateCommandOrThrow({
    program: gitProgram,
    args: [
      '-C',
      repository,
      'rev-parse',
      `--short=${String(shortShaDigits)}`,
      'main',
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: gitEnvironment,
  })).stdout
    .trim();
  /**
   What the gate worktree holds that its commit does not, which stops the move.
   */
  const status = (await runGateCommandOrThrow({
    program: gitProgram,
    args: [
      '-C',
      worktreePath,
      'status',
      '--short',
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: gitEnvironment,
  })).stdout;
  if (status.trim() !== '') {
    console.log(`the gate worktree is not clean, so it was not moved:\n${status.slice(
      0,
      printedStatusCharacters
    )}`);
    return worktreeNotCleanExitCode;
  }
  await runGateCommandOrThrow({
    program: gitProgram,
    args: [
      '-C',
      worktreePath,
      'checkout',
      '--quiet',
      '--detach',
      sha,
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: gitEnvironment,
  });
  console.log(`gate worktree at ${sha}`);
  /**
   How the launch ended, which is this command's own status when no gate was started.
   */
  let launchStatus = 0;
  try {
    await startGateUnit({
      repository,
      worktree,
      label: gateLabelOnMainTip({
        worktree,
        sha,
      }),
      logDirectory: gateLogDirectoryOf({
        environment: process.env,
        home: homedir(),
        ...(namedLogDirectory === undefined ? {} : { named: namedLogDirectory, }),
      }),
      caps: gateCapsFromOptions({ options, }),
      moment: new Date(),
      print: printLine,
    });
  }
  catch (error) {
    if (!(error instanceof NativeGateError))
      throw error;
    console.error(error.message);
    launchStatus = gateRefusalExitCode;
  }
  console.log(`launch status ${String(launchStatus)}`);
  return launchStatus;
}

if (import.meta.main)
  process.exitCode = await runGateMain({ main, });
