/**
 Runs one parsed command and reports the exit status.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';

import {
  parseAnswers,
  type Answers,
} from './answers.ts';
import {
  parseArguments,
  USAGE,
} from './cli-args.ts';
import { AnswersError, } from './errors.ts';
import { recordAnswers, } from './record.ts';

/**
 Status of a run that did everything it was asked.
 */
export const EXIT_OK: number = 0;

/**
 Status of a run that was invoked wrongly.
 */
export const EXIT_MISUSE: number = 2;

/**
 Where a run's output goes.
 */
export type CliStreams = {
  readonly write: (line: string,) => void;
};

/**
 One invocation.
 */
export type RunCliRequest = {
  readonly arguments: readonly string[];
  readonly streams: CliStreams;
};

/**
 One recording request.
 */
export type RunRecordRequest = {
  readonly streams: CliStreams;
  readonly answersPath: string;
  readonly apply: boolean;
};

/**
 Reads one file, naming it when it cannot be read.
 
 @param path - File path.
 
 @returns The file's contents.
 
 @throws {@link AnswersError} when the file cannot be read.
 
 @example
 ```ts
 const text = await readText({ path: 'answers.json' });
 ```
 */
async function readText({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new AnswersError(`${path} could not be read: ${error.message}`,);
  }
}

/**
 Reads and validates one answers file.
 
 @param path - Answers file path.
 
 @returns The validated answers.
 
 @throws {@link AnswersError} when the file cannot be read or does not validate.
 
 @example
 ```ts
 const answers = await readAnswers({ path: 'answers.json' });
 ```
 */
async function readAnswers({ path, }: { readonly path: string; },): Promise<Answers> {
  return parseAnswers({
    text: await readText({ path, },),
    at: path,
  },);
}

/**
 Records one answers file in its questions file, or prints what that would do.
 
 @param streams - Stream the summary goes to.
 
 @param answersPath - Answers file to record.
 
 @param apply - Whether to write the questions file.
 
 @returns Exit status.
 
 @throws {@link AnswersError} when a named part is not in the questions file.
 
 @example
 ```ts
 const status = await runRecord({ streams, answersPath: 'answers.json', apply: false });
 ```
 */
export async function runRecord({
  streams,
  answersPath,
  apply,
}: RunRecordRequest,): Promise<number> {
  /**
   Answers to record.
   */
  const answers = await readAnswers({ path: answersPath, },);
  /**
   Questions file as it stands now, read fresh so an answer saved meanwhile is kept.
   */
  const text = await readText({ path: answers.file, },);
  /**
   What the recording removed and where it inserts.
   */
  const result = recordAnswers({
    text,
    answers,
  },);
  streams.write(`sections ${String(result.removedSections,)}, bullets ${String(result.removedBullets,)}, line runs ${String(result.removedRuns,)}, insert before line ${String(result.insertedAtLine + 1,)}\n`,);
  if (!apply) {
    streams.write(`would rewrite ${answers.file}; pass --apply to write it\n`,);
    return EXIT_OK;
  }
  await writeFile(
    answers.file,
    result.text,
    'utf8',
  );
  streams.write(`recorded in ${answers.file}\n`,);
  return EXIT_OK;
}

/**
 Runs one command line and reports the exit status.
 
 @param arguments - Tokens after the executable and script paths.
 
 @param streams - Stream the run prints on.
 
 @returns The status the process should exit with.
 
 @throws {@link CliInvocationError} when the tokens do not form a command.
 
 @throws {@link AnswersError} when an input cannot be read or a named part is missing.
 
 @example
 ```ts
 const status = await runCli({ arguments: ['--answers', 'answers.json'], streams: { write } });
 ```
 */
export function runCli({
  arguments: arguments_,
  streams,
}: RunCliRequest,): Promise<number> {
  /**
   Command the tokens form.
   */
  const command = parseArguments({ tokens: arguments_, },);
  if (command.kind === 'help') {
    streams.write(`${USAGE}\n`,);
    return Promise.resolve(EXIT_OK,);
  }
  return runRecord({
    streams,
    answersPath: command.answersPath,
    apply: command.apply,
  },);
}
