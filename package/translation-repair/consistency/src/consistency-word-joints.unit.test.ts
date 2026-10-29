/**
 Tests for telling whether a stand of some words in a folded English is cut out
 of a longer word: which marks join two letters, two digits or a letter and a
 digit, what a corner bracket and a dash do not join, that emphasis marks beside
 a joint hide it from no one, that an apostrophe after closing quotation marks,
 brackets, links and emphasis joins the word before them, and that the reading
 visits a bounded number of characters for each character of the English,
 counted without a clock.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { normalizePunctuation, } from '@monochromatic-dev/translation-repair-typography';
import {
  closingJointsOf,
  cutsWord,
  emphasisDelimitersOf,
  foldedLinksOf,
  foldWithOffsets,
} from '../dist/final/node/index.mjs';

/**
 Whether the one stand of a word in a text is cut out of a longer word, the
 text read as the locator reads an English.

 @param text - text before the fold, of characters in the first plane

 @param word - word, which stands once in the text

 @returns Whether an edge of the stand is a joint

 @example
 ```ts
 const cut = cutIn({ text: 'the so-called cat', word: 'so', },); // true
 ```
 */
function cutIn(
  {
    text,
    word,
  }: {
    readonly text: string;
    readonly word: string;
  },
): boolean {
  const folded = normalizePunctuation({ text, },);
  const originals = Array.from(text,);
  const at = folded.indexOf(word,);
  // The texts of these cases fold unit for unit, so the offsets of the parse are the offsets of the fold.
  const { offsets, closing, } = emphasisDelimitersOf({ english: text, },);
  return cutsWord({
    folded,
    originals,
    joints: closingJointsOf({
      folded,
      originals,
      closing,
    },),
    emphasis: offsets,
    at,
    end: at + word.length,
  },);
}

/**
 A text that counts the characters `cutsWord` reads from it, one for each call
 of `charAt` or `codePointAt`, so the work of a reading is measured without a
 clock. It stands in for the folded English, of which the reading asks only
 its length, characters and slices; any other method it asks for is absent and
 the case throws.

 @param text - folded English the double holds

 @returns Double, which `cutsWord` takes as the folded English, and a
 reader of the count of characters read from it so far

 @example
 ```ts
 const { folded, readsSoFar, } = countedText({ text: 'so-called', });
 ```
 */
function countedText({ text, }: { readonly text: string; },): {
  readonly folded: string;
  readonly readsSoFar: () => number;
} {
  /**
   Characters read so far, mutated only by the double's own methods.
   */
  const counted = { reads: 0, };
  const double = {
    length: text.length,
    charAt: (position: number,) => {
      counted.reads += 1;
      return text.charAt(position,);
    },
    codePointAt: (position: number,) => {
      counted.reads += 1;
      return text.codePointAt(position,);
    },
    slice: (start: number, end: number,) => text.slice(start, end,),
  };
  return {
    // The double has the reads `cutsWord` makes of a string, and no others.
    folded: double as unknown as string,
    readsSoFar: () => counted.reads,
  };
}

/**
 The most characters the reading may read for each character of the English
 when it is asked about every position of it: a constant number of reads for a
 stand and its neighbours, none for a walk along the run of emphasis marks, and
 the reads of the one pass over the whole English that the closing joints cost.
 */
const LINEAR_READS_PER_CHARACTER = 16;

/**
 How many characters are read when the closing joints of a text are read once and
 `cutsWord` is then asked about a stand of one character at each position of the
 text in turn.

 @param text - text before the fold, of characters in the first plane

 @returns Characters read over all the positions

 @example
 ```ts
 const reads = readsWhenProbingEveryPosition({ text: '**so**-called', },); // 1 or more
 ```
 */
function readsWhenProbingEveryPosition({ text, }: { readonly text: string; },): number {
  const { folded, readsSoFar, } = countedText({ text, },);
  const originals = Array.from(text,);
  const { offsets, closing, } = emphasisDelimitersOf({ english: text, },);
  // The closing joints are read once for the whole text, and their reads are counted with the rest.
  const joints = closingJointsOf({
    folded,
    originals,
    closing,
  },);
  for (let at = 0; at < text.length; at += 1)
    cutsWord({
      folded,
      originals,
      joints,
      emphasis: offsets,
      at,
      end: at + 1,
    },);
  return readsSoFar();
}

/**
 How many characters `cutsWord` reads for the one stand of a word in a text.

 @param text - text before the fold, of characters in the first plane

 @param word - word, which stands once in the text

 @returns Characters read

 @example
 ```ts
 const reads = readsForStand({ text: 'the so-called cat', word: 'so', },); // 1 or more
 ```
 */
function readsForStand(
  {
    text,
    word,
  }: {
    readonly text: string;
    readonly word: string;
  },
): number {
  const { folded, readsSoFar, } = countedText({ text, },);
  const originals = Array.from(text,);
  const at = text.indexOf(word,);
  const { offsets, closing, } = emphasisDelimitersOf({ english: text, },);
  cutsWord({
    folded,
    originals,
    // The joints are read from the plain text, so only the reads of the one stand are counted.
    joints: closingJointsOf({
      folded: text,
      originals,
      closing,
    },),
    emphasis: offsets,
    at,
    end: at + word.length,
  },);
  return readsSoFar();
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: cutsWord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE WHERE A HYPHEN OR AN APOSTROPHE JOINS THE STAND TO A LETTER, before it or after it, in every '
            + 'spelling of the two marks',
          fn: async () => {
            expect([
              cutIn({ text: 'the so-called cat', word: 'so', },),
              cutIn({ text: 'the so-called cat', word: 'called', },),
              cutIn({ text: 'the so\u{2010}called cat', word: 'so', },),
              cutIn({ text: 'the so\u{2011}called cat', word: 'so', },),
              cutIn({ text: 'the so\u{00AD}called cat', word: 'so', },),
              cutIn({ text: 'the dog\'s nap', word: 'dog', },),
              cutIn({ text: 'the dog’s nap', word: 'dog', },),
              cutIn({ text: 'the dog\u{02BC}s nap', word: 'dog', },),
              cutIn({ text: 'she don’t nap', word: 't', },),
            ],).toEqual([true, true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS FALSE FOR A STAND WITH A LETTER DIRECTLY BESIDE IT AND NO LINK BETWEEN, since a stand inside a longer word is '
            + 'refused by the word starts and the reading of a link run goes on only past a link that shows nothing',
          fn: async () => {
            expect([
              cutIn({ text: 'kitten', word: 'kit', },),
              cutIn({ text: 'kitten', word: 'ten', },),
              cutIn({ text: 'a kitten', word: 'kit', },),
              cutIn({ text: 'a kitten', word: 'ten', },),
              cutIn({ text: 'a kit tens', word: 'kit', },),
            ],).toEqual([false, false, false, false, false,],);
          },
        },),
        it({
          name: 'IS TRUE WHERE A FULL STOP OR A COMMA JOINS THE STAND TO A DIGIT, and where the stand itself starts or ends '
            + 'with the joint',
          fn: async () => {
            expect([
              cutIn({ text: 'it cost 0.5 dollars', word: '5 dollars', },),
              cutIn({ text: 'it cost 1,500 dollars', word: '500', },),
              cutIn({ text: 'it cost 0.5 dollars', word: '.5', },),
              cutIn({ text: 'the so-called cat', word: 'so-', },),
              cutIn({ text: 'it cost 0.5 cats', word: '5 cats', },),
            ],).toEqual([true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS TRUE WHERE A HYPHEN JOINS THE STAND TO A DIGIT OR A LETTER BESIDE A DIGIT, in every spelling of the '
            + 'hyphen, and for a range written with one',
          fn: async () => {
            expect([
              cutIn({ text: 'the Catch-22 nap', word: '22', },),
              cutIn({ text: 'the Catch-22 nap', word: 'Catch', },),
              cutIn({ text: 'the cat-5 nap', word: 'cat', },),
              cutIn({ text: 'the cat-5 nap', word: '5', },),
              cutIn({ text: 'the T-1000 nap', word: '1000', },),
              cutIn({ text: 'the T-1000 nap', word: 'T', },),
              cutIn({ text: 'a 3-D nap', word: '3', },),
              cutIn({ text: 'a 3-D nap', word: 'D', },),
              cutIn({ text: 'on 2026-10-13 she napped', word: '10', },),
              cutIn({ text: 'on 2026-10-13 she napped', word: '2026', },),
              cutIn({ text: 'on 2026-10-13 she napped', word: '13', },),
              cutIn({ text: 'see pages 10-12 on naps', word: '10', },),
              cutIn({ text: 'see pages 10-12 on naps', word: '12', },),
              cutIn({ text: 'the cat\u{2010}5 nap', word: 'cat', },),
              cutIn({ text: 'the cat\u{2011}5 nap', word: 'cat', },),
              cutIn({ text: 'the cat\u{00AD}5 nap', word: 'cat', },),
              cutIn({ text: 'a **cat**-5 nap', word: 'cat', },),
              cutIn({ text: 'a cat-**5** nap', word: '5', },),
            ],).toEqual([
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
            ],);
          },
        },),
        it({
          name: 'IS TRUE WHERE AN APOSTROPHE JOINS THE STAND TO A DIGIT OR A LETTER BESIDE A DIGIT, in every spelling of the '
            + 'apostrophe: a height, a letter then a digit, a decade after a letter, with emphasis beside the joint',
          fn: async () => {
            expect([
              cutIn({ text: 'He said 5\'2 tall.', word: '2 tall', },),
              cutIn({ text: 'He said 5\'2 tall.', word: '5', },),
              cutIn({ text: 'He said 5’2 tall.', word: '2 tall', },),
              cutIn({ text: 'He said 5\u{02BC}2 tall.', word: '2 tall', },),
              cutIn({ text: 'the cat\'5 nap', word: 'cat', },),
              cutIn({ text: 'the cat\'5 nap', word: '5', },),
              cutIn({ text: 'in mid\'90s she napped', word: '90s', },),
              cutIn({ text: 'in mid\'90s she napped', word: 'mid', },),
              cutIn({ text: 'in mid’90s she napped', word: '90s', },),
              cutIn({ text: 'a **cat**\'5 nap', word: 'cat', },),
              cutIn({ text: 'a cat\'**5** nap', word: '5', },),
              cutIn({ text: 'the cats of the 1990\'s napped', word: '1990', },),
              cutIn({ text: 'the cats of the 1990’s napped', word: '1990', },),
              cutIn({ text: 'the cat\'s nap', word: 'cat', },),
            ],).toEqual([true, true, true, true, true, true, true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS TRUE WHERE A COLON OR A SLASH HAS A DIGIT ON BOTH SIDES OF IT, at either end of the number',
          fn: async () => {
            expect([
              cutIn({ text: 'at 10:30 she napped', word: '30', },),
              cutIn({ text: 'at 10:30 she napped', word: '10', },),
              cutIn({ text: 'ate 1/2 of the fish', word: '1', },),
              cutIn({ text: 'ate 1/2 of the fish', word: '2', },),
            ],).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'READS A JOINT THROUGH THE EMPHASIS MARKS BESIDE THE STAND, and through those beside the joint',
          fn: async () => {
            expect([
              cutIn({ text: 'a **so**-called nap', word: 'so', },),
              cutIn({ text: 'a *cat*-like nap', word: 'cat', },),
              cutIn({ text: 'a so-**called** nap', word: 'called', },),
              cutIn({ text: 'a **so**-**called** nap', word: 'so', },),
              cutIn({ text: 'she liked *Catsong*’s tune', word: 'Catsong', },),
              cutIn({ text: 'a ~~so~~-called nap', word: 'so', },),
              cutIn({ text: 'a **so**-called nap', word: 'called', },),
              cutIn({ text: 'a _so_-called nap', word: 'so', },),
            ],).toEqual([true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS TRUE WHERE AN APOSTROPHE AFTER MARKUP JOINS A LETTER: emphasis or strikethrough that ends in a word, a '
            + 'link or a bracket or a closing curly quotation mark directly before the apostrophe, whether the stand '
            + 'ends before the apostrophe or starts after it',
          fn: async () => {
            expect([
              cutIn({ text: 'Fill the **cat**\'s bowl.', word: 'the **cat**', },),
              cutIn({ text: 'Fill the **cat**’s bowl.', word: 'the **cat**', },),
              cutIn({ text: 'Fill the _cat_\'s bowl.', word: 'the _cat_', },),
              cutIn({ text: 'Fill the ~~cat~~\'s bowl.', word: 'the ~~cat~~', },),
              cutIn({ text: 'Fill *Yorushika*’s bowl.', word: 'Yorushika', },),
              cutIn({ text: 'Fill [the cat](x)\'s bowl.', word: '[the cat](x)', },),
              cutIn({ text: 'Fill [the cat](x)’s bowl.', word: '[the cat](x)', },),
              cutIn({ text: 'Fill the (grey cat)\'s bowl.', word: '(grey cat)', },),
              cutIn({ text: 'Fill the [grey cat]\'s bowl.', word: '[grey cat]', },),
              cutIn({ text: 'She said “hi”’s bowl.', word: '"hi"', },),
              cutIn({ text: 'Fill [the cat](x)\'s bowl.', word: 's bowl', },),
              cutIn({ text: 'Fill the **cat**\'s bowl.', word: 's bowl', },),
              cutIn({ text: 'She said “hi”’s bowl.', word: 's bowl', },),
            ],).toEqual([true, true, true, true, true, true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS FALSE WHERE AN APOSTROPHE AFTER MARKUP JOINS NOTHING: beyond the emphasis stands a space or the start of '
            + 'the text, an opening bracket or an opening quotation mark, a straight double quotation mark before an '
            + 'apostrophe that starts the stand\'s word, and no letter after the apostrophe',
          fn: async () => {
            expect([
              cutIn({ text: '*\'Tis the season*', word: 'Tis', },),
              cutIn({ text: 'Give (\'em) more.', word: 'em', },),
              cutIn({ text: 'He said "\'Tis" twice.', word: 'Tis', },),
              cutIn({ text: 'He said “\'Tis” twice.', word: 'Tis', },),
              cutIn({ text: 'Fill the "cat"\'s bowl.', word: 's bowl', },),
              cutIn({ text: 'Fill the (grey cat)\' bowl.', word: '(grey cat)', },),
              cutIn({ text: 'Fill the **cat**\' bowl.', word: 'the **cat**', },),
              cutIn({ text: 'Fill the **cat** \'s bowl.', word: 'the **cat**', },),
              cutIn({ text: 'Fill the (grey cat) \'s bowl.', word: '(grey cat)', },),
              cutIn({ text: 'Fill the **cat**\'', word: 'the **cat**', },),
            ],).toEqual([false, false, false, false, false, false, false, false, false, false,],);
          },
        },),
        it({
          name: 'IS TRUE WHERE CLOSING MARKS STAND BETWEEN THE STAND AND AN APOSTROPHE THAT A LETTER OR A DIGIT FOLLOWS, '
            + 'whether or not the stand itself ends in one of them: a straight or curly closing double quotation mark, a '
            + 'closing round or square bracket, a link and its destination, and emphasis, in any mix',
          fn: async () => {
            expect([
              cutIn({ text: 'Fill the "cat"\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the "cat"\'s bowl.', word: '"cat"', },),
              cutIn({ text: 'Fill the “cat”’s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the “cat”\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the "cat"’s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the (grey cat)\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the (grey cat)\'s bowl.', word: 'grey cat', },),
              cutIn({ text: 'Fill the [grey cat]\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill [the cat](x)\'s bowl.', word: 'the cat', },),
              cutIn({ text: 'Fill [the cat](x)\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill [the cat](https://example.com/cats)\'s bowl.', word: 'the cat', },),
              cutIn({ text: 'Fill [the cat](https://example.com/Cat_(animal))\'s bowl.', word: 'the cat', },),
              cutIn({ text: 'Fill the **"cat"**\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the ("cat")\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the *(cat)*\'s bowl.', word: '(cat)', },),
              cutIn({ text: 'Fill the "cat"\'5 bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the **cat**\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the _cat_\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the ~~cat~~\'s bowl.', word: 'cat', },),
              cutIn({ text: `Fill the cat${')'.repeat(8,)}'s bowl.`, word: 'cat', },),
              cutIn({ text: `Fill [the cat](${'a'.repeat(100,)})'s bowl.`, word: 'the cat', },),
            ],).toEqual([
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
              true,
            ],);
          },
        },),
        it({
          name: 'IS FALSE WHERE CLOSING MARKS STAND BEFORE NO APOSTROPHE THAT A LETTER OR A DIGIT FOLLOWS: a quoted, '
            + 'bracketed or linked word beside a space, a full stop or the end of the text, an apostrophe with a space or '
            + 'a quotation mark after it, a link destination that is not closed, and a stand of one word in another quotation',
          fn: async () => {
            expect([
              cutIn({ text: 'Fill the "cat" bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the “cat” bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the cat\'s bowl.', word: 'the cat\'s', },),
              cutIn({ text: 'Fill the (grey cat) sleeps.', word: 'grey cat', },),
              cutIn({ text: 'Fill the [grey cat] sleeps.', word: 'grey cat', },),
              cutIn({ text: 'Fill [the cat](x) sleeps.', word: 'the cat', },),
              cutIn({ text: 'Say "hi"\' now.', word: 'hi', },),
              cutIn({ text: 'Say “hi”\' now.', word: 'hi', },),
              cutIn({ text: 'Say "hi"\'', word: 'hi', },),
              cutIn({ text: 'Say "hi" \'s now.', word: 'hi', },),
              cutIn({ text: 'Say "hi"\'. Then nap.', word: 'hi', },),
              cutIn({ text: 'He said "I like \'cats\'" today.', word: 'cats', },),
              cutIn({ text: 'Fill [the cat](unclosed\'sbowl', word: 'the cat', },),
              cutIn({ text: 'Fill the "cat" and the "dog"\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Say *\'Tis the season*', word: 'Say *', },),
              cutIn({ text: 'He said "\'Tis" twice.', word: 'He said "', },),
              cutIn({ text: 'He said “\'Tis” twice.', word: 'He said "', },),
              cutIn({ text: 'He called it 「cat」\'s bowl.', word: 'cat', },),
              cutIn({ text: 'He called it 『cat』\'s bowl.', word: 'cat', },),
              cutIn({ text: 'Fill the bowl.\'s cat', word: 'bowl.', },),
            ],).toEqual([
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
          name: 'CUTS A STAND HOWEVER MANY CLOSING MARKS AND HOWEVER LONG A LINK DESTINATION STAND BETWEEN IT AND THE '
            + 'APOSTROPHE: ten thousand closing marks and a destination of a hundred thousand characters, each before `\'s`',
          fn: async () => {
            expect([
              cutIn({ text: `Fill the cat${')'.repeat(9,)}'s bowl.`, word: 'cat', },),
              cutIn({ text: `Fill the cat${')'.repeat(10_000,)}'s bowl.`, word: 'cat', },),
              cutIn({ text: `Fill the cat${'"'.repeat(10_000,)}'s bowl.`, word: 'cat', },),
              // A run of asterisks that closes nothing is ten thousand characters, not emphasis.
              cutIn({ text: `Fill the cat${'*'.repeat(10_000,)}'s bowl.`, word: 'cat', },),
              cutIn({ text: `Fill *the cat${'*'.repeat(1,)}'s bowl.`, word: 'the cat', },),
              cutIn({ text: `Fill [the cat](${'a'.repeat(512,)})'s bowl.`, word: 'the cat', },),
              cutIn({ text: `Fill [the cat](${'a'.repeat(100_000,)})'s bowl.`, word: 'the cat', },),
              cutIn({ text: `Fill [the cat](${'a'.repeat(100_000,)})${')'.repeat(10_000,)}'s bowl.`, word: 'the cat', },),
              cutIn({ text: `Fill the cat${')'.repeat(10_000,)}' bowl.`, word: 'cat', },),
              cutIn({ text: `Fill [the cat](${'a'.repeat(100_000,)}'s bowl.`, word: 'the cat', },),
              cutIn({ text: `Fill [the cat](${'a b '.repeat(25_000,)})'s bowl.`, word: 'the cat', },),
            ],).toEqual([true, true, true, false, true, true, true, true, false, false, false,],);
          },
        },),
        it({
          name: 'IS FALSE FOR A WORD THAT STANDS ALONE: beside a space, a corner bracket, a dash between clauses or '
            + 'between two numbers, a slash or colon with no digit on a side, a hyphen at the end of a word, an apostrophe '
            + 'with nothing joined before it or a space after it, a corner bracket beside a digit, a mark that is no '
            + 'apostrophe, emphasis marks with no joint behind them, a number before a space, and at the two ends of the text',
          fn: async () => {
            expect([
              cutIn({ text: 'the so - called cat', word: 'so', },),
              cutIn({ text: 'he called it『cat』and left', word: 'cat', },),
              cutIn({ text: 'the cat\u{2013}dog truce', word: 'cat', },),
              cutIn({ text: 'the cat\u{2014}dog truce', word: 'cat', },),
              cutIn({ text: 'the cat/dog truce', word: 'cat', },),
              cutIn({ text: 'a **so** called nap', word: 'so', },),
              cutIn({ text: 'cat', word: 'cat', },),
              cutIn({ text: '**cat**', word: 'cat', },),
              cutIn({ text: 'the cats’ bowl', word: 'cats', },),
              cutIn({ text: 'the cats - and dogs nap', word: 'cats', },),
              cutIn({ text: 'the cats \u{2013} and dogs nap', word: 'cats', },),
              cutIn({ text: 'the cats \u{2014} and dogs nap', word: 'cats', },),
              cutIn({ text: 'the cat- nap', word: 'cat', },),
              cutIn({ text: 'see pages 10\u{2013}12 on naps', word: '10', },),
              cutIn({ text: 'see pages 10\u{2014}12 on naps', word: '10', },),
              cutIn({ text: 'the cat/5 nap', word: 'cat', },),
              cutIn({ text: 'the cat:5 nap', word: 'cat', },),
              cutIn({ text: 'the cat: 5 naps', word: 'cat', },),
              cutIn({ text: 'ate 1 / 2 of the fish', word: '1', },),
              cutIn({ text: 'the cat『5』nap', word: 'cat', },),
              cutIn({ text: 'the cat『5』nap', word: '5', },),
              cutIn({ text: 'the \'90s cats', word: '90s', },),
              cutIn({ text: 'give \'em more', word: 'em', },),
              cutIn({ text: '\'Tis the season', word: 'Tis', },),
              cutIn({ text: '’90s cats nap', word: '90s', },),
              cutIn({ text: 'the cats\' 5 naps', word: 'cats', },),
              cutIn({ text: 'she said \'5\' twice', word: '5', },),
              cutIn({ text: 'the cat`s nap', word: 'cat', },),
              cutIn({ text: 'the cat\u{00B4}s nap', word: 'cat', },),
              cutIn({ text: 'the cat\u{2032}s nap', word: 'cat', },),
              cutIn({ text: 'she ate 5 cats nap', word: '5 cats', },),
              cutIn({ text: 'at 10:30 she napped', word: '10:30', },),
              cutIn({ text: 'the Catch-22 nap', word: 'Catch-22', },),
            ],).toEqual([
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
          name: 'READS NO MORE THAN A SMALL MULTIPLE OF THE ENGLISH, however many emphasis marks run together: a stand '
            + 'of one emphasis mark at each position of a run of 4,000 reads each character a bounded number of times',
          fn: async () => {
            /**
             Four thousand emphasis marks of the three kinds, run together.
             */
            const run = '*_~*'.repeat(1_000,);
            /**
             Characters read with the run itself, a mark at each position.
             */
            const runReads = readsWhenProbingEveryPosition({ text: run, },);
            /**
             Characters read for a stand with emphasis to step over; the emphasis is a set of offsets the parse gave, so
             stepping over it reads the folded English no more than a plain stand does, and the case shows the count moves.
             */
            const steppedReads = readsForStand({ text: 'a **so**-called nap', word: 'so', },);
            /**
             Characters read for the same stand with no emphasis to step over.
             */
            const plainReads = readsForStand({ text: 'a so-called nap', word: 'so', },);
            expect({
              runWithinBound: runReads <= (LINEAR_READS_PER_CHARACTER * run.length),
              readsAreCounted: (steppedReads > 0) && (plainReads > 0) && (steppedReads <= (plainReads + 8)),
            },).toEqual({ runWithinBound: true, readsAreCounted: true, },);
          },
        },),
        it({
          name: 'READS NO MORE CHARACTERS FOR EACH POSITION OF A LONGER TEXT THAN OF A SHORTER ONE, however many closing marks '
            + 'or links run together: a stand of one character at each position of a run of straight quotation marks, round '
            + 'or square brackets, a link and its destination repeated, or a link opened and never closed',
          fn: async () => {
            /**
             The runs, each as a function of how many characters it has.
             */
            const runs: readonly (readonly [string, (length: number,) => string])[] = [
              ['quotation marks', (length,) => '"'.repeat(length,)],
              ['round brackets', (length,) => ')'.repeat(length,)],
              ['square brackets', (length,) => ']'.repeat(length,)],
              ['links', (length,) => '](x'.repeat(Math.floor(length / 3,),)],
              ['links never closed', (length,) => ']('.repeat(Math.floor(length / 2,),)],
              ['links with long destinations', (length,) => `](${'a'.repeat(length,)}`],
            ];
            /**
             For each run, how many characters are read for each position at two lengths, and whether the longer text reads
             no more for each position than the shorter, within a quarter (a reading that walked the run would read twice as
             many).
             */
            const grown = runs.map(function growsWith([name, run,],): readonly [string, boolean] {
              const shortText = run(2_000,);
              const longText = run(4_000,);
              return [
                name,
                (readsWhenProbingEveryPosition({ text: longText, },) / longText.length)
                  > (1.25 * (readsWhenProbingEveryPosition({ text: shortText, },) / shortText.length)),
              ];
            },);
            expect(grown,).toEqual(runs.map(function ungrown([name,],): readonly [string, boolean] {
              return [name, false,];
            },),);
          },
        },),
      ],
    },),
    describe({
      name: `${cutsWord.name} by the parse and the neighbour test of the consistency pass`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A LONE ASTERISK AS A CHARACTER, so no stand is cut by an apostrophe past it, and still reads a closed '
            + 'stretch as markup',
          fn: async () => {
            expect([
              cutIn({ text: 'The cat*\'s bowl.', word: 'cat', },),
              cutIn({ text: 'The cat*\'s bowl.', word: 's bowl', },),
              cutIn({ text: 'The so*-called cat.', word: 'so', },),
              cutIn({ text: 'The *cat*\'s bowl.', word: 'cat', },),
              cutIn({ text: 'The **so**-called cat.', word: 'so', },),
              cutIn({ text: 'The *cat*\'s bowl.', word: 's bowl', },),
            ],).toEqual([false, false, false, true, true, true,],);
          },
        },),
        it({
          name: 'ASKS THE SAME QUESTION OF A NEIGHBOUR EVERYWHERE, a Latin letter with its accent or an ASCII digit, so a digit '
            + 'after a closing bracket joins as a letter does and a letter of another script joins nothing',
          fn: async () => {
            expect([
              cutIn({ text: 'The (grey)\'5 tall.', word: 'grey', },),
              cutIn({ text: 'The (grey)\'s bowl.', word: 's bowl', },),
              cutIn({ text: 'The (grey)\'5 tall.', word: '5 tall', },),
              cutIn({ text: 'The αβ\'γδ cat.', word: 'αβ', },),
              cutIn({ text: 'The café\'s cat.', word: 'café', },),
            ],).toEqual([true, true, true, false, true,],);
          },
        },),
        it({
          name: 'COUNTS A WHOLE CHARACTER WHOSE NFKC FORM IS LATIN AS A NEIGHBOUR, so an apostrophe between a letter and a fullwidth '
            + 'letter or a mathematical script letter joins them, and one beside a micro sign (NFKC Greek mu) joins nothing',
          fn: async () => {
            expect([
              cutIn({ text: 'The cat\'\u{FF53} bowl.', word: 'cat', },),
              cutIn({ text: 'The cat\'\u{1D4FE} bowl.', word: 'cat', },),
              cutIn({ text: 'The cat\'\u{00B5} bowl.', word: 'cat', },),
            ],).toEqual([true, true, false,],);
          },
        },),
        it({
          name: 'STEPS OVER A RUN OF LINKS THAT SHOW NOTHING on either side of a joint, so a stand beside an empty link is cut as it is '
            + 'in the text a reader sees: by an apostrophe, a hyphen or a full stop between digits, with emphasis marks beside the run, '
            + 'and by no joint of the closing marks, which are not asked here (n169)',
          fn: async () => {
            /**
             Whether the stand of a word in an English with links is cut out of a longer word, the closing joints left out so that only
             the reading beside the stand answers.

             @param english - English, which folds unit for unit
             @param word - word
             @param which - `first` for the first stand of the word, `last` for the last
             @returns Whether an edge of the stand is a joint
             */
            function cutWith(
              {
                english,
                word,
                which,
              }: {
                readonly english: string;
                readonly word: string;
                readonly which: 'first' | 'last';
              },
            ): boolean {
              /**
               The English in the fold.
               */
              const folded = foldWithOffsets({ text: english, },);
              /**
               Where the stand starts.
               */
              const at = (which === 'first') ? folded.folded.indexOf(word,) : folded.folded.lastIndexOf(word,);
              return cutsWord({
                folded: folded.folded,
                originals: folded.originals,
                joints: new Set(),
                emphasis: emphasisDelimitersOf({ english, },).offsets,
                links: foldedLinksOf({
                  english,
                  folded,
                },),
                at,
                end: at + word.length,
              },);
            }
            /**
             Each English, the word, which stand, and whether that stand is cut.
             */
            const rows: readonly (readonly [string, string, 'first' | 'last', boolean])[] = [
              ['kit[](u)\'kit', 'kit', 'first', true,],
              ['kit[](u)[](v)\'kit', 'kit', 'first', true,],
              ['kit[](u)![](v)[]()\'kit', 'kit', 'first', true,],
              ['kit[](u)-kit', 'kit', 'first', true,],
              ['kit[](u)-kit', 'kit', 'last', true,],
              ['kit-[](u)kit', 'kit', 'first', true,],
              ['kit-[](u)kit', 'kit', 'last', true,],
              ['kit\'[](u)kit', 'kit', 'last', true,],
              ['kit[](u)\'[](v)kit', 'kit', 'last', true,],
              ['1[](u).5', '1', 'first', true,],
              ['1.[](u)5', '5', 'last', true,],
              ['*kit*[](u)\'kit', 'kit', 'first', true,],
              ['*kit[](u)*\'kit', 'kit', 'first', true,],
              ['[](u)kit', 'kit', 'first', false,],
              ['kit[](u)', 'kit', 'first', false,],
              ['a [](u)kit', 'kit', 'last', false,],
              ['kit [](u)\'kit', 'kit', 'last', false,],
              ['kit[ ](u)\'kit', 'kit', 'first', false,],
              // n173: a link with a label shows its label, so `kit[x](u)'kit` reads as `kitx'kit` and the stand is cut.
              ['kit[x](u)\'kit', 'kit', 'first', true,],
              ['kit[](u)kit', 'kit', 'first', true,],
              ['kit[](u)kit', 'kit', 'last', true,],
              ['kit[](u)tens', 'kit', 'first', true,],
              ['kit[](u)tens', 'tens', 'last', true,],
              ['kit[](u)![](v)[]()kit', 'kit', 'first', true,],
              ['1[](u)2', '1', 'first', true,],
              ['kit[](u) kit', 'kit', 'first', false,],
              ['kit [](u)kit', 'kit', 'last', false,],
              ['kit*[](u)kit*', 'kit', 'first', false,],
            ];
            expect(rows.map(function cutOf([english, word, which,],): boolean {
              return cutWith({
                english,
                word,
                which,
              },);
            },),).toEqual(rows.map(function expectedOf(row,): boolean {
              return row.at(-1,) === true;
            },),);
            expect(cutWith({ english: `kit${'[](u)'.repeat(5_000,)}'kit`, word: 'kit', which: 'first', },),).toBe(true,);
            expect(cutWith({ english: `kit${'[](u)'.repeat(5_000,)} kit`, word: 'kit', which: 'first', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
