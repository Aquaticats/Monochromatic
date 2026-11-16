/**
 What a review of a change reads: the range that names what changed, the diff of it under the reviewed source folder, and the
 code files a reviewer is given. Every git answer here is handed in by the test, so no checkout, worktree or process is involved.

 @module
 */

import { spawnSync, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type APinnedCommit,
  type CommandOutput,
  ReviewGitError,
  aCommitOneQuestionNamed,
  codeFilesAmong,
  materialOfAReview,
  rangeOfABranch,
} from '../dist/final/node/index.mjs';

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-material-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The branch under review, which the expectations are built from.

 @returns the branch's name

 @example
 ```ts
 const branch = aBranch();
 ```
 */
function aBranch(): string {
  return 'fix/rewrite-the-gate';
}

/**
 The integration branch a change is read against.

 @returns the branch's name

 @example
 ```ts
 const integration = anIntegrationBranch();
 ```
 */
function anIntegrationBranch(): string {
  return 'main';
}

/**
 The commit the integration branch's tip is, in this file's answers.

 @returns the commit's id

 @example
 ```ts
 const tip = anIntegrationTip();
 ```
 */
function anIntegrationTip(): APinnedCommit {
  return aCommitOneQuestionNamed({ commit: 'main-tip', },);
}

/**
 The commit under review.

 @returns the commit's id

 @example
 ```ts
 const commit = aCommit();
 ```
 */
function aCommit(): APinnedCommit {
  return aCommitOneQuestionNamed({ commit: 'abc123', },);
}

/**
 The source folder whose changes are reviewed.

 @returns the folder's path

 @example
 ```ts
 const source = aSourceFolder();
 ```
 */
function aSourceFolder(): string {
  return 'package/a/src';
}

/**
 An answer from a stand-in git.

 @param stdout - what it prints

 @returns the answer

 @example
 ```ts
 const answered = answered('main');
 ```
 */
function answered(stdout: string,): CommandOutput {
  return {
    status: 0,
    stdout,
    stderr: '',
  };
}

/**
 The manifest of the package whose change is reviewed, which decides how that package is built and run.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const manifest = aManifestPath();
 ```
 */
function aManifestPath(): string {
  return `${aSourceFolder()}/Cargo.toml`;
}

/**
 The note beside the code, which no build reads.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const note = aNotePath();
 ```
 */
function aNotePath(): string {
  return `${aSourceFolder()}/notes.md`;
}

/**
 A stand-in git whose change touches exactly the paths given, which the reviewed commit still holds.

 Every revision is answered with a commit of its own, so a test can tell a question asked of a name from one asked of a commit.

 @param paths - the paths the change touched, each from the top of the checkout

 @returns a git that answers the questions one reading asks

 @example
 ```ts
 const git = gitTouching({ paths: [`${aSourceFolder()}/Cargo.toml`], },);
 ```
 */
function gitTouching({ paths, }: { readonly paths: readonly string[] },): (
  request: { readonly words: readonly string[] },
) => Promise<CommandOutput> {
  return async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
    if (words[0] === 'rev-parse') {
      return answered(`${(words[2] ?? '').replace('^{commit}', '')}-commit`,);
    }
    if (words[0] === 'merge-base') {
      return answered('base-commit',);
    }
    if (words[0] === 'ls-tree') {
      // Every listing whose lines are paths is NUL-separated.
      return answered(paths.map(function ended(path,) {
        return `${path}\0`;
      },)
        .join('',),);
    }
    return answered(rawAndPatchOf({ paths, },),);
  };
}

/**
 Writes one answer of `git diff --raw -z --patch` for a change that modifies the paths given.

 @param paths - the paths the change touched

 @returns the answer, with a raw entry per path and a patch that names each

 @example
 ```ts
 const answer = rawAndPatchOf({ paths: [`${aSourceFolder()}/gate.rs`], },);
 ```
 */
function rawAndPatchOf({ paths, }: { readonly paths: readonly string[] },): string {
  return `${paths.map(function entryOf(path,) {
    return `:100644 100644 0000000 1234567 M\0${path}\0`;
  },)
    .join('',)
     }\0${
     paths.map(function patchOf(path,) {
      return `diff --git a/${path} b/${path}\n`;
    },)
      .join('',)}`;
}

await describe({
  name: '',
  children: [
    describe({
      name: rangeOfABranch.name,
      children: [
        it({
          name: 'reads a branch the integration has not merged as the integration\'s tip commit against the tip\'s object name, so the range excludes what both share and later reads cannot see another tip',
          fn: async () => {
            expect(rangeOfABranch({
              tip: aCommit(),
              integrationTip: anIntegrationTip(),
              merges: [],
            },),).toBe(`${anIntegrationTip()}...${aCommit()}`,);
          },
        },),
        it({
          name: 'reads a branch a merge already took as that merge\'s first parent commit against the tip\'s object name, because the integration range would be empty',
          fn: async () => {
            /**
             The merge that took the branch's tip.
             */
            const merge = 'def456';
            /**
             The merge's own first parent, which is the commit the range starts at.
             */
            const parents = '789abc';
            expect(rangeOfABranch({
              tip: aCommit(),
              integrationTip: anIntegrationTip(),
              merges: [`${merge} ${parents} ${aCommit()}`,],
            },),).toBe(`${parents}...${aCommit()}`,);
          },
        },),
      ],
    },),
    describe({
      name: codeFilesAmong.name,
      children: [
        it({
          name: 'keeps the code a reviewer reads, and leaves out the tests, the fixtures and the notes beside it',
          fn: async () => {
            expect(codeFilesAmong({
              paths: [
                'package/a/src/gate.rs',
                'package/a/src/gate_tests.rs',
                'package/a/src/gate.unit.test.ts',
                'package/a/src/stand-in-git.ts',
                'package/a/test_support/fixture.rs',
                'package/a/test_support/Cargo.toml',
                'package/a/src/notes.md',
              ],
            },),).toEqual(['package/a/src/gate.rs',],);
          },
        },),
        it({
          name: 'keeps the manifest a change touched, and leaves out the lockfile a tool writes and the note beside it, because nobody reads a lockfile whole',
          fn: async () => {
            expect(codeFilesAmong({
              paths: [
                aManifestPath(),
                `${aSourceFolder()}/Cargo.lock`,
                aNotePath(),
              ],
            },),).toEqual([aManifestPath(),],);
          },
        },),
        it({
          name: 'keeps every variant of a manifest named for the tool it configures, whether the tool\'s word begins the name or ends it, because each variant decides the same build',
          fn: async () => {
            /**
             The variants, each a file this repository names the way it names that tool's other files.
             */
            const variants = [
              `${aSourceFolder()}/tsconfig.build.json`,
              `${aSourceFolder()}/nextest.windows-gnu.toml`,
              `${aSourceFolder()}/Containerfile.release`,
              `${aSourceFolder()}/native-test-base.Containerfile`,
              `${aSourceFolder()}/.cargo/config.toml`,
            ];
            expect(codeFilesAmong({ paths: variants, },),).toEqual(variants,);
          },
        },),
        it({
          name: 'leaves out a note that merely begins with a tool word, and a configuration inside a folder that only ends with the tool word, because neither is the build decision it looks like',
          fn: async () => {
            expect(codeFilesAmong({
              paths: [
                `${aSourceFolder()}/tsconfig-notes.md`,
                `${aSourceFolder()}/ContainerfileREADME.md`,
                `${aSourceFolder()}/foo.cargo/config.toml`,
              ],
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: materialOfAReview.name,
      children: [
        it({
          name: 'asks every name as a commit once, reads one diff question per range, and keeps only the code the reviewed commit still holds',
          fn: async () => {
            /**
             The git questions the reading asked, in order.
             */
            const asked: (readonly string[])[] = [];
            /**
             A git that records every question and answers the reading's.
             */
            async function git({ words, }: { readonly words: readonly string[] }): Promise<CommandOutput> {
              asked.push(words,);
              if (words[0] === 'rev-parse') {
                return answered(`${(words[2] ?? '').replace('^{commit}', '')}-commit`,);
              }
              if (words[0] === 'merge-base') {
                return answered('base-commit',);
              }
              if (words[0] === 'ls-tree') {
                return answered(`${aSourceFolder()}/gate.rs\0`,);
              }
              return answered(rawAndPatchOf({
                paths: [
                  `${aSourceFolder()}/gate.rs`,
                  `${aSourceFolder()}/gate_tests.rs`,
                  `${aSourceFolder()}/gone.rs`,
                ],
              },),);
            }
            /**
             The range the review reads.
             */
            const range = `${anIntegrationBranch()}...${aBranch()}`;
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git,
              checkoutFolder: '/repo',
              ranges: [range,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.diff,).toContain(aSourceFolder(),);
            expect(material.files,).toEqual([`${aSourceFolder()}/gate.rs`,],);
            // Both sides of the range and the commit under review are commits, and the range names them.
            expect(material.commit,).toBe(`${aCommit()}-commit`,);
            expect(material.ranges,).toEqual([`base-commit...${aCommit()}-commit`,],);
            // The range's left side is read before the commit under review, so a name that moves is read once.
            expect(asked.slice(0, 2,),).toEqual([
              ['rev-parse', '--verify', `${anIntegrationBranch()}^{commit}`,],
              ['rev-parse', '--verify', `${aCommit()}^{commit}`,],
            ],);
            expect(asked,).toContainEqual([
              'merge-base',
              `${anIntegrationBranch()}-commit`,
              `${aCommit()}-commit`,
            ],);
            // One question gives the diff and the paths it touches, so no second read can name another change.
            expect(asked.filter(function isADiff(words,) {
              return words[0] === 'diff';
            },),).toEqual([[
              'diff',
              '--raw',
              '--patch',
              '-z',
              `base-commit...${aCommit()}-commit`,
              '--',
              aSourceFolder(),
            ],],);
            expect(asked,).toContainEqual([
              'ls-tree',
              '--name-only',
              '-r',
              '-z',
              `${aCommit()}-commit`,
              '--',
              aSourceFolder(),
            ],);
          },
        },),
        it({
          name: 'keeps a path holding a line break, a quote, a backslash and a non-UTF-8 byte, because every listing of paths is read NUL-separated',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Where the disposable repository is.
             */
            const repository = join(folder.path, 'repository',);
            await mkdir(repository, { recursive: true, },);
            /**
             Runs one git command in the repository and answers what it printed.
             */
            function git({ words, input, }: { readonly words: readonly string[]; readonly input?: string },): string {
              /**
               What git answered.
               */
              const printed = spawnSync('git', words, {
                cwd: repository,
                encoding: 'utf8',
                ...((input === undefined) ? {} : { input, }),
              },);
              if ((printed.status !== 0) || (printed.error !== undefined)) {
                throw new Error(`git ${words.join(' ',)} answered ${String(printed.status,)}: ${printed.stderr}`,);
              }
              return printed.stdout;
            }
            git({ words: ['init', '--initial-branch=main', '--quiet',], },);
            /**
             The path a change adds, which holds the four bytes a line-based listing mangles: a line break, a quote, a backslash
             and a byte that is no UTF-8, which Git writes as the octal `\377` and this reads back as the replacement character.
             */
            const changedPath = 'package/a/src/line\nbreak"quote\\backslash\uFFFD.ts';
            /**
             The base file, and the change that adds that path.
             */
            const base = 'export const base = true;\n';
            /**
             The changed file's text.
             */
            const changed = 'export const changed = true;\n';
            git({
              words: ['fast-import', '--quiet',],
              input: [
                `blob\nmark :1\ndata ${String(Buffer.byteLength(base,),)}\n${base}`,
                'commit refs/heads/main\nmark :2\ncommitter Disposable Review <disposable@example.invalid> 1700000000 +0000\n'
                  + 'data 5\nbase\nM 100644 :1 package/a/src/base.ts\n',
                `blob\nmark :3\ndata ${String(Buffer.byteLength(changed,),)}\n${changed}`,
                'commit refs/heads/fix\nmark :4\ncommitter Disposable Review <disposable@example.invalid> 1700000000 +0000\n'
                  + 'data 7\nchange\nfrom :2\nM 100644 :3 "package/a/src/line\\nbreak\\"quote\\\\backslash\\377.ts"\n',
                'done\n',
              ].join('',),
            },);
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: async function answers({ words, folder: where, },) {
                /**
                 What git answered in that checkout.
                 */
                const printed = spawnSync('git', words, {
                  cwd: where,
                  encoding: 'utf8',
                },);
                return {
                  status: printed.status ?? 1,
                  stdout: printed.stdout ?? '',
                  stderr: printed.stderr ?? '',
                };
              },
              checkoutFolder: repository,
              ranges: ['main...fix',],
              source: aSourceFolder(),
              commit: 'fix',
            },);
            expect(material.diff,).not.toBe('',);
            expect(material.files,).toEqual([changedPath,],);
          },
        },),
        it({
          name: 'refuses to answer when git did not answer with a success, naming the question and what git said',
          fn: async () => {
            /**
             A git that refuses every question.
             */
            async function git(): Promise<CommandOutput> {
              return {
                status: 128,
                stdout: '',
                stderr: 'not a git repository',
              };
            }
            try {
              await materialOfAReview({
                git,
                checkoutFolder: '/repo',
                ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
                source: aSourceFolder(),
                commit: aCommit(),
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(ReviewGitError,);
              expect(String(error,),).toContain('not a git repository',);
              return;
            }
            throw new Error('a git that refused every question produced a review material',);
          },
        },),
        it({
          name: 'answers no file for a change that touches only tests under the source folder, which is what makes a review of it not start',
          fn: async () => {
            /**
             A git whose change is one test file.
             */
            const git = gitTouching({ paths: [`${aSourceFolder()}/gate_tests.rs`,], },);
            expect((await materialOfAReview({
              git,
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },)).files,).toEqual([],);
          },
        },),
        it({
          name: 'keeps a manifest a change touched beside a note, because a reviewer can say what a changed profile, feature, dependency or task does, so the change is reviewed instead of answered as nothing to review',
          fn: async () => {
            expect((await materialOfAReview({
              git: gitTouching({ paths: [aManifestPath(), aNotePath(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },)).files,).toEqual([aManifestPath(),],);
          },
        },),
        it({
          name: 'keeps a manifest a change touched beside the lockfile a tool writes, and no lockfile with it, because nobody reads a lockfile whole',
          fn: async () => {
            expect((await materialOfAReview({
              git: gitTouching({ paths: [aManifestPath(), `${aSourceFolder()}/Cargo.lock`,], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },)).files,).toEqual([aManifestPath(),],);
          },
        },),
        it({
          name: 'answers no file for a change that touches only notes and tests under the source folder, which is what makes a review of it not start',
          fn: async () => {
            expect((await materialOfAReview({
              git: gitTouching({ paths: [aNotePath(), `${aSourceFolder()}/gate.unit.test.ts`,], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },)).files,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
