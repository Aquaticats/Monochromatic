/**
 Tests for the reading of the page that spares the trials of the splices put back one at a time: a splice whose
 apostrophe no trial could prove is blocked from one reading of the page, and every other splice is left to its trial.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  screenSide,
  readmitLedgerOf,
} from '../../dist/final/node/index.mjs';
import {
  droppedRound,
  type SecondSliceSplice,
} from './consistency-readmit-page.test-fixture.ts';

/**
 The key of the one splice the pages of these cases leave out: the second slice's first.
 */
const KEY = '1 group 1 OCCURRENCE 2';

/**
 The text the published page carries around its two slices when an opener stands between them.
 */
const OPENED = ['', '\n\nShe said ‘fill it.\n\n', '',];

/**
 The splice that puts a trailing apostrophe on the family of the second slice.
 */
const TRAILING: SecondSliceSplice = { quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', };

/**
 The splice that puts a leading apostrophe on the family of the second slice.
 */
const LEADING: SecondSliceSplice = { quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: '’the family', };

/**
 What a screen of one side blocked, for a page whose second slice leaves the splices named out.

 @param second - English of the second slice
 @param edits - replacements of the second slice
 @param side - edge the screen reads
 @param between - text the page carries around its slices, blank line between them where none is given
 @param refused - indexes of the slices the floors refused
 @returns Keys of the splices blocked, and the reason given for each
 @example
 ```ts
 const seen = screened({ second, edits: [TRAILING,], side: 'end', },);
 ```
 */
function screened(
  {
    second,
    edits,
    side,
    between,
    refused = new Set<number>(),
  }: {
    readonly second: string;
    readonly edits: readonly SecondSliceSplice[];
    readonly side: 'start' | 'end';
    readonly between?: readonly string[];
    readonly refused?: ReadonlySet<number>;
  },
): { readonly blocked: readonly string[]; readonly reasons: readonly (readonly [string, unknown])[]; } {
  const { page, round, } = droppedRound({
    second,
    edits,
    dropped: edits.map(function nameOf(_edit, at,): string {
      return `group ${String(at + 1,)} OCCURRENCE 2`;
    },),
  },);
  const ledger = readmitLedgerOf({
    page: (between === undefined) ? page : { ...page, between, },
    round,
    refused,
  },);
  screenSide({ ledger, side, },);
  return {
    blocked: [...ledger.blocked,],
    reasons: ledger.reasonLog.map(function pairOf({ key, reason, },): readonly [string, unknown] {
      return [key, reason,];
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: screenSide.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BLOCKS A TRAILING MARK AN OPENER OF THE PAGE COULD PAIR WITH, with the reason that a mark could pair with it, '
            + 'and leaves the same splice to its trial on a page with no opener',
          fn: async () => {
            const second = 'Fill the family bowl.';
            expect([
              screened({ second, edits: [TRAILING,], side: 'end', between: OPENED, },),
              screened({ second, edits: [TRAILING,], side: 'end', },),
            ],).toEqual([
              { blocked: [KEY,], reasons: [[KEY, { kind: 'pairs', },],], },
              { blocked: [], reasons: [], },
            ],);
          },
        },),
        it({
          name: 'BLOCKS A LEADING MARK A CLOSER LATER ON THE PAGE COULD PAIR WITH, and reads only the side it is asked for',
          fn: async () => {
            const closed = 'Fill the family bowl. Then the pups’ nap.';
            const open = 'Fill the family bowl. Then the pups nap.';
            expect([
              screened({ second: closed, edits: [LEADING,], side: 'start', },),
              screened({ second: open, edits: [LEADING,], side: 'start', },),
              screened({ second: closed, edits: [LEADING,], side: 'end', },),
              screened({ second: 'Fill the family bowl.', edits: [TRAILING,], side: 'start', between: OPENED, },),
            ],).toEqual([
              { blocked: [KEY,], reasons: [[KEY, { kind: 'pairs', },],], },
              { blocked: [], reasons: [], },
              { blocked: [], reasons: [], },
              { blocked: [], reasons: [], },
            ],);
          },
        },),
        it({
          name: 'LEAVES TO ITS TRIAL A SPLICE THAT TOUCHES ANOTHER REPLACEMENT OF ITS SLICE, whose words one reading cannot '
            + 'speak for',
          fn: async () => {
            /**
             A replacement that begins where the splice ends.
             */
            const touching: SecondSliceSplice = { quote: 'bowl', chosen: 'the pups’', start: 15, end: 20, text: 'bowls', };
            expect(screened({
              second: 'Fill the family bowl.',
              edits: [TRAILING, touching,],
              side: 'end',
              between: OPENED,
            },).blocked,).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES TO ITS TRIAL A SPLICE WHOSE WORDS HOLD ANOTHER QUOTATION MARK BESIDE THE APOSTROPHE IT GAINS',
          fn: async () => {
            /**
             A splice whose words hold a second quotation mark.
             */
            const quoted: SecondSliceSplice = { ...TRAILING, text: '“the parents’', };
            expect(screened({ second: 'Fill the family bowl.', edits: [quoted,], side: 'end', between: OPENED, },).blocked,).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES TO ITS TRIAL A SPLICE WHOSE WORDS TAKE OUT A QUOTATION MARK',
          fn: async () => {
            expect(screened({
              second: 'Fill the “family bowl.',
              edits: [{ ...TRAILING, end: 16, },],
              side: 'end',
              between: OPENED,
            },).blocked,).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES TO ITS TRIAL A SPLICE WITH A LETTER BESIDE ITS MARK, in its own slice or in the text the page carries '
            + 'after it, since the letter would continue the word the mark ends',
          fn: async () => {
            expect([
              screened({ second: 'Fill the familyx bowl.', edits: [{ ...TRAILING, end: 15, },], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family', edits: [TRAILING,], side: 'end', between: ['', '\n\nShe said ‘x. ', 'Then',], },).blocked,
            ],).toEqual([[], [],],);
          },
        },),
        it({
          name: 'READS A SPLICE AT THE EDGE OF THE PAGE, where nothing stands beside the mark and no other slice\'s words can',
          fn: async () => {
            expect(screened({
              second: 'Fill the family',
              edits: [TRAILING,],
              side: 'end',
              between: ['', '\n\nShe said ‘x. ', '',],
            },).blocked,).toEqual([KEY,],);
          },
        },),
        it({
          name: 'LEAVES TO ITS TRIAL A SPLICE WHOSE MARK THE PROOF READS FREE, a letter of another script, a symbol, a pictograph or '
            + 'a single mark beside it, since such a mark is a possible partner of every later mark and the page with it made '
            + 'would block splices that are proven without it; a leading mark likewise',
          fn: async () => {
            expect([
              screened({ second: 'Fill the familyα bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family猫 bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family© bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family™ bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family→ bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family🐱 bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family\' bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family‚ bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill αthe family bowl. Then the pups’ nap.', edits: [{ ...LEADING, start: 6, end: 16, },], side: 'start', },).blocked,
              screened({ second: 'Fill 猫the family bowl. Then the pups’ nap.', edits: [{ ...LEADING, start: 6, end: 16, },], side: 'start', },).blocked,
              screened({ second: 'Fill 🐱the family bowl. Then the pups’ nap.', edits: [{ ...LEADING, start: 7, end: 17, },], side: 'start', },).blocked,
              screened({ second: 'Fill ’the family bowl. Then the pups’ nap.', edits: [{ ...LEADING, start: 6, end: 16, },], side: 'start', },).blocked,
            ],).toEqual([[], [], [], [], [], [], [], [], [], [], [], [],],);
          },
        },),
        it({
          name: 'LEAVES TO ITS TRIAL A SPLICE WHOSE MARK HAS NO LATIN LETTER INSIDE IT, a space or a bracket between the letters and '
            + 'the mark, since the proof reads such a mark free whatever stands beside it',
          fn: async () => {
            expect([
              screened({ second: 'Fill the family bowl.', edits: [{ ...TRAILING, text: 'the parents ’', },], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family bowl.', edits: [{ ...TRAILING, text: 'the parents)’', },], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family bowl. Then the pups’ nap.', edits: [{ ...LEADING, text: '’ the family', },], side: 'start', },).blocked,
              screened({ second: 'Fill the family bowl. Then the pups’ nap.', edits: [{ ...LEADING, text: '’(the family', },], side: 'start', },).blocked,
              screened({ second: 'Fill the family bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
            ],).toEqual([[], [], [], [], [KEY,],],);
          },
        },),
        it({
          name: 'STILL BLOCKS A SPLICE WHOSE MARK THE PROOF READS AS TRAILING OR LEADING ALONE, a space, a full stop, a closing bracket '
            + 'or the edge of the text beside it, and a Latin letter beside it is left to its trial',
          fn: async () => {
            expect([
              screened({ second: 'Fill the family bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family. bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill (the family) bowl.', edits: [{ ...TRAILING, start: 6, end: 16, },], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the familyx bowl.', edits: [TRAILING,], side: 'end', between: OPENED, },).blocked,
              screened({ second: 'Fill the family bowl. Then the pups’ nap.', edits: [LEADING,], side: 'start', },).blocked,
              screened({ second: 'Fill,the family bowl. Then the pups’ nap.', edits: [LEADING,], side: 'start', },).blocked,
              screened({ second: 'Fill xthe family bowl. Then the pups’ nap.', edits: [{ ...LEADING, start: 6, end: 16, },], side: 'start', },).blocked,
            ],).toEqual([[KEY,], [KEY,], [KEY,], [], [KEY,], [KEY,], [],],);
          },
        },),
        it({
          name: 'LEAVES TO ITS TRIAL A LEADING SPLICE AT THE START OF A SLICE WHOSE GAP IS EMPTY, where the words of the slice '
            + 'before could stand beside the mark and a splice of that slice could change them',
          fn: async () => {
            expect(screened({
              second: 'the family bowl. Then the pups’ nap.',
              edits: [{ ...LEADING, start: 0, end: 10, },],
              side: 'start',
              between: ['', '', '',],
            },).blocked,).toEqual([],);
          },
        },),
        it({
          name: 'READS ONLY A SPLICE WHOSE MARK THE PROOF WOULD READ AS THE APOSTROPHE OF ITS EDGE ON ITS OWN: a splice that ends in '
            + 'an opening double mark beside a space brings no single mark, and one that ends in a loose opening single mark '
            + 'brings a mark that is no apostrophe (no page proves it), so neither is made in the reading, and neither can '
            + 'block a splice whose apostrophe is proven without it; each is left to its trial',
          fn: async () => {
            /**
             A splice whose words end in a loose opening double mark, and a trailing apostrophe after it on the page.
             */
            const double: SecondSliceSplice = { quote: 'the pups', chosen: 'the pups “', start: 5, end: 13, text: 'the pups “', };
            /**
             A splice whose words end in a loose opening single mark.
             */
            const single: SecondSliceSplice = { quote: 'the pups', chosen: 'the pups ‘', start: 5, end: 13, text: 'the pups ‘', };
            const family: SecondSliceSplice = { quote: 'the family', chosen: 'the parents’', start: 23, end: 33, text: 'the parents’', };
            expect([double, single,].map(function blockedWith(loose,): readonly string[] {
              return screened({
                second: 'Fill the pups bowl and the family dish.',
                edits: [loose, family,],
                side: 'end',
              },).blocked;
            },),).toEqual([[], [],],);
          },
        },),
        it({
          name: 'READS NO SLICE THE FLOORS REFUSED, which keeps its wording whole and puts none of its splices back',
          fn: async () => {
            expect(screened({
              second: 'Fill the family bowl.',
              edits: [TRAILING,],
              side: 'end',
              between: OPENED,
              refused: new Set([1,]),
            },),).toEqual({ blocked: [], reasons: [], },);
          },
        },),
      ],
    },),
  ],
},);
