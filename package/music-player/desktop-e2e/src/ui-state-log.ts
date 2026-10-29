/**
 The reader of the app's log file: it follows the file, keeps the `ui_state` lines, and waits for the state a key
 press should cause.
 
 @example
 ```ts
 import { createUiStateLog, } from './ui-state-log.ts';
 
 const log = createUiStateLog({ logPath, },);
 const state = await log.last();
 ```
 
 @packageDocumentation
 */

import {
  open,
  readFile,
  stat,
} from 'node:fs/promises';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { stripAnsi, } from './ansi.ts';
import {
  CheckFailureError,
  expect,
} from './errors.ts';
import {
  countShortcutUses,
  waylandDisplayOf,
} from './log-scan.ts';
import { pollUntilDeadline, } from './poll.ts';
import {
  STATE_POLL_MS,
  STATE_TIMEOUT_MS,
} from './timing.ts';
import {
  parseUiStateLine,
  type UiState,
} from './ui-state.ts';

/**
 How many of the latest states a timeout message shows.
 */
const RECENT_STATES = 3;

/**
 What a check says to wait for a state.
 */
export type WaitForState = {
  /**
   Only states with a larger sequence number count; every state when absent.
   */
  readonly after?: number;
  /**
   True for the wanted state.
   */
  readonly predicate: (state: UiState,) => boolean;
  /**
   Milliseconds to wait before giving up.
   */
  readonly timeoutMs?: number;
  /**
   Words for the timeout message.
   */
  readonly what: string;
};

/**
 The reader's operations.
 */
export type UiStateLog = {
  /**
   Reads the whole log without color sequences, line by line.
   */
  readonly lines: () => Promise<readonly string[]>;
  /**
   Reads new log lines and returns the latest state; fails when the app has logged none.
   */
  readonly last: () => Promise<UiState>;
  /**
   Reads new log lines and returns the sequence number of the latest state, or 0 when there is none.
   */
  readonly seq: () => Promise<number>;
  /**
   Every state read so far, in order. Call `last` or `seq` first to read new lines.
   */
  readonly states: () => readonly UiState[];
  /**
   The last characters of the log, for failure messages.
   */
  readonly tail: (chars: number,) => Promise<string>;
  /**
   How many times the app reported using a shortcut action, or any action when absent.
   */
  readonly used: (action?: string,) => Promise<number>;
  /**
   Waits for a state newer than a sequence number that satisfies a predicate.
   */
  readonly waitState: (wait: WaitForState,) => Promise<UiState>;
  /**
   The nested Wayland display name from the compositor's start-up lines, or the empty string.
   */
  readonly waylandDisplay: () => Promise<string>;
};

/**
 Creates a reader for one log file.
 
 @param logPath - the file the unit writes the app's output to
 
 @returns the reader
 
 @example
 ```ts
 const log = createUiStateLog({ logPath: '/work/wide.log', },);
 ```
 */
export function createUiStateLog(
  {
    logPath,
  }: {
    readonly logPath: string;
  },
): UiStateLog {
  /**
   Tagged logger naming the reader.
   */
  const rl = tagged({
    tag: createUiStateLog.name,
    l: logger,
  },);

  /**
   States read so far.
   */
  const states: UiState[] = [];

  /**
   How far the file has been read, and the text of an unfinished last line.
   */
  const cursor = {
    offset: 0,
    pending: '',
  };

  /**
   Decoder that keeps a multi-byte character cut by a read boundary until its rest arrives.
   */
  const decoder = new TextDecoder();

  /**
   Reads the lines the file gained since the last read and keeps their states.
   */
  async function pump(): Promise<void> {
    /**
     Current size of the file.
     */
    const { size, } = await stat(logPath,);
    if (size <= cursor.offset)
      return;
    /**
     Read handle of the log, closed when the read ends.
     */
    await using handle = await open(
      logPath,
      'r',
    );

    /**
     Room for the new bytes.
     */
    const buffer = Buffer.alloc(size - cursor.offset,);

    /**
     How many bytes the read returned.
     */
    const { bytesRead, } = await handle.read(
      buffer,
      0,
      buffer.length,
      cursor.offset,
    );
    cursor.offset += bytesRead;
    cursor.pending += decoder.decode(
      buffer.subarray(
        0,
        bytesRead,
      ),
      { stream: true, },
    );

    /**
     Complete lines, then the unfinished one.
     */
    const complete = cursor.pending
      .split('\n',);
    cursor.pending = complete.pop() ?? '';
    for (const line of complete)
      states.push(...parseUiStateLine(line,),);
  }

  /**
   Reads the whole log without color sequences.
   
   @returns the lines; none when the file cannot be read
   */
  async function lines(): Promise<readonly string[]> {
    try {
      return stripAnsi(await readFile(
        logPath,
        'utf8',
      ),)
        .split('\n',);
    }
    catch (error: unknown) {
      rl.debug(`could not read ${logPath}: ${String(error,)}`,);
      return [];
    }
  }

  /**
   Reads new lines and returns the latest state.
   
   @returns the latest state
   
   @throws CheckFailureError when the app has logged no state yet
   */
  async function last(): Promise<UiState> {
    await pump();

    /**
     The latest state, if any.
     */
    const state = states.at(-1,);
    if (state === undefined)
      throw new CheckFailureError('the app has written no ui_state line yet',);
    return state;
  }

  /**
   Reads new lines and returns the latest sequence number.
   
   @returns the sequence number of the latest state; 0 when there is none
   */
  async function seq(): Promise<number> {
    await pump();
    return states.at(-1,)
      ?.seq
      ?? 0;
  }

  /**
   Waits for a wanted state.
   
   @param wanted - what to wait for
   
   @returns the first state newer than `after` that satisfies the predicate
   
   @throws CheckFailureError when the wait times out, with the latest states in the message
   */
  async function waitState(wanted: WaitForState,): Promise<UiState> {
    /**
     The parts of the request, with defaults.
     */
    const {
      predicate,
      what,
      after = -1,
      timeoutMs = STATE_TIMEOUT_MS,
    } = wanted;

    /**
     Tells whether a state is newer than `after` and satisfies the predicate.
     
     @param state - a state read from the log
     
     @returns whether it is the wanted one
     */
    function isWanted(state: UiState,): boolean {
      return (state.seq > after) && predicate(state,);
    }

    /**
     One look at the states read so far.
     
     @returns whether a wanted state is there
     */
    async function look(): Promise<boolean> {
      await pump();
      return states.some(isWanted,);
    }

    /**
     Whether a wanted state arrived in time.
     */
    const arrived = await pollUntilDeadline({
      timeoutMs,
      intervalMs: STATE_POLL_MS,
      check: look,
    },);
    if (arrived)
      return nonNullishOrThrow(states.find(isWanted,),);

    /**
     The latest states, one per line, for the message.
     */
    const recent = states
      .slice(-RECENT_STATES,)
      .map(function describe(state,): string {
        return JSON.stringify(state,);
      },)
      .join('\n   ',);
    throw new CheckFailureError(`timed out waiting for ${what}; last states:\n   ${recent}`,);
  }

  return {
    lines,
    last,
    seq,
    waitState,
    states: function currentStates(): readonly UiState[] {
      return states;
    },
    tail: async function tail(chars,): Promise<string> {
      return (await lines()).join('\n',)
        .slice(-chars,);
    },
    used: async function used(action,): Promise<number> {
      return countShortcutUses({
        lines: await lines(),
        ...action === undefined ? {} : { action, },
      },);
    },
    waylandDisplay: async function waylandDisplay(): Promise<string> {
      return waylandDisplayOf(await lines(),);
    },
  };
}
