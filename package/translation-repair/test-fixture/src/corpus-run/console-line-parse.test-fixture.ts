import type { Level, } from '@monochromatic-dev/logger-core/ts';

//region Console line parse
// READS A CASE'S LINE AS THE CONSOLE SINK PRINTS IT, `[level] [ISO instant] message`, which is the text every report
// parser was written against and every case was first written in, so the two writers of record files
// (`report-log-write.test-fixture.ts` by hand, `run-log-write.test-fixture.ts` through the logger) split a line the
// same way.
//
// A LINE IS IN THAT FORM ONLY WHEN THE LOGGER COULD HAVE PRINTED IT: a level the logger has, and an instant in the
// form `Date.prototype.toISOString` gives, with its milliseconds and its `Z`. A line whose instant lacks a zone or
// reads as local time is a foreign line, which is what the cases that hold the reports to their "no stamp as the logger
// writes one" count rely on.

/**
 Levels the logger has.
 */
const LEVELS: ReadonlySet<string> = new Set([
  'debug',
  'error',
  'fatal',
  'info',
  'trace',
  'warn',
],);

/**
 Characters the separator between the level and the instant takes.
 */
const LEVEL_TO_STAMP = '] [';

/**
 Characters the separator between the instant and the message takes.
 */
const STAMP_TO_MESSAGE = '] ';

/**
 A line read as the console prints one.

 @example
 ```ts
 const parts: ConsoleLineParts = { level: 'info', at: 0, message: 'purr', };
 ```
 */
type ConsoleLineParts = {
  /**
   Level the line names.
   */
  readonly level: Level;

  /**
   Epoch milliseconds the line's instant names.
   */
  readonly at: number;

  /**
   Text after the instant.
   */
  readonly message: string;
};

/**
 Whether a text names a level the logger has.

 @param value - text between the first brackets of a line

 @returns Whether it is a level

 @example
 ```ts
 isLevel('warn',); // true
 ```
 */
function isLevel(value: string,): value is Level {
  return LEVELS.has(value,);
}

/**
 Reads a line as the console prints one.

 @param line - case's line

 @returns Its parts, or that it is not in the console's form

 @example
 ```ts
 const parts = parseConsoleLine({ line: '[info] [2026-10-23T12:00:00.000Z] purr', },);
 ```
 */
export function parseConsoleLine({ line, }: { readonly line: string; },): ConsoleLineParts | 'not-console-form' {
  if (!line.startsWith('[',))
    return 'not-console-form';
  /**
   Where the level ends.
   */
  const levelEnd = line.indexOf(LEVEL_TO_STAMP,);
  if (levelEnd === (-1))
    return 'not-console-form';
  /**
   Level the line names.
   */
  const level = line.slice(
    1,
    levelEnd,
  );
  if (!isLevel(level,))
    return 'not-console-form';
  /**
   Where the instant starts.
   */
  const stampStart = levelEnd + LEVEL_TO_STAMP.length;
  /**
   Where the instant ends.
   */
  const stampEnd = line.indexOf(
    STAMP_TO_MESSAGE,
    stampStart,
  );
  if (stampEnd === (-1))
    return 'not-console-form';
  /**
   Instant as the line writes it.
   */
  const stamp = line.slice(
    stampStart,
    stampEnd,
  );
  /**
   The instant, in milliseconds.
   */
  const at = Date.parse(stamp,);
  if (Number.isNaN(at,) || (new Date(at,).toISOString() !== stamp))
    return 'not-console-form';
  return {
    level,
    at,
    message: line.slice(stampEnd + STAMP_TO_MESSAGE.length,),
  };
}

//endregion Console line parse
