/**
 Tests for the refrain finders behind the repeat groups: the clauses cut from a
 wording stream and the frames that clauses share. Fixtures are cat-themed
 invention only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  CLAUSE_BOUNDARY,
  clausesOf,
  refrainFramesOf,
  wordingStreamOf,
} from '../dist/final/node/index.mjs';

/**
 Reads the clauses of a page's wording.

 @param text - full page text

 @returns Clauses of its body paragraphs, in page order

 @example
 ```ts
 const clauses = clausesOfPage({ text: '所以这只猫是个胖猫咪吧。\n', },);
 ```
 */
function clausesOfPage({ text, }: { readonly text: string; },) {
  /** Wording stream of the page. */
  const wording = wordingStreamOf({
    text,
    document: parseDocument({ text, },),
  },);

  return clausesOf({
    wording,
    stream: wording.chars,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: clausesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Cuts clauses at punctuation, quotation marks and paragraph ends and marks the one inside a quotation',
          fn: async () => {
            expect(clausesOfPage({ text: '所以这只猫是个胖猫咪吧，短。\n\n「所以这只猫是个瘦猫咪吧」\n', },),).toEqual([
              {
                start: 0,
                end: 11,
                points: Array.from('所以这只猫是个胖猫咪吧'),
                text: '所以这只猫是个胖猫咪吧',
                quoted: false,
              },
              {
                start: 16,
                end: 27,
                points: Array.from('所以这只猫是个瘦猫咪吧'),
                text: '所以这只猫是个瘦猫咪吧',
                quoted: true,
              },
            ],);
          },
        },),

        it({
          name: 'Cuts a clause at each character of the boundary set',
          fn: async () => {
            /** Clauses read from a stream with each boundary character between two six-element stretches. */
            const clauses = Array.from(CLAUSE_BOUNDARY).map((mark,) => {
              /** Two stretches with the mark between them, as one stream. */
              const stream = [...Array.from('猫咪窗台阳盹'), mark, ...Array.from('晒太阳打盹儿'),];

              return clausesOf({
                wording: {
                  chars: stream,
                  offsets: stream.map((_, index,) => index,),
                  depths: stream.map(() => 0,),
                },
                stream,
              },).map((clause,) => `${clause.start}-${clause.end}`,);
            },);

            expect(clauses,).toEqual(Array.from(CLAUSE_BOUNDARY).map(() => ['0-6', '7-13',],),);
          },
        },),

        it({
          name: 'Keeps a clause of six elements and drops one of five',
          fn: async () => {
            expect(clausesOfPage({ text: '猫咪窗台阳，猫咪窗台阳盹。\n', },).map((clause,) => clause.text,),).toEqual(['猫咪窗台阳盹',],);
          },
        },),

        it({
          name: 'Counts elements and not UTF-16 units toward the clause minimum',
          fn: async () => {
            expect(clausesOfPage({ text: '😀😀😀猫咪。\n\n😀猫咪窗台阳。\n', },).map((clause,) => clause.text,),).toEqual(['😀猫咪窗台阳',],);
          },
        },),

        it({
          name: 'Ends the last clause at the end of a stream that holds no closing break',
          fn: async () => {
            /** Six elements and nothing after them. */
            const stream = Array.from('猫咪窗台阳盹');

            expect(clausesOf({
              wording: {
                chars: stream,
                offsets: [0, 1, 2, 3, 4, 5,],
                depths: [0, 0, 0, 0, 0, 0,],
              },
              stream,
            },),).toEqual([
              {
                start: 0,
                end: 6,
                points: stream,
                text: '猫咪窗台阳盹',
                quoted: false,
              },
            ],);
          },
        },),

        it({
          name: 'Returns no clause for an empty stream',
          fn: async () => {
            expect(clausesOf({
              wording: {
                chars: [],
                offsets: [],
                depths: [],
              },
              stream: [],
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: refrainFramesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Groups two clauses that share a prefix and a suffix around a slot of one element',
          fn: async () => {
            /** Two clauses of one frame. */
            const clauses = clausesOfPage({ text: '所以这只猫是个胖猫咪吧。\n\n所以这只猫是个瘦猫咪吧。\n', },);

            expect(refrainFramesOf({ clauses, },),).toEqual([
              {
                prefix: '所以这只猫是个',
                suffix: '猫咪吧',
                prefixLength: 7,
                suffixLength: 3,
                members: clauses,
              },
            ],);
          },
        },),

        it({
          name: 'Returns no frame for two identical clauses',
          fn: async () => {
            expect(refrainFramesOf({
              clauses: clausesOfPage({ text: '所以这只猫是个胖猫咪吧。\n\n所以这只猫是个胖猫咪吧。\n', },),
            },),).toEqual([],);
          },
        },),

        it({
          name: 'Returns no frame for fewer than two clauses',
          fn: async () => {
            expect(refrainFramesOf({
              clauses: clausesOfPage({ text: '所以这只猫是个胖猫咪吧。\n', },),
            },),).toEqual([],);
            expect(refrainFramesOf({ clauses: [], },),).toEqual([],);
          },
        },),

        it({
          name: 'Holds a frame whose shared prefix has four elements and whose shared suffix has three',
          fn: async () => {
            /** Clauses sharing exactly four leading and three trailing elements. */
            const clauses = clausesOfPage({ text: '猫咪窗台胖阳盹儿。\n\n猫咪窗台瘦阳盹儿。\n', },);

            expect(refrainFramesOf({ clauses, },).map((frame,) => [frame.prefix, frame.suffix,],),).toEqual([['猫咪窗台', '阳盹儿',],],);
          },
        },),

        it({
          name: 'Returns no frame for a shared prefix of three elements',
          fn: async () => {
            expect(refrainFramesOf({
              clauses: clausesOfPage({ text: '猫咪窗胖台阳盹儿。\n\n猫咪窗瘦台阳盹儿。\n', },),
            },),).toEqual([],);
          },
        },),

        it({
          name: 'Returns no frame for a shared suffix of two elements',
          fn: async () => {
            expect(refrainFramesOf({
              clauses: clausesOfPage({ text: '猫咪窗台胖阳盹儿。\n\n猫咪窗台瘦呀盹儿。\n', },),
            },),).toEqual([],);
          },
        },),

        it({
          name: 'Caps the shared suffix so the shorter clause keeps a slot of one element',
          fn: async () => {
            /** A clause and the same clause with one element put into its middle, so the whole of the shorter is shared. */
            const clauses = clausesOfPage({ text: '猫咪窗台阳盹儿呀。\n\n猫咪窗台晒阳盹儿呀。\n', },);

            expect(refrainFramesOf({ clauses, },),).toEqual([
              {
                prefix: '猫咪窗台',
                suffix: '盹儿呀',
                prefixLength: 4,
                suffixLength: 3,
                members: clauses,
              },
            ],);
          },
        },),

        it({
          name: 'Holds a slot of eight elements and leaves out a slot of nine',
          fn: async () => {
            /** Clauses whose slots hold one, eight and nine elements. */
            const clauses = clausesOfPage({
              text: `所以这只瘦猫咪吧。\n\n所以这只${'胖'.repeat(8,)}猫咪吧。\n\n所以这只${'胖'.repeat(9,)}猫咪吧。\n`,
            },);

            expect(refrainFramesOf({ clauses, },),).toEqual([
              {
                prefix: '所以这只',
                suffix: '猫咪吧',
                prefixLength: 4,
                suffixLength: 3,
                members: [clauses[0], clauses[1],],
              },
            ],);
          },
        },),

        it({
          name: 'Counts prefix, suffix and slot in code points',
          fn: async () => {
            /** Clauses whose shared prefix ends in an element beyond the first plane and whose slots are two such elements. */
            const clauses = clausesOfPage({ text: '所以这𠀀𠀁猫咪吧。\n\n所以这𠀀𠀂猫咪吧。\n', },);

            expect(refrainFramesOf({ clauses, },),).toEqual([
              {
                prefix: '所以这𠀀',
                suffix: '猫咪吧',
                prefixLength: 4,
                suffixLength: 3,
                members: clauses,
              },
            ],);
          },
        },),

        it({
          name: 'Keeps a clause in every frame it forms, one frame for each prefix length and suffix length that two clauses share',
          fn: async () => {
            /** Three clauses: the first two share seven leading elements, the others share five. */
            const clauses = clausesOfPage({
              text: '所以这只猫是个胖猫咪吧。\n\n所以这只猫是个瘦猫咪吧。\n\n所以这只猫非个瘦猫咪吧。\n',
            },);

            expect(refrainFramesOf({ clauses, },).toSorted((a, b,) => (a.prefixLength - b.prefixLength) || (a.suffixLength - b.suffixLength),),).toEqual([
              {
                prefix: '所以这只猫',
                suffix: '猫咪吧',
                prefixLength: 5,
                suffixLength: 3,
                members: [clauses[0], clauses[2],],
              },
              {
                prefix: '所以这只猫',
                suffix: '个瘦猫咪吧',
                prefixLength: 5,
                suffixLength: 5,
                members: [clauses[1], clauses[2],],
              },
              {
                prefix: '所以这只猫是个',
                suffix: '猫咪吧',
                prefixLength: 7,
                suffixLength: 3,
                members: [clauses[0], clauses[1],],
              },
            ],);
          },
        },),

        it({
          name: 'Holds three clauses that share one frame as the members of one frame, in page order',
          fn: async () => {
            /** Three clauses of one frame. */
            const clauses = clausesOfPage({
              text: '所以这只猫是个胖猫咪吧。\n\n所以这只猫是个瘦猫咪吧。\n\n所以这只猫是个软猫咪吧。\n',
            },);

            expect(refrainFramesOf({ clauses, },).map((frame,) => frame.members,),).toEqual([
              clauses,
            ],);
          },
        },),
      ],
    },),
  ],
},);
