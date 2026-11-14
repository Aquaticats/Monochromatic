/**
 The scratch repository the reload proof commits in, and the line of shell that starts one of its callers.
 */
import {
  containerGit,
  inside,
  type Machine,
  worktreePaths,
} from './service-manager-machine.ts';

/**
 What:
  Makes the scratch repository with a second worktree of its own.

 Why:
  Two worktrees are two commit service instances, which is what the incident's manager held dozens of.

 @param machine - the machine to make the repository on

 @example
 ```ts
 await makeWorktrees({ machine });
 ```
 */
export async function makeWorktrees({ machine, }: { readonly machine: Machine; }): Promise<void> {
  await inside({
    machine,
    script: [
      'mkdir --parents /work',
      `rm -rf ${worktreePaths.first} ${worktreePaths.second}`,
      `mkdir --parents ${worktreePaths.first}`,
      `cd ${worktreePaths.first}`,
      `${containerGit} init --quiet --initial-branch=main .`,
      `${containerGit} config user.email a@example.invalid`,
      `${containerGit} config user.name 'a person'`,
      String.raw`printf "base\n" > a.txt`,
      `${containerGit} add a.txt`,
      `${containerGit} commit --quiet --message base`,
      `cd ${worktreePaths.first} && ${containerGit} worktree add --quiet --detach ${worktreePaths.second}`,
    ].join('\n',),
  });
}

/**
 One background caller of the "several callers at once" step: a commit in one worktree through one release.

 @param release - the release's directory

 @param worktree - the worktree the commit is made in

 @param change - the text the commit adds

 @returns one line of shell that runs the caller in the background and prints its status

 @example
 ```ts
 starter({ release: '/opt/release-a', worktree: '/work/one', change: 'p1' });
 ```
 */
export function starter({
  release,
  worktree,
  change,
}: {
  readonly release: string;
  readonly worktree: string;
  readonly change: string;
}): string {
  /**
   The commands the caller runs, backgrounded as one group so `wait` collects it.
   */
  const commands = [
    `printf 'base\\n${change}\\n' > ${worktree}/a.txt`,
    `cd ${worktree}`,
    `${release}/git add a.txt`,
    `${release}/git commit --message '${change}' -- a.txt >/dev/null 2>&1`,
    `echo "done ${change} $?"`,
  ].join('; ',);
  return `( ${commands} ) &`;
}
