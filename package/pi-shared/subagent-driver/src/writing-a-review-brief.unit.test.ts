/**
 Writes what a review of one change reads: the diff of its ranges, the record of what it was given, and its brief, as one
 generation.

 Two things this file checks are about one moment of answers. Every revision the ranges and the commit name is turned into a commit
 once, before anything is asked about it, and one git answer holds both the diff and the paths it touches, so an integration that
 moves between two questions cannot make a change with a file read as nothing to review. And the three files are written whole or
 not at all, so a second writing for another commit never leaves an old brief beside a new diff and record.

 @module
 */

import { spawnSync, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type CommandOutput,
  type GitQuestion,
  ReviewGitError,
  writeReviewBrief,
} from '../dist/final/node/index.mjs';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The change every brief here is written for.

 @returns the change's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'rewrite-the-gate';
}

/**
 The files a change touches, which the git questions here answer with.

 @returns the files, from the top of the checkout

 @example
 ```ts
 const files = theFilesTouched();
 ```
 */
function theFilesTouched(): readonly string[] {
  return ['package/a/src/gate.rs',];
}

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-brief-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Answers the git questions one brief is written from: each revision as a commit, the range's merge base, the answer that holds
 both the diff and the paths it touches, and the files the commit holds.

 @param files - the files the range touched and the commit holds

 @returns the answers, by the question asked

 @example
 ```ts
 const git = gitAnswering({ files: theFilesTouched(), },);
 ```
 */
function gitAnswering({ files, }: { readonly files: readonly string[] },): GitQuestion {
  return async function answer({ words, }: {
    readonly words: readonly string[];
    readonly folder: string;
  },): Promise<CommandOutput> {
    if (words[0] === 'rev-parse') {
      return {
        status: 0,
        stdout: `${(words[2] ?? '').replace('^{commit}', '')}\n`,
        stderr: '',
      };
    }
    if (words[0] === 'merge-base') {
      return {
        status: 0,
        stdout: 'base-commit\n',
        stderr: '',
      };
    }
    if (words[0] === 'ls-tree') {
      return {
        status: 0,
        stdout: `${files.join('\n',)}\n`,
        stderr: '',
      };
    }
    return {
      status: 0,
      stdout: `${files.map(function entryOf(path,) {
        return `:100644 100644 0000000 1234567 M\0${path}\0`;
      },)
        .join('',)
         }\0${
         files.map(function patchOf(path,) {
          return `diff --git a/${path} b/${path}\n`;
        },)
          .join('',)}`,
      stderr: '',
    };
  };
}


/**
 A disposable repository whose `main` a test can advance under the writer, and the commits it asks about.

 It is built with `git fast-import`, so no command of the machine's Git wrapper decides what the commits are, and every commit the
 test names is one it wrote.

 @param folder - the folder the repository is made in

 @returns where the repository is, and the commits a brief is written for

 @example
 ```ts
 const repository = await aDisposableRepository({ folder: '/scratch', },);
 ```
 */
async function aDisposableRepository({ folder, }: { readonly folder: string },): Promise<{
  readonly checkoutFolder: string;
  readonly firstChangeCommit: string;
  readonly firstChangeFile: string;
  readonly secondChangeCommit: string;
  readonly secondChangeFile: string;
  readonly mergedMain: string;
}> {
  /**
   Where the repository is.
   */
  const checkoutFolder = join(folder, 'repository',);
  await mkdir(checkoutFolder, { recursive: true, },);
  /**
   Runs one git command in the repository and answers what it printed.
   */
  function git({ words, input, }: { readonly words: readonly string[]; readonly input?: string },): string {
    /**
     What git answered.
     */
    const answered = spawnSync('git', words, {
      cwd: checkoutFolder,
      encoding: 'utf8',
      ...((input === undefined) ? {} : { input, }),
    },);
    if ((answered.status !== 0) || (answered.error !== undefined)) {
      throw new Error(`git ${words.join(' ',)} answered ${String(answered.status,)}: ${answered.stderr}`,);
    }
    return answered.stdout.trim();
  }
  git({ words: ['init', '--initial-branch=main', '--quiet',], },);
  /**
   A blob of a file's text, as fast-import takes it.
   */
  function data(text: string,): string {
    /**
     The text with the line feed a file ends with.
     */
    const raw = text.endsWith('\n',) ? text : `${text}\n`;
    return `data ${String(Buffer.byteLength(raw,),)}\n${raw}`;
  }
  /**
   One commit of the stream.
   */
  function commit({
    ref,
    mark,
    message,
    from,
    merge,
    files,
  }: {
    readonly ref: string;
    readonly mark: string;
    readonly message: string;
    readonly from?: string;
    readonly merge?: string;
    readonly files: readonly { readonly blob: string; readonly path: string }[];
  },): string {
    return `commit ${ref}\nmark ${mark}\ncommitter Disposable Review <disposable@example.invalid> 1700000000 +0000\n${data(message,)}${(from === undefined) ? '' : `from ${from}\n`}${(merge === undefined) ? '' : `merge ${merge}\n`}${files.map(function added(file,) {
        return `M 100644 ${file.blob} ${file.path}\n`;
      },)
        .join('',)}`;
  }
  /**
   The base file every commit holds, and the one main adds beside it.
   */
  const baseFile = { blob: ':1', path: 'package/a/src/base.ts', };
  /**
   The file the first change adds.
   */
  const firstChangeFile = 'package/a/src/first.ts';
  /**
   The file the second change adds.
   */
  const secondChangeFile = 'package/a/src/second.ts';
  /**
   The file main adds before its merge.
   */
  const mainFile = { blob: ':7', path: 'main-only.txt', };
  git({
    words: ['fast-import', '--quiet',],
    input: [
      `blob\nmark :1\n${data('export const base = 0;\n',)}`,
      commit({
        ref: 'refs/heads/main',
        mark: ':2',
        message: 'base',
        files: [baseFile,],
      },),
      `blob\nmark :3\n${data('export const first = true;\n',)}`,
      commit({
        ref: 'refs/heads/fix/first',
        mark: ':4',
        message: 'the first change',
        from: ':2',
        files: [baseFile, { blob: ':3', path: firstChangeFile, },],
      },),
      `blob\nmark :5\n${data('export const second = true;\n',)}`,
      commit({
        ref: 'refs/heads/fix/second',
        mark: ':6',
        message: 'the second change',
        from: ':2',
        files: [baseFile, { blob: ':5', path: secondChangeFile, },],
      },),
      `blob\nmark :7\n${data('independent main change\n',)}`,
      commit({
        ref: 'refs/heads/main-before-merge',
        mark: ':8',
        message: 'main before its merge',
        from: ':2',
        files: [baseFile, mainFile,],
      },),
      commit({
        ref: 'refs/heads/merged-main',
        mark: ':9',
        message: 'integrate the first change',
        from: ':8',
        merge: ':4',
        files: [baseFile, mainFile, { blob: ':3', path: firstChangeFile, },],
      },),
      'done\n',
    ].join('',),
  },);
  /**
   The commit main starts at.
   */
  const mainBeforeMerge = git({ words: ['rev-parse', 'refs/heads/main-before-merge',], },);
  git({ words: ['update-ref', 'refs/heads/main', mainBeforeMerge,], },);
  return {
    checkoutFolder,
    firstChangeCommit: git({ words: ['rev-parse', 'refs/heads/fix/first',], },),
    firstChangeFile,
    secondChangeCommit: git({ words: ['rev-parse', 'refs/heads/fix/second',], },),
    secondChangeFile,
    mergedMain: git({ words: ['rev-parse', 'refs/heads/merged-main',], },),
  };
}

/**
 Answers the writer's questions from a real repository, moving `main` to a commit of its own after every answer.

 Moving `main` between every pair of questions is what a merge into it does on this machine many times an hour; a writer that
 answers from one moment comes back with the same material whatever happens between its questions.

 @param repository - the repository, and the commit main is moved to

 @param moveMainTo - the commit to move `main` to after every answer, or nothing to leave `main` alone

 @returns the answers

 @example
 ```ts
 const git = gitOfTheRepository({ repository, moveMainTo: repository.mergedMain, },);
 ```
 */
function gitOfTheRepository({
  repository,
  moveMainTo,
}: {
  readonly repository: { readonly checkoutFolder: string };
  readonly moveMainTo?: string;
},): GitQuestion {
  return async function answer({ words, folder, }: {
    readonly words: readonly string[];
    readonly folder: string;
  },): Promise<CommandOutput> {
    /**
     What git answered.
     */
    const answered = spawnSync('git', words, {
      cwd: folder,
      encoding: 'utf8',
    },);
    if (moveMainTo !== undefined) {
      spawnSync('git', ['update-ref', 'refs/heads/main', moveMainTo,], {
        cwd: repository.checkoutFolder,
        encoding: 'utf8',
      },);
    }
    return {
      status: answered.status ?? 1,
      stdout: answered.stdout ?? '',
      stderr: answered.stderr ?? '',
    };
  };
}

/**
 Reads the three files one review folder holds as one generation.

 @param reviewsFolder - the folder that holds one folder per review

 @returns what each file says, with the record's commit and ranges

 @example
 ```ts
 const generation = await generationIn('/reviews');
 ```
 */
async function generationIn(reviewsFolder: string,): Promise<{
  readonly brief: string;
  readonly diff: string;
  readonly commit: string;
  readonly ranges: readonly string[];
}> {
  /**
   The review's own folder.
   */
  const folder = join(reviewsFolder, aReview(),);
  /**
   What the record says the review was given.
   */
  const record: unknown = JSON.parse(await readFile(join(folder, 'what-the-review-read.json',), 'utf8',),);
  /**
   The record's fields, which a generation always keeps together.
   */
  const fields = ((typeof record) === 'object') && (record !== null) ? record as {
    readonly commit: string;
    readonly ranges: readonly string[];
  } : { commit: '', ranges: [], };
  return {
    brief: await readFile(join(folder, 'brief.md',), 'utf8',),
    diff: await readFile(join(folder, 'change.diff',), 'utf8',),
    commit: fields.commit,
    ranges: fields.ranges,
  };
}

/**
 The request every brief in this file is written from.

 @param reviewsFolder - the folder that holds one folder per review

 @param git - how the git questions are answered

 @returns the request

 @example
 ```ts
 const request = aBriefRequest({ reviewsFolder: '/reviews', git: gitAnswering({ files: theFilesTouched(), },), },);
 ```
 */
function aBriefRequest({
  reviewsFolder,
  git,
}: {
  readonly reviewsFolder: string;
  readonly git: GitQuestion;
}): Parameters<typeof writeReviewBrief>[0] {
  return {
    git,
    checkoutFolder: '/repo',
    review: aReview(),
    commit: 'abc1234',
    ranges: ['main...fix/rewrite-the-gate',],
    source: 'package/a/src',
    focus: 'data lost or overwritten',
    reviewsFolder,
    runsFolder: '/runs',
    rulesFile: '/repo/doc/rules-every-pi-brief-carries.md',
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: writeReviewBrief.name,
      children: [
        it({
          name: 'writes the diff, the record of what the review was given and the brief, and answers the files it names',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             What writing the brief answered.
             */
            const briefed = await writeReviewBrief(aBriefRequest({
              reviewsFolder,
              git: gitAnswering({ files: theFilesTouched(), },),
            },),);
            expect(briefed.written,).toBe(true,);
            expect(await readFile(join(reviewsFolder, aReview(), 'change.diff',), 'utf8',),)
              .toBe(theFilesTouched().map(function patchOf(path,) {
                return `diff --git a/${path} b/${path}\n`;
              },)
                .join('',),);
            expect(
              JSON.parse(await readFile(
                join(reviewsFolder, aReview(), 'what-the-review-read.json',),
                'utf8',
              ),),
            ).toEqual({
              review: aReview(),
              commit: 'abc1234',
              // Both sides of the range are commits, so a reader can ask it again and get the same answer.
              ranges: ['base-commit...abc1234',],
              source: 'package/a/src',
              files: theFilesTouched(),
            },);
          },
        },),
        it({
          name: 'never writes into a review folder that is already there, and answers with what it holds',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             The review's own folder, which holds the brief of an earlier writing and a file this writer did not make.
             */
            const reviewFolder = join(reviewsFolder, aReview(),);
            await mkdir(reviewFolder, { recursive: true, },);
            await writeFile(join(reviewFolder, 'brief.md',), 'a brief of an earlier writing\n', 'utf8',);
            await writeFile(join(reviewFolder, 'owner.json',), '{"unit":"a"}\n', 'utf8',);
            /**
             What writing the brief answered over that folder.
             */
            const briefed = await writeReviewBrief(aBriefRequest({
              reviewsFolder,
              git: gitAnswering({ files: theFilesTouched(), },),
            },),);
            expect(briefed.written,).toBe(false,);
            // The caller reads which folder is in the way and what it holds, and nothing of it is replaced or removed.
            expect(briefed.line,).toContain(reviewFolder,);
            expect(briefed.line,).toContain('brief.md',);
            expect(briefed.line,).toContain('owner.json',);
            expect(await readFile(join(reviewFolder, 'brief.md',), 'utf8',),)
              .toBe('a brief of an earlier writing\n',);
            await expect(
              readFile(join(reviewFolder, 'change.diff',),),
            ).rejects.toThrow(MISSING_PATH_CODE,);
            await expect(
              readFile(join(reviewFolder, 'what-the-review-read.json',),),
            ).rejects.toThrow(MISSING_PATH_CODE,);
          },
        },),
        it({
          name: 'writes nothing and says nothing to review when the change leaves no file a review reads',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What writing the brief answered for a change that touches only a note.
             */
            const briefed = await writeReviewBrief(aBriefRequest({
              reviewsFolder: join(folder.path, 'reviews',),
              git: gitAnswering({ files: ['package/a/src/NOTES.md',], },),
            },),);
            expect(briefed.written,).toBe(false,);
            expect(briefed.line,).toContain('nothing to review',);
          },
        },),
        it({
          name: 'refuses to answer when git did not answer with a success, naming the question and what git said',
          fn: async () => {
            await using folder = await makeFolder();
            await expect(
              writeReviewBrief(aBriefRequest({
                reviewsFolder: join(folder.path, 'reviews',),
                git: async function fails() {
                  return {
                    status: 128,
                    stdout: '',
                    stderr: 'fatal: bad revision',
                  };
                },
              },),),
            ).rejects.toThrow(ReviewGitError,);
          },
        },),
      ],
    },),
  ],
},);
