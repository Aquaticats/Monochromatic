import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  isIsoStampText,
  isWholeNumberText,
  WHOLE_NUMBER_RULE,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Run timing parse
// Reads a run's own log back into the two shapes that say where its hours
// went, and answers the question the timing work was opened on.
//
// TWO LINES CARRY THE CLOCK, and neither did before the timing work:
//
//   The ROUND line, from `runGatherRound`, splits one fan-out into the time it
//   spent working and the time it spent waiting on a straggler after quorum
//   already stood. Only the second half is straggler cost. A round whose
//   quorum never stood says so instead, with the count it needed (ledger P12).
//
//   The STREAM line, from `reportStreamProgress`, carries `elapsed`, which with
//   the line's own timestamp gives each call an interval. Overlapping those
//   intervals is what achieved concurrency IS, and nothing could compute it
//   while a completion line said only when a call finished.
//
// EVERY READ NAMES WHAT IT FOUND rather than returning an absence. A completion
// line with no duration and a line that is not a completion at all are
// different facts about a log: the first says this run predates the timing work, and a
// report that folded them together would describe a mixed archive's readable
// half as the whole of it. A line whose own fields will not read is refused
// with the field named (T8, nineteenth batch: a missing field once read as one
// that lost its unit, and a completion with no outcome as an outcome of "").
//
// NO PATTERNS. Both lines are written by this codebase with fixed separators,
// so an index scan reads them exactly and says which field was missing when one
// drifts, where a pattern would only fail to match.
//
// READS IDS, COUNTS AND DURATIONS. A run log carries no corpus wording by
// construction, and nothing here would print it if it did.

/**
 Marker that opens a stream completion line's payload.
 */
export const STREAM_MARKER = '] [reportStreamProgress] stream ';

/**
 Marker that opens a round line's payload, after the stage label.
 */
const ROUND_MARKER = ' round: ';

/**
 Separator between the fields of either payload.
 */
const FIELD_SEPARATOR = ', ';

/**
 Fields a round payload carries, whether its quorum stood or not.
 */
const ROUND_FIELDS = 4;

/**
 Text the ratio field ends with.
 */
const RATIO_CLOSE = ' heard';

/**
 Field a round whose quorum stood ends with, which tells a complete line from
 one truncated at the tail of a log still being written.
 */
const GRACE_FIELD = 'ms in grace';

/**
 Field a round whose quorum never stood ends with (ledger P12).
 */
const SETTLED_FIELD = 'every ask settled';

/**
 Text a no-quorum field opens with, ahead of its heard count.
 */
const NO_QUORUM_OPEN = 'no quorum (';

/**
 Text between a no-quorum field's heard count and its needed count.
 */
const NO_QUORUM_JOIN = ' of ';

/**
 Text a no-quorum field closes with, after its needed count.
 */
const NO_QUORUM_CLOSE = ' needed)';

/**
 Field a completion line carries only since the timing work.
 */
const ELAPSED_FIELD = 'elapsed ';

/**
 Text between a completion's label and its outcome.
 */
const OUTCOME_JOIN = ': ';

/**
 Value `indexOf` returns for text that is not there.
 */
const NOT_FOUND = -1;

/**
 Refusal of one field of a timing line, naming the field as the line carries
 it and what it lacks.

 @example
 ```ts
 throw new TimingFieldError({ reason: 'count field is empty', },);
 ```
 */
export class TimingFieldError extends Error {
  /**
   Writes the refusal's one sentence.

   @param reason - what the field lacks, with the field quoted where it has text
   */
  constructor({ reason, }: { readonly reason: string; },) {
    super(reason,);
    this.name = 'TimingFieldError';
  }
}

/**
 Refusal of a whole timing line whose own fields will not read, carrying the
 field's refusal as its cause.

 A ROUND OR CALL REPORTED IN PART IS WORSE THAN ONE NOT REPORTED: a straggler
 cost or an interval read from the fields that happened to parse describes a
 round or call the log never wrote.

 @example
 ```ts
 throw new TimingLineError({ kind: 'round', line, cause: error, },);
 ```
 */
export class TimingLineError extends Error {
  /**
   Writes the refusal around the line it could not read.

   @param kind - which line the reader took it for

   @param line - log line its marker chose, quoted whole so an operator
   can find it

   @param cause - field's refusal
   */
  constructor(
    {
      kind,
      line,
      cause,
    }: {
      readonly kind: 'round' | 'completion';
      readonly line: string;
      readonly cause: unknown;
    },
  ) {
    super(
      `${kind} line unreadable: ${line}`,
      { cause, },
    );
    this.name = 'TimingLineError';
  }
}

/**
 Whether a round's quorum stood, and its timing either side of that instant
 where it did.

 TWO KINDS RATHER THAN ZEROS. A round whose quorum never stood measured no
 time to quorum and spent no grace; writing zeros for them would report a
 measurement the round line never made (T8, nineteenth batch).

 @example
 ```ts
 const quorum: RoundQuorum = { kind: 'never', needed: 4, };
 ```
 */
export type RoundQuorum =
  | {
    readonly kind: 'stood';

    /**
     Time before quorum stood, which is the round doing its work.
     */
    readonly toQuorumMs: number;

    /**
     Time after quorum stood, which is the round waiting on voices it may
     never hear. THIS IS THE STRAGGLER COST, measured rather than bounded.
     */
    readonly inGraceMs: number;
  }
    | {
      readonly kind: 'never';

    /**
     Voices the round needed and never heard enough of.
     */
      readonly needed: number;
    };

/**
 One fan-out round, as its own line reported it.

 @example
 ```ts
 const round: RoundTiming = { stage: 'editor', heard: 6, asked: 7, totalMs: 91_402, quorum: { kind: 'stood', toQuorumMs: 61_401, inGraceMs: 30_001, }, };
 ```
 */
export type RoundTiming = {
  /**
   Stage that ran this round.
   */
  readonly stage: string;

  /**
   Voices this round heard, which the grace window can raise past quorum.
   */
  readonly heard: number;

  /**
   Models this round asked.
   */
  readonly asked: number;

  /**
   Time the whole round took.
   */
  readonly totalMs: number;

  /**
   Whether quorum stood, with the time either side of it where it did.
   */
  readonly quorum: RoundQuorum;
};

/**
 What one line turned out to say about a round.

 @example
 ```ts
 const reading: RoundReading = readRoundTiming({ line, },);
 ```
 */
export type RoundReading =
  | {
    readonly kind: 'round';

    /**
     Numbers the round reported about itself.
     */
    readonly round: RoundTiming;
  }
    | {
    /**
     Line says nothing about a round, including a round line truncated by a
     log still being written.
     */
      readonly kind: 'other-line';
    };

/**
 One model call, as its completion line reported it.

 @example
 ```ts
 const call: CallTiming = { label: 'hf:whiskers', outcome: 'completed', endedAt: 1_760_000_000_000, elapsedMs: 4_210, };
 ```
 */
export type CallTiming = {
  /**
   Model the call went to.
   */
  readonly label: string;

  /**
   How the call ended: `completed`, `cut`, `degenerate`.
   */
  readonly outcome: string;

  /**
   Epoch milliseconds the completion line was written, which is when the call
   ended.
   */
  readonly endedAt: number;

  /**
   How long the call ran, from arming to its end.
   */
  readonly elapsedMs: number;
};

/**
 What one line turned out to say about a call.

 @example
 ```ts
 const reading: CallReading = readCallTiming({ line, },);
 ```
 */
export type CallReading =
  | {
    readonly kind: 'timed';

    /**
     Interval the call occupied, which is what an overlap count needs.
     */
    readonly call: CallTiming;
  }
    | {
    /**
     Completion line carrying no duration, which every log written before
     the timing work is made of. Counted rather than skipped, so a report can say how
     much of an archive it could not read.
     */
      readonly kind: 'untimed';
    }
    | {
    /**
     Completion line carrying a duration but no stamp as the logger writes
     one, so no instant places the call (ledger B73). Counted rather than
     skipped, like an untimed line.
     */
      readonly kind: 'unstamped';
    }
    | {
    /**
     Line is not a completion at all.
     */
      readonly kind: 'other-line';
    };

/**
 Reads one field as a whole number, refusing anything else.

 @param field - digits and nothing else

 @returns Count the field carries

 @throws TimingFieldError when the field is empty, or is not digits a double
 holds exactly

 @example
 ```ts
 const heard = countIn({ field: '5', },);
 ```
 */
function countIn({ field, }: { readonly field: string; },): number {
  if (field === '')
    throw new TimingFieldError({ reason: 'count field is empty', },);
  if (!isWholeNumberText({ text: field, },))
    throw new TimingFieldError({ reason: `count field is not ${WHOLE_NUMBER_RULE}: "${field}"`, },);
  return Number(field,);
}

/**
 Reads one field's number, given the unit it must carry.

 NAMES THE FIELD IT COULD NOT READ rather than returning a zero, because a
 silent zero in a timing report reads as a measurement of nothing happening.

 @param field - one comma-separated field, shaped `<number>ms <name>`

 @param unit - text the number is followed by

 @returns Number the field opened with

 @throws TimingFieldError when the field does not carry that unit, or what
 precedes it is not a whole number of milliseconds written in digits

 @example
 ```ts
 const ms = durationIn({ field: '30001ms in grace', unit: 'ms ', },);
 ```
 */
function durationIn(
  {
    field,
    unit,
  }: {
    readonly field: string;
    readonly unit: string;
  },
): number {
  /**
   Where the unit starts, ending the digits.
   */
  const end = field.indexOf(unit,);
  if (end === NOT_FOUND)
    throw new TimingFieldError({ reason: `timing field carries no ${unit}unit: "${field}"`, },);
  /**
   Text ahead of the unit.
   */
  const digits = field.slice(
    0,
    end,
  );
  // WHOLE MILLISECONDS IN DIGITS, as the round and stream lines write them:
  // `Number` read nothing before the unit as no time at all, NaN out of a word,
  // and a sign or an exponent as a time no line writes (ledger B73).
  if (!isWholeNumberText({ text: digits, },))
    throw new TimingFieldError({ reason: `timing field's milliseconds are not ${WHOLE_NUMBER_RULE}: "${field}"`, },);
  return Number(digits,);
}

/**
 Reads the heard and asked counts off a ratio field.

 TWO WHOLE NUMBERS OR NOTHING. `Number('')` is 0 and `Number('x')` is NaN,
 and either rode into the report as a round that heard nobody or a round that
 never summed.

 @param field - payload's first field, shaped `<heard>/<asked> heard`

 @returns Heard and asked counts

 @throws TimingFieldError when the field is not two whole numbers joined by a
 slash ahead of ` heard`

 @example
 ```ts
 const { heard, asked, } = ratioIn({ field: '6/7 heard', },);
 ```
 */
function ratioIn({ field, }: { readonly field: string; },): {
  readonly heard: number;
  readonly asked: number;
} {
  /**
   Counts ahead of the closing word, or the whole field where it has none.
   */
  const ratio = field.endsWith(RATIO_CLOSE,)
    ? field.slice(
      0,
      -RATIO_CLOSE.length,
    )
    : field;
  /**
   Heard and asked counts, which the ratio joins with a slash.
   */
  const counts = ratio.split('/',);
  if ((!field.endsWith(RATIO_CLOSE,)) || (counts.length !== 2))
    throw new TimingFieldError({ reason: `round ratio is not heard/asked: "${ratio}"`, },);
  return {
    heard: countIn({ field: nonNullishOrThrow(counts[0],), },),
    asked: countIn({ field: nonNullishOrThrow(counts[1],), },),
  };
}

/**
 Reads the count a round needed off its no-quorum field, checking the heard
 count the field repeats against the ratio's.

 @param field - payload's third field, shaped `no quorum (<heard> of <needed> needed)`

 @param heard - heard count the ratio carries

 @returns Count the round needed

 @throws TimingFieldError when the field is not that shape, its counts are not
 whole numbers, or its heard count is not the ratio's

 @example
 ```ts
 const needed = neededIn({ field: 'no quorum (1 of 4 needed)', heard: 1, },);
 ```
 */
function neededIn(
  {
    field,
    heard,
  }: {
    readonly field: string;
    readonly heard: number;
  },
): number {
  /**
   Whether the field opens and closes as the writer writes it.
   */
  const framed = field.startsWith(NO_QUORUM_OPEN,) && field.endsWith(NO_QUORUM_CLOSE,);
  /**
   Counts between the field's opening and closing text.
   */
  const counts = framed
    ? field
      .slice(
      NO_QUORUM_OPEN.length,
      -NO_QUORUM_CLOSE.length,
      )
      .split(NO_QUORUM_JOIN,)
    : [];
  if (counts.length !== 2) {
    throw new TimingFieldError({
      reason: `quorum field is not "${NO_QUORUM_OPEN}<heard>${NO_QUORUM_JOIN}<needed>${NO_QUORUM_CLOSE}": "${field}"`,
    },);
  }
  /**
   Heard count the field repeats.
   */
  const repeated = countIn({ field: nonNullishOrThrow(counts[0],), },);
  if (repeated !== heard) {
    throw new TimingFieldError({
      reason: `quorum field counts ${String(repeated,)} heard where the ratio counts ${String(heard,)}`,
    },);
  }
  return countIn({ field: nonNullishOrThrow(counts[1],), },);
}

/**
 Reads one round line, or says the line is not one.

 A COMPLETE LINE ENDS WITH ONE OF TWO FIELDS: the grace a round spent after
 its quorum stood, or `every ask settled` where it never stood (ledger P12).
 A line ending in anything else is a round line a log still being written cut
 off, and says nothing about a round. Reading every line without the grace
 field as cut off once dropped the no-quorum rounds, the ones that lost the
 most voices, from the report (T8, nineteenth batch).

 @param line - one log line

 @returns What the line turned out to say about a round

 @throws TimingLineError when a complete round line's own fields will not
 read, since a round reporting a partial straggler cost is worse than one
 reporting none

 @example
 ```ts
 const reading = readRoundTiming({ line, },);
 ```
 */
export function readRoundTiming(
  { line, }: { readonly line: string; },
): RoundReading {
  /**
   Where the round payload starts, after the tagged prefix and stage label.
   */
  const at = line.indexOf(ROUND_MARKER,);
  if (at === NOT_FOUND)
    return { kind: 'other-line', };

  /**
   Comma-separated fields of the payload, one per reported number.
   */
  const fields = line
    .slice(at + ROUND_MARKER.length,)
    .split(FIELD_SEPARATOR,);

  /**
   Field the payload ends with, which `split` always yields.
   */
  const closing = nonNullishOrThrow(fields.at(-1,),);
  /**
   Whether the round's quorum stood, by the field a complete line ends with.
   */
  const stood = closing.endsWith(GRACE_FIELD,);
  if ((!stood) && (closing !== SETTLED_FIELD))
    return { kind: 'other-line', };

  /**
   Stage label, which is the last word before the marker and always present.
   */
  const stage = nonNullishOrThrow(line
    .slice(
    0,
    at,
    )
    .split(' ',)
    .at(-1,),);

  try {
    if (fields.length !== ROUND_FIELDS) {
      throw new TimingFieldError({
        reason: `round payload has ${String(fields.length,)} ${
          wordForCount({
            count: fields.length,
            one: 'field',
            many: 'fields',
          },)
        } where its writer writes ${String(ROUND_FIELDS,)}`,
      },);
    }
    /**
     Heard and asked counts.
     */
    const {
      heard,
      asked,
    } = ratioIn({ field: nonNullishOrThrow(fields[0],), },);
    /**
     Third field: time to quorum where it stood, the needed count where not.
     */
    const quorumField = nonNullishOrThrow(fields[2],);
    return {
      kind: 'round',
      round: {
        stage,
        heard,
        asked,
        totalMs: durationIn({
          field: nonNullishOrThrow(fields[1],),
          unit: 'ms ',
        },),
        quorum: stood
          ? {
            kind: 'stood',
            toQuorumMs: durationIn({
              field: quorumField,
              unit: 'ms ',
            },),
            inGraceMs: durationIn({
              field: closing,
              unit: 'ms ',
            },),
          }
          : {
            kind: 'never',
            needed: neededIn({
              field: quorumField,
              heard,
            },),
          },
      },
    };
  } catch (error) {
    throw new TimingLineError({
      kind: 'round',
      line,
      cause: error,
    },);
  }
}

/**
 Reads one stream completion line, saying whether it carried a duration and
 a stamp as the logger writes one.

 @param line - one log line

 @returns What the line turned out to say about a call

 @throws TimingLineError when a timed, stamped completion line's own fields
 will not read: a first field that is not `<label>: <outcome>`, or a duration
 not written in whole milliseconds

 @example
 ```ts
 const reading = readCallTiming({ line, },);
 ```
 */
export function readCallTiming(
  { line, }: { readonly line: string; },
): CallReading {
  /**
   Where the stream payload starts, after the tagged prefix.
   */
  const at = line.indexOf(STREAM_MARKER,);
  if (at === NOT_FOUND)
    return { kind: 'other-line', };

  /**
   Timestamp the line was written, which the logger prints second among the
   prefix's bracketed fields; absent from a prefix with one field.
   */
  const stamp = line
    .slice(
    0,
    at,
    )
    .split('] [',)
    .at(1,);

  /**
   Comma-separated fields of the payload, the first pairing label to outcome.
   */
  const fields = line
    .slice(at + STREAM_MARKER.length,)
    .split(FIELD_SEPARATOR,);

  /**
   Second field, the duration on a line written since the timing work.
   */
  const elapsedField = fields.at(1,);
  if ((elapsedField === undefined) || (!elapsedField.startsWith(ELAPSED_FIELD,)))
    return { kind: 'untimed', };

  // A STAMP THE LOGGER DID NOT WRITE PLACES NO CALL: `Date.parse` reads one
  // without its zone as local time and one cut off as NaN, which the sweep
  // cannot order (ledger B73).
  if ((stamp === undefined) || (!isIsoStampText({ text: stamp, },)))
    return { kind: 'unstamped', };

  try {
    /**
     Label and outcome, which `split` always yields as the first field.
     */
    const named = nonNullishOrThrow(fields[0],);
    /**
     Where the outcome's separator starts.
     */
    const join = named.indexOf(OUTCOME_JOIN,);
    /**
     Model the call went to.
     */
    const label = (join === NOT_FOUND)
      ? ''
      : named.slice(
        0,
        join,
      );
    /**
     How the call ended.
     */
    const outcome = (join === NOT_FOUND) ? '' : named.slice(join + OUTCOME_JOIN.length,);
    if ((label === '') || (outcome === '')) {
      throw new TimingFieldError({
        reason: `completion field is not "<label>${OUTCOME_JOIN}<outcome>": "${named}"`,
      },);
    }
    return {
      kind: 'timed',
      call: {
        label,
        outcome,
        endedAt: Date.parse(stamp,),
        elapsedMs: durationIn({
          field: elapsedField.slice(ELAPSED_FIELD.length,),
          unit: 'ms',
        },),
      },
    };
  } catch (error) {
    throw new TimingLineError({
      kind: 'completion',
      line,
      cause: error,
    },);
  }
}

//endregion Run timing parse
