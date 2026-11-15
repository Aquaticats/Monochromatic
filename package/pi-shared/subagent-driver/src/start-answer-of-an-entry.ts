/**
 Why one entry's start failed, as the launcher says it: a value beside the runs that the settling reads as a field.

 The batch prints one line per entry for a reader, and the last line it prints is how many entries failed. A settling that fished the
 last line of that output therefore read `1 of 1 entries failed: a` as the reason, so a retrying start that had only given up waiting
 for a place at its deadline was set aside instead of put back at the head of the queue, with the deadline itself hidden. The review
 of `main...090e416990` reproduced that on 2026-11-15.

 So the batch writes `<runs>/<name>-start-answer.json` for every entry it started, beside its own output, holding the status that
 entry's start ended with and whether that status is the retrying start giving up for want of a place. No line of prose decides
 anything.

 @module
 */

import {
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { errorCodeOf, } from './error-code-of.ts';
import { NO_SLOT_EXIT_STATUS, } from './pool-gate.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The end of the file one entry's start answer is written to, which sits beside the runs under the entry's name.
 */
export const START_ANSWER_FILE_SUFFIX = '-start-answer.json';

/**
 The end of the file an answer is written through, which is renamed over the answer.
 */
const BEING_WRITTEN_SUFFIX = '.being-written';

/**
 No start answer for that entry, as a symbol so no answer can be mistaken for one.
 */
export const NO_START_ANSWER: unique symbol = Symbol('runs folder holds no start answer for that entry',);

/**
 What one entry's start ended with, as the batch that started it wrote it down.
 */
export type StartAnswerOfAnEntry = {
  /**
   The entry's name.
   */
  readonly name: string;

  /**
   The status its start ended with, or nothing when it never ran to an exit.
   */
  readonly status?: number;

  /**
   Whether the retrying start gave up asking for a place before its deadline, which is no fault of the entry.
   */
  readonly gaveUpWaitingForAPlace: boolean;

  /**
   The line the batch printed for a reader, which says the same thing in prose.
   */
  readonly line: string;

  /**
   Where the whole of what the failed start printed was kept, when it was kept.
   */
  readonly keptOutputAt?: string;
};

/**
 A start answer that cannot be read as one.
 */
export class StartAnswerError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'StartAnswerError';
}

/**
 Says whether a status is the retrying start giving up for want of a place.

 @param status - the status one entry's start ended with, or nothing when it never ran to an exit

 @returns whether the start gave up waiting for a place at its deadline

 @example
 ```ts
 const gaveUp = gaveUpWaitingForAPlaceIn({ status: 3, },);
 ```
 */
export function gaveUpWaitingForAPlaceIn({ status, }: { readonly status?: number },): boolean {
  return status === NO_SLOT_EXIT_STATUS;
}

/**
 Reads the file one entry's start answer is written to.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @returns the answer's path

 @example
 ```ts
 const file = startAnswerFileOf({ runsFolder: '/runs', name: 'a', });
 ```
 */
export function startAnswerFileOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): string {
  return join(
    runsFolder,
    `${name}${START_ANSWER_FILE_SUFFIX}`,
  );
}

/**
 Writes one entry's start answer, through a file renamed over it so a reader never sees half an answer.

 @param runsFolder - folder the driver writes its runs into

 @param answer - what the entry's start ended with

 @returns the answer's path

 @throws StartAnswerError when the answer cannot be written

 @example
 ```ts
 const file = await writeStartAnswer({ runsFolder: '/runs', answer: { name: 'a', status: 0, gaveUpWaitingForAPlace: false, line: 'a: status 0' }, });
 ```
 */
export async function writeStartAnswer({
  runsFolder,
  answer,
}: {
  readonly runsFolder: string;
  readonly answer: StartAnswerOfAnEntry;
},): Promise<string> {
  /**
   The answer's path.
   */
  const file = startAnswerFileOf({
    runsFolder,
    name: answer.name,
  },);
  try {
    await writeFile(
      `${file}${BEING_WRITTEN_SUFFIX}`,
      `${JSON.stringify(
        answer,
        undefined,
        2,
      )}\n`,
      { encoding: 'utf8', },
    );
    await rename(
      `${file}${BEING_WRITTEN_SUFFIX}`,
      file,
    );
    return file;
  }
  catch (error) {
    throw new StartAnswerError(`the start answer of ${answer.name} could not be written at ${file}: ${caughtValueText(error,)}`,);
  }
}

/**
 Keeps what each entry's start ended with beside the runs, so the filling that watches the batch reads a value rather than a line
 of prose.

 A file that cannot be written is reported and does not end a batch whose runs were started: the runs are the batch's work, and the
 answer is what lets a filling settle its entry afterwards.

 @param runsFolder - folder the driver writes its runs into

 @param answers - what each entry's start ended with, in the list's order

 @param report - where an answer that could not be written is reported

 @example
 ```ts
 await keepStartAnswersOfABatch({ runsFolder: '/runs', answers, report: console.error, },);
 ```
 */
export async function keepStartAnswersOfABatch({
  runsFolder,
  answers,
  report,
}: {
  readonly runsFolder: string;
  readonly answers: readonly StartAnswerOfAnEntry[];
  readonly report: (problem: string) => void;
},): Promise<void> {
  for (const answer of answers) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- one answer file after another, and each is its own file
      await writeStartAnswer({
        runsFolder,
        answer,
      },);
    }
    catch (error) {
      report(`${answer.name}: its start answer could not be written, so a filling cannot tell why it ended: ${caughtValueText(error,)}`,);
    }
  }
}

/**
 Reads one entry's start answer.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @returns the answer, or {@link NO_START_ANSWER} when the batch that started the entry wrote none

 @throws StartAnswerError when the file is there and is not an answer this package wrote

 @example
 ```ts
 const answer = await startAnswerOf({ runsFolder: '/runs', name: 'a', });
 ```
 */
export async function startAnswerOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): Promise<StartAnswerOfAnEntry | typeof NO_START_ANSWER> {
  /**
   The answer's path.
   */
  const file = startAnswerFileOf({
    runsFolder,
    name,
  },);
  /**
   The text it holds, or nothing when the batch wrote no answer.
   */
  const shown = await answerTextIn({ file, },);
  if (shown === NO_START_ANSWER) {
    return NO_START_ANSWER;
  }
  /**
   The text read as anything JSON holds.
   */
  const written = valueInJson({
    shown,
    file,
  },);
  if (
    ((typeof written) !== 'object') || (written === null)
      || (!('name' in written))
      || ((typeof written.name) !== 'string')
      || (!('gaveUpWaitingForAPlace' in written))
      || ((typeof written.gaveUpWaitingForAPlace) !== 'boolean')
      || (!('line' in written))
      || ((typeof written.line) !== 'string')
  ) {
    throw new StartAnswerError(`${file} does not hold the answer of a start this package wrote`,);
  }
  /**
   The status the answer names, when it names one.
   */
  const kept = ('keptOutputAt' in written) && ((typeof written.keptOutputAt) === 'string')
    ? { keptOutputAt: written.keptOutputAt, }
    : {};
  return {
    name: written.name,
    gaveUpWaitingForAPlace: written.gaveUpWaitingForAPlace,
    line: written.line,
    ...(('status' in written) && ((typeof written.status) === 'number') ? { status: written.status, } : {}),
    ...kept,
  };
}

/**
 Reads the text one answer's file holds.

 @param file - the answer's path

 @returns its text, or {@link NO_START_ANSWER} when the file is not there

 @throws StartAnswerError when the file is there and cannot be read

 @example
 ```ts
 const shown = await answerTextIn({ file: '/runs/a-start-answer.json', },);
 ```
 */
async function answerTextIn({ file, }: { readonly file: string },): Promise<string | typeof NO_START_ANSWER> {
  try {
    return await readFile(
      file,
      'utf8',
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return NO_START_ANSWER;
    }
    throw new StartAnswerError(`${file} could not be read: ${caughtValueText(error,)}`,);
  }
}

/**
 Reads the value a text holds, which a text that is not JSON is refused with.

 @param shown - the text

 @param file - the answer's path, for the message

 @returns the value

 @throws StartAnswerError when the text is not JSON

 @example
 ```ts
 const value = valueInJson({ shown: '{"name":"a"}', file: '/runs/a-start-answer.json', },);
 ```
 */
function valueInJson({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
},): unknown {
  try {
    return JSON.parse(shown,);
  }
  catch (error) {
    throw new StartAnswerError(`${file} is not JSON, so the answer it holds cannot be read: ${caughtValueText(error,)}`,);
  }
}
