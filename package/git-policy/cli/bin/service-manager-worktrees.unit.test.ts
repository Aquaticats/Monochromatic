import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  containerGit,
  containerUser,
  worktreePaths,
} from './service-manager-machine.ts';
import {
  starter,
  worktreeRoot,
  worktreeScript,
} from './service-manager-worktrees.ts';

/**
 The script `makeWorktrees` hands the machine, which is what these controls read.

 @returns the script, one line per step
 */
function script(): string {
  return worktreeScript({
    root: worktreeRoot,
    first: worktreePaths.first,
    second: worktreePaths.second,
    git: containerGit,
    user: containerUser,
  });
}

/**
 Where a line sits in the script, which is how a control says one step comes before another.

 @param line - the whole line to look for

 @returns the line's position, or -1 when the script holds none
 */
function positionOf({ line, }: { readonly line: string; }): number {
  return script()
    .split('\n',)
    .indexOf(line,);
}

await describe({
  name: '',
  children: [
    describe({
      name: worktreeScript.name,
      children: [
        it({
          name: 'hands the folder Git writes to the user every command in the machine runs as',
          fn: async () => {
            expect(script(),).toContain(`chown ${containerUser}:${containerUser} ${worktreeRoot}`,);
          },
        }),
        it({
          name: 'hands the folder over before the first Git call',
          fn: async () => {
            /**
             Where the hand-over sits, which is -1 when the script has none.
             */
            const handOver = positionOf({ line: `chown ${containerUser}:${containerUser} ${worktreeRoot}`, },);
            /**
             Where the first Git call sits, which is -1 when the script has none.
             */
            const firstGit = script()
              .split('\n',)
              .findIndex(function runsGit(line) {
                return line.includes(containerGit,);
              },);
            expect(handOver,).toBeGreaterThanOrEqual(0,);
            expect(firstGit,).toBeGreaterThan(handOver,);
          },
        }),
        it({
          name: 'makes the folder before it hands it over, so a machine whose folder is gone still starts',
          fn: async () => {
            expect(positionOf({ line: `mkdir --parents ${worktreeRoot}`, },),)
              .toBeLessThan(positionOf({ line: `chown ${containerUser}:${containerUser} ${worktreeRoot}`, },),);
          },
        }),
        it({
          name: 'makes the first worktree, commits its first file, and adds the second worktree from it',
          fn: async () => {
            expect(script(),).toContain(`${containerGit} init --quiet --initial-branch=main .`,);
            expect(script(),).toContain(`${containerGit} commit --quiet --message base`,);
            expect(script(),).toContain(`${containerGit} worktree add --quiet --detach ${worktreePaths.second}`,);
          },
        }),
      ],
    },),
    describe({
      name: starter.name,
      children: [
        it({
          name: 'runs one caller in the background through its own release and prints the status it ended with',
          fn: async () => {
            /**
             The line of shell one caller becomes.
             */
            const line = starter({
              release: '/opt/release-a',
              worktree: worktreePaths.first,
              change: 'p1',
            },);
            expect(line.startsWith('( ',),).toBe(true,);
            expect(line.endsWith(' ) &',),).toBe(true,);
            expect(line,).toContain(`/opt/release-a/git commit --message 'p1' -- a.txt`,);
            expect(line,).toContain('echo "done p1 $?"',);
          },
        }),
      ],
    },),
  ],
},);
