/**
 Committing a merge that is already open in a checkout, retried while the installed wrapper's scratch commit ceiling stops it.

 The answer is read from the merge itself: the commit is done only when the merge marker is gone.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  headOf,
  mergeIsOpen,
  type Repository,
  unresolvedPaths,
} from './git-commands.ts';
import {
  commitStaged,
} from './git-changes.ts';
import { LandingConfigurationError, } from './errors.ts';
import type { Report, } from './report.ts';

/**
 What committing an open merge came to.
 */
export type CommitOutcome =
  /**
   No merge was open, so nothing was committed.
   */
  | { readonly commit: 'no-merge-open' }
  /**
   The merge is committed, and the checkout holds what the commit landed.
   */
    | {
      readonly commit: 'committed';
      readonly tries: number;
      readonly head: string
    }
  /**
   Every try was refused, so the merge is still open.
   */
    | {
      readonly commit: 'refused';
      readonly tries: number
    }
  /**
   Git said there was nothing to commit, so no number of further tries would help.
   */
    | {
      readonly commit: 'nothing-to-commit';
      readonly tries: number
    };

/**
 Longest commit name kept on a report line, which is a short name and room for one.
 */
const COMMIT_NAME_CHARACTERS = 40;

/**
 Longest text of a refused commit kept on a report line, which is what says why it was refused.
 */
const REFUSAL_CHARACTERS = 200;

/**
 What Git says when a commit would record nothing.
 */
const NOTHING_TO_COMMIT = [
  'nothing to commit',
  'no changes added to commit',
] as const;

/**
 Commits the merge open in a checkout, trying again while the wrapper refuses.

 @param repository - the checkout to ask

 @param message - the message its commit carries

 @param paths - the paths

 @param tries - how many times its commit is tried

 @param pauseMilliseconds - milliseconds between two tries

 @param commitTimeoutMilliseconds - milliseconds one commit may take

 @param report - where to report

 @returns what committing came to

 @throws LandingConfigurationError when the merge still holds unresolved paths

 @example
 ```ts
 const outcome = await commitTheOpenMerge({ repository, message: 'merge(a): b', paths: [], tries: 15, pauseMilliseconds: 20_000, report, },);
 ```
 */
export async function commitTheOpenMerge({
  repository,
  message,
  paths,
  tries,
  pauseMilliseconds,
  commitTimeoutMilliseconds,
  report,
}: {
  readonly repository: Repository;
  readonly message: string;
  readonly paths: readonly string[];
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
  readonly report: Report;
},): Promise<CommitOutcome> {
  if (!(await mergeIsOpen(repository,))) {
    report.line('no merge is open',);
    return { commit: 'no-merge-open', };
  }
  /**
   The paths the merge left unresolved.
   */
  const unresolved = await unresolvedPaths(repository,);
  if (unresolved.length > 0) {
    throw new LandingConfigurationError(
      `unresolved files remain:\n${unresolved.join('\n',)}`,
    );
  }
  for (let attempt = 1; attempt <= tries; attempt += 1) {
    /**
     What the commit said.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- each try must see whether the last one landed
    const committed = await commitStaged({
      repository,
      message,
      paths,
      timeoutMilliseconds: commitTimeoutMilliseconds,
    },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the answer is read from the merge itself, after this try
    if (!(await mergeIsOpen(repository,))) {
      /**
       The commit that landed.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- the name of the commit that landed, read once it has
      const head = await headOf(repository,);
      report.line(`committed on try ${String(attempt,)}: ${head.slice(
        0,
        COMMIT_NAME_CHARACTERS,
      )}`,);
      return {
        commit: 'committed',
        tries: attempt,
        head,
      };
    }
    /**
     Everything the refused commit said.
     */
    const output = `${committed.stdout}${committed.stderr}`;
    if (NOTHING_TO_COMMIT.some(function saysNothing(text: string,) {
      return output.includes(text,);
    },)) {
      report.line(`nothing to commit on try ${String(attempt,)}: the tree already matches what the checkout holds, and the merge is still open`,);
      return {
        commit: 'nothing-to-commit',
        tries: attempt,
      };
    }
    report.line(`try ${String(attempt,)}: status ${String(committed.status,)}: ${output.trim()
      .slice(
        0,
        REFUSAL_CHARACTERS,
      )
      .replaceAll(
        '\n',
        ' ',
      )}`,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the wait between tries is the design, see doc/lint/sequential-lands.md
    await wait(pauseMilliseconds,);
  }
  report.line(`the merge is still open after ${String(tries,)} tries`,);
  return {
    commit: 'refused',
    tries,
  };
}
