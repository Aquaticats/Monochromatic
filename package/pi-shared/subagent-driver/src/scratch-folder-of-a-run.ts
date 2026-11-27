/**
 One run's scratch folder under `${HOME}/temp/agent`, and clearing it once its run has ended: everything beside a `keep/`
 subfolder goes, and `keep/` stays whole.

 The owner on 2026-11-13: "At run end, all but keep/" (`doc/decision/owner-instructions-of-2026-11-13-after-the-restart-the-cap-of-four-and-the-scratch-rules.md`).
 Every brief names one scratch folder to its run (`doc/rules-every-pi-brief-carries.md`, rule `ATH`), and until issue 6618 nothing
 removed one: 346 folders named `*-scratch` stood under `~/temp/agent` on 2026-11-27, the oldest holding the build folders and
 checkouts of runs that ended days before, and the daily expiry of `~/temp` took `keep/` with the rest a day after a run's last
 write.

 The launcher clears it, not the command that removes finished worktrees: the launcher is the thing that started the run and it is
 still there when the unit ends, so the folder goes at the run's end rather than at the next sweep a coordinator runs by hand. The
 checks that command already makes are made here from the same evidence: a run whose unit still runs, whose own work still runs,
 or whose folder a process still works in keeps its folder whole.

 @module
 */

import {
  lstat,
  readdir,
  rm,
  rmdir,
} from 'node:fs/promises';
import {
  isAbsolute,
  join,
} from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { errorCodeOf, } from './error-code-of.ts';
import { folderSomeProcessWorksIn, } from './finished-worktrees.ts';
import { processesAtTheirFolders, } from './wake-a-run-whose-work-outlives-its-turn.ts';

//region The folder

/**
 The end of the name a run's scratch folder carries under the folder every run's scratch lives in: the brief of a run named `copy-route` names `${HOME}/temp/agent/copy-route-scratch`.
 */
export const SCRATCH_FOLDER_NAME_END = '-scratch';

/**
 The subfolder that stays when the run ends: its report, the logs and the measurements a reader of the report needs.
 */
export const KEPT_FOLDER_NAME = 'keep';

/**
 The parts of the folder every run's scratch lives in below the home directory, which rule `ATH` names as `${HOME}/temp/agent`.

 The caller joins them to the home directory it gives its run, so no path here is a username's: a test hands over a home of its own
 and no run reads the home of the account it runs under.
 */
export const AGENT_SCRATCH_FOLDER_PARTS: readonly string[] = [
  'temp',
  'agent',
];

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Names the scratch folder one run's agent writes in, or says why no folder may be named.

 A name holding a path separator would put the folder outside the folder every run's scratch lives in, and a root that is not
 absolute names no place at all, so both are refused before any folder is read or deleted: an empty variable widens what gets
 deleted (`AGENTS.md` rule `RLW`).

 @param agentScratchFolder - the folder every run's scratch lives in, which is absolute

 @param runName - the run's name, which the folder's name starts with

 @returns the folder, or the refusal in words for a reader

 @example
 ```ts
 const named = scratchFolderOfARun({ agentScratchFolder: '/home/a/temp/agent', runName: 'copy-route', },);
 ```
 */
export function scratchFolderOfARun({
  agentScratchFolder,
  runName,
}: {
  readonly agentScratchFolder: string;
  readonly runName: string;
},): { readonly folder: string } | { readonly refusal: string } {
  if (!isAbsolute(agentScratchFolder,)) {
    return {
      refusal: `the folder every run's scratch lives in is not an absolute path, so no folder of a run may be named from it (${(agentScratchFolder === '') ? 'it is empty' : agentScratchFolder})`,
    };
  }
  if (runName === '') {
    return { refusal: 'the run has no name, so no scratch folder of its own may be named', };
  }
  if (runName.includes('/',) || runName.includes('\\',)) {
    return { refusal: `the run's name ${runName} holds a path separator, so it would name a folder outside ${agentScratchFolder}`, };
  }
  return { folder: join(
    agentScratchFolder,
    `${runName}${SCRATCH_FOLDER_NAME_END}`,
  ), };
}

//endregion The folder

//region Clearing it

/**
 What became of one run's scratch folder, as the line the launcher prints about it.

 Every answer the launcher acts on is one of these rather than an error: a run whose folder cannot be cleared still ended the way
 its unit ended, and a coordinator reading the launcher's output is told what happened and why.
 */
export type ScratchFolderOutcome =
  | {
    /**
     Everything beside `keep/` was removed.
     */
    readonly action: 'cleared';

    /**
     How many entries were removed, which the folder's removal counts as one of them when it held no `keep/`.
     */
    readonly removed: number;

    /**
     The line that says what was removed.
     */
    readonly line: string;
  }
    | {
    /**
     Nothing was removed, and the folder stays as it is.
     */
      readonly action: 'kept';

    /**
     Why the folder stays, in words a coordinator acts on.
     */
      readonly reason: string;

    /**
     The line that says why, naming the folder.
     */
      readonly line: string;
    }
    | {
    /**
     The run left no scratch folder, which is no failure: a run whose brief names one that it never wrote in leaves nothing.
     */
      readonly action: 'nothing-there';

    /**
     The line that says so, naming the folder that was looked for.
     */
      readonly line: string;
    }
    | {
    /**
     Nothing was read and nothing was deleted, because no folder could be named from what the caller gave.
     */
      readonly action: 'refused';

    /**
     The line that says why, and that nothing was deleted.
     */
      readonly line: string;
    };

/**
 The line and reason of a folder that stays as it is.

 @param folder - the run's scratch folder

 @param reason - why it stays

 @returns the answer the caller prints

 @example
 ```ts
 const answer = keptScratchFolderOf({ folder: '/home/a/temp/agent/copy-route-scratch', reason: 'a process still works in it', },);
 ```
 */
function keptScratchFolderOf({
  folder,
  reason,
}: {
  readonly folder: string;
  readonly reason: string;
},): ScratchFolderOutcome {
  return {
    action: 'kept',
    reason,
    line: `the scratch folder ${folder} is kept: ${reason}`,
  };
}

/**
 Words a count of removed entries with its noun, so a line never reads `1 entries`.

 @param count - how many entries were removed

 @returns the count and its noun

 @example
 ```ts
 const text = entriesTextOf({ count: 1 });
 ```
 */
function entriesTextOf({ count, }: { readonly count: number },): string {
  return (count === 1) ? '1 entry' : `${String(count,)} entries`;
}

/**
 Reads what was removed beside `keep/`, in words for a reader.

 @param count - how many entries were removed

 @returns the clause the line carries

 @example
 ```ts
 const clause = removedClauseOf({ count: 0 });
 ```
 */
function removedClauseOf({ count, }: { readonly count: number },): string {
  return (count === 0) ? 'nothing was removed' : `${entriesTextOf({ count, })} removed`;
}

/**
 Removes one run's scratch folder, all but its `keep/` subfolder.

 The folder is read without following anything: a link under its name is kept and never followed, and a path that is not a folder
 is kept too, so nothing outside the folder is ever removed. A symbolic link inside the folder is removed as the link it is, which
 `rm` does by reading the entry itself rather than what it names.

 The folder itself goes when it holds no `keep/`: a run that left nothing worth keeping leaves no folder either, so the next run of
 that name makes one that holds only its own work.

 @param agentScratchFolder - the folder every run's scratch lives in, which is absolute

 @param runName - the run's name, which names its scratch folder

 @param unitEnded - whether the manager answered that the run's unit is not running, so the run has ended

 @param ownWorkStillRuns - whether any of the run's own work still runs, which may write in the folder

 @param processFolder - the folder holding one entry per process, `/proc` on a Linux machine

 @returns what became of the folder, and the line that says so

 @example
 ```ts
 const outcome = await clearScratchFolderOfARun({ agentScratchFolder: '/home/a/temp/agent', runName: 'copy-route', unitEnded: true, ownWorkStillRuns: false, processFolder: '/proc', },);
 ```
 */
export async function clearScratchFolderOfARun({
  agentScratchFolder,
  runName,
  unitEnded,
  ownWorkStillRuns,
  processFolder,
}: {
  readonly agentScratchFolder: string;
  readonly runName: string;
  readonly unitEnded: boolean;
  readonly ownWorkStillRuns: boolean;
  readonly processFolder: string;
},): Promise<ScratchFolderOutcome> {
  /**
   The folder this run's agent wrote in, or why none may be named.
   */
  const named = scratchFolderOfARun({
    agentScratchFolder,
    runName,
  },);
  if (!('folder' in named)) {
    return {
      action: 'refused',
      line: `no scratch folder was looked for and nothing was deleted: ${named.refusal}`,
    };
  }
  /**
   The run's scratch folder.
   */
  const { folder, } = named;
  if (!unitEnded) {
    return keptScratchFolderOf({
      folder,
      reason: 'the launcher\'s wait for the run\'s unit gave up while the unit may still run',
    },);
  }
  if (ownWorkStillRuns) {
    return keptScratchFolderOf({
      folder,
      reason: 'the run\'s own work still runs, and it may write in there',
    },);
  }
  try {
    /**
     What the folder's own name is, read without following anything.
     */
    const shown = await lstat(folder,);
    if (shown.isSymbolicLink()) {
      return keptScratchFolderOf({
        folder,
        reason: 'it is a symbolic link, and a link is never followed',
      },);
    }
    if (!shown.isDirectory()) {
      return keptScratchFolderOf({
        folder,
        reason: 'it is not a folder, so what it names is not this run\'s to remove',
      },);
    }
    /**
     The folder a process of this machine still works in, if one does.
     */
    const inUse = folderSomeProcessWorksIn({
      folders: [folder,],
      workingFolders: (await processesAtTheirFolders({ processFolder, },))
        .map(function folderOf(process,) {
          return process.folder;
        },),
    },);
    if (inUse.folder !== undefined) {
      return keptScratchFolderOf({
        folder,
        reason: 'a process still works in it, so a removal would take the folder of work the run left running',
      },);
    }
    return await clearBesideTheKeptFolder({ folder, },);
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {
        action: 'nothing-there',
        line: `no scratch folder is there (${folder}), and nothing was deleted`,
      };
    }
    // Anything else is the operating system's refusal to read or remove an entry, which the launcher says rather than ending the
    // launch on: the run ended the way its unit ended, and the folder is left as it is for the next end or a coordinator.
    return keptScratchFolderOf({
      folder,
      reason: `it could not be read or cleared: ${caughtValueText(error,)}`,
    },);
  }
}

/**
 Removes every entry of a run's scratch folder beside `keep/`, then the folder itself when no `keep/` is there.

 The entries are removed all at once because they are disjoint paths: one entry's removal waits on nothing another's does.

 @param folder - the run's scratch folder, which is a folder and holds no link of its own under its name

 @returns what was removed, in the answer the launcher prints

 @example
 ```ts
 const outcome = await clearBesideTheKeptFolder({ folder: '/home/a/temp/agent/copy-route-scratch', },);
 ```
 */
async function clearBesideTheKeptFolder({
  folder,
}: {
  readonly folder: string;
},): Promise<ScratchFolderOutcome> {
  /**
   Every entry of the folder.
   */
  const entries = await readdir(folder,);
  /**
   The entries that go, which are every entry but the one named `keep`: a folder named `keepsake` is no `keep/`, and goes with the
   rest.
   */
  const going = entries.filter(function isNotTheKeptFolder(entry,) {
    return entry !== KEPT_FOLDER_NAME;
  },);
  await Promise.all(going.map(function removesOne(entry,) {
    return rm(
      join(
        folder,
        entry,
      ),
      {
        recursive: true,
        force: true,
      },
    );
  },),);
  if (entries.includes(KEPT_FOLDER_NAME,)) {
    return {
      action: 'cleared',
      removed: going.length,
      line: `the scratch folder ${folder} is cleared beside ${KEPT_FOLDER_NAME}/: ${removedClauseOf({ count: going.length, },)}`,
    };
  }
  await rmdir(folder,);
  return {
    action: 'cleared',
    removed: going.length,
    line: `the scratch folder ${folder} held no ${KEPT_FOLDER_NAME}/, so it was removed: ${removedClauseOf({ count: going.length, },)}`,
  };
}

//endregion Clearing it
