/**
 Tests for widening the span of a quote's words over the proven apostrophe beside
 them: a trailing one after the last character, a leading one before the first,
 and no other single mark.

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
  ConsistencyInvariantError,
  isPartOfAWord,
  markReadingsInEnglish,
  proofViewOf,
  provenApostrophesInEnglish,
  readProof,
  withProvenApostrophes,
} from '../dist/final/node/index.mjs';

/**
 What a call throws.

 @param run - call

 @returns Thrown value, none when the call returns

 @example
 ```ts
 const refusal = caught({ run: function fails(): void { throw new Error('no',); }, },);
 ```
 */
function caught({ run, }: { readonly run: () => unknown; },): unknown {
  try {
    run();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: withProvenApostrophes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES A PROVEN TRAILING APOSTROPHE AFTER THE WORDS AND A PROVEN LEADING ONE BEFORE THEM INTO THE SPAN',
          fn: async () => {
            expect([
              withProvenApostrophes({ haystack: 'Fill the parents’ bowl.', start: 5, end: 16, },),
              withProvenApostrophes({ haystack: 'Guard her ’round the cats.', start: 11, end: 20, },),
            ],).toEqual([
              { start: 5, end: 17, },
              { start: 10, end: 20, },
            ],);
          },
        },),
        it({
          name: 'TAKES A PROVEN MARK DIRECTLY AFTER WORDS THAT END IN CLOSING EMPHASIS INTO THE SPAN WHEN A LETTER STANDS BEFORE THE '
            + 'EMPHASIS, since the markup is no part of the word, and leaves one after a closing bracket outside, which is read '
            + 'where it stands, and one after an opener outside',
          fn: async () => {
            expect([
              withProvenApostrophes({ haystack: 'Say Come *here*\' now.', start: 4, end: 15, },),
              withProvenApostrophes({ haystack: 'Say Come (here)\' now.', start: 4, end: 15, },),
              withProvenApostrophes({ haystack: 'Say Come ()\' now.', start: 4, end: 11, },),
              withProvenApostrophes({ haystack: 'Say \'Come *here*\' now.', start: 5, end: 16, },),
            ],).toEqual([
              { start: 4, end: 16, },
              { start: 4, end: 15, },
              { start: 4, end: 11, },
              { start: 5, end: 16, },
            ],);
          },
        },),
        it({
          name: 'LEAVES THE SPAN AT THE WORDS WHERE NO SINGLE MARK STANDS BESIDE THEM, without reading the paragraphs',
          fn: async () => {
            expect(withProvenApostrophes({ haystack: 'Fill the parents bowl.', start: 5, end: 16, },),).toEqual({
              start: 5,
              end: 16,
            },);
          },
        },),
        it({
          name: 'LEAVES THE OPENING CURLY MARK AND A MARK WITH A POSSIBLE PARTNER OUTSIDE THE SPAN',
          fn: async () => {
            expect([
              withProvenApostrophes({ haystack: 'Fill the parents‘ bowl.', start: 5, end: 16, },),
              withProvenApostrophes({ haystack: '‘Mio’ is here.', start: 1, end: 4, },),
            ],).toEqual([
              { start: 5, end: 16, },
              { start: 1, end: 4, },
            ],);
          },
        },),
        it({
          name: 'LEAVES A MARK OUTSIDE THE SPAN WHEN ITS PARTNER STANDS IN ANOTHER PARAGRAPH, the reviewer\'s cases: a closer '
            + 'whose quotation opened before the blank line, an opener whose quotation closes after it, and a closer '
            + 'inside a block quote',
          fn: async () => {
            expect([
              withProvenApostrophes({ haystack: '‘Fill the bowls.\n\nThen feed the kittens’.', start: 18, end: 39, },),
              withProvenApostrophes({
                haystack: 'He said, \'Fill the bowls for the kittens.\n\nThen feed them.\'',
                start: 10,
                end: 40,
              },),
              withProvenApostrophes({ haystack: '> ‘Fill the bowls.\n>\n> Then feed the kittens’.', start: 23, end: 44, },),
            ],).toEqual([
              { start: 18, end: 39, },
              { start: 10, end: 40, },
              { start: 23, end: 44, },
            ],);
          },
        },),
        it({
          name: 'READS THE PAGE THE HAYSTACK BELONGS TO: a closer whose quotation opened in an earlier slice is left outside '
            + 'the span, an opener whose quotation closes in a later slice is too, and the same slices with no partner give '
            + 'the mark to the span at the haystack\'s own offset',
          fn: async () => {
            /**
             The page: an opener in the first slice, the haystack in the second, a closer in the third.
             */
            const opened = '## One\n\n‘Fill the bowls.\n\n## Two\n\nThen feed the kittens’.\n\n## Three\n\nShe said \'hello.\n\n## Four\n\nThe cats’ nap';
            /**
             The same page with no opener before the closing mark.
             */
            const plain = opened.replace('‘Fill', 'Fill',);
            /**
             The haystack, the second slice, and where it starts in the page.
             */
            const haystack = '## Two\n\nThen feed the kittens’.';
            expect([
              withProvenApostrophes({
                haystack,
                start: 18,
                end: 29,
                page: { text: opened, offset: opened.indexOf(haystack,), },
              },),
              withProvenApostrophes({
                haystack,
                start: 18,
                end: 29,
                page: { text: plain, offset: plain.indexOf(haystack,), },
              },),
              withProvenApostrophes({ haystack, start: 18, end: 29, },),
            ],).toEqual([
              { start: 18, end: 29, },
              { start: 18, end: 30, },
              { start: 18, end: 30, },
            ],);
          },
        },),
        it({
          name: 'REFUSES A PAGE THAT CANNOT HOLD THE HAYSTACK AT ITS OFFSET, since the offsets of the proof would address other '
            + 'text',
          fn: async () => {
            const refusal = caught({
              run: function readTooShortPage(): unknown {
                return withProvenApostrophes({
                  haystack: 'Then feed the kittens’.',
                  start: 10,
                  end: 21,
                  page: { text: 'Then', offset: 0, },
                },);
              },
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toContain('does not lie inside its page',);
          },
        },),
        it({
          name: 'LEAVES A MARK OUTSIDE THE SPAN WHEN A QUOTATION CHARACTER OF ANY KIND COULD BE ITS PARTNER: a mark glued '
            + 'between two letters, the low, reversed and angle single marks, and a backtick that opens no code',
          fn: async () => {
            expect([
              withProvenApostrophes({ haystack: 'She said‘fill the kittens’ bowl and left.', start: 14, end: 25, },),
              withProvenApostrophes({ haystack: 'She said ‚fill the kittens’ bowl and left.', start: 15, end: 26, },),
              withProvenApostrophes({ haystack: 'She said ‛fill the kittens’ bowl and left.', start: 15, end: 26, },),
              withProvenApostrophes({ haystack: 'She said ‹fill the kittens’ bowl and left.', start: 15, end: 26, },),
              withProvenApostrophes({ haystack: 'Say ` the parents’ bowl.', start: 6, end: 17, },),
            ],).toEqual([
              { start: 14, end: 25, },
              { start: 15, end: 26, },
              { start: 15, end: 26, },
              { start: 15, end: 26, },
              { start: 6, end: 17, },
            ],);
          },
        },),
        it({
          name: 'SKIPS WHAT THE PAGE DOES NOT READ AS PROSE when it looks for a partner: the strings of a component, code and '
            + 'a comment hold no partner, and an English the grammar cannot read is read as text, where every mark is one',
          fn: async () => {
            expect([
              withProvenApostrophes({
                haystack: '<PhotoScroll photos={[\'a.webp\']} />\n\nThe parents’ bowl.',
                start: 37,
                end: 48,
              },),
              withProvenApostrophes({ haystack: 'Say `‘x` the parents’ bowl.', start: 9, end: 20, },),
              withProvenApostrophes({ haystack: '<!-- ‘x -->\n\nThe parents’ bowl.', start: 13, end: 24, },),
              withProvenApostrophes({ haystack: 'Say \'{the parents’ bowl.', start: 6, end: 17, },),
            ],).toEqual([
              { start: 37, end: 49, },
              { start: 9, end: 21, },
              { start: 13, end: 25, },
              { start: 6, end: 17, },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: provenApostrophesInEnglish.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE MARK AFTER CLOSING MARKUP AS PART OF THE WORD AFTER IT, the reading the joint rule makes of it, so '
            + 'it is no possible opener and a plural possessive later on the page is proven: after emphasis, a straight or '
            + 'curly double quotation mark, a round or square bracket and the destination of a link',
          fn: async () => {
            /**
             The shapes that put a mark after closing markup, each standing in a sentence of its own.
             */
            const shapes = [
              '**cat**\'s',
              '_cat_\'s',
              '"cat"\'s',
              '“cat”\'s',
              '(grey cat)\'s',
              '[the cat]\'s',
              '[the cat](x)\'s',
              '**cat**’s',
            ];
            expect(shapes.map(function provenAfter(shape,): readonly number[] {
              /**
               The English: the shape, then a plural possessive and no other mark.
               */
              const english = `Fill the ${shape} bowl. Then fill the dogs' bowl.`;
              return [...provenApostrophesInEnglish({ english, },).trailing,]
                .map(function offsetFromEnd(at,): number {
                  return english.length - at;
                },);
            },),).toEqual(shapes.map(function onePossessive(): readonly number[] {
              // The mark and the ` bowl.` after it.
              return [7,];
            },),);
          },
        },),
        it({
          name: 'KEEPS A MARK AFTER OPENING MARKUP A POSSIBLE OPENER: `*\'Tis*` and `"\'cat"` open a quotation as far as the '
            + 'page can tell, so the plural possessive after it is not proven',
          fn: async () => {
            expect([
              '*\'Tis* well. Then fill the dogs\' bowl.',
              'Fill the "\'cat" bowl. Then fill the dogs\' bowl.',
            ].map(function provenIn(english,): readonly number[] {
              return [...provenApostrophesInEnglish({ english, },).trailing,];
            },),).toEqual([[], [],],);
          },
        },),
        it({
          name: 'READS A MARK AFTER CLOSING EMPHASIS OR A LINK THAT NO LETTER FOLLOWS AS A TRAILING MARK WHEN A LETTER OR A DIGIT '
            + 'STANDS BEFORE THE MARKUP, so it is proven when nothing pairs with it, and no longer stops the plural possessive '
            + 'after it from being proven: after emphasis, strong emphasis, strikethrough and the destination of a link',
          fn: async () => {
            /**
             The shapes that put a mark after closing markup with a space after it.
             */
            const shapes = [
              '**cats**\'',
              '*cats*’',
              '~~cats~~’',
              '[the cats](x)’',
              '*cats 5*’',
            ];
            expect(shapes.map(function provenAfter(shape,): readonly number[] {
              /**
               The English: the shape, then a plural possessive and no other mark.
               */
              const english = `Fill the ${shape} bowl. Then fill the dogs' bowl.`;
              return [...provenApostrophesInEnglish({ english, },).trailing,];
            },),).toEqual(shapes.map(function bothMarks(shape,): readonly number[] {
              // The mark ends the shape, which starts at the ninth unit; the plural possessive is the last mark.
              /**
               Where the shape ends, one past its last unit.
               */
              const shapeEnd = 9 + shape.length;
              return [shapeEnd - 1, shapeEnd + ' bowl. Then fill the dogs'.length,];
            },),);
          },
        },),
        it({
          name: 'READS A MARK AFTER A CLOSING BRACKET, A DOUBLE QUOTATION MARK OR INLINE CODE WHERE IT STANDS, free, so it is never '
            + 'proven and stops the plural possessive after it from being proven: `"ok"\'` and `(here)’` close a quotation',
          fn: async () => {
            /**
             The shapes that put a mark after a closing character other than emphasis, with a space after it.
             */
            const shapes = [
              '"cats"\'',
              '“cats”’',
              '(grey cats)’',
              '[the cats]\'',
              '`cats`’',
            ];
            expect(shapes.map(function provenAfter(shape,): readonly number[] {
              return [...provenApostrophesInEnglish({ english: `Fill the ${shape} bowl. Then fill the dogs' bowl.`, },).trailing,];
            },),).toEqual(shapes.map(function noneProven(): readonly number[] {
              return [];
            },),);
          },
        },),
        it({
          name: 'READS A MARK BEFORE OPENING MARKUP WHERE IT STANDS, free, so it is never proven as a leading mark: the reading run '
            + 'has no leading mirror',
          fn: async () => {
            expect([
              [...provenApostrophesInEnglish({ english: 'He said \'*tis* well.', },).leading,],
              [...provenApostrophesInEnglish({ english: 'He said \'(tis) well.', },).leading,],
              [...provenApostrophesInEnglish({ english: 'He said ’**cats** now.', },).leading,],
              [...provenApostrophesInEnglish({ english: 'He said \'_cats_ now.', },).leading,],
            ],).toEqual([[], [], [], [],],);
          },
        },),
        it({
          name: 'DOES NOT PROVE THE MARK AFTER CLOSING MARKUP A LEADING APOSTROPHE EITHER, since it is part of the word: the '
            + '`s` after emphasis is attached, a mark that stands neither for nor against a quotation',
          fn: async () => {
            /**
             The English: a possessive after emphasis, and a dropped letter after plain space.
             */
            const english = 'Read *Yorushika*\'s songs, and sing \'round the fire.';
            expect({
              leading: [...provenApostrophesInEnglish({ english, },).leading,],
              trailing: [...provenApostrophesInEnglish({ english, },).trailing,],
            },).toEqual({ leading: [english.indexOf('\'round',),], trailing: [], },);
          },
        },),
        it({
          name: 'READS A MARK AFTER A LONE ASTERISK AS A LEADING MARK, a character before it that closes nothing, and keeps a mark '
            + 'after a closed stretch attached',
          fn: async () => {
            /**
             The leading marks of an English that holds no other mark.
             */
            function leadingIn({ english, }: { readonly english: string; },): readonly number[] {
              return [...provenApostrophesInEnglish({ english, },).leading,];
            }
            expect([
              leadingIn({ english: 'The cat*\'s bowl.', },),
              leadingIn({ english: 'The *cat*\'s bowl.', },),
              leadingIn({ english: 'The ~~cat~~\'s bowl.', },),
            ],).toEqual([[8,], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: proofViewOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A PAGE ONCE WHATEVER HOW OFTEN IT IS ASKED, giving the one view for an equal English, so a page with many '
            + 'places does not parse and prove the whole page for each place',
          fn: async () => {
            /**
             An English, and a second text equal to it that is another string.
             */
            const english = 'Fill the parents’ bowl. Then fill the *cats*’s bowl.';
            const copy = `${english.slice(0, 20,)}${english.slice(20,)}`;
            /**
             The view the first time the English is asked about.
             */
            const first = proofViewOf({ english, },);
            expect([
              proofViewOf({ english, },) === first,
              proofViewOf({ english: copy, },) === first,
              proofViewOf({ english: `${english} More.`, },) === first,
            ],).toEqual([true, true, false,],);
          },
        },),
        it({
          name: 'GIVES THE MARKS THE PROOF READS AS ATTACHED AFTER MARKUP, by their offset in the English as written',
          fn: async () => {
            /**
             The English: a possessive after emphasis.
             */
            const english = 'Fill the *cats*’s bowl and the cat*\'s dish.';
            expect([...proofViewOf({ english, },).joined,],).toEqual([english.indexOf('’s',),],);
          },
        },),
        it({
          name: 'LISTS THE APOSTROPHES THE PAGE WRITES AS PART OF A WORD, once for the page: one between letters, one joined after '
            + 'markup and a proven one at an edge, and not a mark of a quotation',
          fn: async () => {
            /**
             The English: a possessive after emphasis, one between letters, a plural possessive, and a single-quoted word.
             */
            const english = 'Fill the *cats*’s bowl, the dog\'s dish, the parents’ cup and ‘hi’ now.';
            expect(proofViewOf({ english, },).wordApostrophes,).toEqual([
              { at: english.indexOf('’s',), character: '’', },
              { at: english.indexOf('\'s',), character: '\'', },
              { at: english.indexOf('’ cup',), character: '’', },
            ],);
          },
        },),
        it({
          name: 'LISTS A PROVEN MARK PAST CLOSING EMPHASIS AMONG THE APOSTROPHES OF THE PAGE, so the form reader and the proof agree: a '
            + 'plural possessive after emphasis, and no mark before emphasis, which is read where it stands',
          fn: async () => {
            /**
             The English: a plural possessive after emphasis, a plural possessive in plain text and a mark before emphasis.
             */
            const english = 'Fill the *kittens*’ bowl, the pups’ bowl and ’*tis* now.';
            expect(proofViewOf({ english, },).wordApostrophes,).toEqual([
              { at: english.indexOf('’ bowl',), character: '’', },
              { at: english.indexOf('’ bowl and',), character: '’', },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: markReadingsInEnglish.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EVERY APOSTROPHE-LIKE MARK OF AN ENGLISH BY THE PROSE AND THE JOINTS, by its offset in the English as written',
          fn: async () => {
            /**
             The English: a joint after emphasis, an attached mark, a trailing mark, a leading mark and a free mark.
             */
            const english = 'The *cats*’s bowl, don’t, the pups’ bowl, ’tis, and ’ alone.';
            expect([...markReadingsInEnglish({ english, },),],).toEqual([
              [english.indexOf('’s',), 'joint',],
              [english.indexOf('’t',), 'attached',],
              [english.indexOf('’ bowl',), 'trailing',],
              [english.indexOf('’tis',), 'leading',],
              [english.indexOf('’ alone',), 'free',],
            ],);
          },
        },),
        it({
          name: 'READS A MARK PAST CLOSING EMPHASIS BY THE LETTER BEFORE THE MARKUP, trailing, and a mark before opening emphasis '
            + 'where it stands, free, since the reading run has no leading mirror',
          fn: async () => {
            /**
             The English: a mark after emphasis, one before emphasis, one between two emphases and one after a space.
             */
            const english = 'The *kittens*’ bowl, ’*tis* now, the *cat*’*dog* and a * ’ here.';
            expect([...markReadingsInEnglish({ english, },),],).toEqual([
              [english.indexOf('’ bowl',), 'trailing',],
              [english.indexOf('’*tis',), 'free',],
              [english.indexOf('’*dog',), 'trailing',],
              [english.indexOf('’ here',), 'free',],
            ],);
          },
        },),
        it({
          name: 'READS A MARK BESIDE A LONE EMPHASIS CHARACTER AS FREE, since a `*`, `_` or `~` the parse makes no delimiter is a '
            + 'character of the text and no markup to read past, and keeps the mark after real emphasis trailing',
          fn: async () => {
            /**
             How the proof reads each mark of an English, by offset.
             */
            function readingsIn({ english, }: { readonly english: string; },): readonly (readonly [number, string])[] {
              return [...markReadingsInEnglish({ english, },),];
            }
            expect([
              readingsIn({ english: 'He said 猫_cats_’ now.', },),
              readingsIn({ english: 'He said \'_cats_s now.', },),
              readingsIn({ english: 'the kittens*’ bowl and *', },),
              readingsIn({ english: 'the kittens_’ bowl', },),
              readingsIn({ english: 'the *kittens*’ bowl', },),
            ],).toEqual([
              [[15, 'free',],],
              [[8, 'free',],],
              [[12, 'free',],],
              [[12, 'free',],],
              [[13, 'trailing',],],
            ],);
          },
        },),
        it({
          name: 'READS THE MARK AFTER A CLOSING QUOTATION MARK OR BRACKET AND THE MARK BEFORE OPENING MARKUP AS FREE, and the mark '
            + 'after strong emphasis or a link\'s destination as trailing and proven',
          fn: async () => {
            /**
             How the proof reads each mark of an English, by offset.
             */
            function readingsIn({ english, }: { readonly english: string; },): readonly (readonly [number, string])[] {
              return [...markReadingsInEnglish({ english, },),];
            }
            expect([
              readingsIn({ english: 'He said cats are "ok"\' now.', },),
              readingsIn({ english: 'Come (here)’ now', },),
              readingsIn({ english: 'He said ’**cats** now.', },),
              readingsIn({ english: 'He said \'_cats_ now.', },),
              readingsIn({ english: 'the **kittens**’ bowl', },),
              readingsIn({ english: 'the [kittens](http://x.y)’ bowl', },),
            ],).toEqual([
              [[21, 'free',],],
              [[11, 'free',],],
              [[8, 'free',],],
              [[8, 'free',],],
              [[15, 'trailing',],],
              [[25, 'trailing',],],
            ],);
            expect([
              [...provenApostrophesInEnglish({ english: 'the **kittens**’ bowl', },).trailing,],
              [...provenApostrophesInEnglish({ english: 'the [kittens](http://x.y)’ bowl', },).trailing,],
            ],).toEqual([[15,], [25,],],);
          },
        },),
      ],
    },),
    describe({
      name: readProof.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE PROSE, THE PROVEN MARKS, THE JOINED MARKS AND THE READING OF EVERY MARK OF AN ENGLISH FROM ONE PARSE, '
            + 'the code blanked, the mark after emphasis joined, the plural possessive proven',
          fn: async () => {
            /**
             The English: a possessive after emphasis, a plural possessive and a mark inside code.
             */
            const english = 'Fill the *cats*’s bowl, the parents’ cup and `it’s` here.';
            /**
             What the proof read of it.
             */
            const reading = readProof({ english, },);
            expect({
              prose: reading.prose.length,
              inCode: reading.prose.includes('it’s',),
              joined: [...reading.joined,],
              trailing: [...reading.proven.trailing,],
              leading: [...reading.proven.leading,],
              readings: [...reading.readings,],
            },).toEqual({
              prose: english.length,
              inCode: false,
              joined: [english.indexOf('’s bowl',),],
              trailing: [english.indexOf('’ cup',),],
              leading: [],
              readings: [
                [english.indexOf('’s bowl',), 'joint',],
                [english.indexOf('’ cup',), 'trailing',],
              ],
            },);
          },
        },),
        it({
          name: 'READS A LINK THAT SHOWS NOTHING AS THE CUT OF A WORD DOES: the apostrophe of `see [](u)\'kit here` starts a '
            + 'word as the apostrophe of `see \'kit here` does, so it is joined to nothing and reads as the unlinked mark reads',
          fn: async () => {
            /**
             The English with the empty link and the text a reader sees of it.
             */
            const [linked, twin,] = ['see [](u)\'kit here', 'see \'kit here',].map(function proofOf(english,) {
              return readProof({ english, },);
            },);
            expect({
              joined: [...(linked?.joined ?? []),],
              reading: linked?.readings.get(9,),
              twinJoined: [...(twin?.joined ?? []),],
              twinReading: twin?.readings.get(4,),
            },).toEqual({
              joined: [],
              reading: 'leading',
              twinJoined: [],
              twinReading: 'leading',
            },);
          },
        },),
        it({
          name: 'READS A LINK THAT SHOWS A CLOSING MARK AS THE CUT OF A WORD DOES: the apostrophe of `see [\\]](u)\'kit here` is '
            + 'joined to the word after it as the apostrophe of `see ]\'kit here` is',
          fn: async () => {
            /**
             The English with the link of a closing mark and the text a reader sees of it.
             */
            const [linked, twin,] = [String.raw`see [\]](u)'kit here`, 'see ]\'kit here',].map(function proofOf(english,) {
              return readProof({ english, },);
            },);
            expect({
              joined: [...(linked?.joined ?? []),],
              reading: linked?.readings.get(11,),
              twinJoined: [...(twin?.joined ?? []),],
              twinReading: twin?.readings.get(5,),
            },).toEqual({
              joined: [11,],
              reading: 'joint',
              twinJoined: [5,],
              twinReading: 'joint',
            },);
          },
        },),
        it({
          name: 'READS AN ENGLISH WITH NO MARK AS HOLDING NONE, and reads it afresh each time, where the view of the proof '
            + 'remembers an equal English',
          fn: async () => {
            /**
             An English with no apostrophe-like mark.
             */
            const english = 'Fill the bowl.';
            /**
             The first reading.
             */
            const first = readProof({ english, },);
            /**
             The view of the proof asked for twice.
             */
            const viewTwice = [proofViewOf({ english, },), proofViewOf({ english, },),];
            expect({
              marks: first.readings.size,
              joined: first.joined.size,
              proven: first.proven.trailing.size + first.proven.leading.size,
              sameReading: readProof({ english, },) === first,
              sameView: viewTwice[0] === viewTwice[1],
            },).toEqual({
              marks: 0,
              joined: 0,
              proven: 0,
              sameReading: false,
              sameView: true,
            },);
          },
        },),
      ],
    },),
    describe({
      name: isPartOfAWord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR A MARK READ AS ATTACHED OR JOINT WHATEVER IS PROVEN, since a letter or a digit stands on both sides of '
            + 'it or it joins the word after it across markup',
          fn: async () => {
            /**
             Nothing proven.
             */
            const proven = {
              trailing: new Set<number>(),
              leading: new Set<number>(),
            };
            expect([
              isPartOfAWord({ reading: 'attached', at: 3, proven, },),
              isPartOfAWord({ reading: 'joint', at: 3, proven, },),
            ],).toEqual([true, true,],);
          },
        },),
        it({
          name: 'IS TRUE FOR A TRAILING MARK ONLY WHERE IT IS PROVEN TRAILING, and for a leading mark only where it is proven '
            + 'leading, so a proven mark of the other kind at the same offset proves nothing',
          fn: async () => {
            /**
             Offset 3 proven trailing and offset 7 proven leading.
             */
            const proven = {
              trailing: new Set<number>([3,],),
              leading: new Set<number>([7,],),
            };
            expect([
              isPartOfAWord({ reading: 'trailing', at: 3, proven, },),
              isPartOfAWord({ reading: 'trailing', at: 7, proven, },),
              isPartOfAWord({ reading: 'trailing', at: 9, proven, },),
              isPartOfAWord({ reading: 'leading', at: 7, proven, },),
              isPartOfAWord({ reading: 'leading', at: 3, proven, },),
              isPartOfAWord({ reading: 'leading', at: 9, proven, },),
            ],).toEqual([true, false, false, true, false, false,],);
          },
        },),
        it({
          name: 'IS FALSE FOR A FREE MARK EVEN AT AN OFFSET PROVEN IN EITHER KIND, since a mark with no letter beside it is no part of a word',
          fn: async () => {
            /**
             Offset 3 proven in both kinds.
             */
            const proven = {
              trailing: new Set<number>([3,],),
              leading: new Set<number>([3,],),
            };
            expect(isPartOfAWord({ reading: 'free', at: 3, proven, },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
