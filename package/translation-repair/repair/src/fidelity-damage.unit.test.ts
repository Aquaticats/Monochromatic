/**
 Tests for the two constructed defects the judge-quality bench puts on the
 ballot.

 What these pin is the property each fixture EXISTS for: the deletion leaves
 the complete text longer, the insertion leaves it shorter, and both leave
 every other word alone. A fixture that quietly failed either would produce a
 number that reads exactly like a good one.

 Fixtures mirror corpus structure only. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  alterSharedNumber,
  deleteOneSentence,
  insertBorrowedSentence,
} from '../dist/final/node/index.mjs';

/**
 Slice English long enough for a sentence to be worth moving.
 */
const CLEAN_TEXT = [
  'The tortoiseshell cat arrived at the shelter on a rainy Tuesday in November.',
  'She had been found under a parked van near the harbour, thin and unwilling to be touched.',
  'The volunteers named her Marmalade because of the orange patch across her left shoulder.',
].join(' ',);

/**
 English of another slice of the same imagined document.
 */
const DONOR_TEXT = [
  'Her favourite place in the whole building turned out to be the top of the filing cabinet.',
  'From there she could watch the front door without anybody watching her back.',
].join(' ',);

/**
 Sentence the donor offers, which is its longest unique one.
 */
const BORROWED = 'Her favourite place in the whole building turned out to be the top of the filing cabinet.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: deleteOneSentence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'removes one whole sentence and leaves the rest word for word, so the damaged text can '
            + 'lose on coverage and on nothing else',
          fn: async () => {
            const attempt = deleteOneSentence({ cleanText: CLEAN_TEXT, },);
            if (attempt.kind !== 'damaged')
              throw new Error(`expected damage, got ${attempt.reason}`,);
            expect(attempt.damageKind,).toBe('deletion',);
            expect(attempt.damagedText
              .length,).toBeLessThan(CLEAN_TEXT.length,);
            // Every surviving sentence must appear untouched: a deletion that also
            // rewrote its neighbours would let a judge choose on wording.
            expect(CLEAN_TEXT.includes('The volunteers named her Marmalade',),).toBe(true,);
            expect(attempt.damagedText
              .includes('The tortoiseshell cat arrived at the shelter',),).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES NO WHITESPACE MARK at the join, so the damaged candidate cannot be spotted by '
            + 'its typography instead of by what it fails to say',
          fn: async () => {
            const attempt = deleteOneSentence({ cleanText: CLEAN_TEXT, },);
            if (attempt.kind !== 'damaged')
              throw new Error(`expected damage, got ${attempt.reason}`,);
            // A sentence is stored trimmed and prose separates sentences on both
            // sides, so cutting the sentence alone leaves both separators: this ran
            // as a double space before the splice was written.
            expect(attempt.damagedText
              .includes('  ',),).toBe(false,);
          },
        },),
        it({
          name: 'leaves ONE paragraph break where a whole MIDDLE paragraph was, rather than the three '
            + 'consecutive line breaks a bare cut leaves behind',
          fn: async () => {
            /**
             Three paragraphs whose MIDDLE one carries the longest sentence, which
             is the one `deriveOmissionSeeds` picks. An earlier fixture put the
             longest sentence first, so the cut happened at the start of the text
             and this guard passed without ever exercising a middle join.
             */
            const document = [
              'The shelter opens at eight and closes late.',
              'Marmalade spent her first fortnight refusing to come out from behind the radiator in the back office, '
                + 'where the volunteers left her a bowl every morning and pretended not to watch.',
              'By March she was sleeping in the window.',
            ].join('\n\n',);
            const attempt = deleteOneSentence({ cleanText: document, },);
            if (attempt.kind !== 'damaged')
              throw new Error(`expected damage, got ${attempt.reason}`,);
            expect(attempt.damagedText
              .includes('\n\n\n',),).toBe(false,);
            expect(attempt.damagedText
              .includes('\n\n',),).toBe(true,);
          },
        },),
        it({
          name: 'keeps ONE trailing line break when the LAST paragraph goes, so the damaged text ends '
            + 'the way every other document does',
          fn: async () => {
            /**
             Two paragraphs whose LAST one carries the longest sentence, so the cut
             lands against the end of the text.
             */
            const document = 'The shelter opens at eight and closes late.\n\n'
              + 'She had been found under a parked van near the harbour that winter, thin and unwilling to be '
              + 'touched by anybody who came near her.\n';
            const attempt = deleteOneSentence({ cleanText: document, },);
            if (attempt.kind !== 'damaged')
              throw new Error(`expected damage, got ${attempt.reason}`,);
            expect(attempt.damagedText,).toBe('The shelter opens at eight and closes late.\n',);
          },
        },),
        it({
          name: 'REFUSES a passage with no sentence long enough to delete, rather than returning the '
            + 'text unchanged as though it had damaged it',
          fn: async () => {
            const attempt = deleteOneSentence({ cleanText: 'Short. Also short.', },);
            expect(attempt.kind,).toBe('undamageable',);
          },
        },),
        it({
          name: 'REFUSES a deletion that would leave the slice empty, which is a shape no translation '
            + 'takes',
          fn: async () => {
            expect(deleteOneSentence({
              cleanText: 'The tortoiseshell cat arrived at the shelter on a rainy Tuesday and stayed.',
            },),).toEqual({
              kind: 'undamageable',
              reason: 'deletion would leave the slice empty, which is a shape no translation takes',
            },);
          },
        },),
      ],
    },),

    describe({
      name: insertBorrowedSentence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'splices the borrowed sentence in, which makes the COMPLETE text the shorter candidate '
            + 'and is the whole reason this fixture exists beside the deletion',
          fn: async () => {
            const attempt = insertBorrowedSentence({
              cleanText: CLEAN_TEXT,
              donorTexts: [DONOR_TEXT,],
            },);
            if (attempt.kind !== 'damaged')
              throw new Error(`expected damage, got ${attempt.reason}`,);
            expect(attempt.damageKind,).toBe('insertion',);
            // THE INVERSION, stated as an assertion rather than as a comment: a
            // roster preferring length scores every deletion trial and fails here.
            expect(attempt.damagedText
              .length,).toBeGreaterThan(CLEAN_TEXT.length,);
            expect(attempt.damagedText
              .includes(BORROWED,),).toBe(true,);
            // Nothing of the clean text may be lost, or the damaged candidate would
            // carry two defects and a judge choosing it could be right about one.
            expect(attempt.damagedText
              .includes('The volunteers named her Marmalade',),).toBe(true,);
            expect(attempt.changedChars,).toBe(BORROWED.length,);
          },
        },),
        it({
          name: 'REFUSES to borrow a sentence the slice already carries, which would add nothing and '
            + 'would score a judge wrong for keeping a text that says the same things',
          fn: async () => {
            const attempt = insertBorrowedSentence({
              cleanText: `${CLEAN_TEXT} ${BORROWED}`,
              donorTexts: [DONOR_TEXT,],
            },);
            expect(attempt.kind,).toBe('undamageable',);
          },
        },),
        it({
          name: 'REFUSES when the donor offers no sentence, so an entry of one slice cannot produce a '
            + 'trial whose damaged candidate is the clean text',
          fn: async () => {
            const attempt = insertBorrowedSentence({
              cleanText: CLEAN_TEXT,
              donorTexts: [],
            },);
            expect(attempt.kind,).toBe('undamageable',);
          },
        },),
        it({
          name: 'SKIPS an empty offered sentence and borrows the first usable one, set after the clean '
            + 'text\'s second sentence',
          fn: async () => {
            expect(insertBorrowedSentence({
              cleanText: CLEAN_TEXT,
              donorTexts: ['', DONOR_TEXT,],
            },),).toEqual({
              kind: 'damaged',
              damageKind: 'insertion',
              damagedText: CLEAN_TEXT.replace(
                'touched. ',
                `touched. ${BORROWED} `,
              ),
              changedChars: BORROWED.length,
              damageDetail: `inserted a borrowed sentence of ${String(BORROWED.length,)} characters`,
            },);
          },
        },),
        it({
          name: 'REFUSES an insertion where no sentence is long enough and unique enough to splice '
            + 'after',
          fn: async () => {
            expect(insertBorrowedSentence({
              cleanText: 'Short. Also short.',
              donorTexts: [DONOR_TEXT,],
            },),).toEqual({
              kind: 'undamageable',
              reason: 'no sentence long enough and unique enough to splice after',
            },);
          },
        },),
      ],
    },),

    describe({
      name: alterSharedNumber.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'changes a number the ORIGINAL also states, leaving a candidate of the same length that '
            + 'reads exactly as well, so nothing but the original can decide it',
          fn: async () => {
            /** Chinese carrying the same digits the English renders. */
            const sourceText = '小猫于2011年3月7日出生，来自猫爪镇。她有3只玩具老鼠。';
            const cleanText = 'Marmalade was born on 7 March 2011 in Pawford. She owns 3 toy mice.';
            const attempt = alterSharedNumber({
              cleanText,
              sourceText,
            },);
            if (attempt.kind !== 'damaged')
              throw new Error(`expected damage, got ${attempt.reason}`,);
            expect(attempt.damageKind,).toBe('alteration',);
            // SAME LENGTH, which is what takes both length and fluency off the
            // table and leaves only the original.
            expect(attempt.damagedText
              .length,).toBe(cleanText.length,);
            expect(attempt.damagedText,).not
              .toBe(cleanText,);
            // The year the source states is gone, and what replaced it is stated
            // nowhere on either side.
            expect(attempt.damagedText
              .includes('2011',),).toBe(false,);
            /** Where the year sits in the damaged text. */
            const yearAt = attempt.damagedText
              .indexOf('March ',) + 'March '.length;

            /** Year the damaged text now states. */
            const statedYear = attempt.damagedText
              .slice(
              yearAt,
              yearAt + '2011'.length,
              );
            expect(sourceText.includes(statedYear,),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES a slice whose numbers the original does not state, since altering one would '
            + 'damage a claim the source never made and the trial could not call either text right',
          fn: async () => {
            const attempt = alterSharedNumber({
              cleanText: 'Marmalade was born in 2004 and owns 3 toy mice.',
              sourceText: '小猫出生于春天，她有几只玩具老鼠。',
            },);
            expect(attempt.kind,).toBe('undamageable',);
          },
        },),
        it({
          name: 'REFUSES a number the translation states TWICE, because the fixture would then change '
            + 'one mention and leave the other, which is a self-contradiction rather than an error',
          fn: async () => {
            const attempt = alterSharedNumber({
              cleanText: 'She joined in 2004. Everything changed in 2004.',
              sourceText: '她在2004年加入。2004年一切都变了。',
            },);
            expect(attempt.kind,).toBe('undamageable',);
          },
        },),
        it({
          name: 'REFUSES a number the original carries only INSIDE a longer one, since a Chinese QQ or '
            + 'phone number containing the digits does not state the year, and altering it would leave '
            + 'neither candidate supported',
          fn: async () => {
            const attempt = alterSharedNumber({
              cleanText: 'Marmalade was born in 2004 and still answers to nobody at all.',
              // The source states 120045, a handle, and no year: `includes` would
              // read 2004 out of it and license the alteration.
              sourceText: '小猫的编号是120045，她谁也不理。',
            },);
            expect(attempt.kind,).toBe('undamageable',);
          },
        },),
        it({
          name: 'ignores a lone digit, which collides with list markers and with digits inside longer '
            + 'numbers on both sides',
          fn: async () => {
            const attempt = alterSharedNumber({
              cleanText: 'She owns 3 toy mice and nothing else worth counting here.',
              sourceText: '她有3只玩具老鼠。',
            },);
            expect(attempt.kind,).toBe('undamageable',);
          },
        },),
        it({
          name: 'REFUSES an alteration where every same-shape number already appears on one side or '
            + 'the other',
          fn: async () => {
            expect(alterSharedNumber({
              cleanText: 'Mittens counted 2009 birds over 2000 2001 2002 2003 2004 2005 2006 2007 2008 days.',
              sourceText: '2009 年冬天，小猫在窗台上数了很多鸟。',
            },),).toEqual({
              kind: 'undamageable',
              reason: 'every same-shape number already appears on one side or the other',
            },);
          },
        },),
      ],
    },),
  ],
},);
