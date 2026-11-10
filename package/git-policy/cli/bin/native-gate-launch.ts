#!/usr/bin/env node
/**
 Start the wrapper's full release gate in the CachyOS test image, as its own transient systemd unit with its caps, and
 print where the unit and its log are.

 This is what `native:gate` runs, and it is how a wrapper release gets its evidence: the unit runs
 `native:test:container` (the image build, the compile, the suite and Clippy), writes its log to the gate log folder, and
 ends its log with the exit status the unit wrote, which `native:gate:verdict` reads. The command answers as soon as
 systemd has queued the unit, so nothing waits here for an hour-long run.
 */
import { homedir } from 'node:os';

import {
  gateCapsFromOptions,
  gateCapsOptionNames,
  gateOptions,
} from './native-gate-arguments.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  printLine,
  runGateMain,
} from './native-gate-main.ts';
import {
  gateLogDirectoryOf,
  gateRepositoryOf,
} from './native-gate-paths.ts';
import { startGateUnit } from './native-gate-start.ts';

/**
 Option that names the checkout the gate worktree belongs to.
 */
const repositoryOption = '--repository';

/**
 Option that names the folder the gate's log is written to.
 */
const logDirectoryOption = '--log-directory';

/**
 How the command is called, printed by every refusal that reads the command line.
 */
const usage = 'usage: native:gate [--repository <path>] [--log-directory <path>] [--unit-cpu-quota <value>] [--unit-memory-max <value>] [--unit-tasks-max <count>] [--unit-io-weight <weight>] [--container-pids-limit <count>] <worktree name under the worktrees folder, or its full path> <label>';

/**
 Start one gate and print the unit and log it was started with.

 @returns zero once the unit is queued

 @throws NativeGateError when the command line, the label, the worktree or the caps cannot start a gate

 @example
 ```ts
 process.exitCode = await runGateMain({ main });
 ```
 */
async function main(): Promise<number> {
  /**
   The command line, split into its options and its two positionals.
   */
  const options = gateOptions({
    argv: process.argv
      .slice(2),
    knownOptions: [
      repositoryOption,
      logDirectoryOption,
      ...gateCapsOptionNames,
    ],
  });
  /**
   The worktree to gate and what it covers.
   */
  const [worktree, label, ...rest] = options.positionals;
  if ((worktree === undefined) || (label === undefined))
    throw new NativeGateError(`native:gate needs a worktree and a label. ${usage}`);
  if (rest.length > 0)
    throw new NativeGateError(`native:gate takes one worktree and one label, and "${rest.join(' ')}" is neither. ${usage}`);
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
  await startGateUnit({
    repository: await gateRepositoryOf({
      environment: process.env,
      ...(namedRepository === undefined ? {} : { named: namedRepository, }),
    }),
    worktree,
    label,
    logDirectory: gateLogDirectoryOf({
      environment: process.env,
      home: homedir(),
      ...(namedLogDirectory === undefined ? {} : { named: namedLogDirectory, }),
    }),
    caps: gateCapsFromOptions({ options, }),
    moment: new Date(),
    print: printLine,
  });
  return 0;
}

if (import.meta.main)
  process.exitCode = await runGateMain({ main, });
