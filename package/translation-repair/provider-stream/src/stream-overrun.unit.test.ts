/**
 Tests for the content volume bound and the shared self-ended predicate.

 The bound exists because repetition cannot see this failure: a model writing
 ten times more answer than any legitimate call is not repeating itself, so
 every window is distinct and both detectors report a healthy stream while it
 runs to its deadline.

 The reasoning half of the same idea was measured and REFUSED, and one test
 here pins that refusal so it cannot be reintroduced by someone reading only
 the symmetry. `doc/decision/translation-repair-runaway-call-termination.md`
 carries the numbers.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeAbandon,
  isSelfEndedStream,
  StreamCutShortError,
  StreamDegenerateError,
  StreamOverrunError,
  watchRunaway,
} from '../dist/final/node/index.mjs';
import { frameOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/sse-frame.test-fixture.ts';

/**
 Builds internally varied cat-themed text of at least `length` characters.

 VARIED BY CONSTRUCTION, so a volume test measures volume rather than
 tripping the repetition detectors on the way past the bound. A padded single
 sentence would end the stream as degenerate long before the bound was
 reached, and the test would pass while proving nothing about volume.

 @param length - characters to reach

 @returns Varied text of at least that many characters

 @example
 ```ts
 const answer = variedText({ length: 11_000, },);
 ```
 */
function variedText({ length, }: { readonly length: number; },): string {
  /**
   Sentences built so far, joined once at the end.
   */
  const parts: string[] = [];

  /**
   Generator's counter and how many characters it has produced.
   */
  const cursor = {
    at: 1,
    sized: 0,
  };

  while (cursor.sized < length) {
    /**
     One varied sentence, built from the current counter.
     */
    const piece = `Cat ${String(cursor.at,)} moved basket ${String((cursor.at * 7) % 991,)} on day `
      + `${String(cursor.at % 31,)} while cat ${String((cursor.at * 13) % 877,)} watched the window. `;
    parts.push(piece,);
    cursor.sized += piece.length;
    cursor.at += 1;
  }

  return parts.join('',);
}

/**
 Feeds text to a watch on one channel and returns the last verdict.

 FED IN PIECES, the way the drain feeds it, so a bound that only reads a
 whole-stream total would fail here rather than pass.

 @param watch - watch under test

 @param channel - channel to feed

 @param text - text to deliver

 @param pieceSize - characters per frame

 @returns Verdict after the last piece, or the first non-continuing one

 @example
 ```ts
 const verdict = feed({ watch, channel: 'content', text, pieceSize: 500, },);
 ```
 */
function feed(
  {
    watch,
    channel,
    text,
    pieceSize,
  }: {
    readonly watch: ReturnType<typeof watchRunaway>;
    readonly channel: 'content' | 'reasoning';
    readonly text: string;
    readonly pieceSize: number;
  },
): ReturnType<ReturnType<typeof watchRunaway>['notifyChunk']> {
  /**
   Last verdict seen, kept in a record so the function root holds no
   reassigned binding.
   */
  const seen = { verdict: { kind: 'continuing', } as ReturnType<ReturnType<typeof watchRunaway>['notifyChunk']>, };

  for (let at = 0; at < text.length; at += pieceSize) {
    seen.verdict = watch.notifyChunk({
      chunk: frameOf({
        channel,
        text: text.slice(at, at + pieceSize,),
      },),
    },);
    if (seen.verdict.kind !== 'continuing')
      return seen.verdict;
  }

  return seen.verdict;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'the content volume bound',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES AN ANSWER THAT RUNS PAST THE BOUND, naming the channel and what it cost, which is '
            + 'the failure repetition cannot see: every window of it is distinct',
          fn: async () => {
            const watch = watchRunaway();
            const verdict = feed({
              watch,
              channel: 'content',
              text: variedText({ length: 34_000, },),
              pieceSize: 400,
            },);

            expect(verdict.kind,).toBe('overrun',);
            if (verdict.kind !== 'overrun')
              throw new Error('overrun by construction',);
            expect(verdict.channel,).toBe('content',);
            expect(verdict.cap,).toBe(32_000,);
            expect(verdict.charsSeen,).toBeGreaterThanOrEqual(32_000,);
          },
        },),

        it({
          name: 'ACCEPTS THE LARGEST LEGITIMATE ANSWER ON RECORD, 11392 characters of picture '
            + 'transcription. This call is why the bound is not ten thousand: the population the first '
            + 'bound was set on held no reading-lane call, and ten thousand would have ended seven of '
            + 'the 1887 real completions that have since been pooled',
          fn: async () => {
            const watch = watchRunaway();
            const verdict = feed({
              watch,
              channel: 'content',
              text: variedText({ length: 11_392, },),
              pieceSize: 400,
            },);

            expect(verdict.kind,).toBe('continuing',);
          },
        },),

        it({
          name: 'ACCEPTS SIXTY-FOUR THOUSAND SILENT REASONING CHARACTERS, the bound that was measured and '
            + 'refused. Reasoning precedes content on every thinking model here, so a bound on silent '
            + 'reasoning fires mid-stream on calls that were about to answer: across 1887 real completions '
            + 'forty thousand would have ended 24 of them and even sixty thousand would have ended 5. The '
            + 'largest legitimate completion carried 64501 reasoning characters and then answered',
          fn: async () => {
            const watch = watchRunaway();
            const verdict = feed({
              watch,
              channel: 'reasoning',
              text: variedText({ length: 64_501, },),
              pieceSize: 500,
            },);

            expect(verdict.kind,).toBe('continuing',);
            expect(watch.generatedChars().reasoning,).toBeGreaterThanOrEqual(64_501,);
            expect(watch.generatedChars().content,).toBe(0,);
          },
        },),

        it({
          name: 'REACHES A REPETITIVE ANSWER FOUR TIMES EARLIER THAN REPETITION DOES, and so relabels '
            + 'it. Both observed repetition endings on the answer channel were called degenerate only '
            + 'after 131078 content characters, so the volume bound gets there first and they now read '
            + 'overrun. The same call is ended either way, far sooner, which is the trade this records',
          fn: async () => {
            const watch = watchRunaway();

            /**
             A model repeating one sentence into the answer, the shape that used
             to run to 131078 characters before either detector called it.
             */
            const repeated = 'The cat sat on the mat and said nothing at all. '.repeat(800,);

            expect(repeated.length,).toBeGreaterThan(32_000,);

            const verdict = feed({
              watch,
              channel: 'content',
              text: repeated,
              pieceSize: 400,
            },);

            expect(verdict.kind,).toBe('overrun',);
            if (verdict.kind !== 'overrun')
              throw new Error('overrun by construction',);
            expect(verdict.charsSeen,).toBeLessThanOrEqual(33_000,);
          },
        },),

        it({
          name: 'LEAVES THE THINKING CHANNEL TO THE REPETITION DETECTORS, reporting runaway rather than '
            + 'overrun there, because no volume bound applies to reasoning and that is where the guard '
            + 'was measured to belong',
          fn: async () => {
            const watch = watchRunaway();

            /**
             A model thinking one sentence forever, which is the case the ratio
             detector was built for and still owns.

             SIZED PAST 131072 CHARACTERS on purpose: that is where the detector
             actually fires, measured, and it is the same figure the two real
             degenerate calls reached before anything stopped them.
             */
            const verdict = feed({
              watch,
              channel: 'reasoning',
              text: 'I will output. '.repeat(12_000,),
              pieceSize: 4_096,
            },);

            expect(verdict.kind,).toBe('runaway',);
            if (verdict.kind !== 'runaway')
              throw new Error('runaway by construction',);
            expect(verdict.channel,).toBe('reasoning',);
          },
        },),

        it({
          name: 'TAKES A BOUND FROM THE CALL SITE, since the measurement behind the default came from one '
            + 'bed and a role that legitimately writes more should be able to say so',
          fn: async () => {
            const watch = watchRunaway({ contentCap: 900, },);
            const verdict = feed({
              watch,
              channel: 'content',
              text: variedText({ length: 1_400, },),
              pieceSize: 100,
            },);

            expect(verdict.kind,).toBe('overrun',);
            if (verdict.kind !== 'overrun')
              throw new Error('overrun by construction',);
            expect(verdict.cap,).toBe(900,);
          },
        },),
      ],
    },),

    describe({
      name: isSelfEndedStream.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES BOTH GUARD ERRORS, which is the property the shared predicate exists for: a retry ladder that '
            + 'knew about one class re-bought the runaway once per remaining attempt, five transport calls '
            + 'over twelve seconds of backoff',
          fn: async () => {
            expect(
              isSelfEndedStream({
                error: new StreamDegenerateError({
                  label: 'critic',
                  channel: 'reasoning',
                  distinctRatio: 0.02,
                  charsSeen: 400_000,
                  rawChars: 400_000,
                },),
              },),
            ).toBe(true,);

            expect(
              isSelfEndedStream({
                error: new StreamOverrunError({
                  label: 'editor',
                  channel: 'content',
                  charsSeen: 40_000,
                  rawChars: 40_000,
                  cap: 32_000,
                },),
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a stall and an ordinary failure, which are weather rather than a decision of ours: '
            + 'a stall is worth retrying and this predicate exists to say what is not',
          fn: async () => {
            expect(isSelfEndedStream({ error: new Error('socket closed',), },),).toBe(false,);

            expect(
              isSelfEndedStream({
                error: new StreamCutShortError({
                  label: 'judge',
                  partialText: 'The cat ',
                  progress: {
                    firstByteMs: 40,
                    firstContentMs: 40,
                    maxGapMs: 900,
                    chars: 8,
                    elapsedMs: 1_400,
                  },
                  cause: new Error('socket closed',),
                },),
              },),
            ).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: describeAbandon.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES AN OVERRUN BY ITS VOLUME AND ITS BOUND rather than as a repetition, so a log grouped '
            + 'by cause does not report a distinct-window share that was never computed',
          fn: async () => {
            /**
             How the log line reads for a call the volume bound ended.
             */
            const described = describeAbandon({
              error: new StreamOverrunError({
                label: 'editor',
                channel: 'content',
                charsSeen: 40_000,
                rawChars: 40_000,
                cap: 32_000,
              },),
            },);

            expect(described.includes('40000',),).toBe(true,);
            expect(described.includes('32000',),).toBe(true,);
            expect(described.includes('content',),).toBe(true,);
            expect(described.includes('distinct',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
