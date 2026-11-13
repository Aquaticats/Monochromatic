/**
 Proving, at the moment of the deletion, that the run which ended still owns the scratch folder that is about to be emptied.

 A scratch folder is named by the run's name, and a run's name is free as soon as its unit is collected: a new attempt of that name
 starts, reuses the folder, and writes its own work there while the attempt before it is still ending. The readings an end step takes
 on its way (the process table, the unit's state) are old by the time it acts, so the proof is taken where the deletion begins, in
 the command that deletes.

 Two things are asked, and a folder is emptied only when both say it is the ended run's own: no unit of that name is going, and no
 process of the account works in the folder. A reading that could not be taken is no proof, so the folder stays and the line says
 which reading failed rather than emptying on a guess.

 @module
 */

import type { RunCommand, } from './run-command.ts';
import {
  type ProcessAtAFolder,
  processesAtTheirFolders,
} from './wake-a-run-whose-work-outlives-its-turn.ts';
import {
  unitStateIsRunning,
  unitStateOf,
} from './wait-for-a-unit-to-end.ts';
import { isInside, } from './finished-worktrees.ts';

/**
 The unit a run of one name is started as, which is the name the manager knows it by.
 */
const UNIT_PREFIX = 'pi-';

/**
 The unit's own suffix.
 */
const UNIT_SUFFIX = '.service';

/**
 Says why one folder is not provably the ended run's own, from the readings a caller took at the moment of the deletion.

 @param runName - the run that ended

 @param unitState - the state the manager answered for that run's unit, or empty text when it could not be asked

 @param processes - what the process table shows every process working in

 @param folder - the folder about to be emptied

 @param excludingPid - the process doing the emptying, which works wherever it was started

 @returns why the folder is not provably that run's own, or nothing when it is

 @example
 ```ts
 const problem = ownershipProblemOf({ runName: 'copy-route', unitState: 'inactive', processes: [], folder: '/home/a/temp/agent/copy-route-scratch', excludingPid: 1, },);
 ```
 */
export function ownershipProblemOf({
  runName,
  unitState,
  processes,
  folder,
  excludingPid,
}: {
  readonly runName: string;
  readonly unitState: string;
  readonly processes: readonly ProcessAtAFolder[];
  readonly folder: string;
  readonly excludingPid: number;
},): { readonly problem?: string } {
  /**
   The unit this run is started as, which the manager was asked about.
   */
  const unitName = `${UNIT_PREFIX}${runName}${UNIT_SUFFIX}`;
  // A state nobody could read is not a state that says the unit has gone: the older attempt's launcher may be waiting on a manager
  // that stopped answering while a newer attempt of the same name works in the folder.
  if (unitState === '') {
    return { problem: `${unitName} is in no state anybody could read, so there is no proof that a run of that name has ended and ${folder} may be a later run's`, };
  }
  if (unitStateIsRunning({ state: unitState, },)) {
    return { problem: `a run of that name is going now (${unitName} is ${unitState}), so ${folder} may be a later run's and was left as it is`, };
  }
  /**
   The process working in the folder, if one does.
   */
  const working = processes.find(function worksThere(process,) {
    return (process.pid !== excludingPid) && isInside({
      entry: process.folder,
      folder,
    },);
  },);
  if (working !== undefined) {
    return { problem: `a process still works in ${working.folder} (${String(working.pid,)} ${working.command}), so ${folder} may be its own and was left as it is`, };
  }
  return {};
}

/**
 Reads, now, whether the run that ended still owns its scratch folder.

 @param run - how a command is run, which a test replaces

 @param runName - the run that ended

 @param folder - the folder about to be emptied

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param excludingPid - the process doing the emptying

 @returns why the folder is not provably that run's own, or nothing when it is

 @example
 ```ts
 const problem = await ownershipProblemNow({ run: runCommandWithChildProcess, runName: 'copy-route', folder, processFolder: '/proc', excludingPid: process.pid, },);
 ```
 */
export async function ownershipProblemNow({
  run,
  runName,
  folder,
  processFolder,
  excludingPid,
}: {
  readonly run: RunCommand;
  readonly runName: string;
  readonly folder: string;
  readonly processFolder: string;
  readonly excludingPid: number;
},): Promise<{ readonly problem?: string }> {
  /**
   The state the manager answers for the run's unit, which is asked first: a run that is going is the whole reason this proof exists.
   */
  const unitState = await unitStateOf({
    run,
    unitName: `${UNIT_PREFIX}${runName}${UNIT_SUFFIX}`,
  },);
  return ownershipProblemOf({
    runName,
    unitState,
    processes: await processesAtTheirFolders({ processFolder, },),
    folder,
    excludingPid,
  },);
}
