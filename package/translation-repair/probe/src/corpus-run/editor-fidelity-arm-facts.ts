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

 @returns Line's own words

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
 Whether a body is a row line: the line of a trial the arm finished.

 @param body - one line's body

 @returns Whether it starts a row

 @example
 ```ts
 const isRow = isRowBody({ body: 'row 2 of 40: card on tabby/insertion: clean fix exact; entry tabby; stopped ok', },);
 ```
 */
export function isRowBody({ body, }: { readonly body: string; },): boolean {
  return body.startsWith('row ',) && body.includes(' of ',);
}

/**
 Whether a body is the line the run client logs for a reply it gives from an earlier reply and not from the model: the
 words `PROMPT-REUSE` behind any number of bracketed tags, since a logger tagged at its function boundary writes one more
 tag in front of them.

 @param body - one line's body

 @returns Whether reuse marker follows any leading tags

 @example
 ```ts
 // yields true
 const reused = isReuseBody({ body: '[claimedReply] PROMPT-REUSE source=memory model=m digest=d', },);
 ```
 */
export function isReuseBody({ body, }: { readonly body: string; },): boolean {
  // Each pass of the loop steps over one tag, which closes with a bracket and a space, and ends when the words begin.
  for (let at = 0; ;) {
    if (!body.startsWith(
      '[',
      at,
    ))
      return body.startsWith(
        'PROMPT-REUSE ',
        at,
      );
    /**
     Where this tag closes.
     */
    const close = body.indexOf(
      '] ',
      at,
    );
    if (close === (-1))
      return false;
    at = close + '] '.length;
  }
}

/**
 Which rows made no call, in the order the row lines were written: a row follows a `PROMPT-REUSE` line written since the
 row before it, which the run client logs for every reply it gives from an earlier reply and not from the model.

 @param bodies - log's bodies

 @returns One flag for each row line, true for a row whose reply was a reuse

 @example
 ```ts
 // yields [false, true]
 const flags = reusedRowFlags({ bodies: ['row 1 of 2: a', 'PROMPT-REUSE source=memory model=m digest=d', 'row 2 of 2: b',], },);
 ```
 */
export function reusedRowFlags({ bodies, }: { readonly bodies: readonly string[]; },): readonly boolean[] {
  /**
   One flag for each row line met so far.
   */
  const flags: boolean[] = [];
  /**
   Whether a reuse line stands since the last row line.
   */
  let reuseSince = false;
  for (const body of bodies) {
    if (isReuseBody({ body, },))
      reuseSince = true;
    else if (isRowBody({ body, },)) {
      flags.push(reuseSince,);
      reuseSince = false;
    }
  }
  return flags;
}

/**
 The first body that starts with a marker.

 @param bodies - log's bodies

 @param marker - what the body starts with

 @returns Body, empty when there is none

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

 @returns Number, or minus one when the marker is absent or no digit follows it

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

 @returns Word, empty when the marker is absent

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

 @param one - noun for one

 @param many - noun for any other count

 @returns Count and noun

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

 @returns Amount as text

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
   Times the planned line says the arm walks the pool, one when it names none, which is how a single pass reads.
   */
  readonly passes: number;

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

  /**
   Row lines of rows that made no call, a reply of the prompt reuse layer standing in front of each.
   */
  readonly reusedRows: number;
};

/**
 Reads the facts of an arm's log.

 @param log - arm

 @returns Facts

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
    passes: Math.max(
      1,
      numberAfter({
        text: firstStarting({
          bodies,
          marker: 'planned: ',
        },),
        marker: ' asks (',
      },),
    ),
    completed: numberAfter({
      text: trialsLine,
      marker: 'TRIALS completed ',
    },),
    completedOf: numberAfter({
      text: trialsLine,
      marker: ' of ',
    },),
    capped: bodies.some(function isCapped(body,): boolean {
      return body.startsWith('CAPPED at unit ',);
    },),
    reusedRows: reusedRowFlags({ bodies, },)
      .filter(function isReused(flag,): boolean {
        return flag;
      },)
      .length,
  };
}

//endregion Editor fidelity arm facts
