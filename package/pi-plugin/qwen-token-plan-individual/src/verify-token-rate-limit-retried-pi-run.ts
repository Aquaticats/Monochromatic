/**
 One pi run of the passing-token-rate-limit proof:
 the command both runs start, the feed each one prints, and the reading of what
 it settled on.

 @module
 */

import {
  spawn,
  type ChildProcess,
} from 'node:child_process';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';
import { once, } from 'node:events';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Constants

/**
 Bound on one pi run, so a run that never settles fails the proof instead of
 holding it.
 */
const RUN_TIMEOUT_MILLISECONDS = 120_000;

/**
 Text an abort shows up as, which is how a run reaches its bound.
 */
const ABORT_ERROR_NAME = 'AbortError';

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

//endregion Reading a feed

//region Running pi

/**
 Wait for one child to end, reporting a kill at the run's bound as such.

 @param child - child the run started

 @returns the exit status when the child ended by itself, or that it was killed

 @example
 ```typescript
 const ending = await endingOf({ child });
 ```
 */
async function endingOf({
  child,
}: {
  readonly child: ChildProcess;
},): Promise<{
  readonly exitCode?: number;
  readonly timedOut: boolean;
}> {
  try {
    await once(
      child,
      'exit',
    );
    /**
     Exit status the child reported, which it leaves absent when a signal
     ended it.
     */
    const {exitCode} = child;
    return {
      ...(exitCode === null ? {} : { exitCode, }),
      timedOut: false,
    };
  } catch (error) {
    return {
      timedOut: (Error.isError(error,))
        && (error.name === ABORT_ERROR_NAME),
    };
  }
}

/**
 Run pi once against the stand-in, keeping its feed and its error output.

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
   Child process for this run.
   */
  const child = spawn(
    process.execPath,
    [
      piCli,
      '--mode',
      'json',
      '--offline',
      '--no-session',
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
    {
      cwd,
      // Standard input is refused rather than piped: pi reads a piped standard
      // input as the prompt, and a pipe nobody closes would hold the run open.
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ] as const,
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
      signal: AbortSignal.timeout(RUN_TIMEOUT_MILLISECONDS,),
    },
  );
  /**
   Standard output the run prints, in order.
   */
  const stdoutChunks: Buffer[] = [];
  /**
   Standard error the run prints, in order.
   */
  const stderrChunks: Buffer[] = [];
  child.stdout
    .on(
    'data',
    function collectStdout(chunk: Buffer,) {
      stdoutChunks.push(chunk,);
    },
    );
  child.stderr
    .on(
    'data',
    function collectStderr(chunk: Buffer,) {
      stderrChunks.push(chunk,);
    },
    );
  /**
   How the child ended.
   */
  const ending = await endingOf({ child, },);
  await writeFile(
    join(
      root,
      'runs',
      `${name}-stderr.log`,
    ),
    Buffer.concat(stderrChunks,)
      .toString('utf8',),
    'utf8',
  );
  /**
   Session events the run printed.
   */
  const parsed = await parseFeed({
    stdout: Buffer.concat(stdoutChunks,)
      .toString('utf8',),
    root,
    name,
  },);
  return {
    events: parsed.events,
    ...(ending.exitCode === undefined ? {} : { exitCode: ending.exitCode, }),
    timedOut: ending.timedOut,
  };
}

//endregion Running pi
