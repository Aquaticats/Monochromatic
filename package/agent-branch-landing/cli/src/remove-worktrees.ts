/**
 Removing the worktrees whose work is entirely on `main` and which nothing would lose, after repeating every check that made them
 removable.

 @module
 */

import type {
  Repository,
} from './git-commands.ts';
import {
  removeWorktree,
} from './git-changes.ts';
import {
  MILLISECONDS_PER_SECOND,
  REMOVAL_TIMEOUT_SECONDS,
} from './constants.ts';
import type { Report, } from './report.ts';
import type { MachineState, } from './machine-state.ts';
import {
  reportOnWorktrees,
  type WorktreeReport,
} from './worktrees.ts';

/**
 Longest text of a removal Git refused kept on a report line, which is what says why.
 */
const REFUSAL_CHARACTERS = 160;

/**
 What became of one worktree a run considered.
 */
export type RemovalOutcome = {
  /**
   The worktree's folder.
   */
  readonly path: string;

  /**
   Its folder's own name.
   */
  readonly name: string;

  /**
   Whether it is gone.
   */
  readonly removed: boolean;

  /**
   The line a report keeps for it.
   */
  readonly line: string;
};

/**
 Removes every worktree nothing would be lost by removing, one at a time, and reports on each one the judgement kept.

 @param repository - the checkout to ask

 @param machine - the machine's own state

 @param worktreesFolder - the folder worktrees live in

 @param report - where to report

 @returns what became of each worktree

 @example
 ```ts
 const outcomes = await removeRemovableWorktrees({ repository, machine, worktreesFolder, report, },);
 ```
 */
export async function removeRemovableWorktrees({
  repository,
  machine,
  worktreesFolder,
  report,
}: {
  readonly repository: Repository;
  readonly machine: MachineState;
  readonly worktreesFolder: string;
  readonly report: Report;
},): Promise<readonly RemovalOutcome[]> {
  /**
   Every worktree, judged again in this run, so nothing is removed on an answer an earlier run gave.
   */
  const reports = await reportOnWorktrees({
    repository,
    machine,
    worktreesFolder,
  },);
  /**
   What became of each.
   */
  const outcomes: RemovalOutcome[] = [];
  for (const each of reports) {
    if (each.reasonToKeep !== undefined) {
      outcomes.push(kept({
        worktree: each,
        report,
      },),);
      continue;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- one removal at a time, so a refusal names the worktree it stopped at
    outcomes.push(await removeOne({
      repository,
      report,
      worktree: each,
    },),);
  }
  report.line(`${String(outcomes.filter(function removed(outcome: RemovalOutcome,) {
    return outcome.removed;
  },)
    .length,)} removed of ${String(reports.length,)} worktrees`,);
  return outcomes;
}

/**
 Removes one worktree and says what Git made of it.

 @param repository - the checkout to ask

 @param report - where to report

 @param worktree - the worktree's folder name

 @returns what became of it

 @example
 ```ts
 const outcome = await removeOne({ repository, report, worktree },);
 ```
 */
async function removeOne({
  repository,
  report,
  worktree,
}: {
  readonly repository: Repository;
  readonly report: Report;
  readonly worktree: WorktreeReport;
},): Promise<RemovalOutcome> {
  /**
   What Git made of removing it.
   */
  const removed = await removeWorktree({
    repository,
    path: worktree.path,
    timeoutMilliseconds: REMOVAL_TIMEOUT_SECONDS * MILLISECONDS_PER_SECOND,
  },);
  /**
   The line this worktree gets.
   */
  const line = removed.status === 0
    ? `REMOVED ${worktree.name}`
    : `KEPT ${worktree.name}: Git refused to remove it with status ${String(removed.status,)}: ${removed.stderr
      .trim()
      .slice(
        0,
        REFUSAL_CHARACTERS,
      )}`;
  report.line(line,);
  return {
    path: worktree.path,
    name: worktree.name,
    removed: removed.status === 0,
    line,
  };
}

/**
 Says why one worktree stayed.

 @param worktree - the worktree's folder name

 @param report - where to report

 @returns what became of it, which is nothing

 @example
 ```ts
 const outcome = kept({ worktree, report },);
 ```
 */
function kept({
  worktree,
  report,
}: {
  readonly worktree: WorktreeReport;
  readonly report: Report;
},): RemovalOutcome {
  /**
   The line this worktree gets.
   */
  const line = `KEPT ${worktree.name}: ${worktree.reasonToKeep ?? 'nothing says why'}`;
  report.line(line,);
  return {
    path: worktree.path,
    name: worktree.name,
    removed: false,
    line,
  };
}
