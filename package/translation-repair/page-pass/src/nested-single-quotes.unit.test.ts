/**
 Guards class one hundred forty-seven (XingZ6013, 2026-09-26). The butterfly
 speech quotes a law inside its own quotation, 「醒着就要活下去」 inside
 「…」, and the page shipped “… for the law that says 'to be awake is to keep
 living.' They can fly …”: the double marks curled, the inner pair straight,
 on a page every other mark of which is curly. The restoration curled a
 straight single quote only as an apostrophe, never as a quotation mark,
 while the corpus writes a nested quotation ‘…’ on every page that has one
 and never straight. A straight single pair that opens after a space, an
 opening mark or an emphasis delimiter in opening position and closes before a
 space or a closing mark is a quotation on one line, and on a curly page it is
 written curly. Pairs nest, so a quotation inside a quotation curls both; a
 doubled opener before an elision (`''tis the season'`) opens a quotation; `'99`
 is an elision like `'90s`; the `'n'` of `rock 'n' roll` is the elision unit only
 between two words, so a quoted letter elsewhere pairs. A replacement whose
 marks still cannot all pair keeps every one of them straight. Cat-themed
 invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { restoreTypography, } from '../dist/final/node/index.mjs';

/**
 Page text whose convention is curly throughout.
 */
const CURLY_PAGE = '“The cat’s bowl,” she said.';

/**
 Quotations in straight single marks whose first character is markup or punctuation, with the text the restoration writes on
 a curly page (the n155 review's W3 rows, `s224/probes/p-w-typo-own2.mjs`): the opening mark is one after white space and before
 any character that is not white space, so both marks curl.
 */
const OPENED_BY_MARKUP: readonly (readonly [string, string,])[] = [
  ['He said \'_hi_\' to me', 'He said ‘_hi_’ to me',],
  ['He said \'**hi**\' to me', 'He said ‘**hi**’ to me',],
  ['He said \'~~hi~~\' to me', 'He said ‘~~hi~~’ to me',],
  ['He said \'(hi)\' to me', 'He said ‘(hi)’ to me',],
  ['He said \'[hi]\' to me', 'He said ‘[hi]’ to me',],
  ['He said \'“hi”\' to me', 'He said ‘“hi”’ to me',],
  ['He said \'`hi`\' to me', 'He said ‘`hi`’ to me',],
  ['He said \'*hi* there\' to me', 'He said ‘*hi* there’ to me',],
  ['He said \'*hi*\' and \'bye\' to me', 'He said ‘*hi*’ and ‘bye’ to me',],
  ['He said \'*hi*\' and the dogs\' bowl', 'He said ‘*hi*’ and the dogs’ bowl',],
  ['He said \'...hi\' to me', 'He said ‘...hi’ to me',],
  ['He said \'—hi\' to me', 'He said ‘—hi’ to me',],
  ['He said \'¡hi!\' to me', 'He said ‘¡hi!’ to me',],
  ['He said \'¿hi?\' to me', 'He said ‘¿hi?’ to me',],
  ['He said \'@hi\' and \'#tag\' to me', 'He said ‘@hi’ and ‘#tag’ to me',],
  ['He said \'$5\' to me', 'He said ‘$5’ to me',],
  ['He said \'[link](http://x.example) is ok\' to me', 'He said ‘[link](http://x.example) is ok’ to me',],
];

await describe({
  name: 'a nested quotation in straight single marks (class one hundred forty-seven)',
  children: [
    it({
      name: 'CURLS a straight single pair inside a curly double quotation on a curly page',
      fn: async () => {
        expect(restoreTypography({
          replacement: '“The cat obeyed the law that says \'a nap is never wasted.\' Then it slept.”',
          replaced: '',
          convention: CURLY_PAGE,
        },),).toBe('“The cat obeyed the law that says ‘a nap is never wasted.’ Then it slept.”',);
      },
    },),
    it({
      name: 'CURLS the pair and the apostrophes inside it together',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'She kept saying \'it\'s only a mouse\' to the kitten.',
          replaced: '',
          convention: CURLY_PAGE,
        },),).toBe('She kept saying ‘it’s only a mouse’ to the kitten.',);
      },
    },),
    it({
      name: 'LEAVES the pair straight on a page written straight, and a lone opening mark with no partner on its line, but '
        + 'curls a lone mark before an elision (a decade) to the apostrophe, which no partner is needed to read',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'The law says \'a nap is never wasted.\'',
          replaced: '',
          convention: 'The cat\'s bowl.',
        },),).toBe('The law says \'a nap is never wasted.\'',);
        expect(restoreTypography({
          replacement: 'The \'ninety cat\nslept.',
          replaced: '',
          convention: CURLY_PAGE,
        },),).toBe('The \'ninety cat\nslept.',);
        expect(restoreTypography({
          replacement: 'The \'90s cat\nslept.',
          replaced: '',
          convention: CURLY_PAGE,
        },),).toBe('The ’90s cat\nslept.',);
      },
    },),
    it({
      name: 'LEAVES a possessive after an emphasis span as the apostrophe it is',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'The *Whiskers*\'s opening line was \'meow.\'',
          replaced: '',
          convention: CURLY_PAGE,
        },),).toBe('The *Whiskers*’s opening line was ‘meow.’',);
      },
    },),
    it({
      name: 'READS A STRAIGHT SINGLE QUOTE STANDING BETWEEN SPACES AS NEITHER MARK, so it neither closes the '
        + 'quotation open before it nor opens one, and the pair around it still curls',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'The cat kept \'a nap \' alone\' today.',
          replaced: '',
          convention: CURLY_PAGE,
        },),).toBe('The cat kept ‘a nap \' alone’ today.',);
      },
    },),
    it({
      name: 'READS `\'n\'` AS THE ELISION UNIT ONLY WITH A WORD DIRECTLY BEFORE AND AFTER IT, EACH ACROSS ONE SPACE (U5): a quoted '
        + 'letter in a list, at the end of a clause or at the start of a text pairs as a quotation, and `rock \'n\' roll` and '
        + '`fish \'n\' chips` stay two apostrophes',
      fn: async () => {
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention: CURLY_PAGE, },);
        }
        expect([
          curled('The letters \'a\', \'n\' and \'e\'',),
          curled('He said \'n\'',),
          curled('\'n\' alone',),
          curled('rock \'n\' roll',),
          curled('fish \'n\' chips',),
          curled('She said \'rock \'n\' roll rules\' and left.',),
          curled('rock  \'n\'  roll',),
          curled('rock\'n\'roll',),
        ],).toEqual([
          'The letters ‘a’, ‘n’ and ‘e’',
          'He said ‘n’',
          '‘n’ alone',
          'rock ’n’ roll',
          'fish ’n’ chips',
          'She said ‘rock ’n’ roll rules’ and left.',
          'rock  ‘n’  roll',
          'rock’n’roll',
        ],);
      },
    },),
    it({
      name: 'KEEPS THE ELISION READING OF A QUOTED LETTER BETWEEN TWO WORDS (U5, accepted limit): no test of the neighbours tells '
        + '`the letter \'n\' is silent` from `rock \'n\' roll`, so a word on each side of a quoted `n` or `N` reads as the unit',
      fn: async () => {
        expect([
          restoreTypography({
            replacement: 'the letter \'n\' is silent',
            replaced: '',
            convention: CURLY_PAGE,
          },),
          restoreTypography({
            replacement: 'Press \'N\' to continue',
            replaced: '',
            convention: CURLY_PAGE,
          },),
        ],).toEqual([
          'the letter ’n’ is silent',
          'Press ’N’ to continue',
        ],);
      },
    },),
    it({
      name: 'PAIRS SINGLE MARKS AS A STACK (U6): a quotation inside a quotation curls both pairs, and a replacement whose '
        + 'marks cannot all pair keeps every one of them straight',
      fn: async () => {
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention: CURLY_PAGE, },);
        }
        expect([
          curled('He said \'she said \'hi\' to me\' and left',),
          curled('“He said \'she said \'hi\' to me\' and left”',),
          curled('\'a \'b \'c\' d\' e\'',),
          curled('\'a \'b\' c\' \'d \'e\' f\'',),
          curled('He said \'O\'Brien said \'hi\'\' to us',),
          curled('He said \'the dogs\' and the cats\' bowls\' were full',),
          curled('the dogs\' bowl and \'quote start',),
          curled('\'quote start and the dogs\' bowl',),
        ],).toEqual([
          'He said ‘she said ‘hi’ to me’ and left',
          '“He said ‘she said ‘hi’ to me’ and left”',
          '‘a ‘b ‘c’ d’ e’',
          '‘a ‘b’ c’ ‘d ‘e’ f’',
          'He said ‘O’Brien said ‘hi’’ to us',
          'He said ‘the dogs’ and the cats’ bowls’ were full',
          'the dogs\' bowl and \'quote start',
          '‘quote start and the dogs’ bowl',
        ],);
      },
    },),
    it({
      name: 'READS AN EMPHASIS DELIMITER IN OPENING POSITION AS A PLACE A QUOTATION OPENS (U6): `*`, `_`, `~` and runs of them '
        + 'open it, and a delimiter that closes a span after a word does not, so `Chorus*\'sad` is read as `Chorus\'sad` is (n159)',
      fn: async () => {
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention: CURLY_PAGE, },);
        }
        expect([
          curled('He said *\'hi\'* and the dogs\' bowl',),
          curled('He said _\'hi\'_ and the dogs\' bowl',),
          curled('He said ~\'hi\'~ and the dogs\' bowl',),
          curled('He said **\'hi\'** and the dogs\' bowl',),
          curled('He said ([\'hi\']) and the dogs\' bowl',),
          curled('*\'hi\'* and the dogs\' bowl',),
          curled('She hummed *Cat Chorus*\'sad tune\' twice.',),
          curled('She hummed _Cat Chorus_\'sad tune\' twice.',),
          curled('[the dogs\' bowl](http://x.example/\'q\')',),
        ],).toEqual([
          'He said *‘hi’* and the dogs’ bowl',
          'He said _‘hi’_ and the dogs’ bowl',
          'He said ~‘hi’~ and the dogs’ bowl',
          'He said **‘hi’** and the dogs’ bowl',
          'He said ([‘hi’]) and the dogs’ bowl',
          '*‘hi’* and the dogs’ bowl',
          // CHANGED IN n159 (X7): the delimiters show a reader nothing, so `Chorus*'sad` is the mark between two letters that
          // `Chorus'sad` is, an apostrophe, and the mark that ends `tune'` is a trailing apostrophe.
          'She hummed *Cat Chorus*’sad tune’ twice.',
          'She hummed _Cat Chorus_’sad tune’ twice.',
          // The destination of a link is no prose to the mask (n156, W1), so its `'q'` is no pair of marks that no word before
          // opens: the label's apostrophe curls, and the address stays as written.
          '[the dogs’ bowl](http://x.example/\'q\')',
        ],);
      },
    },),
    it({
      name: 'PAIRS A DOUBLED OPENER BEFORE AN ELISION (U6): `\'\'tis the season\'` opens a quotation with its first mark and '
        + 'writes the second as the apostrophe of the elision, and `\'99` is an elision like `\'90s`',
      fn: async () => {
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention: CURLY_PAGE, },);
        }
        expect([
          curled('He said \'\'tis the season\' and left',),
          curled('\'\'em\'',),
          curled('\'\'Tis the season,\' she said',),
          curled('Back in \'99, she left',),
          curled('Back in \'99 and \'00.',),
          curled('He said \'99 kids\' twice.',),
          curled('Back in \'1999, she left',),
          curled('\'\'\'',),
          curled('\'\'',),
        ],).toEqual([
          'He said ‘’tis the season’ and left',
          '‘’em’',
          '‘’Tis the season,’ she said',
          'Back in ’99, she left',
          'Back in ’99 and ’00.',
          'He said ‘99 kids’ twice.',
          'Back in \'1999, she left',
          '\'\'\'',
          '\'\'',
        ],);
      },
    },),
    ...OPENED_BY_MARKUP.map(function markupOpenerCase([input, expected,],) {
      return it({
        name: `PAIRS A QUOTATION WHOSE FIRST CHARACTER IS NOT A WORD CHARACTER (n156, W3): ${input}`,
        fn: async () => {
          expect(restoreTypography({
            replacement: input,
            replaced: '',
            convention: CURLY_PAGE,
          },),).toBe(expected,);
        },
      },);
    },),
    it({
      name: 'NEVER CURLS ONE MARK OF A PAIR AND LEAVES ITS PARTNER STRAIGHT (n156, W3): after the restoration no text of the '
        + 'opened-by-markup rows holds a straight single mark beside a curly one',
      fn: async () => {
        /**
         Rows that hold a straight and a curly single mark together after the restoration.
         */
        const mixed = OPENED_BY_MARKUP
          .map(function restored([input,],): string {
            return restoreTypography({
              replacement: input,
              replaced: '',
              convention: CURLY_PAGE,
            },);
          },)
          .filter(function holdsBoth(output,): boolean {
            return output.includes('\'',) && (output.includes('\u{2018}',) || output.includes('\u{2019}',));
          },);
        expect(mixed,).toEqual([],);
      },
    },),
    it({
      name: 'STILL PAIRS A QUOTATION WHOSE FIRST CHARACTER IS A WORD CHARACTER, a number, and a quotation after white space '
        + 'inside the sentence (n156, W3 controls)',
      fn: async () => {
        expect([
          'He said \'hi *there*\' to me',
          'He said \'1\' and \'2\'',
          'He said \'hi\' and \'bye\' to me',
        ].map(function curled(replacement,): string {
          return restoreTypography({
            replacement,
            replaced: '',
            convention: CURLY_PAGE,
          },);
        },),).toEqual([
          'He said ‘hi *there*’ to me',
          'He said ‘1’ and ‘2’',
          'He said ‘hi’ and ‘bye’ to me',
        ],);
      },
    },),
    it({
      name: 'PAIRS A TWO-DIGIT ELISION DIRECTLY FOLLOWED BY A CLOSING STRAIGHT MARK AS A QUOTED NUMBER (n156, W7), inside a '
        + 'quotation and beside the same elision written without a closing mark',
      fn: async () => {
        expect([
          'He said \'he is \'42\' years old\' today.',
          'the 1990s and \'90s and \'90s\' music',
          'He said \'99\' and left.',
          'Back in \'99, she left',
          'The \'99 and \'00 crops.',
          'Class of \'99 was \'great\'.',
          'He said \'99 kids\' twice.',
        ].map(function curled(replacement,): string {
          return restoreTypography({
            replacement,
            replaced: '',
            convention: CURLY_PAGE,
          },);
        },),).toEqual([
          'He said ‘he is ‘42’ years old’ today.',
          // CHANGED IN n159: both readings pair one pair and leave one mark unpaired, and a tie keeps the elision, so the closing
          // mark closes the quotation that the first `'90s` opens (before, the quoted number always took the pair).
          'the 1990s and ‘90s and ’90s’ music',
          'He said ‘99’ and left.',
          'Back in ’99, she left',
          'The ’99 and ’00 crops.',
          'Class of ’99 was ‘great’.',
          'He said ‘99 kids’ twice.',
        ],);
      },
    },),
  ],
},);
