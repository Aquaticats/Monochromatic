/**
 Readings of the options the commands that edit a queue of runs share: which queue, which runs and reviews folders, the position an
 entry goes to, and the entry itself as the queue writes it.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { optionIn, } from './option-in.ts';
import {
  isQueueRecord,
  type QueueRecord,
  QueueFileError,
} from './queue-of-runs.ts';
import { requiredText, } from './required-option-values.ts';
import {
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
} from './reviewer-of-a-change.ts';

/**
 The option that names the queue.
 */
export const QUEUE_FILE_OPTION = '--queue=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names the folder that holds one folder per review.
 */
const REVIEWS_FOLDER_OPTION = '--reviews-folder=';

/**
 The option that names the position an entry goes to, counted from the head.
 */
export const POSITION_OPTION = '--position=';

/**
 The option that names one entry, as the queue writes it.
 */
export const ENTRY_OPTION = '--entry=';

/**
 The option that names one entry.
 */
const NAME_OPTION = '--name=';

/**
 What a command needs to refuse one of its options: its usage line and the error it answers with.
 */
export type QueueOptionWords = {
  /**
   The command's usage line, without the `Usage: ` printed before it.
   */
  readonly usage: string;

  /**
   Builds the error the command answers a refused option with.
   */
  readonly errorOf: (message: string) => Error;
};

/**
 Reads the queue a command was told to edit.

 @param commandLine - the words the command was given

 @param words - the command's usage line and the error it refuses with

 @returns the queue's path

 @throws the error the caller builds, when no queue is named

 @example
 ```ts
 const queue = queueFileIn({ commandLine, words });
 ```
 */
export function queueFileIn({
  commandLine,
  words,
}: {
  readonly commandLine: readonly string[];
  readonly words: QueueOptionWords;
},): string {
  return requiredText({
    value: optionIn({
      commandLine,
      option: QUEUE_FILE_OPTION,
    },),
    name: 'queue',
    usage: words.usage,
    errorOf: words.errorOf,
  },);
}

/**
 Reads the folder the runs live in.

 @param commandLine - the words the command was given

 @returns the folder, which is the coordinator's own under the home folder when none is named

 @example
 ```ts
 const runs = runsFolderIn({ commandLine });
 ```
 */
export function runsFolderIn({ commandLine, }: { readonly commandLine: readonly string[] },): string {
  /**
   The folder the caller named.
   */
  const named = optionIn({
    commandLine,
    option: RUNS_FOLDER_OPTION,
  },);
  return (named === '')
    ? join(
      homedir(),
      ...DEFAULT_RUNS_FOLDER_IN_HOME,
    )
    : named;
}

/**
 Reads the folder that holds one folder per review.

 @param commandLine - the words the command was given

 @returns the folder, which is the coordinator's own under the home folder when none is named

 @example
 ```ts
 const reviews = reviewsFolderIn({ commandLine });
 ```
 */
export function reviewsFolderIn({ commandLine, }: { readonly commandLine: readonly string[] },): string {
  /**
   The folder the caller named.
   */
  const named = optionIn({
    commandLine,
    option: REVIEWS_FOLDER_OPTION,
  },);
  return (named === '')
    ? join(
      homedir(),
      ...DEFAULT_REVIEWS_FOLDER_IN_HOME,
    )
    : named;
}

/**
 Reads a position counted from the head of the queue.

 @param commandLine - the words the command was given

 @param words - the command's usage line and the error it refuses with

 @returns the position, which is the head when none is named

 @throws the error the caller builds, when the position is no whole number from zero

 @example
 ```ts
 const position = positionIn({ commandLine, words });
 ```
 */
export function positionIn({
  commandLine,
  words,
}: {
  readonly commandLine: readonly string[];
  readonly words: QueueOptionWords;
},): number {
  /**
   The position as the caller wrote it.
   */
  const written = optionIn({
    commandLine,
    option: POSITION_OPTION,
  },);
  if (written === '') {
    return 0;
  }
  /**
   The position as a number.
   */
  const position = Number(written,);
  if ((!Number.isInteger(position,)) || (position < 0)) {
    throw words.errorOf(`--position must be a whole number from 0, got ${written}.\nUsage: ${words.usage}`,);
  }
  return position;
}

/**
 Reads the name one entry is named by.

 @param commandLine - the words the command was given

 @param words - the command's usage line and the error it refuses with

 @returns the name

 @throws the error the caller builds, when no name is given

 @example
 ```ts
 const name = nameIn({ commandLine, words });
 ```
 */
export function nameIn({
  commandLine,
  words,
}: {
  readonly commandLine: readonly string[];
  readonly words: QueueOptionWords;
},): string {
  return requiredText({
    value: optionIn({
      commandLine,
      option: NAME_OPTION,
    },),
    name: 'name',
    usage: words.usage,
    errorOf: words.errorOf,
  },);
}

/**
 Reads one entry as the queue writes it.

 @param commandLine - the words the command was given

 @param words - the command's usage line and the error it refuses with

 @returns the entry

 @throws the error the caller builds, when no entry is named or it is no JSON object with a name

 @example
 ```ts
 const entry = entryIn({ commandLine, words });
 ```
 */
export function entryIn({
  commandLine,
  words,
}: {
  readonly commandLine: readonly string[];
  readonly words: QueueOptionWords;
},): QueueRecord {
  /**
   The entry as the caller wrote it.
   */
  const written = requiredText({
    value: optionIn({
      commandLine,
      option: ENTRY_OPTION,
    },),
    name: 'entry',
    usage: words.usage,
    errorOf: words.errorOf,
  },);
  /**
   The entry, of any shape, or the text that says it is no JSON.
   */
  const parsed: unknown = (function parseIt() {
    try {
      /**
       What the entry's text parsed to.
       */
      const json: unknown = JSON.parse(written,);
      return json;
    }
    catch (error) {
      throw words.errorOf(`--entry is no JSON: ${String(error,)}.\nUsage: ${words.usage}`,);
    }
  })();
  if (!isQueueRecord(parsed,)) {
    throw words.errorOf(`--entry holds no JSON object naming a run.\nUsage: ${words.usage}`,);
  }
  return parsed;
}

/**
 Reads an option that must be a whole number from one, or one of the defaults a command ships.

 @param commandLine - the words the command was given

 @param option - the option, with its equals sign

 @param words - the command's usage line and the error it refuses with

 @returns the number, or nothing when the option is not named

 @throws the error the caller builds, when the value is no whole number from one

 @example
 ```ts
 const level = wholeNumberFromOne({ commandLine, option: '--level=', words });
 ```
 */
export function wholeNumberFromOne({
  commandLine,
  option,
  words,
}: {
  readonly commandLine: readonly string[];
  readonly option: string;
  readonly words: QueueOptionWords;
},): { readonly value?: number } {
  /**
   The value as the caller wrote it.
   */
  const written = optionIn({
    commandLine,
    option,
  },);
  if (written === '') {
    return {};
  }
  /**
   The value as a number.
   */
  const value = Number(written,);
  if ((!Number.isInteger(value,)) || (value < 1)) {
    throw words.errorOf(`${option.slice(
      0,
      -1,
    )} must be a whole number from 1, got ${written}.\nUsage: ${words.usage}`,);
  }
  return { value, };
}

/**
 Reads an error the queue file answers a refused edit with, as one of the command's own.

 @param error - what the queue file refused with

 @param words - the command's usage line and the error it refuses with

 @returns the error the command answers with, which names the queue's refusal

 @example
 ```ts
 const refused = queueRefusalOf({ error, words });
 ```
 */
export function queueRefusalOf({
  error,
  words,
}: {
  readonly error: unknown;
  readonly words: QueueOptionWords;
},): Error {
  // An entry already in the queue is a refusal the command answers with its own error, so the caller prints one line rather than a
  // stack through the file operations that got there.
  if (error instanceof QueueFileError) {
    return words.errorOf(error.message,);
  }
  return (Error.isError(error,)) ? error : new Error(String(error,),);
}
