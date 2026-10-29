/**
 Tests for the size evidence a contest judge is shown, and for the policy that
 teaches how to read it.

 WHY THE FAR-LONGER DIRECTION IS TESTED FOR EVIDENCE RATHER THAN FOR A FAULT.
 `CONTEST_POLICY` tells judges that keeping page-only content is correct where
 the Chinese is silent, so a candidate preserving a long page-only region is
 far longer than its original AND is the right candidate. A test asserting a
 fault there would be pinning the wrong behaviour in place.

 WHY THE SOURCE FLOOR IS TESTED IN ONE DIRECTION ONLY. A short original
 cannot support a SHORTFALL reading, because the ratio reports rounding. It
 supports a SURPLUS reading perfectly well: a 56-character original against a
 10381-character rendering is a lane looping. The test that used to live here
 asserted silence for an 800-character rendering of a 79-character original,
 which is 10.1 times and exactly the shape the note exists to surface, so it
 was pinning the defect rather than the behaviour.

 WHY A BLOCK GAP IS TESTED FOR SILENCE. That reason describes the PAIRING
 rather than the rendering, and it was the sole cause for 20 of 36 flagged
 slices on the corpus. Showing a judge a ratio the pairing does not support
 would be showing it noise, so the exclusion is load-bearing rather than
 incidental.

 Fixtures are invented cat text sized to exact character counts, not corpus
 passages.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  contestSizeNote,
  SIZE_NOTE_POLICY,
} from '../dist/final/node/index.mjs';

/**
 Filler long enough to cut every fixture from.
 */
const FILLER = 'the tabby naps in the sun while the calico watches a moth cross the window '
  .repeat(40,);

/**
 Builds invented text of an exact character count, so a ratio is set precisely.

 @param chars - length wanted

 @returns Text of exactly that many characters, in one block

 @example
 ```ts
 const original = catText({ chars: 100, },);
 ```
 */
function catText({ chars, }: { readonly chars: number; },): string {
  return FILLER.slice(0, chars,);
}

/**
 Original every fixture is measured against, comfortably over the floor.
 */
const ORIGINAL = catText({ chars: 100, },);

/**
 Rendering in proportion to {@link ORIGINAL}, at three times its size.
 */
const IN_PROPORTION = catText({ chars: 300, },);

/**
 Builds the note for one rendering beside an in-proportion companion.

 @param text - rendering under test

 @param sourceText - original it is measured against

 @returns Note, or an empty string

 @example
 ```ts
 const note = noteFor({ text: catText({ chars: 70, },), },);
 ```
 */
function noteFor(
  {
    text,
    sourceText = ORIGINAL,
  }: {
    readonly text: string;
    readonly sourceText?: string;
  },
): string {
  return contestSizeNote({
    sourceText,
    renderings: [
      {
        label: 'ARCHIVE RENDERING',
        text: IN_PROPORTION,
      },
      {
        label: 'CANDIDATE "tabby"',
        text,
      },
    ],
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: contestSizeNote.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS NOTHING when every rendering is in proportion, so a note that does appear is about '
            + 'this passage rather than boilerplate a judge learns to skim',
          fn: async function silentOnOrdinarySizes() {
            expect(noteFor({ text: IN_PROPORTION, },),).toBe('',);
          },
        },),

        it({
          name: 'REPORTS a rendering far shorter than its original, which is the direction that means '
            + 'Chinese content went unrendered whatever the archive did',
          fn: async function reportsFarShorter() {
            /**
             Seventy characters against one hundred, which is 0.7 and under 0.8.
             */
            const note = noteFor({ text: catText({ chars: 70, },), },);

            expect(note,).toContain('SIZE NOTE',);
            expect(note,).toContain('0.7 times the original',);
          },
        },),

        it({
          name: 'REPORTS a rendering far longer than its original as evidence, WITHOUT naming a fault, '
            + 'because keeping page-only content is correct and produces exactly this shape',
          fn: async function reportsFarLongerWithoutBlame() {
            /**
             Eleven hundred characters against one hundred, which is over ten.
             */
            const note = noteFor({ text: catText({ chars: 1_100, },), },);

            expect(note,).toContain('SIZE NOTE',);
            expect(note,).toContain('11.0 times the original',);
            expect(note,).toContain('Evidence, not a verdict',);
          },
        },),

        it({
          name: 'SAYS NOTHING about a SHORTFALL below the source floor, where a ratio over a short '
            + 'line reports rounding rather than a rendering having left content unrendered',
          fn: async function silentOnAShortfallUnderTheFloor() {
            // Fifty-five characters against seventy-nine, which is 0.70 and under 0.8.
            expect(noteFor({
              text: catText({ chars: 55, },),
              sourceText: catText({ chars: 79, },),
            },),).toBe('',);
          },
        },),

        it({
          name: 'SPEAKS about a SURPLUS below the source floor, because a rendering fifty times its '
            + 'original is a lane looping rather than rounding, and this is the case the note was '
            + 'measured to be silent on across 37 of 40 far-longer candidates',
          fn: async function speaksOnASurplusUnderTheFloor() {
            /**
             Two thousand eight hundred characters against fifty-six, the shape a
             translate lane produced on a real settled slice at 185 times.
             */
            const note = noteFor({
              text: catText({ chars: 2_800, },),
              sourceText: catText({ chars: 56, },),
            },);

            expect(note,).toContain('SIZE NOTE',);
            expect(note,).toContain('50.0 times the original',);
            expect(note,).toContain('Chinese original: 56',);
          },
        },),

        it({
          name: 'SPEAKS about a SHORTFALL at the floor exactly, so the one-directional floor is pinned '
            + 'on the side that still has one rather than only on the side that lost it',
          fn: async function speaksOnAShortfallAtTheFloor() {
            // Sixty characters against eighty, which is 0.75 and under 0.8.
            expect(noteFor({
              text: catText({ chars: 60, },),
              sourceText: catText({ chars: 80, },),
            },),).toContain('SIZE NOTE',);
          },
        },),

        it({
          name: 'SPEAKS at the floor exactly, so the boundary is pinned from both sides rather than '
            + 'only from the middle of the band',
          fn: async function speaksAtTheFloor() {
            expect(noteFor({
              text: catText({ chars: 810, },),
              sourceText: catText({ chars: 80, },),
            },),).toContain('SIZE NOTE',);
          },
        },),

        it({
          name: 'SAYS NOTHING for a block-count gap on its own, because that reason describes the '
            + 'pairing rather than the rendering and makes a ratio meaningless rather than extreme',
          fn: async function silentOnABlockGapAlone() {
            /**
             Three hundred characters across five blocks: in proportion at three
             times the original, and four blocks away from its one.
             */
            const scattered = [
              catText({ chars: 59, },),
              catText({ chars: 59, },),
              catText({ chars: 59, },),
              catText({ chars: 59, },),
              catText({ chars: 56, },),
            ].join('\n\n',);

            expect(scattered.length,).toBe(300,);
            expect(noteFor({ text: scattered, },),).toBe('',);
          },
        },),

        it({
          name: 'LISTS EVERY rendering once anything trips, not only the one that tripped, because a '
            + 'judge deciding between them needs the comparison rather than one number',
          fn: async function listsEveryRendering() {
            const note = noteFor({ text: catText({ chars: 70, },), },);

            expect(note,).toContain('ARCHIVE RENDERING: 300',);
            expect(note,).toContain('CANDIDATE "tabby": 70',);
            expect(note,).toContain('Chinese original: 100',);
          },
        },),

        it({
          name: 'SAYS NOTHING for an empty list of renderings, rather than emitting a heading with no '
            + 'rows under it',
          fn: async function silentOnNoRenderings() {
            expect(contestSizeNote({
              sourceText: ORIGINAL,
              renderings: [],
            },),).toBe('',);
          },
        },),

        it({
          name: 'SAYS NOTHING for a rendering of zero length, which is a different failure that the '
            + 'stage guards name, rather than a passage that is the wrong size',
          fn: async function silentOnAnEmptyRendering() {
            expect(noteFor({ text: '', },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: 'SIZE_NOTE_POLICY',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES to name any candidate, which is what lets the lane contest and the '
            + 'consolidate gate share one policy despite naming their candidates differently',
          fn: async function namesNoCandidate() {
            for (const name of [ 'repair', 'translate', 'consolidated', 'standing', ])
              expect(SIZE_NOTE_POLICY.includes(`"${name}"`,),).toBe(false,);
          },
        },),

        it({
          name: 'carries a DIFFERENT reading for each direction, which is the property two named '
            + 'faults were wanted for and the one thing a single name would have blurred',
          fn: async function carriesBothReadings() {
            expect(SIZE_NOTE_POLICY,).toContain('FAR SHORTER',);
            expect(SIZE_NOTE_POLICY,).toContain('DROPPED question',);
            expect(SIZE_NOTE_POLICY,).toContain('FAR LONGER',);
            expect(SIZE_NOTE_POLICY,).toContain('DROPPED-ALSO rule',);
          },
        },),

        it({
          name: 'REFUSES to let size decide either reading on its own, and names verse as a reason a '
            + 'large ratio can be innocent, which is why no verse marker is threaded to this stage',
          fn: async function sizeSettlesNothingAlone() {
            expect(SIZE_NOTE_POLICY,).toContain('SIZE ALONE SETTLES NEITHER READING',);
            expect(SIZE_NOTE_POLICY,).toContain('line-structured original',);
          },
        },),

        it({
          name: 'CALLS silent surplus page content only where the archive carries it, so a looping candidate\'s '
            + 'surplus is unsupported (ledger S7: the note exists for a loop and once called its surplus page content)',
          fn: async function loopSurplusIsUnsupported() {
            expect(SIZE_NOTE_POLICY,).toContain('Surplus the ARCHIVE RENDERING also carries',);
            expect(SIZE_NOTE_POLICY,).toContain('a passage repeated or looping',);
            expect(SIZE_NOTE_POLICY.includes('Surplus the Chinese is SILENT about is page content',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
