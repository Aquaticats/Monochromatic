/**
 The retrying start: it asks the pool gate again and again until a pool has room or its own deadline passes, so a run waits for
 the slot it should fill instead of being refused once.

 A shorter `--period-seconds` takes a freed slot sooner, which is how the queue's order is kept.

 @module
 */

import type { CommandOutput, } from './run-command.ts';
import { NO_SLOT_EXIT_STATUS, } from './pool-gate.ts';
import {
  MODEL_OF_A_RUN_NAMING_NONE,
  PROVIDER_OF_A_RUN_NAMING_NONE,
  poolNamed,
} from './pool-limits.ts';

/**
 How many kibibytes of what the gate printed are kept for a reader.
 */
const GATE_OUTPUT_KIBIBYTES = 64;

/**
 How many characters one kibibyte of what the gate printed is.
 */
const CHARACTERS_PER_KIBIBYTE = 1_024;

/**
 How much of what the gate printed is kept for a reader. Its answer names the pools and why it started nothing, and it carries the
 launcher's own stack when the launcher failed, whose message hangs above frames, so a bound smaller than the batch's kept bound
 could cut the one line that says why.
 */
export const GATE_OUTPUT_TAIL_CHARACTERS: number = GATE_OUTPUT_KIBIBYTES * CHARACTERS_PER_KIBIBYTE;

/**
 The word the pool gate's options name the run with, which a waiting line repeats.
 */
const RUN_NAME_OPTION = '--name=';

/**
 The word the pool gate's options name a run's provider with.
 */
const PROVIDER_OPTION = '--provider=';

/**
 The word the pool gate's options name a run's model with.
 */
const MODEL_OPTION = '--model=';

/**
 The name a waiting line gives a run whose words name none.
 */
const UNNAMED_RUN = 'a run';

/**
 The pool a waiting line names for a run that asks for whichever pool has room.
 */
const ANY_POOL_WITH_ROOM = 'the first pool with room';

/**
 What one retrying start ends with.
 */
export type RetriedStart = {
  /**
   The status the gate ended with on the last attempt, which the calling command exits with.
   */
  readonly status: number;

  /**
   What the calling command prints.
   */
  readonly text: string;
};

/**
 Asks the pool gate for a slot again and again until it starts the run or the deadline passes.

 Every wait is the shorter of the period and what the deadline leaves, so this command ends on its own even when no slot ever
 frees.

 @param attempt - one ask of the pool gate, which a test replaces

 @param now - where the wait reads the time

 @param wait - how the wait passes time

 @param deadline - when asking stops, as milliseconds since the epoch

 @param periodMilliseconds - how long to wait between asks

 @param words - the words the pool gate is asked with, which the deadline line repeats

 @returns the status to exit with, and what to print

 @example
 ```ts
 const outcome = await retryStartUntilAPoolHasRoom({ attempt, now: Date.now, wait, deadline, periodMilliseconds: 30_000, words: ['--name=a',], },);
 ```
 */
export async function retryStartUntilAPoolHasRoom({
  attempt,
  now,
  wait,
  deadline,
  periodMilliseconds,
  words,
  report,
}: {
  readonly attempt: () => Promise<CommandOutput>;
  readonly now: () => number;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly deadline: number;
  readonly periodMilliseconds: number;
  readonly words: readonly string[];
  readonly report: (line: string) => void;
},): Promise<RetriedStart> {
  /**
   When the wait began, which every line it reports repeats.
   */
  const beganAt = now();
  /**
   How many times the pool gate has been asked so far, so a reader can tell a slow ask from a stuck one.
   */
  const asking: { asks: number } = { asks: 0, };
  /* oxlint-disable eslint/no-await-in-loop -- one ask, then a wait, then the next ask: this command exists to keep asking while
     the runs already going fill the pools it reads. See doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     What the gate answered this time.
     */
    const asked = await attempt();
    asking.asks += 1;
    if (asked.status !== NO_SLOT_EXIT_STATUS) {
      return {
        status: asked.status,
        text: `${asked.stdout}${asked.stderr}`.slice(-GATE_OUTPUT_TAIL_CHARACTERS,),
      };
    }
    /**
     When this ask was answered, which is both how much of the deadline is left and what the line reports.
     */
    const askedAt = now();
    /**
     How much of the deadline is left.
     */
    const left = deadline - askedAt;
    // A wait that prints nothing looks like a start that never happened: every ask that found no slot says which run waits, for
    // which pool, what the gate counted against each limit, and both ends of the wait.
    report(waitingLineOf({
      words,
      asked,
      ask: asking.asks,
      askedAt,
      beganAt,
      deadline,
    },),);
    if (left <= 0) {
      break;
    }
    await wait(Math.min(
      periodMilliseconds,
      left,
    ),);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return {
    status: NO_SLOT_EXIT_STATUS,
    text: `no slot freed before the deadline: ${words.join(' ',)}\n`,
  };
}

/**
 Reads the value of one of the pool gate's options, when the words carry it.

 @param words - the words the pool gate is asked with

 @param option - the option as it is written, with its `--` and its `=`

 @returns the value, or nothing when no word names it

 @example
 ```ts
 const name = wordValueOf({ words: ['--name=a',], option: '--name=', },);
 ```
 */
function wordValueOf({
  words,
  option,
}: {
  readonly words: readonly string[];
  readonly option: string;
}): { readonly value?: string } {
  /**
   The word that names the option.
   */
  const written = words.find(function isTheOption(word,) {
    return word.startsWith(option,);
  },);
  return (written === undefined) ? {} : { value: written.slice(option.length,), };
}

/**
 Says which run a wait is for and which pool it asks for.

 A run whose words name no provider or model takes the first pool with room, which is what the pool gate does with it; a run
 naming one names the pool that provider and model share, and the counts the line carries say how full it is.

 @param words - the words the pool gate is asked with

 @returns the run's name as the gate's words write it, and the pool it asks for

 @example
 ```ts
 const asking = askingOf({ words: ['--name=a', '--provider=neuralwatt', '--model=deepseek-v4.1-flash'], },);
 ```
 */
function askingOf({ words, }: { readonly words: readonly string[] }): {
  readonly run: string;
  readonly pool: string
} {
  /**
   The provider the run's words name, or the one a run that names none gets.
   */
  const provider = wordValueOf({
    words,
    option: PROVIDER_OPTION,
  },);
  /**
   The model the run's words name, or the one a run that names none gets.
   */
  const model = wordValueOf({
    words,
    option: MODEL_OPTION,
  },);
  /**
   The pool the run's own provider and model name, which is none when they name none.
   */
  const pool = poolNamed({
    provider: provider.value ?? PROVIDER_OF_A_RUN_NAMING_NONE,
    model: model.value ?? MODEL_OF_A_RUN_NAMING_NONE,
  },);
  /**
   The name the wait reports the run under.
   */
  const run = wordValueOf({
    words,
    option: RUN_NAME_OPTION,
  },);
  /**
   The name the line reports the run under.
   */
  const named = run.value ?? UNNAMED_RUN;
  if (pool.holds !== 'a-pool') {
    return {
      run: named,
      pool: ANY_POOL_WITH_ROOM,
    };
  }
  /**
   The pool the run named, which is only ever the one the review asks for.
   */
  const namedPool = pool.pool;
  return {
    run: named,
    pool: namedPool.takesDefaultRuns ? ANY_POOL_WITH_ROOM : namedPool.name,
  };
}

/**
 Builds the line one ask that found no slot reports.

 It carries the pool the run asks for and the pool gate's own counts, so the number and the limit of every pool are the gate's
 answer rather than a second count taken here, and both ends of the wait so a reader knows how long to keep looking.

 @param words - the words the pool gate is asked with

 @param asked - the gate's answer to this ask

 @param ask - which ask this was, counting from one

 @param askedAt - when the ask was answered, as milliseconds since the epoch

 @param beganAt - when the wait began, as milliseconds since the epoch

 @param deadline - when asking stops, as milliseconds since the epoch

 @returns the line

 @example
 ```ts
 const line = waitingLineOf({ words: ['--name=a',], asked: { status: 3, stdout: 'NO SLOT: x 1/1', stderr: '' }, ask: 1, askedAt: 0, beganAt: 0, deadline: 1_000, },);
 ```
 */
export function waitingLineOf({
  words,
  asked,
  ask,
  askedAt,
  beganAt,
  deadline,
}: {
  readonly words: readonly string[];
  readonly asked: CommandOutput;
  readonly ask: number;
  readonly askedAt: number;
  readonly beganAt: number;
  readonly deadline: number;
},): string {
  /**
   The run that waits and the pool it asks for.
   */
  const asking = askingOf({ words, },);
  /**
   What the gate said, on either stream, as one line.
   */
  const said = `${asked.stdout}${asked.stderr}`.trim()
    .split('\n',)
    .join(' ',);
  return `WAITING: ${asking.run} asks for ${asking.pool}; waiting since ${new Date(beganAt,)
    .toISOString()}, gives up at ${new Date(deadline,)
    .toISOString()}; ask ${String(ask,)} at ${new Date(askedAt,)
    .toISOString()}: ${said}\n`;
}
