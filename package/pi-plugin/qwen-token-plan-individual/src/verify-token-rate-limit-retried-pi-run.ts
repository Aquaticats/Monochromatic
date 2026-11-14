/**
 One pi run of the passing-token-rate-limit proof:
 the command both runs start, the feed each one prints, and the reading of what
 it settled on.

 @module
 */

import type { Dirent, } from 'node:fs';
import {
  mkdir,
  readFile,
  readdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { runBoundedChild, } from './verify-token-rate-limit-retried-bounded-child.ts';

//region Constants

/**
 Bound on one pi run, so a run that never settles fails the proof instead of
 holding it.
 */
const RUN_TIMEOUT_MILLISECONDS = 120_000;

//endregion Constants

//region Types

/**
 What one pi run left behind.
 */
export type PiRun = {
  /**
   Session events the run printed, in order.
   */
  readonly events: readonly Readonly<Record<string, unknown>>[];
  /**
   Exit status the run ended with, absent when it was killed instead.
   */
  readonly exitCode?: number;
  /**
   Whether the run reached its bound and was killed.
   */
  readonly timedOut: boolean;
};

/**
 The feed one run printed.
 */
export type ParsedFeed = {
  /**
   Parsed events, in the order pi printed them.
   */
  readonly events: readonly Readonly<Record<string, unknown>>[];
  /**
   Lines that were not events, kept so a broken feed is reported rather than
   hidden.
   */
  readonly unparsed: readonly string[];
};

/**
 What one run needs to start.
 */
export type PiRunRequest = {
  /**
   Folder the run's feed and error output are kept in.
   */
  readonly root: string;
  /**
   Name the run's files are named for.
   */
  readonly name: string;
  /**
   Pi's CLI entry.
   */
  readonly piCli: string;
  /**
   Bundle the run loads.
   */
  readonly extension: string;
  /**
   Throwaway home the run sees, so this machine's `~/.pi` is never read.
   */
  readonly home: string;
  /**
   Agent directory inside that home.
   */
  readonly agentDir: string;
  /**
   Provider route the run asks for.
   */
  readonly provider: string;
  /**
   Model the run asks for.
   */
  readonly model: string;
  /**
   Prompt the run is given.
   */
  readonly prompt: string;
};

//endregion Types

//region Small helpers

/**
 Whether a parsed value is a plain object an event can be read by key.

 @param value - parsed JSON value

 @returns `true` for an object that is not an array

 @example
 ```typescript
 if (isRecord(parsed)) {
   // parsed's fields can be read here
 }
 ```
 */
export function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Read one assistant message's text,
 so the reply a run ended with can be compared with what the stand-in sent.

 @param message - assistant message a session event carried

 @returns every text block the message holds, joined

 @example
 ```typescript
 const text = assistantText({ message });
 ```
 */
export function assistantText({
  message,
}: {
  readonly message: Readonly<Record<string, unknown>>;
},): string {
  /**
   Content blocks the message carries, which it may hold as anything.
   */
  const { content, } = message;
  if (!Array.isArray(content,))
    return '';
  return content.flatMap(function textOf(block: unknown,) {
    if (!isRecord(block,))
      return [];
    /**
     Text the block carries, when it is a text block.
     */
    const { text, } = block;
    return (typeof text) === 'string' ? [text,] : [];
  },)
    .join('',);
}

//endregion Small helpers

//region Reading a feed

/**
 Parse the events one pi run printed, and keep the raw feed for a reader.

 @param stdout - everything the run wrote to standard output

 @param root - folder the raw feed is kept in

 @param name - run name the feed file is named for

 @returns the parsed events, and the lines that were not events

 @example
 ```typescript
 const parsed = await parseFeed({ stdout, root, name: 'retried' });
 ```
 */
export async function parseFeed({
  stdout,
  root,
  name,
}: {
  readonly stdout: string;
  readonly root: string;
  readonly name: string;
},): Promise<ParsedFeed> {
  await writeFile(
    join(
      root,
      'runs',
      `${name}-feed.jsonl`,
    ),
    stdout,
    'utf8',
  );
  /**
   Parsed events, in the order pi printed them.
   */
  const events: Readonly<Record<string, unknown>>[] = [];
  /**
   Lines that were not events.
   */
  const unparsed: string[] = [];
  for (const line of stdout.split('\n',)) {
    if (line.trim()
      .length
      === 0) {
      continue;
    }
    try {
      /**
       Line parsed as JSON, before its shape is known.
       */
      const parsed: unknown = JSON.parse(
        line,
      );
      if (isRecord(parsed,))
        events.push(parsed,);
      else
        unparsed.push(line,);
    }
    catch (error) {
      unparsed.push(`${line} (${caughtValueText(error,)})`,);
    }
  }
  return {
    events,
    unparsed,
  };
}

/**
 The assistant messages one run reported.

 @param run - run to read

 @returns each assistant message the run reported, in order

 @example
 ```typescript
 const messages = assistantMessagesOf({ run });
 ```
 */
export function assistantMessagesOf({
  run,
}: {
  readonly run: PiRun;
},): Readonly<Record<string, unknown>>[] {
  return run.events
    .flatMap(function assistantMessageOf(event,) {
      if (event.type !== 'message_end')
        return [];
    /**
     Message the event carried.
     */
      const {message} = event;
      if (!isRecord(message,))
        return [];
      if (message.role !== 'assistant')
        return [];
      return [message,];
    },);
}

/**
 The waits pi announced before its own automatic retry attempts.

 @param run - run to read

 @returns the announced waits, in order

 @example
 ```typescript
 const waits = retryWaitsOf({ run });
 ```
 */
export function retryWaitsOf({
  run,
}: {
  readonly run: PiRun;
},): number[] {
  return run.events
    .flatMap(function waitOf(event,) {
      if (event.type !== 'auto_retry_start')
        return [];
    /**
     Wait pi announced before the attempt.
     */
      const wait = event.delayMs;
      return (typeof wait) === 'number' ? [wait,] : [];
    },);
}

/**
 The session files one run left behind, read whole.

 A session is where pi stores the messages it built,
 which is what a reader of a run inspects afterwards.

 @param folder - folder to search, which must exist;
 start at the run's throwaway home

 @throws when the folder cannot be read, since the caller starts at a folder it
 created itself

 @returns every session file's text, in the order the files were found

 @example
 ```typescript
 const sessions = await sessionTextsUnder({ folder: home });
 ```
 */
export async function sessionTextsUnder({
  folder,
}: {
  readonly folder: string;
},): Promise<string[]> {
  /**
   Entries in the folder being read,
   which the caller starts at a folder the run itself created.
   */
  const names = await readdir(
    folder,
    { withFileTypes: true, },
  );
  /**
   Texts of the session files found directly in this folder.
   */
  const here = await Promise.all(names
    .filter(function isSessionFile(entry,) {
      return entry.isFile()
        && entry.name
          .endsWith('.jsonl',);
    },)
    .map(function textOf(entry,) {
      return readFile(
        join(
          folder,
          entry.name,
        ),
        'utf8',
      );
    },),);
  /**
   Texts of the sessions found in the folders below this one.
   */
  const below = await Promise.all(names
    .filter(function isFolder(entry,) {
      return entry.isDirectory();
    },)
    .map(function textsOf(entry,) {
      return sessionTextsUnder({
        folder: join(
          folder,
          entry.name,
        ),
      },);
    },),);
  return [
    ...here,
    ...below.flat(),
  ];
}

//endregion Reading a feed

//region Running pi

/**
 Run pi once against the stand-in, keeping its feed and its error output.

 The run is bounded like every child this harness starts:
 a pi that outlives its bound is signalled again and then killed,
 and the result says it reached the bound,
 so a run that never ends fails the proof instead of holding it.

 @param request - the run's folder, files, bundle, home and prompt

 @returns what the run printed, and how it ended

 @example
 ```typescript
 const run = await runPi({ root, name: 'retried', piCli, extension, home, agentDir, provider, model, prompt });
 ```
 */
export async function runPi(request: PiRunRequest,): Promise<PiRun> {
  /**
   Fields of the request this run reads.
   */
  const {
    root,
    name,
    piCli,
    extension,
    home,
    agentDir,
    provider,
    model,
    prompt,
  } = request;
  /**
   Directory the run works in, outside every checkout, so no project file is
   loaded beside the prompt.
   */
  const cwd = join(
    root,
    'runs',
    `${name}-working-directory`,
  );
  await mkdir(
    cwd,
    { recursive: true, },
  );
  /**
   What the bounded run left behind.
   */
  const child = await runBoundedChild({
    file: process.execPath,
    args: [
      piCli,
      '--mode',
      'json',
      '--offline',
      '--no-extensions',
      '--extension',
      extension,
      '--no-tools',
      '--no-skills',
      '--no-prompt-templates',
      '--no-context-files',
      '--no-mcp',
      '--model',
      `${provider}/${model}`,
      '--print',
      prompt,
    ],
    cwd,
    env: {
      PATH: process.env
        .PATH
        ?? '',
      HOME: home,
      PI_CODING_AGENT_DIR: agentDir,
      PI_OFFLINE: '1',
      PI_TELEMETRY: '0',
      PI_SKIP_VERSION_CHECK: '1',
    },
    boundMilliseconds: RUN_TIMEOUT_MILLISECONDS,
  },);
  await writeFile(
    join(
      root,
      'runs',
      `${name}-stderr.log`,
    ),
    child.stderr,
    'utf8',
  );
  /**
   Session events the run printed.
   */
  const parsed = await parseFeed({
    stdout: child.stdout,
    root,
    name,
  },);
  return {
    events: parsed.events,
    ...(child.exitCode === undefined ? {} : { exitCode: child.exitCode, }),
    timedOut: child.timedOut,
  };
}

//endregion Running pi
