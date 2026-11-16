/**
 The writer of review briefs against disposable folders: a change of tests and fixtures alone gets a brief that lists each
 changed test and asks for a break planted under it, a change of code is described as it was, a range that changes nothing a
 review reads is refused with words that say why, and a file of a kind no list names is read as code and named in the line.

 Every git answer here is handed in by the test, so no checkout or worktree is involved, and every folder is one the test made and
 throws away. Nothing starts a run.

 @module
 */

import { existsSync, } from 'node:fs';
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
  type BriefOfOneReview,
  type CommandOutput,
  FOCUS_OF_A_CHANGE_REVIEW,
  briefFileOfAReview,
  readingFileOfAReview,
  writeReviewBrief,
} from '../dist/final/node/index.mjs';

/**
 The change a review reads.

 @returns the change's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'a-change-to-tests';
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
  return 'abc1234';
}

/**
 The folder whose changes are reviewed.

 @returns the folder's path, from the top of the checkout

 @example
 ```ts
 const source = aSourceFolder();
 ```
 */
function aSourceFolder(): string {
  return 'package/a/src';
}

/**
 What changed, as `git diff` takes a range.

 @returns the range

 @example
 ```ts
 const range = aRange();
 ```
 */
function aRange(): string {
  return 'main...fix/rewrite-the-gate';
}

/**
 The code file a change of code leaves to read.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const code = aCodeFile();
 ```
 */
function aCodeFile(): string {
  return `${aSourceFolder()}/gate.rs`;
}

/**
 The changed test file a change to tests alone leaves to read.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const test = aTestFile();
 ```
 */
function aTestFile(): string {
  return `${aSourceFolder()}/gate_tests.rs`;
}

/**
 The changed fixture a change to tests alone leaves to read.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const fixture = aFixtureFile();
 ```
 */
function aFixtureFile(): string {
  return `${aSourceFolder()}/stand-in-git.ts`;
}

/**
 The Kotlin test the review's first finding named, which a `src/test` layout and a `Test.kt` name both mark.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const test = aKotlinTestFile();
 ```
 */
function aKotlinTestFile(): string {
  return 'package/linter/kotlin/src/test/kotlin/dev/monochromatic/detekt/RequireKDocTest.kt';
}

/**
 The real Rust test module this repository tracks as `tests.rs`, which holds `#[test]` tests in a file of its own name.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const test = aRustTestModule();
 ```
 */
function aRustTestModule(): string {
  return 'package/ide/app/src/native/tests.rs';
}

/**
 A test the change removed, which the reviewed commit no longer holds.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const test = aRemovedTestFile();
 ```
 */
function aRemovedTestFile(): string {
  return `${aSourceFolder()}/deleted.unit.test.ts`;
}

/**
 The recorded answer the review's second finding named, which sits in a `fixture` folder beside its test.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const answer = aRecordedAnswerFile();
 ```
 */
function aRecordedAnswerFile(): string {
  return 'package/crate-release/cli/src/fixture/run-still-going.json';
}

/**
 The shim package's own CommonJS source and the type file beside it, which a review reads because both extensions are code: the
 change of `7ec2fd422d` held these, their test and their note, and was answered "nothing to review" before the extensions were
 measured.

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
 The note beside the code, which a review reads no part of.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const note = aNoteFile();
 ```
 */
function aNoteFile(): string {
  return `${aSourceFolder()}/README.md`;
}

/**
 The source files of extensions in neither list of the files a review reads, which a review reads as code rather than drop.

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
 The rules file the brief names first.

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-briefs-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 A stand-in git whose range changes exactly the paths given, which the reviewed commit still holds.

 @param paths - the paths the change touched, each from the top of the checkout

 @returns a git that answers the three questions one reading asks

 @example
 ```ts
 const git = gitTouching({ paths: [aTestFile()], },);
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
    return {
      status: 0,
      stdout: (words[0] === 'ls-tree') ? heldListing : ((words[1] === '--name-only') ? listed : 'a diff of them'),
      stderr: '',
    };
  };
}

/**
 Runs the writer for one change, in a folder of its own.

 @param paths - the paths the change touched, each from the top of the checkout

 @returns what the writer answered, the reviews folder it wrote into, and the folder to throw away

 @example
 ```ts
 await using written = await writeAChangeThrough([aTestFile()]);
 ```
 */
async function writeAChangeThrough(
  paths: readonly string[],
  removed: readonly string[] = [],
): Promise<{
  readonly briefed: BriefOfOneReview
  readonly reviewsFolder: string
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder this test throws away.
   */
  const folder = await makeFolder();
  /**
   The folder one review of the change keeps its own files in.
   */
  const reviewsFolder = join(folder.path, 'reviews',);
  /**
   What the writer answered.
   */
  const briefed = await writeReviewBrief({
    git: gitTouching({ paths, removed, },),
    checkoutFolder: '/repo',
    review: aReview(),
    commit: aCommit(),
    ranges: [aRange(),],
    source: aSourceFolder(),
    focus: FOCUS_OF_A_CHANGE_REVIEW,
    reviewsFolder,
    runsFolder: join(folder.path, 'runs',),
    rulesFile: aRulesFile(),
  },);
  return {
    briefed,
    reviewsFolder,
    [Symbol.asyncDispose]: async function dispose() {
      await folder[Symbol.asyncDispose]();
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: writeReviewBrief.name,
      children: [
        it({
          name: 'writes a brief for a change of tests and fixtures alone, listing each changed test and fixture and asking for a break planted under each test, which is what makes such a change reviewable at all',
          fn: async () => {
            await using written = await writeAChangeThrough([aTestFile(), aFixtureFile(),],);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(written.briefed.written,).toBe(true,);
            expect(written.briefed.line,).toContain('2 files to read',);
            expect(brief,).toContain(`Review a change to the tests under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain('the change is to them alone',);
            expect(brief,).toContain(`- \`${aTestFile()}\``,);
            expect(brief,).toContain(`- \`${aFixtureFile()}\``,);
            expect(brief,).toContain('For each changed test above, plant a break under it',);
            expect(brief,).toContain('run that test by name, and say whether it fails',);
            expect(brief,).toContain('a test that still passes with the break in is a finding',);
            expect(brief,).toContain('Say what you read and what you ran',);
            expect(brief,).toContain('A review that ends with no file is lost',);
          },
        },),
        it({
          name: 'keeps the record of what the review read for a change of tests alone, so a fix that answers it starts at the commit it read',
          fn: async () => {
            await using written = await writeAChangeThrough([aTestFile(),],);
            /**
             What the writer recorded the review read.
             */
            const reading = JSON.parse(await readFile(
              readingFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            ),) as { readonly commit?: unknown };
            expect(reading.commit,).toBe(aCommit(),);
          },
        },),
        it({
          name: 'writes the brief of a change to tests when the range holds code beside the test, naming both and asking for a break and for the code requirements, because a changed test is read whatever else the change holds',
          fn: async () => {
            await using written = await writeAChangeThrough([aCodeFile(), aTestFile(),],);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(written.briefed.written,).toBe(true,);
            expect(brief,).toContain('a change to the tests under',);
            expect(brief,).toContain(`- \`${aCodeFile()}\``,);
            expect(brief,).toContain(`- \`${aTestFile()}\``,);
            expect(brief,).toContain('plant a break',);
            expect(brief,).toContain('The code must meet these requirements:',);
          },
        },),
        it({
          name: 'refuses a range that changes nothing under the source folder, in words that say the ranges change no file there and never say tests, and writes no brief',
          fn: async () => {
            await using written = await writeAChangeThrough([],);
            expect(written.briefed,).toEqual({
              written: false,
              line: `nothing to review under ${aSourceFolder()} in ${aRange()}: it changes no file under ${aSourceFolder()}`,
            },);
            expect(written.briefed.line.toLowerCase(),).not.toContain('test',);
            /**
             Where the review would keep its brief.
             */
            const briefFile = briefFileOfAReview({
              reviewsFolder: written.reviewsFolder,
              review: aReview(),
            },);
            expect(existsSync(briefFile,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a change of a note alone, in words that name the note and say what a review does read, so a refusal no longer reads as an empty review',
          fn: async () => {
            await using written = await writeAChangeThrough([aNoteFile(),],);
            expect(written.briefed.written,).toBe(false,);
            expect(written.briefed.line,).toBe(
              `nothing to review under ${aSourceFolder()} in ${aRange()}: a review reads the code of a change, the files that `
                + `decide its build, or its tests and fixtures alone, and it changes \`${aNoteFile()}\``,
            );
          },
        },),
        it({
          name: 'writes the brief of a change to tests when a note changed beside the test, naming the note as left out with the reason, so a line of prose never takes the review away',
          fn: async () => {
            await using written = await writeAChangeThrough([aNoteFile(), aTestFile(),],);
            expect(written.briefed.written,).toBe(true,);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(brief,).toContain(`- \`${aTestFile()}\``,);
            expect(brief,).toContain(`\`${aNoteFile()}\` is left out: a note, which no test reads.`,);
            // The note is named once, in the left-out paragraph, and never in the list of files to read.
            expect(brief.split(aNoteFile(),).length - 1,).toBe(1,);
            expect(brief,).toContain('plant a break',);
          },
        },),
        it({
          name: 'writes the brief of a change to tests for the shim\'s `.cjs` source and its `.d.cts` types beside their test, naming the code and the test and leaving the note out by name, which that range was once answered "nothing to review" for',
          fn: async () => {
            await using written = await writeAChangeThrough([...shimSourceAndItsTypes(), aTestFile(), aNoteFile(),],);
            expect(written.briefed.written,).toBe(true,);
            expect(written.briefed.line,).toContain('3 files to read',);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            for (const file of [...shimSourceAndItsTypes(), aTestFile(),]) {
              expect(brief,).toContain(`- \`${file}\``,);
            }
            expect(brief,).toContain(`\`${aNoteFile()}\` is left out: a note, which no test reads.`,);
            expect(written.briefed.line,).not.toContain('no list of this writer',);
          },
        },),
        it({
          name: 'reads and names the files of a kind no list of its own names, saying in the line it prints that they are read as code and where their extensions belong, so no kind is dropped silently',
          fn: async () => {
            await using written = await writeAChangeThrough(filesOfAnUnlistedKind(),);
            expect(written.briefed.written,).toBe(true,);
            expect(written.briefed.line,).toContain('2 files to read',);
            expect(written.briefed.line,).toContain(
              `are read as code: ${filesOfAnUnlistedKind().map(function inBackticks(path,) {
                return `\`${path}\``;
              },)
                .join(', ',)}`,
            );
            expect(written.briefed.line,).toContain('src/which-files-a-review-reads.ts',);
            /**
             The brief the writer wrote, which names those files among the files to read.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            for (const file of filesOfAnUnlistedKind()) {
              expect(brief,).toContain(`- \`${file}\``,);
            }
          },
        },),
        it({
          name: 'asks for a break under a real Rust file named tests.rs, because a test module named for what it holds is a test and not the code beside it',
          fn: async () => {
            await using written = await writeAChangeThrough([aRustTestModule(),],);
            expect(written.briefed.written,).toBe(true,);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(brief,).toContain('a change to the tests under',);
            expect(brief,).toContain(`- \`${aRustTestModule()}\``,);
            expect(brief,).toContain('For each changed test above, plant a break under it',);
          },
        },),
        it({
          name: 'asks of a test the change removed only that its text be read in the diff, and never that a break be planted or a file run that the reviewed commit no longer holds; a test the commit holds is still asked for both',
          fn: async () => {
            /**
             A writer for a change whose only test was removed.
             */
            await using removed = await writeAChangeThrough([aRemovedTestFile(),], [aRemovedTestFile(),],);
            /**
             The brief of that change.
             */
            const removedBrief = await readFile(
              briefFileOfAReview({
                reviewsFolder: removed.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(removedBrief,).toContain(`- \`${aRemovedTestFile()}\``,);
            expect(removedBrief,).toContain('the change diff is where they are read',);
            expect([removedBrief.includes('plant a break under it',),],).toEqual([false,],);
            expect([removedBrief.includes('run that test by name',),],).toEqual([false,],);
            /**
             The control: the same test held by the commit, whose brief asks for the break.
             */
            await using held = await writeAChangeThrough([aRemovedTestFile(),],);
            /**
             The brief of the control.
             */
            const heldBrief = await readFile(
              briefFileOfAReview({
                reviewsFolder: held.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(heldBrief,).toContain('For each changed test above, plant a break under it',);
          },
        },),
        it({
          name: 'writes the brief of a change to tests for a Kotlin test alone, asking for a break planted under it, which a change under a `src/test` layout was answered as a change of code before the layout and the name were counted',
          fn: async () => {
            await using written = await writeAChangeThrough([aKotlinTestFile(),],);
            expect(written.briefed.written,).toBe(true,);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(brief,).toContain('a change to the tests under',);
            expect(brief,).toContain(`- \`${aKotlinTestFile()}\``,);
            expect(brief,).toContain('plant a break under it',);
          },
        },),
        it({
          name: 'writes the brief of a change to tests for a test and the recorded answer in its `fixture` folder, naming both, which was answered as nothing to review before the folder was counted',
          fn: async () => {
            await using written = await writeAChangeThrough([aTestFile(), aRecordedAnswerFile(),],);
            expect(written.briefed.written,).toBe(true,);
            expect(written.briefed.line,).toContain('2 files to read',);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(brief,).toContain(`- \`${aTestFile()}\``,);
            expect(brief,).toContain(`- \`${aRecordedAnswerFile()}\``,);
            expect(brief,).toContain('For a changed fixture, name the tests that use it',);
          },
        },),
      ],
    },),
  ],
},);
