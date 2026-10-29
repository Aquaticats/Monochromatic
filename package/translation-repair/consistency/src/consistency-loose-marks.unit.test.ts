/**
 Tests for reading the single marks and the loose marks of a place's English:
 which characters are single marks, which single marks are part of a word, and
 how two marks beside the words of two places are compared.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isLetterOrDigitOfAnyScript, } from '@monochromatic-dev/translation-repair-markup';
import {
  apostropheMarksOf,
  holdsEdgeApostrophe,
  holdsEdgeMark,
  isCountedSingleMark,
  isSingleMark,
  looseMarksOf,
  sameBesideMark,
  singleMarkAt,
  withApostropheForm,
  withLooseMarksOf,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isSingleMark.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR THE STRAIGHT APOSTROPHE, BOTH CURLY SINGLE MARKS AND THE MODIFIER LETTER APOSTROPHE, and for no '
            + 'other character (the low, reversed and angle single marks are counted apart, `isCountedSingleMark`)',
          fn: async () => {
            expect([
              '\'',
              '‘',
              '’',
              '\u{02BC}',
              '\u{201A}',
              '‹',
              '"',
              '“',
              '”',
              '`',
              '\u{00B4}',
              '『',
              '',
              '\'\'',
            ].map(function single(character,): boolean {
              return isSingleMark({ character, },);
            },),).toEqual([true, true, true, true, false, false, false, false, false, false, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: isCountedSingleMark.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR THE STRAIGHT APOSTROPHE, BOTH CURLY SINGLE MARKS, THE MODIFIER LETTER APOSTROPHE, THE LOW AND THE '
            + 'REVERSED SINGLE MARK AND BOTH SINGLE ANGLE MARKS, the eight the pass counts as single marks and the record stores, '
            + 'and for no other character (the double guillemets among them)',
          fn: async () => {
            expect([
              '\'',
              '‘',
              '’',
              '\u{02BC}',
              '\u{201A}',
              '\u{201B}',
              '‹',
              '›',
              '"',
              '“',
              '”',
              '«',
              '»',
              '`',
              '\u{00B4}',
              '『',
              '',
              '\'\'',
            ].map(function counted(character,): boolean {
              return isCountedSingleMark({ character, },);
            },),).toEqual([
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
            ],);
          },
        },),
      ],
    },),
    describe({
      name: singleMarkAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE SINGLE MARK AT AN OFFSET AS THE TEXT WRITES IT, and none for another character or an offset '
            + 'outside the text',
          fn: async () => {
            const english = 'the cats’ bowl "is" full';
            expect([
              singleMarkAt({ text: english, at: 8, },),
              singleMarkAt({ text: english, at: 7, },),
              singleMarkAt({ text: english, at: 11, },),
              singleMarkAt({ text: english, at: 14, },),
              singleMarkAt({ text: english, at: -1, },),
              singleMarkAt({ text: english, at: english.length, },),
              singleMarkAt({ text: '', at: 0, },),
            ],).toEqual([['’',], [], [], [], [], [], [],],);
          },
        },),
        it({
          name: 'GIVES THE LOW, THE REVERSED AND THE SINGLE ANGLE MARKS TOO, as the text writes them, since the pass counts all eight '
            + 'as single marks beside a span',
          fn: async () => {
            const english = 'a‚b‛c‹d›e«f»g';
            expect(Array.from(english,).map(function markAtEach(_character, at,): readonly string[] {
              return singleMarkAt({ text: english, at, },);
            },),).toEqual([[], ['‚',], [], ['‛',], [], ['‹',], [], ['›',], [], [], [], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: holdsEdgeMark.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR A SINGLE MARK AT THE ASKED EDGE WHATEVER STANDS INSIDE IT, a letter, markup that closes nothing, a '
            + 'space or another mark, each of the four single marks alike',
          fn: async () => {
            expect([
              holdsEdgeMark({ text: 'the parents’', side: 'end', },),
              holdsEdgeMark({ text: '_cats_ʼ', side: 'end', },),
              holdsEdgeMark({ text: 'the cats ʼ', side: 'end', },),
              holdsEdgeMark({ text: 'ʼ 5 cats', side: 'start', },),
              holdsEdgeMark({ text: '’round', side: 'start', },),
              holdsEdgeMark({ text: 'the cats\'', side: 'end', },),
              holdsEdgeMark({ text: 'the cats‘', side: 'end', },),
              holdsEdgeMark({ text: 'the cats’’', side: 'end', },),
            ],).toEqual([true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS TRUE FOR THE LOW, REVERSED AND ANGLE SINGLE MARKS AT AN EDGE TOO, since the counter and the record reader know '
            + 'all eight single marks and a gain the apply stage does not count would be made without a proof, though no page '
            + 'could prove such a mark an apostrophe',
          fn: async () => {
            expect([
              holdsEdgeMark({ text: 'the cats‚', side: 'end', },),
              holdsEdgeMark({ text: '‛the cats', side: 'start', },),
              holdsEdgeMark({ text: 'the cats‹', side: 'end', },),
              holdsEdgeMark({ text: '›the cats', side: 'start', },),
              holdsEdgeMark({ text: 'the cats‛', side: 'end', },),
              holdsEdgeMark({ text: 'the cats›', side: 'end', },),
              holdsEdgeMark({ text: '‚the cats', side: 'start', },),
              holdsEdgeMark({ text: '‹the cats', side: 'start', },),
            ],).toEqual([true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS FALSE FOR A MARK AT THE OTHER EDGE, A MARK THAT IS NO SINGLE MARK, AND AN EMPTY TEXT: a double mark, a corner '
            + 'bracket, a guillemet, a letter and a text that only holds a single mark inside',
          fn: async () => {
            expect([
              holdsEdgeMark({ text: 'the parents’', side: 'start', },),
              holdsEdgeMark({ text: '’round', side: 'end', },),
              holdsEdgeMark({ text: 'the cats”', side: 'end', },),
              holdsEdgeMark({ text: '「the cats', side: 'start', },),
              holdsEdgeMark({ text: 'the cats»', side: 'end', },),
              holdsEdgeMark({ text: '«the cats', side: 'start', },),
              holdsEdgeMark({ text: 'the cats', side: 'end', },),
              holdsEdgeMark({ text: 'the cat’s bowl', side: 'end', },),
              holdsEdgeMark({ text: '', side: 'end', },),
              holdsEdgeMark({ text: '', side: 'start', },),
            ],).toEqual([false, false, false, false, false, false, false, false, false, false,],);
          },
        },),
        it({
          name: 'READS A SINGLE MARK BEYOND THE FIRST PLANE AS WHOLE CHARACTERS, the last character being the asked edge: a '
            + 'pictograph before the mark and one after the words',
          fn: async () => {
            expect([
              holdsEdgeMark({ text: 'the 🐱’', side: 'end', },),
              holdsEdgeMark({ text: '’🐱 the', side: 'start', },),
              holdsEdgeMark({ text: 'the ’🐱', side: 'end', },),
            ],).toEqual([true, true, false,],);
          },
        },),
      ],
    },),
    describe({
      name: sameBesideMark.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS THE STRAIGHT APOSTROPHE, THE CLOSING CURLY MARK AND THE MODIFIER LETTER APOSTROPHE TO BE ONE MARK, and '
            + 'each mark equal to itself',
          fn: async () => {
            expect([
              sameBesideMark({ left: ['\'',], right: ['’',], },),
              sameBesideMark({ left: ['’',], right: ['\u{02BC}',], },),
              sameBesideMark({ left: ['\u{02BC}',], right: ['\'',], },),
              sameBesideMark({ left: ['‘',], right: ['‘',], },),
              sameBesideMark({ left: ['’',], right: ['’',], },),
            ],).toEqual([true, true, true, true, true,],);
          },
        },),
        it({
          name: 'KEEPS THE OPENING CURLY MARK APART FROM THE OTHERS AND AN ABSENT MARK EQUAL ONLY TO AN ABSENT ONE',
          fn: async () => {
            expect([
              sameBesideMark({ left: ['‘',], right: ['\'',], },),
              sameBesideMark({ left: ['’',], right: ['‘',], },),
              sameBesideMark({ left: [], right: ['\'',], },),
              sameBesideMark({ left: ['‘',], right: [], },),
              sameBesideMark({ left: [], right: [], },),
            ],).toEqual([false, false, false, false, true,],);
          },
        },),
      ],
    },),
    describe({
      name: looseMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES OUT A SINGLE MARK WITH A LATIN LETTER, WITH ITS ACCENT, OR AN ASCII DIGIT DIRECTLY ON BOTH SIDES, which is '
            + 'part of a word, and keeps one beside any other script, which is loose',
          fn: async () => {
            const small = String.fromCodePoint(0x1_04_28,);
            expect([
              looseMarksOf({ text: 'He doesn’t know the cat\'s name', },),
              looseMarksOf({ text: 'the dog\u{02BC}s bowl', },),
              looseMarksOf({ text: 'rock ’n’ roll in ’90s', },),
              looseMarksOf({ text: 'the 9\'10 cats', },),
              looseMarksOf({ text: 'α’β', },),
              looseMarksOf({ text: `${small}'${small}`, },),
              looseMarksOf({ text: 'the café’s cat', },),
              looseMarksOf({ text: '猫’猫', },),
              looseMarksOf({ text: 'αβ’γδ', },),
            ],).toEqual([[], [], ['\'', '\'', '\'',], [], ['\'',], ['\'',], [], ['\'',], ['\'',],],);
          },
        },),
        it({
          name: 'LEAVES OUT A SINGLE MARK WITH A LETTER OR DIGIT ON BOTH SIDES WHOSE NFKC FORM IS LATIN, read by whole character, '
            + 'and keeps one beside a character whose form is not',
          fn: async () => {
            expect([
              // Mathematical bold script letters beyond the first plane, then `s`.
              looseMarksOf({ text: '\u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}’s', },),
              // Fullwidth `Ａ`, the mark, fullwidth `ｓ`.
              looseMarksOf({ text: '\u{FF21}’\u{FF53}', },),
              // A fullwidth digit one, a decimal digit whose NFKC form is the ASCII digit `1`, then `s`.
              looseMarksOf({ text: '\u{FF11}’s', },),
              looseMarksOf({ text: '猫’猫', },),
              looseMarksOf({ text: 'αβ’γδ', },),
              // A micro sign (NFKC Greek mu) before `s`, and a lone surrogate before `s`.
              looseMarksOf({ text: '\u{00B5}’s', },),
              looseMarksOf({ text: '\u{D835}’s', },),
              // A circled digit one (category No) and a trade mark sign before `s`: no letter and no decimal digit.
              looseMarksOf({ text: '\u{2460}’s', },),
              looseMarksOf({ text: 'cat\u{2122}’s', },),
            ],).toEqual([[], [], [], ['\'',], ['\'',], ['\'',], ['\'',], ['\'',], ['\'',],],);
          },
        },),
        it({
          name: 'KEEPS EVERY OTHER SINGLE MARK AND EVERY DOUBLE MARK IN ORDER, each in its folded form: the double marks as the '
            + 'straight one, the apostrophe-like marks as the straight apostrophe, the opening curly mark as itself',
          fn: async () => {
            expect([
              looseMarksOf({ text: '“So she was kind,” she said', },),
              looseMarksOf({ text: 'She said ‘come in’ to "them"', },),
              looseMarksOf({ text: 'the cats’ bowl', },),
              looseMarksOf({ text: 'goin\u{02BC} home', },),
              looseMarksOf({ text: '‘Tis', },),
              looseMarksOf({ text: 'a cat’ s bowl', },),
            ],).toEqual([
              ['"', '"',],
              ['‘', '\'', '"', '"',],
              ['\'',],
              ['\'',],
              ['‘',],
              ['\'',],
            ],);
          },
        },),
        it({
          name: 'COUNTS EACH CORNER BRACKET AS A LOOSE MARK OF ITS OWN, in order and as itself: the four are never one mark, '
            + 'and none is folded to a double or a single mark',
          fn: async () => {
            expect([
              looseMarksOf({ text: '「So she was kind」, she said', },),
              looseMarksOf({ text: '『So she was kind』, she said', },),
              looseMarksOf({ text: '「She said 『come in』 to them」', },),
              looseMarksOf({ text: '“So she was kind,” 「she said」', },),
              looseMarksOf({ text: 'He said \'hi\' and 『hi』', },),
            ],).toEqual([
              ['「', '」',],
              ['『', '』',],
              ['「', '『', '』', '」',],
              ['"', '"', '「', '」',],
              ['\'', '\'', '『', '』',],
            ],);
          },
        },),
        it({
          name: 'NEVER TAKES A CORNER BRACKET FOR PART OF A WORD, a letter or a digit on both sides of it making no difference',
          fn: async () => {
            expect([
              looseMarksOf({ text: 'it『cat』and', },),
              looseMarksOf({ text: 'the 9「10」cats', },),
              looseMarksOf({ text: 'α「β」γ', },),
            ],).toEqual([
              ['『', '』',],
              ['「', '」',],
              ['「', '」',],
            ],);
          },
        },),
        it({
          name: 'COUNTS THE LOW, REVERSED, ANGLE AND GUILLEMET MARKS AS LOOSE MARKS OF THEIR OWN, in order and as themselves, '
            + 'so a carry that changes their number or order is refused',
          fn: async () => {
            expect([
              looseMarksOf({ text: 'She said ‹hello› to the cats', },),
              looseMarksOf({ text: 'She said «hello» to the cats', },),
              looseMarksOf({ text: 'She said ‚hello‘ to the cats', },),
              looseMarksOf({ text: 'She said ‛hello’ to the cats', },),
              looseMarksOf({ text: 'it‹cat›and', },),
              looseMarksOf({ text: 'She said hello to the cats', },),
            ],).toEqual([
              ['‹', '›',],
              ['«', '»',],
              ['‚', '‘',],
              ['‛', '\'',],
              ['‹', '›',],
              [],
            ],);
          },
        },),
        it({
          name: 'COUNTS THE LOW DOUBLE AND THE REVERSED DOUBLE QUOTATION MARK AS DOUBLE MARKS, each folded to the straight double '
            + 'mark, so a carry that adds, removes or moves one is refused like any double mark, and the okina, the prime and the '
            + 'double prime, which are a letter and unit marks and no quotation marks, stay no loose mark at all',
          fn: async () => {
            expect([
              looseMarksOf({ text: 'She said „hello” to the cats', },),
              looseMarksOf({ text: 'She said ‟hello” to the cats', },),
              looseMarksOf({ text: 'She said „hello to the cats', },),
              looseMarksOf({ text: 'it„cat‟and', },),
              looseMarksOf({ text: 'The Hawaiʻi cats', },),
              looseMarksOf({ text: 'The 5′ and 5″ cats', },),
            ],).toEqual([
              ['"', '"',],
              ['"', '"',],
              ['"',],
              ['"', '"',],
              [],
              [],
            ],);
          },
        },),
        it({
          name: 'WRITES A LOW OR REVERSED DOUBLE MARK IN THE FORM OF THE DOUBLE MARK THE OTHER TEXT HOLDS AT THE SAME POSITION, as it '
            + 'does a curly one',
          fn: async () => {
            expect([
              withLooseMarksOf({ text: '"So kind," she said', like: '„Be kind” he said', },),
              withLooseMarksOf({ text: '„So kind” she said', like: '“Be kind” he said', },),
            ],).toEqual(['„So kind,” she said', '“So kind” she said',],);
          },
        },),
        it({
          name: 'COUNTS THE OPENING CURLY SINGLE MARK BETWEEN TWO LETTERS AS A QUOTATION MARK, never as part of a word, while the '
            + 'other single marks between letters stay part of one',
          fn: async () => {
            expect([
              looseMarksOf({ text: 'the cat‘s bowl', },),
              looseMarksOf({ text: 'the cat’s bowl', },),
              looseMarksOf({ text: 'the 5‘2 tall', },),
              apostropheMarksOf({ text: 'the cat‘s bowl', },),
            ],).toEqual([['‘',], [], ['‘',], [],],);
          },
        },),
        it({
          name: 'FINDS NO LOOSE MARK IN A TEXT THAT HOLDS NONE, or in no text',
          fn: async () => {
            expect([
              looseMarksOf({ text: 'The cat naps (quietly), **well**.', },),
              looseMarksOf({ text: '', },),
            ],).toEqual([[], [],],);
          },
        },),
        it({
          name: 'LEAVES OUT AN APOSTROPHE-LIKE MARK AT AN EDGE OF THE SPAN WITH A LETTER OR A DIGIT DIRECTLY INSIDE IT, which '
            + 'is a proven apostrophe the span took in with its word, and keeps every other mark at an edge',
          fn: async () => {
            expect([
              looseMarksOf({ text: 'the parents’', },),
              looseMarksOf({ text: '’round the cats', },),
              looseMarksOf({ text: 'the \'cats\' bowl and the dogs\'', },),
              looseMarksOf({ text: '‘Mio', },),
              looseMarksOf({ text: 'the cats ’', },),
              looseMarksOf({ text: '“So kind,” she said', },),
              looseMarksOf({ text: '', },),
            ],).toEqual([
              [],
              [],
              ['\'', '\'',],
              ['‘',],
              ['\'',],
              ['"', '"',],
              [],
            ],);
          },
        },),
        it({
          name: 'HOLDS NO LOOSE MARK FOR AN EDGE APOSTROPHE PAST CLOSING EMPHASIS, a proven mark after closing emphasis that a '
            + 'letter stands before being part of its word as one directly beside the letter is, and keeps a mark beside no '
            + 'letter, a mark before opening markup and a mark after a closing quotation mark loose',
          fn: async () => {
            expect([
              looseMarksOf({ text: '*kittens*’', },),
              looseMarksOf({ text: '’*tis*', },),
              looseMarksOf({ text: '"kittens"’', },),
              looseMarksOf({ text: '* ’', },),
              looseMarksOf({ text: '*kittens*‘', },),
            ],).toEqual([
              [],
              ['\'',],
              ['"', '"', '\'',],
              ['\'',],
              ['‘',],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: holdsEdgeApostrophe.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR AN APOSTROPHE-LIKE MARK AT THE ASKED EDGE WITH A LETTER OR A DIGIT DIRECTLY INSIDE IT, and false '
            + 'for the opening curly mark, a mark beside a space, a mark at the other edge and an empty text',
          fn: async () => {
            expect([
              holdsEdgeApostrophe({ text: 'the parents’', side: 'end', },),
              holdsEdgeApostrophe({ text: 'the parents\'', side: 'end', },),
              holdsEdgeApostrophe({ text: 'the parents\u{02BC}', side: 'end', },),
              holdsEdgeApostrophe({ text: '’round the cats', side: 'start', },),
              holdsEdgeApostrophe({ text: 'the parents’', side: 'start', },),
              holdsEdgeApostrophe({ text: '’round the cats', side: 'end', },),
              holdsEdgeApostrophe({ text: 'the parents‘', side: 'end', },),
              holdsEdgeApostrophe({ text: 'the parents ’', side: 'end', },),
              holdsEdgeApostrophe({ text: '', side: 'end', },),
            ],).toEqual([true, true, true, true, false, false, false, false, false,],);
          },
        },),
        it({
          name: 'READS A MARK PAST CLOSING EMPHASIS OR A LINK AS THE EDGE APOSTROPHE OF THE WORD BEFORE IT, the way the proof reads '
            + 'it, and no other mark beside markup: a mark after a bracket, inline code or a double mark is read where it stands, '
            + 'and a mark before opening markup has no mirror reading',
          fn: async () => {
            expect([
              holdsEdgeApostrophe({ text: '*kittens*’', side: 'end', },),
              holdsEdgeApostrophe({ text: '[kittens](https://cats.example/k)’', side: 'end', },),
              holdsEdgeApostrophe({ text: 'Come (here)\'', side: 'end', },),
              holdsEdgeApostrophe({ text: 'the `kittens`\u{02BC}', side: 'end', },),
              holdsEdgeApostrophe({ text: '"kittens"’', side: 'end', },),
              holdsEdgeApostrophe({ text: '’*tis*', side: 'start', },),
              holdsEdgeApostrophe({ text: '\'(tis) well', side: 'start', },),
              holdsEdgeApostrophe({ text: '*kittens*‘', side: 'end', },),
              holdsEdgeApostrophe({ text: '* ’', side: 'end', },),
              holdsEdgeApostrophe({ text: '*’', side: 'end', },),
              holdsEdgeApostrophe({ text: '猫*’', side: 'end', },),
              holdsEdgeApostrophe({ text: '*kittens* ’', side: 'end', },),
              holdsEdgeApostrophe({ text: '’* tis', side: 'start', },),
              holdsEdgeApostrophe({ text: '’*猫*', side: 'start', },),
              holdsEdgeApostrophe({ text: '*kittens*’', side: 'start', },),
            ],).toEqual([
              true,
              true,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
            ],);
          },
        },),
        it({
          name: 'READS A LONE EMPHASIS CHARACTER BETWEEN THE LETTER AND THE MARK AS A CHARACTER OF THE WORDS, so the mark is no edge '
            + 'apostrophe, while real emphasis, strong emphasis or strikethrough before the mark still lets the letter before it count',
          fn: async () => {
            expect([
              holdsEdgeApostrophe({ text: 'kittens*’', side: 'end', },),
              holdsEdgeApostrophe({ text: 'kittens_’', side: 'end', },),
              holdsEdgeApostrophe({ text: '猫_cats_’', side: 'end', },),
              holdsEdgeApostrophe({ text: 'a*’', side: 'end', },),
              holdsEdgeApostrophe({ text: '*kittens*’', side: 'end', },),
              holdsEdgeApostrophe({ text: '_kittens_’', side: 'end', },),
              holdsEdgeApostrophe({ text: '**kittens**’', side: 'end', },),
              holdsEdgeApostrophe({ text: '~~kittens~~’', side: 'end', },),
            ],).toEqual([false, false, false, false, true, true, true, true,],);
          },
        },),
      ],
    },),
    describe({
      name: apostropheMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE APOSTROPHES THAT ARE PART OF A WORD, in order and as written: those a letter or a digit attaches on '
            + 'both sides and one at an edge of the words with a letter or a digit inside it, and not a loose mark',
          fn: async () => {
            expect([
              apostropheMarksOf({ text: 'the dog\'s and the cat’s bowl', },),
              apostropheMarksOf({ text: 'the parents’', },),
              apostropheMarksOf({ text: '’round the cats', },),
              apostropheMarksOf({ text: 'the dogs\u{02BC}', },),
              apostropheMarksOf({ text: 'rock\'n\'roll', },),
              apostropheMarksOf({ text: 'He said ‘go’ and \'stop\' now', },),
              apostropheMarksOf({ text: '', },),
            ],).toEqual([
              ['\'', '’',],
              ['’',],
              ['’',],
              ['\u{02BC}',],
              ['\'', '\'',],
              [],
              [],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: withApostropheForm.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES EVERY APOSTROPHE THAT IS PART OF A WORD IN THE GIVEN FORM AND TOUCHES NO LOOSE MARK, the straight '
            + 'apostrophe, the closing curly mark and the modifier letter apostrophe each a form of its own',
          fn: async () => {
            expect([
              withApostropheForm({ text: 'the dog\'s and the cat’s bowl', form: '’', },),
              withApostropheForm({ text: 'He said \'go\' and the cat\'s bowl', form: '’', },),
              withApostropheForm({ text: 'the parents’', form: '\'', },),
              withApostropheForm({ text: '’round the cats', form: '\u{02BC}', },),
              withApostropheForm({ text: 'the cats nap', form: '’', },),
            ],).toEqual([
              'the dog’s and the cat’s bowl',
              'He said \'go\' and the cat’s bowl',
              'the parents\'',
              '\u{02BC}round the cats',
              'the cats nap',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: withLooseMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES EACH LOOSE MARK AS THE LOOSE MARK AT THE SAME POSITION OF THE OTHER TEXT, straight onto curly and curly '
            + 'onto straight, the apostrophes alike, and one that is part of a word as the text wrote it',
          fn: async () => {
            expect([
              withLooseMarksOf({ text: '"So kind," she said', like: '“Be kind,” he said', },),
              withLooseMarksOf({ text: '“So kind,” she said', like: '"Be kind," he said', },),
              withLooseMarksOf({ text: 'the cats\' bowl and \'dish\'', like: 'the kittens’ bowl and \u{02BC}pot\u{02BC}', },),
              withLooseMarksOf({ text: 'the cat\'s "bowl"', like: 'the kitten’s “dish”', },),
              withLooseMarksOf({ text: 'He said ‘go’ now', like: 'She said ‘stop’ soon', },),
              withLooseMarksOf({ text: 'she was 「so kind」', like: 'she was 「so merry」', },),
              withLooseMarksOf({ text: 'the cats nap', like: 'the dogs sleep', },),
              withLooseMarksOf({ text: '', like: '', },),
            ],).toEqual([
              '“So kind,” she said',
              '"So kind," she said',
              // The mark at the end of the words is a proven apostrophe of the span, a part of its word, kept as written.
              'the cats’ bowl and \u{02BC}dish\'',
              'the cat\'s “bowl”',
              'He said ‘go’ now',
              'she was 「so kind」',
              'the cats nap',
              '',
            ],);
          },
        },),
        it({
          name: 'REFUSES TWO TEXTS WHOSE SEQUENCES OF LOOSE MARKS DIFFER IN LENGTH, since the caller compares them first and '
            + 'nothing is guessed',
          fn: async () => {
            /**
             The error the call throws.
             */
            const thrown = caught(function act(): unknown {
              return withLooseMarksOf({ text: '"So kind," she said', like: 'Be kind, he said', },);
            },);
            expect(String(thrown,),).toBe(
              'ConsistencyInvariantError: unreachable: the words hold 2 loose marks and the words they take the marks\' form from '
                + 'hold 0, and the sequences are compared before the marks are written',
            );
          },
        },),
      ],
    },),
    describe({
      name: isLetterOrDigitOfAnyScript.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR ONE WHOLE LETTER OR DECIMAL DIGIT OF ANY SCRIPT, Latin, Han, Greek, Arabic-Indic and beyond the first '
            + 'plane, where the neighbour test of the consistency pass names Latin letters, ASCII digits and their '
            + 'compatibility forms only',
          fn: async () => {
            expect([
              'a',
              'é',
              '7',
              '猫',
              'α',
              // U+0663 ARABIC-INDIC DIGIT THREE, a decimal digit that is no ASCII digit.
              '\u{0663}',
              // U+10428 DESERET SMALL LETTER LONG I, a letter beyond the first plane.
              '\u{10428}',
            ].map(function asked(character,): boolean {
              return isLetterOrDigitOfAnyScript({ character, },);
            },),).toEqual([true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS FALSE FOR NOTHING, MARKS, SPACE, SIGNS, A NUMBER THAT IS NO DECIMAL DIGIT, A LONE SURROGATE AND MORE THAN ONE '
            + 'CHARACTER, since the test is anchored to exactly one whole character',
          fn: async () => {
            expect([
              '',
              '’',
              ' ',
              '_',
              // U+0301 COMBINING ACUTE ACCENT, a mark.
              '\u{0301}',
              // U+00B2 SUPERSCRIPT TWO, a number that is no decimal digit.
              '\u{00B2}',
              // U+D835 a lone high surrogate.
              '\u{D835}',
              '🐱',
              'ab',
              '猫猫',
            ].map(function asked(character,): boolean {
              return isLetterOrDigitOfAnyScript({ character, },);
            },),).toEqual([false, false, false, false, false, false, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: `${holdsEdgeApostrophe.name} by the neighbour test of the consistency pass`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS AN EDGE APOSTROPHE ONLY WITH A LATIN LETTER, WITH ITS ACCENT, OR AN ASCII DIGIT INSIDE IT, so a mark beside '
            + 'a Han character or a Greek letter is no apostrophe of a word the gain case could carry',
          fn: async () => {
            expect([
              holdsEdgeApostrophe({ text: 'the parents’', side: 'end', },),
              holdsEdgeApostrophe({ text: 'the 猫’', side: 'end', },),
              holdsEdgeApostrophe({ text: 'the αβ’', side: 'end', },),
              holdsEdgeApostrophe({ text: '’猫 cats', side: 'start', },),
              holdsEdgeApostrophe({ text: '’5 cats', side: 'start', },),
              holdsEdgeApostrophe({ text: '’é cats', side: 'start', },),
            ],).toEqual([true, false, false, false, true, true,],);
          },
        },),
      ],
    },),
  ],
},);
