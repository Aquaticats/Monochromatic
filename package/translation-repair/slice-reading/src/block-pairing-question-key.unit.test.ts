import { createHash, } from 'node:crypto';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { DocumentNode, } from '@monochromatic-dev/translation-repair-document-structure';
import type { ContentChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  blockPairingQuestion,
  blockPairingQuestionKey,
  pairingQuestionKey,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

type KeyCase = {
  readonly name: string;
  readonly source: readonly string[];
  readonly target: readonly string[];
  readonly key: string;
};

// These golden keys were captured from the built implementation before extraction, captured again
// when PAIRING_CACHE_VERSION moved to 3 (2026-09-28, ledger M28), again when the roster that
// answers joined the key under ROSTER (2026-09-28, ledger X13), and again when the material became
// one JSON value (2026-09-28, ledger X15), which parted the two embedded NUL cases.
// Roster every golden key is taken under.
const ROSTER = [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,] as const;

const cases: readonly KeyCase[] = [
  { name: 'both empty', source: [], target: [], key: '1c1d8b977798d5431d258a5ca1c1617dfd6b7296b9a57b89aeb1cc19d3e6413d' },
  { name: 'source empty', source: [], target: ['Cat'], key: '4fb34b35f19f108f4144f98129d171bf0570d5accf9b6ed0c33162b45b634f61' },
  { name: 'target empty', source: ['Cat'], target: [], key: 'fdcac5ba834c90178d0ddb440e49f031a83862e6ca79119b8ccbb59da1d18dec' },
  { name: 'simple', source: ['Cat'], target: ['Chat'], key: '88cfb0fd506b0f0d598116be09c7389d8ac14c4b654600e3ebe19e84ed30f39d' },
  { name: 'reversed sides', source: ['Chat'], target: ['Cat'], key: 'bb2cb34675d9c9958abfeacf2528e40ad31b32b945cb17ec8eed688aea19ae80' },
  { name: 'empty block text', source: [''], target: [''], key: '0ce9ef9cdd71f0b6cbb0c143f45f267792fe99981b1ebc17dc30d3e9936a6737' },
  { name: 'Unicode and astral text', source: ['猫😺'], target: ['chaté'], key: 'eb86c601f04bd48cb87453f9858610a954c06d6beddbb33e17c59c73996ffed7' },
  { name: 'embedded NUL left boundary', source: ['Cat\u0000Dog', 'Owl'], target: ['Chat'], key: '515919e06554e81d0ad264e8754e31385550f0128eddb91113473924eba748b2' },
  { name: 'embedded NUL right boundary', source: ['Cat', 'Dog\u0000Owl'], target: ['Chat'], key: '9c5a5fb20dbcba11cd892a7867f92487cf45b35cf45d803a4c757092eaee993f' },
  { name: 'syntax boundaries', source: ['```\nCat "quote" \\ tail', '</script>\n## Cat'], target: ["Chat 'quoted'\n````"], key: 'cfbdbe542e41fc4d5cc0c840e1be25998792723e5b13ce509e1f908ca9a485d2' },
  { name: 'multiple blocks', source: ['Cat', 'Dog', 'Owl'], target: ['Chat', 'Chien'], key: 'f68291312c7aeaf5ba0a220834709feee11d6721d8bc3a2b4fc6eb8f1234cd9f' },
];

// One pairing question's texts, each side in document order.
type Question = { readonly source: readonly string[]; readonly target: readonly string[] };

// Pairs of different questions the key must tell apart (ledger X15): a NUL-joined layout
// aliased each pair, and UTF-8 folds both lone surrogates into U+FFFD.
const distinctQuestions: readonly { readonly label: string; readonly left: Question; readonly right: Question }[] = [
  {
    label: 'an empty text beside the side boundary',
    left: { source: ['Cat', ''], target: ['Chat'] },
    right: { source: ['Cat'], target: ['', 'Chat'] },
  },
  {
    label: 'a NUL inside a text, across a block boundary',
    left: { source: ['Cat\u0000Dog', 'Owl'], target: ['Chat'] },
    right: { source: ['Cat', 'Dog\u0000Owl'], target: ['Chat'] },
  },
  {
    label: 'two different lone surrogates',
    left: { source: ['\uD800'], target: ['Chat'] },
    right: { source: ['\uDC00'], target: ['Chat'] },
  },
];

// Key of one question under the fixture roster.
function keyOf(question: Question): string {
  return blockPairingQuestionKey({
    sourceBlocks: question.source.map((text, index) => ({ index, text })),
    targetBlocks: question.target.map((text, index) => ({ index, text })),
    modelIds: ROSTER,
  });
}

function chunk(texts: readonly string[]): ContentChunk {
  const nodes: DocumentNode[] = [];
  texts.reduce(function placeNode(
    offset,
    text,
    index,
  ): number {
    nodes.push({
      id: `block/${String(index)}`, kind: 'paragraph', zone: 'body', text,
      startOffset: offset, endOffset: offset + text.length,
      contentHash: createHash('sha256').update(text).digest('hex'),
    });
    return offset + text.length + 1;
  }, 0);
  const text = texts.join('\n');
  return { sliceIndex: 0, nodes, startOffset: 0, endOffset: text.length, text };
}

await describe({ name: '', children: [
  describe({ name: 'picture context in the key (class thirty-four, 2026-09-16)', children: [
    it({
      name: 'changes the key when a picture context is given, and keys an empty one as none',
      fn: async () => {
        const sourceBlocks = [{ index: 0, text: 'Cat' }];
        const targetBlocks = [{ index: 0, text: 'Chat' }];
        const bare = blockPairingQuestionKey({ sourceBlocks, targetBlocks, modelIds: ROSTER });
        expect(bare).toBe('88cfb0fd506b0f0d598116be09c7389d8ac14c4b654600e3ebe19e84ed30f39d');
        const sighted = blockPairingQuestionKey({ sourceBlocks, targetBlocks, pictureContext: 'PICTURE nap.webp', modelIds: ROSTER });
        expect(sighted).not.toBe(bare);
        const pair = { source: chunk(['Cat']), target: chunk(['Chat']) };
        expect(blockPairingQuestion({ pair, pictureContext: 'PICTURE nap.webp', modelIds: ROSTER }).key).toBe(sighted);
        expect(blockPairingQuestion({ pair, pictureContext: '', modelIds: ROSTER }).key).toBe(bare);
      },
    }),
  ] }),
  describe({ name: 'roster in the key (ledger X13, 2026-09-28)', children: [
    it({
      name: 'moves the key for another roster, so a pairing one bench settled is never resumed for another',
      fn: async () => {
        const sourceBlocks = [{ index: 0, text: 'Cat' }];
        const targetBlocks = [{ index: 0, text: 'Chat' }];
        const settled = blockPairingQuestionKey({ sourceBlocks, targetBlocks, modelIds: ROSTER });
        expect(blockPairingQuestionKey({ sourceBlocks, targetBlocks, modelIds: [...ROSTER] })).toBe(settled);
        expect(blockPairingQuestionKey({
          sourceBlocks, targetBlocks, modelIds: [SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_VISION_NO_OPENROUTER],
        })).not.toBe(settled);
      },
    }),
  ] }),
  describe({ name: 'one key per question (ledger X15, 2026-09-28)', children: [
    ...distinctQuestions.map(({ label, left, right }) => it({
      name: `KEYS TWO DIFFERENT QUESTIONS APART: ${label}`,
      fn: async () => {
        expect(keyOf(left)).not.toBe(keyOf(right));
      },
    })),
    it({
      name: 'KEYS A SECTION QUESTION APART FROM A BLOCK QUESTION over the same texts, so neither store can serve the other',
      fn: async () => {
        const material = { sourceTexts: ['Cat'], targetTexts: ['Chat'], pictureContext: '', modelIds: ROSTER };
        expect(pairingQuestionKey({ question: 'section', ...material }))
          .not.toBe(pairingQuestionKey({ question: 'block', ...material }));
        expect(pairingQuestionKey({ question: 'block', ...material }))
          .toBe(keyOf({ source: ['Cat'], target: ['Chat'] }));
      },
    }),
  ] }),
  describe({ name: blockPairingQuestionKey.name, children: cases.map(({ name, source, target, key }) => it({
    name: `keeps the recorded ${name} bytes`,
    fn: async () => {
      const sourceBlocks = source.map((text, index) => ({ index, text }));
      const targetBlocks = target.map((text, index) => ({ index, text }));
      expect(blockPairingQuestionKey({ sourceBlocks, targetBlocks, modelIds: ROSTER })).toBe(key);
      const pair = { source: chunk(source), target: chunk(target) };
      expect(blockPairingQuestion({ pair, modelIds: ROSTER }).key).toBe(key);
    },
  })) }),
] });
