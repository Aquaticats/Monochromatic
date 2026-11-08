/**
 The stand-in Pi's log and the state its scenarios share: which scenario runs, what it has seen, and the lines a test waits for
 instead of sleeping.

 @module
 */

import { appendFile, } from 'node:fs/promises';
import { env, } from 'node:process';

/**
 Where a scenario writes the lines a test waits for.
 */
export const LOG: string = env.STAND_IN_PI_LOG ?? '/dev/null';

/**
 The behaviour chosen by the test.
 */
export const scenario: string = env.STAND_IN_PI_SCENARIO ?? 'normal';

/**
 What the stand-in has seen and is doing.
 */
export const seen: {
  prompts: number;
  answering: number;
} = {
  /**
   How many prompts and steers have arrived.
   */
  prompts: 0,

  /**
   How many answers are still being prepared, so a command that arrives meanwhile is an overlap.
   */
  answering: 0,
};

/**
 Writes a line a test can wait for, saying which point the stand-in has reached.

 @param line - line to write

 @example
 ```ts
 await writeLogLine('marker agent_end',);
 ```
 */
export async function writeLogLine(line: string,): Promise<void> {
  await appendFile(
    LOG,
    `${line}\n`,
  );
}
