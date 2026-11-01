/**
 Tests for deterministic quote location:
 byte-exact hits, punctuation-normalized rescue, ambiguity refusal,
 block-crossing splits, and every failure reason.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { locateQuote, } from '../dist/final/node/index.mjs';

/**
 Invented document carrying a unique phrase, a repeated phrase,
 curly punctuation for the normalized fallback, mirrored curly single
 quotes that collide after normalization, and front matter no block
 node covers.
 */
const DOCUMENT_TEXT =
  '---\nname: 小猫-quote\n---\n\n## 猫的日常\n\n小猫喜欢追蝴蝶，也喜欢晒太阳。\n\n老猫说：“打盹最舒服。”重复的句子。重复的句子。\n\n有人写‘猫’，也有人写’猫‘。\n';

/**
 Parsed document every location test searches.
 */
const DOCUMENT = parseDocument({ text: DOCUMENT_TEXT, },);

/**
 Invented document whose one paragraph soft-wraps, so a quote spanning the
 wrap holds a line break where a critic returns a space.
 */
const WRAPPED = parseDocument({
  text: '## 猫的午睡\n\n小猫在窗台上打盹，\n阳光晒得暖洋洋。\n',
},);

/**
 Invented document repeating one soft-wrapped sentence, so collapsing the
 wrap finds the same quote twice.
 */
const WRAPPED_TWICE = parseDocument({
  text: '## 猫的午睡\n\n小猫趴着睡，\n阳光很暖和。\n\n小猫趴着睡，\n阳光很暖和。\n',
},);

await describe({
  name: locateQuote.name,
  children: [
    //region Byte-exact location

    it({
      name: 'locates a unique byte-exact quote and binds one anchor',
      fn: async () => {
        const located = locateQuote({
          document: DOCUMENT,
          side: 'target',
          quote: '小猫喜欢追蝴蝶',
        },);
        expect(located.located,).toBe(true,);
        if (!located.located)
          throw new Error('unreachable: asserted located',);
        expect(located.anchors,).toHaveLength(1,);
        const [anchor,] = located.anchors;
        expect(anchor?.side,).toBe('target',);
        expect(anchor?.quotedText,).toBe('小猫喜欢追蝴蝶',);
        expect(anchor?.startOffset,).toBe(DOCUMENT.text.indexOf('小猫喜欢追蝴蝶',),);
      },
    },),
    it({
      name: 'refuses a quote occurring twice as ambiguous',
      fn: async () => {
        const located = locateQuote({
          document: DOCUMENT,
          side: 'target',
          quote: '重复的句子。',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'ambiguous-quote (target)',
        },);
      },
    },),

    //endregion Byte-exact location

    //region Normalized fallback

    it({
      name: 'rescues an ASCII-quoted copy of curly punctuation, anchoring canonical bytes',
      fn: async () => {
        const located = locateQuote({
          document: DOCUMENT,
          side: 'target',
          quote: '老猫说："打盹最舒服。"',
        },);
        expect(located.located,).toBe(true,);
        if (!located.located)
          throw new Error('unreachable: asserted located',);
        const [anchor,] = located.anchors;
        expect(anchor?.quotedText,).toBe('老猫说：“打盹最舒服。”',);
      },
    },),
    it({
      name: 'refuses a normalized quote matching two curly variants as ambiguous',
      fn: async () => {
        const located = locateQuote({
          document: DOCUMENT,
          side: 'target',
          quote: "'猫'",
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'ambiguous-quote (target)',
        },);
      },
    },),

    //endregion Normalized fallback

    //region Failure reasons

    it({
      name: 'refuses an empty quote',
      fn: async () => {
        const located = locateQuote({
          document: DOCUMENT,
          side: 'target',
          quote: '',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'empty-quote (target)',
        },);
      },
    },),
    it({
      name: 'refuses a quote that shows a reader nothing as empty, though the document holds it exactly once: an '
        + 'ideographic space, a zero-width space, a Hangul filler, or spaces around invisible characters',
      fn: async () => {
        /**
         Original holding each invisible character once, between words.
         */
        const document = parseDocument({ text: '## 猫的日常\n\n小猫喜欢追蝴蝶，\u{3000}也喜欢\u{200B}晒太阳\u{3164}。\n', },);
        expect([
          '\u{3000}',
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ].map(function locate(quote,) {
          return locateQuote({
            document,
            side: 'source',
            quote,
          },);
        },),).toEqual([
          {
            located: false,
            reason: 'empty-quote (source)',
          },
          {
            located: false,
            reason: 'empty-quote (source)',
          },
          {
            located: false,
            reason: 'empty-quote (source)',
          },
          {
            located: false,
            reason: 'empty-quote (source)',
          },
        ],);
      },
    },),
    it({
      name: 'reports an absent quote as not found',
      fn: async () => {
        const located = locateQuote({
          document: DOCUMENT,
          side: 'source',
          quote: '狗狗喜欢游泳',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'quote-not-found (source) needle=6 chars, 0 Latin tokens',
        },);
      },
    },),
    it({
      name: 'refuses a quote only front matter contains as outside blocks',
      fn: async () => {
        const located = locateQuote({
          document: DOCUMENT,
          side: 'source',
          quote: 'name: 小猫-quote',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'quote-outside-blocks (source)',
        },);
      },
    },),

    //endregion Failure reasons

    //region Soft line breaks

    it({
      name: 'LOCATES a quote the wrap explains, which it used only to diagnose. A quote differing from '
        + 'the document by where a paragraph was wrapped is the document text, and refusing it '
        + 'discarded correct evidence: 45 of 844 stored not-found failures were this, and 10 of 11 on '
        + 'the coverage question, whose quotes are whole sentences',
      fn: async () => {
        const located = locateQuote({
          document: WRAPPED,
          side: 'target',
          quote: '小猫在窗台上打盹， 阳光晒得暖洋洋。',
        },);
        expect(located.located,).toBe(true,);
      },
    },),
    it({
      name: 'refuses a wrap-explained quote that collapses onto TWO occurrences as ambiguous, which is '
        + 'what it is: locating it would pick one of two passages by position alone',
      fn: async () => {
        const located = locateQuote({
          document: WRAPPED_TWICE,
          side: 'target',
          quote: '小猫趴着睡， 阳光很暖和。',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'ambiguous-quote (target)',
        },);
      },
    },),
    it({
      name: 'refuses to join text across a PARAGRAPH BREAK even when the quote carries exactly as many '
        + 'spaces as the break carries line endings, which is what a model does when it joins lines '
        + 'blindly: a run of line breaks is structure and matches nothing but itself',
      fn: async () => {
        /** Two paragraphs, so the break between them is two line endings. */
        const twoParagraphs = 'The cat naps at noon.\n\nShe watches the birds.\n';
        const located = locateQuote({
          document: {
            text: twoParagraphs,
            nodes: parseDocument({ text: twoParagraphs, },).nodes,
          },
          side: 'target',
          // Two spaces, matching the two line endings one for one.
          quote: 'The cat naps at noon.  She watches the birds.',
        },);
        expect(located.located,).toBe(false,);
      },
    },),
    it({
      name: 'refuses a quote that is unique CHARACTER FOR CHARACTER while a wrapped twin sits earlier '
        + 'in the document, since a model rewraps whatever it copies and its whitespace cannot say '
        + 'which of the two it read',
      fn: async () => {
        /** Same sentence twice, wrapped the first time and not the second. */
        const twinText = 'The cat naps on\nits cushion. Later: The cat naps on its cushion.\n';
        const located = locateQuote({
          document: {
            text: twinText,
            nodes: parseDocument({ text: twinText, },).nodes,
          },
          side: 'target',
          quote: 'The cat naps on its cushion',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'ambiguous-quote (target)',
        },);
      },
    },),
    it({
      name: 'refuses the same way when the twin differs only in curly versus straight punctuation, '
        + 'which an archive mixing both makes ordinary',
      fn: async () => {
        /** One sentence with a curly apostrophe, the same one with an ASCII apostrophe. */
        const twinText = 'It was the cat’s cushion. Later it was the cat\'s cushion.\n';
        const located = locateQuote({
          document: {
            text: twinText,
            nodes: parseDocument({ text: twinText, },).nodes,
          },
          side: 'target',
          quote: 'the cat\'s cushion',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'ambiguous-quote (target)',
        },);
      },
    },),
    it({
      name: 'leaves a genuinely absent quote unsuffixed',
      fn: async () => {
        const located = locateQuote({
          document: WRAPPED,
          side: 'target',
          quote: '狗在院子里跑步',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'quote-not-found (target) needle=7 chars, 0 Latin tokens',
        },);
      },
    },),
    it({
      name: 'COUNTS a paragraph-length needle rather than quoting any of it, since a finding travels '
        + 'into logs and artifacts where corpus text does not belong',
      fn: async () => {
        /**
         Absent quote longer than the preview bound, built from one repeated
         character so the truncation point is countable rather than guessed.
         */
        const quote = '狗'.repeat(200,);

        /**
         Failure this miss produced.
         */
        const located = locateQuote({
          document: WRAPPED,
          side: 'target',
          quote,
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'quote-not-found (target) needle=200 chars, 0 Latin tokens',
        },);
      },
    },),
    it({
      name: 'FLATTENS a needle carrying a line break onto one line, so a '
        + 'finding stays a single line whatever the critic quoted',
      fn: async () => {
        /**
         Absent quote carrying its own line break.
         */
        const located = locateQuote({
          document: WRAPPED,
          side: 'target',
          quote: '狗在院子\n里跑步',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'quote-not-found (target) needle=8 chars, 0 Latin tokens',
        },);
      },
    },),

    it({
      name: 'COUNTS an accented word as one Latin token, accent composed or combining, and a digit run as '
        + 'another (ledger B18)',
      fn: async () => {
        // ASCII runs alone counted `Émile` as `mile`, `Château` as two runs,
        // `Cafe` plus a combining acute as `Cafe` then nothing, and `à` not at
        // all.
        const located = locateQuote({
          document: WRAPPED,
          side: 'target',
          quote: 'Émile, Château, Cafe\u{0301} à 9',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'quote-not-found (target) needle=25 chars, 5 Latin tokens',
        },);
      },
    },),

    //endregion Soft line breaks

    //region Block-crossing splits

    it({
      name: 'splits a block-crossing quote into one anchor per touched block',
      fn: async () => {
        /**
         Quote spanning the heading, the inter-block gap, and the
         following paragraph's opening characters.
         */
        const quote = '## 猫的日常\n\n小猫';
        const located = locateQuote({
          document: DOCUMENT,
          side: 'target',
          quote,
        },);
        expect(located.located,).toBe(true,);
        if (!located.located)
          throw new Error('unreachable: asserted located',);
        expect(located.anchors.length,).toBeGreaterThanOrEqual(2,);
        for (const anchor of located.anchors) {
          expect(anchor.quotedText,).toBe(
            DOCUMENT.text.slice(
              anchor.startOffset,
              anchor.endOffset,
            ),
          );
        }
        const [first,] = located.anchors;
        const last = located.anchors.at(-1,);
        expect(first?.startOffset,).toBe(DOCUMENT.text.indexOf(quote,),);
        expect(last?.endOffset,).toBe(DOCUMENT.text.indexOf(quote,) + quote.length,);
      },
    },),
    //endregion Block-crossing splits
    it({
      name: 'LOCATES a quote that differs from the document only in where a paragraph was wrapped, '
        + 'since a model copying a sentence out of a wrapped paragraph writes it on one line and the '
        + 'document holds the same characters with a newline in the middle',
      fn: async () => {
        /** Translation whose paragraph is wrapped mid-sentence. */
        const wrappedText = 'The cat sleeps on the windowsill each morning\nand naps on its cushion at noon.\n';

        /** That document as an anchor target. */
        const document = {
          text: wrappedText,
          nodes: parseDocument({ text: wrappedText, },).nodes,
        };

        /** Same sentence as one line, which is how a model copies it back. */
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'on the windowsill each morning and naps on its cushion',
        },);
        expect(located.located,).toBe(true,);
      },
    },),
    it({
      name: 'still refuses to join across a BLANK LINE, since a paragraph break carries two line '
        + 'endings where a space-joined quote carries one space, so collapsing cannot merge passages '
        + 'the document keeps apart',
      fn: async () => {
        /** Two paragraphs, separated as paragraphs are. */
        const twoParagraphs = 'The cat sleeps on the windowsill.\n\nShe watches the birds outside.\n';

        /** That document as an anchor target. */
        const document = {
          text: twoParagraphs,
          nodes: parseDocument({ text: twoParagraphs, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'on the windowsill. She watches',
        },);
        expect(located.located,).toBe(false,);
      },
    },),

    //region Notes between translators

    // A JUDGE IS SHOWN THE TRANSLATION WITHOUT THE NOTES THE ARCHIVE'S TRANSLATORS LEFT EACH OTHER, and copies a
    // quote out of that. A quote across the place a note was cut from finds nothing in the stored text, which
    // is the miss these cases pin: the quote is found in the text as shown, and the anchors it binds are the
    // stored words on each side of the note, never the note.
    it({
      name: 'LOCATES a quote across a note that ends a line, as two anchors that leave the note out',
      fn: async () => {
        /** Translation whose first line ends with a note between translators. */
        const text = 'The cat likes the sun.{/* 这句不太对劲 */}\nThen it naps on the cushion.\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'The cat likes the sun.\nThen it naps on the cushion.',
          original: '',
        },);
        expect(located.located,).toBe(true,);
        if (!located.located)
          throw new Error('unreachable: asserted located',);
        expect(
          located.anchors.map(function quotedOf(anchor,): string {
            return anchor.quotedText;
          },),
        ).toStrictEqual(['The cat likes the sun.', 'Then it naps on the cushion.',],);
        for (const anchor of located.anchors) {
          expect(anchor.quotedText,).toBe(text.slice(anchor.startOffset, anchor.endOffset,),);
        }
      },
    },),
    it({
      name: 'LOCATES a quote across a note alone on its line, the line itself left out of every anchor',
      fn: async () => {
        /** Translation with a note on a line of its own between two sentences. */
        const text = 'The cat sat by the door.\n{/* 注释 */}\nThe cat slept on the mat.\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'The cat sat by the door.\nThe cat slept on the mat.',
          original: '',
        },);
        expect(located.located,).toBe(true,);
        if (!located.located)
          throw new Error('unreachable: asserted located',);
        expect(
          located.anchors.map(function quotedOf(anchor,): string {
            return anchor.quotedText;
          },),
        ).toStrictEqual(['The cat sat by the door.', 'The cat slept on the mat.',],);
        expect(
          located.anchors.some(function holdsNote(anchor,): boolean {
            return anchor.quotedText.includes('注释',);
          },),
        ).toBe(false,);
      },
    },),
    it({
      name: 'LOCATES a quote across a note that ends a paragraph, one anchor in each block',
      fn: async () => {
        /** Translation whose first paragraph ends with a note and the space around it. */
        const text = 'Trans aid work went on. <!-- 前面这句中文懂什么意思 --> \n\nHer actions helped people.\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'Trans aid work went on.\n\nHer actions helped people.',
          original: '',
        },);
        expect(located.located,).toBe(true,);
        if (!located.located)
          throw new Error('unreachable: asserted located',);
        expect(
          located.anchors.map(function quotedOf(anchor,): string {
            return anchor.quotedText;
          },),
        ).toStrictEqual(['Trans aid work went on.', 'Her actions helped people.',],);
      },
    },),
    it({
      name: 'LOCATES a quote across a note that ends a CRLF line, the carriage return kept with the line break',
      fn: async () => {
        /** Translation with CRLF line ends. */
        const text = 'The cat likes the sun.{/* 注 */}\r\nThen it naps.\r\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'The cat likes the sun.\r\nThen it naps.',
          original: '',
        },);
        expect(located.located,).toBe(true,);
        if (!located.located)
          throw new Error('unreachable: asserted located',);
        expect(
          located.anchors.map(function quotedOf(anchor,): string {
            return anchor.quotedText;
          },),
        ).toStrictEqual(['The cat likes the sun.', 'Then it naps.',],);
      },
    },),
    it({
      name: 'JUDGES AMBIGUITY IN THE TEXT AS SHOWN: a sentence that stands once in the stored text, once more with a '
        + 'note inside it, is two places to the judge who read both',
      fn: async () => {
        /** Translation repeating one sentence, the second time with a note between words. */
        const text = 'The cat sat on the mat. It purred.\n\nThe cat sat on {/* 注 */}the mat. It purred.\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'The cat sat on the mat.',
          original: '',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'ambiguous-quote (target)',
        },);
      },
    },),
    it({
      name: 'LEAVES THE NOTE OUT of the anchors of a quote that stops where the note begins and of one that starts where it ends',
      fn: async () => {
        /** Translation with a note between two sentences of one line. */
        const text = 'The cat likes the sun. {/* 注 */} Then it naps.\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const before = locateQuote({
          document,
          side: 'target',
          quote: 'The cat likes the sun.',
          original: '',
        },);
        const after = locateQuote({
          document,
          side: 'target',
          quote: 'Then it naps.',
          original: '',
        },);
        expect(before.located && (before.anchors.length === 1),).toBe(true,);
        expect(after.located && (after.anchors.length === 1),).toBe(true,);
        if ((!before.located) || (!after.located))
          throw new Error('unreachable: asserted located',);
        expect(before.anchors[0]?.endOffset,).toBe(text.indexOf(' {/*',),);
        expect(after.anchors[0]?.startOffset,).toBe(text.indexOf('Then',),);
      },
    },),
    it({
      name: 'KEEPS A COMMENT THE ORIGINAL ALSO WRITES in the text it searches, so a quote across it finds nothing',
      fn: async () => {
        /** Translation repeating a comment the original carries. */
        const text = 'The cat sat {/* 注 */} on the mat.\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'The cat sat on the mat.',
          original: 'The cat sat {/* 注 */} on the mat.',
        },);
        expect(located.located,).toBe(false,);
      },
    },),
    it({
      name: 'READS THE TEXT AS IT STANDS when no original is given, so a quote across a note finds nothing',
      fn: async () => {
        /** Translation with a note that ends a line. */
        const text = 'The cat likes the sun.{/* 注 */}\nThen it naps.\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'The cat likes the sun.\nThen it naps.',
        },);
        expect(located.located,).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES a quote of front matter as outside blocks even when the text holds a note, the stretches '
        + 'of a cut text binding to nothing between blocks as a whole quote does',
      fn: async () => {
        /** Translation with front matter and a note that ends its paragraph. */
        const text = '---\nname: 小猫-quote\n---\n\nThe cat slept.{/* 注 */}\n';

        /** That text as an anchor target. */
        const document = {
          text,
          nodes: parseDocument({ text, },).nodes,
        };
        const located = locateQuote({
          document,
          side: 'target',
          quote: 'name: 小猫-quote',
          original: '',
        },);
        expect(located,).toEqual({
          located: false,
          reason: 'quote-outside-blocks (target)',
        },);
      },
    },),

    //endregion Notes between translators
  ],
},);
