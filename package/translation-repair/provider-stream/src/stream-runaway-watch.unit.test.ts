/**
 Tests for the runaway watch.

 This is the piece the drain calls, so it is tested the way the drain drives
 it: chunk by chunk, stopping at the first runaway verdict rather than reading
 the whole stream and asking afterwards. A watch that only reached the right
 answer at the end would be useless, since the streams it exists to end never
 reach an end.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  StreamDegenerateError,
  watchRunaway,
} from '../dist/final/node/index.mjs';
import {
  frameOf,
  longVariedStream,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/sse-frame.test-fixture.ts';

/**
 Builds internally varied cat-themed text of exactly `length` characters,
 so a block built from it is never itself internally repetitive.

 THE BLOCK MUST BE VARIED or a test built from it measures the block's own
 repetition rather than whatever pattern the test arranges around it: this
 mirrors the probe kept at `~/temp/agent/degeneration-period-probe.mjs`,
 whose first attempt padded a single sentence and proved nothing for
 exactly this reason.

 @param length - exact character count to build

 @param from - starting index the generator counts up from, so two blocks
 built from different `from` values share no content

 @returns Varied text of exactly `length` characters

 @example
 ```ts
 const block = variedBlock({ length: 501, from: 1, },);
 ```
 */
function variedBlock(
  {
    length,
    from,
  }: {
    readonly length: number;
    readonly from: number;
  },
): string {
  /**
   Sentences built so far, joined once at the end.
   */
  const parts: string[] = [];

  /**
   Generator's own counter and how many characters it has produced.
   */
  const cursor = {
    at: from,
    sized: 0,
  };

  while (cursor.sized < length) {
    /**
     One varied sentence, built from the current counter.
     */
    const piece = `Cat ${String(cursor.at,)} inspected shelf ${String((cursor.at * 7) % 991,)} at hour `
      + `${String(cursor.at % 24,)} and reported nothing of note to cat ${String((cursor.at * 13) % 877,)}. `;
    parts.push(piece,);
    cursor.sized += piece.length;
    cursor.at += 1;
  }

  return parts.join('',).slice(
    0,
    length,
  );
}

/**
 Builds text that cycles a varied block of the given period, for exactly
 `total` characters.

 @param period - length of the repeating block

 @param total - exact character count of the whole cycling text

 @returns Text repeating a `period`-character varied block for `total`
 characters

 @example
 ```ts
 const cycling = cyclingText({ period: 501, total: 200_000, },);
 ```
 */
function cyclingText(
  {
    period,
    total,
  }: {
    readonly period: number;
    readonly total: number;
  },
): string {
  /**
   One period's worth of varied text, repeated to cover the whole length.
   */
  const block = variedBlock({
    length: period,
    from: 1,
  },);

  return block.repeat(Math.ceil(total / block.length,),).slice(
    0,
    total,
  );
}

/**
 Wraps long text as many small frames rather than one, so the scanner
 reading complete lines sees it progressively.

 ONE FRAME CANNOT CARRY ARBITRARILY LONG TEXT AND STILL BE READ AS IT
 ARRIVES. `scanStreamDeltas` only extracts a frame's text once it has seen
 that frame's whole line, so a single frame carrying an entire long reply
 would hand every detector the whole reply in one `notifyText` call at the
 very end, which is not how a real stream delivers it and not what any of
 these detectors are checked against. This is the same reason every other
 fixture in this file builds many small frames rather than one large one.

 @param channel - which channel the text arrives on

 @param text - whole text to frame

 @param pieceChars - characters carried by each frame

 @returns Raw stream body, one frame per piece

 @example
 ```ts
 const raw = framedText({ channel: 'reasoning', text, pieceChars: 501, },);
 ```
 */
function framedText(
  {
    channel,
    text,
    pieceChars,
  }: {
    readonly channel: 'content' | 'reasoning';
    readonly text: string;
    readonly pieceChars: number;
  },
): string {
  return Array.from(
    { length: Math.ceil(text.length / pieceChars,), },
    function piece(
      _unused,
      at,
    ): string {
      return frameOf({
        channel,
        text: text.slice(
          at * pieceChars,
          (at + 1) * pieceChars,
        ),
      },);
    },
  ).join('',);
}

/**
 Feeds a raw stream chunk by chunk, stopping at the first runaway verdict.

 @param raw - whole stream body

 @returns Verdict reached, and how much of the stream had been read

 @example
 ```ts
 const { verdict, readBytes, } = drive({ raw, },);
 ```
 */
function drive({ raw, }: { readonly raw: string; },): {
  readonly verdict: ReturnType<ReturnType<typeof watchRunaway>['notifyChunk']>;
  readonly readBytes: number;
} {
  /**
   Watch under test.
   */
  const watch = watchRunaway();

  /**
   Chunk width, near what a socket actually delivers.
   */
  const width = 4_096;

  /**
   Where the read stopped, and what it concluded.
   */
  const outcome = Array.from(
    { length: Math.ceil(raw.length / width,), },
    function at(
      _unused,
      index,
    ): number {
      return index;
    },
  ).reduce(
    function step(
      carried: {
        readonly verdict: ReturnType<ReturnType<typeof watchRunaway>['notifyChunk']>;
        readonly readBytes: number;
      },
      index,
    ) {
      if (carried.verdict.kind === 'runaway')
        return carried;

      return {
        verdict: watch.notifyChunk({
          chunk: raw.slice(
            index * width,
            (index + 1) * width,
          ),
        },),
        readBytes: Math.min(
          (index + 1) * width,
          raw.length,
        ),
      };
    },
    {
      verdict: { kind: 'continuing', } as ReturnType<ReturnType<typeof watchRunaway>['notifyChunk']>,
      readBytes: 0,
    },
  );

  return outcome;
}

await describe({
  name: watchRunaway.name,
  children: [
    it({
      name: 'ENDS A THINKING RUNAWAY BEFORE THE STREAM DOES, naming the reasoning channel: this '
        + 'is the case that produces no answer at all, so nothing downstream would ever notice it',
      fn: async () => {
        /**
         A model that thinks the same sentence forever.
         */
        const raw = Array.from(
          { length: 30_000, },
          function think(): string {
            return frameOf({
              channel: 'reasoning',
              text: 'I will output. ',
            },);
          },
        ).join('',);

        const {
          verdict,
          readBytes,
        } = drive({ raw, },);

        expect(verdict.kind,).toBe('runaway',);
        if (verdict.kind !== 'runaway')
          throw new Error('runaway by construction',);
        expect(verdict.channel,).toBe('reasoning',);
        expect(verdict.distinctRatio,).toBeLessThan(0.1,);

        // Stopped rather than merely diagnosed: the point is to end the call
        // early, so reading the whole body would be a failure even with the
        // right verdict at the end of it.
        expect(readBytes,).toBeLessThan(raw.length,);
      },
    },),

    it({
      name: 'LETS A HEALTHY LONG CALL FINISH, with UNBOUNDED THINKING and an answer inside the '
        + 'volume bound, so a model that simply writes a great deal is never cut off for it. The '
        + 'thinking side is 6000 frames on purpose: a reasoning volume bound was measured and refused, '
        + 'and this is what pins that refusal',
      fn: async () => {
        /**
         Long, varied thinking followed by a long, varied answer.
         */
        const raw = longVariedStream();

        expect(drive({ raw, },).verdict.kind,).toBe('continuing',);
      },
    },),

    it({
      name: 'REFUSES A LONG-PERIOD LOOP PAST THE LENGTH BAR, which the windowed ratio detector '
        + 'cannot see: period 501 measures 0.1223 distinct on that detector alone, above its 0.1 '
        + 'threshold, because gcd(501, 32) leaves only 205 of the trailing sample windows '
        + 'distinct. This is the escape the ratio detector window arithmetic cannot close alone',
      fn: async () => {
        /**
         A model looping a 501-character paragraph forever, well past the
         length bar both detectors share, delivered as many small frames
         so the checks run progressively rather than all at once.
         */
        const raw = framedText({
          channel: 'reasoning',
          text: cyclingText({
            period: 501,
            total: 200_000,
          },),
          pieceChars: 501,
        },);

        const {
          verdict,
          readBytes,
        } = drive({ raw, },);

        expect(verdict.kind,).toBe('runaway',);
        if (verdict.kind !== 'runaway')
          throw new Error('runaway by construction',);
        expect(verdict.channel,).toBe('reasoning',);

        // Ended early rather than only diagnosed after the fact, the same
        // property the ratio-based runaway tests assert.
        expect(readBytes,).toBeLessThan(raw.length,);
      },
    },),

    it({
      name: 'LETS A PERIOD-501 LOOP FINISH WHEN IT NEVER CROSSES THE LENGTH BAR, the same '
        + 'verse-safety guarantee the ratio detector carries: no slice translation this pipeline '
        + 'produces approaches the bar, so a reply this short is never judged by either detector',
      fn: async () => {
        /**
         The same period-501 pattern as the refused case, kept well under
         the length bar.
         */
        const raw = framedText({
          channel: 'reasoning',
          text: cyclingText({
            period: 501,
            total: 100_000,
          },),
          pieceChars: 501,
        },);

        expect(drive({ raw, },).verdict.kind,).toBe('continuing',);
      },
    },),

    it({
      name: 'LETS A CANDIDATE QUOTED TWICE BACK TO BACK FINISH, past the length bar, which is '
        + 'ordinary work in this pipeline reasoning traces: a model restating a whole source '
        + 'slice or candidate a second time must not read as a loop for doing its job',
      fn: async () => {
        /**
         Varied filler well past the length bar, an 8000-character varied
         block quoted twice back to back, and more varied filler after it.
         The duplication happens AFTER the bar is crossed: were it before,
         the bar alone would explain a `continuing` verdict and this test
         would say nothing about the persistence check specifically. At
         8000 characters the candidate is far past the roughly 3072-
         character window where a back-to-back requote can produce any hit
         at all, so the earlier copy has already scrolled out of the
         recurrence buffer entirely by the time the second copy finishes.
         */
        const prefix = variedBlock({
          length: 140_000,
          from: 1,
        },);
        const candidate = variedBlock({
          length: 8_000,
          from: 500_000,
        },);
        const suffix = variedBlock({
          length: 10_000,
          from: 900_000,
        },);

        const raw = framedText({
          channel: 'reasoning',
          text: prefix + candidate + candidate + suffix,
          pieceChars: 500,
        },);

        expect(drive({ raw, },).verdict.kind,).toBe('continuing',);
      },
    },),

    it({
      name: 'LETS A CANDIDATE QUOTED TWICE BACK TO BACK AT EXACTLY THE FALSE-POSITIVE-PRONE '
        + 'LENGTH FINISH, the one candidate length and check phase where a single bounded requote '
        + 'reaches its measured maximum of five consecutive hits: REQUIRED_CONSECUTIVE_HITS is set '
        + 'one past that exact figure so this still reads as continuing rather than runaway',
      fn: async () => {
        /**
         Filler well past the length bar, delivered as one whole frame so
         the recurrence check that fires at the end of it lands on a clean
         phase: sinceLastCheck resets to exactly zero the instant the
         candidate starts arriving, which is what lets `candidate`
         land on the exact worst-case alignment rather than an arbitrary
         one.
         */
        const prefix = variedBlock({
          length: 140_000,
          from: 1,
        },);

        /**
         A candidate of exactly 3072 characters: BUFFER_CHARS (4096) minus
         TAIL_CHARS (1024) in `stream-recurrence-watch.ts`, the one length
         at which a back-to-back requote's consecutive-hit count reaches
         its proven maximum rather than staying below it. Quoted twice back
         to back and delivered one character per frame, so the checks stay
         locked to the exact 512-character grid the whole-frame `prefix`
         established, landing on that worst-case alignment rather
         than being blurred by a coarser frame size that could skip past it.
         */
        const candidate = variedBlock({
          length: 3_072,
          from: 500_000,
        },);
        const suffix = variedBlock({
          length: 2_000,
          from: 900_000,
        },);

        const raw = framedText({
          channel: 'reasoning',
          text: prefix,
          pieceChars: prefix.length,
        },) + framedText({
          channel: 'reasoning',
          text: candidate + candidate + suffix,
          pieceChars: 1,
        },);

        expect(drive({ raw, },).verdict.kind,).toBe('continuing',);
      },
    },),

    it({
      name: 'NAMES THE ANSWER CHANNEL when that is the one repeating, since the two failures call '
        + 'for different reading and pooling them would let either excuse the other',
      fn: async () => {
        const raw = Array.from(
          { length: 30_000, },
          function repeat(): string {
            return frameOf({
              channel: 'content',
              text: 'The cat sat on the mat. ',
            },);
          },
        ).join('',);

        /**
         What the watch concluded.
         */
        const { verdict, } = drive({ raw, },);
        expect(verdict.kind,).toBe('runaway',);
        if (verdict.kind !== 'runaway')
          throw new Error('runaway by construction',);
        expect(verdict.channel,).toBe('content',);
      },
    },),

    it({
      name: 'COUNTS GENERATED CHARACTERS PER CHANNEL, independent of any verdict, so a progress '
        + 'line can report how much the model actually produced rather than repeating the raw '
        + 'stream byte count under a different name',
      fn: async () => {
        const watch = watchRunaway();

        /**
         Two channels' worth of ordinary, non-repeating text, well short of
         anything a verdict would notice.
         */
        const said = 'Whiskers dozed on the windowsill. ';
        const thought = 'Considering whether the shelf holds. ';

        expect(watch.generatedChars(),).toEqual({
          content: 0,
          reasoning: 0,
        },);

        watch.notifyChunk({
          chunk: frameOf({
            channel: 'content',
            text: said,
          },),
        },);
        watch.notifyChunk({
          chunk: frameOf({
            channel: 'reasoning',
            text: thought,
          },),
        },);

        // Counted separately, and neither channel's arrival moves the other's
        // total: pooling them would let a verbose thinking trace hide a silent
        // answer channel, or the reverse.
        expect(watch.generatedChars(),).toEqual({
          content: said.length,
          reasoning: thought.length,
        },);

        watch.notifyChunk({
          chunk: frameOf({
            channel: 'content',
            text: said,
          },),
        },);

        expect(watch.generatedChars(),).toEqual({
          content: said.length * 2,
          reasoning: thought.length,
        },);
      },
    },),

    it({
      name: 'SHOWS GENERATED TEXT IN THE OPENING, not the server-sent-event envelope: a raw '
        + 'excerpt always opens with the JSON wrapper, because every frame carries an identical '
        + 'envelope by construction, and none of the words the model generated are in that wrapper',
      fn: async () => {
        const watch = watchRunaway();

        /**
         What the model actually said.
         */
        const said = 'Whiskers considered the shelf at length. ';

        /**
         One frame as the wire actually sends it, `id` included. Production
         ids are a bare hexadecimal string; this fixture spells it
         `chatcmpl-tabby`, matching the shape recorded in the decision
         document, precisely so that the id would show up in the excerpt if
         this were reading the envelope rather than the generated text.
         */
        const raw = `data: ${
          JSON.stringify({
            id: 'chatcmpl-tabby',
            object: 'chat.completion.chunk',
            choices: [{
              index: 0,
              delta: { content: said, },
              finish_reason: null,
            },],
          },)
        }\n\n`;

        watch.notifyChunk({ chunk: raw, },);

        /**
         What the opening excerpt would show.
         */
        const opening = watch.openingText();

        expect(opening.includes(said,),).toBe(true,);

        // Neither the envelope's own prefix nor the id inside it survives:
        // this reads generated text, not the wire.
        expect(opening.includes('data: {',),).toBe(false,);
        expect(opening.includes('chatcmpl',),).toBe(false,);
      },
    },),

    it({
      name: 'CARRIES WHAT A LOG LINE NEEDS in its error: which channel, how repetitive, what '
        + 'the call had already cost when it was ended, and which model ran away',
      fn: async () => {
        /**
         Error as the drain would raise it.
         */
        const error = new StreamDegenerateError({
          label: 'critic hf:zai-org/GLM-5.3-Flash',
          channel: 'reasoning',
          distinctRatio: 0.0021,
          charsSeen: 412_000,
          rawChars: 412_000,
        },);

        expect(error.name,).toBe('StreamDegenerateError',);
        expect(error.channel,).toBe('reasoning',);
        expect(error.message.includes('reasoning',),).toBe(true,);
        expect(error.message.includes('0.0021',),).toBe(true,);
        expect(error.message.includes('412000',),).toBe(true,);

        // Carried as a property, not only baked into the message: a per-model
        // figure has to read `.label` off every error in a batch, and the
        // message is prose meant for one line of a log rather than for that.
        expect(error.label,).toBe('critic hf:zai-org/GLM-5.3-Flash',);
      },
    },),
  ],
},);
