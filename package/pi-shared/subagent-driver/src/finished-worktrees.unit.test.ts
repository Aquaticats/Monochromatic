/**
 Which run worktrees are finished, and whether one may be removed: the worktree listing, the run name a folder carries, the
 folders of a run that must hold no process, a folder some process still works in, and the decision each reading leads to.

 Nothing here removes anything: the decision is read, and a worktree is only removed by the command that is asked to.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type Worktree,
  briefQueuedAfter,
  decideWorktree,
  folderSomeProcessWorksIn,
  isInside,
  ownFoldersOf,
  pausedNamesIn,
  runNameOfWorktree,
  runWorktreesIn,
  worktreesIn,
} from '../dist/final/node/index.mjs';

/**
 One worktree, as a test writes it.

 @param path - its path

 @param branch - the branch its HEAD is on

 @returns the worktree

 @example
 ```ts
 const worktree = aWorktree({ path: '/repo/.claude/worktrees/pi-a', branch: 'feat/a' });
 ```
 */
function aWorktree({
  path,
  branch = '',
}: {
  readonly path: string;
  readonly branch?: string;
},): Worktree {
  return {
    path,
    branch,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: worktreesIn.name,
      children: [
        it({
          name: 'reads every worktree a porcelain listing names, with the branch its HEAD is on, and leaves a detached one without a branch',
          fn: async () => {
            expect(worktreesIn({
              shown: [
                'worktree /repo',
                'branch refs/heads/main',
                '',
                'worktree /repo/.claude/worktrees/pi-a',
                'branch refs/heads/feat/a',
                '',
                'worktree /repo/.claude/worktrees/pi-b',
                'detached',
                '',
              ].join('\n',),
            },),).toEqual([
              {
                path: '/repo',
                branch: 'main',
              },
              {
                path: '/repo/.claude/worktrees/pi-a',
                branch: 'feat/a',
              },
              {
                path: '/repo/.claude/worktrees/pi-b',
                branch: '',
              },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: runWorktreesIn.name,
      children: [
        it({
          name: 'keeps the worktrees a launcher made for runs, and leaves the checkout and its other worktrees out',
          fn: async () => {
            expect(runWorktreesIn({
              worktrees: [
                aWorktree({ path: '/repo', },),
                aWorktree({ path: '/repo/.claude/worktrees/pi-a', },),
                aWorktree({ path: '/repo/.claude/worktrees/integrate-x', },),
              ],
              worktreeFolder: '/repo/.claude/worktrees',
            },),).toEqual([aWorktree({ path: '/repo/.claude/worktrees/pi-a', },),],);
          },
        },),
      ],
    },),
    describe({
      name: runNameOfWorktree.name,
      children: [
        it({
          name: 'reads the run name a worktree folder carries',
          fn: async () => {
            expect(runNameOfWorktree({
              worktree: aWorktree({ path: '/repo/.claude/worktrees/pi-kimi-review-a-round-1', },),
              worktreeFolder: '/repo/.claude/worktrees',
            },),).toBe('kimi-review-a-round-1',);
          },
        },),
      ],
    },),
    describe({
      name: isInside.name,
      children: [
        it({
          name: 'answers for the folder itself and something inside it, and never for a folder whose name merely starts with it',
          fn: async () => {
            expect(isInside({
              entry: '/repo/.claude/worktrees/pi-a',
              folder: '/repo/.claude/worktrees/pi-a',
            },),).toBe(true,);
            expect(isInside({
              entry: '/repo/.claude/worktrees/pi-a/package/x',
              folder: '/repo/.claude/worktrees/pi-a',
            },),).toBe(true,);
            expect(isInside({
              entry: '/repo/.claude/worktrees/pi-ab',
              folder: '/repo/.claude/worktrees/pi-a',
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: ownFoldersOf.name,
      children: [
        it({
          name: 'names the worktree, every attempt folder of that run, and the review folder of a review run, and another run attempts',
          fn: async () => {
            expect(ownFoldersOf({
              name: 'a',
              worktree: '/repo/.claude/worktrees/pi-a',
              runsFolder: '/runs',
              entries: ['a-1', 'a-2', 'a-prompt.md', 'another-1', 'a-extra-9',],
              reviewFolder: '/reviews',
            },),).toEqual([
              '/repo/.claude/worktrees/pi-a',
              '/runs/a-1',
              '/runs/a-2',
            ],);
            expect(ownFoldersOf({
              name: 'kimi-review-a-round-1',
              worktree: '/repo/.claude/worktrees/pi-kimi-review-a-round-1',
              runsFolder: '/runs',
              entries: ['kimi-review-a-round-1-1',],
              reviewFolder: '/reviews',
            },),).toEqual([
              '/repo/.claude/worktrees/pi-kimi-review-a-round-1',
              '/runs/kimi-review-a-round-1-1',
              '/reviews/a-round-1',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: folderSomeProcessWorksIn.name,
      children: [
        it({
          name: 'names the first of a run folders a process works in, and answers none when no process works in any',
          fn: async () => {
            expect(folderSomeProcessWorksIn({
              folders: ['/repo/.claude/worktrees/pi-a', '/runs/a-1',],
              workingFolders: ['/elsewhere', '/runs/a-1/reproductions',],
            },),).toEqual({ folder: '/runs/a-1', },);
            expect(folderSomeProcessWorksIn({
              folders: ['/repo/.claude/worktrees/pi-a',],
              workingFolders: ['/elsewhere',],
            },),).toEqual({},);
          },
        },),
      ],
    },),
    describe({
      name: pausedNamesIn.name,
      children: [
        it({
          name: 'reads the name each line of a pause list holds after its first word, and nothing for a blank line',
          fn: async () => {
            expect([...pausedNamesIn({ shown: 'disk-full a\n\ndisk-full b\n', },),],).toEqual(['a', 'b',],);
            expect([...pausedNamesIn({ shown: '', },),],).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: briefQueuedAfter.name,
      children: [
        it({
          name: 'says a brief written after the latest run ended is waiting, and says nothing is when the latest run has no summary',
          fn: async () => {
            expect(await briefQueuedAfter({
              runsFolder: '/not-a-folder',
              name: 'a',
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: decideWorktree.name,
      children: [
        it({
          name: 'removes the worktree of a run that finished its turn, with nothing of it working there and nothing uncommitted',
          fn: async () => {
            expect(decideWorktree({
              endReason: 'idle',
              isReview: false,
              reviewWroteItsFile: false,
              paused: false,
              briefQueued: false,
              unitState: 'inactive',
              inUseFolder: '',
              status: 0,
              statusText: '',
            },),).toEqual({ action: 'remove', },);
            expect(decideWorktree({
              endReason: 'stop-file',
              isReview: true,
              reviewWroteItsFile: true,
              paused: false,
              briefQueued: false,
              unitState: 'failed',
              inUseFolder: '',
              status: 0,
              statusText: '',
            },),).toEqual({ action: 'remove', },);
          },
        },),
        it({
          name: 'keeps a worktree some process still works in, which is the check that keeps a build a run left running from being removed under it',
          fn: async () => {
            expect(decideWorktree({
              endReason: 'idle',
              isReview: false,
              reviewWroteItsFile: false,
              paused: false,
              briefQueued: false,
              unitState: 'inactive',
              inUseFolder: '/repo/.claude/worktrees/pi-a/package/x',
              status: 0,
              statusText: '',
            },),).toEqual({
              action: 'skip',
              reason: 'a process still works in /repo/.claude/worktrees/pi-a/package/x',
            },);
          },
        },),
        it({
          name: 'leaves alone a run whose unit still runs, one that has not finished, one with a brief waiting, one that holds uncommitted work, and one git could not read',
          fn: async () => {
            /**
             The decision for each reading, which all leave the worktree alone.
             */
            const reasons = [
              decideWorktree({
                endReason: 'idle',
                isReview: false,
                reviewWroteItsFile: false,
                paused: false,
                briefQueued: false,
                unitState: 'active',
                inUseFolder: '',
                status: 0,
                statusText: '',
              },),
              decideWorktree({
                endReason: 'model-error',
                isReview: false,
                reviewWroteItsFile: false,
                paused: false,
                briefQueued: false,
                unitState: 'inactive',
                inUseFolder: '',
                status: 0,
                statusText: '',
              },),
              decideWorktree({
                endReason: 'idle',
                isReview: false,
                reviewWroteItsFile: false,
                paused: false,
                briefQueued: true,
                unitState: 'inactive',
                inUseFolder: '',
                status: 0,
                statusText: '',
              },),
              decideWorktree({
                endReason: 'idle',
                isReview: false,
                reviewWroteItsFile: false,
                paused: false,
                briefQueued: false,
                unitState: 'inactive',
                inUseFolder: '',
                status: 0,
                statusText: ' M package/x\n',
              },),
              decideWorktree({
                endReason: 'idle',
                isReview: false,
                reviewWroteItsFile: false,
                paused: false,
                briefQueued: false,
                unitState: 'inactive',
                inUseFolder: '',
                status: 1,
                statusText: '',
              },),
            ];
            expect(reasons.map(function actionOf(decision,) {
              return decision.action;
            },),).toEqual([
              'skip',
              'skip',
              'skip',
              'skip',
              'skip',
            ],);
            expect(reasons.every(function hasAReason(decision,) {
              return (decision.action === 'skip') && (decision.reason !== '');
            },),).toBe(true,);
          },
        },),
        it({
          name: 'leaves a paused name alone while its latest run did not finish its turn, and removes it once that run finished',
          fn: async () => {
            expect(decideWorktree({
              endReason: 'wall-clock',
              isReview: false,
              reviewWroteItsFile: false,
              paused: true,
              briefQueued: false,
              unitState: 'inactive',
              inUseFolder: '',
              status: 0,
              statusText: '',
            },).action,).toBe('skip',);
            expect(decideWorktree({
              endReason: 'idle',
              isReview: false,
              reviewWroteItsFile: false,
              paused: true,
              briefQueued: false,
              unitState: 'inactive',
              inUseFolder: '',
              status: 0,
              statusText: '',
            },).action,).toBe('remove',);
          },
        },),
      ],
    },),
  ],
},);
