import { isUnsignedNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Editor fidelity arm facts
// WHAT AN ARM'S LOG SAYS, read by its markers and not by a regex: each line the probe writes starts with a word this module looks
// for after the logger's three bracketed groups, and a number is the digits that follow a marker. A writer that changes its
// wording leaves a fact absent, which the checks answer as "no line was logged", the right answer for a log they cannot read.

/**
 Decimals the guard's total line prints dollars with.
 */
const USD_DECIMALS = 6;

/**
 What the arm's exit status is known as: the number a file beside the log holds, or that nobody recorded one.

 @example
 ```ts
 const exit: ExitRecord = { recorded: true, code: 0, };
 ```
 */
export type ExitRecord =
  | {
    /**
     A status was recorded.
     */
    readonly recorded: true;

    /**
     The status, zero for success.
     */
    readonly code: number;
  }
    | {
    /**
     No status was recorded for this arm.
     */
      readonly recorded: false;
    };

/**
 One arm's log and what is known of its process.

 @example
 ```ts
 const arm: ArmLog = { arm: 'low', lines: [], exit: { recorded: false, }, };
 ```
 */
export type ArmLog = {
  /**
   Name of the arm, the log's file name without its extension.
   */
  readonly arm: string;

  /**
   The log's lines, each as the logger wrote it.
   */
  readonly lines: readonly string[];

  /**
   The status the process exited with.
   */
  readonly exit: ExitRecord;
};

/**
 The text of a log line after the logger's level, stamp and tag, or the line itself when it has none.

 @param line - one line of a log

 @returns The line's own words

 @example
 ```ts
 // yields 'pool: 40 trials'
 const body = bodyOf({ line: '[info] [2026-10-23T12:00:00.000Z] [probe] pool: 40 trials', },);
 ```
 */
export function bodyOf({ line, }: { readonly line: string; },): string {
  /**
   The level, the stamp, the tag, and the line's own words cut after each closing bracket and space.
   */
  const [level = '', stamp = '', tag = '', ...words] = line.split('] ',);
  /**
   Whether the first three pieces are the level, the stamp and a tag.
   */
  const prefixed = (words.length > 0) && [
    level,
    stamp,
    tag,
  ].every(function opens(part,): boolean {
    return part.startsWith('[',);
  },);
  return prefixed
    ? words.join('] ',)
    : line;
}

/**
 The first body that starts with a marker.

 @param bodies - the log's bodies

 @param marker - what the body starts with

 @returns The body, empty when there is none

 @example
 ```ts
 const pool = firstStarting({ bodies, marker: 'pool: ', },);
 ```
 */
export function firstStarting({
  bodies,
  marker,
}: {
  readonly bodies: readonly string[];
  readonly marker: string
},): string {
  return bodies.find(function starts(body,): boolean {
    return body.startsWith(marker,);
  },) ?? '';
}

/**
 The number written right after a marker, a decimal point included.

 @param text - text read

 @param marker - what stands just before the number

 @returns The number, or minus one when the marker is absent or no digit follows it

 @example
 ```ts
 // yields 40
 const trials = numberAfter({ text: 'pool: 40 trials', marker: 'pool: ', },);
 ```
 */
export function numberAfter({
  text,
  marker,
}: {
  readonly text: string;
  readonly marker: string
}): number {
  /**
   Where the marker sits.
   */
  const at = text.indexOf(marker,);
  if (at === (-1))
    return -1;
  /**
   The characters after the marker.
   */
  const after = Array.from(text.slice(at + marker.length,),);
  /**
   Where the number ends: the first character that is neither a digit nor a point.
   */
  const ends = after.findIndex(function isOther(character,): boolean {
    return ((character < '0') || (character > '9')) && (character !== '.');
  },);
  /**
   The digits, with a point if the number has one.
   */
  const digits = after.slice(
    0,
    (ends === (-1)) ? after.length : ends,
  )
    .join('',);
  return isUnsignedNumberText({ text: digits, },) ? Number(digits,) : -1;
}

/**
 The word written right after a marker, up to the next space.

 @param text - text read

 @param marker - what stands just before the word

 @returns The word, empty when the marker is absent

 @example
 ```ts
 // yields 'abc'
 const digest = wordAfter({ text: 'digest abc def', marker: 'digest ', },);
 ```
 */
export function wordAfter({
  text,
  marker,
}: {
  readonly text: string;
  readonly marker: string
}): string {
  /**
   Where the marker sits.
   */
  const at = text.indexOf(marker,);
  if (at === (-1))
    return '';
  return text.slice(at + marker.length,)
    .split(' ',)[0]
    ?? '';
}

/**
 Says how many of a noun, with the noun in its form for the count.

 @param count - how many

 @param one - the noun for one

 @param many - the noun for any other count

 @returns The count and the noun

 @example
 ```ts
 // yields '39 rows'
 const said = countWithNoun({ count: 39, one: 'row', many: 'rows', },);
 ```
 */
export function countWithNoun({
  count,
  one,
  many,
}: {
  readonly count: number;
  readonly one: string;
  readonly many: string
}): string {
  return `${String(count,)} ${wordForCount({
    count,
    one,
    many,
  },)}`;
}

/**
 Dollars to the six decimals the guard's total line prints.

 @param usd - dollars

 @returns The amount as text

 @example
 ```ts
 // yields '0.875000'
 const said = usdSaid({ usd: 0.875, },);
 ```
 */
export function usdSaid({ usd, }: { readonly usd: number; }): string {
  return usd.toFixed(USD_DECIMALS,);
}

/**
 The facts one arm's checks need, read off its log once.

 @example
 ```ts
 const facts: ArmFacts = factsOf({ log, },);
 ```
 */
export type ArmFacts = {
  /**
   The log's bodies.
   */
  readonly bodies: readonly string[];

  /**
   The card the arm seated, from the editors line.
   */
  readonly card: string;

  /**
   Trials the pool line says, minus one when there is no pool line.
   */
  readonly poolTrials: number;

  /**
   Dollars the guard's total line says, minus one when there is none.
   */
  readonly guardUsd: number;

  /**
   Cap the guard line says, minus one when there is none.
   */
  readonly maxUsd: number;

  /**
   Spend of the earlier arms the guard line says the arm was told, minus one when there is none.
   */
  readonly benchSpentUsd: number;

  /**
   Digest the manifest line says, empty when there is none.
   */
  readonly digest: string;

  /**
   Trials the planned line says the arm was asked to walk, minus one when there is none.
   */
  readonly planned: number;

  /**
   Trials the TRIALS line says were completed, minus one when there is none.
   */
  readonly completed: number;

  /**
   Trials the TRIALS line says were planned, minus one when there is none.
   */
  readonly completedOf: number;

  /**
   Whether the log says the spend guard capped the arm.
   */
  readonly capped: boolean;
};

/**
 Reads the facts of an arm's log.

 @param log - the arm

 @returns The facts

 @example
 ```ts
 const facts = factsOf({ log, },);
 ```
 */
export function factsOf({ log, }: { readonly log: ArmLog; }): ArmFacts {
  /**
   The log's bodies.
   */
  const bodies = log.lines
    .map(function body(line,): string {
      return bodyOf({ line, },);
    },);
  /**
   The guard's total line, if any.
   */
  const total = bodies.find(function isTotal(body,): boolean {
    return body.startsWith('GUARD total=',);
  },) ?? '';
  /**
   The line that says how many trials were completed, if any.
   */
  const trialsLine = firstStarting({
    bodies,
    marker: 'TRIALS completed ',
  },);
  return {
    bodies,
    card: wordAfter({
      text: firstStarting({
        bodies,
        marker: 'editors: ',
      },),
      marker: 'editors: ',
    },),
    poolTrials: numberAfter({
      text: firstStarting({
        bodies,
        marker: 'pool: ',
      },),
      marker: 'pool: ',
    },),
    guardUsd: numberAfter({
      text: total,
      marker: 'GUARD total=',
    },),
    maxUsd: numberAfter({
      text: firstStarting({
        bodies,
        marker: 'spend guard: ',
      },),
      marker: '--max-usd ',
    },),
    benchSpentUsd: numberAfter({
      text: firstStarting({
        bodies,
        marker: 'spend guard: ',
      },),
      marker: '--bench-spent-usd ',
    },),
    digest: wordAfter({
      text: firstStarting({
        bodies,
        marker: 'manifest: ',
      },),
      marker: 'digest ',
    },),
    planned: numberAfter({
      text: firstStarting({
        bodies,
        marker: 'planned: ',
      },),
      marker: 'planned: ',
    },),
    completed: numberAfter({
      text: trialsLine,
      marker: 'TRIALS completed ',
    },),
    completedOf: numberAfter({
      text: trialsLine,
      marker: ' of ',
    },),
    capped: bodies.some(function isCapped(body,): boolean {
      return body.startsWith('CAPPED at trial ',);
    },),
  };
}

//endregion Editor fidelity arm facts
