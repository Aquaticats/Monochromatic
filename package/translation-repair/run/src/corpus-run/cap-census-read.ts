import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  isIsoStampText,
  isWholeNumberText,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { STREAM_MARKER, } from './run-timing-parse.ts';
import {
  readSpendLine,
  type SpendRecord,
} from './spend-read.ts';

//region Cap census read
// THE COMPLETION CAP RULE, RE-READ OVER LATER RUNS (ledger P10). The caps were
// measured once, on 2026-09-09 (`completion-cap.ts`), and a card that took the
// pooled placeholder for want of calls kept it long after it had 67,353 of its
// own. No unit test can see the run logs, and a test that failed on a date
// would fail for the calendar, so the rule is re-read by this census before a
// launch instead.
//
// TWO READINGS, BECAUSE THE CAP CENSORS WHAT IT MEASURES. The rule's own
// reading (the highest provider 99th percentile over providers with enough
// calls, floored at the pooled 90th) takes every completed call; the cut
// reading takes the calls since the caps went on the wire and asks how many ran
// to the card's cap and whether their streams carried content. A capped call
// cannot show a longer answer, so a rule reading equal to the cap means "at
// least one percent ran into it", and the cut reading says whether those were
// answers or reasoning runaways.
//
// PAIRED BY LABEL AND CLOCK. A stream's completion line and its `SPEND` line
// are written by the same exchange within milliseconds, so a `SPEND` line takes
// the latest unpaired completion of the same served id within the window; one
// with none is counted as unpaired, never guessed. A line whose stamp the
// logger did not write is left out and counted, since no clock pairs or dates
// it (ledger B73).
//
// NO PATTERNS AND NO WORDING. Both lines are this codebase's own, read by index
// scans; the census carries ids and numbers only.

/**
 Value `indexOf` returns for text that is not there.
 */
const NOT_FOUND = -1;

/**
 Most milliseconds between a stream's completion line and its `SPEND` line.
 */
const PAIR_WINDOW_MS = 50;

/**
 Text a completed stream line carries after its label.
 */
const COMPLETED_FIELD = ': completed, ';

/**
 Field naming the content characters a stream delivered, up to its unit's
 plural ending: the stream line writes "char" after a count of one and
 "chars" after any other (ledger B109), and lines logged before that wrote
 "chars" after every count.
 */
const CONTENT_FIELD = ' content char';

/**
 Whether text begins where a stream line's field ends: at the separator
 before the next field, or at the line's end.

 @param text - text after a field's last word

 @returns True where the field ends there

 @example
 ```ts
 const ended = endsField({ text: ', 0 reasoning chars', },);
 ```
 */
function endsField({ text, }: { readonly text: string; },): boolean {
  return (text === '') || text.startsWith(',',);
}

/**
 When every client began sending the cap as `max_tokens` (`completion-cap.ts`,
 measured as of 16:20 UTC on 2026-09-09); calls after it are capped.
 */
export const CAPS_ON_WIRE_AT: number = Date.parse('2026-09-09T16:20:00Z',);

/**
 One completed call the census reads.

 @example
 ```ts
 const sample: CapSample = { provider: 'hyper', model: 'kimi-k3', completion: 800, at: 0, content: 1_200, };
 ```
 */
export type CapSample = {
  /**
   Provider that served the call.
   */
  readonly provider: SpendRecord['provider'];

  /**
   Model as that provider names it.
   */
  readonly model: string;

  /**
   Completion tokens the wire reported, thinking included.
   */
  readonly completion: number;

  /**
   Epoch milliseconds the `SPEND` line was written.
   */
  readonly at: number;

  /**
   Content characters the paired stream delivered, or that no stream line
   paired with it.
   */
  readonly content: number | 'unpaired';
};

/**
 Reads the epoch milliseconds a logged line was written at.

 @param line - one log line, `[level] [iso] ...`

 @returns Milliseconds, or that the line carries no stamp as the logger
 writes one (ledger B73)

 @example
 ```ts
 const at = stampOf({ line, },);
 ```
 */
function stampOf({ line, }: { readonly line: string; },): number | 'unstamped' {
  /**
   What the line's second bracket holds, absent where the line has none.
   */
  const [, stamp,] = line.split('] [',);

  // A spend line need not carry the logger's prefix. `readSpendLine` reads
  // the bare line `reportSpend` returns and the same line behind any text
  // ending in a space, so a record reaches here with one bracket or none. It
  // has no stamp the logger wrote, which is what `unstamped` says; throwing
  // on it ended the whole census on one line (ledger B73's rule is that such
  // a line costs its call, not the report).
  if (stamp === undefined)
    return 'unstamped';
  return isIsoStampText({ text: stamp, },) ? Date.parse(stamp,) : 'unstamped';
}

/**
 Reads a completed stream line's label and content characters.

 @param line - one log line

 @returns Label and content, or that the line is no completed stream

 @example
 ```ts
 const stream = streamContentOf({ line, },);
 ```
 */
function streamContentOf(
  { line, }: { readonly line: string; },
): {
  readonly label: string;
  readonly content: number
} | 'other-line' {
  /**
   Where the stream payload starts.
   */
  const at = line.indexOf(STREAM_MARKER,);
  if (at === NOT_FOUND)
    return 'other-line';

  /**
   Payload after the marker: label, outcome, then the fields.
   */
  const payload = line.slice(at + STREAM_MARKER.length,);

  /**
   Where the label ends, on a completed stream only.
   */
  const labelEnd = payload.indexOf(COMPLETED_FIELD,);

  /**
   Where the content field's unit starts.
   */
  const contentAt = payload.indexOf(CONTENT_FIELD,);
  if ((labelEnd === NOT_FOUND) || (contentAt === NOT_FOUND))
    return 'other-line';

  /**
   What follows the unit's stem: its plural ending, if any, then the field's end.
   */
  const unitTail = payload.slice(contentAt + CONTENT_FIELD.length,);
  if (!(endsField({ text: unitTail, },) || (unitTail.startsWith('s',) && endsField({ text: unitTail.slice(1,), },))))
    return 'other-line';

  /**
   Text ahead of the unit, whose last word is the count.
   */
  const before = payload.slice(
    0,
    contentAt,
  );

  /**
   The count as written, the last word before the unit.
   */
  const contentText = before.slice(before.lastIndexOf(' ',) + 1,);
  // DIGITS AS THE STREAM LINE WRITES THEM, or no completed stream: `Number`
  // read an empty count (two spaces before the unit) as a stream that
  // delivered nothing, and took a sign or an exponent (ledger B73).
  if (!isWholeNumberText({ text: contentText, },))
    return 'other-line';
  return {
    label: payload.slice(
      0,
      labelEnd,
    ),
    content: Number(contentText,),
  };
}

/**
 What one pass-run log says about the calls it completed.

 @example
 ```ts
 const { samples, unstampedLines, } = readCapLog({ lines: text.split('\n',), },);
 ```
 */
export type CapLogReading = {
  /**
   One sample per reported `SPEND` line with a completion count and a stamp
   as the logger writes one.
   */
  readonly samples: readonly CapSample[];

  /**
   Stream and `SPEND` lines left out because the logger did not write their
   stamp, so no clock pairs or dates them (ledger B73).
   */
  readonly unstampedLines: number;
};

/**
 Reads every completed call a pass-run log reports, each with what its stream
 delivered.

 @param lines - lines of one log, in order

 @returns One sample per reported `SPEND` line with a completion count;
 reckoned lines and unreported counts are left out, since neither is a length
 the wire measured, and lines whose stamp the logger did not write are left
 out and counted

 @example
 ```ts
 const { samples, unstampedLines, } = readCapLog({ lines: text.split('\n',), },);
 ```
 */
export function readCapLog(
  { lines, }: { readonly lines: readonly string[]; },
): CapLogReading {
  /**
   Completed streams not yet paired, by label, oldest first. Each queue grows
   and shrinks in place, so no line copies it (ledger B74).
   */
  const waiting = new Map<string, {
    readonly at: number;
    readonly content: number
  }[]>();

  /**
   Samples read so far.
   */
  const samples: CapSample[] = [];

  /**
   Lines left out so far for a stamp the logger did not write.
   */
  const unstamped = { lines: 0, };
  for (const line of lines) {
    /**
     The line as a completed stream, if it is one.
     */
    const stream = streamContentOf({ line, },);
    if (stream !== 'other-line') {
      /**
       When the stream line was written.
       */
      const streamAt = stampOf({ line, },);
      if (streamAt === 'unstamped') {
        unstamped.lines += 1;
        continue;
      }
      /**
       The stream, as its label's queue holds it.
       */
      const entry = {
        at: streamAt,
        content: stream.content,
      };
      /**
       Streams of this label already waiting, which this one joins.
       */
      const labelQueue = waiting.get(stream.label,);
      if (labelQueue === undefined) {
        waiting.set(
          stream.label,
          [entry,],
        );
      }
      else
        labelQueue.push(entry,);
      continue;
    }

    /**
     The line as a spend record, if it is one.
     */
    const record = readSpendLine({ line, },);
    if (((typeof record) !== 'object') || (record.reckoning !== 'reported')
      || ((typeof record.completion) !== 'number'))
      continue;

    /**
     When the spend line was written.
     */
    const at = stampOf({ line, },);
    if (at === 'unstamped') {
      unstamped.lines += 1;
      continue;
    }

    /**
     Streams of this label still unpaired.
     */
    const queue = waiting.get(record.model,) ?? [];

    /**
     Latest of them within the window, or none.
     */
    const index = queue.findLastIndex(function inWindow(entry,): boolean {
      return Math.abs(at - entry.at,) <= PAIR_WINDOW_MS;
    },);

    /**
     The stream this line pairs with, taken off the queue, or that none does.
     The index comes from the queue itself, so the entry is present.
     */
    const paired = (index === NOT_FOUND) ? 'unpaired' : nonNullishOrThrow(queue[index],)
      .content;
    if (index !== NOT_FOUND) {
      queue.splice(
        index,
        1,
      );
    }
    samples.push({
      provider: record.provider,
      model: record.model,
      completion: record.completion,
      at,
      content: paired,
    },);
  }
  return {
    samples,
    unstampedLines: unstamped.lines,
  };
}

//endregion Cap census read
