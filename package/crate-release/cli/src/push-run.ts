import { join } from 'node:path';

import {
  buildCommitRequest,
  type CommitFile,
  parseCommitResponse,
} from './commit-request.ts';
import {
  failureExitCode,
  ghProgram,
  gitProgram,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import type { PushOptions } from './push-options.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';
import { buildGraphqlInputArguments } from './service-arguments.ts';

/**
 Runs one program call to completion and reports how it ended.
 */
export type PushRunner = (call: ProgramCall) => Promise<ProgramResult>;

/**
 Name of the request file inside the scratch directory.
 */
const requestFileName = 'crate-release-commit-request.json';

/**
 Arguments of the `git` call that lists the head commit and its parents.
 */
const headParentsArguments: readonly string[] = [
  'rev-list',
  '--parents',
  '--max-count=1',
  'HEAD',
];

/**
 Arguments of the `git` call that prints the subject of the head commit.
 */
const headSubjectArguments: readonly string[] = [
  'log',
  '--max-count=1',
  '--format=%s',
  'HEAD',
];

/**
 Arguments of the `git` call that lists, NUL-separated, the files the head commit changes.
 */
const headPathsArguments: readonly string[] = [
  'diff-tree',
  '-z',
  '--no-commit-id',
  '--name-only',
  '-r',
  'HEAD',
];

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
   Answers of the three `git` calls.
   */
  const [parents, subject, paths] = await Promise.all([
    headParentsArguments,
    headSubjectArguments,
    headPathsArguments,
  ].map(async function ask(args) {
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
        `git ${args.join(' ')} failed with status ${String(result.status)} in ${options.root}: ${result.stderr.trim()}`,
      );
    return result.stdout;
  }));
  /**
   The head's own name, then its parents' names.
   */
  const [, ...parentNames] = (parents ?? '').trim()
    .split(' ');
  if (parentNames.length !== 1)
    throw new CrateReleaseError(
      `The head commit must have exactly one parent, the commit that was checked out, but it has ${String(parentNames.length)}.`,
    );
  /**
   The changed paths, without the empty text after the last NUL.
   */
  const changed = (paths ?? '').split('\0')
    .filter(function isPath(path) {
      return path !== '';
    });
  if (changed.length === 0)
    throw new CrateReleaseError('The head commit changes no file, so there is nothing to push.');
  return {
    parent: parentNames[0] ?? '',
    headline: (subject ?? '').replace(
      /\n$/u,
      ''
    ),
    paths: changed,
  };
}

/**
 Append the release commit of a checkout to the remote branch through the GraphQL API, so GitHub signs it.

 A branch that requires signed commits refuses the unsigned commit that `git push` would send from a runner. The commit
 is rebuilt on the commit that was checked out, from the head commit's subject and the whole contents of the files it
 changes, and GitHub refuses it when the branch has moved since the checkout, so a release never lands on top of work
 it did not see.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @param readBytes - reads a file by absolute path

 @param writeText - writes a file by absolute path

 @param print - prints one line

 @returns zero when the commit was created, the failure status when GitHub refused it

 @throws CrateReleaseError when the head commit cannot be read, or GitHub's answer cannot be understood

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
   The changed files with their contents.
   */
  const files: readonly CommitFile[] = await Promise.all(head.paths.map(async function readFileOf(path) {
    return {
      path,
      contents: await readBytes(join(
        options.root,
        path
      )),
    };
  }));
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
      expectedHeadOid: head.parent,
      headline: head.headline,
      files,
    }),
  });
  print(`pushing ${JSON.stringify(head.headline)} to ${options.repo}@${options.branch} on top of ${head.parent}`);
  print(`files: ${head.paths.join(', ')}`);
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
  if (!created.found) {
    print(`PUSH REFUSED: ${created.reason}`);
    print(
      `If the branch moved after the checkout, nothing was pushed: dispatch the release again. gh exited ${String(result.status)}${
        result.stderr.trim() === '' ? '' : `: ${result.stderr.trim()}`
      }`,
    );
    return failureExitCode;
  }
  print(`PUSHED ${created.value.oid} ${created.value.url}`);
  return 0;
}
