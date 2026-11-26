/**
 The alert streak: one desktop alert per hour while a unit keeps failing, and one
 recovery alert when it runs cleanly again.

 The streak is remembered per unit in a file under the state directory, which survives
 logout (the previous script remembered nothing, so it alerted once per run). The file
 is written only after `notify-send` accepted the alert, so a failed notification does
 not silence the next attempt for an hour.

 @module
 */

import {
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { ALERT_STATE_DIRECTORY_NAME, } from './units.ts';

/**
 Seconds in one minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in one second.
 */
const MS_PER_SECOND = 1_000;

/**
 Milliseconds in one minute.
 */
const MS_PER_MINUTE = MS_PER_SECOND * SECONDS_PER_MINUTE;

/**
 Minutes between two failure alerts of one unit.
 */
const MINUTES_PER_ALERT_GAP = 60;

/**
 Shortest gap between two failure alerts of one unit, in milliseconds.

 An hour matches the old hourly run: a failure that persists is announced again at
 about the pace a person expects from a background sync, and a unit that fails every
 five minutes does not fill the notification history.
 */
export const ALERT_QUIET_WINDOW_MS: number = MS_PER_MINUTE * MINUTES_PER_ALERT_GAP;

/**
 What the state file records about a unit that is failing.
 */
export type Streak = {
  readonly lastAlertAtMs: number;
};

/**
 The state file's contents, or the reason it could not be used.

 An absent file is not a problem: it means the unit has no open streak.
 */
export type StreakLookup = {
  readonly streak?: Streak;
  readonly problem?: string;
};

/**
 What to do with a failure.
 */
export type FailureDecision =
  | 'alert'
    | 'suppressed';

/**
 What to do with a success.
 */
export type RecoveryDecision =
  | 'recovery-alert'
    | 'nothing';

/**
 Decides whether a failure deserves an alert.

 @param lookup - the unit's streak lookup, whose `streak` is absent when none is open

 @param nowMs - current time in milliseconds since the Unix epoch

 @returns `alert` for a first failure or one an hour or more after the last alert,
   `suppressed` for a repeat inside the quiet window

 @example
 ```ts
 decideFailure({ lookup: { streak: { lastAlertAtMs: 0, }, }, nowMs: 60 * 60 * 1000, },); // 'alert'
 decideFailure({ lookup: { streak: { lastAlertAtMs: 0, }, }, nowMs: 1, },); // 'suppressed'
 ```
 */
export function decideFailure({
  lookup,
  nowMs,
}: Readonly<{
  lookup: Readonly<StreakLookup>;
  nowMs: number;
}>): FailureDecision {
  if (lookup.streak === undefined) return 'alert';
  return (nowMs
    - lookup.streak
      .lastAlertAtMs) >= ALERT_QUIET_WINDOW_MS ? 'alert' : 'suppressed';
}

/**
 Decides whether a clean run closes an open streak.

 @param lookup - the unit's streak lookup, whose `streak` is absent when none is open

 @returns `recovery-alert` when a streak is open, `nothing` otherwise, so the
   `OnSuccess=` start that the five-minute timer causes every five minutes stays free

 @example
 ```ts
 decideRecovery({ lookup: { streak: { lastAlertAtMs: 0, }, }, },); // 'recovery-alert'
 decideRecovery({ lookup: {}, },); // 'nothing'
 ```
 */
export function decideRecovery({ lookup, }: Readonly<{ lookup: Readonly<StreakLookup>; }>,): RecoveryDecision {
  return lookup.streak === undefined ? 'nothing' : 'recovery-alert';
}

/**
 Folder that holds one streak file per unit.

 @param home - home folder of the user the alerts describe

 @returns the absolute state folder, whether or not it exists yet

 @example
 ```ts
 stateDirectory({ home: '/home/example', },); // '/home/example/.local/state/rclone-bisync-garage-alert'
 ```
 */
export function stateDirectory({ home, }: Readonly<{ home: string; }>,): string {
  return join(
    home,
    '.local',
    'state',
    ALERT_STATE_DIRECTORY_NAME,
  );
}

/**
 Streak file of one unit.

 @param home - home folder of the user the alerts describe

 @param unit - unit name, which is also the file name

 @returns the absolute path of that unit's streak file

 @example
 ```ts
 streakFilePath({ home: '/home/example', unit: 'a.service', },); // '.../rclone-bisync-garage-alert/a.service.json'
 ```
 */
export function streakFilePath({
  home,
  unit,
}: Readonly<{
  home: string;
  unit: string
}>,): string {
  return join(
    stateDirectory({ home, },),
    `${unit}.json`,
  );
}

/**
 Whether a caught read failure means the file is simply absent.

 @param value - caught value

 @returns true for an error whose code is `ENOENT`

 @example
 ```ts
 isMissingFileError(Object.assign(new Error('x',), { code: 'ENOENT', },)); // true
 ```
 */
function isMissingFileError(value: unknown,): boolean {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('code' in value)
    && (value.code === 'ENOENT');
}

/**
 Parses JSON without throwing.

 @param text - text to parse

 @returns the parsed value, or the reason the text is not JSON
 */
function parseJson(text: string,): {
  readonly value?: unknown;
  readonly problem?: string
} {
  try {
    return { value: JSON.parse(text,), };
  } catch (error) {
    return { problem: `the text does not hold JSON: ${String(error,)}`, };
  }
}

/**
 Reads a streak file's text into a streak.

 @param text - file contents

 @returns the streak, or the reason the text does not describe one
 */
function parseStreak(text: string,): StreakLookup {
  /**
   Parsed file contents, of unknown shape.
   */
  const json = parseJson(text,);
  if (json.problem !== undefined) return { problem: `the streak file ${json.problem}`, };
  /**
   Parsed file contents, still of unknown shape.
   */
  const parsed = json.value;
  if (((typeof parsed) !== 'object') || (parsed === null)
    || (!('lastAlertAt' in parsed))) {
    return { problem: 'the streak file does not hold an object with a lastAlertAt field', };
  }
  /**
   ISO 8601 timestamp of the last alert.
   */
  const {lastAlertAt} = parsed;
  if ((typeof lastAlertAt) !== 'string') {
    return { problem: "the streak file's lastAlertAt is not a timestamp string", };
  }
  /**
   Milliseconds since the Unix epoch, `NaN` when the text is not a timestamp.
   */
  const lastAlertAtMs = Date.parse(lastAlertAt,);
  if (Number.isNaN(lastAlertAtMs,)) {
    return { problem: `the streak file lastAlertAt "${lastAlertAt}" is not a timestamp`, };
  }
  return { streak: { lastAlertAtMs, }, };
}

/**
 Reads a unit's open streak.

 @param home - home folder of the user the alerts describe

 @param unit - unit whose streak is read

 @returns the streak, nothing when there is no file, or a problem when the file is
   unreadable or unrecognizable; a problem is treated as no streak, so the alert still
   goes out

 @example
 ```ts
 await readStreak({ home: '/home/example', unit: 'rclone-bisync-garage.service', },);
 ```
 */
export async function readStreak({
  home,
  unit,
}: Readonly<{
  home: string;
  unit: string;
}>): Promise<StreakLookup> {
  try {
    /**
     File contents of this unit's streak file.
     */
    const text = await readFile(
      streakFilePath({
        home,
        unit,
      },),
      'utf8',
    );
    return parseStreak(text,);
  } catch (error) {
    if (isMissingFileError(error,)) return {};
    return { problem: `could not read ${streakFilePath({
      home,
      unit,
    },)}: ${String(error,)}`, };
  }
}

/**
 Records an alert, opening or refreshing the unit's streak.

 @param home - home folder of the user the alerts describe

 @param unit - unit whose streak is written

 @param streak - the alert time to record

 @example
 ```ts
 await writeStreak({ home, unit, streak: { lastAlertAtMs: Date.now(), }, },);
 ```
 */
export async function writeStreak({
  home,
  unit,
  streak,
}: Readonly<{
  home: string;
  unit: string;
  streak: Streak;
}>): Promise<void> {
  await mkdir(
    stateDirectory({ home, },),
    {
      recursive: true,
      mode: 0o700,
    },
  );
  /**
   File contents: the unit for a reader, and the alert time that decides the next one.
   */
  const text = `${JSON.stringify(
    {
      unit,
      lastAlertAt: new Date(streak.lastAlertAtMs,).toISOString(),
    },
    undefined,
    2,
  )}\n`;
  await writeFile(
    streakFilePath({
      home,
      unit,
    },),
    text,
    'utf8',
  );
}

/**
 Closes a unit's streak, after its recovery alert went out.

 @param home - home folder of the user the alerts describe

 @param unit - unit whose streak is removed

 @example
 ```ts
 await removeStreak({ home, unit, },);
 ```
 */
export async function removeStreak({
  home,
  unit,
}: Readonly<{
  home: string;
  unit: string;
}>): Promise<void> {
  await rm(
    streakFilePath({
      home,
      unit,
    },),
    { force: true, },
  );
}
