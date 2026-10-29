/**
 Name-family classification for retention sweeps. @module
 */

//region Logger-owned names, identified by shape without regular expressions.
/**
 Sentinel for names outside the logger's owned families,
 a unique symbol rather than a nullish value,
 so callers narrow honestly.
 */
export const NOT_LOGGER_OWNED: unique symbol = Symbol('name outside the logger-owned day and log file families');

/**
 Characters of a `YYYY-MM-DD` day key.
 */
export const DAY_KEY_LENGTH = 10;

/**
 Separator of a `YYYY-MM-DD` day key.
 */
const DAY_KEY_SEPARATOR = '-';

/**
 Digit width of the year segment in a `YYYY-MM-DD` day key.
 */
const YEAR_WIDTH = 4;

/**
 Digit width of the month segment in a `YYYY-MM-DD` day key.
 */
const MONTH_WIDTH = 2;

/**
 Digit width of the day segment in a `YYYY-MM-DD` day key.
 */
const DAY_WIDTH = 2;

/**
 Digit widths of the three `YYYY-MM-DD` day-key segments.
 */
const DAY_KEY_SEGMENT_WIDTHS = [
  YEAR_WIDTH,
  MONTH_WIDTH,
  DAY_WIDTH,
] as const;

/**
 Separator between the day key and the rest of a timestamped name.
 */
const TIMESTAMP_SEPARATOR = 'T';

/**
 Suffix of sink files: `{timestamp}.{uuid}.log.jsonl`.
 */
const LOG_FILE_SUFFIX = '.log.jsonl';

/**
 Suffix of directory probes: `{timestamp}.{uuid}.probe.tmp`.
 */
const PROBE_FILE_SUFFIX = '.probe.tmp';

/**
 Digit check over one character of a candidate name.

 @param character - single character under test

 @returns Whether the character is an ASCII digit

 @example
 ```ts
 const digit = isDigit('7');
 ```
 */
function isDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Digit-run check over one day-key segment.

 @param segment - characters of one segment under test

 @returns Whether the segment is nonempty and every character is an ASCII digit

 @example
 ```ts
 const digits = isDigitRun('2026');
 ```
 */
function isDigitRun(segment: string,): boolean {
  if (segment.length === 0)
    return false;
  for (const character of segment) {
    if (!isDigit(character,))
      return false;
  }
  return true;
}

/**
 Day-key shape over the leading characters of a name: `YYYY-MM-DD`.
 One linear pass over the fixed-width key,
 so no regular expression can backtrack.

 @param name - candidate entry name

 @returns Whether the name opens with a day key

 @example
 ```ts
 const keyed = isDayKey('2026-10-22T14-00-01.123Z.abc.log.jsonl');
 ```
 */
function isDayKey(name: string,): boolean {
  if (name.length < DAY_KEY_LENGTH)
    return false;
  /**
   Day-key segments split on the separator,
   whose digit widths come from the named layout constants.
   */
  const segments = name.slice(
    0,
    DAY_KEY_LENGTH,
  )
    .split(DAY_KEY_SEPARATOR,);
  if (segments.length !== DAY_KEY_SEGMENT_WIDTHS.length)
    return false;
  return segments.every(function matchesWidth(
    segment: string,
    index: number,
  ) {
    /**
     Expected digit width of this day-key segment.
     */
    const expected = DAY_KEY_SEGMENT_WIDTHS[index] ?? 0;
    return (segment.length === expected) && isDigitRun(segment,);
  },);
}

/**
 Classify a name against the logger's owned families:
 `YYYY-MM-DD` day directories,
 timestamp `*.log.jsonl` sink files,
 and timestamp `*.probe.tmp` probes.
 Deliberate neighbors such as `translation-repair-runs*` classify as foreign,
 so a sweep never touches them.

 @param name - entry name from a destination directory listing

 @returns Day key of the name, or {@link NOT_LOGGER_OWNED} for foreign entries

 @example
 ```ts
 const day = loggerOwnedDay('2026-10-22T14-00-01.123Z.abc.log.jsonl');
 ```
 */
export function loggerOwnedDay(name: string,): string | typeof NOT_LOGGER_OWNED {
  if (!isDayKey(name,))
    return NOT_LOGGER_OWNED;
  /**
   Day key of the name, the age key of everything it identifies.
   */
  const day = name.slice(
    0,
    DAY_KEY_LENGTH,
  );
  if (name.length === DAY_KEY_LENGTH)
    return day;
  if (name.charAt(DAY_KEY_LENGTH,) !== TIMESTAMP_SEPARATOR)
    return NOT_LOGGER_OWNED;
  return (name.endsWith(LOG_FILE_SUFFIX,) || name.endsWith(PROBE_FILE_SUFFIX,))
    ? day
    : NOT_LOGGER_OWNED;
}
//endregion Logger-owned names, identified by shape without regular expressions.
