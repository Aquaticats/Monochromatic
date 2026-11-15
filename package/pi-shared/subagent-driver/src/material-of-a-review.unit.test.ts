/**
 What a review of a change reads: the range that names what changed, the diff of it under the reviewed source folder, and the
 files a reviewer is given. Every git answer here is handed in by the test, so no checkout, worktree or process is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type CommandOutput,
  CODE_FILE_EXTENSIONS,
  LEFT_OUT_FILE_EXTENSIONS,
  type ReviewMaterial,
  ReviewGitError,
  codeFilesAmong,
  materialOfAReview,
  rangeOfABranch,
  testsAndFixturesAmong,
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
function gitTouching({ paths, }: { readonly paths: readonly string[] },): (
  request: { readonly words: readonly string[] },
) => Promise<CommandOutput> {
  return async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
    /**
     The paths as git prints them, one to a line.
     */
    const listed = `${paths.join('\n',)}\n`;
    return answered((words[0] === 'ls-tree') ? listed : ((words[1] === '--name-only') ? listed : 'a diff of them'),);
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
          name: 'reads each range\'s diff under the source folder, keeps only the code the reviewed commit still holds, and asks git for each with the words that filter them',
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
                ? `${aSourceFolder()}/gate.rs\n${aSourceFolder()}/gate_tests.rs\n${aSourceFolder()}/gone.rs`
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
              '--diff-filter=AM',
              ranges[0],
              '--',
              aSourceFolder(),
            ],);
            expect(asked,).toContainEqual([
              'ls-tree',
              '--name-only',
              '-r',
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
          name: 'reads a change of code and a test as the code alone, so a range that holds code is reviewed as it was',
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
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([`${aSourceFolder()}/gate.rs`,],);
          },
        },),
        it({
          name: 'reads a change of a `.cjs` source and its `.d.cts` types beside their test and their note as code, which is the change of `7ec2fd422d` that a range names and that was once answered "nothing to review"',
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
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual(shimSourceAndItsTypes(),);
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
          name: 'answers nothing for a change of a note beside a test, because the change is not tests and fixtures alone and the note is not something a review reads',
          fn: async () => {
            expect((await materialOfAReview({
              git: gitTouching({ paths: [aNotePath(), `${aSourceFolder()}/gate.unit.test.ts`,], },),
              checkoutFolder: '/repo',
              ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
              source: aSourceFolder(),
              commit: aCommit(),
            },)).kind,).toBe('nothing-a-review-reads',);
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
                  ? `${aSourceFolder()}/gate.unit.test.ts\n${aTestFilePath()}`
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
      ],
    },),
  ],
},);
