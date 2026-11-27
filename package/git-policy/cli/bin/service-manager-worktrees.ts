/**
 The scratch repository the reload proof commits in, and the line of shell that starts one of its callers.
 */
import {
  containerGit,
  containerUser,
  inside,
  type Machine,
  worktreePaths,
} from './service-manager-machine.ts';

/**
 The folder inside the machine that the scratch repository and its worktrees are made in, which is also the folder every
 Git call of these proofs writes.
 */
export const worktreeRoot: string = '/work';

/**
 What:
  The lines of shell that make the scratch repository and its second worktree of its own.

 Why:
  This host's `git` AppArmor profile attaches to `/usr/bin/git` inside a container too, and it grants `/work` to the folder's
  owner alone: a process of another user writing there asks for `dac_override`, which the profile refuses, so Git cannot
  create even a leading directory and the step ends with `fatal: could not create leading directories of
  '/work/two/.git': Permission denied`. The image leaves `/work` owned by its own user (1000), and every command of these
  proofs runs as `containerUser`, so the first line gives that folder to that user; after it every Git call writes files
  whose owner is the user running it, and no call needs a capability
  (`doc/troubleshooting/git-cannot-mkdir-inside-the-container-proof.md`).

  The hand-over comes before every Git call on purpose, and it is written as one line of a script rather than beside the
  calls: a machine started from an older image, or an image whose `/work` comes back owned by another user, is fixed by the
  same line, and a Git call that runs while the folder still belongs to someone else is the failure this script began with.

 @param root - the folder the repository is made in, whose owner the hand-over sets

 @param first - the repository's first worktree

 @param second - the second worktree, which is a second commit service instance

 @param git - the real Git inside the machine, which every call here runs as the user above

 @param user - the user every command in the machine runs as, which owns what Git writes below the folder

 @returns the script, one line per step, in the order the machine runs them

 @example
 ```ts
 worktreeScript({ root: '/work', first: '/work/one', second: '/work/two', git: '/usr/bin/git', user: '0' });
 ```
 */
export function worktreeScript({
  root,
  first,
  second,
  git,
  user,
}: {
  readonly root: string;
  readonly first: string;
  readonly second: string;
  readonly git: string;
  readonly user: string;
}): string {
  return [
    `mkdir --parents ${root}`,
    `chown ${user}:${user} ${root}`,
    `rm -rf ${first} ${second}`,
    `mkdir --parents ${first}`,
    `cd ${first}`,
    `${git} init --quiet --initial-branch=main .`,
    `${git} config user.email a@example.invalid`,
    `${git} config user.name 'a person'`,
    String.raw`printf "base\n" > a.txt`,
    `${git} add a.txt`,
    `${git} commit --quiet --message base`,
    `cd ${first} && ${git} worktree add --quiet --detach ${second}`,
  ].join('\n',);
}

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
    script: worktreeScript({
      root: worktreeRoot,
      first: worktreePaths.first,
      second: worktreePaths.second,
      git: containerGit,
      user: containerUser,
    }),
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
