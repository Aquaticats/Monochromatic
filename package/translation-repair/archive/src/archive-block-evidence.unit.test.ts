/**
 Tests the evidence an archive block's source quote must carry: a quote is an
 anchor only when it holds the minimum number of characters a reader sees and
 stands in the section it is claimed for.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isArchiveSourceQuoteAnchored, } from '../dist/final/node/index.mjs';

await describe({
  name: isArchiveSourceQuoteAnchored.name,
  children: [
    it({
      name: 'COUNTS ONLY THE CHARACTERS A READER SEES toward the minimum, so a quote of two ideographs among '
        + 'zero-width spaces or Hangul fillers anchors nothing though the section holds it whole',
      fn: async () => {
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫\u{200B}在\u{200B}窗边。',
          sourceQuote: '猫\u{200B}在\u{200B}',
        },),).toBe(false,);
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫\u{3164}在\u{3164}窗边。',
          sourceQuote: '猫\u{3164}在\u{3164}',
        },),).toBe(false,);
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫\u{200B}在\u{200B}窗边。',
          sourceQuote: '猫\u{200B}在\u{200B}窗边',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'COUNTS NO SPACE toward the minimum either, since a space shows a reader nothing',
      fn: async () => {
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: 'Tabby naps a b c d.',
          sourceQuote: 'a b c',
        },),).toBe(false,);
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: 'Tabby naps a b c d.',
          sourceQuote: 'a b c d',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'ANCHORS A QUOTE THAT LEAVES OUT THE LINE BREAKS OF A SCREENSHOT TRANSCRIPT (ledger B715): the readers '
        + 'are told to preserve line breaks, so a transcript of a chat picture wraps mid-sentence at the picture\'s '
        + 'edge, and a reviewer quoting the sentence whole quotes the words, which the section holds',
      fn: async () => {
        /** A transcript as a reader returns it: the picture wraps after every eight characters. */
        const wrapped = '猫猫今天在窗边安静\n地睡觉，尾巴轻轻地摇\n来摇去。\n\n晚安';
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: wrapped,
          sourceQuote: '猫猫今天在窗边安静地睡觉，尾巴轻轻地摇来摇去。',
        },),).toBe(true,);
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: `CORROBORATED PICTURE SOURCE SUPPORT cat.webp (4 readers agree)\n${wrapped}`,
          sourceQuote: '地睡觉，尾巴轻轻地摇来摇去。',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'ANCHORS A QUOTE WHOSE SPACES DIFFER FROM THE SECTION\'S (ledger B715), read the way the attestation '
        + 'reads a reference quote: the words are what the anchor asks about',
      fn: async () => {
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫猫  喜欢 Mittens 的 窗边。',
          sourceQuote: '猫猫喜欢Mittens的窗边',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'STILL REFUSES A QUOTE STITCHED ACROSS TWO PICTURES (ledger B715): the sheet asks for one span of one '
        + 'transcript, and the heading and the picture\'s own words between two transcripts keep a stitch from being one',
      fn: async () => {
        /** Two transcripts as the sheet carries them, each with the chrome its picture shows. */
        const twoPictures = [
          'CORROBORATED PICTURE SOURCE SUPPORT first.webp (4 readers agree; the transcript the others carry most)',
          '猫猫今天在窗边安静\n地睡觉。\n回复全部 Mittens...',
          '',
          'CORROBORATED PICTURE SOURCE SUPPORT second.webp (4 readers agree; the transcript the others carry most)',
          '尾巴轻轻地摇来摇去。\n晚安',
        ].join('\n',);
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: twoPictures,
          sourceQuote: '猫猫今天在窗边安静地睡觉。尾巴轻轻地摇来摇去。',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'STILL REFUSES A QUOTE WHOSE WORDS THE SECTION DOES NOT HOLD though whitespace is set aside (ledger B715)',
      fn: async () => {
        expect(isArchiveSourceQuoteAnchored({
          sourceContext: '猫猫今天在窗边安静\n地睡觉。',
          sourceQuote: '猫猫今天在窗边安静地吃鱼',
        },),).toBe(false,);
      },
    },),
  ],
},);
