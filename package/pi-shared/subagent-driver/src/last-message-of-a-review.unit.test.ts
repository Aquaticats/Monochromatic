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
import { join, } from 'node:path';
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
  await writeFile(sessionFile, JSON.stringify({
    type: 'message',
    message: {
      role: 'assistant',
      content: [{ type: 'text', text: 'Let me check the build target location.', },],
    },
  },),);
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
              shown: [
                JSON.stringify({ type: 'session', id: 's', },),
                sessionLine('user', ['only the question'],),
              ].join('\n',),
            },),).toBe('',);
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
              line: `a-round-1: saved its last message as review.md (${String('a finding that matters'.length,) } characters)`,
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
              line: 'a-round-1: has its review.md',
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
  ],
},);
