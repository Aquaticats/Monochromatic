/**
 The line a check prints to say which checkout and commit it ran on, and the line that says why it could not read one.

 This repository's `git` is a wrapper, and it refuses every command whose effective working directory is not the checkout's
 root (`require-root/not-at-root`, `src/native/command/rule_require_root.rs`), while `mise` gives a package's own task the
 package's folder as its working directory. So `git -C <that folder> rev-parse` is refused, and a run started there printed
 the path it checked in and no commit at all (issue 6617). The commit is read from the checkout's root instead, which the
 folder tree names without asking git, because asking git is the command that is refused.

 A folder that is in no checkout, and a git that cannot answer, each get a line saying the commit could not be read and why:
 a log that holds no commit and no reason reads as one that was never asked.

 This was the only `git -C` in the package's `bin/` refused in silence:
 every other one names a checkout's root, or runs `/usr/bin/git`, or throws with the refusal's words.
 */
import {
  join,
  resolve,
} from 'node:path';

import {
  checkoutMarkerName,
  gitDeadlineMilliseconds,
  gitProgram,
  type PathKind,
  pathKindOf,
  promptlessGitSettings,
} from './native-gate-paths.ts';
import {
  type GateCommandOutcome,
  runGateCommand,
} from './native-gate-run.ts';
import { lastLines } from './native-release-host-control-requirements.ts';

/**
 Characters of the commit the line prints, which is the short form git's `--short` answers with, long enough that two commits
 of one checkout are told apart in a log.
 */
const shortCommitDigits = 10;

/**
 The words that ask git which commit a checkout points at, after the `-C` pair that names the folder to ask in.
 */
const shortCommitWords: readonly string[] = [
  'rev-parse',
  `--short=${String(shortCommitDigits)}`,
  'HEAD',
];

/**
 What the search for the folder a git command may run in found.

 The absence is a case of its own rather than a nullish value, so the caller says in its own words what it does when no
 checkout is there.
 */
type CheckoutRoot =
  | {
    /**
     Marks a checkout root the search found.
     */
    readonly kind: 'found';
    /**
     The folder a git command may run in.
     */
    readonly root: string;
  }
    | {
    /**
     Marks a folder tree that is in no checkout.
     */
      readonly kind: 'none';
    };

/**
 The folder this repository's `git` lets a command run in: the nearest folder at or above one that holds a
 {@link checkoutMarkerName} at all, which is a folder in a checkout and the file a worktree writes.

 A worktree's marker is a file, so anything there and not missing makes the folder a checkout's root. The folder tree is read
 rather than asked of git, because the command that would ask is the one the wrapper refuses away from the root.

 @param from - the folder the search starts at

 @returns the checkout's root, or that no folder at or above the given one is one

 @example
 ```ts
 await checkoutRootOf({ from: '/repo/package/a' });
 ```
 */
async function checkoutRootOf({ from }: { readonly from: string; }): Promise<CheckoutRoot> {
  /**
   The folder the search starts at and every folder above it, up to the filesystem's root.
   */
  const folders: string[] = [];
  for (let folder = resolve(from,); ;) {
    folders.push(folder,);
    /**
     The folder above this one, which is this one at the root of the filesystem.
     */
    const above = resolve(
      folder,
      '..',
    );
    if (above === folder)
      break;
    folder = above;
  }
  /**
   What each folder holds under the marker's name.
   */
  const markers = await Promise.all(folders.map(async function markerIn(folder): Promise<{
    readonly folder: string;
    readonly kind: PathKind;
  }> {
    return {
      folder,
      kind: await pathKindOf({ path: join(
        folder,
        checkoutMarkerName
      ), }),
    };
  },));
  /**
   The nearest folder that holds the marker at all, which is the one the wrapper lets a command run in.
   */
  const found = markers.find(function holdsAMarker(marker) {
    return marker.kind !== 'missing';
  });
  return found === undefined ? { kind: 'none', } : {
    kind: 'found',
    root: found.folder,
  };
}

/**
 The words a line that says the commit could not be read quotes from what git printed.

 @param answered - what the command printed, on both streams

 @returns the last lines it printed, or text saying it printed none

 @example
 ```ts
 saidOf({ answered });
 ```
 */
function saidOf({ answered }: { readonly answered: GateCommandOutcome; }): string {
  /**
   The command's own words, which are the reason a person reads beside the status.
   */
  const said = lastLines(`${answered.stderr}\n${answered.stdout}`)
    .join(' | ');
  return said === '' ? '(nothing)' : said;
}

/**
 The line that says which checkout and commit the checks ran on, or why no commit could be read.

 @param worktree - the folder the checks ran in

 @returns the line to print before the checks' own lines

 @example
 ```ts
 await commitLinesOf({ worktree: '/repo/.claude/worktrees/a' });
 ```
 */
export async function commitLinesOf({ worktree }: { readonly worktree: string; }): Promise<readonly string[]> {
  /**
   The folder a git command may run in, which is the checkout the checks ran in.
   */
  const found = await checkoutRootOf({ from: worktree, });
  if (found.kind === 'none')
    return [`${worktree}: the commit could not be read: no folder at or above it holds a ${checkoutMarkerName}, so there is no checkout root to ask ${gitProgram} from`,];
  /**
   The checkout's root, which is the folder the commit is read in.
   */
  const { root, } = found;
  /**
   What `git rev-parse` answered at that root.
   */
  const answered = await runGateCommand({
    program: gitProgram,
    args: [
      '-C',
      root,
      ...shortCommitWords,
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: {
      ...process.env,
      ...promptlessGitSettings,
    },
  });
  /**
   How a line that says the commit could not be read names the command that did not answer.
   */
  const step = `${gitProgram} ${shortCommitWords.join(' ')} at the checkout root ${root}`;
  if (answered.timedOut)
    return [`${worktree}: the commit could not be read: ${step} was still running after ${String(gitDeadlineMilliseconds)} ms, the bound it is given, so it was stopped and told nothing`,];
  if (answered.problem !== undefined)
    return [`${worktree}: the commit could not be read: ${step} could not be run: ${answered.problem}`,];
  if (answered.status !== 0)
    return [`${worktree}: the commit could not be read: ${step} exited ${String(answered.status)} and said: ${saidOf({ answered, })}`,];
  /**
   The commit it named.
   */
  const commit = answered.stdout
    .trim();
  return [commit === '' ? `${worktree}: the commit could not be read: ${step} exited 0 and printed no commit` : `${worktree} at ${commit}`,];
}
