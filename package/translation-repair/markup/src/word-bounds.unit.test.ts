/**
 Tests the word-bounded match the refusal readers share (ledger B23): each
 Latin edge of a needle at a word boundary, the end left open, digits and
 combining marks continuing a word, Han and punctuation edges needing no
 boundary, overlapping starts, and the empty needle. Then the stricter token
 reading the neutral pronoun floor uses: a word joined to another across an
 address's joiners, joiners trimmed at a token's ends, a handle, and marks
 that join nothing.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  carriesWord,
  tokenStarts,
  wordStarts,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wordStarts.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS EVERY START WHERE BOTH LATIN EDGES STAND APART, and none where a word runs on at either edge',
          fn: async () => {
            expect(wordStarts({
              text: 'cat, cats, bobcat, a cat.',
              needle: 'cat',
              end: 'word',
            },),).toEqual([
              0,
              21,
            ],);
          },
        },),
        it({
          name: 'LEAVES THE END OPEN when asked, so a marker is the start of a word and still not the middle of one',
          fn: async () => {
            expect(wordStarts({
              text: 'cat, cats, bobcat, a cat.',
              needle: 'cat',
              end: 'open',
            },),).toEqual([
              0,
              5,
              21,
            ],);
          },
        },),
        it({
          name: 'READS A DIGIT OR A COMBINING MARK BESIDE THE NEEDLE AS THE WORD RUNNING ON',
          fn: async () => {
            for (const text of [
              'cat9',
              '9cat',
              // U+0301 COMBINING ACUTE ACCENT after the needle's last letter.
              'cat\u{0301}',
              // An accented letter from Latin-1 before the needle.
              '\u{00E9}cat',
            ]) {
              expect(wordStarts({
                text,
                needle: 'cat',
                end: 'word',
              },),).toEqual([],);
            }
          },
        },),
        it({
          name: 'NEEDS NO BOUNDARY AT A HAN EDGE OR A PUNCTUATION EDGE, since Chinese writes no spaces and a mark '
            + 'already stands apart',
          fn: async () => {
            expect(wordStarts({
              text: 'ab\u{732B}\u{732B}cd',
              needle: '\u{732B}\u{732B}',
              end: 'word',
            },),).toEqual([2,],);
            expect(wordStarts({
              text: 'a(cat)b',
              needle: '(cat)',
              end: 'word',
            },),).toEqual([1,],);
            expect(wordStarts({
              text: '\u{732B}cat\u{732B}',
              needle: 'cat',
              end: 'word',
            },),).toEqual([1,],);
          },
        },),
        it({
          name: 'FINDS OVERLAPPING STARTS, stepping one unit past each rather than past the whole needle',
          fn: async () => {
            expect(wordStarts({
              text: 'a-a-a',
              needle: 'a-a',
              end: 'word',
            },),).toEqual([
              0,
              2,
            ],);
          },
        },),
        it({
          name: 'FINDS NOTHING FOR AN EMPTY NEEDLE rather than a start at every offset',
          fn: async () => {
            expect(wordStarts({
              text: 'cat',
              needle: '',
              end: 'word',
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: tokenStarts.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'JOINS A WORD TO ANOTHER ACROSS AN ADDRESS\'S JOINERS, on either side and through a run of them',
          fn: async () => {
            expect(tokenStarts({
              text: 'meta.cat cat-nap @cat_x /home/cat/ e.-/cat cat',
              needle: 'cat',
            },),).toEqual([43,],);
          },
        },),
        it({
          name: 'TRIMS JOINERS AT A TOKEN\'S ENDS, so a slash after han and a closing mark leave the word standing',
          fn: async () => {
            expect(tokenStarts({
              text: '\u{732B}/cat cat? cat. x',
              needle: 'cat',
            },),).toEqual([
              2,
              6,
              11,
            ],);
          },
        },),
        it({
          name: 'READS A WORD AFTER A MENTION MARK AS A HANDLE, and one before it as an address',
          fn: async () => {
            expect(tokenStarts({
              text: '@cat cat@example.invalid',
              needle: 'cat',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES A DASH, AN ELLIPSIS, AN ARROW OR EMPHASIS OUTSIDE THE TOKEN, since none sits inside an address',
          fn: async () => {
            expect(tokenStarts({
              text: 'a\u{2014}cat\u{2026}cat\u{2192}*cat*',
              needle: 'cat',
            },),).toEqual([
              2,
              6,
              11,
            ],);
          },
        },),
        it({
          name: 'READS A DIGIT BESIDE THE NEEDLE AS THE WORD RUNNING ON, and no edge of a Han needle',
          fn: async () => {
            expect(tokenStarts({
              text: 'cat9',
              needle: 'cat',
            },),).toEqual([],);
            expect(tokenStarts({
              text: 'a/\u{732B}',
              needle: '\u{732B}',
            },),).toEqual([2,],);
          },
        },),
        it({
          name: 'FINDS NOTHING FOR AN EMPTY NEEDLE',
          fn: async () => {
            expect(tokenStarts({
              text: 'cat',
              needle: '',
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: carriesWord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS WHETHER ANY START STANDS APART',
          fn: async () => {
            expect(carriesWord({
              text: 'the cat served as an aide',
              needle: 'as an ai',
              end: 'word',
            },),).toBe(false,);
            expect(carriesWord({
              text: 'as an aide, and as an ai.',
              needle: 'as an ai',
              end: 'word',
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
