/**
 Landing one branch: every way it can end, each with the answers a test writes instead of a real Git.

 @module
 */

import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  commitTheOpenMerge,
  cgroupWordsOf,
  landABranch,
  type Report,
  type Repository,
  WRAPPER_COMPILE_LIMITS,
} from '../dist/final/node/index.mjs';
import {
  disposableFolder,
  fakeRepository,
  type Handlers,
  openAMerge,
} from './fixtures-of-a-fake-repository.ts';

/**
 The words a merge of the branch every test lands is asked with.
 */
const MERGE = 'merge --no-ff --no-commit --quiet fix/a';

/**
 The words a commit of the merge is asked with.
 */
const COMMIT = 'commit --quiet --message merge(a): b';

/**
 Milliseconds a test's commit wait is bounded by, short so a retry is quick.
 */
const PAUSE_MILLISECONDS = 1;

/**
 Most tries a test gives a merge's commit.
 */
const TRIES = 2;

/**
 What bounds one Git command in a test.
 */
const GIT_TIMEOUT_MILLISECONDS = 5_000;

/**
 Most bytes of a command's output a test keeps.
 */
const OUTPUT_BYTES = 1_000_000;

/**
 A report that keeps every line, so a test reads what a landing said.

 @returns the report and what it kept

 @example
 ```ts
 const said = rememberedReport();
 ```
 */
function rememberedReport(): {
  readonly report: Report;
  readonly lines: string[];
  readonly warnings: string[];
} {
  /**
   The lines written to standard output.
   */
  const lines: string[] = [];
  /**
   The lines written to standard error.
   */
  const warnings: string[] = [];
  return {
    lines,
    warnings,
    report: {
      line: function line(text: string,) {
        lines.push(text,);
      },
      warning: function warning(text: string,) {
        warnings.push(text,);
      },
    },
  };
}

/**
 The checkout a test's landing asks its questions of.

 @param repository - the answers a test wrote

 @returns the checkout, bounded the way a sequence expects

 @example
 ```ts
 const checkout = checkoutOf(repository,);
 ```
 */
function checkoutOf(repository: {
  readonly folder: string;
  readonly run: Repository['run'];
},): Repository {
  return {
    folder: repository.folder,
    run: repository.run,
    timeoutMilliseconds: GIT_TIMEOUT_MILLISECONDS,
    outputBytes: OUTPUT_BYTES,
  };
}

/**
 The words `systemd-run` is asked with when it runs one task for a landing.

 @param task - the task, repository-qualified

 @returns the words, joined as the answers a test writes are keyed

 @example
 ```ts
 const line = cgroupLine('//package/a/b:lint',);
 ```
 */
function cgroupLine(task: string,): string {
  return [
    ...cgroupWordsOf({
      limits: WRAPPER_COMPILE_LIMITS,
      collect: false,
    },),
    'mise',
    'run',
    task,
    '--',
    '--no-run',
  ].join(' ',);
}

/**
 The bounds every test's landing is given.

 @returns what bounds every command it starts

 @example
 ```ts
 const landingBounds = bounds();
 ```
 */
function bounds(): {
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
} {
  return {
    tries: TRIES,
    pauseMilliseconds: PAUSE_MILLISECONDS,
    commitTimeoutMilliseconds: GIT_TIMEOUT_MILLISECONDS,
  };
}

/**
 The answers a merge of `fix/a` that lands cleanly leaves.

 @param folder - the folder the repository sits in

 @returns what each command answers

 @example
 ```ts
 const answers = landingAnswers('/folder',);
 ```
 */
function landingAnswers(folder: string,): Handlers {
  /**
   Where an open merge is recorded.
   */
  const marker = join(
    folder,
    '.git',
    'MERGE_HEAD',
  );
  return {
    [MERGE]: function merged() {
      return { stdout: 'merged', };
    },
    'diff --cached --name-only': function staged() {
      return { stdout: 'doc/a.md\n', };
    },
    [COMMIT]: async function committed() {
      await rm(marker, { force: true, },);
      return { stdout: '' };
    },
    'rev-list --count main..fix/a': function left() {
      return { stdout: '0\n', };
    },
    'log -1 --format=%h %s main': function last() {
      return { stdout: 'abcdef1234 merge(a): b\n', };
    },
    'diff --name-only main...fix/a': function changed() {
      return { stdout: '', };
    },
  };
}

/**
 Lands `fix/a` in a repository whose answers a test wrote.

 @param answers - what each command answers

 @param folder - the folder the repository sits in

 @returns what the landing came to, what it said, and what it asked Git

 @example
 ```ts
 const landing = await land({ answers, folder, },);
 ```
 */
async function land({
  answers,
  folder,
}: {
  readonly answers: Handlers;
  readonly folder: string;
},): Promise<{
  readonly outcome: Awaited<ReturnType<typeof landABranch>>;
  readonly lines: string[];
  readonly gitLines: readonly string[];
}> {
  /**
   The fake repository.
   */
  const repository = await fakeRepository({
    folder,
    answers,
  },);
  /**
   What the landing said.
   */
  const said = rememberedReport();
  return {
    outcome: await landABranch({
      repository: checkoutOf(repository,),
      branch: 'fix/a',
      message: 'merge(a): b',
      ...bounds(),
      report: said.report,
    },),
    lines: said.lines,
    gitLines: repository.gitLines(),
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: 'landABranch',
      children: [
        it({
          name: 'commits the merge and reports what landed, what is left and which release paths it changed',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('landed',);
            /**
             * What the landing came to.
             */
            const landing = await land({
              folder,
              answers: {
                ...landingAnswers(folder,),
                'diff --name-only main...fix/a': function changed() {
                  return { stdout: 'package/git-policy/cli/src/lib.rs\nCargo.toml\ndoc/a.md\n', };
                },
              },
            },);
            expect(landing.outcome.landing,).toBe('landed',);
            expect(landing.outcome.landing === 'landed' && landing.outcome.report.commitsNotOnMain,).toBe(0,);
            expect(landing.outcome.landing === 'landed' && landing.outcome.report.releasePaths,).toEqual([
              'package/git-policy/cli/src/lib.rs',
              'Cargo.toml',
            ],);
            expect(landing.lines,).toContain('abcdef1234 merge(a): b',);
            expect(landing.lines,).toContain('0 commits of the branch not on main',);
            expect(landing.gitLines,).toContain('merge --no-ff --no-commit --quiet fix/a',);
          },
        },),
        it({
          name: 'refuses to start while a merge is already open, and asks Git nothing else',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('open-merge',);
            /**
             * The fake repository.
             */
            const repository = await fakeRepository({
              folder,
              answers: {},
            },);
            await openAMerge(repository,);
            /**
             * What the landing said.
             */
            const said = rememberedReport();
            /**
             * What the landing came to.
             */
            const outcome = await landABranch({
              repository: checkoutOf(repository,),
              branch: 'fix/a',
              message: 'merge(a): b',
              ...bounds(),
              report: said.report,
            },);
            expect(outcome.landing,).toBe('refused-merge-already-open',);
            expect(repository.gitLines(),).toEqual(['rev-parse --path-format=absolute --git-path MERGE_HEAD',],);
          },
        },),
        it({
          name: 'commits the reflow a format run left in the main checkout, then lands the branch',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('reflow',);
            /**
             * How many times the merge was asked.
             */
            let merges = 0;
            await mkdir(join(
              folder,
              'doc',
            ), { recursive: true, },);
            await writeFile(join(
              folder,
              'doc/a.md',
            ), 'one\ntwo\n',);
            /**
             * What the landing came to.
             */
            const landing = await land({
              folder,
              answers: {
                ...landingAnswers(folder,),
                [MERGE]: function merged() {
                  merges += 1;
                  return merges === 1
                    ? { status: 1, stderr: 'error: Your local changes would be overwritten by merge:\n\tdoc/a.md\n', }
                    : { stdout: '' };
                },
                'show HEAD:doc/a.md': function committed() {
                  return { stdout: 'one two\n', };
                },
                'show fix/a:doc/a.md': function incoming() {
                  return { status: 128, stdout: '' };
                },
                'commit --quiet --message style(*): commit the reflow a format run left in the main checkout -- doc/a.md': function reflowed() {
                  return { stdout: '' };
                },
              },
            },);
            expect(landing.lines,).toContain('committed reflow edits as they stand: doc/a.md',);
            expect(landing.outcome.landing,).toBe('landed',);
            expect(merges,).toBe(2,);
          },
        },),
        it({
          name: 'leaves uncommitted edits that change words where they are, naming every one',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('words',);
            /**
             * What the landing came to.
             */
            const landing = await land({
              folder,
              answers: {
                ...landingAnswers(folder,),
                [MERGE]: function blocked() {
                  return { status: 1, stderr: 'error: would be overwritten by merge:\n\tdoc/a.md\n', };
                },
                'show HEAD:doc/a.md': function committed() {
                  return { stdout: 'one two\n', };
                },
                'show fix/a:doc/a.md': function incoming() {
                  return { stdout: 'one three\n', };
                },
              },
            },);
            expect(landing.outcome.landing,).toBe('refused-uncommitted-edits',);
            expect(landing.outcome.landing === 'refused-uncommitted-edits' && landing.outcome.paths,).toEqual(['doc/a.md',],);
            expect(landing.lines.join('\n',),).toContain('these uncommitted edits change words',);
          },
        },),
        it({
          name: 'reports a merge Git refused and says whether it left one open',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('conflict',);
            /**
             * What the landing came to.
             */
            const landing = await land({
              folder,
              answers: {
                ...landingAnswers(folder,),
                [MERGE]: function conflicted() {
                  return { status: 1, stderr: 'CONFLICT (content): merge conflict in doc/a.md', };
                },
              },
            },);
            expect(landing.outcome.landing,).toBe('merge-failed',);
            expect(landing.outcome.landing === 'merge-failed' && landing.outcome.mergeIsOpen,).toBe(false,);
            expect(landing.lines,).toContain('no merge was left open',);
          },
        },),
        it({
          name: 'reports a merged tree that does not build and leaves the merge open',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('build',);
            /**
             * What the landing came to.
             */
            const landing = await land({
              folder,
              answers: {
                ...landingAnswers(folder,),
                'diff --cached --name-only': function staged() {
                  return { stdout: 'package/git-policy/cli/src/lib.rs\n', };
                },
                [cgroupLine('//package/git-policy/cli:native:test:host',)]: function failed() {
                  return { status: 101, stderr: 'error[E0308]: mismatched types\n --> src/lib.rs:1:1\n' };
                },
              },
            },);
            expect(landing.outcome.landing,).toBe('tree-does-not-compile',);
            expect(landing.lines.join('\n',),).toContain('A MERGE IS OPEN IN THE MAIN CHECKOUT: fix it, or run git merge --abort',);
            expect(landing.lines.join('\n',),).toContain('error[E0308]: mismatched types',);
            expect(landing.gitLines.includes(COMMIT,),).toBe(false,);
          },
        },),
        it({
          name: 'stops after every try when the commit never closes the merge',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('ceiling',);
            /**
             * What the landing came to.
             */
            const landing = await land({
              folder,
              answers: {
                ...landingAnswers(folder,),
                [MERGE]: async function merged() {
                  await writeFile(join(
                    folder,
                    '.git',
                    'MERGE_HEAD',
                  ), 'HEAD\n',);
                  return { stdout: '' };
                },
                [COMMIT]: function refused() {
                  return { status: 1, stderr: 'the branch keeps changing', };
                },
              },
            },);
            expect(landing.outcome.landing,).toBe('commit-refused',);
            expect(landing.outcome.landing === 'commit-refused' && landing.outcome.tries,).toBe(TRIES,);
            expect(landing.lines,).toContain(`A MERGE IS OPEN IN THE MAIN CHECKOUT after ${String(TRIES,)} commit tries`,);
          },
        },),
      ],
    },),
    describe({
      name: 'commitTheOpenMerge',
      children: [
        it({
          name: 'says no merge is open when the marker is gone',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('no-merge',);
            /**
             * The fake repository.
             */
            const repository = await fakeRepository({
              folder,
              answers: {},
            },);
            /**
             * What the commit said.
             */
            const said = rememberedReport();
            /**
             * What committing came to.
             */
            const outcome = await commitTheOpenMerge({
              repository: checkoutOf(repository,),
              message: 'merge(a): b',
              paths: [],
              ...bounds(),
              commitTimeoutMilliseconds: GIT_TIMEOUT_MILLISECONDS,
              report: said.report,
            },);
            expect(outcome.commit,).toBe('no-merge-open',);
            expect(said.lines,).toEqual(['no merge is open',],);
          },
        },),
        it({
          name: 'refuses to commit over a path the merge left unresolved',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('unresolved',);
            /**
             * The fake repository.
             */
            const repository = await fakeRepository({
              folder,
              answers: {
                'diff --name-only --diff-filter=U': function unresolved() {
                  return { stdout: 'doc/a.md\n', };
                },
              },
            },);
            await openAMerge(repository,);
            /**
             * What the commit said.
             */
            const said = rememberedReport();
            /**
             * What was thrown.
             */
            let thrown: unknown;
            try {
              await commitTheOpenMerge({
                repository: checkoutOf(repository,),
                message: 'merge(a): b',
                paths: [],
                ...bounds(),
                commitTimeoutMilliseconds: GIT_TIMEOUT_MILLISECONDS,
                report: said.report,
              },);
            }
            catch (caught) {
              thrown = caught;
            }
            expect((thrown as Error).message,).toContain('unresolved files remain:\ndoc/a.md',);
          },
        },),
        it({
          name: 'stops at once when Git says there is nothing to commit, naming that the merge stays open',
          fn: async () => {
            /**
             * A folder this test alone uses.
             */
            const folder = await disposableFolder('nothing-to-commit',);
            /**
             * The fake repository.
             */
            const repository = await fakeRepository({
              folder,
              answers: {
                'commit --quiet --no-edit': function nothing() {
                  return { status: 1, stdout: 'nothing to commit, working tree clean\n' };
                },
              },
            },);
            await openAMerge(repository,);
            /**
             * What the commit said.
             */
            const said = rememberedReport();
            /**
             * What committing came to.
             */
            const outcome = await commitTheOpenMerge({
              repository: checkoutOf(repository,),
              message: '',
              paths: [],
              ...bounds(),
              commitTimeoutMilliseconds: GIT_TIMEOUT_MILLISECONDS,
              report: said.report,
            },);
            expect(outcome.commit,).toBe('nothing-to-commit',);
            expect(said.lines.join('\n',),).toContain('the merge is still open',);
            expect(repository.gitLines().filter(function isCommit(line: string,) {
              return line.startsWith('commit ',);
            },).length,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
