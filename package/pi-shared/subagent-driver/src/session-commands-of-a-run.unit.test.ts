/**
 Reading the commands a Pi session records: each `bash` call with the exit code its result carries, each `process` start, the folder
 the session began in, and how many `codemode` calls it holds whose own commands no record shows.

 Every case is a session text the test writes, so no session file, run or process is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  SessionRecordError,
  sessionReadings,
} from '../dist/final/node/index.mjs';

/**
 The file name every message names.

 @returns the session file's path

 @example
 ```ts
 const file = aFile();
 ```
 */
function aFile(): string {
  return '/sessions/a.jsonl';
}

/**
 Writes one session record as a line.

 @param record - the record to write

 @returns the line, with its newline

 @example
 ```ts
 const line = aLine({ type: 'session' });
 ```
 */
function aLine(record: Readonly<Record<string, unknown>>,): string {
  return `${JSON.stringify(record,)}\n`;
}

/**
 The session's own first record, which names the folder it began in.

 @returns the line

 @example
 ```ts
 const line = aSessionLine();
 ```
 */
function aSessionLine(): string {
  return aLine({
    type: 'session',
    version: 3,
    cwd: '/worktrees/pi-review-a',
  },);
}

/**
 Writes one tool call and its result.

 @param call - the call's id, tool and arguments

 @param result - the result's text and whether Pi marked it as an error

 @returns the two lines

 @example
 ```ts
 const lines = aCallLine({ id: 'c1' }, { text: '' });
 ```
 */
function aCall({
  call,
  result,
}: {
  readonly call: Readonly<Record<string, unknown>>;
  readonly result: {
    readonly text: string;
    readonly isError?: boolean;
  };
},): string {
  return aLine({
    type: 'message',
    message: {
      role: 'assistant',
      content: [{
        type: 'toolCall',
        ...call,
      },],
    },
  },) + aLine({
    type: 'message',
    message: {
      role: 'toolResult',
      toolCallId: call.id,
      content: [{
        type: 'text',
        text: result.text,
      },],
      isError: result.isError === true,
    },
  },);
}

/**
 Reads the error one session answers with.

 @param shown - the session's text

 @returns the error's message, or the words that say it read

 @example
 ```ts
 const message = messageOf('not json');
 ```
 */
function messageOf(shown: string,): string {
  try {
    sessionReadings({
      shown,
      file: aFile(),
    },);
    return 'it read';
  }
  catch (error) {
    // A session this reader refuses is what each such case asserts on.
    return (error instanceof SessionRecordError) ? error.message : String(error,);
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: sessionReadings.name,
      children: [
        it({
          name: 'reads the folder the session began in, and each bash call with the code its result ends with',
          fn: async () => {
            expect(sessionReadings({
              shown: aSessionLine() + aCall({
                call: {
                  id: 'c1',
                  name: 'bash',
                  arguments: { command: 'mise run build', },
                },
                result: { text: 'built\n\nCommand exited with code 0', },
              },) + aCall({
                call: {
                  id: 'c2',
                  name: 'bash',
                  arguments: { command: 'false', },
                },
                result: {
                  text: 'no\n\nCommand exited with code 3',
                  isError: true,
                },
              },),
              file: aFile(),
            },),).toEqual({
              workingFolder: '/worktrees/pi-review-a',
              commands: [
                {
                  command: 'mise run build',
                  tool: 'bash',
                  exitCode: 0,
                },
                {
                  command: 'false',
                  tool: 'bash',
                  exitCode: 3,
                },
              ],
              codemodeCalls: 0,
            },);
          },
        },),
        it({
          name: 'reads a bash result Pi did not mark as an error and that carries no code line as a command that exited zero',
          fn: async () => {
            expect(sessionReadings({
              shown: aCall({
                call: {
                  id: 'c1',
                  name: 'bash',
                  arguments: { command: 'true', },
                },
                result: { text: '(no output)', },
              },),
              file: aFile(),
            },).commands,).toEqual([{
              command: 'true',
              tool: 'bash',
              exitCode: 0,
            },],);
          },
        },),
        it({
          name: 'reads an error result with no code line as a command whose code it cannot name',
          fn: async () => {
            expect(sessionReadings({
              shown: aCall({
                call: {
                  id: 'c1',
                  name: 'bash',
                  arguments: { command: 'missing-program', },
                },
                result: {
                  text: 'spawn missing-program ENOENT',
                  isError: true,
                },
              },),
              file: aFile(),
            },).commands,).toEqual([{
              command: 'missing-program',
              tool: 'bash',
            },],);
          },
        },),
        it({
          name: 'reads a process start as a command with no code, and ignores the other process actions',
          fn: async () => {
            expect(sessionReadings({
              shown: aCall({
                call: {
                  id: 'c1',
                  name: 'process',
                  arguments: {
                    action: 'start',
                    command: 'mise run test',
                  },
                },
                result: { text: '{"processId":"7"}', },
              },) + aCall({
                call: {
                  id: 'c2',
                  name: 'process',
                  arguments: { action: 'output', },
                },
                result: { text: 'nothing yet', },
              },),
              file: aFile(),
            },).commands,).toEqual([{
              command: 'mise run test',
              tool: 'process',
            },],);
          },
        },),
        it({
          name: 'counts codemode calls, whose own commands it cannot see, and reads none of them as a command',
          fn: async () => {
            expect(sessionReadings({
              shown: aCall({
                call: {
                  id: 'c1',
                  name: 'codemode',
                  arguments: { code: 'await tools.bash({ command: "false" });', },
                },
                result: { text: 'done', },
              },),
              file: aFile(),
            },),).toEqual({
              workingFolder: '',
              commands: [],
              codemodeCalls: 1,
            },);
          },
        },),
        it({
          name: 'reads a call with no command text as no command, which a read or a write call is',
          fn: async () => {
            expect(sessionReadings({
              shown: aCall({
                call: {
                  id: 'c1',
                  name: 'bash',
                  arguments: {},
                },
                result: { text: '(no output)', },
              },),
              file: aFile(),
            },).commands,).toEqual([],);
          },
        },),
        //region What a session cannot be read from
        it({
          name: 'refuses a line that is no JSON, naming the line',
          fn: async () => {
            expect(messageOf(`${aSessionLine()}not json\n`,),).toContain('line 2 is no JSON');
          },
        },),
        it({
          name: 'refuses a line that is no JSON object, naming the line',
          fn: async () => {
            expect(messageOf('[1, 2]\n',),).toContain('line 1 is no JSON object');
          },
        },),
        it({
          name: 'refuses a tool call with no id or no tool, because its result could not be joined to it',
          fn: async () => {
            expect(
              messageOf(aLine({
                type: 'message',
                message: {
                  role: 'assistant',
                  content: [{
                    type: 'toolCall',
                    id: '',
                    name: 'bash',
                  },],
                },
              },),),
            ).toContain('holds a tool call with no id or no tool');
          },
        },),
        //endregion What a session cannot be read from
      ],
    },),
  ],
},);
