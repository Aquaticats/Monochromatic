/**
 Tests for the `mdx-downgraded` integrity signal.

 `downgradeCount` had no test. It is four lines, and it decides an integrity
 verdict: candidate selection ranks integrity above every other measurement,
 so a patch that raises this count loses to unchanged no matter how many
 issues it fixed. A count that silently included the wrong finding kind would
 therefore discard correct repairs, and a count that missed real downgrades
 would ship text whose document grammar the patch broke.

 The kind filter is the whole function, so the cases pin it against the other
 finding kind the parser actually emits rather than against an invented one.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { downgradeCount, } from '../dist/final/node/index.mjs';

await describe({
  name: downgradeCount.name,
  children: [
    it({
      name: 'counts nothing for a document that parsed cleanly, so an ordinary '
        + 'repair is never penalized on integrity',
      fn: async () => {
        expect(
          downgradeCount({
            document: parseDocument({
              text: 'The cat naps on the windowsill.\n\nShe wakes when the sun moves.\n',
            },),
          },),
        ).toBe(0,);
      },
    },),

    it({
      name: 'counts a document whose grammar forced the markdown fallback, '
        + 'which is the signal itself: the patch broke something the strict '
        + 'parser could read before',
      fn: async () => {
        expect(
          downgradeCount({
            document: parseDocument({ text: 'The cat naps {unclosed\n\nShe wakes.\n', },),
          },),
        ).toBe(1,);
      },
    },),

    it({
      name: 'IGNORES the other finding kind the parser emits. A skipped HTML '
        + 'comment is a note about what was not parsed, not a grammar failure, '
        + 'and counting it would make integrity fail on documents that merely '
        + 'contain comments, discarding their repairs',
      fn: async () => {
        expect(
          downgradeCount({
            document: parseDocument({
              text: 'The cat naps.\n\n<!-- a note about the cat -->\n\nShe wakes.\n',
            },),
          },),
        ).toBe(0,);
      },
    },),

    it({
      name: 'counts only the downgrade when both kinds are present, so a '
        + 'comment riding alongside a real grammar failure neither hides it '
        + 'nor inflates it',
      fn: async () => {
        /**
         Document carrying a skipped comment and a grammar failure at once.
         */
        const document = parseDocument({
          text: 'The cat naps {unclosed\n\n<!-- a note about the cat -->\n\nShe wakes.\n',
        },);

        expect(document.parseFindings.length,).toBeGreaterThan(1,);
        expect(downgradeCount({ document, },),).toBe(1,);
      },
    },),

    it({
      name: 'counts zero for an empty document rather than throwing, since an '
        + 'emptied slice is a legitimate repair outcome the integrity check '
        + 'still has to rank',
      fn: async () => {
        expect(
          downgradeCount({ document: parseDocument({ text: '', },), },),
        ).toBe(0,);
      },
    },),
  ],
},);
