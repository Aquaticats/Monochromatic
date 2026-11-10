/**
 Where every command of this package puts and finds a run: the folder that holds the run folders, the worktree a run works in,
 the unit systemd runs it as, and the review folder a Kimi review writes to.

 The conventions are the ones the commands of `.agents/skills/pi-as-subagent` use, so a run started by one command is found by
 the others: the unit is `pi-<name>.service`, each attempt writes `<name>-<number>`, and the worktree is
 `<repository>/.claude/worktrees/pi-<name>`.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';

/**
 Folder under the home directory that holds one run folder per run, which the skill recipe names.
 */
const RUNS_FOLDER_UNDER_HOME = 'temp/agent/pi-runs';

/**
 The checkout folder under the home directory. `/var/home/user/Monochromatic` is a link to this machine's repository, and the
 skill starts every run from it.
 */
const REPOSITORY_UNDER_HOME = 'Monochromatic';

/**
 Folder under the home directory where a Kimi review keeps its diff, its reproductions and its answer.
 */
const KIMI_REVIEWS_FOLDER_UNDER_HOME = 'temp/agent/kimi-reviews';

/**
 Folder inside the repository that holds one worktree per run.
 */
const WORKTREES_FOLDER_IN_REPOSITORY = '.claude/worktrees';

/**
 What a run's worktree and run folder names start with, after the worktrees folder and before the run's name.
 */
const RUN_PREFIX = 'pi-';

/**
 The folder under the runs folder that holds what a coordinator publishes into a run's inbox.
 */
const INBOX_FOLDER = 'inbox';

/**
 Driver entry, relative to the repository. It is run from source, so a worktree started by this command needs no build of this
 package, and every file it loads is checked in.
 */
const DRIVER_ENTRY_IN_REPOSITORY = 'package/pi-shared/subagent-driver/src/drive-pi-subagent.ts';

/**
 The character between a run's name and the number of one attempt.
 */
const ATTEMPT_SEPARATOR = '-';

/**
 The end of a run folder's prompt file.
 */
const PROMPT_FILE_SUFFIX = '-prompt.md';

/**
 Where the runs registered here live when the caller names no folder.

 @param home - the home directory to read them under

 @returns the runs folder

 @example
 ```ts
 const runsFolder = defaultRunsFolder({ home: '/home/a', });
 ```
 */
export function defaultRunsFolder({
  home = homedir()
}: {
  readonly home?: string;
},): string {
  return join(
    home,
    RUNS_FOLDER_UNDER_HOME,
  );
}

/**
 Where this machine keeps the repository, when the caller names no folder.

 @param home - the home directory to read it under

 @returns the repository folder

 @example
 ```ts
 const repository = defaultRepository({ home: '/home/a', });
 ```
 */
export function defaultRepository({
  home = homedir()
}: {
  readonly home?: string;
},): string {
  return join(
    home,
    REPOSITORY_UNDER_HOME,
  );
}

/**
 Where a Kimi review keeps what it wrote, when the caller names no folder.

 @param home - the home directory to read it under

 @returns the reviews folder

 @example
 ```ts
 const reviewsFolder = defaultKimiReviewsFolder({ home: '/home/a', });
 ```
 */
export function defaultKimiReviewsFolder({
  home = homedir()
}: {
  readonly home?: string;
},): string {
  return join(
    home,
    KIMI_REVIEWS_FOLDER_UNDER_HOME,
  );
}

/**
 Names one attempt's run folder.

 @param name - the run's name

 @param run - the number of the attempt

 @returns the folder's name, which the runs folder holds

 @example
 ```ts
 const folderName = runFolderNameOf({ name: 'copy-route', run: 2, });
 ```
 */
export function runFolderNameOf({
  name,
  run,
}: {
  readonly name: string;
  readonly run: number;
},): string {
  return `${name}${ATTEMPT_SEPARATOR}${String(run,)}`;
}

/**
 Names one attempt's run folder with the runs folder in front of it.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param name - the run's name

 @param run - the number of the attempt

 @returns the run folder's path

 @example
 ```ts
 const folder = runFolderPathOf({ runsFolder: '/runs', name: 'copy-route', run: 1, });
 ```
 */
export function runFolderPathOf({
  runsFolder,
  name,
  run,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly run: number;
},): string {
  return join(
    runsFolder,
    runFolderNameOf({
      name,
      run,
    },),
  );
}

/**
 Where a coordinator publishes a message for one run to read.

 @param runFolder - the run folder of the attempt going now

 @returns the inbox folder, which the driver creates and reads

 @example
 ```ts
 const inbox = inboxOfARun({ runFolder: '/runs/copy-route-1', });
 ```
 */
export function inboxOfARun({
  runFolder,
}: {
  readonly runFolder: string;
},): string {
  return join(
    runFolder,
    INBOX_FOLDER,
  );
}

/**
 Names the prompt file of a run, which is what a coordinator writes before starting it and what the batch list defaults to.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param name - the run's name

 @returns the prompt file's path

 @example
 ```ts
 const prompt = promptFileOf({ runsFolder: '/runs', name: 'copy-route', });
 ```
 */
export function promptFileOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): string {
  return join(
    runsFolder,
    `${name}${PROMPT_FILE_SUFFIX}`,
  );
}

/**
 Where one run works, which is the folder its branch is checked out in.

 @param repository - the checkout that holds the worktrees folder

 @param name - the run's name

 @returns the worktree's path

 @example
 ```ts
 const worktree = worktreeOfARun({ repository: '/repo', name: 'copy-route', });
 ```
 */
export function worktreeOfARun({
  repository,
  name,
}: {
  readonly repository: string;
  readonly name: string;
},): string {
  return join(
    repository,
    WORKTREES_FOLDER_IN_REPOSITORY,
    `${RUN_PREFIX}${name}`,
  );
}

/**
 The driver entry this package starts for each run.

 @param repository - the checkout that holds this package

 @returns the driver entry's path

 @example
 ```ts
 const entry = driverEntryOf({ repository: '/repo', });
 ```
 */
export function driverEntryOf({
  repository,
}: {
  readonly repository: string;
},): string {
  return join(
    repository,
    DRIVER_ENTRY_IN_REPOSITORY,
  );
}
