/**
 Tests the verdict on one reply to the archive block review: whether it is
 evidence, and when it is not, the sentence that says which test refused it and
 the numbers that decided it (ledger B715).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { archiveReplyVerdict, } from '../dist/final/node/index.mjs';

/**
 What the one page the invented original links says.
 */
const REFERENCE_CONTEXT = '- reference 1 https://cats.example/posts/mittens ("Mittens"): Mittens had an older sister '
  + 'who was also a tabby, and she slept by the stove.';

/**
 A block that is apparatus alone.
 */
const APPARATUS_BLOCK = 'Translator: Cat Friend';

/**
 A block of prose, which no apparatus test passes.
 */
const PROSE_BLOCK = 'The cat sleeps quietly by the window.';

await describe({
  name: archiveReplyVerdict.name,
  children: [
    it({
      name: 'STANDS for a revision whatever its quote says, since a revision earns publication through the selector',
      fn: async () => {
        expect(archiveReplyVerdict({
          disposition: 'revise',
          sourceQuote: '',
          sourceContext: '猫在窗边安静地睡觉。',
          referenceContext: '',
          blockText: PROSE_BLOCK,
        },),).toStrictEqual({ kind: 'stands', },);
      },
    },),
    it({
      name: 'STANDS for a quote that is a span of the section, wrapped or not',
      fn: async () => {
        for (const sourceContext of ['猫在窗边安静地睡觉。', '猫在窗边安\n静地睡觉。',]) {
          expect(archiveReplyVerdict({
            disposition: 'source-supported',
            sourceQuote: '窗边安静地睡觉',
            sourceContext,
            referenceContext: '',
            blockText: PROSE_BLOCK,
          },),).toStrictEqual({ kind: 'stands', },);
        }
      },
    },),
    it({
      name: 'STANDS for a quote that is a span of one cited page',
      fn: async () => {
        expect(archiveReplyVerdict({
          disposition: 'source-supported',
          sourceQuote: 'an older sister who was also a tabby',
          sourceContext: '猫在窗边安静地睡觉。',
          referenceContext: REFERENCE_CONTEXT,
          blockText: PROSE_BLOCK,
        },),).toStrictEqual({ kind: 'stands', },);
      },
    },),
    it({
      name: 'DROPS A QUOTE OF TOO FEW SHOWN CHARACTERS and says how many it shows against the minimum',
      fn: async () => {
        expect(archiveReplyVerdict({
          disposition: 'source-supported',
          sourceQuote: ' 猫\u{200B}在 ',
          sourceContext: '猫在窗边安静地睡觉。',
          referenceContext: '',
          blockText: PROSE_BLOCK,
        },),).toStrictEqual({
          kind: 'dropped',
          reason: 'its quote shows 2 characters a reader sees, below the 4 an anchor needs',
        },);
      },
    },),
    it({
      name: 'DROPS A QUOTE THE SECTION DOES NOT HOLD and says how much of its opening the section and the cited pages do',
      fn: async () => {
        expect(archiveReplyVerdict({
          disposition: 'source-supported',
          sourceQuote: '猫在窗边 吃了一条鱼',
          sourceContext: '猫在窗边安静地睡觉。',
          referenceContext: '',
          blockText: PROSE_BLOCK,
        },),).toStrictEqual({
          kind: 'dropped',
          reason: 'its quote runs 9 characters with whitespace set aside and opens "猫在窗边吃了一条鱼"; it is no span '
            + 'of the expected section and its picture transcripts (10 characters, the longest opening of the quote '
            + 'found there 4 of 9) or of any of the 0 cited pages (the longest opening found in one, 0)',
        },);
        expect(archiveReplyVerdict({
          disposition: 'source-supported',
          sourceQuote: 'Mittens had an older brother',
          sourceContext: '猫在窗边安静地睡觉。',
          referenceContext: REFERENCE_CONTEXT,
          blockText: PROSE_BLOCK,
        },),).toStrictEqual({
          kind: 'dropped',
          reason: 'its quote runs 24 characters with whitespace set aside and opens "Mittenshadanolderbro"; it is no '
            + 'span of the expected section and its picture transcripts (10 characters, the longest opening of the '
            + 'quote found there 0 of 24) or of any of the 1 cited page (the longest opening found in one, 17)',
        },);
      },
    },),
    it({
      name: 'DROPS A QUOTE STITCHED FROM TWO PICTURES and says that the sheet holds only the opening that lies in the first',
      fn: async () => {
        /** Two transcripts as the sheet carries them, each with the chrome its picture shows. */
        const sourceContext = [
          'CORROBORATED PICTURE SOURCE SUPPORT first.webp (4 readers agree)',
          '猫猫今天在窗边安静\n地睡觉。\n回复全部 Mittens...',
          'CORROBORATED PICTURE SOURCE SUPPORT second.webp (4 readers agree)',
          '尾巴轻轻地摇来摇去。',
        ].join('\n',);
        /** The verdict on a quote across the boundary. */
        const verdict = archiveReplyVerdict({
          disposition: 'source-supported',
          sourceQuote: '猫猫今天在窗边安静地睡觉。尾巴轻轻地摇来摇去。',
          sourceContext,
          referenceContext: '',
          blockText: PROSE_BLOCK,
        },);
        expect(verdict.kind,).toBe('dropped',);
        expect(JSON.stringify(verdict,),).toContain('the longest opening of the quote found there 13 of 23',);
      },
    },),
    it({
      name: 'STANDS for an editorial-context reply about a block of apparatus alone',
      fn: async () => {
        expect(archiveReplyVerdict({
          disposition: 'editorial-context',
          sourceQuote: '',
          sourceContext: '猫在窗边安静地睡觉。',
          referenceContext: '',
          blockText: APPARATUS_BLOCK,
        },),).toStrictEqual({ kind: 'stands', },);
      },
    },),
    it({
      name: 'DROPS an editorial-context reply about a block of prose and says how long the block is',
      fn: async () => {
        expect(archiveReplyVerdict({
          disposition: 'editorial-context',
          sourceQuote: '',
          sourceContext: '猫在窗边安静地睡觉。',
          referenceContext: '',
          blockText: PROSE_BLOCK,
        },),).toStrictEqual({
          kind: 'dropped',
          reason: 'the block of 37 characters is not wholly apparatus (every line it shows must be a contributor '
            + 'declaration, an apparatus label or pictures alone, or the block closed comments alone)',
        },);
      },
    },),
  ],
},);
