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
// PAIRED BY LABEL, FILE ORDER AND CLOCK (ledger B1070). A stream's completion
// line and its `SPEND` line are written by the same exchange, the stream line
// first, but not within milliseconds: the gap grows with the body the stream
// read, from 1 to 10 ms under 300,000 raw characters to 139 to 589 ms near 4.1
// million, and 1,107 ms at the longest among capped calls, so a window of 50 ms
// left every capped call of a long body unpaired. A `SPEND` line looks at the
// completed streams of its served id that come before it in the file, that no
// earlier `SPEND` line has taken, and that sit within `PAIR_WINDOW_MS` of its
// stamp. One such stream is its own and is taken. None is counted as unpaired.
// Two or more are counted as ambiguous, never guessed, and all stay waiting as
// contested, since nothing in either line says which belongs to which: a later
// line that finds only one of them left is ambiguous as well, because an
// earlier line may already be its pair. A line whose stamp the logger did not
// write is left out and counted, since no clock pairs or dates it (ledger B73).
//
// NO PATTERNS AND NO WORDING. Both lines are this codebase's own, read by index
// scans; the census carries ids and numbers only.

/**
 Value `indexOf` returns for text that is not there.
 */
const NOT_FOUND = -1;

/**
 Most milliseconds between a stream's completion line and its `SPEND` line, three seconds.

 MEASURED OVER THE 447 PASS-RUN LOGS UNDER THE AGENT ROOT ON 2026-10-29 (582,710 reported `SPEND` lines, each against
 the latest completed stream of its label that no earlier line had taken; ledger B1070): the gap is 1 ms at the
 median, 4 ms at the 90th percentile, 40 ms at the 99th and 115 ms at the 99.9th, and 36 calls sit past two seconds,
 the longest 13.5 seconds. Among the 34,255 calls of 3,800 completion tokens or more, which takes in the cap of every
 card, the longest gap is 2,096 ms, and 3 calls sit past one second. Three seconds holds all of those with room, and
 leaves 23 of the 582,710 calls outside it.
 The window is no wider because a wider one puts a second stream in reach of more calls, and the calls that read as
 ambiguous grow with it: 502 at one second, 871 at two, 1,335 at three and 2,947 at five.
 */
const PAIR_WINDOW_MS = 3_000;

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
   Content characters the paired stream delivered, that no stream line paired
   with it (`unpaired`), or that two or more streams could each be its own and
   none was chosen (`ambiguous`).
   */
  readonly content: number | 'unpaired' | 'ambiguous';
};

/**
 One completed stream waiting for its `SPEND` line.

 @example
 ```ts
 const waiting: WaitingStream = { at: 0, content: 59, sequence: 0, };
 ```
 */
type WaitingStream = {
  /**
   Epoch milliseconds the stream line was written.
   */
  readonly at: number;

  /**
   Content characters the stream delivered.
   */
  readonly content: number;

  /**
   Place the stream took among those of its label, from zero, so that streams an ambiguous `SPEND` line has seen can be
   told from those that arrived after it.
   */
  readonly sequence: number;
};

/**
 The completed streams of one label that no `SPEND` line has taken.

 @example
 ```ts
 const queue: LabelQueue = { entries: [], head: 0, nextSequence: 0, contestedThrough: -1, };
 ```
 */
type LabelQueue = {
  /**
   Streams in file order, those before `head` already dropped.
   */
  readonly entries: WaitingStream[];

  /**
   Place in `entries` of the oldest stream still waiting.
   */
  head: number;

  /**
   Sequence the next stream of the label takes.
   */
  nextSequence: number;

  /**
   Highest sequence among the streams an ambiguous `SPEND` line has seen, or minus one where none has. A waiting stream
   numbered at or below it is contested: an earlier line may be its pair, so no later line takes it alone.
   */
  contestedThrough: number;
};

/**
 Waiting streams of a label shorter than this are never compacted, so a short queue costs nothing to keep.
 */
const COMPACT_AT = 64;

/**
 Drops, in place, the streams at the front of a queue that are too old for any later line to pair with.

 Lines come in file order, whose stamps rise, so a stream older than the window against the newest line of its
 label is older than the window against every line after it. The front moves by an index, and the array is cut only
 when the dropped half has grown past `COMPACT_AT`, so a line costs a constant amount however many streams wait (ledger
 B74).

 @param queue - waiting streams of one label

 @param now - epoch milliseconds of the line being read

 @mutates queue - moves its front past the streams older than the window

 @example
 ```ts
 dropStale({ queue, now: Date.now(), },);
 ```
 */
function dropStale(
  {
    queue,
    now,
  }: {
    readonly queue: LabelQueue;
    readonly now: number;
  },
): void {
  /**
   The streams of the label, named once.
   */
  const { entries, } = queue;
  while (queue.head < entries.length) {
    /**
     The oldest stream still waiting.
     */
    const oldest = nonNullishOrThrow(entries[queue.head],);
    if ((now - oldest.at) <= PAIR_WINDOW_MS)
      break;
    queue.head += 1;
  }
  if ((queue.head >= COMPACT_AT) && ((queue.head * 2) >= entries.length)) {
    entries.splice(
      0,
      queue.head,
    );
    queue.head = 0;
  }
}

/**
 Pairs a `SPEND` line with the stream that is its own, when there is exactly one it can be told from the rest.

 @param queue - waiting streams of the line's label

 @param now - epoch milliseconds the `SPEND` line was written

 @returns The content characters of the stream taken off the queue; `unpaired` where none waits within the window;
 `ambiguous` where two or more wait, or the one that waits was seen by an earlier ambiguous line

 @mutates queue - drops the streams older than the window, takes the one stream paired, and marks the streams seen by an
 ambiguous line as contested

 @example
 ```ts
 const content = pairedContentOf({ queue, now: Date.now(), },);
 ```
 */
function pairedContentOf(
  {
    queue,
    now,
  }: {
    readonly queue: LabelQueue;
    readonly now: number;
  },
): CapSample['content'] {
  dropStale({
    queue,
    now,
  },);

  /**
   The streams of the label, named once.
   */
  const { entries, } = queue;

  /**
   Streams of the label still waiting within the window.
   */
  const waiting = entries.length - queue.head;
  if (waiting === 0)
    return 'unpaired';
  // TWO OR MORE ARE NOT CHOSEN BETWEEN and every one of them is now contested,
  // so the lines after this one read them as ambiguous too, as they are.
  if (waiting >= 2) {
    queue.contestedThrough = queue.nextSequence - 1;
    return 'ambiguous';
  }

  /**
   The one stream waiting. The count says it is present.
   */
  const only = nonNullishOrThrow(entries[queue.head],);
  // A STREAM AN EARLIER AMBIGUOUS LINE SAW MAY BE THAT LINE'S PAIR, so a line
  // that finds it alone cannot tell it is this call's.
  if (only.sequence <= queue.contestedThrough)
    return 'ambiguous';
  queue.head += 1;
  return only.content;
}

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
   and shrinks in place, so no line copies it (ledger B74), and holds only
   streams within the window of the newest line of its label.
   */
  const waiting = new Map<string, LabelQueue>();

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
       Streams of this label already waiting, which this one joins; the queue is made with the first stream of a label.
       */
      const labelQueue: LabelQueue = waiting.get(stream.label,) ?? {
        entries: [],
        head: 0,
        nextSequence: 0,
        contestedThrough: -1,
      };
      waiting.set(
        stream.label,
        labelQueue,
      );
      dropStale({
        queue: labelQueue,
        now: streamAt,
      },);
      /**
       The streams of the label, named once.
       */
      const { entries, } = labelQueue;
      entries.push({
        at: streamAt,
        content: stream.content,
        sequence: labelQueue.nextSequence,
      },);
      labelQueue.nextSequence += 1;
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
     What this call's stream delivered, or that none was paired or none could be told from another; a label with no
     stream line yet has no queue, and none is paired.
     */
    const queue = waiting.get(record.model,);
    samples.push({
      provider: record.provider,
      model: record.model,
      completion: record.completion,
      at,
      content: (queue === undefined)
        ? 'unpaired'
        : pairedContentOf({
          queue,
          now: at,
        },),
    },);
  }
  return {
    samples,
    unstampedLines: unstamped.lines,
  };
}

//endregion Cap census read
