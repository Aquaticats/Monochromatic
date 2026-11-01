/**
 Tests for the degeneration detector.

 The cases that matter are the two ways this can be wrong in production, and
 they pull in opposite directions. Missing a cycling model leaves the failure
 this exists to stop; calling healthy output degenerate aborts good work and
 costs a voice, which is the exact harm the straggler-grace decision spent a
 whole document avoiding.

 So the false-positive cases carry as much weight here as the detection ones,
 and three of them are real rather than imagined. Stages ask for structured
 replies, so repeated field names are ordinary output. Some models simply
 write a great deal, so length must never condemn on its own. And this corpus
 contains verse, so a refrain is content rather than a symptom.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { watchForDegeneration, } from '../dist/final/node/index.mjs';

/**
 Feeds text to a fresh detector and reads what it says.

 @param chunks - text pieces, in arrival order

 @returns Verdict after the last piece

 @example
 ```ts
 const verdict = verdictAfter({ chunks: ['a cat ', 'and a mat',], },);
 ```
 */
function verdictAfter({ chunks, }: { readonly chunks: readonly string[]; },): ReturnType<
  ReturnType<typeof watchForDegeneration>['verdict']
  > {
  /**
   Detector under test.
   */
  const detector = watchForDegeneration();
  chunks.forEach(function feed(text,): void {
    detector.notifyText({ text, },);
  },);
  return detector.verdict();
}

/**
 Builds varied prose, every sentence differing from every other.

 @param lines - how many sentences to write

 @returns Text with no repetition in it

 @example
 ```ts
 const prose = variedProse({ lines: 2_000, },);
 ```
 */
function variedProse({ lines, }: { readonly lines: number; },): string {
  return Array.from(
    { length: lines, },
    function sentence(
      _unused,
      at,
    ): string {
      return `Tabby number ${String(at,)} climbed the ${String(at * 3,)}th shelf `
        + `and knocked down ${String(at % 7,)} jars before napping for ${String(at % 13,)} hours. `;
    },
  ).join('',);
}

/**
 Splits text into small pieces, the way a network delivers it.

 @param text - whole reply

 @param size - characters per piece

 @returns Pieces in order

 @example
 ```ts
 const pieces = inPieces({ text, size: 17, },);
 ```
 */
function inPieces(
  {
    text,
    size,
  }: {
    readonly text: string;
    readonly size: number;
  },
): readonly string[] {
  return Array.from(
    { length: Math.ceil(text.length / size,), },
    function piece(
      _unused,
      at,
    ): string {
      return text.slice(
        at * size,
        (at + 1) * size,
      );
    },
  );
}

await describe({
  name: watchForDegeneration.name,
  children: [
    it({
      name: 'WITHHOLDS a verdict on an ordinary reply, because length is not the signal and a '
        + 'short answer carries too few windows for the ratio to mean anything',
      fn: async () => {
        expect(verdictAfter({ chunks: ['The cat naps. ',], },).kind,).toBe('undecided',);
        expect(verdictAfter({ chunks: [variedProse({ lines: 300, },),], },).kind,).toBe('undecided',);
      },
    },),

    it({
      name: 'ACCEPTS long varied prose, the negative control: some models legitimately write a '
        + 'great deal, and if verbosity alone ever read as degenerate this guard would abort the '
        + 'most productive calls in the run',
      fn: async () => {
        expect(verdictAfter({ chunks: [variedProse({ lines: 2_000, },),], },).kind,).toBe('healthy',);
      },
    },),

    it({
      name: 'REFUSES a stream cycling on one phrase, the failure the provider does not end and '
        + 'that no token cap bounds, since none is sent',
      fn: async () => {
        /**
         A reply that says one thing forever.
         */
        const verdict = verdictAfter({
          chunks: ['The cat sat on the mat. '.repeat(8_000,),],
        },);
        expect(verdict.kind,).toBe('degenerate',);
        if (verdict.kind !== 'degenerate')
          throw new Error('degenerate by construction',);
        expect(verdict.distinctRatio,).toBeLessThan(0.1,);
        expect(verdict.charsSeen,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'REFUSES a two-phrase cycle and a whitespace runaway, so the detector is not fooled '
        + 'by degeneration that never repeats a single phrase twice in a row',
      fn: async () => {
        expect(verdictAfter({ chunks: ['The cat sat. The mat sat. '.repeat(8_000,),], },).kind,)
          .toBe('degenerate',);
        expect(verdictAfter({ chunks: [`The cat naps.${' '.repeat(300_000,)}`,], },).kind,)
          .toBe('degenerate',);
      },
    },),

    it({
      name: 'REFUSES a stream that degenerates only AFTER a healthy opening, which is why the '
        + 'sample trails: a cumulative ratio over a long good beginning can never fall far enough '
        + 'to trip, however long the model then cycles',
      fn: async () => {
        expect(
          verdictAfter({
            chunks: [
              variedProse({ lines: 3_000, },),
              'A ginger cat batted the same bell over and over. '.repeat(8_000,),
            ],
          },).kind,
        ).toBe('degenerate',);
      },
    },),

    it({
      name: 'ACCEPTS structured output with repeated keys, a realistic false positive: stages ask '
        + 'for structured replies, so a reply full of identical field names is ordinary output',
      fn: async () => {
        /**
         A structured reply shaped like the ones stages request, every value
         different and every key the same.
         */
        const structured = `{"findings":[${
          Array.from(
            { length: 1_500, },
            function finding(
              _unused,
              at,
            ): string {
              return `{"index":${String(at,)},"kind":"whisker-drift","evidence":"the tabby on shelf `
                + `${String(at,)} was described as ${String(at % 5,)} years old","confidence":0.${String(at % 90,)}}`;
            },
          ).join(',',)
        }]}`;

        expect(verdictAfter({ chunks: [structured, ], },).kind,).toBe('healthy',);
      },
    },),

    it({
      name: 'ACCEPTS boilerplate repeated with one varying token, which scores far nearer the '
        + 'threshold than prose does and is the case that sets how much margin there really is',
      fn: async () => {
        /**
         The same note over and over, differing only in its number.
         */
        const boilerplate = Array.from(
          { length: 1_400, },
          function note(
            _unused,
            at,
          ): string {
            return 'Note: this passage concerns the household cat and its habits, which the '
              + 'translator has rendered faithfully without adding detail, omitting nothing, and '
              + `preserving the register of the original throughout. Variant ${String(at,)}. `;
          },
        ).join('',);

        expect(verdictAfter({ chunks: [boilerplate,], },).kind,).toBe('healthy',);
      },
    },),

    it({
      name: 'WITHHOLDS on verse carrying a refrain, the false positive this corpus would actually '
        + 'have suffered: a repeated line is content, and measured alone the ratio condemns it, so '
        + 'what protects it is that no slice translation is ever long enough to be judged',
      fn: async () => {
        /**
         A poem whose refrain returns every stanza.
         */
        const verse = Array.from(
          { length: 800, },
          function stanza(
            _unused,
            at,
          ): string {
            /**
             Where the cat waits this time, so the stanzas are not identical.
             */
            const place = [
              'window',
              'doorway',
              'stairwell',
              'garden',
            ][at % 4] ?? 'window';

            return `The grey cat waits by the ${place} at dusk,\nand no one calls her name.\n`
              + 'Still the lamps come on, still the kettle sings,\nand no one calls her name.\n';
          },
        ).join('',);

        expect(verdictAfter({ chunks: [verse,], },).kind,).toBe('undecided',);
      },
    },),

    it({
      name: 'READS THE SAME VERDICT however the text is split, since chunk boundaries are an '
        + 'accident of the network and a detector that saw them would report differently on '
        + 'identical output',
      fn: async () => {
        /**
         One cycling reply.
         */
        const cycling = 'A calico chased its tail in the hallway. '.repeat(6_000,);
        expect(verdictAfter({ chunks: [cycling,], },).kind,).toBe('degenerate',);
        expect(
          verdictAfter({
            chunks: inPieces({
              text: cycling,
              size: 17,
            },),
          },).kind,
        ).toBe('degenerate',);

        /**
         One healthy reply.
         */
        const prose = variedProse({ lines: 2_000, },);
        expect(verdictAfter({ chunks: [prose,], },).kind,).toBe('healthy',);
        expect(
          verdictAfter({
            chunks: inPieces({
              text: prose,
              size: 23,
            },),
          },).kind,
        ).toBe('healthy',);
      },
    },),

    it({
      name: 'IGNORES empty arrivals, which a stream produces at its end and which must not count '
        + 'toward the sample',
      fn: async () => {
        /**
         Detector fed nothing but empty strings.
         */
        const detector = watchForDegeneration();
        detector.notifyText({ text: '', },);
        detector.notifyText({ text: '', },);

        /**
         What it says with no text at all.
         */
        const verdict = detector.verdict();
        expect(verdict.kind,).toBe('undecided',);
        if (verdict.kind !== 'undecided')
          throw new Error('undecided by construction',);
        expect(verdict.windows,).toBe(0,);
      },
    },),

    it({
      name: 'HOLDS MEMORY FLAT across a stream that never ends, which is the whole population this '
        + 'guard exists for: a detector that grew with the stream would fail on exactly the calls '
        + 'it is meant to stop',
      fn: async () => {
        /**
         Detector fed far more text than the trailing sample can hold.
         */
        const detector = watchForDegeneration();

        /**
         One healthy block, repeated with a changing tail so no two are equal.
         */
        const block = variedProse({ lines: 2_000, },);
        Array.from(
          { length: 40, },
          function round(
            _unused,
            at,
          ): number {
            detector.notifyText({ text: `${block}${String(at,)}`, },);
            return at;
          },
        );

        /**
         Sample size after all of it, which must be the cap and not the total.
         */
        const verdict = detector.verdict();
        expect(verdict.kind,).toBe('healthy',);
        if (verdict.kind !== 'healthy')
          throw new Error('healthy by construction',);
        expect(verdict.windows,).toBe(4_096,);
      },
    },),

    it({
      name: 'REPORTS CHARACTERS SEEN UNCONDITIONALLY, before any verdict is reachable and after a '
        + 'healthy one, because a progress line needs a figure for every stream and `verdict` only '
        + 'carries this count on its degenerate case, which is silent on every stream that never '
        + 'trips it',
      fn: async () => {
        /**
         Detector fed too little text for `verdict` to say anything but
         `undecided`.
         */
        const short = watchForDegeneration();
        expect(short.charsSeen(),).toBe(0,);
        short.notifyText({ text: 'The cat naps. ', },);
        expect(short.charsSeen(),).toBe(14,);
        expect(short.verdict().kind,).toBe('undecided',);

        /**
         Detector fed enough varied text to read healthy.
         */
        const long = watchForDegeneration();
        const prose = variedProse({ lines: 2_000, },);
        long.notifyText({ text: prose, },);
        expect(long.charsSeen(),).toBe(prose.length,);
        expect(long.verdict().kind,).toBe('healthy',);
      },
    },),
  ],
},);
