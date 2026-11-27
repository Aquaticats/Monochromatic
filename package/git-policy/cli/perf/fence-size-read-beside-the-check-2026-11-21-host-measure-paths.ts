/**
 Where the 2026-11-21 host measurement keeps its fixture, which builds its arms run, and what one add of it is.

 The paths are the ones the run that took the measurement used, so a later run of the same script measures the same tree in
 the same place; the arms name the two builds it compares.
 */
import { join, } from 'node:path';

/**
 The scratch root the 2026-11-21 run kept: the tar of the source tree and the two builds' folders.
 */
export const SCRATCH: string = '/home/user/temp/agent/fence-size-read-beside-source-check-2026-11-21/scratch';

/**
 The fixture every phase builds: the source, the destinations and the homes the adds run with.
 */
export const ROOT: string = join(
  SCRATCH,
  'host-zfs',
);

/**
 The source repository every add is made from.
 */
export const SOURCE: string = join(
  ROOT,
  'src',
);

/**
 The destinations the adds write, one directory per add.
 */
export const DESTS: string = join(
  ROOT,
  'dests',
);

/**
 The homes and runtime directories the adds run with, one pair per add.
 */
export const HOMES: string = join(
  ROOT,
  'homes',
);

/**
 The tar of this repository's tracked tree the fixture source is extracted from.
 */
export const TAR: string = join(
  SCRATCH,
  'tree-40783c23ea.tar',
);

/**
 The file the wrapper appends one line per add to.
 */
export const RECORD: string = join(
  SOURCE,
  '.git',
  'monochromatic-git-policy-cli-worktree-add-timing.jsonl',
);

/**
 The home the fixture-building Git runs take.
 */
export const PLAIN_HOME: string = join(
  HOMES,
  'plain',
);

/**
 The host's own runtime directory, which the fixture-building Git runs name rather than making one.
 */
export const hostRuntimeDirectory = '/run/user/1000';

/**
 The two builds the arms run, by the key each arm names: this host's build of `main`, and the measurement build of the
 branch.
 */
export const BINARIES: Readonly<Record<string, string>> = {
  main: join(
    SCRATCH,
    'target-main',
    'release',
    'monochromatic-git-policy-cli',
  ),
  branch: join(
    SCRATCH,
    'bin',
    'branch-measurement',
  ),
};

/**
 The variable one binary reads to start the size read beside the source check (measurement builds only).
 */
export const SWITCH = 'MONOCHROMATIC_CLONE_SIZE_READ_BESIDE';

/**
 How many nanoseconds make a millisecond, which `process.hrtime.bigint()` reports its reading in.
 */
export const nanosecondsPerMillisecond = 1e6;

/**
 The longest one add may run before it and everything it started are stopped.
 */
export const addDeadlineMilliseconds = 300_000;

/**
 How much of a failed add's standard error its row carries.
 */
export const stderrTailInRow = 300;

/**
 How many rounds the `adds` phase makes when the run names none.
 */
export const defaultRounds = 8;

/**
 How many rounds the `refused` phase makes when the run names none.
 */
export const defaultRefusedRounds = 2;

/**
 One arm of the measurement: the binary it runs and whether the size read starts beside the check.
 */
export type Arm = {
  /**
   The arm's name, which every row and every file it writes carries.
   */
  readonly name: string;
  /**
   Which build the arm runs, by the key `BINARIES` holds it under.
   */
  readonly binary: string;
  /**
   Whether this arm's binary starts the size read beside the source check.
   */
  readonly overlap: boolean;
};

/**
 The arms every phase measures, in the order they are named.
 */
export const ARMS: readonly Arm[] = [
  {
    name: 'main',
    binary: 'main',
    overlap: false,
  },
  {
    name: 'branch-serial',
    binary: 'branch',
    overlap: false,
  },
  {
    name: 'branch-overlap',
    binary: 'branch',
    overlap: true,
  },
  {
    name: 'branch-serial-again',
    binary: 'branch',
    overlap: false,
  },
];

/**
 One add a phase makes, which is every field of its row that the phase decides.
 */
export type Addition = {
  /**
   The arm the add runs.
   */
  readonly arm: Arm;
  /**
   The add's own tag, unique within the phase.
   */
  readonly tag: string;
  /**
   The layout the phase measures.
   */
  readonly layout: string;
  /**
   Whether the add counts towards the phase's medians.
   */
  readonly counted: boolean;
  /**
   The counted round the add belongs to, absent for a warm-up or a probe.
   */
  readonly round?: number;
  /**
   The add's place in that round, absent for a warm-up or a probe.
   */
  readonly position?: number;
};

/**
 What one add runs with, and the two directories that go with it.
 */
export type ArmEnvironment = {
  /**
   The environment the add runs with: a shim of its arm's binary, its own home and its own runtime directory.
   */
  readonly env: NodeJS.ProcessEnv;
  /**
   The home the add writes, which is removed with the add.
   */
  readonly home: string;
  /**
   The runtime directory the add writes, which is removed with the add.
   */
  readonly runtime: string;
};

/**
 The name of one arm.

 @param arm - the arm to name

 @returns the name every row and file of that arm carries

 @example
 ```ts
 nameOf(ARMS[0]); // 'main'
 ```
 */
export function nameOf(arm: Arm): string {
  return arm.name;
}
