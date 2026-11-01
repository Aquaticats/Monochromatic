/**
 Tests for holding target-only English out of translation.

 WHAT THESE PIN is that a passage the source cannot account for survives. The
 translate lane writes each slice fresh from its source, so a transcript a
 human added to the English has nothing to produce it: measured on the pool
 settled 2026-08-18, one slice went from 1766 archive characters to 215
 shipped and another from 1228 to 175. Both are memorial pages and the lost
 blocks are the accessible reading of an image.

 The second thing they pin is the anchor comparison. A byte-identical
 comparison was written first and it MISSED the very case that prompted this,
 because the source writes two spaces inside a component where the archive
 writes one. That case has its own test, since it is the difference between
 this working and this looking like it works.

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
  restoreTargetOnlyRun,
  splitTargetOnlyRun,
} from '../dist/final/node/index.mjs';

/**
 Component ending a source slice, which the archive repeats because markup is
 not prose.
 */
const COMPONENT = '<PhotoScroll photos={[ \'/photos/tabby.webp\' ]} />';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: splitTargetOnlyRun.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS BACK A TRANSCRIPT THE SOURCE NEVER HAD, which is the whole point: nothing '
            + 'downstream can produce a passage that has no original, so a lane writing from the '
            + 'source alone deletes it',
          fn: async () => {
            /**
             Source ending on the component, with no transcript after it.
             */
            const sourceText = `窗台上的猫。\n\n${COMPONENT}`;

            /**
             Archive carrying the same blocks plus a lead-in and a transcript.
             */
            const incumbentText = `The cat on the sill.\n\n${COMPONENT}\n\n`
              + 'English transcript of the photo above:\n\n'
              + '> Name: Mittens. Likes: sunbeams, boxes.\n> Dislikes: closed doors.';

            const split = splitTargetOnlyRun({
              sourceText,
              incumbentText,
            },);

            expect(split,).toStrictEqual({
              judgedText: `The cat on the sill.\n\n${COMPONENT}`,
              protectedText: '\n\nEnglish transcript of the photo above:\n\n'
                + '> Name: Mittens. Likes: sunbeams, boxes.\n> Dislikes: closed doors.',
            },);
          },
        },),

        it({
          name: 'MATCHES AN ANCHOR THAT DIFFERS ONLY IN WHITESPACE, which a byte-identical comparison '
            + 'did not: the case this exists for writes two spaces inside the component on one side '
            + 'and one on the other, so an exact comparison protected nothing and looked correct',
          fn: async () => {
            /**
             Source spelling the component with two spaces before its closer.
             */
            const spaced = '<PhotoScroll photos={[ \'/photos/tabby.webp\'  ]} />';

            const split = splitTargetOnlyRun({
              sourceText: `窗台上的猫。\n\n${spaced}`,
              incumbentText: `The cat on the sill.\n\n${COMPONENT}\n\n`
                + 'Transcript:\n\n> Name: Mittens.',
            },);

            expect(split.protectedText,).toBe('\n\nTranscript:\n\n> Name: Mittens.',);
          },
        },),

        it({
          name: 'RESTORES THE ARCHIVE EXACTLY when the two halves are put back together, since a '
            + 'retention has to leave the document byte-identical',
          fn: async () => {
            /**
             Archive as it stands.
             */
            const incumbentText = `The cat on the sill.\n\n${COMPONENT}\n\n`
              + 'Transcript:\n\n> Name: Mittens.';

            const split = splitTargetOnlyRun({
              sourceText: `窗台上的猫。\n\n${COMPONENT}`,
              incumbentText,
            },);

            expect(restoreTargetOnlyRun({
              text: split.judgedText,
              protectedText: split.protectedText,
            },),).toBe(incumbentText,);
          },
        },),

        it({
          name: 'LEAVES AN ORDINARY SLICE WHOLE, because protecting a passage nobody added would put '
            + 'the archive beyond every lane and freeze wording that has a source and can be improved',
          fn: async () => {
            /**
             A pair with no markup and no surplus.
             */
            const incumbentText = 'The cat naps.\n\nIt wakes at dusk.';

            const split = splitTargetOnlyRun({
              sourceText: '猫在睡觉。\n\n黄昏时醒来。',
              incumbentText,
            },);

            expect(split.judgedText,).toBe(incumbentText,);
            expect(split.protectedText,).toBe('',);
          },
        },),

        it({
          name: 'LEAVES A TRAILING RUN CARRYING NO BLOCKQUOTE ALONE, since ordinary trailing prose is '
            + 'wording a translator may legitimately reword and only a transcript is protected',
          fn: async () => {
            const split = splitTargetOnlyRun({
              sourceText: `窗台上的猫。\n\n${COMPONENT}`,
              incumbentText: `The cat on the sill.\n\n${COMPONENT}\n\nShe was much loved.`,
            },);

            expect(split.protectedText,).toBe('',);
          },
        },),

        it({
          name: 'REQUIRES THE ANCHOR TO BE THE SOURCE’S LAST BLOCK, because an identical block in the '
            + 'middle says nothing about what follows it: the source has more to say there too, and '
            + 'protecting that would freeze wording the source can account for',
          fn: async () => {
            const split = splitTargetOnlyRun({
              sourceText: `${COMPONENT}\n\n她很想念。`,
              incumbentText: `${COMPONENT}\n\nShe is missed.\n\n> A quote nobody anchored.`,
            },);

            expect(split.protectedText,).toBe('',);
          },
        },),

        it({
          name: 'REPORTS AN EMPTY PASSAGE as nothing to protect rather than failing, since a slice the '
            + 'archive never translated is an ordinary case this lane fills',
          fn: async () => {
            const split = splitTargetOnlyRun({
              sourceText: '窗台上的猫。',
              incumbentText: '',
            },);

            expect(split.judgedText,).toBe('',);
            expect(split.protectedText,).toBe('',);
          },
        },),

        it({
          name: 'KEEPS A PASSAGE WHOLE WHEN THE ORIGINAL HOLDS NO BLOCK, since with nothing for the archive to '
            + 'repeat there is no anchor to protect what follows, a quoted run included',
          fn: async () => {
            expect(splitTargetOnlyRun({
              sourceText: '',
              incumbentText: 'The cat naps.\n\n> Dear cat, rest well.',
            },),).toEqual({
              judgedText: 'The cat naps.\n\n> Dear cat, rest well.',
              protectedText: '',
            },);
          },
        },),

        it({
          name: 'PROTECTS A TRANSCRIPT WRITTEN DIRECTLY AFTER ITS ANCHOR WITH NO BLANK LINE, a photo '
            + 'gallery component the archive repeats followed on the next line by a quoted caption of '
            + 'what the photos show, and keeps the archive\'s own bytes so restoring rebuilds it exactly',
          fn: async () => {
            /**
             Archive wording: the anchor, one line break, the caption.
             */
            const incumbentText = 'Introduction of the cat.\n\n<PhotoScroll images={[1,2]} />\n'
              + '> A photo shows the cat napping.\n> Another shows it stretching.\n';
            const split = splitTargetOnlyRun({
              sourceText: '猫的介绍。\n\n<PhotoScroll images={[1,2]} />\n',
              incumbentText,
            },);

            expect(split,).toStrictEqual({
              judgedText: 'Introduction of the cat.\n\n<PhotoScroll images={[1,2]} />',
              protectedText: '\n> A photo shows the cat napping.\n> Another shows it stretching.\n',
            },);
            expect(restoreTargetOnlyRun({
              text: split.judgedText,
              protectedText: split.protectedText,
            },),).toBe(incumbentText,);
          },
        },),

        it({
          name: 'ANCHORS ON A CONTAINER\'S CLOSING TAG THE SOURCE ENDS ON, which the slice grammar holds aside, '
            + 'so the tag stays with the wording the lane renders and is not carried twice (ledger B100)',
          fn: async () => {
            expect(splitTargetOnlyRun({
              sourceText: '猫在睡觉。\n\n</details>',
              incumbentText: 'The cat naps.\n\n</details>\n\n> A note nobody wrote in Chinese.',
            },),).toStrictEqual({
              judgedText: 'The cat naps.\n\n</details>',
              protectedText: '\n\n> A note nobody wrote in Chinese.',
            },);
          },
        },),

        it({
          name: 'LEAVES A PASSAGE THE SLICE GRAMMAR REFUSES WHOLE, since a run it cannot read into '
            + 'blocks cannot be shown to carry a transcript the source lacks (ledger B68)',
          fn: async () => {
            /**
             Archive repeating the anchor, then an unclosed component and a
             quote.
             */
            const incumbentText = `The cat on the sill.\n\n${COMPONENT}\n\n<Cat unclosed\n\n> Name: Mittens.`;

            expect(splitTargetOnlyRun({
              sourceText: `窗台上的猫。\n\n${COMPONENT}`,
              incumbentText,
            },),).toStrictEqual({
              judgedText: incumbentText,
              protectedText: '',
            },);
          },
        },),
      ],
    },),

    describe({
      name: restoreTargetOnlyRun.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS TEXT UNTOUCHED when nothing was protected, so the ordinary slice takes no '
            + 'separator it did not already have',
          fn: async () => {
            expect(restoreTargetOnlyRun({
              text: 'The cat naps.',
              protectedText: '',
            },),).toBe('The cat naps.',);
          },
        },),

        it({
          name: 'PUTS THE RUN AFTER A REPLACEMENT with the line breaks the archive set before it, so a '
            + 'slice whose wording changed still carries the passage nobody could translate',
          fn: async () => {
            expect(restoreTargetOnlyRun({
              text: 'The tabby naps.',
              protectedText: '\n\n> Name: Mittens.',
            },),).toBe('The tabby naps.\n\n> Name: Mittens.',);
          },
        },),
      ],
    },),
  ],
},);
