/**
 A repository whose answers a test writes, so a landing's own logic is exercised without a real Git, which this machine's wrapper
 refuses to run outside its commit service.

 The answers are keyed by the words Git is asked, and every request is kept, so a test can assert the command line a landing
 builds as well as what it makes of the answer.

 @module
 */

import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import type {
  ProgramOutput,
  RunProgram,
  RunProgramRequest,
} from '../dist/final/node/index.mjs';

/**
 What one answer holds.
 */
export type Answer = {
  /**
   The status the command exited with.
   */
  readonly status?: number;

  /**
   What it printed.
   */
  readonly stdout?: string;

  /**
   What it complained about.
   */
  readonly stderr?: string;
};

/**
 The answers one test gives, keyed by the words of the command joined by spaces.

 @internal
 */
export type Handlers = Readonly<Record<string, (request: RunProgramRequest,) => Answer | Promise<Answer>>>;

/**
 A repository a test asks with its own answers, and the requests it made.
 */
export type FakeRepository = {
  /**
   How a program is run, which is the seam a landing asks every question through.
   */
  readonly run: RunProgram;

  /**
   Every request the landing made, in order.
   */
  readonly requests: RunProgramRequest[];

  /**
   The folder the repository is judged to sit in.
   */
  readonly folder: string;

  /**
   The path where an open merge would be recorded, which a test creates to open one and removes to commit it.
   */
  readonly mergeHeadPath: string;

  /**
   The words of each Git request, one line each, which is what a test reads the built command line from.
   */
  gitLines(): readonly string[];
};

/**
 A folder no other test uses, under the scratch root rather than the system temporary directory.

 @param name - a word naming the folder's purpose

 @returns its path

 @example
 ```ts
 const folder = await disposableFolder('land',);
 ```
 */
export async function disposableFolder(name: string,): Promise<string> {
  await mkdir(scratchRoot(), { recursive: true, },);
  return mkdtemp(join(
    scratchRoot(),
    `landing-${name}-`,
  ),);
}

/**
 Builds a repository whose answers a test writes.

 @param answers - what each command answers, keyed by its words joined by spaces

 @param folder - the folder the repository sits in

 @returns the seam, the requests, and the merge marker's path

 @example
 ```ts
 const repository = await fakeRepository({ answers: {'merge --no-ff --no-commit --quiet fix/a': () => ({ status: 0, },)}, });
 ```
 */
export async function fakeRepository({
  answers,
  folder,
}: {
  readonly answers: Handlers;
  readonly folder: string;
},): Promise<FakeRepository> {
  /**
   The folder the repository's Git records sit in.
   */
  const gitFolder = join(
    folder,
    '.git',
  );
  await mkdir(gitFolder, { recursive: true, },);
  /**
   Where an open merge is recorded, which is a real file this test's answers leave alone.
   */
  const mergeHeadPath = join(
    gitFolder,
    'MERGE_HEAD',
  );
  /**
   Every request made so far.
   */
  const requests: RunProgramRequest[] = [];
  return {
    folder,
    mergeHeadPath,
    requests,
    run: async function run(request: RunProgramRequest,) {
      requests.push(request,);
      /**
       The words this command is asked with.
       */
      const line = request.words.join(' ',);
      /**
       What this command answers, which for the merge marker is where this repository keeps it.
       */
      const handler = answers[line]
        ?? (line === 'rev-parse --path-format=absolute --git-path MERGE_HEAD'
          ? function marker() {
            return { stdout: `${mergeHeadPath}\n`, };
          }
          : undefined);
      /**
       What it left.
       */
      const answer: Answer = handler === undefined ? {} : await handler(request,);
      return {
        status: answer.status ?? 0,
        stdout: answer.stdout ?? '',
        stderr: answer.stderr ?? '',
      } satisfies ProgramOutput;
    },
    gitLines: function gitLines() {
      return requests
        .filter(function isGit(request: RunProgramRequest,) {
          return request.program === 'git';
        },)
        .map(function lineOf(request: RunProgramRequest,) {
          return request.words.join(' ',);
        },);
    },
  };
}

/**
 Opens a merge in a fake repository, which is what makes a landing refuse to start.

 @param repository - the fake repository

 @example
 ```ts
 await openAMerge(repository,);
 ```
 */
export async function openAMerge(repository: FakeRepository,): Promise<void> {
  await writeFile(repository.mergeHeadPath, 'HEAD\n',);
}
