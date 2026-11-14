/**
 The writer of review briefs against disposable folders: a change of tests and fixtures alone gets a brief that lists each
 changed test and asks for a break planted under it, a change of code is described as it always was, a range that deletes or renames
 code is a change of code whose brief names what was removed, and a range that changes nothing a review reads is refused with words
 that say so.

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
 The name a rename gives the code file.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const renamed = aRenamedCodeFile();
 ```
 */
function aRenamedCodeFile(): string {
  return `${aSourceFolder()}/gate2.rs`;
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
 A second test file, which a change to tests alone may delete.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const test = aDeletedTestFile();
 ```
 */
function aDeletedTestFile(): string {
  return `${aSourceFolder()}/old_tests.rs`;
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
 The shim package's own source and type file, whose language a review does not read.

 @returns their paths, from the top of the checkout

 @example
 ```ts
 const source = sourceFilesNoReviewReads();
 ```
 */
function sourceFilesNoReviewReads(): readonly string[] {
  return [
    `${aSourceFolder()}/index.cjs`,
    `${aSourceFolder()}/index.d.cts`,
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
 One path a stand-in git says a range changed: the status Git reported for it, and the two names a rename carries.

 This is what the writer reads from `git diff --name-status -z`: a status, the path, and, for a rename, the path it came from and
 the path it became.
 */
type ChangedInATest = {
  /**
   The status, as Git prints it, score included: `M`, `A`, `D`, `R100`, `T` or `C100`.
   */
  readonly status: string;

  /**
   The path in the newer side of the range, from the top of the checkout.
   */
  readonly path: string;

  /**
   The path in the older side of the range, which a rename also names.
   */
  readonly from?: string;
};

/**
 A modified path, which is Git's `M`.

 @param path - the path, from the top of the checkout

 @returns the change Git reports for it

 @example
 ```ts
 const changed = aModifiedFile(aCodeFile());
 ```
 */
function aModifiedFile(path: string,): ChangedInATest {
  return {
    status: 'M',
    path,
  };
}

/**
 A deleted path, which is Git's `D`, and which leaves no file at the commit for a reviewer to read.

 @param path - the path, from the top of the checkout

 @returns the change Git reports for it

 @example
 ```ts
 const changed = aDeletedFile(aCodeFile());
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
 const changed = aRenamedFile({ was: aCodeFile(), now: aRenamedCodeFile() },);
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
 The names the reviewed commit holds for a set of changes: the newer name of every path that was added, copied, modified, renamed
 or changed the type of, which is what `git ls-tree` prints for a commit that holds them.

 @param changed - the changes a stand-in git reports

 @returns the paths the commit holds, one to a line

 @example
 ```ts
 const held = heldPathsIn({ changed: [aModifiedFile(aCodeFile())], },);
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
 What `git diff --name-status -z` prints for a set of changes: the status, the path, and for a rename both names, each field ended
 by a NUL byte.

 @param changed - the changes a stand-in git reports

 @returns the text git prints

 @example
 ```ts
 const printed = nameStatusPrintedFor({ changed: [aDeletedFile(aCodeFile())], },);
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

 It answers the questions the writer asks now, and also the `--name-only` question the writer asked before it read statuses, so a
 control still says what the old reading answered when the fix is taken out.

 @param changed - the changes the range made

 @returns a git that answers the three questions one reading asks

 @example
 ```ts
 const git = gitChanging({ changed: [aDeletedFile(aCodeFile()), aModifiedFile(aTestFile())], },);
 ```
 */
function gitChanging({ changed, }: { readonly changed: readonly ChangedInATest[] },): (
  request: { readonly words: readonly string[] },
) => Promise<CommandOutput> {
  return async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
    if (words[0] === 'ls-tree') {
      return {
        status: 0,
        stdout: heldPathsIn({ changed, },),
        stderr: '',
      };
    }
    if (words.includes('--name-status',)) {
      return {
        status: 0,
        stdout: nameStatusPrintedFor({ changed, },),
        stderr: '',
      };
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
      return {
        status: 0,
        stdout: (kept.length === 0) ? '' : `${kept.map(function newerName(one,) {
          return one.path;
        },)
          .join('\n',)}\n`,
        stderr: '',
      };
    }
    return {
      status: 0,
      stdout: 'a diff of them',
      stderr: '',
    };
  };
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
 Runs the writer for one change, in a folder of its own.

 @param changed - the changes the range made

 @returns what the writer answered, the reviews folder it wrote into, and the folder to throw away

 @example
 ```ts
 await using written = await writeAChangeThrough([aModifiedFile(aTestFile())]);
 ```
 */
async function writeAChangeThrough(changed: readonly ChangedInATest[],): Promise<{
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
    git: gitChanging({ changed, },),
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

/**
 Reads the brief the writer wrote for one change.

 @param reviewsFolder - the folder the writer wrote into

 @returns the brief's text

 @example
 ```ts
 const brief = await theBriefIn(written.reviewsFolder);
 ```
 */
async function theBriefIn(reviewsFolder: string,): Promise<string> {
  return await readFile(briefFileOfAReview({
    reviewsFolder,
    review: aReview(),
  },), 'utf8',);
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
            await using written = await writeAChangeThrough([aModifiedFile(aTestFile(),), aModifiedFile(aFixtureFile(),),],);
            /**
             The brief the writer wrote.
             */
            const brief = await theBriefIn(written.reviewsFolder,);
            expect(written.briefed.written,).toBe(true,);
            expect(written.briefed.line,).toContain('2 files to read',);
            expect(brief,).toContain(`Review a change to the tests under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain('The change is to tests and the fixtures tests use, and to nothing else.',);
            expect(brief,).toContain(`- \`${aTestFile()}\``,);
            expect(brief,).toContain(`- \`${aFixtureFile()}\``,);
            expect(brief,).toContain('For each changed test the commit still holds, plant a break under it',);
            expect(brief,).toContain('run that test by name, and say whether it fails',);
            expect(brief,).toContain('a test that still passes with the break in is a finding',);
            expect(brief,).toContain('Say what you read and what you ran',);
            expect(brief,).toContain('A review that ends with no file is lost',);
          },
        },),
        it({
          name: 'keeps the record of what the review read for a change of tests alone, so a fix that answers it starts at the commit it read',
          fn: async () => {
            await using written = await writeAChangeThrough([aModifiedFile(aTestFile(),),],);
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
          name: 'writes the brief of a change of code as it always did, naming the code and asking for no break, even when the range changed a test beside it',
          fn: async () => {
            await using written = await writeAChangeThrough([aModifiedFile(aCodeFile(),), aModifiedFile(aTestFile(),),],);
            /**
             The brief the writer wrote.
             */
            const brief = await theBriefIn(written.reviewsFolder,);
            expect(written.briefed.written,).toBe(true,);
            expect(brief,).toContain(`Review a change under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain(`- \`${aCodeFile()}\``,);
            expect(brief,).not.toContain(`- \`${aTestFile()}\``,);
            expect(brief,).not.toContain('plant a break',);
          },
        },),
        it({
          name: 'writes the brief of a change of code when the range deleted the code and edited its test, naming what was deleted, asking what depended on it, and asking for no break, because the change is not tests alone',
          fn: async () => {
            await using written = await writeAChangeThrough([aDeletedFile(aCodeFile(),), aModifiedFile(aTestFile(),),],);
            /**
             The brief the writer wrote.
             */
            const brief = await theBriefIn(written.reviewsFolder,);
            expect(written.briefed.written,).toBe(true,);
            expect(written.briefed.line,).toContain('0 files to read',);
            expect(brief,).toContain(`Review a change under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain('The reviewed commit holds no file at a path this change touches',);
            expect(brief,).toContain(`- \`${aCodeFile()}\` (deleted)`,);
            expect(brief,).toContain('Say what depended on each of them, and what does its work now or why nothing does.',);
            expect(brief,).not.toContain('the change is to tests',);
            expect(brief,).not.toContain('plant a break',);
          },
        },),
        it({
          name: 'writes the brief of a change of code when the range renamed the code and edited its test, naming both names and listing the file the commit holds, because the rename removed the old one',
          fn: async () => {
            await using written = await writeAChangeThrough([
              aRenamedFile({
                was: aCodeFile(),
                now: aRenamedCodeFile(),
              },),
              aModifiedFile(aTestFile(),),
            ],);
            /**
             The brief the writer wrote.
             */
            const brief = await theBriefIn(written.reviewsFolder,);
            expect(written.briefed.written,).toBe(true,);
            expect(brief,).toContain(`Review a change under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain(`- \`${aRenamedCodeFile()}\``,);
            expect(brief,).toContain(`- \`${aCodeFile()}\` (renamed to \`${aRenamedCodeFile()}\`)`,);
            expect(brief,).toContain('Say what depended on each of them, and what does its work now or why nothing does.',);
            expect(brief,).not.toContain('the change is to tests',);
            expect(brief,).not.toContain('plant a break',);
          },
        },),
        it({
          name: 'names a test the range deleted in the brief of a change to tests alone, so a review sees the coverage that left, while the breaks stay with the tests the commit holds',
          fn: async () => {
            await using written = await writeAChangeThrough([aDeletedFile(aDeletedTestFile(),), aModifiedFile(aTestFile(),),],);
            /**
             The brief the writer wrote.
             */
            const brief = await theBriefIn(written.reviewsFolder,);
            expect(written.briefed.written,).toBe(true,);
            expect(brief,).toContain(`Review a change to the tests under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain(`- \`${aTestFile()}\``,);
            expect(brief,).toContain(`- \`${aDeletedTestFile()}\` (deleted)`,);
            expect(brief,).toContain('Say what depended on each of them',);
            expect(brief,).toContain('For each changed test the commit still holds, plant a break under it',);
          },
        },),
        it({
          name: 'refuses a range that changes nothing under the source folder, in words that say so and never say tests, and writes no brief',
          fn: async () => {
            await using written = await writeAChangeThrough([],);
            expect(written.briefed,).toEqual({
              written: false,
              line: `nothing to review under ${aSourceFolder()} in ${aRange()}`,
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
          name: 'refuses a change of a source file whose language a review does not read beside its test, because a brief of the test alone would hide the change itself',
          fn: async () => {
            await using written = await writeAChangeThrough([
              ...sourceFilesNoReviewReads()
                .map(function asAModifiedFile(path,) {
                  return aModifiedFile(path,);
                },),
              aModifiedFile(aTestFile(),),
            ],);
            expect(written.briefed,).toEqual({
              written: false,
              line: `nothing to review under ${aSourceFolder()} in ${aRange()}`,
            },);
          },
        },),
      ],
    },),
  ],
},);
