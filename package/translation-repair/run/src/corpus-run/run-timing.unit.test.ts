/**
 Tests for reading a run's own timing lines back, which is what the timing work built
 so a run could say where its hours went.

 THE OVERLAP CASE IS THE POINT. Achieved concurrency is an overlap count over
 call intervals, and before the timing work a completion line said only when a call
 ended, so no interval existed and the question had no answer. The figures
 asserted here are hand-computed from the fixture rather than recorded from a
 run, so a change in the sweep fails the case instead of moving the target.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { roundLine, } from '@monochromatic-dev/translation-repair-stage';
import { reportStreamProgress, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  type CallTiming,
  measureInFlight,
  NothingInFlightError,
  readCallTiming,
  readRoundTiming,
  readRunTiming,
  summariseRounds,
  TimingFieldError,
  TimingLineError,
} from '../../dist/final/node/index.mjs';
import {
  STAMPS_NOT_WRITTEN,
  WRITTEN_STAMP,
} from '@monochromatic-dev/translation-repair-markup/ts/iso-stamp-text.test-fixture.ts';

//region Fixtures
// Every fixture in this region that stands for a line the current writers
// still produce is BUILT BY CALLING THOSE WRITERS (`reportStreamProgress` in
// `stream-cut.ts`, which returns its own line, and `roundLine` in
// `stage-round-line.ts`, extracted from `runGatherRound` for exactly this),
// the way `cap-census.unit.test.ts`'s `streamLine` already does, rather than
// retyped by hand. Only the logger's own tag-chain prefix is a literal,
// since neither writer controls or returns it.

/**
 A round line as a real corpus pass's repair lane writes it for the editor
 stage, prefix and payload both.

 THE PREFIX IS A TRACED TAG CHAIN, NOT A GUESS (ledger B82). `l` is a
 `ForeignBorrowed` parameter everywhere between `runGatherRound` and its
 caller, so `stage-round.ts` adds no tag of its own; the chain actually
 logged is whichever caller built `l`. Followed from a real corpus pass:
 `corpus-run/pass-lanes.ts` tags the entry (`tagged({ tag: entryId, },)`);
 `document-lanes.ts` wraps that with `tagged({ tag: runDocumentLanes.name,
 },)`; `repair-translation.ts` wraps it again with `tagged({ tag:
 repairPreparedDocument.name, },)` and, through `sliceTagged`, adds whatever
 `<lane> slice <n>` the live log context carries around the slice being
 settled (`log-context.ts`); and `repair-slice-settle.ts`, `repair-chunk.ts`,
 `repair-editor-stage.ts` and `stage-quorum.ts` each forward that same
 logger untouched on the way to `runGatherRound`. `tagged`'s own composition
 puts the outermost wrap's tag closest to the message
 (`module-logger/src/tagged.ts`), so {@link ROUND_LINE}'s tag order is entry,
 then `runDocumentLanes`, then `repairPreparedDocument`, then the slice
 context.

 NO CALLER TAGS A ROUND WITH ITS STAGE NAME. `editor` in the payload is
 `roundLine`'s own `stage` argument, interpolated into the text, not a
 logger tag; grepping this package for `tag: 'editor'` finds nothing. The
 line this fixture replaced had invented an `[editor]` tag that no source
 file writes.

 THE ENTRY NAME AND SLICE INDEX ARE INVENTED (`Tabby`, slice 3): which entry
 and which slice is data a real run supplies, not text any source file
 fixes, so a cat-themed stand-in names them here the way the rest of this
 module's fixtures are cat-themed.
 */
const ROUND_LINE = `[info] [2026-08-25T10:00:30.000Z] [Tabby] [runDocumentLanes] [repairPreparedDocument] [repair slice 3] ${
  roundLine({
    stage: 'editor',
    heard: 6,
    asked: 7,
    totalMs: 91_402,
    quorum: {
      kind: 'stood',
      toQuorumMs: 61_401,
      inGraceMs: 30_001,
    },
  },)
}`;

/**
 A round line off the same traced chain as {@link ROUND_LINE}, when quorum
 never stood (ledger P12): no time to quorum and no grace, since the round
 stopped once every ask settled.
 */
const NO_QUORUM_LINE = `[info] [2026-08-25T10:00:40.000Z] [Tabby] [runDocumentLanes] [repairPreparedDocument] [repair slice 3] ${
  roundLine({
    stage: 'editor',
    heard: 1,
    asked: 7,
    totalMs: 120_000,
    quorum: {
      kind: 'never',
      needed: 4,
    },
  },)
}`;

/**
 A completion line as `reportStreamProgress` writes it since the timing
 work: the logger's own prefix (`contextRoot({ tag: 'translation-repair',
 },)` in `stream-cut.ts`, then `tagged({ tag: reportStreamProgress.name,
 },)`, outside any entry or slice context), then the payload the function
 itself returns.
 */
const TIMED_CALL_LINE = `[info] [${WRITTEN_STAMP}] [translation-repair] [reportStreamProgress] ${
  reportStreamProgress({
    label: 'hf:whiskers',
    progress: {
      firstByteMs: 40,
      firstContentMs: 40,
      maxGapMs: 4,
      chars: 512,
      elapsedMs: 10_000,
    },
    unreadableFrames: 0,
    outcome: 'completed',
    openingText: '',
    generatedChars: {
      content: 40,
      reasoning: 7,
    },
  },)
}`;

/**
 A completion line as every log written before the timing work carries it,
 with no duration anywhere on it.

 DELIBERATELY AN OLD LINE SHAPE, kept as a literal rather than built through
 `reportStreamProgress`: the writer has carried an `elapsed` field on every
 line since the timing work and has no argument that omits it, so this shape
 is one `reportStreamProgress` can no longer produce. It stands for an
 archive written before that work, which `readCallTiming` must still read as
 `untimed` rather than refuse.
 */
const UNTIMED_CALL_LINE = '[info] [2026-08-25T10:00:11.000Z] [translation-repair] '
  + '[reportStreamProgress] stream hf:mittens: completed, firstByte 40ms, maxGap 4ms, '
  + '512 raw chars, 0 unreadable frames, 40 content chars, 7 reasoning chars';

/**
 Base instant the interval fixtures are measured from.
 */
const BASE_MS = Date.parse('2026-08-25T10:00:00.000Z',);

/**
 Milliseconds in a second, so the interval fixtures read as seconds.
 */
const SECOND = 1_000;

/**
 Builds one call interval.

 @param endsAtSeconds - seconds past the base instant the call ended

 @param ranSeconds - how long the call ran

 @returns Call the sweep can read

 @example
 ```ts
 const call = callRunning({ endsAtSeconds: 10, ranSeconds: 10, },);
 ```
 */
function callRunning(
  {
    endsAtSeconds,
    ranSeconds,
  }: {
    readonly endsAtSeconds: number;
    readonly ranSeconds: number;
  },
): CallTiming {
  return {
    label: 'hf:whiskers',
    outcome: 'completed',
    endedAt: BASE_MS + (endsAtSeconds * SECOND),
    elapsedMs: ranSeconds * SECOND,
  };
}

/**
 How the readers name the numbers they take: whole, in digits, and no larger
 than a double holds exactly.
 */
const WHOLE_NUMBER_RULE = `a whole number written in digits, at most ${String(Number.MAX_SAFE_INTEGER,)}`;

/**
 A digit run two past that limit, which `Number` reads as the limit plus one.
 */
const PAST_SAFE = String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,);

/**
 Messages a read threw: its own, then the one it wraps where it wraps one.

 @param read - read that must throw

 @returns Messages, outermost first

 @example
 ```ts
 const texts = refusalTexts({ read: () => readRoundTiming({ line, },), },);
 ```
 */
function refusalTexts({ read, }: { readonly read: () => void; },): readonly string[] {
  /**
   What the read threw.
   */
  const refusal = caught(read,);
  expect(refusal,).toBeInstanceOf(Error,);
  /**
   The error it wraps, where it wraps one.
   */
  const { cause, } = refusal as Error;
  return [
    (refusal as Error).message,
    ...(Error.isError(cause,) ? [cause.message,] : []),
  ];
}

/**
 Refusal texts a round line throws through `readRoundTiming`, the read every
 round-line case in this describe block maps a line list through.

 @param line - round line under test

 @returns Messages the read threw, outermost first

 @example
 ```ts
 const texts = refusalsOfRound(unframedRatio,);
 ```
 */
function refusalsOfRound(line: string,): readonly string[] {
  return refusalTexts({
    read: function readsRound(): void {
      readRoundTiming({ line, },);
    },
  },);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: readRoundTiming.name,
      children: [
        it({
          name: 'reads the stage, the ratio and all three durations off a round line',
          fn: async () => {
            /**
             What the line said.
             */
            const reading = readRoundTiming({ line: ROUND_LINE, },);
            if (reading.kind !== 'round')
              throw new Error('the fixture round line was not read as a round',);

            expect(reading.round,).toEqual({
              stage: 'editor',
              heard: 6,
              asked: 7,
              totalMs: 91_402,
              quorum: {
                kind: 'stood',
                toQuorumMs: 61_401,
                inGraceMs: 30_001,
              },
            },);
          },
        },),

        it({
          name: 'REFUSES A TRUNCATED ROUND LINE rather than reading the fields that survived: a '
            + 'log still being written ends mid-line, and a round whose grace field was cut off '
            + 'would report a straggler cost of whatever the parse happened to reach',
          fn: async () => {
            /**
             {@link ROUND_LINE} cut off mid-field, as a log still being
             written would leave it.
             */
            const truncated = ROUND_LINE.slice(
              0,
              ROUND_LINE.indexOf(' to quorum',) + ' to q'.length,
            );
            expect(readRoundTiming({ line: truncated, },).kind,).toBe('other-line',);
          },
        },),

        it({
          name: 'passes over a line that is not a round at all',
          fn: async () => {
            expect(readRoundTiming({ line: TIMED_CALL_LINE, },).kind,).toBe('other-line',);
          },
        },),

        it({
          name: 'THROWS ON A FIELD THAT LOST ITS UNIT rather than reading it as a zero, since a '
            + 'zero in a timing report is a measurement of nothing happening',
          fn: async () => {
            expect(function readsBroken(): void {
              readRoundTiming({
                // {@link ROUND_LINE} with its total field's unit stripped off.
                line: ROUND_LINE.replace('91402ms total', 'ninety total',),
              },);
            },).toThrow(Error,);
          },
        },),

        it({
          name: 'THROWS ON A RATIO THAT IS NOT TWO WHOLE NUMBERS rather than reading NaN into the '
            + 'heard count, since a round that heard NaN sums into a report as nothing at all',
          fn: async () => {
            expect(function readsWordCount(): void {
              readRoundTiming({
                // {@link ROUND_LINE} with its heard count spelled out.
                line: ROUND_LINE.replace('6/7 heard', 'six/7 heard',),
              },);
            },).toThrow(Error,);
            expect(function readsNoSlash(): void {
              readRoundTiming({
                // {@link ROUND_LINE} with its ratio's slash and asked count dropped.
                line: ROUND_LINE.replace('6/7 heard', '6 heard',),
              },);
            },).toThrow(Error,);
          },
        },),

        it({
          name: 'READS A ROUND WHERE QUORUM NEVER STOOD as the round it is, and keeps a truncated one out: '
            + 'the writer logs it with no grace field, and reading every line without one as cut off '
            + 'dropped the rounds that lost the most voices from the report (T8, nineteenth batch)',
          fn: async () => {
            expect({
              reading: readRoundTiming({ line: NO_QUORUM_LINE, },),
              truncated: readRoundTiming({ line: NO_QUORUM_LINE.slice(0, -'ded), every ask settled'.length,), },)
                .kind,
              rounds: readRunTiming({
                lines: [
                  ROUND_LINE,
                  NO_QUORUM_LINE,
                ],
              },).rounds.length,
            },).toEqual({
              reading: {
                kind: 'round',
                round: {
                  stage: 'editor',
                  heard: 1,
                  asked: 7,
                  totalMs: 120_000,
                  quorum: {
                    kind: 'never',
                    needed: 4,
                  },
                },
              },
              truncated: 'other-line',
              rounds: 2,
            },);
          },
        },),

        it({
          name: 'NAMES WHAT A ROUND LINE LACKS OR CONTRADICTS: a payload short of the four fields its writer writes, '
            + 'and a no-quorum field whose heard count is not the ratio\'s, rather than reading a missing field '
            + 'as one that lost its unit (T8, nineteenth batch)',
          fn: async () => {
            /**
             Round line that kept only its ratio and its grace.
             */
            const short = ROUND_LINE.replace(', 91402ms total, 61401ms to quorum', '',);
            /**
             No-quorum line whose two heard counts disagree.
             */
            const disagreeing = NO_QUORUM_LINE.replace('1/7 heard', '2/7 heard',);
            /**
             No-quorum line whose needed count is a word.
             */
            const wordy = NO_QUORUM_LINE.replace('1 of 4 needed', '1 of four needed',);
            /**
             What reading the short line threw.
             */
            const refusal = caught(function readsShort(): void {
              readRoundTiming({ line: short, },);
            },);
            // The line's refusal and the field's are named classes.
            expect(refusal,).toBeInstanceOf(TimingLineError,);
            expect((refusal as Error).cause,).toBeInstanceOf(TimingFieldError,);
            expect([
              short,
              disagreeing,
              wordy,
            ].map(function refusalsOf(line,): readonly string[] {
              return refusalsOfRound(line,);
            },),).toEqual([
              [
                `round line unreadable: ${short}`,
                'round payload has 2 fields where its writer writes 4',
              ],
              [
                `round line unreadable: ${disagreeing}`,
                'quorum field counts 1 heard where the ratio counts 2',
              ],
              [
                `round line unreadable: ${wordy}`,
                `count field is not ${WHOLE_NUMBER_RULE}: "four"`,
              ],
            ],);
          },
        },),

        it({
          name: 'NAMES A RATIO OR QUORUM FIELD NOT IN ITS WRITER\'S FRAME: a ratio that does not close on "heard", and '
            + 'a no-quorum field without its bracketed counts (T8, nineteenth batch)',
          fn: async () => {
            /**
             Round line whose ratio counts something other than voices heard.
             */
            const unframedRatio = ROUND_LINE.replace('6/7 heard', '6/7 voices',);
            /**
             No-quorum line that lost its bracketed counts.
             */
            const unframedQuorum = NO_QUORUM_LINE.replace('no quorum (1 of 4 needed)', 'no quorum',);
            expect([
              unframedRatio,
              unframedQuorum,
            ].map(function refusalsOf(line,): readonly string[] {
              return refusalsOfRound(line,);
            },),).toEqual([
              [
                `round line unreadable: ${unframedRatio}`,
                'round ratio is not heard/asked: "6/7 voices"',
              ],
              [
                `round line unreadable: ${unframedQuorum}`,
                'quorum field is not "no quorum (<heard> of <needed> needed)": "no quorum"',
              ],
            ],);
          },
        },),

        it({
          name: 'NAMES AN EMPTY COUNT rather than reading a ratio missing its heard count as a round that heard nobody',
          fn: async () => {
            /**
             Round line whose ratio lost its heard count.
             */
            const line = ROUND_LINE.replace('6/7 heard', '/7 heard',);
            expect(refusalTexts({
              read: function readsEmptyHeard(): void {
                readRoundTiming({ line, },);
              },
            },),).toEqual([
              `round line unreadable: ${line}`,
              'count field is empty',
            ],);
          },
        },),

        it({
          name: 'THROWS ON A DURATION NOT WRITTEN IN DIGITS rather than reading an empty one as no time at all, or a '
            + 'signed or exponent one as a time the round line never writes (ledger B73)',
          fn: async () => {
            /**
             Grace fields the round line's writer never writes.
             */
            const graceFields = [
              'ms in grace',
              '-5ms in grace',
              '1e3ms in grace',
            ];
            expect(graceFields.map(function refusalsOf(field,): readonly string[] {
              /**
               Round line carrying that grace field.
               */
              const line = `${ROUND_LINE.slice(0, ROUND_LINE.lastIndexOf(', ',),)}, ${field}`;
              return refusalTexts({
                read: function readsGrace(): void {
                  readRoundTiming({ line, },);
                },
              },);
            },),).toEqual(graceFields.map(function expectedOf(field,): readonly string[] {
              return [
                `round line unreadable: ${ROUND_LINE.slice(0, ROUND_LINE.lastIndexOf(', ',),)}, ${field}`,
                `timing field's milliseconds are not ${WHOLE_NUMBER_RULE}: "${field}"`,
              ];
            },),);
          },
        },),

        it({
          name: 'THROWS ON A COUNT PAST THE LARGEST WHOLE NUMBER A DOUBLE HOLDS EXACTLY, which `Number` reads as a '
            + 'neighbouring number nobody wrote (ledger B73)',
          fn: async () => {
            /**
             Round line whose heard and asked counts lie two past the exact range.
             */
            const line = ROUND_LINE.replace('6/7 heard', `${PAST_SAFE}/${PAST_SAFE} heard`,);
            expect(refusalTexts({
              read: function readsHuge(): void {
                readRoundTiming({ line, },);
              },
            },),).toEqual([
              `round line unreadable: ${line}`,
              `count field is not ${WHOLE_NUMBER_RULE}: "${PAST_SAFE}"`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: readCallTiming.name,
      children: [
        it({
          name: 'reads the label, outcome, end instant and duration off a completion line',
          fn: async () => {
            /**
             What the line said.
             */
            const reading = readCallTiming({ line: TIMED_CALL_LINE, },);
            if (reading.kind !== 'timed')
              throw new Error('the fixture completion line was not read as timed',);

            expect(reading.call.label,).toBe('hf:whiskers',);
            expect(reading.call.outcome,).toBe('completed',);
            expect(reading.call.elapsedMs,).toBe(10_000,);
            expect(reading.call.endedAt,).toBe(Date.parse('2026-08-25T10:00:10.000Z',),);
          },
        },),

        it({
          name: 'SKIPS A COMPLETION LINE THAT PREDATES THE DURATION FIELD, which every log '
            + 'written before the timing work is, so a concurrency is never computed from the half of a '
            + 'mixed archive that happens to be readable',
          fn: async () => {
            expect(readCallTiming({ line: UNTIMED_CALL_LINE, },).kind,).toBe('untimed',);
          },
        },),

        it({
          name: 'passes over a line that is not a completion at all',
          fn: async () => {
            expect(readCallTiming({ line: ROUND_LINE, },).kind,).toBe('other-line',);
          },
        },),

        it({
          name: 'TAKES ONLY A STAMP SPELLED AS THE LOGGER WRITES ONE, so no call is placed at an instant its line '
            + 'never recorded: a stamp without its zone reads as local time, a date alone as midnight, and no stamp '
            + 'as NaN, which no sweep can order (ledger B73)',
          fn: async () => {
            expect(STAMPS_NOT_WRITTEN.map(function kindOf(stamp,): string {
              return readCallTiming({
                line: TIMED_CALL_LINE.replace(
                  WRITTEN_STAMP,
                  stamp,
                ),
              },).kind;
            },),).toEqual(STAMPS_NOT_WRITTEN.map(function unstamped(): string {
              return 'unstamped';
            },),);
          },
        },),

        it({
          name: 'COUNTS A COMPLETION LINE MISSING ITS STAMP FIELD AS UNSTAMPED, and one with no field after its outcome '
            + 'as untimed, so neither is placed at an instant or given a duration its line never wrote',
          fn: async () => {
            expect([
              readCallTiming({
                line: `[info${TIMED_CALL_LINE.slice(TIMED_CALL_LINE.indexOf('] [reportStreamProgress] ',),)}`,
              },).kind,
              readCallTiming({
                line: TIMED_CALL_LINE.slice(0, TIMED_CALL_LINE.indexOf(', ',),),
              },).kind,
            ],).toEqual([
              'unstamped',
              'untimed',
            ],);
          },
        },),

        it({
          name: 'THROWS ON A DURATION WITH NO DIGITS BEFORE ITS UNIT rather than reading it as a call that took no '
            + 'time (ledger B73)',
          fn: async () => {
            /**
             Completion line whose duration lost its digits.
             */
            const line = TIMED_CALL_LINE.replace('elapsed 10000ms', 'elapsed ms',);
            expect(refusalTexts({
              read: function readsUnitAlone(): void {
                readCallTiming({ line, },);
              },
            },),).toEqual([
              `completion line unreadable: ${line}`,
              `timing field's milliseconds are not ${WHOLE_NUMBER_RULE}: "ms"`,
            ],);
          },
        },),

        it({
          name: 'REFUSES A COMPLETION LINE WHOSE FIRST FIELD IS NOT LABEL AND OUTCOME rather than reading a call that '
            + 'ended in no outcome at all (T8, nineteenth batch)',
          fn: async () => {
            /**
             Completion line whose label and outcome lost the colon between them.
             */
            const line = TIMED_CALL_LINE.replace('hf:whiskers: completed', 'hf:whiskers completed',);
            expect(refusalTexts({
              read: function readsNoOutcome(): void {
                readCallTiming({ line, },);
              },
            },),).toEqual([
              `completion line unreadable: ${line}`,
              'completion field is not "<label>: <outcome>": "hf:whiskers completed"',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: readRunTiming.name,
      children: [
        it({
          name: 'SEPARATES ROUNDS FROM CALLS AND COUNTS WHAT IT COULD NOT TIME OR PLACE, so a mixed log '
            + 'reports its own blind spot instead of reporting the readable half as the whole',
          fn: async () => {
            /**
             Every timing line a mixed log holds, one completion stamped as the
             logger never stamps.
             */
            const reading = readRunTiming({
              lines: [
                ROUND_LINE,
                TIMED_CALL_LINE,
                UNTIMED_CALL_LINE,
                TIMED_CALL_LINE.replace(
                  WRITTEN_STAMP,
                  STAMPS_NOT_WRITTEN[0],
                ),
                'a line about nothing in particular',
              ],
            },);

            expect(reading.rounds.length,).toBe(1,);
            expect(reading.calls.length,).toBe(1,);
            expect(reading.callsWithoutDuration,).toBe(1,);
            expect(reading.callsWithoutStamp,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: summariseRounds.name,
      children: [
        it({
          name: 'COUNTS A ROUND WHOSE QUORUM NEVER STOOD in the time and the lost voices, and in grace only where '
            + 'quorum stood, rather than leaving the round out of every total (T8, nineteenth batch)',
          fn: async () => {
            /**
             One round of each kind, read off the lines their writer writes.
             */
            const { rounds, } = readRunTiming({
              lines: [
                ROUND_LINE,
                NO_QUORUM_LINE,
              ],
            },);
            // 91402 + 120000 total; grace only from the round whose quorum
            // stood; (7 - 6) + (7 - 1) voices never heard.
            expect(summariseRounds({ rounds, },),).toEqual({
              rounds: 2,
              withoutQuorum: 1,
              totalMs: 211_402,
              graceMs: 30_001,
              lost: 7,
            },);
          },
        },),
      ],
    },),

    describe({
      name: measureInFlight.name,
      children: [
        it({
          name: 'COUNTS OVERLAP RATHER THAN AVERAGING ANYTHING: three calls at [0,10], [2,8] and '
            + '[15,20] seconds put two in flight at once and leave the run idle between, which no '
            + 'summed duration divided by a span could tell apart from steady single-file work',
          fn: async () => {
            /**
             Hand-computed fixture: peak 2 during [2,8], idle during [10,15].
             */
            const flight = measureInFlight({
              calls: [
                callRunning({ endsAtSeconds: 10, ranSeconds: 10, },),
                callRunning({ endsAtSeconds: 8, ranSeconds: 6, },),
                callRunning({ endsAtSeconds: 20, ranSeconds: 5, },),
              ],
            },);

            expect(flight.spanMs,).toBe(20 * SECOND,);
            expect(flight.busyMs,).toBe(21 * SECOND,);
            expect(flight.peakInFlight,).toBe(2,);
            // 21 call-seconds across a 20 second span.
            expect(flight.meanInFlight,).toBeCloseTo(1.05, 2,);
          },
        },),

        it({
          name: 'DOES NOT COUNT TWO CALLS THAT MERELY ABUT AS OVERLAPPING, which is the whole '
            + 'difference between a run that fans out and one that works strictly single-file',
          fn: async () => {
            /**
             Back-to-back calls: one ends exactly where the next begins.
             */
            const flight = measureInFlight({
              calls: [
                callRunning({ endsAtSeconds: 10, ranSeconds: 10, },),
                callRunning({ endsAtSeconds: 20, ranSeconds: 10, },),
              ],
            },);

            expect(flight.peakInFlight,).toBe(1,);
            expect(flight.meanInFlight,).toBeCloseTo(1, 2,);
          },
        },),

        it({
          name: 'REFUSES CALLS THAT SPAN NO TIME rather than reporting every one of them in flight at a mean while '
            + 'the sweep counts a peak of none: a span of zero is the empty list\'s division by nothing (T8, '
            + 'nineteenth batch)',
          fn: async () => {
            expect(refusalTexts({
              read: function measuresInstant(): void {
                measureInFlight({
                  calls: [
                    callRunning({ endsAtSeconds: 10, ranSeconds: 0, },),
                    callRunning({ endsAtSeconds: 10, ranSeconds: 0, },),
                  ],
                },);
              },
            },),).toEqual(['every timed call in this log took no time, so no span holds a call in flight',],);
            expect(caught(function measuresOneInstant(): void {
              measureInFlight({ calls: [callRunning({ endsAtSeconds: 10, ranSeconds: 0, },),], },);
            },),).toBeInstanceOf(NothingInFlightError,);
          },
        },),

        it({
          name: 'REFUSES AN EMPTY CALL LIST rather than reporting a mean of zero in flight, which '
            + 'a log that simply predates the duration field would otherwise produce',
          fn: async () => {
            /**
             What measuring no call threw.
             */
            const refusal = caught(function measuresNothing(): void {
              measureInFlight({ calls: [], },);
            },);
            expect(refusal,).toBeInstanceOf(NothingInFlightError,);
            expect((refusal as Error).message,).toBe(
              'no call in this log carried a duration, so nothing can be counted in flight',
            );
          },
        },),
      ],
    },),
  ],
},);
