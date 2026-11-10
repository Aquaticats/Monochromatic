/**
 Reading one option out of what the parser produced, so every value a landing uses is read the same way and a wrong one is
 refused with a sentence a caller can act on.

 @module
 */

import {
  COMMIT_PAUSE_SECONDS,
  COMMIT_TIMEOUT_SECONDS,
  COMMIT_TRIES,
  GIT_TIMEOUT_SECONDS,
  MILLISECONDS_PER_SECOND,
} from './constants.ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';
import { PROGRAM_NAME, } from './usage.ts';

/**
 How long each command a landing starts may run, and how often it tries again.
 */
export type Bounds = {
  /**
   How many times a merge's commit is tried before the landing gives up.
   */
  readonly tries: number;

  /**
   Milliseconds between two tries of a merge's commit.
   */
  readonly pauseMilliseconds: number;

  /**
   Milliseconds one commit may take.
   */
  readonly commitTimeoutMilliseconds: number;

  /**
   Milliseconds one Git command may take.
   */
  readonly gitTimeoutMilliseconds: number;
};

/**
 Reads one option that holds text.

 @param values - the parsed options

 @param name - the option's name

 @returns its text, or why it holds none

 @example
 ```ts
 const branch = textOption({ values, name: 'branch', },);
 ```
 */
export function textOption({
  values,
  name,
}: {
  readonly values: Readonly<Record<string, unknown>>;
  readonly name: string;
},): Looked<string> {
  /**
   What the parser put there.
   */
  const value = values[name];
  return (typeof value) === 'string'
    ? found(value,)
    : missing(`--${name}=<value> was not given`,);
}

/**
 Reads one option that holds a list of texts, each once.

 @param values - the parsed options

 @param name - the option's name

 @returns its texts, empty when it was not given

 @example
 ```ts
 const tasks = textsOption({ values, name: 'task', },);
 ```
 */
export function textsOption({
  values,
  name,
}: {
  readonly values: Readonly<Record<string, unknown>>;
  readonly name: string;
},): readonly string[] {
  /**
   What the parser put there.
   */
  const value = values[name];
  return Array.isArray(value,)
    ? [...new Set(value.filter(function isText(entry: unknown,) {
      return (typeof entry) === 'string';
    },) as readonly string[],),]
    : [];
}

/**
 Reads one option that a subcommand cannot do without.

 @param values - the parsed options

 @param name - the option's name

 @returns its text

 @throws LandingConfigurationError when it was not given, or was given nothing

 @example
 ```ts
 const message = requiredTextOption({ values, name: 'message', },);
 ```
 */
export function requiredTextOption({
  values,
  name,
}: {
  readonly values: Readonly<Record<string, unknown>>;
  readonly name: string;
},): string {
  /**
   What the option holds.
   */
  const value = textOption({
    values,
    name,
  },);
  if ((!value.found) || (value.value === '')) {
    throw new LandingConfigurationError(
      `${PROGRAM_NAME} needs --${name}=<value> here; run it with --help to see the subcommands`,
    );
  }
  return value.value;
}

/**
 Reads the whole-number value of one option.

 @param values - the parsed options

 @param name - the option's name

 @param smallest - the smallest value it may hold

 @param fallback - the value used when it was not given

 @returns the number

 @throws LandingConfigurationError when the text is not a whole number at least the smallest

 @example
 ```ts
 const tries = wholeNumberOf({ values, name: 'commit-tries', smallest: 1, fallback: 15, },);
 ```
 */
function wholeNumberOf({
  values,
  name,
  smallest,
  fallback,
}: {
  readonly values: Readonly<Record<string, unknown>>;
  readonly name: string;
  readonly smallest: number;
  readonly fallback: number;
},): number {
  /**
   What the option holds.
   */
  const text = textOption({
    values,
    name,
  },);
  if (!text.found) return fallback;
  /**
   What the text holds as a number.
   */
  const value = Number(text.value,);
  if ((!Number.isInteger(value,)) || (value < smallest)) {
    throw new LandingConfigurationError(
      `--${name}=${text.value}: this must be a whole number of at least ${String(smallest,)}`,
    );
  }
  return value;
}

/**
 A value in seconds as milliseconds.

 @param values - the parsed options

 @param name - the option's name

 @param fallbackSeconds - the value used when it was not given, in seconds

 @returns the milliseconds

 @throws LandingConfigurationError when the text is not a number of seconds at least zero

 @example
 ```ts
 const pause = millisecondsOfSeconds({ values, name: 'commit-pause-seconds', fallbackSeconds: 20, },);
 ```
 */
function millisecondsOfSeconds({
  values,
  name,
  fallbackSeconds,
}: {
  readonly values: Readonly<Record<string, unknown>>;
  readonly name: string;
  readonly fallbackSeconds: number;
},): number {
  /**
   What the option holds.
   */
  const text = textOption({
    values,
    name,
  },);
  if (!text.found) return fallbackSeconds * MILLISECONDS_PER_SECOND;
  /**
   What the text holds as a number.
   */
  const value = Number(text.value,);
  if ((!Number.isFinite(value,)) || (value < 0)) {
    throw new LandingConfigurationError(
      `--${name}=${text.value}: this must be a number of seconds, zero or more`,
    );
  }
  return value * MILLISECONDS_PER_SECOND;
}

/**
 Reads the bounds every subcommand accepts.

 @param values - the parsed options

 @returns what bounds this run's commands

 @example
 ```ts
 const bounds = boundsOf({ values, },);
 ```
 */
export function boundsOf({ values, }: {
  readonly values: Readonly<Record<string, unknown>>;
},): Bounds {
  return {
    tries: wholeNumberOf({
      values,
      name: 'commit-tries',
      smallest: 1,
      fallback: COMMIT_TRIES,
    },),
    pauseMilliseconds: millisecondsOfSeconds({
      values,
      name: 'commit-pause-seconds',
      fallbackSeconds: COMMIT_PAUSE_SECONDS,
    },),
    commitTimeoutMilliseconds: millisecondsOfSeconds({
      values,
      name: 'commit-timeout-seconds',
      fallbackSeconds: COMMIT_TIMEOUT_SECONDS,
    },),
    gitTimeoutMilliseconds: millisecondsOfSeconds({
      values,
      name: 'git-timeout-seconds',
      fallbackSeconds: GIT_TIMEOUT_SECONDS,
    },),
  };
}
