/**
 Tests for the join rule that removes a sentence without leaving an edit-mark.

 REACHED ONLY THROUGH `fidelity-damage.ts`, whose own cases ask whether a
 sentence disappeared. Every join rule answers that question identically,
 including no join rule at all, so the whitespace decision this module exists
 for is invisible to them.

 WHAT A DIAGNOSTIC MUTATION FOUND. Removing the line-break precedence from the
 private `survivingRun`, leaving only the two boundary rules and the length
 tiebreak, left the whole suite green. That branch decides whether a paragraph
 cut from the middle of a page leaves a paragraph break behind or collapses two
 paragraphs into one line, which is the visible edit-mark the module was
 written to prevent. Both of its arms are pinned here.

 WHY THE TWO EXOTIC SPACES GET CASES. The corpus is Chinese, and Chinese prose
 separates sentences with `\u3000`. A separator set that missed it would cut
 the sentence and leave BOTH ideographic spaces behind, a doubled gap visible
 only on the half of the corpus this pipeline exists for.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spliceOutSentence, } from '../dist/final/node/index.mjs';

/**
 Ideographic space, written as an escape so no invisible character hides in a
 fixture the way the module keeps none in its own separator set.
 */
const IDEOGRAPHIC_SPACE = '\u3000';

/**
 No-break space, written as an escape for the same reason.
 */
const NO_BREAK_SPACE = '\u00A0';

/**
 Sentence every fixture cuts, so each case differs only in what surrounds it.
 */
const CUT = 'The cat slept.';

await describe({
  name: spliceOutSentence.name,
  children: [
    it({
      name: 'KEEPS the passage whole when the sentence does not occur in it, '
        + 'which is the control the rest of these cases depart from one rule at '
        + 'a time',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: 'The cat sat. The cat woke.',
            needle: CUT,
          },),
        )
          .toBe('The cat sat. The cat woke.',);
      },
    },),

    it({
      name: 'DROPS one of the two spaces around a sentence cut from the middle '
        + 'of a paragraph, leaving the single space prose writes rather than the '
        + 'double space a plain deletion leaves',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat. ${CUT} The cat woke.`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat. The cat woke.',);
      },
    },),

    it({
      name: 'KEEPS the preceding paragraph break over a longer run of spaces '
        + 'that follows, so the two paragraphs either side of a cut sentence stay '
        + 'two paragraphs instead of collapsing onto one line',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat.\n\n${CUT}   The cat woke.`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat.\n\nThe cat woke.',);
      },
    },),

    it({
      name: 'KEEPS the following paragraph break over a longer run of spaces '
        + 'that precedes, which is the same rule read from the other side and the '
        + 'other arm of the same comparison',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat.   ${CUT}\n\nThe cat woke.`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat.\n\nThe cat woke.',);
      },
    },),

    it({
      name: 'KEEPS a carriage-return paragraph break over a single one, so a '
        + 'page written with Windows line endings is weighed by its breaks like '
        + 'any other',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat.\r\n\r\n${CUT}\r\nThe cat woke.`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat.\r\n\r\nThe cat woke.',);
      },
    },),

    it({
      name: 'KEEPS exactly one paragraph break when a whole paragraph is cut '
        + 'from between two others, rather than the two breaks and blank line a '
        + 'plain deletion leaves',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat.\n\n${CUT}\n\nThe cat woke.`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat.\n\nThe cat woke.',);
      },
    },),

    it({
      name: 'KEEPS the trailing run when the cut reaches the end of the '
        + 'passage, so the page still ends with its newline instead of the blank '
        + 'line the stronger preceding break would leave',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat.\n\n${CUT}\n`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat.\n',);
      },
    },),

    it({
      name: 'KEEPS the leading run when the cut reaches the start of the '
        + 'passage, so the page does not begin with the blank line the stronger '
        + 'following break would leave',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `\nThe cat sat.\n\n${CUT}`,
            needle: 'The cat sat.',
          },),
        )
          .toBe(`\n${CUT}`,);
      },
    },),

    it({
      name: 'KEEPS the trailing run when the cut reaches both ends at once, '
        + 'which pins the order the two boundary rules are asked in and is the '
        + 'only case where they disagree',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: ` ${CUT}\n`,
            needle: CUT,
          },),
        )
          .toBe('\n',);
      },
    },),

    it({
      name: 'KEEPS the wider of two runs that carry the same number of line '
        + 'breaks, so a deliberately wide gap survives a cut that a narrower '
        + 'neighbour would have narrowed',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat. ${CUT}  The cat woke.`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat.  The cat woke.',);
      },
    },),

    it({
      name: 'KEEPS the preceding run when both runs weigh the same, so a tab '
        + 'before the cut survives a space after it and the choice never depends '
        + 'on which side was read first',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat.\t${CUT} The cat woke.`,
            needle: CUT,
          },),
        )
          .toBe('The cat sat.\tThe cat woke.',);
      },
    },),

    it({
      name: 'KEEPS one ideographic space where a Chinese sentence sat between '
        + 'two of them, which is the separator the corpus this pipeline reads '
        + 'actually uses',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `猫睡了。${IDEOGRAPHIC_SPACE}猫醒了。${IDEOGRAPHIC_SPACE}猫走了。`,
            needle: '猫醒了。',
          },),
        )
          .toBe(`猫睡了。${IDEOGRAPHIC_SPACE}猫走了。`,);
      },
    },),

    it({
      name: 'KEEPS one no-break space where a sentence sat between two of '
        + 'them, against the same rule that just kept one ideographic space',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat.${NO_BREAK_SPACE}${CUT}${NO_BREAK_SPACE}The cat woke.`,
            needle: CUT,
          },),
        )
          .toBe(`The cat sat.${NO_BREAK_SPACE}The cat woke.`,);
      },
    },),

    it({
      name: 'DROPS only the first occurrence when the same sentence is written '
        + 'twice, since the damage instrument asks for one sentence to go and a '
        + 'second removal would be damage nobody seeded',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: `The cat sat. ${CUT} The cat sat.`,
            needle: 'The cat sat.',
          },),
        )
          .toBe(`${CUT} The cat sat.`,);
      },
    },),

    it({
      name: 'JOINS the two halves directly when nothing separates the sentence '
        + 'from its neighbours, taking no character that was never whitespace',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: '[cat][nap][sun]',
            needle: '[nap]',
          },),
        )
          .toBe('[cat][sun]',);
      },
    },),

    it({
      name: 'RETURNS nothing at all when the sentence is the whole passage, '
        + 'rather than the separator a run of length zero would otherwise leave',
      fn: async () => {
        expect(
          spliceOutSentence({
            text: CUT,
            needle: CUT,
          },),
        )
          .toBe('',);
      },
    },),
  ],
},);
