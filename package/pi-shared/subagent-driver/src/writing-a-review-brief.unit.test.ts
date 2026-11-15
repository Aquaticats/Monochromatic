/**
 The writer of review briefs against disposable folders: a change of tests and fixtures alone gets a brief that lists each
 changed test and asks for a break planted under it, a change of code is described as it always was, a range that deletes or renames
 code is a change of code whose brief names what was removed, and a range that changes nothing a review reads is refused with words
 that say so.

 Every git answer here is handed in by the test, so no checkout or worktree is involved, and every folder is one the test made and
 throws away. Nothing starts a run.

 @module
 */

import { spawn, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
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
  ReviewFolderTakenError,
  briefFileOfAReview,
  ownerFileOfAReview,
  readingFileOfAReview,
  writeReviewBrief,
} from '../dist/final/node/index.mjs';

/**
 A manager that says no review of any unit runs, which a test that starts no unit hands the writer.
 */
async function anIdleManager(): Promise<CommandOutput> {
  return {
    status: 3,
    stdout: 'inactive\n',
    stderr: '',
  };
}

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
function heldPathsIn({ changed, }: { readonly changed: readonly ChangedInATest[] },): readonly string[] {
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
  return changed.filter(function leavesAFile(one,) {
    return leavingAFile.has(one.status.charAt(0,),);
  },)
    .map(function newerName(one,) {
      return one.path;
    },);
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
      /**
       The paths the commit holds, in the form the question asked for: raw fields ended by NUL bytes when it asked for `-z`, or
       one to a line otherwise. Every path a test gives this stand-in is plain ASCII, so Git's quoting prints it as itself.
       */
      const paths = heldPathsIn({ changed, },);
      return {
        status: 0,
        stdout: (words.includes('-z',))
          ? paths.map(function asAField(path,) {
            return `${path}\0`;
          },)
            .join('',)
          : ((paths.length === 0) ? '' : `${paths.join('\n',)}\n`),
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
 The branch a change of this file is reviewed as.

 @returns the branch's name

 @example
 ```ts
 const branch = aBranch();
 ```
 */
function aBranch(): string {
  return 'fix/a-change-to-tests';
}

/**
 One start's identity, read the way the writer reads it.

 @returns this process's number, boot and start time

 @example
 ```ts
 const identity = await thisProcessIdentity();
 ```
 */
async function thisProcessIdentity(): Promise<{
  readonly pid: number;
  readonly bootId: string;
  readonly startTicks: string;
}> {
  /**
   What the kernel says of this process.
   */
  const stat = await readFile('/proc/self/stat', 'utf8',);
  /**
   Where the program's name, which may hold spaces, ends.
   */
  const close = stat.lastIndexOf(')',);
  /**
   The fields after the name, the state first.
   */
  const fields = stat.slice(close + 1,)
    .trim()
    .split(' ',);
  return {
    pid: process.pid,
    bootId: (await readFile('/proc/sys/kernel/random/boot_id', 'utf8',)).trim(),
    startTicks: fields[19] ?? '',
  };
}

/**
 Writes an owner record into a review folder, as a start that once took it left it.

 @param reviewsFolder - folder that holds one folder per review

 @param branch - the branch the record names

 @param process - the identity the record names

 @returns the owner file's path

 @example
 ```ts
 const ownerFile = await writeOwnerRecord({ reviewsFolder, branch: aBranch(), process: aDeadStart },);
 ```
 */
async function writeOwnerRecord({
  reviewsFolder,
  branch,
  process: identity,
}: {
  readonly reviewsFolder: string;
  readonly branch: string;
  readonly process: {
    readonly pid: number;
    readonly bootId: string;
    readonly startTicks: string;
  };
},): Promise<string> {
  /**
   The file the record is kept in.
   */
  const ownerFile = ownerFileOfAReview({
    reviewsFolder,
    review: aReview(),
  },);
  await mkdir(
    dirname(ownerFile,),
    { recursive: true, },
  );
  await writeFile(
    ownerFile,
    `${JSON.stringify({
      branch,
      commit: aCommit(),
      ranges: [aRange(),],
      source: aSourceFolder(),
      claimedAt: '2026-11-14T00:00:00.000Z',
      process: identity,
    },)}
`,
    'utf8',
  );
  return ownerFile;
}

/**
 The identity of a start that ended, which no kernel entry answers for.

 @returns the identity

 @example
 ```ts
 const dead = aStartThatEnded();
 ```
 */
function aStartThatEnded(): {
  readonly pid: number;
  readonly bootId: string;
  readonly startTicks: string;
} {
  return {
    pid: 999_999,
    bootId: 'a-boot-this-machine-is-not-on',
    startTicks: '0',
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
    manager: anIdleManager,
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
 Runs the writer for one change of one branch, in a folder the test gives.

 @param changed - the changes the range made

 @param folder - the test's folder, which holds the reviews and runs folders

 @param branch - the branch the change is reviewed as

 @param commit - the commit under review

 @returns what the writer answered

 @example
 ```ts
 const briefed = await writeAChangeOfBranch({ changed: [aModifiedFile(aCodeFile())], folder: folder.path },);
 ```
 */
async function writeAChangeOfBranch({
  changed,
  folder,
  branch = aBranch(),
  commit = aCommit(),
}: {
  readonly changed: readonly ChangedInATest[];
  readonly folder: string;
  readonly branch?: string;
  readonly commit?: string;
}): Promise<BriefOfOneReview> {
  return await writeReviewBrief({
    git: gitChanging({ changed, },),
    checkoutFolder: '/repo',
    review: aReview(),
    commit,
    ranges: [aRange(),],
    source: aSourceFolder(),
    branch,
    focus: FOCUS_OF_A_CHANGE_REVIEW,
    reviewsFolder: join(folder, 'reviews',),
    runsFolder: join(folder, 'runs',),
    rulesFile: aRulesFile(),
    manager: anIdleManager,
  },);
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
          name: 'writes the brief of a change to tests alone whose file is named like a placeholder, listing it as that name, because only the template is read',
          fn: async () => {
            /**
             A test file whose name is shaped like a placeholder.
             */
            const named = `${aSourceFolder()}/@BREAK@.test.ts`;
            await using written = await writeAChangeThrough([aModifiedFile(named,),],);
            /**
             The brief the writer wrote.
             */
            const brief = await theBriefIn(written.reviewsFolder,);
            expect(written.briefed.written,).toBe(true,);
            expect(brief,).toContain(`- \`${named}\``,);
            expect(brief,).toContain('plant a break under it',);
          },
        },),
        it({
          name: 'writes the brief of a change that deleted a code file named like a placeholder, naming it as that name, because only the template is read',
          fn: async () => {
            /**
             A code file whose name is shaped like a placeholder.
             */
            const named = `${aSourceFolder()}/@BREAK@.ts`;
            await using written = await writeAChangeThrough([aModifiedFile(aCodeFile(),), aDeletedFile(named,),],);
            /**
             The brief the writer wrote.
             */
            const brief = await theBriefIn(written.reviewsFolder,);
            expect(written.briefed.written,).toBe(true,);
            expect(brief,).toContain(`- \`${aCodeFile()}\``,);
            expect(brief,).toContain(`- \`${named}\` (deleted)`,);
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
          name: 'takes no review folder for a change that leaves nothing to read, so a branch with nothing to review stands in no one\'s way',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What the writer answered for a change with nothing to read.
             */
            const nothing = await writeAChangeOfBranch({
              changed: [],
              folder: folder.path,
            },);
            expect(nothing.written,).toBe(false,);
            /**
             The folder the review would keep its files in.
             */
            const reviewFolder = join(folder.path, 'reviews', aReview(),);
            expect(existsSync(reviewFolder,),).toBe(false,);
            /**
             What the writer answered for the next change of the same review, which has code to read.
             */
            const briefed = await writeAChangeOfBranch({
              changed: [aModifiedFile(aCodeFile(),),],
              folder: folder.path,
            },);
            expect(briefed.written,).toBe(true,);
            /**
             The brief the writer wrote for the second change.
             */
            const briefFile = join(reviewFolder, 'brief.md',);
            expect(existsSync(briefFile,),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a second branch of one change name before it writes over the first, naming both branches, when both starters run at once',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The two distinct branches whose names derive one change name.
             */
            const firstBranch = 'fix/shared-change';
            const secondBranch = 'feat/shared-change';
            /**
             What each concurrent start answered.
             */
            const answers = await Promise.allSettled([
              writeAChangeOfBranch({
                changed: [aModifiedFile(aCodeFile(),),],
                folder: folder.path,
                branch: firstBranch,
              },),
              writeAChangeOfBranch({
                changed: [aModifiedFile(aTestFile(),),],
                folder: folder.path,
                branch: secondBranch,
              },),
            ],);
            /**
             The starts that wrote their brief.
             */
            const wrote = answers.filter(function wroteItsBrief(answer,) {
              return answer.status === 'fulfilled';
            },);
            /**
             The starts that were refused.
             */
            const refused = answers.filter(function wasRefused(answer,) {
              return answer.status === 'rejected';
            },);
            expect(wrote,).toHaveLength(1,);
            expect(refused,).toHaveLength(1,);
            /**
             What the refused start threw.
             */
            const reason: unknown = (refused[0]?.status === 'rejected') ? refused[0].reason : undefined;
            expect(reason,).toBeInstanceOf(ReviewFolderTakenError,);
            expect(String(reason,),).toContain(firstBranch,);
            expect(String(reason,),).toContain(secondBranch,);
            /**
             What the folder records of the start that took it.
             */
            const owner = JSON.parse(await readFile(
              ownerFileOfAReview({
                reviewsFolder: join(folder.path, 'reviews',),
                review: aReview(),
              },),
              'utf8',
            ),) as { readonly branch?: unknown };
            expect(owner.branch,).toBe(((answers[0]?.status === 'fulfilled') ? firstBranch : secondBranch));
          },
        },),
        it({
          name: 'takes a review folder a start that ended held for the same branch, so a start killed between taking a folder and writing its brief blocks nobody for good',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The reviews folder the dead start wrote into.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             The owner file the dead start left.
             */
            const ownerFile = await writeOwnerRecord({
              reviewsFolder,
              branch: aBranch(),
              process: aStartThatEnded(),
            },);
            /**
             What this start answered.
             */
            const briefed = await writeAChangeOfBranch({
              changed: [aModifiedFile(aCodeFile(),),],
              folder: folder.path,
            },);
            expect(briefed.written,).toBe(true,);
            /**
             What the folder now records.
             */
            const owner = JSON.parse(await readFile(ownerFile, 'utf8',),) as {
              readonly branch?: unknown;
              readonly process?: { readonly pid?: unknown };
            };
            expect(owner.branch,).toBe(aBranch(),);
            expect(owner.process?.pid,).toBe(process.pid,);
            /**
             The brief the writer wrote.
             */
            const briefFile = join(reviewsFolder, aReview(), 'brief.md',);
            expect(existsSync(briefFile,),).toBe(true,);
          },
        },),
        it({
          name: 'takes a review folder a starter left when it was killed after taking it, so a killed starter blocks nobody for good',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The reviews folder the killed start wrote into.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             The owner file the killed start leaves.
             */
            const ownerFile = ownerFileOfAReview({ reviewsFolder, review: aReview(), },);
            /**
             The start that takes the folder and waits to be killed, writing the record of the process it is.
             */
            const childFile = join(folder.path, 'a-start-that-takes-the-folder.ts',);
            await writeFile(childFile, `
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
/**
 The kernel's answer about this process, whose fields after the name begin at its state.
 */
const stat = readFileSync('/proc/self/stat', 'utf8',);
const fields = stat.slice(stat.lastIndexOf(')') + 2,).split(' ',);
/**
 The moment this process began, which tells it from a later process of the same number.
 */
const startTicks = fields[19];
/**
 The boot this process runs on.
 */
const bootId = readFileSync('/proc/sys/kernel/random/boot_id', 'utf8',).trim();
mkdirSync(dirname(process.argv[3],), { recursive: true, },);
writeFileSync(process.argv[3], JSON.stringify({
  branch: process.argv[2],
  commit: 'abc1234',
  ranges: ['main...feat/one-change'],
  source: 'package/a-package',
  claimedAt: new Date().toISOString(),
  process: { pid: process.pid, bootId, startTicks },
}, null, 2,) + '\\n',);
process.stdout.write('took the folder\\n',);
setInterval(() => {}, 1000,);
`,);
            /**
             The start that took the folder, killed when this test ends, however it ends.
             */
            using started = {
              child: spawn(process.execPath, [childFile, aBranch(), ownerFile,], {
                stdio: ['ignore', 'pipe', 'pipe',],
              },),
              [Symbol.dispose]: function endTheStart() {
                this.child.kill('SIGKILL',);
              },
            };
            /**
             The start the test watches.
             */
            const { child, } = started;
            /**
             What the start said on its error stream, for a start that ends before it takes the folder.
             */
            let childErrors = '';
            child.stderr.setEncoding('utf8',);
            child.stderr.on('data', function saidOnTheErrorStream(chunk: string,) {
              childErrors += chunk;
            },);
            await new Promise<void>(function waitForTheFolder(resolve, reject,) {
              child.stdout.setEncoding('utf8',);
              child.stdout.once('data', function tookTheFolder(chunk: string,) {
                if (chunk.includes('took the folder',)) {
                  resolve();
                }
              },);
              child.once('exit', function endedBeforeTaking(code,) {
                reject(new Error(`the start ended before it took the folder, with ${String(code,)}: ${childErrors}`,),);
              },);
            },);
            /**
             The number of the start that took the folder.
             */
            const killed = child.pid;
            expect(typeof killed,).toBe('number',);
            child.kill('SIGKILL',);
            await new Promise<void>(function waitForTheEnd(resolve,) {
              child.once('exit', function ended() {
                resolve();
              },);
            },);
            /**
             What the folder held before this start wrote its brief.
             */
            const before = JSON.parse(await readFile(ownerFile, 'utf8',),) as {
              readonly process?: { readonly pid?: unknown };
            };
            expect(before.process?.pid,).toBe(killed,);
            /**
             What this start answered.
             */
            const briefed = await writeAChangeOfBranch({
              changed: [aModifiedFile(aCodeFile(),),],
              folder: folder.path,
            },);
            expect(briefed.written,).toBe(true,);
            /**
             What the folder now records.
             */
            const owner = JSON.parse(await readFile(ownerFile, 'utf8',),) as {
              readonly branch?: unknown;
              readonly process?: { readonly pid?: unknown };
            };
            expect(owner.branch,).toBe(aBranch(),);
            expect(owner.process?.pid,).toBe(process.pid,);
            /**
             The brief the writer wrote.
             */
            const briefFile = join(reviewsFolder, aReview(), 'brief.md',);
            expect(existsSync(briefFile,),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a review folder a start that ended held for another branch, naming both branches, because it is another change\'s brief',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The branch the dead start wrote for.
             */
            const otherBranch = 'fix/another-change';
            /**
             The owner file the dead start left.
             */
            const ownerFile = await writeOwnerRecord({
              reviewsFolder: join(folder.path, 'reviews',),
              branch: otherBranch,
              process: aStartThatEnded(),
            },);
            /**
             What this start answered.
             */
            let caught: unknown;
            try {
              await writeAChangeOfBranch({
                changed: [aModifiedFile(aCodeFile(),),],
                folder: folder.path,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ReviewFolderTakenError,);
            expect(String(caught,),).toContain(otherBranch,);
            expect(String(caught,),).toContain(aBranch(),);
            /**
             The brief the other change keeps in the folder.
             */
            const briefFile = join(dirname(ownerFile,), 'brief.md',);
            expect(existsSync(briefFile,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a review folder a start that still runs holds, even when it is the same branch, so two starts never write one folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The owner file a start of this very process left.
             */
            const ownerFile = await writeOwnerRecord({
              reviewsFolder: join(folder.path, 'reviews',),
              branch: aBranch(),
              process: await thisProcessIdentity(),
            },);
            /**
             What this start answered.
             */
            let caught: unknown;
            try {
              await writeAChangeOfBranch({
                changed: [aModifiedFile(aCodeFile(),),],
                folder: folder.path,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ReviewFolderTakenError,);
            expect(String(caught,),).toContain('still running',);
            /**
             The brief the running start keeps in the folder.
             */
            const briefFile = join(dirname(ownerFile,), 'brief.md',);
            expect(existsSync(briefFile,),).toBe(false,);
          },
        },),
        it({
          name: 'writes a review folder of its own beside a folder a live start made, instead of refusing or taking it over',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The first start's brief, which the second must leave as it is.
             */
            const first = await writeAChangeOfBranch({
              changed: [aModifiedFile(aCodeFile(),),],
              folder: folder.path,
            },);
            expect(first.written,).toBe(true,);
            /**
             Where the reviews live.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            const firstBrief = join(reviewsFolder, aReview(), 'brief.md',);
            const firstDiff = join(reviewsFolder, aReview(), 'change.diff',);
            const firstBriefText = await readFile(firstBrief, 'utf8',);
            const firstDiffText = await readFile(firstDiff, 'utf8',);
            /**
             The second start of the same change.
             */
            const second = await writeAChangeOfBranch({
              changed: [aModifiedFile(aCodeFile(),),],
              folder: folder.path,
            },);
            expect(second.written,).toBe(true,);
            /**
             The folder the second start took for itself.
             */
            const secondFolder = join(reviewsFolder, `${aReview()}-2`,);
            expect(existsSync(join(secondFolder, 'brief.md',)),).toBe(true,);
            expect(await readFile(firstBrief, 'utf8',),).toBe(firstBriefText,);
            expect(await readFile(firstDiff, 'utf8',),).toBe(firstDiffText,);
            expect(second.line,).toContain(`${aReview()}-2`,);
          },
        },),
        it({
          name: 'leaves a folder a dead start left untouched and writes this review under a name of its own',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Where the reviews live.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             The folder and record a dead start left.
             */
            const ownerFile = await writeOwnerRecord({
              reviewsFolder,
              branch: aBranch(),
              process: aStartThatEnded(),
            },);
            const leftoverText = await readFile(ownerFile, 'utf8',);
            /**
             What this start answered.
             */
            const briefed = await writeAChangeOfBranch({
              changed: [aModifiedFile(aCodeFile(),),],
              folder: folder.path,
            },);
            expect(briefed.written,).toBe(true,);
            expect(await readFile(ownerFile, 'utf8',),).toBe(leftoverText,);
            expect(existsSync(join(dirname(ownerFile,), 'brief.md',)),).toBe(false,);
            expect(existsSync(join(reviewsFolder, `${aReview()}-2`, 'brief.md',)),).toBe(true,);
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
