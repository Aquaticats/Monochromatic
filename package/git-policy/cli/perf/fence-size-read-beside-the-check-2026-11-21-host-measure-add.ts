/**
 One add of the 2026-11-21 host measurement: the shim, home and runtime directory it runs with, the command it makes, and
 the row it leaves behind.

 The add is the measurement's unit, so this module is where the command, the clock and the record meet: the wall time is
 taken around the command itself, and everything else the row carries comes from the record the add appended for itself.
 */
import {
  cp,
  mkdir,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { runCommand, } from '../bin/native-verification-process.ts';
import {
  type AddTimingRecord,
  lastAddTimingRecordOf,
} from '../bin/fence-size-read-beside-the-check-2026-11-21-add-timing-record.ts';
import {
  ARMS,
  type Addition,
  type Arm,
  type ArmEnvironment,
  BINARIES,
  DESTS,
  HOMES,
  nanosecondsPerMillisecond,
  addDeadlineMilliseconds,
  nameOf,
  RECORD,
  SOURCE,
  stderrTailInRow,
  SWITCH,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-paths.ts';
import {
  durationsOf,
  orderOfRound,
  refusedOrderOfRound,
  type Row,
} from '../bin/fence-size-read-beside-the-check-2026-11-21-host-measure-phase.ts';

/**
 What one add runs with: a shim of its arm's binary, a home and a runtime directory of its own.

 @param arm - the arm the add runs

 @param tag - the add's own tag, which its home and runtime directory are named after

 @returns the environment and the two directories of that one add

 @throws Error when the arm names a build the measurement has no path for

 @example
 ```ts
 await armEnvironment({ arm: ARMS[0], tag: 'r1p1' });
 ```
 */
export async function armEnvironment({
  arm,
  tag,
}: {
  readonly arm: Arm;
  readonly tag: string;
}): Promise<ArmEnvironment> {
  /**
   The directory the arm's binary is put in under the name `git`.
   */
  const shim = join(
    HOMES,
    `shim-${arm.name}`,
  );
  await mkdir(
    shim,
    { recursive: true, },
  );
  /**
   The build this arm runs.
   */
  const binary = BINARIES[arm.binary];
  if (binary === undefined) throw new Error(`the arm ${arm.name} names no build`);
  await rm(
    join(
      shim,
      'git',
    ),
    { force: true, },
  );
  await cp(
    binary,
    join(
      shim,
      'git',
    ),
  );
  /**
   The home this add runs with.
   */
  const home = join(
    HOMES,
    `${arm.name}-${tag}`,
  );
  await mkdir(
    home,
    { recursive: true, },
  );
  /**
   The runtime directory this add runs with, private to the account as a session's own is.
   */
  const runtime = join(
    HOMES,
    `runtime-${arm.name}-${tag}`,
  );
  await mkdir(
    runtime,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  return {
    env: {
      PATH: `${shim}:/usr/bin:/bin:/usr/sbin:/sbin`,
      HOME: home,
      XDG_RUNTIME_DIR: runtime,
      GIT_CONFIG_NOSYSTEM: '1',
      ...(arm.overlap ? { [SWITCH]: '1', } : {}),
    },
    home,
    runtime,
  };
}

/**
 The record the add that just ended wrote.

 @returns the last add's own account of itself

 @throws Error when the record file holds no line, or when its last line is not one the wrapper wrote
 */
async function lastRecord(): Promise<AddTimingRecord> {
  return lastAddTimingRecordOf({ text: await readFile(
    RECORD,
    'utf8',
  ), },);
}

/**
 Makes one add and reads what it left behind.

 @param addition - the arm, tag, layout and round this add belongs to

 @returns the add's row: the time this host measured around the command and the record's own figures

 @example
 ```ts
 oneAdd({ arm: ARMS[0], tag: 'r1p1', layout: 'loose', counted: true, round: 1, position: 1 });
 ```
 */
export async function oneAdd(addition: Addition): Promise<Row> {
  /**
   The environment this add runs with and the two directories that go with it.
   */
  const environment = await armEnvironment({
    arm: addition.arm,
    tag: addition.tag,
  },);
  /**
   The destination this add writes.
   */
  const destination = join(
    DESTS,
    `${addition.arm
      .name}-${addition.tag}`,
  );
  /**
   The branch this add makes.
   */
  const branch = `wt-${addition.arm
    .name}-${addition.tag}`;
  /**
   The add's start, on the monotonic clock its wall time is measured with.
   */
  const began = process.hrtime
    .bigint();
  /**
   What the add answered.
   */
  const added = await runCommand({
    command: 'git',
    args: [
      'worktree',
      'add',
      '--quiet',
      '-b',
      branch,
      destination,
    ],
    cwd: SOURCE,
    env: environment.env,
    capture: true,
    allowFailure: true,
    deadlineMilliseconds: addDeadlineMilliseconds,
  },);
  /**
   The add's wall time in milliseconds, taken around the command itself.
   */
  const wallMs = Number(process.hrtime
    .bigint()
    - began,) / nanosecondsPerMillisecond;
  /**
   The record the add appended for itself.
   */
  const record = await lastRecord();
  /**
   Whether the add failed.
   */
  const failed = (added.status ?? 1) !== 0;
  /**
   The row this add leaves behind.
   */
  const row: Row = {
    layout: addition.layout,
    arm: addition.arm
      .name,
    overlap: addition.arm
      .overlap,
    tag: addition.tag,
    counted: addition.counted,
    wallMs: Math.round(wallMs * 10,) / 10,
    totalMs: record.totalMs,
    ...(record.notes
      .route
      === undefined ? {} : { route: record.notes
        .route, }),
    exitCode: record.exitCode,
    failed,
    stderr: failed ? added.stderr
      .slice(-stderrTailInRow,) : '',
    steps: durationsOf({ spent: record.steps, },),
    work: durationsOf({ spent: record.work, },),
    ...(addition.round === undefined ? {} : { round: addition.round, }),
    ...(addition.position === undefined ? {} : { position: addition.position, }),
  };
  await runCommand({
    command: 'git',
    args: [
      'worktree',
      'remove',
      '--force',
      destination,
    ],
    cwd: SOURCE,
    env: environment.env,
    capture: true,
    allowFailure: true,
  },);
  await runCommand({
    command: 'git',
    args: [
      'worktree',
      'prune',
    ],
    cwd: SOURCE,
    env: environment.env,
    capture: true,
    allowFailure: true,
  },);
  await runCommand({
    command: 'git',
    args: [
      'branch',
      '-D',
      branch,
    ],
    cwd: SOURCE,
    env: environment.env,
    capture: true,
    allowFailure: true,
  },);
  await rm(
    destination,
    {
      recursive: true,
      force: true,
    },
  );
  await rm(
    environment.home,
    {
      recursive: true,
      force: true,
    },
  );
  await rm(
    environment.runtime,
    {
      recursive: true,
      force: true,
    },
  );
  return row;
}

/**
 The arm one name of a round's order belongs to.

 @param name - the arm name the order names

 @param round - the round's number, so a refusal names the round that could not find it

 @returns the arm

 @throws Error when this measurement has no arm of that name
 */
function armNamed({
  name,
  round,
}: {
  readonly name: string;
  readonly round: number;
}): Arm {
  /**
   The arm that name belongs to.
   */
  const arm = ARMS.find(function named(candidate) {
    return candidate.name === name;
  },);
  if (arm === undefined) throw new Error(`the order of round ${String(round,)} names no arm ${name}`);
  return arm;
}

/**
 The arms of one counted round, in the order that round takes them.

 @param round - the round's number, counted from zero

 @returns the arms, in the order their adds are made

 @example
 ```ts
 countedArmsOfRound({ round: 0 }); // the four arms in the order they are named
 ```
 */
export function countedArmsOfRound({ round, }: { readonly round: number; }): readonly Arm[] {
  return orderOfRound({
    round,
    armNames: ARMS.map(function nameOfArm(arm) {
      return nameOf(arm,);
    },),
  },)
    .map(function armOfName(name) {
      return armNamed({
        name,
        round,
      },);
    },);
}

/**
 The arms of one refused-source round, in the order that round takes them.

 @param round - the round's number, counted from zero

 @returns the arms, in the order their adds are made

 @example
 ```ts
 refusedArmsOfRound({ round: 1 }); // the arms rotated by one place
 ```
 */
export function refusedArmsOfRound({ round, }: { readonly round: number; }): readonly Arm[] {
  return refusedOrderOfRound({
    round,
    armNames: ARMS.map(function nameOfArm(arm) {
      return nameOf(arm,);
    },),
  },)
    .map(function armOfName(name) {
      return armNamed({
        name,
        round,
      },);
    },);
}
