/**
 Tests for the capped failure text a pass prints.

 WHAT THESE PIN: a class that may be quoted is quoted up to the cap and no
 further, and a class that may not be quoted is named, which is the refusal
 rule every stdout printer follows.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { UnansweredContestSliceError, } from '@monochromatic-dev/translation-repair-page-check';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import { CollapsedHeadingError, } from '@monochromatic-dev/translation-repair-page-pass';
import {
  TALLY_ERROR_CAP,
  tallyErrorText,
} from '../../dist/final/node/index.mjs';

/**
 Commit a long entry's page was asked for at.
 */
const PINNED = 'c'.repeat(40,);

/**
 Entry id long enough to push a corpus read's kind past the cap, as a long
 picture name or a long id does.
 */
const LONG_ID = 'tabby'.repeat(40,);

/**
 A corpus read of the long entry's English page that found no such path at a
 commit the clone holds.
 */
const LONG_ID_REFUSAL = new CorpusReadError({
  detail: `${PINNED}:people/${LONG_ID}/page.en.md`,
  cause: { stderr: `fatal: path 'people/${LONG_ID}/page.en.md' does not exist in '${PINNED}'`, },
  commit: 'held',
},);

await describe({
  name: tallyErrorText.name,
  children: [
    it({
      name: 'quotes a stated refusal and caps it at the line budget',
      fn: async () => {
        /**
         Refusal longer than any line should carry.
         */
        const error = new StatedRefusalError({ says: 'x'.repeat(TALLY_ERROR_CAP * 2,), },);

        expect(tallyErrorText({ error, },),).toHaveLength(TALLY_ERROR_CAP,);
      },
    },),

    it({
      name: 'KEEPS A CORPUS READ\'S KIND AND REMEDY WHOLE after a long entry id, cutting only the path and '
        + 'revision before them at the cap and marking the cut with an ellipsis inside it',
      fn: async () => {
        expect(tallyErrorText({ error: LONG_ID_REFUSAL, },),).toBe(
          `corpus read failed for ${PINNED}:people/${'tabby'.repeat(25,)}tab… (missing-object); the commit has `
            + 'no such path: check the path, or pin a commit that has it.',
        );
      },
    },),

    it({
      name: 'REFUSES AS UNREACHABLE a corpus read whose message no longer ends on the kind and remedy its class '
        + 'built it with, rather than cutting a closing it cannot find',
      fn: async () => {
        /**
         A corpus read whose message was written over after it was built.
         */
        const rewritten = new CorpusReadError({
          detail: 'people/tabby/page.md at deadbeef',
          cause: { stderr: "fatal: path 'people/tabby/page.md' does not exist in 'deadbeef'", },
          commit: 'held',
        },);
        rewritten.message = 'corpus read failed for people/tabby/page.md at deadbeef';

        /**
         What rendering it threw.
         */
        const refusal = caught(function render(): unknown {
          return tallyErrorText({ error: rewritten, },);
        },);
        expect(refusal,).toBeInstanceOf(Error,);
        expect(String(refusal,),).toBe(
          'Error: unreachable: a corpus read refusal\'s message does not end on its kind and remedy, which its '
            + 'constructor writes as the message\'s closing',
        );
      },
    },),

    it({
      name: 'SAYS WHICH SLICE A CONTEST LEFT UNANSWERED, as it says the sentence of a sibling refusal that stops an '
        + 'entry, rather than naming the class alone',
      fn: async () => {
        expect({
          collapsed: tallyErrorText({
            error: new CollapsedHeadingError({
              entryId: 'Mittens',
              sourceDistinct: 2,
              pageDistinct: 1,
            },),
          },),
          unanswered: tallyErrorText({ error: new UnansweredContestSliceError({ sliceIndex: 3, },), },),
        },).toEqual({
          collapsed: 'entry Mittens would render 2 distinct source heading(s) as 1 distinct page heading(s)',
          unanswered: 'slice 3 differs across lanes and the contest names it nowhere',
        },);
      },
    },),

    it({
      name: 'names a plain error instead of quoting its message',
      fn: async () => {
        /**
         Error whose message must not reach stdout.
         */
        const error = new Error('the archive said something here',);

        /**
         What the line would carry.
         */
        const text = tallyErrorText({ error, },);

        expect(text,).not.toContain('archive',);
        expect(text,).toContain('Error',);
      },
    },),
  ],
},);
