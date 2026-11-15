/**
 The last message of a review that wrote no review file: read whole from the session file its newest finished run names, and
 written as that review's `review.md`.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { spawnSync, } from 'node:child_process';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  CLEAN_END_REASONS,
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  END_REASONS,
  SAVED_REVIEW_NOTE,
  lastAssistantTextIn,
  newestRunOf,
  runNameOfAReview,
  parseSaveReviewOptions,
  saveTheLastMessage,
} from '../dist/final/node/index.mjs';

/**
 The command that saves a review, as the package builds it, which is what a coordinator runs.
 */
const SAVE_REVIEW_COMMAND = fileURLToPath(new URL('../dist/final/node/save-the-final-message-of-a-review-that-wrote-no-review-file.mjs', import.meta.url,),);

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-message-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 One line of a Pi session file.

 @param role - the message's role

 @param texts - the text parts of its content

 @returns the line, as the session writes it

 @example
 ```ts
 const line = sessionLine('assistant', ['a finding']);
 ```
 */
function sessionLine(role: string, texts: readonly string[],): string {
  return JSON.stringify({
    type: 'message',
    message: {
      role,
      content: texts.map(function asPart(text,) {
        return {
          type: 'text',
          text,
        };
      },),
    },
  },);
}

/**
 The ranges every test here gives a review.

 @returns the ranges, as `git diff` takes them

 @example
 ```ts
 const ranges = theRangesGiven();
 ```
 */
function theRangesGiven(): readonly string[] {
  return ['main...fix/a',];
}

/**
 The files every test here gives a review, from which a stated count is derived.

 @returns the files a brief listed

 @example
 ```ts
 const files = theFilesGiven();
 ```
 */
function theFilesGiven(): readonly string[] {
  return ['package/a/src/gate.rs', 'package/a/src/gate_second.rs',];
}

/**
 Writes the record of what a review was given, as the brief writer leaves it.

 @param reviewFolder - folder that holds one folder per review

 @param review - the review's name

 @param files - the files the brief listed, or nothing for a record written before a brief listed them

 @returns nothing

 @example
 ```ts
 await recordWhatTheReviewWasGiven({ reviewFolder: '/reviews', review: 'a-round-1', files: theFilesGiven(), },);
 ```
 */
async function recordWhatTheReviewWasGiven({
  reviewFolder,
  review,
  files,
}: {
  readonly reviewFolder: string;
  readonly review: string;
  readonly files?: readonly string[];
},): Promise<void> {
  await mkdir(join(reviewFolder, review,), { recursive: true, },);
  await writeFile(
    join(reviewFolder, review, 'what-the-review-read.json',),
    `${JSON.stringify({
      review,
      commit: 'abc123',
      ranges: theRangesGiven(),
      source: 'package/a/src',
      ...((files === undefined) ? {} : { files, }),
    }, undefined, 2)}\n`,
  );
}

/**
 The text of a review's own file, with the two lines its brief asks for at its top.

 @param ranges - the ranges the review says it read

 @param filesRead - how many of the listed files it says it read

 @param filesListed - how many files it says the brief listed

 @returns the review's text

 @example
 ```ts
 const text = reviewOfAChange({ ranges: theRangesGiven(), filesRead: 2, filesListed: 2, },);
 ```
 */
function reviewOfAChange({
  ranges,
  filesRead,
  filesListed,
}: {
  readonly ranges: readonly string[];
  readonly filesRead: number;
  readonly filesListed: number;
},): string {
  return `Not final.\n\nRanges reviewed: ${ranges.map(function inBackticks(range,) {
    return `\`${range}\``;
  },)
    .join(', ',)}\nFiles read in full: ${String(filesRead,)} of ${String(filesListed,)}\n\nNo concrete defects found.\n`;
}


/**
 Runs the command that saves a review over one review folder, as a coordinator runs it.

 @param reviewFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param review - the review's name

 @returns the status the command ended with, and its standard output

 @example
 ```ts
 const ended = runSaveReviewCommand({ reviewFolder: '/reviews', runsFolder: '/runs', review: 'a-round-1', },);
 ```
 */
function runSaveReviewCommand({
  reviewFolder,
  runsFolder,
  review,
}: {
  readonly reviewFolder: string;
  readonly runsFolder: string;
  readonly review: string;
},): {
  readonly status: number;
  readonly stdout: string;
} {
  /**
   How the command ended, which is its own status and not this test's.
   */
  const ended = spawnSync(
    'node',
    [
      SAVE_REVIEW_COMMAND,
      `--review=${review}`,
      `--review-folder=${reviewFolder}`,
      `--runs-folder=${runsFolder}`,
    ],
    { encoding: 'utf8', },
  );
  // A command ended by a signal, or one node could not start, is the failure of the test itself rather than an answer.
  if (ended.status === null) {
    throw new Error(`the command that saves a review did not end with a status: ${ended.stderr}`,);
  }
  return {
    status: ended.status,
    stdout: ended.stdout,
  };
}

/**
 Writes one run a reason ended before it wrote its findings, and asserts nothing that reads as a review was saved.

 @param reason - why the run ended

 @returns nothing

 @example
 ```ts
 await aCutOffRunSavesNothing({ reason: 'turn-cap', },);
 ```
 */
async function aCutOffRunSavesNothing({ reason }: { readonly reason: string },): Promise<void> {
  await using folder = await makeFolder();
  /**
   The run's folder, whose summary says what ended it before it wrote its findings.
   */
  const runFolder = join(folder.path, 'runs', `${runNameOfAReview({ review: 'c-round-1', },)}-1`,);
  /**
   The session the cut-off run left, whose last message is the sentence it was at.
   */
  const sessionFile = join(folder.path, 'session.jsonl',);
  await mkdir(runFolder, { recursive: true, },);
  // The session as Pi leaves it, with a line feed after every record.
  await writeFile(sessionFile, `${JSON.stringify({
    type: 'message',
    message: {
      role: 'assistant',
      content: [{ type: 'text', text: 'Let me check the build target location.', },],
    },
  },)}\n`,);
  await writeFile(join(runFolder, 'summary.json',), JSON.stringify({
    endReason: reason,
    sessionStats: { sessionFile, },
  },),);
  /**
   What saving that review answered.
   */
  const saved = await saveTheLastMessage({
    runsFolder: join(folder.path, 'runs',),
    reviewFolder: folder.path,
    review: 'c-round-1',
  },);
  expect(saved.saved,).toBe(false,);
  expect(saved.line,).toContain(`ended ${reason}`,);
  // Nothing that reads as a review is written, because the sentence the run was at is not one.
  /**
   The file a review would have been written to.
   */
  const reviewFile = join(
    folder.path,
    'c-round-1',
    'review.md',
  );
  await expect(readFile(reviewFile,),).rejects.toThrow(MISSING_PATH_CODE,);
}

await describe({
  name: '',
  children: [
    describe({
      name: lastAssistantTextIn.name,
      children: [
        it({
          name: 'reads the whole text of the last assistant message, not the cut the summary keeps',
          fn: async () => {
            /**
             A session with a message the person wrote, one assistant message, and a later one.
             */
            const shown = [
              sessionLine('user', ['do the work'],),
              sessionLine('assistant', ['the first answer'],),
              sessionLine('assistant', ['a finding', 'another finding'],),
              '',
            ].join('\n',);
            expect(lastAssistantTextIn({ shown, },),).toBe('a finding\nanother finding',);
          },
        },),
        it({
          name: 'answers nothing for a session with no assistant message, and skips a record that is no message',
          fn: async () => {
            expect(lastAssistantTextIn({
              shown: `${
                [
                  JSON.stringify({ type: 'session', id: 's', },),
                  sessionLine('user', ['only the question'],),
                ].join('\n',)
              }\n`,
            },),).toBe('',);
          },
        },),
        it({
          name: 'reads the last message of a session whose final record is still being written, and leaves that fragment alone',
          fn: async () => {
            // A resumed run continues the session of the run before it, so Pi can be appending to this file while the command
            // that saves a review reads it. The fragment is no record yet, and parsing it throws `Unterminated string in JSON`.
            const shown = `${sessionLine('assistant', ['a finding that stands',],)}\n`
              + '{"type":"message","message":{"role":"assistant","cont';

            expect(lastAssistantTextIn({ shown, },),).toBe('a finding that stands',);
          },
        },),
      ],
    },),
    describe({
      name: newestRunOf.name,
      children: [
        it({
          name: 'reads the highest-numbered run folder of a review that has a summary, and skips one that has none',
          fn: async () => {
            await using folder = await makeFolder();
            await mkdir(join(folder.path, `${runNameOfAReview({ review: 'a-round-1', },)}-3`,), { recursive: true, },);
            await writeFile(join(folder.path, `${runNameOfAReview({ review: 'a-round-1', },)}-3`, 'summary.json',), '{}',);
            await mkdir(join(folder.path, `${runNameOfAReview({ review: 'a-round-1', },)}-10`,), { recursive: true, },);
            await mkdir(join(folder.path, `${runNameOfAReview({ review: 'a-round-9', },)}-2`,), { recursive: true, },);
            await writeFile(join(folder.path, `${runNameOfAReview({ review: 'a-round-9', },)}-2`, 'summary.json',), '{}',);
            expect(await newestRunOf({
              runsFolder: folder.path,
              review: 'a-round-9',
            },),).toEqual({ run: `${runNameOfAReview({ review: 'a-round-9', },)}-2`, },);
            expect(await newestRunOf({
              runsFolder: folder.path,
              review: 'nothing-round',
            },),).toEqual({},);
          },
        },),
      ],
    },),
    describe({
      name: saveTheLastMessage.name,
      children: [
        it({
          name: 'writes the last message of the newest run as the review file, with the note that says where it came from',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The session file the run's summary names.
             */
            const sessionFile = join(folder.path, 'session.jsonl',);
            await writeFile(sessionFile, `${sessionLine('assistant', ['a finding that matters'],)}\n`,);
            /**
             The run's folder.
             */
            const runFolder = join(folder.path, 'runs', `${runNameOfAReview({ review: 'a-round-1', },)}-3`,);
            await mkdir(join(folder.path, 'runs',), { recursive: true, },);
            await mkdir(runFolder, { recursive: true, },);
            await writeFile(join(runFolder, 'summary.json',), JSON.stringify({
              endReason: 'idle',
              sessionStats: { sessionFile, },
            },),);
            /**
             The reviews folder.
             */
            const reviewFolder = join(folder.path, 'reviews',);
            expect(await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder,
              review: 'a-round-1',
            },),).toEqual({
              review: 'a-round-1',
              saved: true,
              coverage: {
                kind: 'cannot-be-told',
                why: `nothing kept in ${join(reviewFolder, 'a-round-1', 'what-the-review-read.json',)} can be read as what this review was given`,
              },
              line: `a-round-1: saved its last message as review.md (${String('a finding that matters'.length,) } characters), and whether it covers its change cannot be told: nothing kept in ${join(reviewFolder, 'a-round-1', 'what-the-review-read.json',)} can be read as what this review was given`,
            },);
            expect(await readFile(join(reviewFolder, 'a-round-1', 'review.md',), 'utf8',),).toBe(`<!-- ${SAVED_REVIEW_NOTE} ${runNameOfAReview({ review: 'a-round-1', },)}-3 -->\n\na finding that matters\n`,);
          },
        },),
        it({
          name: 'leaves a review that wrote its own file alone, and says so',
          fn: async () => {
            await using folder = await makeFolder();
            await mkdir(join(folder.path, 'a-round-1',), { recursive: true, },);
            await writeFile(join(folder.path, 'a-round-1', 'review.md',), 'written by the run',);
            expect(await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder: folder.path,
              review: 'a-round-1',
            },),).toEqual({
              review: 'a-round-1',
              saved: false,
              coverage: {
                kind: 'cannot-be-told',
                why: `nothing kept in ${join(folder.path, 'a-round-1', 'what-the-review-read.json',)} can be read as what this review was given`,
              },
              line: `a-round-1: has its review.md, and whether it covers its change cannot be told: nothing kept in ${join(folder.path, 'a-round-1', 'what-the-review-read.json',)} can be read as what this review was given`,
            },);
          },
        },),
        it({
          name: 'reads a review that states the ranges it was given and a count of the listed files as one that covers its change',
          fn: async () => {
            await using folder = await makeFolder();
            await recordWhatTheReviewWasGiven({
              reviewFolder: folder.path,
              review: 'a-round-1',
              files: theFilesGiven(),
            },);
            await writeFile(
              join(folder.path, 'a-round-1', 'review.md',),
              reviewOfAChange({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
            );
            expect(await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder: folder.path,
              review: 'a-round-1',
            },),).toEqual({
              review: 'a-round-1',
              saved: false,
              coverage: {
                kind: 'covers',
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },
              line: `a-round-1: has its review.md, which states the ranges it was given (\`${theRangesGiven().join('`, `',)}\`) and ${String(theFilesGiven().length,)} of ${String(theFilesGiven().length,)} listed files read in full`,
            },);
          },
        },),
        it({
          name: 'reads a review that states another range than the one it was given as one that did not cover its change, and says so',
          fn: async () => {
            await using folder = await makeFolder();
            await recordWhatTheReviewWasGiven({
              reviewFolder: folder.path,
              review: 'a-round-1',
              files: theFilesGiven(),
            },);
            /**
             The range the review says it read, which is the last commit alone rather than the change it was given.
             */
            const stated = 'main...fix/b';
            await writeFile(
              join(folder.path, 'a-round-1', 'review.md',),
              reviewOfAChange({
                ranges: [stated,],
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
            );
            /**
             What saving that review answered.
             */
            const saved = await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder: folder.path,
              review: 'a-round-1',
            },);
            expect(saved.coverage,).toEqual({
              kind: 'does-not-cover',
              why: `it states \`${stated}\`, and the ranges it was given are \`${theRangesGiven().join('`, `',)}\``,
            },);
            expect(saved.line,).toContain('which does not cover its change: it states',);
          },
        },),
        it({
          name: 'reads a review that states no range as one that did not cover its change, and says so',
          fn: async () => {
            await using folder = await makeFolder();
            await recordWhatTheReviewWasGiven({
              reviewFolder: folder.path,
              review: 'a-round-1',
              files: theFilesGiven(),
            },);
            await writeFile(join(folder.path, 'a-round-1', 'review.md',), 'Not final.\n\nNo concrete defects found.\n',);
            /**
             What saving that review answered.
             */
            const saved = await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder: folder.path,
              review: 'a-round-1',
            },);
            expect(saved.coverage,).toEqual({
              kind: 'does-not-cover',
              why: `it states no range, and the ranges it was given are \`${theRangesGiven().join('`, `',)}\``,
            },);
            expect(saved.line,).toContain('which does not cover its change: it states no range',);
          },
        },),
        it({
          name: 'says whether a review written before the statement was asked for covers its change cannot be told, rather than that it does not',
          fn: async () => {
            await using folder = await makeFolder();
            await recordWhatTheReviewWasGiven({
              reviewFolder: folder.path,
              review: 'a-round-1',
            },);
            await writeFile(join(folder.path, 'a-round-1', 'review.md',), 'Not final.\n\nNo concrete defects found.\n',);
            /**
             What saving that review answered.
             */
            const saved = await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder: folder.path,
              review: 'a-round-1',
            },);
            expect(saved.coverage,).toEqual({
              kind: 'cannot-be-told',
              why: `${join(folder.path, 'a-round-1', 'what-the-review-read.json',)} names no listed files, so its brief was written before a review stated what it read`,
            },);
          },
        },),
        it({
          name: 'says a review with no run has nothing to save, and answers nothing for a run whose session is missing',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder: folder.path,
              review: 'gone-round-1',
            },).then(function lineOf(saved,): string {
              return saved.line;
            },),).toContain('no review.md and no finished run',);
            /**
             The run's folder, whose summary names a session file nobody wrote.
             */
            const runFolder = join(folder.path, 'runs', `${runNameOfAReview({ review: 'b-round-1', },)}-1`,);
            await mkdir(runFolder, { recursive: true, },);
            await writeFile(join(runFolder, 'summary.json',), JSON.stringify({
              endReason: 'idle',
              sessionStats: { sessionFile: join(folder.path, 'no-session.jsonl',), },
            },),);
            expect(await saveTheLastMessage({
              runsFolder: join(folder.path, 'runs',),
              reviewFolder: folder.path,
              review: 'b-round-1',
            },).then(function lineOf(saved,): string {
              return saved.line;
            },),).toContain(`no review.md and an empty last message in ${runNameOfAReview({ review: 'b-round-1', },)}-1`,);
          },
        },),
        it({
          name: 'writes nothing for a run any cap, wall clock or abort ended, and says why the review is missing',
          fn: async () => {
            /**
             The reasons a run ends with that are not its own terms, which a review that stopped before writing its findings
             has.
             */
            const cutShort = END_REASONS.filter(function notItsOwnTerms(reason,) {
              return !CLEAN_END_REASONS.has(reason,);
            },);
            // The list is every reason a run can name but the two it chooses, so a reason added to the driver is covered here.
            expect(cutShort.length,).toBeGreaterThan(0,);
            await Promise.all(cutShort.map(function oneReason(reason,) {
              return aCutOffRunSavesNothing({ reason, },);
            },),);
          },
        },),
      ],
    },),
    describe({
      name: parseSaveReviewOptions.name,
      children: [
        it({
          name: 'reads every review named, and takes the folders a coordinator has always had',
          fn: async () => {
            expect(parseSaveReviewOptions([
              '--review=a-round-1',
              '--review=b-round-2',
            ],),).toEqual({
              reviews: ['a-round-1', 'b-round-2',],
              runsFolder: join(process.env.HOME ?? '', 'temp', 'agent', 'pi-runs',),
              reviewFolder: join(process.env.HOME ?? '', 'temp', 'agent', DEFAULT_REVIEWS_FOLDER_IN_HOME.at(-1) ?? '',),
            },);
            expect(parseSaveReviewOptions([
              '--review=a-round-1',
              '--runs-folder=/runs',
              '--review-folder=/reviews',
            ],),).toEqual({
              reviews: ['a-round-1',],
              runsFolder: '/runs',
              reviewFolder: '/reviews',
            },);
          },
        },),
        it({
          name: 'refuses a command line that names no review',
          fn: async () => {
            expect(() => parseSaveReviewOptions([],),).toThrow('--review is required',);
            expect(() => parseSaveReviewOptions(['--review=',],),).toThrow('--review is required',);
          },
        },),
      ],
    },),
    describe({
      name: 'the command that saves a review',
      children: [
        it({
          name: 'ends with a status of 1 and says a review that states another range than the one it was given did not cover its change',
          fn: async () => {
            await using folder = await makeFolder();
            await recordWhatTheReviewWasGiven({
              reviewFolder: join(folder.path, 'reviews',),
              review: 'a-round-1',
              files: theFilesGiven(),
            },);
            await writeFile(
              join(folder.path, 'reviews', 'a-round-1', 'review.md',),
              reviewOfAChange({
                ranges: ['main...fix/b',],
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
            );
            /**
             What the command answered.
             */
            const ended = runSaveReviewCommand({
              reviewFolder: join(folder.path, 'reviews',),
              runsFolder: join(folder.path, 'runs',),
              review: 'a-round-1',
            },);
            expect(ended.status,).toBe(1,);
            expect(ended.stdout,).toContain('which does not cover its change: it states `main...fix/b`',);
          },
        },),
        it({
          name: 'ends with a status of 1 and says a review that states no range did not cover its change',
          fn: async () => {
            await using folder = await makeFolder();
            await recordWhatTheReviewWasGiven({
              reviewFolder: join(folder.path, 'reviews',),
              review: 'a-round-1',
              files: theFilesGiven(),
            },);
            await writeFile(join(folder.path, 'reviews', 'a-round-1', 'review.md',), 'Not final.\n\nNo concrete defects found.\n',);
            /**
             What the command answered.
             */
            const ended = runSaveReviewCommand({
              reviewFolder: join(folder.path, 'reviews',),
              runsFolder: join(folder.path, 'runs',),
              review: 'a-round-1',
            },);
            expect(ended.status,).toBe(1,);
            expect(ended.stdout,).toContain('which does not cover its change: it states no range',);
          },
        },),
        it({
          name: 'ends with a status of 0 and says a review whose last message states the ranges it was given covers its change',
          fn: async () => {
            await using folder = await makeFolder();
            await recordWhatTheReviewWasGiven({
              reviewFolder: join(folder.path, 'reviews',),
              review: 'a-round-1',
              files: theFilesGiven(),
            },);
            // The review wrote no file, so its last message is what is saved as one, and the lines the brief asked for are what
            // the saved file is read against.
            /**
             The session file the run's summary names.
             */
            const sessionFile = join(folder.path, 'session.jsonl',);
            await writeFile(
              sessionFile,
              `${sessionLine('assistant', [
                reviewOfAChange({
                  ranges: theRangesGiven(),
                  filesRead: theFilesGiven().length,
                  filesListed: theFilesGiven().length,
                },),
              ],)}\n`,
            );
            /**
             The run's folder.
             */
            const runFolder = join(folder.path, 'runs', `${runNameOfAReview({ review: 'a-round-1', },)}-1`,);
            await mkdir(runFolder, { recursive: true, },);
            await writeFile(join(runFolder, 'summary.json',), JSON.stringify({
              endReason: 'idle',
              sessionStats: { sessionFile, },
            },),);
            /**
             What the command answered.
             */
            const ended = runSaveReviewCommand({
              reviewFolder: join(folder.path, 'reviews',),
              runsFolder: join(folder.path, 'runs',),
              review: 'a-round-1',
            },);
            expect(ended.status,).toBe(0,);
            expect(ended.stdout,).toContain('saved its last message as review.md',);
            expect(ended.stdout,).toContain('which states the ranges it was given',);
          },
        },),
      ],
    },),
  ],
},);
