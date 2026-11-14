/**
 What a review of a change reads: the range that names what changed, the diff of it under the reviewed source folder, and the
 files a reviewer is given, with what each status Git can report for a path means to that reading. Every git answer here is handed
 in by the test, so no checkout, worktree or process is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type CommandOutput,
  type ReviewMaterial,
  ReviewDiffError,
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
 The code file under review.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const code = aCodePath();
 ```
 */
function aCodePath(): string {
  return `${aSourceFolder()}/gate.rs`;
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
 A source file whose language a review does not read: the shim package's own `.cjs` implementation.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const source = aSourceFileNoReviewReads();
 ```
 */
function aSourceFileNoReviewReads(): string {
  return `${aSourceFolder()}/index.cjs`;
}

/**
 The type file beside that source, which a review does not read either.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const types = aTypeFileNoReviewReads();
 ```
 */
function aTypeFileNoReviewReads(): string {
  return `${aSourceFolder()}/index.d.cts`;
}

/**
 One path a stand-in git says a range changed: the status Git reported for it, and the two names a rename or a copy carries.

 This is what the writer reads from `git diff --name-status -z`: a status, the path, and, for a copy or a rename, the path it came
 from and the path it became.
 */
type ChangedInATest = {
  /**
   The status, as Git prints it, score included: `M`, `A`, `D`, `R100`, `T`, `C100`, or one the writer must refuse.
   */
  readonly status: string;

  /**
   The path in the newer side of the range, from the top of the checkout.
   */
  readonly path: string;

  /**
   The path in the older side of the range, which a rename or a copy also names.
   */
  readonly from?: string;
};

/**
 A modified path, which is Git's `M` and the change a range makes most often.

 @param path - the path, from the top of the checkout

 @returns the change Git reports for it

 @example
 ```ts
 const changed = aModifiedFile(aCodePath());
 ```
 */
function aModifiedFile(path: string,): ChangedInATest {
  return {
    status: 'M',
    path,
  };
}

/**
 An added path, which is Git's `A`.

 @param path - the path, from the top of the checkout

 @returns the change Git reports for it

 @example
 ```ts
 const changed = anAddedFile(aCodePath());
 ```
 */
function anAddedFile(path: string,): ChangedInATest {
  return {
    status: 'A',
    path,
  };
}

/**
 A deleted path, which is Git's `D`, and which leaves no file at the commit for a reviewer to read.

 @param path - the path, from the top of the checkout

 @returns the change Git reports for it

 @example
 ```ts
 const changed = aDeletedFile(aCodePath());
 ```
 */
function aDeletedFile(path: string,): ChangedInATest {
  return {
    status: 'D',
    path,
  };
}

/**
 A renamed path, which Git reports as `R` with a score and both names.

 @param was - the path in the older side of the range

 @param now - the path in the newer side of the range

 @returns the change Git reports for it

 @example
 ```ts
 const changed = aRenamedFile({ was: aCodePath(), now: `${aSourceFolder()}/gate2.rs` },);
 ```
 */
function aRenamedFile({ was, now, }: { readonly was: string; readonly now: string },): ChangedInATest {
  return {
    status: 'R100',
    path: now,
    from: was,
  };
}

/**
 A path whose type changed, which is Git's `T`: a regular file became a symbolic link or a submodule, or one of those became a
 file.

 @param path - the path, from the top of the checkout

 @returns the change Git reports for it

 @example
 ```ts
 const changed = aFileWhoseTypeChanged(aCodePath());
 ```
 */
function aFileWhoseTypeChanged(path: string,): ChangedInATest {
  return {
    status: 'T',
    path,
  };
}

/**
 A copied path, which Git reports as `C` with a score and both names when a configuration or an option asks it to detect copies.

 @param from - the path the copy came from

 @param path - the path the copy was made at

 @returns the change Git reports for it

 @example
 ```ts
 const changed = aCopiedFile({ from: aCodePath(), path: `${aSourceFolder()}/gate_copy.rs` },);
 ```
 */
function aCopiedFile({ from, path, }: { readonly from: string; readonly path: string },): ChangedInATest {
  return {
    status: 'C100',
    path,
    from,
  };
}

/**
 The names the reviewed commit holds for a set of changes: the newer name of every path that was added, copied, modified, renamed
 or changed the type of, which is what `git ls-tree` prints for a commit that holds them.

 @param changed - the changes a stand-in git reports

 @returns the paths the commit holds, one to a line

 @example
 ```ts
 const held = heldPathsIn({ changed: [aModifiedFile(aCodePath())], },);
 ```
 */
function heldPathsIn({ changed, }: { readonly changed: readonly ChangedInATest[] },): string {
  /**
   The statuses that leave a file at the newer name.
   */
  const leavingAFile = new Set([
    'A',
    'C',
    'M',
    'R',
    'T',
  ],);
  /**
   The newer names of those changes.
   */
  const paths = changed.filter(function leavesAFile(one,) {
    return leavingAFile.has(one.status.charAt(0,),);
  },)
    .map(function newerName(one,) {
      return one.path;
    },);
  return (paths.length === 0) ? '' : `${paths.join('\n',)}\n`;
}

/**
 What `git diff --name-status -z` prints for a set of changes: the status, the path, and for a copy or a rename both names, each
 field ended by a NUL byte.

 @param changed - the changes a stand-in git reports

 @returns the text git prints

 @example
 ```ts
 const printed = nameStatusPrintedFor({ changed: [aDeletedFile(aCodePath())], },);
 ```
 */
function nameStatusPrintedFor({ changed, }: { readonly changed: readonly ChangedInATest[] },): string {
  return changed.map(function asFields(one,) {
    return (one.from === undefined)
      ? `${one.status}\0${one.path}\0`
      : `${one.status}\0${one.from}\0${one.path}\0`;
  },)
    .join('',);
}

/**
 A stand-in git whose range changed exactly what a test gave it, and whose commit holds the newer name of every path that was not
 deleted.

 It answers the questions the writer asks now, and also the `--name-only` question the writer asked before it read statuses, so the
 controls still say what the old reading answered when the fix is taken out.

 @param changed - the changes the range made

 @returns a git that answers the three questions one reading asks

 @example
 ```ts
 const git = gitChanging({ changed: [aDeletedFile(aCodePath()), aModifiedFile(aTestFilePath())], },);
 ```
 */
function gitChanging({ changed, }: { readonly changed: readonly ChangedInATest[] },): (
  request: { readonly words: readonly string[] },
) => Promise<CommandOutput> {
  return async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
    if (words[0] === 'ls-tree') {
      return answered(heldPathsIn({ changed, },),);
    }
    if (words.includes('--name-status',)) {
      return answered(nameStatusPrintedFor({ changed, },),);
    }
    if (words.includes('--name-only',)) {
      /**
       The changes the old question would have kept: added and modified paths, when its filter asked for those alone.
       */
      const kept = (words.includes('--diff-filter=AM',))
        ? changed.filter(function isAddedOrModified(one,) {
          return (one.status === 'A') || (one.status === 'M');
        },)
        : changed;
      return answered((kept.length === 0) ? '' : `${kept.map(function newerName(one,) {
        return one.path;
      },)
        .join('\n',)}\n`,);
    }
    return answered('a diff of them',);
  };
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
 The paths a review of code says the change removed or renamed, refusing a material the test did not expect to be about code.

 @param material - what the reading answered

 @returns the removals and renames

 @throws Error when the change answered as something other than code

 @example
 ```ts
 const removed = codeRemovalsOf(material);
 ```
 */
function codeRemovalsOf(material: ReviewMaterial,) {
  if (material.kind !== 'code-and-build-decisions') {
    throw new Error(`a change of code answered as ${material.kind}`,);
  }
  return material.removedOrRenamed;
}

/**
 The paths a review of a change to tests alone says the change removed or renamed, refusing a material the test did not expect to
 be a change to tests.

 @param material - what the reading answered

 @returns the removals and renames

 @throws Error when the change answered as something other than tests

 @example
 ```ts
 const removed = testRemovalsOf(material);
 ```
 */
function testRemovalsOf(material: ReviewMaterial,) {
  if (material.kind !== 'tests-and-fixtures') {
    throw new Error(`a change of tests answered as ${material.kind}`,);
  }
  return material.removedOrRenamed;
}

/**
 Reads one range through a stand-in git.

 @param changed - the changes the range made

 @returns what the reading answered

 @example
 ```ts
 const material = await readAChangeThrough([aModifiedFile(aCodePath())]);
 ```
 */
async function readAChangeThrough(changed: readonly ChangedInATest[],): Promise<ReviewMaterial> {
  return await materialOfAReview({
    git: gitChanging({ changed, },),
    checkoutFolder: '/repo',
    ranges: [`${anIntegrationBranch()}...${aBranch()}`,],
    source: aSourceFolder(),
    commit: aCommit(),
  },);
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
                aCodePath(),
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
          name: 'leaves a source file whose language a review does not read out of both lists, so a change of one is never named as something to read',
          fn: async () => {
            expect(testsAndFixturesAmong({ paths: [aSourceFileNoReviewReads(), aTypeFileNoReviewReads(),], },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: materialOfAReview.name,
      children: [
        it({
          name: 'reads each range\'s diff under the source folder, asks git for every status a path can carry with both names, and keeps the code the reviewed commit still holds',
          fn: async () => {
            /**
             The git questions the reading asked, in order.
             */
            const asked: (readonly string[])[] = [];
            /**
             The changes the range made: code modified, code deleted, and a test modified.
             */
            const changed = [
              aModifiedFile(aCodePath(),),
              aDeletedFile(`${aSourceFolder()}/gone.rs`,),
              aModifiedFile(aTestFilePath(),),
            ];
            /**
             The answer to the diff question, which is written to the review's own file.
             */
            const diff = `diff --git a/${aCodePath()} b/${aCodePath()}`;
            /**
             A git that answers the reading's questions and records them.
             */
            const git = gitChanging({ changed, },);
            /**
             The stand-in, which records every question before answering it.
             */
            async function askingGit({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
              asked.push(words,);
              if ((words[0] === 'diff') && (!words.includes('--name-status',)) && (!words.includes('--name-only',))) {
                return answered(diff,);
              }
              return await git({ words, },);
            }
            /**
             The ranges the review reads.
             */
            const ranges = [`${anIntegrationBranch()}...${aBranch()}`,];
            /**
             What the reading answered.
             */
            const material = await materialOfAReview({
              git: askingGit,
              checkoutFolder: '/repo',
              ranges,
              source: aSourceFolder(),
              commit: aCommit(),
            },);
            expect(material.diff,).toBe(diff,);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([aCodePath(),],);
            expect(codeRemovalsOf(material,),).toEqual([{ status: 'deleted', path: `${aSourceFolder()}/gone.rs` },],);
            expect(asked,).toContainEqual([
              'diff',
              ranges[0],
              '--',
              aSourceFolder(),
            ],);
            expect(asked,).toContainEqual([
              'diff',
              '--name-status',
              '-z',
              '--find-renames',
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
             What the reading answered.
             */
            const material = await readAChangeThrough([aModifiedFile(aTestFilePath(),),],);
            expect(testsFilesOf(material,),).toEqual([aTestFilePath(),],);
            expect(testRemovalsOf(material,),).toEqual([],);
          },
        },),
        it({
          name: 'names a changed fixture with the changed test beside it, because a fixture is what the tests under it run',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([aModifiedFile(aTestFilePath(),), aModifiedFile(aFixturePath(),),],);
            expect(testsFilesOf(material,),).toEqual([aTestFilePath(), aFixturePath(),],);
          },
        },),
        it({
          name: 'reads a change of code and a test as the code alone, so a range that holds code is reviewed as it was',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([aModifiedFile(aCodePath(),), aModifiedFile(aTestFilePath(),),],);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([aCodePath(),],);
          },
        },),
        it({
          name: 'reads an added code file beside a changed test as a change of code, because an addition is a way a range changes code',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([anAddedFile(`${aSourceFolder()}/gate2.rs`,), aModifiedFile(aTestFilePath(),),],);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([`${aSourceFolder()}/gate2.rs`,],);
            expect(codeRemovalsOf(material,),).toEqual([],);
          },
        },),
        it({
          name: 'reads a deleted code file beside a changed test as a change of code, naming what was deleted, so the removal gets a review',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([aDeletedFile(aCodePath(),), aModifiedFile(aTestFilePath(),),],);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([],);
            expect(codeRemovalsOf(material,),).toEqual([{ status: 'deleted', path: aCodePath() },],);
          },
        },),
        it({
          name: 'reads a renamed code file beside a changed test as a change of code, naming both names, so the rename gets a review',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([
              aRenamedFile({
                was: aCodePath(),
                now: `${aSourceFolder()}/gate2.rs`,
              },),
              aModifiedFile(aTestFilePath(),),
            ],);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([`${aSourceFolder()}/gate2.rs`,],);
            expect(codeRemovalsOf(material,),).toEqual([{
              status: 'renamed',
              was: aCodePath(),
              now: `${aSourceFolder()}/gate2.rs`,
            },],);
          },
        },),
        it({
          name: 'reads a code file whose type changed beside a changed test as a change of code, because a change of type is a way a range changes code',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([aFileWhoseTypeChanged(aCodePath(),), aModifiedFile(aTestFilePath(),),],);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([aCodePath(),],);
            expect(codeRemovalsOf(material,),).toEqual([],);
          },
        },),
        it({
          name: 'reads a copied code file beside a changed test as a change of code, naming the copy, because a copy adds code the review must read',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([
              aCopiedFile({
                from: aCodePath(),
                path: `${aSourceFolder()}/gate_copy.rs`,
              },),
              aModifiedFile(aTestFilePath(),),
            ],);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(codeFilesOf(material,),).toEqual([`${aSourceFolder()}/gate_copy.rs`,],);
            expect(codeRemovalsOf(material,),).toEqual([],);
          },
        },),
        it({
          name: 'reads a deleted test beside a changed test as a change to tests alone, naming the test that was deleted, because the commit holds no code the change touched',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([aDeletedFile(aTestFilePath(),), aModifiedFile(`${aSourceFolder()}/gate.unit.test.ts`,),],);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([`${aSourceFolder()}/gate.unit.test.ts`,],);
            expect(testRemovalsOf(material,),).toEqual([{ status: 'deleted', path: aTestFilePath() },],);
          },
        },),
        it({
          name: 'reads a renamed test beside a changed test as a change to tests alone, naming both names, because no code was removed',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([
              aRenamedFile({
                was: aTestFilePath(),
                now: `${aSourceFolder()}/gate2_tests.rs`,
              },),
              aModifiedFile(`${aSourceFolder()}/gate.unit.test.ts`,),
            ],);
            expect(material.kind,).toBe('tests-and-fixtures',);
            expect(testsFilesOf(material,),).toEqual([
              `${aSourceFolder()}/gate2_tests.rs`,
              `${aSourceFolder()}/gate.unit.test.ts`,
            ],);
            expect(testRemovalsOf(material,),).toEqual([{
              status: 'renamed',
              was: aTestFilePath(),
              now: `${aSourceFolder()}/gate2_tests.rs`,
            },],);
          },
        },),
        it({
          name: 'answers nothing for a change of a source file whose language a review does not read beside its test, because a review of the test alone would leave that source unread',
          fn: async () => {
            expect((await readAChangeThrough([
              aModifiedFile(aSourceFileNoReviewReads(),),
              aModifiedFile(aTypeFileNoReviewReads(),),
              aModifiedFile(`${aSourceFolder()}/index.unit.test.ts`,),
            ],)).kind,).toBe('nothing-a-review-reads',);
          },
        },),
        it({
          name: 'answers nothing for a range that changes nothing under the source folder, so no review is started for it',
          fn: async () => {
            expect((await readAChangeThrough([],)).kind,).toBe('nothing-a-review-reads',);
          },
        },),
        it({
          name: 'keeps a manifest a change touched beside a note, because a reviewer can say what a changed profile, feature, dependency or task does, so the change is reviewed instead of answered as nothing to review',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([aModifiedFile(aManifestPath(),), aModifiedFile(aNotePath(),),],);
            expect(codeFilesOf(material,),).toEqual([aManifestPath(),],);
          },
        },),
        it({
          name: 'keeps a manifest a change touched beside the lockfile a tool writes, and no lockfile with it, because nobody reads a lockfile whole',
          fn: async () => {
            /**
             What the reading answered.
             */
            const material = await readAChangeThrough([
              aModifiedFile(aManifestPath(),),
              aModifiedFile(`${aSourceFolder()}/Cargo.lock`,),
            ],);
            expect(codeFilesOf(material,),).toEqual([aManifestPath(),],);
          },
        },),
        it({
          name: 'answers nothing for a change of a note beside a test, because the change is not tests and fixtures alone and the note is not something a review reads',
          fn: async () => {
            expect((await readAChangeThrough([aModifiedFile(aNotePath(),), aModifiedFile(`${aSourceFolder()}/gate.unit.test.ts`,),],)).kind,)
              .toBe('nothing-a-review-reads',);
          },
        },),
        it({
          name: 'leaves out a changed test the reviewed commit no longer holds, so a review is not given a file it cannot read',
          fn: async () => {
            /**
             A git whose range changed two tests, of which the commit holds one.
             */
            async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
              return answered((words[0] === 'ls-tree')
                ? `${aSourceFolder()}/gate.unit.test.ts`
                : (words.includes('--name-status',))
                  ? nameStatusPrintedFor({
                    changed: [
                      aModifiedFile(`${aSourceFolder()}/gate.unit.test.ts`,),
                      aModifiedFile(aTestFilePath(),),
                    ],
                  },)
                  : 'a diff of two tests',);
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
        ...['U', 'X'].map(function oneRefusedStatus(status,) {
          return it({
            name: `refuses the ${status} status Git documents, naming the status and the path, because guessing would decide a review from part of the change`,
            fn: async () => {
              /**
               A git that reports one path with that status.
               */
              async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
                return answered((words[0] === 'ls-tree')
                  ? `${aCodePath()}\n`
                  : ((words.includes('--name-status',)) ? `${status}\0${aCodePath()}\0` : 'a diff'),);
              }
              /**
               What the reading answered, or the error it threw.
               */
              let caught: unknown;
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
                caught = error;
              }
              expect(caught,).toBeInstanceOf(ReviewDiffError,);
              expect(String(caught,),).toContain(status,);
              expect(String(caught,),).toContain(aCodePath(),);
            },
          },);
        },),
        it({
          name: 'refuses a status answer that is not whole records, rather than reading a path as a status',
          fn: async () => {
            /**
             A git that names a status with no path after it.
             */
            async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
              return answered((words[0] === 'ls-tree') ? `${aCodePath()}\n` : ((words.includes('--name-status',)) ? 'M\0' : 'a diff'),);
            }
            /**
             What the reading answered, or the error it threw.
             */
            let caught: unknown;
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
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ReviewDiffError,);
            expect(String(caught,),).toContain('names a path without a whole status and path',);
          },
        },),
      ],
    },),
  ],
},);
