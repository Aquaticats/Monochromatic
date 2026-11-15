/**
 What is to be done with a run's worktree once its unit has ended, and why: only a run that finished its work on a branch, holding
 nothing uncommitted, has its worktree removed.

 The decision is kept apart from the making so that a reader can see what keeps a worktree without reading the git commands that make
 one, and so that every ending other than a finished turn keeps its worktree by name and reason.

 @module
 */

import type { GitRunner, } from './worktree-git.ts';

/**
 The end reasons of a run that finished its work, which are the only runs whose worktree is removed.
 */
const FINISHED_END_REASONS: ReadonlySet<string> = new Set<string>([
  'idle',
  'stop-file',
]);

/**
 The first lines of `git status` a note carries, so a long listing stays readable.
 */
const STATUS_LINES_SHOWN = 5;

/**
 What is to be done with a run's worktree once its unit has ended.
 */
export type WorktreeOutcome =
  | {
    /**
     The worktree is removed, which this command then does.
     */
    readonly action: 'remove';
  }
    | {
      readonly action: 'keep';

    /**
     The line that says why it stays.
     */
      readonly line: string;
    }
    | {
    /**
     Nothing is done and nothing is said: the run works in the checkout itself.
     */
      readonly action: 'keep-quietly';
    };

/**
 Says what is to be done with a run's worktree, given how the run ended and whether the worktree holds anything.

 Only a run that finished its work on a branch, with nothing uncommitted, has its worktree removed: one that ended on a cap or an
 error is likely to be resumed, and a detached HEAD's commits may be reachable from this worktree alone.

 @param readOnlyInMain - whether the run works in the checkout itself

 @param endReason - why the run ended, or nothing when it wrote no summary

 @param onABranch - whether the worktree's HEAD is a branch rather than detached

 @param status - how `git status` ended, which matters only when it printed nothing

 @param statusText - what `git status --porcelain` printed

 @returns what to do, and why it was decided so

 @example
 ```ts
 const outcome = worktreeOutcomeOf({ readOnlyInMain: false, endReason: 'idle', onABranch: true, status: 0, statusText: '', },);
 ```
 */
export function worktreeOutcomeOf({
  readOnlyInMain,
  endReason,
  onABranch,
  status,
  statusText,
}: {
  readonly readOnlyInMain: boolean;
  readonly endReason?: string;
  readonly onABranch: boolean;
  readonly status: number;
  readonly statusText: string;
},): WorktreeOutcome {
  if (readOnlyInMain) {
    return { action: 'keep-quietly', };
  }
  /**
   Whether the run finished its work, which is the first thing a removal needs.
   */
  const finishedItsWork = (endReason !== undefined) && FINISHED_END_REASONS.has(endReason,);
  if ((!finishedItsWork) || (!onABranch)) {
    return {
      action: 'keep',
      line: `worktree kept (run ended ${String(endReason,)}, ${onABranch ? 'on a branch' : 'detached HEAD'})`,
    };
  }
  if ((status !== 0) || (statusText.trim() !== '')) {
    return {
      action: 'keep',
      line: `worktree kept, it holds uncommitted work (git status ${String(status,)}): ${statusText.trim()
        .split('\n',)
        .slice(
          0,
          STATUS_LINES_SHOWN,
        )
        .join(', ',)}`,
    };
  }
  return { action: 'remove', };
}

/**
 Removes a run's worktree, never by force.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree was made in

 @param worktree - the worktree's path

 @returns what git answered, in words for a reader

 @example
 ```ts
 const note = await removeWorktree({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function removeWorktree({
  git,
  checkoutFolder,
  worktree,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
},): Promise<string> {
  /**
   What git answered about removing the worktree.
   */
  const removed = await git({
    folder: checkoutFolder,
    words: [
      'worktree',
      'remove',
      worktree,
    ],
  },);
  return `worktree remove status ${String(removed.status,)}${(removed.status === 0) ? '' : `: ${removed.stderr
    .trim()}`}`;
}
