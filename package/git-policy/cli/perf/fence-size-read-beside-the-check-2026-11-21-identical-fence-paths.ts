/**
 Where the 2026-11-21 identical-fence control keeps its fixture, and the numbers its fixture and its adds are made with.
 */
import { join, } from 'node:path';

/**
 The scratch root the 2026-11-21 control kept.
 */
export const SCRATCH: string = '/home/user/temp/agent/fence-size-read-beside-source-check-2026-11-21/scratch';

/**
 The fixture this control builds: the source, the shims and the two destinations.
 */
export const ROOT: string = join(
  SCRATCH,
  'control',
);

/**
 The source repository both arms' adds are made from.
 */
export const SOURCE: string = join(
  ROOT,
  'src',
);

/**
 The measurement build of the branch, which both arms run through a shim.
 */
export const BRANCH: string = join(
  SCRATCH,
  'bin',
  'branch-measurement',
);

/**
 The variable one binary reads to start the size read beside the source check (measurement builds only).
 */
export const SWITCH = 'MONOCHROMATIC_CLONE_SIZE_READ_BESIDE';

/**
 The home every command of this control runs with.
 */
export const HOME: string = join(
  ROOT,
  'home',
);

/**
 The runtime directory every command of this control runs with.
 */
export const RUNTIME: string = join(
  ROOT,
  'runtime',
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
 How many files of each kind the fixture holds.
 */
export const filesPerKind = 12;

/**
 Every this-many-th text file carries a carriage return, so the text rule and the size proof both matter.
 */
export const carriageReturnEvery = 3;

/**
 How many seconds the fixture waits for a fresh index's stat fields to settle before the check may read it.
 */
export const settleSeconds = 8;

/**
 How many of an add's fence lines this control keeps.
 */
export const verboseFenceLines = 3;

/**
 The mode the shim's copy of the binary needs, which is what makes it the `git` the add runs.
 */
export const executableMode = 0o755;
