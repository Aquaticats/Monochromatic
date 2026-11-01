import { join } from 'node:path';

import {
  buildCommitRequest,
  type CommitFile,
  isStaleHeadRefusal,
  parseCommitResponse,
} from './commit-request.ts';
import {
  failureExitCode,
  ghProgram,
  gitProgram,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import { decideAfterStaleHead } from './push-decision.ts';
import type { PushOptions } from './push-options.ts';
import { collectWatchSet } from './push-watch.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';
import { buildGraphqlInputArguments } from './service-arguments.ts';
import type { WatchSet } from './watch-paths.ts';

/**
 Runs one program call to completion and reports how it ended.
 */
export type PushRunner = (call: ProgramCall) => Promise<ProgramResult>;

/**
 Name of the request file inside the scratch directory.
 */
export const requestFileName = 'crate-release-commit-request.json';

/**
 Arguments of the `git` call that lists the head commit and its parents.
 */
export const headParentsArguments: readonly string[] = [
  'rev-list',
  '--parents',
  '--max-count=1',
  'HEAD',
];

/**
 Arguments of the `git` call that prints the subject of the head commit.
 */
export const headSubjectArguments: readonly string[] = [
  'log',
  '--max-count=1',
  '--format=%s',
  'HEAD',
];

/**
 Arguments of the `git` call that lists, NUL-separated, the files the head commit changes.
 */
export const headPathsArguments: readonly string[] = [
  'diff-tree',
  '-z',
  '--no-commit-id',
  '--name-only',
  '-r',
  'HEAD',
];

/**
 Run one `git` call in the checkout and return what it printed.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @param args - arguments after `git`

 @returns standard output of the call

 @throws CrateReleaseError naming the call, its status and its error text when `git` fails

 @example
 ```ts
 await askGit({ options, run, args: ['rev-parse', 'HEAD'] });
 ```
 */
async function askGit({
  options,
  run,
  args,
}: {
  readonly options: PushOptions;
  readonly run: PushRunner;
  readonly args: readonly string[];
}): Promise<string> {
  /**
   How `git` ended.
   */
  const result = await run({
    program: gitProgram,
    args,
    cwd: options.root
  });
  if (result.status !== 0)
    throw new CrateReleaseError(
      `git ${args.join(' ')} failed with status ${String(result.status)} in ${options.root}: ${result.stderr
        .trim()}`,
    );
  return result.stdout;
}

/**
 Read what the head commit of a checkout is: its parent, its subject and the files it changes.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @returns the full object name of the parent, the subject, and the changed paths

 @throws CrateReleaseError when `git` fails, or the head is not a single-parent commit that changes a file

 @example
 ```ts
 await readHeadCommit({ options, run });
 ```
 */
async function readHeadCommit({
  options,
  run,
}: {
  readonly options: PushOptions;
  readonly run: PushRunner;
}): Promise<{
  readonly parent: string;
  readonly headline: string;
  readonly paths: readonly string[];
}> {
  /**
   The head's own name, then its parents' names.
   */
  const [, ...parentNames] = (await askGit({
    options,
    run,
    args: headParentsArguments
  })).trim()
    .split(' ');
  if (parentNames.length !== 1)
    throw new CrateReleaseError(
      `The head commit must have exactly one parent, the commit that was checked out, but it has ${String(parentNames.length)}.`,
    );
  /**
   The subject followed by the line break that ends the line.
   */
  const subject = await askGit({
    options,
    run,
    args: headSubjectArguments
  });
  /**
   The changed paths, without the empty text after the last NUL.
   */
  const changed = (await askGit({
    options,
    run,
    args: headPathsArguments
  })).split('\0')
    .filter(function isPath(path) {
      return path !== '';
    });
  if (changed.length === 0)
    throw new CrateReleaseError('The head commit changes no file, so there is nothing to push.');
  return {
    parent: parentNames[0] ?? '',
    headline: subject.endsWith('\n') ? subject.slice(
      0,
      -1
    ) : subject,
    paths: changed,
  };
}

/**
 Everything one send of the commit needs besides the head it expects.
 */
type SendInput = {
  readonly options: PushOptions;
  readonly run: PushRunner;
  readonly writeText: (input: {
    readonly path: string;
    readonly text: string
  }) => Promise<void>;
  readonly print: (line: string) => void;
  /**
   The subject of the release commit.
   */
  readonly headline: string;
  /**
   The files of the release commit with their whole contents.
   */
  readonly files: readonly CommitFile[];
  /**
   Full object name of the commit the checkout started from, which the release verified.
   */
  readonly base: string;
  /**
   The paths the release depends on.
   */
  readonly watch: WatchSet;
};

/**
 Send the release commit on a head, and again on the new head while the branch keeps moving harmlessly.

 A send is refused when the branch is no longer at the head it expected. The refusal is then settled by
 `decideAfterStaleHead`: the commit is sent again on the new head only when nothing the release depends on changed since
 the base, and a branch that stays on the move for every allowed send ends the push with a refusal. Any other refusal
 ends the push at once, because repeating it would repeat it. The sends are a bounded chain (the attempt limit), each
 waiting for the one before, so the recursion is as deep as the limit and no deeper.

 @param input - options, program runner, writer, printer, the commit and the watch set

 @param expectedHeadOid - full object name of the head this send expects

 @param attempt - number of this send, from one

 @returns zero when the commit was created, the failure status when it was refused

 @example
 ```ts
 await sendCommit({ input, expectedHeadOid: base, attempt: 1 });
 ```
 */
async function sendCommit({
  input,
  expectedHeadOid,
  attempt,
}: {
  readonly input: SendInput;
  readonly expectedHeadOid: string;
  readonly attempt: number;
}): Promise<number> {
  /**
   Parts of the input this send uses.
   */
  const {
    options,
    run,
    writeText,
    print,
    headline,
    files,
    base,
    watch
  } = input;
  /**
   Absolute path of the request file.
   */
  const requestFile = join(
    options.scratchDirectory,
    requestFileName
  );
  await writeText({
    path: requestFile,
    text: buildCommitRequest({
      repo: options.repo,
      branch: options.branch,
      expectedHeadOid,
      headline,
      files,
    }),
  });
  print(
    `send ${String(attempt)} of at most ${String(options.maxAttempts)}: ${JSON.stringify(headline)} to ${options.repo}@${options.branch} on top of ${expectedHeadOid}`,
  );
  /**
   How `gh` ended.
   */
  const result = await run({
    program: ghProgram,
    args: buildGraphqlInputArguments(requestFile),
    cwd: options.root
  });
  /**
   The created commit, or why there is none.
   */
  const created = parseCommitResponse(result.stdout);
  if (created.found) {
    print(`PUSHED ${created.value
      .oid} ${created.value
        .url}`);
    return 0;
  }
  if (!isStaleHeadRefusal(result.stdout)) {
    print(`PUSH REFUSED: ${created.reason}`);
    print(
      `Nothing was pushed. gh exited ${String(result.status)}${
        result.stderr
          .trim()
          === '' ? '' : `: ${result.stderr
            .trim()}`
      }`,
    );
    return failureExitCode;
  }
  print(`${options.branch} moved after the checkout: ${created.reason}`);
  if (attempt >= options.maxAttempts) {
    print(
      `PUSH REFUSED: ${options.branch} moved before each of the ${String(attempt)} allowed sends. Nothing was pushed: dispatch the release again.`,
    );
    return failureExitCode;
  }
  /**
   The head to send on again, or why the push is refused.
   */
  const next = await decideAfterStaleHead({
    options,
    run,
    writeText,
    base,
    watch,
    print
  });
  if (!next.found) {
    print(`PUSH REFUSED: ${next.reason}. Nothing was pushed: dispatch the release again.`);
    return failureExitCode;
  }
  return await sendCommit({
    input,
    expectedHeadOid: next.value,
    attempt: attempt + 1
  });
}

/**
 Append the release commit of a checkout to the remote branch through the GraphQL API, so GitHub signs it.

 A branch that requires signed commits refuses the unsigned commit that `git push` would send from a runner. The commit
 is rebuilt on the commit that was checked out, from the head commit's subject and the whole contents of the files it
 changes. GitHub refuses it when the branch has moved since the checkout, and a branch of a busy repository does move
 during a release, so before anything is sent the command records every path the release depends on: the commit's
 files, the crate and lock directories, and the path dependency closure that `cargo metadata` reports. A refusal for a
 moved branch is then followed by a comparison of the new head with the checkout, and the commit is sent again on the new
 head only when none of those paths changed. A release never lands on top of a change to what it verified.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @param readBytes - reads a file by absolute path

 @param writeText - writes a file by absolute path

 @param print - prints one line

 @returns zero when the commit was created, the failure status when GitHub refused it or the branch changed under it

 @throws CrateReleaseError when the head commit cannot be read, or the dependency closure cannot be recorded, or
 GitHub's answer cannot be understood

 @example
 ```ts
 await pushRelease({ options, run, readBytes, writeText, print: console.log });
 ```
 */
export async function pushRelease({
  options,
  run,
  readBytes,
  writeText,
  print,
}: {
  readonly options: PushOptions;
  readonly run: PushRunner;
  readonly readBytes: (path: string) => Promise<Uint8Array>;
  readonly writeText: (input: {
    readonly path: string;
    readonly text: string
  }) => Promise<void>;
  readonly print: (line: string) => void;
}): Promise<number> {
  /**
   What the head commit is and changes.
   */
  const head = await readHeadCommit({
    options,
    run
  });
  /**
   The paths the release depends on, recorded before anything is sent.
   */
  const watch = await collectWatchSet({
    options,
    run,
    commitPaths: head.paths
  });
  print(`base ${head.parent}; files: ${head.paths
    .join(', ')}`);
  print(
    `watching the files ${watch.files
      .join(', ')} and the directories ${watch.directories
        .join(', ')}`,
  );
  /**
   The changed files with their contents.
   */
  const files: readonly CommitFile[] = await Promise.all(head.paths
    .map(async function readFileOf(path) {
      return {
        path,
        contents: await readBytes(join(
          options.root,
          path
        )),
      };
    }));
  return await sendCommit({
    input: {
      options,
      run,
      writeText,
      print,
      headline: head.headline,
      files,
      base: head.parent,
      watch
    },
    expectedHeadOid: head.parent,
    attempt: 1,
  });
}
