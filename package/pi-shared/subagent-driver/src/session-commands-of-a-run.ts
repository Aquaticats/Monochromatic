/**
 What a finished run's Pi session records of the commands it ran: each `bash` call and each `process` start with its command, the
 code a `bash` call exited with when the result records one, and the folder the session started in.

 The session is the only evidence of what a review really ran: it holds Pi's own tool calls and their results, which the driver
 writes beside its own summary rather than into it. A command run inside a `codemode` call is not a tool call of its own, so this
 reader counts those calls and says it cannot see inside them rather than guessing.

 @module
 */

import { readRegularFileAt, } from './reading-a-regular-file.ts';
import type { PiRecord, } from './driver-types.ts';
import {
  fieldsOf,
  isRecord,
  recordsIn,
  textOf,
} from './read-pi-fields.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 Mebibytes of session text this reader takes, which is far more than a review writes.

 Measured: the 103-turn review session of 2026-11-12T15:21:26Z was 1,133,922 bytes, and the longest review of that day used 156
 turns; 64 MiB is about fifty-nine times the measured session and still less than a machine's memory.
 */
const SESSION_TEXT_LIMIT_MEBIBYTES = 64;

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE = 1_048_576;

/**
 The most session text one run may leave for this reader to hold.
 */
export const SESSION_TEXT_LIMIT_BYTES: number = SESSION_TEXT_LIMIT_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 A session file this reader cannot read, or a record in it that is not one JSON object per line.
 */
export class SessionRecordError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SessionRecordError';
}

/**
 The tool a command came from: a `bash` call, which the session records a status for, or a `process` start, which it does not.
 */
export type CommandTool = 'bash' | 'process';

/**
 One command a session records.
 */
export type SessionCommand = {
  /**
   The command, as it was written.
   */
  readonly command: string;

  /**
   The tool the command came from.
   */
  readonly tool: CommandTool;

  /**
   The code a `bash` call exited with, when its result records one; absent for a `process` start and for a result no code was
   written into.
   */
  readonly exitCode?: number;
};

/**
 What a session says about the commands its run ran.
 */
export type SessionReadings = {
  /**
   The folder the session started in, which is the run's worktree; empty when the session records none.
   */
  readonly workingFolder: string;

  /**
   Each command the session records, in the order its call was written.
   */
  readonly commands: readonly SessionCommand[];

  /**
   How many `codemode` calls the session holds, whose own commands it does not record and this reader cannot see.
   */
  readonly codemodeCalls: number;

  /**
   The provider and model the run started on, as the session's first model change records them; empty when it records none.
   */
  readonly reviewer: {
    readonly provider: string;
    readonly model: string;
  };
};

/**
 One tool call as the session wrote it.
 */
type RecordedCall = {
  /**
   The tool's name.
   */
  readonly name: string;

  /**
   The call's arguments.
   */
  readonly arguments: Readonly<Record<string, unknown>>;
};

/**
 What one call's result says.
 */
type RecordedResult = {
  /**
   Whether Pi marked the result as a failure.
   */
  readonly isError: boolean;

  /**
   The result's text, joined from its text parts.
   */
  readonly text: string;
};

/**
 The words a bash result ends with when the command exited non-zero, which the harness writes as its own last line.

 A command that exited zero has no such line, so a result Pi did not mark as an error is zero. A result that is an error with no
 code line is a command whose status this reader cannot name.
 */
const EXIT_CODE_MARKER = 'Command exited with code ';

/**
 No exit code in a result, as a symbol so that no code can stand for it.
 */
const NO_EXIT_CODE: unique symbol = Symbol('session result holds no exit code',);

/**
 No command in a tool call, as a symbol so that no command text can stand for it.
 */
const NO_COMMAND: unique symbol = Symbol('tool call holds no command text',);

/**
 Reads the exit code a bash call's result carries.

 @param results - the results the session wrote, by the call's id

 @param id - the call whose result the code is read from

 @returns the code, or {@link NO_EXIT_CODE} when the result does not name one

 @example
 ```ts
 const code = exitCodeIn({ results, id: 'call_5db80b91de' });
 ```
 */
function exitCodeIn({
  results,
  id,
}: {
  readonly results: ReadonlyMap<string, RecordedResult>;
  readonly id: string;
},): number | typeof NO_EXIT_CODE {
  /**
   The result the session wrote for that call.
   */
  const result = results.get(id,);
  if (result === undefined) {
    return NO_EXIT_CODE;
  }
  // Pi marks a command that exited non-zero as a failed result, and the harness writes its marker into such a result. A result Pi
  // did not mark as an error is a command that exited zero, whose own output is the whole text: a line of it that reads like the
  // harness's marker is the command's own, so no marker is read from it.
  if (!result.isError) {
    return 0;
  }
  /**
   Where the harness wrote the exit code, when it wrote one.
   */
  const at = result.text
    .lastIndexOf(EXIT_CODE_MARKER,);
  if (at !== (-1)) {
    /**
     The code itself, which follows the marker and is the last text of the result.
     */
    const written = result.text
      .slice(at + EXIT_CODE_MARKER.length,)
      .trim();
    if (isDigitsOnly(written,)) {
      return Number(written,);
    }
  }
  return NO_EXIT_CODE;
}

/**
 Reads one session file's text, refusing one that is not a regular file of the size this reader takes.

 The descriptor is opened first, without waiting for a writer, and looked at second: a named pipe at a session file's path answers
 at once instead of holding the reader for ever.

 @param file - the session file

 @param limitBytes - the most bytes the file may hold

 @returns the file's text

 @throws SessionRecordError when the file is not there, is no regular file, cannot be read, or holds more than the limit, because a
 caller that cannot compare a run's commands must answer that rather than raise the operating system's own failure

 @example
 ```ts
 const shown = await sessionTextIn({ file: '/sessions/a.jsonl' });
 ```
 */
export async function sessionTextIn({
  file,
  limitBytes,
}: {
  readonly file: string;
  readonly limitBytes?: number;
},): Promise<string> {
  /**
   What the read answered.
   */
  const read = await readRegularFileAt({
    path: file,
    ...((limitBytes === undefined) ? {} : { limitBytes, }),
  },);
  if (read.kind === 'text') {
    return read.text;
  }
  throw new SessionRecordError(
    `${(read.kind === 'no file') ? file : read.reason} means what that run ran cannot be compared`,
  );
}

/**
 Reads one line of a session as the JSON object it must be.

 @param line - the line as the session wrote it

 @param file - the session file's path, for the message

 @param at - the line's index, from zero

 @returns the record, with its fields readable

 @throws SessionRecordError when the line is no JSON, or no JSON object

 @example
 ```ts
 const record = recordIn({ line: '{"type":"session"}', file: '/s/a.jsonl', at: 0 });
 ```
 */
function recordIn({
  line,
  file,
  at,
}: {
  readonly line: string;
  readonly file: string;
  readonly at: number;
},): PiRecord {
  try {
    /**
     What the line holds, of any shape.
     */
    const kept: unknown = JSON.parse(line,);
    if (!isRecord(kept,)) {
      throw new SessionRecordError(
        `${file} line ${String(at + 1,)} is no JSON object, so what that run recorded cannot be read`,
      );
    }
    return kept;
  }
  catch (error) {
    // A line this reader cannot parse stops the comparison rather than being skipped, because a skipped line may be the one that
    // records the command being checked.
    if (error instanceof SessionRecordError) {
      throw error;
    }
    throw new SessionRecordError(
      `${file} line ${String(at + 1,)} is no JSON, so what that run recorded cannot be read: ${String(error,)}`,
    );
  }
}

/**
 Reads the commands a session records.

 @param shown - the session file's text, one JSON record per line

 @param file - the session file's path, for the message

 @returns the folder the session started in and the commands it recorded

 @throws SessionRecordError when a line is no JSON object, or a tool call names no id or no tool

 @example
 ```ts
 const readings = sessionReadings({ shown: await sessionTextIn({ file: '/sessions/a.jsonl' }), file: '/sessions/a.jsonl' });
 ```
 */
export function sessionReadings({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
},): SessionReadings {
  /**
   The tool calls the session wrote, by their id.
   */
  const calls = new Map<string, RecordedCall>();
  /**
   The results the session wrote, by the call's id.
   */
  const results = new Map<string, RecordedResult>();
  /**
   What the session says about itself as its records are read.
   */
  const state = {
    workingFolder: '',
    codemodeCalls: 0,
    provider: '',
    model: '',
  };
  for (const [at, line] of shown.split('\n',)
    .entries()) {
    if (line === '') {
      continue;
    }
    /**
     One record of the session, read before what it says is read.
     */
    const record = recordIn({
      line,
      file,
      at,
    },);
    if (record.type === 'session') {
      state.workingFolder = textOf(record.cwd,) || state.workingFolder;
      continue;
    }
    if (record.type === 'model_change') {
      // The first model change a session records is the model the run started on; a later one is the run moving, and the review a
      // second start was briefed against is the one this run began with.
      if (state.provider === '') {
        state.provider = textOf(record.provider,);
      }
      if (state.model === '') {
        state.model = textOf(record.modelId,);
      }
      continue;
    }
    if (record.type !== 'message') {
      continue;
    }
    /**
     The message the record carries.
     */
    const message = fieldsOf(record.message,);
    if (message.role === 'assistant') {
      for (const part of recordsIn(message.content,)) {
        if (part.type !== 'toolCall') {
          continue;
        }
        /**
         The call's id, which its result names.
         */
        const id = textOf(part.id,);
        /**
         The tool the call names.
         */
        const name = textOf(part.name,);
        if ((id === '') || (name === '')) {
          throw new SessionRecordError(
            `${file} line ${String(at + 1,)} holds a tool call with no id or no tool, so what that run ran cannot be read`,
          );
        }
        calls.set(
          id,
          {
            name,
            arguments: fieldsOf(part.arguments,),
          },
        );
      }
      continue;
    }
    if (message.role !== 'toolResult') {
      continue;
    }
    /**
     The call whose result this is.
     */
    const id = textOf(message.toolCallId,);
    if (id === '') {
      continue;
    }
    results.set(
      id,
      {
        isError: message.isError === true,
        text: recordsIn(message.content,)
          .map(function asText(part,) {
            return textOf(part.text,);
          },)
          .join('\n',),
      },
    );
  }
  /**
   The commands the session records, in the order of their calls.
   */
  const commands: SessionCommand[] = [];
  for (const [id, call] of calls) {
    if (call.name === 'codemode') {
      state.codemodeCalls += 1;
      continue;
    }
    /**
     The command the call wrote, when it wrote one this reader can compare.
     */
    const command = commandOf(call,);
    if (command === NO_COMMAND) {
      continue;
    }
    /**
     The code a bash call's result records, which a process start's result never does: the process is started and reported on
     later, so no single result says how its command ended.
     */
    const exitCode = (call.name === 'bash')
      ? exitCodeIn({
        results,
        id,
      },)
      : NO_EXIT_CODE;
    commands.push({
      command,
      tool: call.name === 'bash' ? 'bash' : 'process',
      ...((exitCode === NO_EXIT_CODE) ? {} : { exitCode, }),
    },);
  }
  return {
    workingFolder: state.workingFolder,
    commands,
    codemodeCalls: state.codemodeCalls,
    reviewer: {
      provider: state.provider,
      model: state.model,
    },
  };
}

/**
 Reads the command one tool call wrote, when the call is one that runs a command.

 @param call - the call

 @returns the command, or nothing when the call is not a `bash` call or a `process` start

 @example
 ```ts
 const command = commandOf({ name: 'bash', arguments: { command: 'true' } });
 ```
 */
function commandOf(call: RecordedCall,): string | typeof NO_COMMAND {
  if (call.name === 'bash') {
    /**
     The command a bash call runs.
     */
    const command = textOf(call.arguments
      .command,);
    return (command === '') ? NO_COMMAND : command;
  }
  if (call.name === 'process') {
    if (textOf(call.arguments
      .action,) !== 'start') {
      return NO_COMMAND;
    }
    /**
     The command a process start runs.
     */
    const command = textOf(call.arguments
      .command,);
    return (command === '') ? NO_COMMAND : command;
  }
  return NO_COMMAND;
}
