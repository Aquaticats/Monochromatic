import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  alignDocumentSections,
  blockPairingQuestion,
  type ChunkPair,
  pairingQuestionKey,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Roster every question here is asked of, which the key folds in (ledger X13).
 */
const ROSTER = [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,] as const;

/**
 Builds one complete parser-backed parent without transport.

 @param sourceText - original fixture

 @param targetText - incumbent fixture

 @returns Corresponding first parent

 @example
 ```ts
 const pair = parent({ sourceText: '猫。', targetText: 'Cat.', });
 ```
 */
function parent({ sourceText, targetText, }: {
  readonly sourceText: string;
  readonly targetText: string;
},): ChunkPair {
  /** Parser output retaining exact node text. */
  const source = parseDocument({ text: sourceText, },);
  /** Archive parser output under the same grammar. */
  const target = parseDocument({ text: targetText, },);
  /** Parent using production section alignment. */
  const [pair,] = alignDocumentSections({ source, target, },).pairs;
  if (pair === undefined) throw new Error('fixture requires a parent',);
  return pair;
}

await describe({ name: blockPairingQuestion.name, children: [
  it({ name: 'numbers the texts and keys the question on them, its roster and no pictures', fn: async (): Promise<void> => {
    /** Complete source and archive paragraphs. */
    const pair = parent({ sourceText: '猫睡了。\n\n它喜欢盒子。', targetText: 'The cat slept.\n\nShe loves boxes.', },);
    /** Actual shared question. */
    const question = blockPairingQuestion({ pair, modelIds: ROSTER, },);
    expect(question.sourceBlocks,).toEqual([{ index: 0, text: '猫睡了。', }, { index: 1, text: '它喜欢盒子。', },]);
    expect(question.targetBlocks,).toEqual([{ index: 0, text: 'The cat slept.', }, { index: 1, text: 'She loves boxes.', },]);
    expect(question.freeOrder,).toEqual({ source: new Set(), target: new Set(), });
    expect(question.key,).toBe(pairingQuestionKey({
      question: 'block',
      sourceTexts: ['猫睡了。', '它喜欢盒子。',],
      targetTexts: ['The cat slept.', 'She loves boxes.',],
      pictureContext: '',
      modelIds: ROSTER,
    },),);
  }, },),
  it({ name: 'numbers definition exemptions within the current parent on each side', fn: async (): Promise<void> => {
    /** Definitions have different local indexes on the two sides. */
    const pair = parent({ sourceText: '猫[^a]。\n\n[^a]: 盒子。\n\n[^b]: 枕头。', targetText: '[^z]: Pillow.\n\n[^y]: Box.', },);
    /** Definition sets must follow parsed positions, not label spelling. */
    const question = blockPairingQuestion({ pair, modelIds: ROSTER, },);
    expect([...question.freeOrder.source,],).toEqual([1, 2,]);
    expect([...question.freeOrder.target,],).toEqual([0, 1,]);
    expect(question.sourceBlocks.map(block => block.index,),).toEqual([0, 1, 2,]);
    expect(question.targetBlocks.map(block => block.index,),).toEqual([0, 1,]);
  }, },),
  ...(['source', 'target',] as const).map(side => it({ name: `binds changed ${side} text and order`, fn: async (): Promise<void> => {
    /** Initial parent and derived current question. */
    const pair = parent({ sourceText: '猫。\n\n盒子。', targetText: 'Cat.\n\nBox.', },);
    /** Existing identity to compare with positive changes. */
    const original = blockPairingQuestion({ pair, modelIds: ROSTER, },);
    /** Reordered nodes must not retain the old key. */
    const reordered = { ...pair, [side]: { ...pair[side], nodes: pair[side].nodes.toReversed(), }, };
    expect(blockPairingQuestion({ pair: reordered, modelIds: ROSTER, },).key,).not.toBe(original.key,);
    /** Changed node text must reach numbering and identity together. */
    const changed = { ...pair, [side]: { ...pair[side], nodes: pair[side].nodes.map(node => ({ ...node, text: `${node.text}！`, })), }, };
    expect(blockPairingQuestion({ pair: changed, modelIds: ROSTER, },).key,).not.toBe(original.key,);
  }, },)),
  it({ name: 'does not normalize embedded quotes, escapes or line endings', fn: async (): Promise<void> => {
    /** Parser-authorized bytes, not prompt or JSON interpolation. */
    const pair = parent({ sourceText: '“猫” \\[note]。\r\n第二行。', targetText: '“Cat” \\[note].\r\nSecond line.', },);
    /** Question copies each exact parsed node without interpreting its syntax. */
    const question = blockPairingQuestion({ pair, modelIds: ROSTER, },);
    expect(question.sourceBlocks,).toEqual(pair.source.nodes.map((node, index,) => ({ index, text: node.text, })),);
    expect(question.targetBlocks,).toEqual(pair.target.nodes.map((node, index,) => ({ index, text: node.text, })),);
  }, },),
  it({ name: 'represents an empty side without inventing a numbered block or correspondence', fn: async (): Promise<void> => {
    /** Complete original singleton used only to derive a structural empty fixture. */
    const pair = parent({ sourceText: '猫。', targetText: 'Cat.', },);
    /** Empty source is representable without buying a question. */
    const empty = { ...pair, source: { ...pair.source, nodes: [], text: '', }, };
    /** Construction describes input, never an acquired relation. */
    const question = blockPairingQuestion({ pair: empty, modelIds: ROSTER, },);
    expect(question.sourceBlocks,).toEqual([],);
    expect([...question.freeOrder.source,],).toEqual([],);
    expect(question.targetBlocks,).toEqual([{ index: 0, text: 'Cat.', },]);
    expect(Object.keys(question,).toSorted(),).toEqual(['freeOrder', 'key', 'sourceBlocks', 'targetBlocks',]);
  }, },),
], },);
