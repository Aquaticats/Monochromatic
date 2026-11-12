/**
 The worktrees of finished Pi runs, and whether one may be removed: a run that ended its turn, whose branch keeps every commit,
 whose folder holds nothing uncommitted, and whose folder nothing of the run still works in.

 The owner: "You're the coordinator and you're supposed to remove finished worktrees automatically." A worktree added on btrfs is a
 snapshot, so finished worktrees left in place filled the disk. Nothing here removes anything by force, and a folder some process
 still works in is left alone: before 2026-11-11 this step ran `cargo clean` in, and removed, the worktree of a run that had ended
 idle while a build or reproduction that run started was still working there, so the wake had nothing to return to.

 @module
 */

import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { entriesOrNothingIn, } from './newest-finished-run-of-a-name.ts';

/**
 The prefix a run's worktree folder carries, which every worktree of a run is named with.
 */
const WORKTREE_FOLDER_START = 'pi-';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The end of a run's prompt file, which a brief written after its run ended leaves behind.
 */
const PROMPT_FILE_END = '-prompt.md';

/**
 One worktree, as `git worktree list --porcelain` describes it.
 */
export type Worktree = {
  /**
   The worktree's path.
   */
  readonly path: string;

  /**
   The branch its HEAD is on, or empty text for a detached HEAD.
   */
  readonly branch: string;
};

/**
 Reads the worktrees `git worktree list --porcelain` printed.

 @param shown - what git printed

 @returns every worktree it named, in the order it printed them

 @example
 ```ts
 const worktrees = worktreesIn({ shown: 'worktree /repo\\nbranch refs/heads/main\\n', },);
 ```
 */
export function worktreesIn({ shown, }: { readonly shown: string },): readonly Worktree[] {
  return shown
    .split('\n\n',)
    .flatMap(function ofBlock(block,) {
      /**
       The worktree's path, when the block names one.
       */
      const path = block
        .split('\n',)
        .find(function namesThePath(line,) {
          return line.startsWith('worktree ',);
        },)
        ?.slice('worktree '.length,);
      if (path === undefined) {
        return [];
      }
      /**
       The branch its HEAD is on, when it is on one.
       */
      /**
       The block's lines.
       */
      const blockLines = block.split('\n',);
      /**
       The line naming the branch its HEAD is on, when it is on one.
       */
      const branchLine = blockLines
        .find(function namesTheBranch(line,) {
          return line.startsWith('branch ',);
        },);
      /**
       The branch its HEAD is on, or empty text for a detached HEAD.
       */
      /**
       The branch its HEAD is on, before the ref prefix is taken off it.
       */
      const branchWithoutPrefix = (branchLine ?? '')
        .slice('branch '.length,);
      /**
       The branch its HEAD is on, or empty text for a detached HEAD.
       */
      const branch = branchWithoutPrefix.replace(
        'refs/heads/',
        '',
      );
      return [{
        path,
        branch: branch ?? '',
      },];
    },);
}

/**
 Reads the worktrees a checkout holds for its runs.

 @param worktrees - every worktree of the checkout

 @param worktreeFolder - the folder the launcher makes run worktrees under

 @returns the run worktrees, in the order git printed them

 @example
 ```ts
 const runs = runWorktreesIn({ worktrees, worktreeFolder: '/repo/.claude/worktrees', },);
 ```
 */
export function runWorktreesIn({
  worktrees,
  worktreeFolder,
}: {
  readonly worktrees: readonly Worktree[];
  readonly worktreeFolder: string;
},): readonly Worktree[] {
  /**
   The start of a run's worktree path.
   */
  const start = `${worktreeFolder}/${WORKTREE_FOLDER_START}`;
  return worktrees.filter(function isTheRunsWorktree(worktree,) {
    /**
     The worktree's path.
     */
    const { path, } = worktree;
    return path.startsWith(start,);
  },);
}

/**
 Reads the name of the run a worktree belongs to.

 @param worktree - the worktree

 @param worktreeFolder - the folder the launcher makes run worktrees under

 @returns the run's name, which is the folder's name without its prefix

 @example
 ```ts
 const name = runNameOfWorktree({ worktree: { path: '/repo/.claude/worktrees/pi-a', branch: 'feat/a' }, worktreeFolder: '/repo/.claude/worktrees', },);
 ```
 */
export function runNameOfWorktree({
  worktree,
  worktreeFolder,
}: {
  readonly worktree: Worktree;
  readonly worktreeFolder: string;
},): string {
  /**
   The worktree's path.
   */
  const { path, } = worktree;
  return path.slice(`${worktreeFolder}/${WORKTREE_FOLDER_START}`.length,);
}

/**
 Reads the folder an entry belongs to, whether it is the folder itself or something inside it.

 @param entry - a path a process works in

 @param folder - the folder to ask about

 @returns whether the process works in that folder

 @example
 ```ts
 const inside = isInside({ entry: '/repo/.claude/worktrees/pi-a/package/x', folder: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export function isInside({
  entry,
  folder,
}: {
  readonly entry: string;
  readonly folder: string;
},): boolean {
  return (entry === folder) || entry.startsWith(`${folder}/`,);
}

/**
 Reads the folders of one run that must hold no process before its worktree may be removed: the worktree itself, every attempt's
 run folder, and the review's own folder when the run is a review's.

 @param name - the run's name

 @param worktree - the run's worktree

 @param runsFolder - folder the driver writes one run folder per run into

 @param entries - every entry of the runs folder

 @param reviewFolder - the folder that holds one folder per review

 @returns the folders a process working in keeps the worktree

 @example
 ```ts
 const folders = ownFoldersOf({ name: 'a', worktree: '/repo/.claude/worktrees/pi-a', runsFolder: '/runs', entries: ['a-1'], reviewFolder: '/reviews', },);
 ```
 */
export function ownFoldersOf({
  name,
  worktree,
  runsFolder,
  entries,
  reviewFolder,
}: {
  readonly name: string;
  readonly worktree: string;
  readonly runsFolder: string;
  readonly entries: readonly string[];
  readonly reviewFolder: string;
},): readonly string[] {
  /**
   The start of a Kimi review's run name.
   */
  const reviewStart = 'kimi-review-';
  return [
    worktree,
    ...entries
      .filter(function isAnAttemptOfThatName(entry,) {
        /**
         What follows the run's name in this entry.
         */
        const after = entry.startsWith(`${name}-`,)
          ? entry.slice(name.length + 1,)
          : '';
        return isDigitsOnlyText({ text: after, },);
      },)
      .map(function folderOf(entry,) {
        return join(
          runsFolder,
          entry,
        );
      },),
    ...((name.startsWith(reviewStart,))
      ? [join(
        reviewFolder,
        name.slice(reviewStart.length,),
      ),]
      : []),
  ];
}

/**
 Reads whether text is digits only, which a run folder's number is written as.

 @param text - the text to read

 @returns whether every character is a digit and there is at least one

 @example
 ```ts
 const isANumber = isDigitsOnlyText({ text: '42', },);
 ```
 */
function isDigitsOnlyText({ text, }: { readonly text: string },): boolean {
  if (text === '') {
    return false;
  }
  for (let index = 0; index < text.length; index += 1) {
    /**
     The character at this position.
     */
    const character = text.charAt(index,);
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Reads whether a process works in any of one run's folders.

 @param folders - the run's own folders

 @param workingFolders - the folder every process of the machine works in

 @returns the folder a process works in, or nothing when none does

 @example
 ```ts
 const working = folderSomeProcessWorksIn({ folders: ['/repo/.claude/worktrees/pi-a'], workingFolders: [], },);
 ```
 */
export function folderSomeProcessWorksIn({
  folders,
  workingFolders,
}: {
  readonly folders: readonly string[];
  readonly workingFolders: readonly string[];
},): { readonly folder?: string } {
  for (const folder of folders) {
    /**
     Whether any process works in this folder.
     */
    const inUse = workingFolders.some(function isInsideThatFolder(entry,) {
      return isInside({
        entry,
        folder,
      },);
    },);
    if (inUse) {
      return { folder, };
    }
  }
  return {};
}

/**
 Reads the names a pause list holds, one name per line after its first word.

 @param shown - the pause list as it was written

 @returns the names it holds

 @example
 ```ts
 const paused = pausedNamesIn({ shown: 'disk-full a\\ndisk-full b\\n', },);
 ```
 */
export function pausedNamesIn({
  shown,
}: {
  readonly shown: string;
},): ReadonlySet<string> {
  return new Set(shown
    .split('\n',)
    .map(function secondWordOf(line,) {
      return line.trim()
        .split(' ',)
        .find(function isNotTheFirstWord(
          word,
          index,
        ) {
          return (index === 1) && (word !== '');
        },)
        ?? '';
    },)
    .filter(function isAName(name,) {
      return name !== '';
    },));
}

/**
 What is to be done with one run's worktree.
 */
export type WorktreeDecision =
  | {
    /**
     The worktree is removed, which the caller then does.
     */
    readonly action: 'remove';
  }
    | {
    /**
     Nothing is done, because the run is not finished: its unit runs, it is paused, its review wrote no file, or a brief is
     waiting for a slot.
     */
      readonly action: 'skip';

    /**
     The line that says why nothing is done.
     */
      readonly reason: string;
    };

/**
 Says what is to be done with one run's worktree.

 A run's worktree is removed only when its unit has ended, the run finished its work, no brief is waiting for a slot, nothing of
 the run works in it, and it holds nothing uncommitted. Everything else is a skip with the reason, so a coordinator reading the
 list knows which runs are still going and which worktrees are kept for their own reason.

 @param endReason - why the run ended, or nothing when it wrote no summary

 @param isReview - whether the worktree belongs to a review

 @param reviewWroteItsFile - whether that review wrote its review file

 @param paused - whether the run is on the pause list

 @param briefQueued - whether a brief was written after its latest run ended

 @param unitState - the state systemd answered for its unit

 @param inUseFolder - the folder a process still works in, or empty text when none does

 @param status - how git status ended

 @param statusText - what git status printed

 @returns what to do with the worktree

 @example
 ```ts
 const decision = decideWorktree({ endReason: 'idle', isReview: false, reviewWroteItsFile: false, paused: false, briefQueued: false, unitState: 'inactive', inUseFolder: '', status: 0, statusText: '', },);
 ```
 */
export function decideWorktree({
  endReason,
  isReview,
  reviewWroteItsFile,
  paused,
  briefQueued,
  unitState,
  inUseFolder,
  status,
  statusText,
}: {
  readonly endReason?: string;
  readonly isReview: boolean;
  readonly reviewWroteItsFile: boolean;
  readonly paused: boolean;
  readonly briefQueued: boolean;
  readonly unitState: string;
  readonly inUseFolder: string;
  readonly status: number;
  readonly statusText: string;
},): WorktreeDecision {
  /**
   The states a unit is in while it has not ended.
   */
  const running = [
    'active',
    'activating',
    'deactivating',
    'reloading',
    'refreshing',
  ];
  if (running.includes(unitState,)) {
    return {
      action: 'skip',
      reason: `its unit still runs (${unitState})`,
    };
  }
  // A paused name counts only while its latest run did not end idle: every paused agent has been resumed since.
  if (paused && (endReason !== 'idle')) {
    return {
      action: 'skip',
      reason: 'it is listed as paused and its latest run did not finish its turn',
    };
  }
  /**
   Whether the run finished its work: a review once its review file exists, and every other run once its summary says it ended
   idle, which is its turn done with its work committed.
   */
  const finished = isReview ? reviewWroteItsFile : (endReason === 'idle');
  if (!finished) {
    return {
      action: 'skip',
      reason: `it has not finished (run ended ${String(endReason,)})`,
    };
  }
  if (briefQueued) {
    return {
      action: 'skip',
      reason: 'a brief was written after its latest run ended, so it is about to be used again',
    };
  }
  if (inUseFolder !== '') {
    return {
      action: 'skip',
      reason: `a process still works in ${inUseFolder}`,
    };
  }
  if (status !== 0) {
    return {
      action: 'skip',
      reason: `git status could not read it (status ${String(status,)})`,
    };
  }
  if (statusText.trim() !== '') {
    return {
      action: 'skip',
      reason: `it holds uncommitted work (${String(statusText.trim()
        .split('\n',)
        .length,)} entries)`,
    };
  }
  return { action: 'remove', };
}

/**
 Reads a file, or answers empty text when it is not there.

 @param file - the file to read

 @returns its text, or empty text when it is missing

 @example
 ```ts
 const shown = await readFileOrNothing('/runs/paused.txt');
 ```
 */
export async function readFileOrNothing(file: string,): Promise<string> {
  try {
    return await readFile(
      file,
      'utf8',
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return '';
    }
    throw error;
  }
}

/**
 Says whether a file is there and when it was last written.

 @param file - the file to ask about

 @returns when it was last written, or nothing when it is not there

 @example
 ```ts
 const written = await writtenAt({ file: '/runs/a-prompt.md' });
 ```
 */
export async function writtenAt({
  file,
}: {
  readonly file: string;
},): Promise<{ readonly at?: number }> {
  try {
    return { at: (await stat(file,)).mtimeMs, };
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {};
    }
    throw error;
  }
}

/**
 Reads whether a brief for a run's name was written after its latest run ended, which is a resume or a re-merge waiting for a
 slot: such a worktree is about to be used again.

 @param runsFolder - folder the driver writes one run folder per run into

 @param name - the run's name

 @param latestSummary - the summary of its newest finished run, when it has one

 @returns whether a brief is waiting

 @example
 ```ts
 const queued = await briefQueuedAfter({ runsFolder: '/runs', name: 'a', },);
 ```
 */
export async function briefQueuedAfter({
  runsFolder,
  name,
  latestSummary,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly latestSummary?: string;
},): Promise<boolean> {
  /**
   When the latest run's summary was written, which a brief written later than this is waiting for a slot.
   */
  const ended = (latestSummary === undefined) ? undefined : (await writtenAt({ file: latestSummary, },)).at;
  if (ended === undefined) {
    return true;
  }
  /**
   Every entry of the runs folder that starts with the run's name.
   */
  const entries = await entriesOrNothingIn(runsFolder,);
  /**
   When each brief of that name was written.
   */
  const briefs = await Promise.all(entries
    .filter(function isABriefOfThatName(entry,) {
      return entry.startsWith(name,) && entry.endsWith(PROMPT_FILE_END,);
    },)
    .map(function ofEntry(entry,) {
      return writtenAt({ file: join(
        runsFolder,
        entry,
      ), },);
    },));
  return briefs.some(function isLater(brief,) {
    return (brief.at ?? 0) > ended;
  },);
}
