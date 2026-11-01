/**
 Tests for the footnote floor on an archive block's revision (class
 eighty-four), read directly rather than through the review stage.

 LEDGER T8, EIGHTEENTH BATCH: a block the archive does not carry verbatim
 passed every revision without reading the page it would leave, a branch
 documented as kept for fixtures a caller composed. Production hands the
 floor an exact slice of the archive (`corpus-run/archive-block-repair.ts`),
 so such a block breaks the caller's contract, and the floor now refuses it
 rather than letting a revision past unread.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BlockOutsideArchiveError,
  revisionFootnoteFindings,
} from '../dist/final/node/index.mjs';

/**
 Archive page whose second block defines the note its first references.
 */
const ARCHIVE = 'The cat napped by the stove.[^1]\n\n[^1]: The stove in the kitchen.\n';

/**
 Archive page whose block `Purr[^1]` stands twice: first inside a code fence,
 where it references nothing, then in the body, where it references the note.
 */
const FENCED_TWICE = '```\nPurr[^1]\n```\n\nPurr[^1]\n\n[^1]: A cat note.';

/**
 Block that stands twice on that page.
 */
const PURR = 'Purr[^1]';

/**
 Finding for a revision of that block that drops the marker the note hangs on.
 */
const ORPHANED = 'archive-revision-refused (hf:cat/Cat-A): the revision gives the page a footnote defect it does not '
  + 'carry as it stands (orphan-definition gfm 1); a footnote is a relation between blocks, and its other end '
  + 'stands outside the block under review';

await describe({
  name: revisionFootnoteFindings.name,
  children: [
    it({
      name: 'REFUSES a block the page does not carry, which once passed every revision without reading the '
        + 'page it would leave',
      fn: async () => {
        /**
         What the floor threw for a block no page carries.
         */
        const refusal = caught(function readsComposedBlock(): unknown {
          return revisionFootnoteFindings({
            modelId: 'hf:cat/Cat-A',
            blockText: '[^1]: The stove in the hall.',
            blockOffset: 0,
            replacementText: '',
            targetText: ARCHIVE,
          },);
        },);
        expect(refusal,).toBeInstanceOf(BlockOutsideArchiveError,);
        expect((refusal as Error).message,).toBe(
          'the archive block under review is not in the page it is reviewed in, so the page its revision would '
            + 'leave cannot be built and its footnotes cannot be checked',
        );
      },
    },),
    it({
      name: 'READS THE REVISION AT THE OFFSET THE CALLER GIVES: a block that stands twice, the first inside a code '
        + 'fence, loses the marker the note hangs on at its second place',
      fn: async () => {
        expect(revisionFootnoteFindings({
          modelId: 'hf:cat/Cat-A',
          blockText: PURR,
          blockOffset: FENCED_TWICE.lastIndexOf(PURR,),
          replacementText: 'Purr',
          targetText: FENCED_TWICE,
        },),).toEqual([ORPHANED,],);
      },
    },),
    it({
      name: 'READS THE REVISION AT THE OFFSET THE CALLER GIVES: the same block at its first place, inside a code '
        + 'fence, leaves the note referenced',
      fn: async () => {
        expect(revisionFootnoteFindings({
          modelId: 'hf:cat/Cat-A',
          blockText: PURR,
          blockOffset: FENCED_TWICE.indexOf(PURR,),
          replacementText: 'Purr',
          targetText: FENCED_TWICE,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES an offset at which the page does not carry the block, though it carries the block elsewhere',
      fn: async () => {
        /**
         What the floor threw for an offset one place past the block's own.
         */
        const refusal = caught(function readsMisplacedBlock(): unknown {
          return revisionFootnoteFindings({
            modelId: 'hf:cat/Cat-A',
            blockText: PURR,
            blockOffset: FENCED_TWICE.indexOf(PURR,) + 1,
            replacementText: 'Purr',
            targetText: FENCED_TWICE,
          },);
        },);
        expect(refusal,).toBeInstanceOf(BlockOutsideArchiveError,);
      },
    },),
    it({
      name: 'READS the page a revision of a block the archive carries would leave, finding nothing where the '
        + 'notes still pair',
      fn: async () => {
        expect(revisionFootnoteFindings({
          modelId: 'hf:cat/Cat-A',
          blockText: '[^1]: The stove in the kitchen.',
          blockOffset: ARCHIVE.indexOf('[^1]: The stove in the kitchen.',),
          replacementText: '[^1]: The kitchen stove.',
          targetText: ARCHIVE,
        },),).toEqual([],);
      },
    },),
  ],
},);
