/**
 Every worktree of the repository with what is known about it: the branch it holds, how much of that branch `main` lacks, whether a
 unit still runs it, and whether anything would be lost by removing it.

 Nothing is removed on an answer a machine could not give: a process table or a unit list that cannot be read keeps every
 worktree, because a worktree a running agent works in must never go.

 @module
 */

import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  ignoredEntriesOf,
  refsHolding,
  type Repository,
  statusOf,
  worktreeRecords,
} from './git-commands.ts';
import {
  isKnownBuildOutput,
  MILLISECONDS_PER_DAY,
} from './constants.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';
import type { MachineState, } from './machine-state.ts';

/**
 What is known about one worktree, and whether removing it would lose anything.
 */
export type WorktreeReport = {
  /**
   Its folder.
   */
  readonly path: string;

  /**
   Its folder's own name, which is what a person calls it.
   */
  readonly name: string;

  /**
   The branch it holds, or empty text when its HEAD is detached.
   */
  readonly branch: string;

  /**
   How many commits its branch has that `main` does not, absent when it holds no branch.
   */
  readonly commitsMainLacks?: number;

  /**
   Whether a `pi-` unit is still running for it.
   */
  readonly runningUnit: boolean;

  /**
   Why it must stay, absent when removing it would lose nothing.
   */
  readonly reasonToKeep?: string;
};

/**
 Longest text of a path kept on a report line, so one long path cannot fill a log.
 */
const PATH_CHARACTERS = 80;

/**
 Characters Git puts before a path in its status: the two status letters and a space.
 */
const STATUS_PREFIX_CHARACTERS = 3;

/**
 Reads when a worktree was last touched, which for a real worktree is its own Git index.

 @param path - the worktree's folder

 @returns the modification time in milliseconds, or why it cannot be judged

 @example
 ```ts
 const touched = await lastTouchedMilliseconds('/repository/.claude/worktrees/a',);
 ```
 */
async function lastTouchedMilliseconds(path: string,): Promise<Looked<number>> {
  /**
   The worktree's own Git pointer.
   */
  const pointer = join(
    path,
    '.git',
  );
  try {
    /**
     What the pointer names, which is the folder holding this worktree's index.
     */
    const named = (await readFile(
      pointer,
      'utf8',
    )).replaceAll(
      'gitdir: ',
      ''
    )
      .trim();
    return found((await stat(join(
      named,
      'index',
    ),)).mtimeMs,);
  }
  catch (caught) {
    // A worktree whose index cannot be read still has its own folder, whose time is when it was last written.
    return missing(caughtValueText(caught,),);
  }
}

/**
 Finds the change in a worktree that a person would lose, apart from the outputs a package install or a compile leaves.

 @param repository - the worktree

 @returns the path that would be lost, or why nothing would be

 @example
 ```ts
 if ((await lostPathIn(worktree,)).found) return;
 ```
 */
async function lostPathIn(repository: Repository,): Promise<Looked<string>> {
  /**
   What the worktree's repository reports.
   */
  const status = await statusOf(repository,);
  if (status.status !== 0) {
    return found(`git status failed with status ${String(status.status,)}: ${status.stderr
      .trim()
      .slice(
        0,
        PATH_CHARACTERS,
      )}`,);
  }
  /**
   The changed or untracked paths Git named, without the two status letters Git puts before each.
   */
  const changed = status.stdout
    .split('\n',)
    .map(function pathOf(line: string,) {
      return line.slice(STATUS_PREFIX_CHARACTERS,);
    },)
    .filter(function isPath(path: string,) {
      return path.trim() !== '';
    },);
  /**
   The first of them that is not a build output.
   */
  const kept = changed.find(function isNotABuildOutput(path: string,) {
    return !isKnownBuildOutput(path,);
  },);
  return kept === undefined ? missing('every changed path is a build output',) : found(kept,);
}

/**
 Describes one worktree and judges whether removing it would lose anything.

 @param repository - the main checkout

 @param record - the worktree as Git lists it

 @param machine - everything the judgement reads of the machine

 @param worktreesFolder - the folder agent worktrees live in

 @returns what is known about the worktree

 @example
 ```ts
 const report = await reportOf({ repository, record, machine, worktreesFolder, },);
 ```
 */
async function reportOf({
  repository,
  record,
  machine,
  worktreesFolder,
}: {
  readonly repository: Repository;
  readonly record: {
    readonly path: string;
    readonly head: string;
    readonly branch: string;
    readonly locked: boolean;
    readonly detached: boolean;
  };
  readonly machine: MachineState;
  readonly worktreesFolder: string;
},): Promise<WorktreeReport> {
  /**
   The worktree's own name, which its unit is named after.
   */
  const name = record.path
    .split('/',)
    .at(-1,)
    ?? record.path;
  /**
   How many commits its branch has that `main` does not.
   */
  const commitsMainLacks = record.branch === '' ? missing('it holds no branch',) : await numberFrom({
    repository,
    words: [
      'rev-list',
      '--count',
      `main..${record.branch}`,
    ],
  },);
  return {
    path: record.path,
    name,
    branch: record.branch,
    ...(commitsMainLacks.found ? { commitsMainLacks: commitsMainLacks.value, } : {}),
    runningUnit: machine.activeUnits
      .found
      && machine.activeUnits
        .value
        .includes(`${name}.service`,),
    ...await reasonToKeepOf({
      repository,
      record,
      name,
      machine,
      worktreesFolder,
      commitsMainLacks,
    },),
  };
}

/**
 Reads a number Git printed, absent when Git refused the question.

 @param repository - the checkout to ask

 @param words - the Git command and its arguments

 @returns the number Git printed, absent when it printed none

 @example
 ```ts
 const ahead = await numberFrom({ repository, words: ['rev-list', '--count', 'main..fix/a',], },);
 ```
 */
async function numberFrom({
  repository,
  words,
}: {
  readonly repository: Repository;
  readonly words: readonly string[];
},): Promise<Looked<number>> {
  /**
   What Git printed.
   */
  const asked = await repository.run({
    program: 'git',
    words,
    folder: repository.folder,
    timeoutMilliseconds: repository.timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
  return asked.status === 0
    ? found(
      Number(asked.stdout
        .trim(),),
    )
    : missing(`git ended with status ${String(asked.status,)}`,);
}

/**
 Decides why a worktree must stay.

 @param repository - the main checkout

 @param record - the worktree as Git lists it

 @param name - the worktree's own name

 @param machine - everything the judgement reads of the machine

 @param worktreesFolder - the folder agent worktrees live in

 @param commitsMainLacks - how many commits its branch has that `main` does not, or why that cannot be read

 @returns the reason to keep it, as the report's own field, or nothing when it may go

 @example
 ```ts
 const kept = await reasonToKeepOf({ repository, record, name, machine, worktreesFolder, commitsMainLacks, },);
 ```
 */
async function reasonToKeepOf({
  repository,
  record,
  name,
  machine,
  worktreesFolder,
  commitsMainLacks,
}: {
  readonly repository: Repository;
  readonly record: {
    readonly path: string;
    readonly head: string;
    readonly branch: string;
    readonly locked: boolean;
    readonly detached: boolean;
  };
  readonly name: string;
  readonly machine: MachineState;
  readonly worktreesFolder: string;
  readonly commitsMainLacks: Looked<number>;
},): Promise<{ readonly reasonToKeep?: string }> {
  /**
   The reason that keeps it, when one is found.
   */
  const reason = await whyItStays({
    repository,
    record,
    name,
    machine,
    worktreesFolder,
    commitsMainLacks,
  },);
  return reason.found ? { reasonToKeep: reason.value, } : {};
}

/**
 Finds the reason one worktree must stay.

 @param repository - the main checkout

 @param record - the worktree as Git lists it

 @param name - the worktree's own name

 @param machine - everything the judgement reads of the machine

 @param worktreesFolder - the folder agent worktrees live in

 @param commitsMainLacks - how many commits its branch has that `main` does not, or why that cannot be read

 @returns the reason it must stay, or why nothing keeps it

 @example
 ```ts
 const why = await whyItStays({ repository, record, name, machine, worktreesFolder, commitsMainLacks: 0, },);
 ```
 */
async function whyItStays({
  repository,
  record,
  name,
  machine,
  worktreesFolder,
  commitsMainLacks,
}: {
  readonly repository: Repository;
  readonly record: {
    readonly path: string;
    readonly head: string;
    readonly branch: string;
    readonly locked: boolean;
    readonly detached: boolean;
  };
  readonly name: string;
  readonly machine: MachineState;
  readonly worktreesFolder: string;
  readonly commitsMainLacks: Looked<number>;
},): Promise<Looked<string>> {
  if (!record.path
    .startsWith(`${worktreesFolder}/`,)) return missing('it is not an agent worktree',);
  if (record.locked) return found('git holds it locked',);
  if (!machine.activeUnits
    .found) return found(`the units running now cannot be read: ${machine.activeUnits
      .why}`,);
  if (machine.activeUnits
    .value
    .includes(`${name}.service`,)) return found('a Pi unit still runs it',);
  if (!machine.workingFolders
    .found) {
    return found(`the processes on this machine cannot be read: ${machine.workingFolders
      .why}`,);
  }
  if (machine.workingFolders
    .value
    .some(function isInside(folder: string,) {
      return (folder === record.path) || folder.startsWith(`${record.path}/`,);
    },)) return found('a process works inside it',);
  /**
   When the worktree was last touched.
   */
  const touched = await lastTouchedMilliseconds(record.path,);
  if (touched.found && ((machine.nowMilliseconds - touched.value) < MILLISECONDS_PER_DAY)) {
    return found('touched within a day',);
  }
  /**
   The worktree's own repository, which is what its changes are read from.
   */
  const worktree: Repository = {
    ...repository,
    folder: record.path,
  };
  /**
   The path a person would lose by removing it.
   */
  const lost = await lostPathIn(worktree,);
  if (lost.found) return found(`it holds a changed or untracked path: ${lost.value
    .slice(
      0,
      PATH_CHARACTERS,
    )}`,);
  if (record.detached) {
    /**
     What holds the commit the worktree is parked on, which must be some ref before it may go.
     */
    const holders = await refsHolding({
      repository,
      commit: record.head,
    },);
    if (holders === '') return found('its commit is on no branch',);
  }
  if (commitsMainLacks.found && (commitsMainLacks.value !== 0)) {
    return found(`${String(commitsMainLacks.value,)} commits main lacks`,);
  }
  /**
   The ignored entries the worktree holds, which must all be build outputs.
   */
  const ignored = await ignoredEntriesOf(worktree,);
  /**
   The first ignored entry that is not a build output.
   */
  const unknown = ignored.find(function isNotABuildOutput(entry: string,) {
    return !isKnownBuildOutput(entry.replace(
      'Would remove ',
      '',
    )
      .replace(
      'Would delete ',
      '',
      ),);
  },);
  return unknown === undefined
    ? missing('nothing it holds is worth keeping',)
    : found(`an ignored entry that is not a build output: ${unknown.slice(
      0,
      PATH_CHARACTERS,
    )}`,);
}

/**
 Reports on every worktree of the repository.

 @param repository - the main checkout

 @param machine - everything the judgement reads of the machine

 @param worktreesFolder - the folder agent worktrees live in

 @returns one report per worktree, in the order Git lists them

 @example
 ```ts
 const reports = await reportOnWorktrees({ repository, machine, worktreesFolder: '/repository/.claude/worktrees', },);
 ```
 */
export async function reportOnWorktrees({
  repository,
  machine,
  worktreesFolder,
}: {
  readonly repository: Repository;
  readonly machine: MachineState;
  readonly worktreesFolder: string;
},): Promise<readonly WorktreeReport[]> {
  /**
   Every worktree Git lists.
   */
  const records = await worktreeRecords(repository,);
  /**
   Each worktree's own report, in that order, one after another so a long listing of Git questions does not flood the host.
   */
  const reports: WorktreeReport[] = [];
  for (const record of records) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one worktree's four Git questions at a time, see doc/lint/sequential-lands.md
    reports.push(await reportOf({
      repository,
      record,
      machine,
      worktreesFolder,
    },),);
  }
  return reports;
}
