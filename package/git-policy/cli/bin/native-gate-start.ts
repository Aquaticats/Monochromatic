/**
 Starts one release gate as its own transient systemd unit and answers where the unit and its log are.

 The unit is started and this answers at once: the gate runs for the better part of an hour (`native:test:host:btrfs`'s
 own deadline sizes a full run from 3286 measured seconds), so whoever started it goes on to other work and reads the
 verdict from the log with `native:gate:verdict`. A worktree that holds no `package/git-policy/cli` stops the launch
 here, because systemd would otherwise start a unit that fails at `CHDIR` with an empty log.
 */
import { mkdir } from 'node:fs/promises';
import {
  join,
  resolve
} from 'node:path';

import { systemdRunProgram } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/constants.ts';

import type { GateCaps } from './native-gate-caps.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  gateLogPath,
  gateWorktreePath,
  pathKindOf,
} from './native-gate-paths.ts';
import { runGateCommandOrThrow } from './native-gate-run.ts';
import { gateUnitArguments } from './native-gate-unit.ts';
import {
  gateLabelProblems,
  gateUnitName,
  localStamp,
} from './native-gate-unit-name.ts';

/**
 Milliseconds a start may take before it is stopped.

 `systemd-run` queues the unit and answers, which took well under a second on every recorded start; the bound is the one
 the scratch relaunch gave the whole launch, and it exists so a `systemd-run` that never answers ends as a refusal
 instead of holding the caller.
 */
export const gateStartDeadlineMilliseconds = 120_000;

/**
 Where the cwd of a worktree shows whether the package that is gated is there.
 */
const gatePackageSegments = [
  'package',
  'git-policy',
  'cli',
] as const;

/**
 The unit and log one started gate answers with.
 */
export type StartedGate = {
  /**
   The unit's name, without the `.service` suffix.
   */
  readonly unit: string;
  /**
   Absolute path of the gate's log file.
   */
  readonly log: string;
  /**
   Absolute path of the worktree the gate runs in.
   */
  readonly worktree: string;
};

/**
 Starts one gate unit.

 @param repository - the checkout the worktree belongs to

 @param worktree - the worktree's name under the worktrees folder, or its absolute path

 @param label - what the gate covers, in lowercase letters, digits and hyphens

 @param logDirectory - the folder the gate's log is written to

 @param caps - the caps of the unit and of its container

 @param moment - the moment the unit name is stamped with

 @param print - prints one line of the launch's own output

 @returns the unit name, the log path and the worktree the gate runs in

 @throws NativeGateError when the label cannot name a unit, or when the worktree holds nothing to gate

 @example
 ```ts
 await startGateUnit({ repository: '/repo', worktree: 'gate-wrapper-next-alpha', label: 'wrapper-next-alpha', logDirectory: '/l', caps: defaultGateCaps, moment: new Date(), print: console.log });
 ```
 */
export async function startGateUnit({
  repository,
  worktree,
  label,
  logDirectory,
  caps,
  moment,
  print,
}: {
  readonly repository: string;
  readonly worktree: string;
  readonly label: string;
  readonly logDirectory: string;
  readonly caps: GateCaps;
  readonly moment: Date;
  readonly print: (line: string) => void;
}): Promise<StartedGate> {
  /**
   Every reason the label cannot name a unit.
   */
  const labelProblems = gateLabelProblems({ label, });
  if (labelProblems.length > 0)
    throw new NativeGateError(labelProblems.join('\n'));
  /**
   The worktree the unit runs in.
   */
  const worktreePath = gateWorktreePath({
    repository,
    worktree,
  });
  /**
   Whether the worktree holds the package a gate works on.
   */
  const holdsPackage = (await pathKindOf({ path: join(
    worktreePath,
    ...gatePackageSegments
  ), })) !== 'missing';

  if (!holdsPackage)
    throw new NativeGateError(`${worktreePath} holds no ${gatePackageSegments.join('/')}, so there is nothing there to gate; pass a worktree name under the repository's worktrees folder or the worktree's full path.`);
  /**
   The unit's name, which the log, the image tag and the journal all carry.
   */
  const unit = gateUnitName({
    label,
    stamp: localStamp({ moment }),
  });
  /**
   The gate's log file.
   */
  const log = gateLogPath({
    logDirectory,
    unitName: unit,
  });
  // The log folder is made here, because systemd creates the log file at the path it is given but never the folder.
  await mkdir(
    resolve(logDirectory),
    { recursive: true, },
  );
  /**
   What `systemd-run` printed while it queued the unit.
   */
  const started = await runGateCommandOrThrow({
    program: systemdRunProgram,
    args: gateUnitArguments({
      name: unit,
      worktree: worktreePath,
      log,
      caps,
    }),
    deadlineMilliseconds: gateStartDeadlineMilliseconds,
  });
  for (const line of started.stdout
    .split('\n'))
    if (line.trim() !== '')
      print(line);
  for (const line of started.stderr
    .split('\n'))
    if (line.trim() !== '')
      print(line);
  print(`${unit}.service=${log}`);
  return {
    unit,
    log,
    worktree: worktreePath,
  };
}
