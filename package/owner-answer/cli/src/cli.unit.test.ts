/**
 Tests for the owner-answer grammar and its entry, driven by disposable files.

 @module
 */

import {
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  AnswersError,
  CliInvocationError,
  EXIT_MISUSE,
  EXIT_OK,
  parseArguments,
  runCli,
} from '../dist/final/node/index.mjs';

/**
 Returns the error a call throws.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call returns, which means it refused nothing.
 
 @example
 ```ts
 const refused = failureOf(() => parseArguments({ tokens: [] }));
 ```
 */
function failureOf(action: () => unknown,): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call returned where the test expects a refusal.',);
}

/**
 Returns the error an asynchronous call rejects with.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call resolves, which means it refused nothing.
 
 @example
 ```ts
 const refused = await rejectionOf(() => runCli({ arguments: [], streams }));
 ```
 */
async function rejectionOf(action: () => Promise<unknown>,): Promise<unknown> {
  try {
    await action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call resolved where the test expects a refusal.',);
}

/**
 Writes a questions file and one answers file that records a block in it.
 
 @param directory - Folder both files live in.
 
 @param apply - Unused; the caller decides.
 
 @returns The answers file's path.
 
 @example
 ```ts
 const answersPath = await writeFixtures('/tmp/x');
 ```
 */
async function writeFixtures(directory: string,): Promise<string> {
  /**
   Questions file both runs read.
   */
  const questions = join(directory, 'questions.md',);
  await writeFile(questions, [
    '## Open',
    '',
    '### A question, asked 2026-11-09T01:16:11-05:00',
    '',
    'Body.',
    '',
    '## Answered in the question tool, recorded',
    '',
  ].join('\n',), 'utf8',);
  /**
   Answers file describing the recording.
   */
  const answers = join(directory, 'answers.json',);
  await writeFile(answers, JSON.stringify({
    file: questions,
    record: '## Answered, recorded 2026-11-10T05:40:16-05:00\n\n"the owner\'s words"\n',
    anchor: '## Answered in the question tool, recorded',
    remove: { sections: ['### A question, asked 2026-11-09T01:16:11-05:00',], },
  },), 'utf8',);
  return answers;
}

/**
 Streams that collect what the command printed.
 
 @returns The stream and the lines it holds.
 
 @example
 ```ts
 const { streams, lines } = collectingStream();
 ```
 */
function collectingStream(): {
  readonly streams: { readonly write: (line: string,) => void };
  readonly lines: string[];
} {
  /**
   Lines the command printed.
   */
  const lines: string[] = [];
  return {
    lines,
    streams: {
      write: function write(line,) {
        lines.push(line,);
      },
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: parseArguments.name,
      children: [
        it({
          name: 'reads --help as the usage request',
          fn: async () => {
            expect(parseArguments({ tokens: ['--help',], },),).toEqual({ kind: 'help', },);
          },
        },),
        it({
          name: 'reads an answers file and an optional apply',
          fn: async () => {
            expect(parseArguments({
              tokens: [
                '--answers',
                'answers.json',
              ],
            },),).toEqual({
              kind: 'record',
              answersPath: 'answers.json',
              apply: false,
            },);
            expect(parseArguments({
              tokens: [
                '--answers',
                'answers.json',
                '--apply',
              ],
            },),).toEqual({
              kind: 'record',
              answersPath: 'answers.json',
              apply: true,
            },);
          },
        },),
        it({
          name: 'refuses a run with no answers file',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseArguments({ tokens: [], },),);
            expect(refused,).toBeInstanceOf(CliInvocationError,);
            expect((refused as Error).message,).toContain('--answers FILE is required',);
          },
        },),
        it({
          name: 'refuses a positional argument',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseArguments({
              tokens: [
                '--answers',
                'answers.json',
                'answers.json',
              ],
            },),);
            expect(refused,).toBeInstanceOf(CliInvocationError,);
            expect((refused as Error).message,).toContain('takes no positional argument',);
          },
        },),
        it({
          name: 'refuses a flag the grammar does not define',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseArguments({ tokens: ['--aply',], },),);
            expect(refused,).toBeInstanceOf(CliInvocationError,);
            expect((refused as Error).message,).toContain('Usage:',);
          },
        },),
      ],
    },),
    describe({
      name: runCli.name,
      children: [
        it({
          name: 'prints the usage text',
          fn: async () => {
            /**
             Stream collecting the output.
             */
            const { streams, lines } = collectingStream();
            expect(await runCli({
              arguments: ['--help',],
              streams,
            },),).toBe(EXIT_OK,);
            expect(lines.join('',),).toContain('owner-answer --answers FILE',);
          },
        },),
        it({
          name: 'prints what a recording would do, and writes nothing, without apply',
          fn: async () => {
            await using directory = await mkdtempDisposable(join(scratchRoot(), 'owner-answer-',),);
            /**
             Answers file the run reads.
             */
            const answersPath = await writeFixtures(directory.path,);
            /**
             Stream collecting the output.
             */
            const { streams, lines } = collectingStream();
            expect(await runCli({
              arguments: ['--answers', answersPath,],
              streams,
            },),).toBe(EXIT_OK,);
            expect(lines.join('',),).toContain('sections 1, bullets 0, line runs 0',);
            expect(lines.join('',),).toContain('pass --apply to write it',);
          },
        },),
        it({
          name: 'writes the questions file when asked to apply',
          fn: async () => {
            await using directory = await mkdtempDisposable(join(scratchRoot(), 'owner-answer-',),);
            /**
             Answers file the run reads.
             */
            const answersPath = await writeFixtures(directory.path,);
            /**
             Stream collecting the output.
             */
            const { streams, lines } = collectingStream();
            expect(await runCli({
              arguments: [
                '--answers',
                answersPath,
                '--apply',
              ],
              streams,
            },),).toBe(EXIT_OK,);
            expect(lines.join('',),).toContain('recorded in',);
            /**
             Answers as the run read them, so the test knows the questions file's path.
             */
            const answers = JSON.parse(await readFile(answersPath, 'utf8',),) as { readonly file: string };
            /**
             Questions file after the write.
             */
            const written = await readFile(answers.file, 'utf8',);
            expect(written,).toContain('## Answered, recorded 2026-11-10T05:40:16-05:00',);
            expect(written,).not.toContain('### A question, asked 2026-11-09T01:16:11-05:00',);
          },
        },),
        it({
          name: 'refuses an answers file that is not there',
          fn: async () => {
            /**
             Stream collecting the output.
             */
            const { streams } = collectingStream();
            /**
             Value the run threw.
             */
            const refused = await rejectionOf(() => runCli({
              arguments: ['--answers', join(scratchRoot(), 'owner-answer-absent', 'answers.json',),],
              streams,
            },),);
            expect(refused,).toBeInstanceOf(AnswersError,);
            expect(EXIT_MISUSE,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);
