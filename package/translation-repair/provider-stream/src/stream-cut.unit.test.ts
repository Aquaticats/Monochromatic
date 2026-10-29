/**
 Tests for the stream progress report, the one place that says how a call
 ended.

 TESTED THROUGH ITS RETURN VALUE rather than by capturing a logger's side
 effect: `reportStreamProgress` hands back the exact line it logs, so an
 assertion here reads as a statement about the LINE rather than about
 whatever the logging subsystem happened to do with it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportStreamProgress, } from '../dist/final/node/index.mjs';

/**
 Progress a call made before it stopped, plain numbers standing in for what
 the idle guard would have measured.
 */
const SOME_PROGRESS = {
  firstByteMs: 40,
  firstContentMs: 55,
  maxGapMs: 4,
  chars: 512,
  elapsedMs: 830,
};

/**
 Generated character totals a call made before it stopped, deliberately
 unequal to `SOME_PROGRESS.chars` and to each other, so no assertion in this
 file could pass by two counts coinciding.
 */
const SOME_GENERATED_CHARS = {
  content: 40,
  reasoning: 7,
};

await describe({
  name: reportStreamProgress.name,
  children: [
    it({
      name: 'NAMES A TERMINATION THIS SYSTEM CHOSE AS ITS OWN OUTCOME, distinct from a stall or '
        + 'steering: a stall is worth retrying and a model that has begun repeating itself will '
        + 'repeat itself again, so a reader counting `cut` lines to measure stalls must not count '
        + 'a deliberate ending among them',
      fn: async () => {
        /**
         The line a genuine cut logs.
         */
        const cutLine = reportStreamProgress({
          label: 'hf:whiskers',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'cut',
          openingText: 'It is a cat. It did a backflip. It cras',
          generatedChars: SOME_GENERATED_CHARS,
        },);

        /**
         The line this guard's own termination logs, everything else equal.
         */
        const degenerateLine = reportStreamProgress({
          label: 'hf:whiskers',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'degenerate',
          openingText: 'The cat sat on the mat. '.repeat(20,),
          generatedChars: SOME_GENERATED_CHARS,
        },);

        expect(cutLine.includes('hf:whiskers: cut,',),).toBe(true,);
        expect(degenerateLine.includes('hf:whiskers: degenerate,',),).toBe(true,);

        // Neither word appears on the other line: they are different
        // outcomes, not one outcome spelled two ways.
        expect(cutLine.includes('degenerate',),).toBe(false,);
        expect(degenerateLine.includes(': cut,',),).toBe(false,);
      },
    },),

    it({
      name: 'NAMES THE UPSTREAM THAT SERVED THE STREAM when the wire named one, quoted like the '
        + 'excerpt, and leaves the line unchanged when it named none, so a cut on a many-endpoint '
        + 'provider is attributable and a single-upstream provider\'s lines keep their shape',
      fn: async () => {
        /**
         The line a cut served by a named upstream logs.
         */
        const attributed = reportStreamProgress({
          label: 'minimax/minimax-m3',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'cut',
          openingText: '{"translation":',
          generatedChars: SOME_GENERATED_CHARS,
          servedBy: 'Parasail',
        },);
        expect(attributed.includes(', served by "Parasail", opening ',),).toBe(true,);

        /**
         The same line with the wire naming nobody.
         */
        const unattributed = reportStreamProgress({
          label: 'minimax/minimax-m3',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'cut',
          openingText: '{"translation":',
          generatedChars: SOME_GENERATED_CHARS,
          servedBy: '',
        },);
        expect(unattributed.includes('served by',),).toBe(false,);
        expect(unattributed,).toBe(reportStreamProgress({
          label: 'minimax/minimax-m3',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'cut',
          openingText: '{"translation":',
          generatedChars: SOME_GENERATED_CHARS,
        },),);
      },
    },),

    it({
      name: 'SHOWS THE OPENING EXCERPT ON A DEGENERATE ENDING, not only on a cut, because seeing '
        + 'what the model was saying when it started repeating is exactly as diagnostic as seeing '
        + 'what it was saying when the connection dropped',
      fn: async () => {
        /**
         Line for a call this guard ended.
         */
        const line = reportStreamProgress({
          label: 'hf:whiskers',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'degenerate',
          openingText: 'The cat sat on the mat. '.repeat(20,),
          generatedChars: SOME_GENERATED_CHARS,
        },);

        expect(line.includes('opening',),).toBe(true,);
      },
    },),

    it({
      name: 'OMITS THE OPENING EXCERPT ON A CLEAN FINISH, since what a healthy reply said is not a '
        + 'diagnostic question and every finished call would otherwise carry one',
      fn: async () => {
        /**
         Line for a call that simply finished.
         */
        const line = reportStreamProgress({
          label: 'hf:mittens',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'completed',
          openingText: 'A tabby naps in the window.',
          generatedChars: SOME_GENERATED_CHARS,
        },);

        expect(line.includes('opening',),).toBe(false,);
      },
    },),

    it({
      name: 'REPORTS GENERATED CHARACTERS PER CHANNEL rather than repeating the raw byte count '
        + 'under a new name: `progress.chars` and `generatedChars` are given deliberately unequal '
        + 'values here, and the line must show the generated ones, not the raw one, for the count '
        + 'beside `raw chars`',
      fn: async () => {
        const line = reportStreamProgress({
          label: 'hf:whiskers',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'completed',
          openingText: 'whatever the model actually said, irrelevant to this count',
          generatedChars: SOME_GENERATED_CHARS,
        },);

        expect(line.includes(`${String(SOME_PROGRESS.chars,)} raw chars`,),).toBe(true,);
        expect(line.includes(`${String(SOME_GENERATED_CHARS.content,)} content chars`,),).toBe(true,);
        expect(line.includes(`${String(SOME_GENERATED_CHARS.reasoning,)} reasoning chars`,),).toBe(true,);

        // The old field is gone rather than renamed alongside the new ones:
        // a reader grepping for "delivered chars" must find nothing, not a
        // second copy of the raw count.
        expect(line.includes('delivered chars',),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS HOW LONG THE CALL TOOK, in the same line as its outcome: a log of phase timings '
        + 'and character counts cannot answer where a pass spent its hours, and a search found no '
        + 'line anywhere that carried a call duration',
      fn: async () => {
        /**
         A completion line, the shape every settled call leaves behind.
         */
        const line = reportStreamProgress({
          label: 'hf:whiskers',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'completed',
          openingText: '',
          generatedChars: SOME_GENERATED_CHARS,
        },);

        // The value, not merely the word: 830 appears in no other fixture
        // field, so this cannot pass by reading a neighbouring count.
        expect(line.includes(`elapsed ${String(SOME_PROGRESS.elapsedMs,)}ms`,),).toBe(true,);
      },
    },),

    it({
      name: 'SAYS HOW LONG THE FIRST TEXT TOOK beside the first byte (ledger B804): the first byte is only a lower '
        + 'bound on the wait the first-content window bounds, so the window cannot be set against a log that states '
        + 'the byte alone',
      fn: async () => {
        /**
         A completion line whose first text came later than its first byte.
         */
        const line = reportStreamProgress({
          label: 'hf:whiskers',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'completed',
          openingText: '',
          generatedChars: SOME_GENERATED_CHARS,
        },);

        // 40 and 55 are distinct, so each reading is told from the other.
        expect(line.includes('firstByte 40ms, firstContent 55ms, maxGap 4ms',),).toBe(true,);
      },
    },),

    it({
      name: 'STATES A FIRST TEXT THAT NEVER CAME as a negative reading, as the first byte does when no byte came',
      fn: async () => {
        /**
         A cut line of a stream that sent bytes and no text.
         */
        const line = reportStreamProgress({
          label: 'hf:whiskers',
          progress: {
            ...SOME_PROGRESS,
            firstContentMs: -1,
          },
          unreadableFrames: 0,
          outcome: 'cut',
          openingText: '',
          generatedChars: {
            content: 0,
            reasoning: 0,
          },
        },);
        expect(line.includes('firstByte 40ms, firstContent -1ms,',),).toBe(true,);
      },
    },),

    it({
      name: 'SHOWS EXACTLY THE OPENING TEXT IT WAS GIVEN, verbatim, in the excerpt: this function '
        + 'no longer has a raw-text parameter to fall back to, so a caller passing generated text '
        + 'is the only source the excerpt can possibly show',
      fn: async () => {
        /**
         Text that could not plausibly be mistaken for a server-sent-event
         envelope, so its presence in the excerpt is unambiguous.
         */
        const said = 'Whiskers considered the shelf at length before deciding against it.';

        const line = reportStreamProgress({
          label: 'hf:whiskers',
          progress: SOME_PROGRESS,
          unreadableFrames: 0,
          outcome: 'cut',
          openingText: said,
          generatedChars: SOME_GENERATED_CHARS,
        },);

        expect(line.includes(said,),).toBe(true,);
        expect(line.includes('data: {',),).toBe(false,);
      },
    },),
  ],
},);
