/**
 What a review of a change reads: the range that names what changed, the diff of it under the reviewed source folder, and the
 files a reviewer is given. Every git answer here is handed in by the test, so no checkout, worktree or process is involved.

 @module
 */

import {
  mkdtemp,
  readFile,
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
  type CommandOutput,
  CODE_FILE_EXTENSIONS,
  FOCUS_OF_A_CHANGE_REVIEW,
  LEFT_OUT_FILE_EXTENSIONS,
  type ReviewMaterial,
  ReviewGitError,
  briefFileOfAReview,
  codeFilesAmong,
  materialOfAReview,
  pathsInAListing,
  rangeOfABranch,
  testsAndFixturesAmong,
  writeReviewBrief,
} from '../dist/final/node/index.mjs';

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
 The commit under review.

 @returns the commit's id

 @example
 ```ts
 const commit = aCommit();
 ```
 */
function aCommit(): string {
  return 'abc123';
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
 A test file of the code under review.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const test = aTestFilePath();
 ```
 */
function aTestFilePath(): string {
  return `${aSourceFolder()}/gate_tests.rs`;
}

/**
 A fixture the tests use, which is not shipped code.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const fixture = aFixturePath();
 ```
 */
function aFixturePath(): string {
  return `${aSourceFolder()}/stand-in-git.ts`;
}

/**
 The shim package's own CommonJS source and the type file beside it, which a review reads because both extensions are code: the
 change of `7ec2fd422d` held these, their test and their note, and a range that changed them was once answered "nothing to
 review".

 @returns their paths, from the top of the checkout

 @example
 ```ts
 const files = shimSourceAndItsTypes();
 ```
 */
function shimSourceAndItsTypes(): readonly string[] {
  return [
    `${aSourceFolder()}/index.cjs`,
    `${aSourceFolder()}/index.d.cts`,
  ];
}

/**
 The source files of extensions in neither list of the files a review reads.

 @returns their paths, from the top of the checkout

 @example
 ```ts
 const files = filesOfAnUnlistedKind();
 ```
 */
function filesOfAnUnlistedKind(): readonly string[] {
  return [
    `${aSourceFolder()}/thing.zig`,
    `${aSourceFolder()}/other.zig`,
  ];
}

/**
 The Kotlin test the review's first finding named, which a `src/test` layout and a `Test.kt` name both mark.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const test = aKotlinTestPath();
 ```
 */
function aKotlinTestPath(): string {
  return 'package/linter/kotlin/src/test/kotlin/dev/monochromatic/detekt/RequireKDocTest.kt';
}

/**
 The recorded answer the review's second finding named, which sits in a `fixture` folder beside its test.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const fixture = aRecordedAnswerInAFixtureFolder();
 ```
 */
function aRecordedAnswerInAFixtureFolder(): string {
  return 'package/crate-release/cli/src/fixture/run-still-going.json';
}

/**
 The recorded answer a test reads that sits in a folder no list names, which a change to tests reads anyway.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const answer = aRecordedAnswerInNoNamedFolder();
 ```
 */
function aRecordedAnswerInNoNamedFolder(): string {
  return `${aSourceFolder()}/recordings/run-still-going.json`;
}

/**
 The files a review of code gives its reviewer, refusing a material the test did not expect to be about tests or nothing.

 @param material - what the reading answered

 @returns the files to read

 @throws Error when the change answered as something other than code

 @example
 ```ts
 const files = codeFilesOf(material);
 ```
 */
function codeFilesOf(material: ReviewMaterial,): readonly string[] {
  if (material.kind !== 'code-and-build-decisions') {
    throw new Error(`a change of code answered as ${material.kind}`,);
  }
  return material.files;
}

/**
 The files a review of a change to tests alone gives its reviewer, refusing a material the test did not expect to be a change to
 tests.

 @param material - what the reading answered

 @returns the tests and fixtures to read

 @throws Error when the change answered as something other than tests

 @example
 ```ts
 const files = testsFilesOf(material);
 ```
 */
function testsFilesOf(material: ReviewMaterial,): readonly string[] {
  if (material.kind !== 'tests-and-fixtures') {
    throw new Error(`a change of tests answered as ${material.kind}`,);
  }
  return material.files;
}

/**
 A stand-in git whose change touches exactly the paths given, which the reviewed commit still holds.

 @param paths - the paths the change touched, each from the top of the checkout

 @returns a git that answers the three questions one reading asks

 @example
 ```ts
 const git = gitTouching({ paths: [`${aSourceFolder()}/Cargo.toml`], },);
 ```
 */
function gitTouching({ paths, removed = [], }: {
  readonly paths: readonly string[];
  readonly removed?: readonly string[];
},): (
  request: { readonly words: readonly string[] },
) => Promise<CommandOutput> {
  return async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
    /**
     The paths as git prints them, one to a line.
     */
    // Git prints a listing it cannot quote with `-z`, so one NUL byte ends each name.
    const listed = (paths.length === 0) ? '' : `${paths.join('\0',)}\0`;
    /**
     The paths the reviewed commit holds, which a removed path is not.
     */
    const held = paths.filter(function isHeld(path,) {
      return !removed.includes(path,);
    },);
    /**
     The held paths as git prints them with `-z`.
     */
    const heldListing = (held.length === 0) ? '' : `${held.join('\0',)}\0`;
    return answered((words[0] === 'ls-tree') ? heldListing : ((words[1] === '--name-only') ? listed : 'a diff of them'),);
  };
}

/**
 The path that is a file of each kind the table names, which each row's range is built from.

 @returns one path per kind, by the table's letter

 @example
 ```ts
 const paths = aPathOfEachKind();
 ```
 */
function aPathOfEachKind(): Readonly<Record<string, string>> {
  return {
    T: `${aSourceFolder()}/gate.unit.test.ts`,
    F: `${aSourceFolder()}/fixture/answer.json`,
    C: `${aSourceFolder()}/gate.rs`,
    U: `${aSourceFolder()}/recordings/answer.zig`,
    M: `${aSourceFolder()}/catalog.json`,
    P: `${aSourceFolder()}/notes.md`,
  };
}

/**
 One file of the table's kind T, which a change to tests holds.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const test = aTestFileOfTheTable();
 ```
 */
function aTestFileOfTheTable(): string {
  return aPathOfEachKind().T ?? '';
}

/**
 The letter the table names a path by.

 @param path - one of the paths of `aPathOfEachKind`

 @returns the letter, or a question mark for a path the table does not know

 @example
 ```ts
 const letter = letterOf(anUnlistedPath);
 ```
 */
function letterOf(path: string,): string {
  /**
   The path of each kind, by its letter.
   */
  const paths = aPathOfEachKind();
  return Object.keys(paths,)
    .find(function isThePath(letter,) {
      return paths[letter] === path;
    },) ?? '?';
}

/**
 The rules file a brief names first.

 @returns the file's path

 @example
 ```ts
 const rules = aRulesFile();
 ```
 */
function aRulesFile(): string {
  return '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md';
}

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeAFolder();
 ```
 */
async function makeAFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-table-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes the brief of one change and reads it back, for a test that asks what a reviewer reads.

 @param folder - the folder the review's own files go in

 @param paths - the paths the change touched, each from the top of the checkout

 @returns the brief, and the line the writer answered with

 @example
 ```ts
 const { brief, line } = await aBriefAndItsLine({ folder, paths: [aTestFileOfTheTable()], },);
 ```
 */
async function aBriefAndItsLine({ folder, paths, removed = [], }: {
  readonly folder: string;
  readonly paths: readonly string[];
  readonly removed?: readonly string[];
},): Promise<{ readonly brief: string; readonly line: string }> {
  /**
   The review this change is read as.
   */
  const review = `row-${paths.length}`;
  /**
   What the writer answered.
   */
  const briefed = await writeReviewBrief({
    git: gitTouching({ paths, removed, },),
    checkoutFolder: '/repo',
    review,
    commit: aCommit(),
    ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
    source: aSourceFolder(),
    focus: FOCUS_OF_A_CHANGE_REVIEW,
    reviewsFolder: join(folder, 'reviews',),
    runsFolder: join(folder, 'runs',),
    rulesFile: aRulesFile(),
  },);
  return {
    brief: await readFile(
      briefFileOfAReview({
        reviewsFolder: join(folder, 'reviews',),
        review,
      },),
      'utf8',
    ),
    line: briefed.line,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: rangeOfABranch.name,
      children: [
        it({
          name: 'reads a branch the integration has not merged as the integration against its own branch, so the range excludes what both share',
          fn: async () => {
            expect(rangeOfABranch({
              branch: aBranch(),
              tip: aCommit(),
              integration: anIntegrationBranch(),
              merges: [],
            },),).toBe(`${anIntegrationBranch()}...${aBranch()}`,);
          },
        },),
        it({
          name: 'reads a branch a merge already took as that merge\'s first parent against the branch, because the integration range would be empty',
          fn: async () => {
            /**
             The merge that took the branch's tip.
             */
            const merge = 'def456';
            /**
             The merge's own parents, of which the second is the branch's tip.
             */
            const parents = '789abc';
            expect(rangeOfABranch({
              branch: aBranch(),
              tip: aCommit(),
              integration: anIntegrationBranch(),
              merges: [`${merge} ${parents} ${aCommit()}`,],
            },),).toBe(`${merge}^1...${aBranch()}`,);
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
                'package/a/tests/shim.rs',
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
      name: testsAndFixturesAmong.name,
      children: [
        it({
          name: 'keeps the tests and fixtures a review of code leaves out, and leaves out the code and the note those tests are about',
          fn: async () => {
            expect(testsAndFixturesAmong({
              paths: [
                `${aSourceFolder()}/gate.rs`,
                aTestFilePath(),
                `${aSourceFolder()}/gate.unit.test.ts`,
                aFixturePath(),
                `${aSourceFolder()}/test_support/fixture.rs`,
                `${aSourceFolder()}/tests/shim.rs`,
                aManifestPath(),
                aNotePath(),
              ],
            },),).toEqual([
              aTestFilePath(),
              `${aSourceFolder()}/gate.unit.test.ts`,
              aFixturePath(),
              `${aSourceFolder()}/test_support/fixture.rs`,
              `${aSourceFolder()}/tests/shim.rs`,
            ],);
          },
        },),
        it({
          name: 'keeps a file under a tests folder whatever its extension, because a cargo integration test is written like the code it tests and its name says nothing',
          fn: async () => {
            expect(testsAndFixturesAmong({
              paths: [
                `${aSourceFolder()}/tests/shim.rs`,
                `tests/whole-checkout.rs`,
                `${aSourceFolder()}/contest.rs`,
              ],
            },),).toEqual([
              `${aSourceFolder()}/tests/shim.rs`,
              'tests/whole-checkout.rs',
            ],);
          },
        },),
        it({
          name: 'keeps a `.cjs` source and its `.d.cts` types out of the tests list, because both are the code of a change and not the tests beside it',
          fn: async () => {
            expect(testsAndFixturesAmong({ paths: [...shimSourceAndItsTypes(), aTestFilePath(),], },),).toEqual([aTestFilePath(),],);
          },
        },),
        it({
          name: 'keeps the files of a kind no list names among the files a review reads, because a review reads them as code rather than drop them',
          fn: async () => {
            expect(codeFilesAmong({ paths: [...filesOfAnUnlistedKind(), aTestFilePath(),], },),).toEqual(filesOfAnUnlistedKind(),);
          },
        },),
        it({
          name: 'keeps one changed file of every code extension this repository tracks, so no language it is written in is dropped from a review',
          fn: async () => {
            /**
             One path per code extension, named the way a file of it is named here.
             */
            const paths = [...CODE_FILE_EXTENSIONS].map(function aPathOfTheExtension(extension,) {
              return `${aSourceFolder()}/a-file${extension}`;
            },);
            expect(codeFilesAmong({ paths, },),).toEqual(paths,);
          },
        },),
        it({
          name: 'leaves one changed file of every left-out extension out of the files a review reads, so a note, a data file, generated output and a picture are never read as the code of a change',
          fn: async () => {
            /**
             One path per left-out extension, named the way a file of it is named here. The two container-file endings are left out
             of this list because a path that ends with one is a build decision by name, whatever the rest of its name says, and
             the test above covers that reading.
             */
            const paths = [...LEFT_OUT_FILE_EXTENSIONS]
              .filter(function isNoBuildFileNameEnding(extension,) {
                return (!extension.endsWith('.Containerfile',)) && (!extension.endsWith('.Dockerfile',));
              },)
              .map(function aPathOfTheExtension(extension,) {
                return `${aSourceFolder()}/a-file${extension}`;
              },);
            expect(paths.length,).toBeGreaterThan(0,);
            expect(codeFilesAmong({ paths, },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: materialOfAReview.name,
      children: [
        it({
          name: 'reads each range\'s diff under the source folder, keeps only the code the reviewed commit still holds, and asks git for a listing it cannot quote, one character per byte',
          fn: async () => {
            /**
             The git questions the reading asked, in order.
             */
            const asked: (readonly string[])[] = [];
            /**
             What the stand-in git answers, by the first word that tells the questions apart.
             */
            const answers = new Map<string, string>([
              ['diff', `diff --git a/${aSourceFolder()}/gate.rs b/${aSourceFolder()}/gate.rs`],
              ['ls-tree', `${aSourceFolder()}/gate.rs`],
            ],);
            /**
             A git that answers the reading's questions.
             */
            async function git({ words, }: { readonly words: readonly string[] }): Promise<CommandOutput> {
              asked.push(words,);
              if (words[0] === 'ls-tree') {
                return answered(answers.get('ls-tree') ?? '',);
              }
              return answered((words[1] === '--name-only')
                ? `${aSourceFolder()}/gate.rs\0${aSourceFolder()}/gate_tests.rs\0${aSourceFolder()}/gone.rs\0`
                : (answers.get('diff') ?? ''),);
            }
            /**
             The ranges the review reads.
             */
            const ranges = [`${anIntegrationBranch()}...${aBranch()}`,];
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git,
              checkoutFolder: '/repo',
              ranges,
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.diff,).toContain(aSourceFolder(),);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([`${aSourceFolder()}/gate.rs`,],);
            expect(asked,).toContainEqual([
              'diff',
              ranges[0],
              '--',
              aSourceFolder(),
            ],);
            expect(asked,).toContainEqual([
              'diff',
              '--name-only',
              '-z',
              '--diff-filter=AM',
              ranges[0],
              '--',
              aSourceFolder(),
            ],);
            expect(asked,).toContainEqual([
              'ls-tree',
              '--name-only',
              '-r',
              '-z',
              aCommit(),
              '--',
              aSourceFolder(),
            ],);
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
          name: 'answers a change of one test file as a change to tests alone, naming that test, so a review of it starts instead of being refused',
          fn: async () => {
            /**
             A git whose change is one test file.
             */
            async function git({ words, }: { readonly words: readonly string[] }): Promise<CommandOutput> {
              return answered((words[0] === 'ls-tree')
                ? aTestFilePath()
                : (words[1] === '--name-only')
                  ? aTestFilePath()
                  : 'a diff of a test');
            }
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git,
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([aTestFilePath(),],);
          },
        },),
        it({
          name: 'names a changed fixture with the changed test beside it, because a fixture is what the tests under it run',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aTestFilePath(), aFixturePath(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(testsFilesOf(material,),).toEqual([aTestFilePath(), aFixturePath(),],);
          },
        },),
        it({
          name: 'reads a change that holds code and a test as a change to tests, listing the code beside the test, because a changed test is read whatever else the change holds',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [`${aSourceFolder()}/gate.rs`, aTestFilePath(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([`${aSourceFolder()}/gate.rs`, aTestFilePath(),],);
            expect((material.kind === 'tests-and-fixtures') ? material.filesOfCode : [],)
              .toEqual([`${aSourceFolder()}/gate.rs`,],);
          },
        },),
        it({
          name: 'reads the change of `7ec2fd422d`, a `.cjs` source and its `.d.cts` types beside their test and their note, as a change to tests that lists the code and leaves the note out by name',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [...shimSourceAndItsTypes(), `${aSourceFolder()}/index.unit.test.ts`, aNotePath(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([
              ...shimSourceAndItsTypes(),
              `${aSourceFolder()}/index.unit.test.ts`,
            ],);
            expect((material.kind === 'tests-and-fixtures') ? material.filesOfCode : [],).toEqual(shimSourceAndItsTypes(),);
            expect((material.kind === 'tests-and-fixtures') ? material.filesLeftOut : [],).toEqual([{
              path: aNotePath(),
              why: 'a note, which no test reads',
            },],);
          },
        },),
        it({
          name: 'names the files of a kind no list names beside the files to read, so the line the writer prints can say a kind it does not know was read as code',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [...filesOfAnUnlistedKind(), `${aSourceFolder()}/gate.rs`,], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect((material.kind === 'code-and-build-decisions') ? material.filesOfAnUnlistedKind : [],)
              .toEqual(filesOfAnUnlistedKind(),);
          },
        },),
        it({
          name: 'answers nothing for a range that changes nothing under the source folder, with words that say the ranges change no file there, so no review is started for it',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('nothing-a-review-reads',);
            expect((material.kind === 'nothing-a-review-reads') ? material.whyNothingIsRead : '',)
              .toBe(`it changes no file under ${aSourceFolder()}`,);
          },
        },),
        it({
          name: 'answers nothing for a change of a note alone, with words that name the note and the rule a review reads a change by, so a refusal says why',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aNotePath(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('nothing-a-review-reads',);
            expect((material.kind === 'nothing-a-review-reads') ? material.whyNothingIsRead : '')
              .toBe('a review reads the code of a change, the files that decide its build, or its tests and fixtures alone, '
                + `and it changes \`${aNotePath()}\``,);
          },
        },),
        it({
          name: 'keeps a manifest a change touched beside a note, because a reviewer can say what a changed profile, feature, dependency or task does, so the change is reviewed instead of answered as nothing to review',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aManifestPath(), aNotePath(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(codeFilesOf(material,),).toEqual([aManifestPath(),],);
          },
        },),
        it({
          name: 'keeps a manifest a change touched beside the lockfile a tool writes, and no lockfile with it, because nobody reads a lockfile whole',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aManifestPath(), `${aSourceFolder()}/Cargo.lock`,], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(codeFilesOf(material,),).toEqual([aManifestPath(),],);
          },
        },),
        it({
          name: 'reads a test whose change also holds a note as a change to tests, and names the note as left out with the reason, so a line of prose never takes the review away',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aNotePath(), `${aSourceFolder()}/gate.unit.test.ts`,], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([`${aSourceFolder()}/gate.unit.test.ts`,],);
            expect((material.kind === 'tests-and-fixtures') ? material.filesLeftOut : [],).toEqual([{
              path: aNotePath(),
              why: 'a note, which no test reads',
            },],);
          },
        },),
        it({
          name: 'reads a test and a recorded answer of a kind no list names as a change to tests, listing both, so the unlisted answer never takes the code path and the test is never dropped',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aTestFilePath(), `${aSourceFolder()}/recordings/answer.zig`,], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([aTestFilePath(), `${aSourceFolder()}/recordings/answer.zig`,],);
          },
        },),
        it({
          name: 'leaves out a changed test the reviewed commit no longer holds, so a review is not given a file it cannot read',
          fn: async () => {
            /**
             A git whose range touched two tests, of which the commit holds one.
             */
            async function git({ words, }: { readonly words: readonly string[] }): Promise<CommandOutput> {
              return answered((words[0] === 'ls-tree')
                ? `${aSourceFolder()}/gate.unit.test.ts`
                : (words[1] === '--name-only')
                  ? `${aSourceFolder()}/gate.unit.test.ts\0${aTestFilePath()}\0`
                  : 'a diff of two tests');
            }
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git,
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(testsFilesOf(material,),).toEqual([`${aSourceFolder()}/gate.unit.test.ts`,],);
          },
        },),
        it({
          name: 'answers a Kotlin test alone as a change to tests, naming it, which a review read as code until a `src/test` layout and a `Test.kt` name were counted',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aKotlinTestPath(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: 'package/linter/kotlin/src',
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([aKotlinTestPath(),],);
          },
        },),
        it({
          name: 'answers a test and the recorded answer in its `fixture` folder as a change to tests, naming both, which was answered as nothing to review until the folder was counted',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aTestFilePath(), aRecordedAnswerInAFixtureFolder(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([aTestFilePath(), aRecordedAnswerInAFixtureFolder(),],);
          },
        },),
        it({
          name: 'reads a recorded answer beside a test although no folder list names the folder it sits in, so a test and the answer it reads are never answered as nothing to review',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: gitTouching({ paths: [aTestFilePath(), aRecordedAnswerInNoNamedFolder(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([aTestFilePath(), aRecordedAnswerInNoNamedFolder(),],);
          },
        },),
        it({
          name: 'answers nothing for a recorded answer alone, because no test in the change reads it and no list names the folder, which is the refusal the writer says why for',
          fn: async () => {
            expect((await materialOfAReview({
              git: gitTouching({ paths: [aRecordedAnswerInNoNamedFolder(),], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },)).kind,).toBe('nothing-a-review-reads',);
          },
        },),


        describe({
          name: 'the table of a change',
          children: [
            it({
              name: 'reads and leaves out exactly what the table in `plan-of-a-review` says, for every one of the sixty-four combinations, so a changed test is never dropped and a file a review cannot judge never decides for the files beside it',
              fn: async () => {
            /**
             The table, one row per combination: the kinds the range holds, the kind of change, the kinds a review reads, and the
             kinds its brief names as left out. It is the table in `plan-of-a-review`'s doc, letter for letter.
             */
                const table: readonly (readonly [string, string, string, string])[] = [
                  ['', 'nothing-a-review-reads', '', ''],
                  ['T', 'tests-and-fixtures', 'T', ''],
                  ['F', 'tests-and-fixtures', 'F', ''],
                  ['TF', 'tests-and-fixtures', 'TF', ''],
                  ['C', 'code-and-build-decisions', 'C', ''],
                  ['TC', 'tests-and-fixtures', 'TC', ''],
                  ['FC', 'tests-and-fixtures', 'FC', ''],
                  ['TFC', 'tests-and-fixtures', 'TFC', ''],
                  ['U', 'code-and-build-decisions', 'U', ''],
                  ['TU', 'tests-and-fixtures', 'TU', ''],
                  ['FU', 'tests-and-fixtures', 'FU', ''],
                  ['TFU', 'tests-and-fixtures', 'TFU', ''],
                  ['CU', 'code-and-build-decisions', 'CU', ''],
                  ['TCU', 'tests-and-fixtures', 'TCU', ''],
                  ['FCU', 'tests-and-fixtures', 'FCU', ''],
                  ['TFCU', 'tests-and-fixtures', 'TFCU', ''],
                  ['M', 'nothing-a-review-reads', '', ''],
                  ['TM', 'tests-and-fixtures', 'TM', ''],
                  ['FM', 'tests-and-fixtures', 'FM', ''],
                  ['TFM', 'tests-and-fixtures', 'TFM', ''],
                  ['CM', 'code-and-build-decisions', 'C', 'M'],
                  ['TCM', 'tests-and-fixtures', 'TCM', ''],
                  ['FCM', 'tests-and-fixtures', 'FCM', ''],
                  ['TFCM', 'tests-and-fixtures', 'TFCM', ''],
                  ['UM', 'code-and-build-decisions', 'U', 'M'],
                  ['TUM', 'tests-and-fixtures', 'TUM', ''],
                  ['FUM', 'tests-and-fixtures', 'FUM', ''],
                  ['TFUM', 'tests-and-fixtures', 'TFUM', ''],
                  ['CUM', 'code-and-build-decisions', 'CU', 'M'],
                  ['TCUM', 'tests-and-fixtures', 'TCUM', ''],
                  ['FCUM', 'tests-and-fixtures', 'FCUM', ''],
                  ['TFCUM', 'tests-and-fixtures', 'TFCUM', ''],
                  ['P', 'nothing-a-review-reads', '', ''],
                  ['TP', 'tests-and-fixtures', 'T', 'P'],
                  ['FP', 'tests-and-fixtures', 'F', 'P'],
                  ['TFP', 'tests-and-fixtures', 'TF', 'P'],
                  ['CP', 'code-and-build-decisions', 'C', 'P'],
                  ['TCP', 'tests-and-fixtures', 'TC', 'P'],
                  ['FCP', 'tests-and-fixtures', 'FC', 'P'],
                  ['TFCP', 'tests-and-fixtures', 'TFC', 'P'],
                  ['UP', 'code-and-build-decisions', 'U', 'P'],
                  ['TUP', 'tests-and-fixtures', 'TU', 'P'],
                  ['FUP', 'tests-and-fixtures', 'FU', 'P'],
                  ['TFUP', 'tests-and-fixtures', 'TFU', 'P'],
                  ['CUP', 'code-and-build-decisions', 'CU', 'P'],
                  ['TCUP', 'tests-and-fixtures', 'TCU', 'P'],
                  ['FCUP', 'tests-and-fixtures', 'FCU', 'P'],
                  ['TFCUP', 'tests-and-fixtures', 'TFCU', 'P'],
                  ['MP', 'nothing-a-review-reads', '', ''],
                  ['TMP', 'tests-and-fixtures', 'TM', 'P'],
                  ['FMP', 'tests-and-fixtures', 'FM', 'P'],
                  ['TFMP', 'tests-and-fixtures', 'TFM', 'P'],
                  ['CMP', 'code-and-build-decisions', 'C', 'MP'],
                  ['TCMP', 'tests-and-fixtures', 'TCM', 'P'],
                  ['FCMP', 'tests-and-fixtures', 'FCM', 'P'],
                  ['TFCMP', 'tests-and-fixtures', 'TFCM', 'P'],
                  ['UMP', 'code-and-build-decisions', 'U', 'MP'],
                  ['TUMP', 'tests-and-fixtures', 'TUM', 'P'],
                  ['FUMP', 'tests-and-fixtures', 'FUM', 'P'],
                  ['TFUMP', 'tests-and-fixtures', 'TFUM', 'P'],
                  ['CUMP', 'code-and-build-decisions', 'CU', 'MP'],
                  ['TCUMP', 'tests-and-fixtures', 'TCUM', 'P'],
                  ['FCUMP', 'tests-and-fixtures', 'FCUM', 'P'],
                  ['TFCUMP', 'tests-and-fixtures', 'TFCUM', 'P'],
                ];
                expect(table.length,).toBe(64,);
            /**
             The contents of the one folder this test writes briefs into.
             */
                await using folder = await makeAFolder();
            /* oxlint-disable eslint/no-await-in-loop -- one row's reading and brief after another, because each assertion names
               the row it failed on and the rows share no state. See doc/lint/sequential-awaits.md. */
                for (const [present, kind, reads, leftOut,] of table) {
              /**
               What the row says about the brief's requirements and the line's notice, which follow from the kinds it reads.
               */
                  const row = {
                    reads,
                    leftOut,
                    kind,
                    asksTheCode: (reads.includes('C',) || reads.includes('U',)),
                    namesAnUnlistedKind: reads.includes('U',),
                  };
              /**
               The paths of the kinds the row holds.
               */
                  const paths = Array.from(present,)
                    .map(function pathOf(letter,) {
                      return aPathOfEachKind()[letter] ?? '';
                    },);
              /**
               What the reading answered for them.
               */
                  const material = await materialOfAReview({
                    git: gitTouching({ paths, },),
                    checkoutFolder: '/repo',
                    ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
                    source: aSourceFolder(),
                    commit: aCommit(),
                  },);
                  expect([present, material.kind,],).toEqual([present, kind,],);
                  if (material.kind === 'nothing-a-review-reads') {
                    expect([present, reads, leftOut,],).toEqual([present, '', '',],);
                    continue;
                  }
                  expect([present, material.files.map(letterOf,).join('',),],).toEqual([present, reads,],);
                  expect([present, material.filesLeftOut.map(function letterOfLeftOut(file,) {
                    return letterOf(file.path,);
                  },).join('',),],).toEqual([present, leftOut,],);
              // A path is in exactly one of the two, so a brief cannot list a file and then say it is left out.
                  expect(
                    material.files.filter(function isAlsoLeftOut(path,) {
                      return material.filesLeftOut.some(function isTheSameFile(file,) {
                        return file.path === path;
                      },);
                    },),
                  ).toEqual([],);
              /**
               The brief this row writes, and the line it answers with.
               */
                  const briefed = await writeReviewBrief({
                    git: gitTouching({ paths, },),
                    checkoutFolder: '/repo',
                    review: `row-${present}`,
                    commit: aCommit(),
                    ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
                    source: aSourceFolder(),
                    focus: FOCUS_OF_A_CHANGE_REVIEW,
                    reviewsFolder: join(folder.path, 'reviews',),
                    runsFolder: join(folder.path, 'runs',),
                    rulesFile: aRulesFile(),
                  },);
              /**
               The brief, read back as a reviewer reads it.
               */
                  const brief = await readFile(
                    briefFileOfAReview({
                      reviewsFolder: join(folder.path, 'reviews',),
                      review: `row-${present}`,
                    },),
                    'utf8',
                  );
                  expect([present, briefed.written,],).toEqual([present, true,],);
              /**
               Where the files to read end and the files left out begin.
               */
                  const headingAt = brief.indexOf('Read each of these files in full before you answer:',);
              /**
               Where the left-out paragraph begins, or minus one when every file is read.
               */
                  const leftOutAt = brief.indexOf('These files are left out, with the reason:',);
              // Every file the row reads is under the files to read, once, and never in the left-out paragraph.
                  for (const file of material.files) {
                    expect([present, file, brief.split(file,).length - 1,],).toEqual([present, file, 1,],);
                    expect([present, brief.indexOf(file,) > headingAt,],).toEqual([present, true,],);
                    expect([present, (leftOutAt === (-1)) || (brief.indexOf(file,) < leftOutAt),],).toEqual([present, true,],);
                  }
              // Every file the row leaves out is named once, with its reason, and never among the files to read.
                  for (const left of material.filesLeftOut) {
                    expect([present, brief.includes(`- \`${left.path}\` is left out: ${left.why}.`,),],).toEqual([present, true,],);
                    expect([present, left.path, brief.split(left.path,).length - 1,],).toEqual([present, left.path, 1,],);
                    expect([present, brief.indexOf(left.path,) > leftOutAt,],).toEqual([present, true,],);
                  }
                  expect([present, brief.includes('Every file the change touched is read above.',),],)
                    .toEqual([present, material.filesLeftOut.length === 0,],);
              // The requirements are the kind of change the row is, and the code's are asked whenever the row reads code.
                  if (row.kind === 'code-and-build-decisions') {
                    expect([present, brief.includes('It must meet these requirements:',),],).toEqual([present, true,],);
                  }
                  else if (row.asksTheCode) {
                    expect([present, brief.includes('The code must meet these requirements:',),],).toEqual([present, true,],);
                    expect([present, brief.includes('the change is to them alone',),],).toEqual([present, false,],);
                  }
                  else {
                    expect([present, brief.includes('the change is to them alone',),],).toEqual([present, true,],);
                    expect([present, brief.includes('The code must meet these requirements:',),],).toEqual([present, false,],);
                  }
              // The breaks are asked for exactly when the row is a change to tests.
                  expect([present, brief.includes('For each changed test above, plant a break under it',),],)
                    .toEqual([present, (row.kind === 'tests-and-fixtures'),],);
              // The line names an unlisted kind wherever the review reads one, and says nothing of it otherwise.
                  expect([present, briefed.line.includes('no list of this writer names are read as code',),],)
                    .toEqual([present, row.namesAnUnlistedKind,],);
                  if (row.namesAnUnlistedKind) {
                    expect([present, briefed.line.includes(aPathOfEachKind().U ?? '',),],).toEqual([present, true,],);
                  }
                  // The same row with every path it holds removed: the kind of the change is the same, every file it reads is
                  // named, and no sentence says every touched file is read while one is absent.
                  const gone = await aBriefAndItsLine({
                    folder: folder.path,
                    paths,
                    removed: paths,
                  },);
                  expect([present, gone.brief.includes('Every file the change touched is read above.',),],).toEqual([present, false,],);
                  for (const file of material.files) {
                    expect([present, file, gone.brief.split(file,).length - 1,],).toEqual([present, file, 1,],);
                  }
                  expect([present, gone.brief.includes('The code must meet these requirements:',),],).toEqual([present, row.asksTheCode,],);
                  if (row.kind === 'code-and-build-decisions') {
                    expect([present, gone.brief.includes('It must meet these requirements:',),],).toEqual([present, true,],);
                  }
                  expect([present, gone.line.includes('no list of this writer names are read as code',),],)
                    .toEqual([present, row.namesAnUnlistedKind,],);
                }
            /* oxlint-enable eslint/no-await-in-loop */
              },
            },),
            it({
              name: 'lists a recorded answer beside its test once, under the files to read, and never says it is left out',
              fn: async () => {
            /**
             The contents of the one folder this test writes its brief into.
             */
                await using folder = await makeAFolder();
            /**
             The recorded answer beside the changed test.
             */
                const recordedAnswer = `${aSourceFolder()}/recordings/answer.json`;
            /**
             The brief this change writes, and the line it answers with.
             */
                const { brief, line, } = await aBriefAndItsLine({
                  folder: folder.path,
                  paths: [aTestFileOfTheTable(), recordedAnswer,],
                },);
                expect([brief.split(recordedAnswer,).length - 1,],).toEqual([1,],);
                expect([brief.includes(`- \`${recordedAnswer}\``,),],).toEqual([true,],);
                expect([brief.includes(`\`${recordedAnswer}\` is left out`,),],).toEqual([false,],);
                expect([line.includes('2 files to read',),],).toEqual([true,],);
              },
            },),
            it({
              name: 'names a production file the change removed, and asks the code requirements for it, so a range that deletes code beside a test is not reviewed as a change to tests alone',
              fn: async () => {
                /**
                 The contents of the one folder this test writes its brief into.
                 */
                await using folder = await makeAFolder();
                /**
                 The production file the change removed, and the test it changed.
                 */
                const removedCode = `${aSourceFolder()}/prod.ts`;
                /**
                 The brief this change writes, and the line it answers with.
                 */
                const { brief, line, } = await aBriefAndItsLine({
                  folder: folder.path,
                  paths: [removedCode, `${aSourceFolder()}/prod.unit.test.ts`,],
                  removed: [removedCode,],
                },);
                expect([brief.includes(`- \`${removedCode}\``,),],).toEqual([true,],);
                expect([brief.includes('The code must meet these requirements:',),],).toEqual([true,],);
                expect([brief.includes('the change is to them alone',),],).toEqual([false,],);
                expect([brief.includes('Every file the change touched is read above.',),],).toEqual([false,],);
                expect([line.includes('2 files to read',),],).toEqual([true,],);
              },
            },),
            it({
              name: 'reads a file of a kind no list names beside a test as code: the brief asks the code requirements and the line names the unlisted kind',
              fn: async () => {
            /**
             The contents of the one folder this test writes its brief into.
             */
                await using folder = await makeAFolder();
            /**
             The changed test, and the file of a kind no list names beside it.
             */
                const unlistedCode = `${aSourceFolder()}/thing.zig`;
            /**
             The brief this change writes, and the line it answers with.
             */
                const { brief, line, } = await aBriefAndItsLine({
                  folder: folder.path,
                  paths: [aTestFileOfTheTable(), unlistedCode,],
                },);
                expect([brief.includes(`- \`${unlistedCode}\``,),],).toEqual([true,],);
                expect([brief.includes('The code must meet these requirements:',),],).toEqual([true,],);
                expect([brief.includes('the change is to them alone',),],).toEqual([false,],);
                expect([line.includes('no list of this writer names are read as code',),],).toEqual([true,],);
                expect([line.includes(unlistedCode,),],).toEqual([true,],);
              },
            },),
          ],
        },),
        describe({
          name: pathsInAListing.name,
          children: [
            it({
              name: 'reads a listing Git cannot quote, one name per NUL byte, so a name with a space, a quote or a non-ASCII letter is the file\'s own name and not a quoted rendering of it',
              fn: async () => {
                expect(pathsInAListing({ listing: '', },),).toEqual([],);
                expect(pathsInAListing({ listing: 'package/a/src/gate.rs\0package/a/src/gate.unit.test.ts\0', },),)
                  .toEqual(['package/a/src/gate.rs', 'package/a/src/gate.unit.test.ts',],);
            /**
             A name whose bytes are UTF-8, as Git prints it with `-z`: one character per byte.
             */
                const utf8Name = Buffer.from('package/a/src/é.unit.test.ts', 'utf8',)
                  .toString('latin1',);
                expect(pathsInAListing({ listing: `${utf8Name}\0`, },),).toEqual(['package/a/src/é.unit.test.ts',],);
                expect(pathsInAListing({ listing: 'package/a/src/a "quoted" name.rs\0', },),)
                  .toEqual(['package/a/src/a "quoted" name.rs',],);
              },
            },),
            it({
              name: 'shows a name whose bytes are not UTF-8, or that holds a newline, a tab, a backslash or a backtick, with a hex escape that is the file\'s bytes, so the brief names a file a shell can still reach',
              fn: async () => {
            /**
             A name holding one byte that is not UTF-8, as a filesystem can.
             */
                const notUtf8 = Buffer.from([...Buffer.from('package/a/src/', 'latin1',), 0xE9, ...Buffer.from('.unit.test.ts', 'latin1',)],)
                  .toString('latin1',);
                expect(pathsInAListing({ listing: `${notUtf8}\0`, },),)
                  .toEqual([String.raw`package/a/src/\xe9.unit.test.ts`,],);
                expect(pathsInAListing({ listing: 'package/a/src/a\\b.rs\0', },),).toEqual([String.raw`package/a/src/a\x5cb.rs`,],);
                expect(pathsInAListing({ listing: 'package/a/src/a\nb.rs\0', },),).toEqual([String.raw`package/a/src/a\x0ab.rs`,],);
                expect(pathsInAListing({ listing: 'package/a/src/a\tb.rs\0', },),).toEqual([String.raw`package/a/src/a\x09b.rs`,],);
                expect(pathsInAListing({ listing: 'package/a/src/a`b.rs\0', },),).toEqual([String.raw`package/a/src/a\x60b.rs`,],);
              },
            },),
          ],
        },),
      ],
    },),
  ],
},);
