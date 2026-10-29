import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { createSyntheticClient, } from '@monochromatic-dev/translation-repair-provider';
import {
  alignDocumentSections,
  blockPairingQuestion,
  type PairedSectionRecord,
  pairingQuestionKey,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { prepareBlockPairing, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

const roster = [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,] as const;
const l = tagged({ tag: 'one-parent-preparation-test', },);
const complete = '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}';
type Fixture = {
  readonly input: Parameters<typeof prepareBlockPairing>[0];
  readonly stored: Map<string, PairedSectionRecord>;
  readonly writes: { readonly key: string; readonly serialized: string; }[];
  readonly calls: string[];
};
function fixture({ sourceText = '猫睡了。\n\n它喜欢盒子。', targetText = 'The cat slept.\n\nShe loves boxes.', reply = complete, cache = true, }: {
  readonly sourceText?: string; readonly targetText?: string; readonly reply?: string; readonly cache?: boolean;
} = {},): Fixture {
  const source = parseDocument({ text: sourceText, },);
  const target = parseDocument({ text: targetText, },);
  const [pair,] = alignDocumentSections({ source, target, },).pairs;
  if (pair === undefined) throw new Error('fixture requires one deterministic parent',);
  const stored = new Map<string, PairedSectionRecord>();
  const writes: Fixture['writes'] = [];
  const calls: string[] = [];
  const pairingCache: SliceCache<PairedSectionRecord> = {
    resumed: stored,
    persist: async record => {
      writes.push(record);
      stored.set(record.key, JSON.parse(record.serialized) as PairedSectionRecord);
    },
  };
  const client = createSyntheticClient({
    apiKey: 'fixture-key',
    transport: async exchange => {
      calls.push(exchange.bodyJson ?? '');
      return { status: 200, bodyText: `data: ${JSON.stringify({ choices: [{ index: 0, delta: { content: reply, }, },], },)}\n\ndata: [DONE]\n\n`, };
    },
  },);
  return { stored, writes, calls, input: {
    client, modelIds: roster, pair, pairIndex: 7, targetContainers: target.containers,
    signal: new AbortController().signal, exchangeTimeoutMs: HANG_STOP_MS, l, stageSettings: SCRIPTED_STAGE_SETTINGS,
    ...(cache ? { pairingCache } : {}),
  } };
}

await describe({
  name: prepareBlockPairing.name,
  children: [
    it({
      name: 'caches the cold round and resumes its pairs and findings warm without asking again',
      fn: async () => {
        const f = fixture();
        const cold = await prepareBlockPairing(f.input);
        expect(cold.kind).toBe('paired');
        expect(f.calls).toHaveLength(2);
        expect(f.writes).toHaveLength(1);
        if (cold.kind !== 'paired') throw new Error('expected explicit pairing');
        expect(cold.findings.join(' ')).toContain('section 7 paired 2 of 2');
        const [storedRecord] = f.writes;
        if (storedRecord === undefined) throw new Error('expected persisted parent record');
        const expectedKey = pairingQuestionKey({
          question: 'block',
          sourceTexts: ['猫睡了。', '它喜欢盒子。'],
          targetTexts: ['The cat slept.', 'She loves boxes.'],
          pictureContext: '',
          modelIds: roster,
        });
        expect(storedRecord.key).toBe(expectedKey);
        expect(storedRecord.serialized).toBe(JSON.stringify({ pairs: [{ source: 0, target: 0 }, { source: 1, target: 1 }], findings: cold.findings }));
        const warm = await prepareBlockPairing(f.input);
        expect(f.calls).toHaveLength(2);
        expect(f.writes).toHaveLength(1);
        expect(warm.findings).toEqual(cold.findings);
        if (warm.kind !== 'paired') throw new Error('expected warm explicit pairing');
        expect(warm.pairs).toEqual(cold.pairs);
      },
    },),
    it({
      name: 'MISSES ON A CACHED PAIRING THAT NAMES A BLOCK ITS SECTION LACKS, warns in the refusal\'s words, and buys '
        + 'the section again',
      fn: async () => {
        const fresh = fixture();
        const bought = await prepareBlockPairing(fresh.input);
        const f = fixture();
        const lines: string[] = [];
        const { key } = blockPairingQuestion({ pair: f.input.pair, modelIds: roster });
        f.stored.set(key, { pairs: [{ source: 9, target: 0 }], findings: ['an older round'], });
        const result = await prepareBlockPairing({ ...f.input, l: levelCapturingLogger({ lines }) });
        expect({
          result,
          calls: f.calls.length,
          stored: f.stored.get(key),
          warned: lines.filter(line => line.startsWith('warn ')),
        }).toEqual({
          result: bought,
          calls: 2,
          stored: fresh.stored.get(key),
          warned: [
            `warn [prepareBlockPairing] section 7 misses the block-pairing cache: the record under ${key} does not `
              + 'fit its blocks (pairing names original block 9, and there are 2), so the roster is asked again',
          ],
        });
      },
    },),
    ...[
      {
        damage: 'MOVES BACKWARDS ON THE ORIGINAL SIDE',
        pairs: [{ source: 1, target: 0 }, { source: 0, target: 1 }],
        refusal: 'pairing moves backwards on the original side at position 1',
        texts: {},
      },
      {
        damage: 'REPEATS ONE CORRESPONDENCE',
        pairs: [{ source: 0, target: 0 }, { source: 0, target: 0 }],
        refusal: 'pairing repeats the same correspondence at position 1',
        texts: {},
      },
      {
        damage: 'PAIRS A FOOTNOTE DEFINITION WITH A BODY BLOCK',
        pairs: [{ source: 0, target: 0 }, { source: 1, target: 3 }],
        refusal: 'pairing pairs a footnote definition with a body block at position 1',
        texts: {
          sourceText: 'Cat one.[^1]\n\nCat two.\n\nCat three.\n\n[^1]: Note about cats.\n',
          targetText: 'Chat un.[^1]\n\nChat deux.\n\nChat trois.\n\n[^1]: Note sur les chats.\n',
        },
      },
    ].map(test => it({
      name: `MISSES ON A CACHED PAIRING THAT ${test.damage}, warns in the refusal's words, and buys the section again`,
      fn: async () => {
        const fresh = fixture(test.texts);
        const bought = await prepareBlockPairing(fresh.input);
        const f = fixture(test.texts);
        const lines: string[] = [];
        const { key } = blockPairingQuestion({ pair: f.input.pair, modelIds: roster });
        f.stored.set(key, { pairs: test.pairs, findings: ['an older round'], });
        const result = await prepareBlockPairing({ ...f.input, l: levelCapturingLogger({ lines }) });
        expect({
          result,
          calls: f.calls.length,
          stored: f.stored.get(key),
          warned: lines.filter(line => line.startsWith('warn ')),
        }).toEqual({
          result: bought,
          calls: 2,
          stored: fresh.stored.get(key),
          warned: [
            `warn [prepareBlockPairing] section 7 misses the block-pairing cache: the record under ${key} does not `
              + `fit its blocks (${test.refusal}), so the roster is asked again`,
          ],
        });
      },
    },)),
    it({
      name: 'keeps uncached acquisition usable without introducing a persistence dependency',
      fn: async () => {
        const f = fixture({ cache: false });
        const result = await prepareBlockPairing(f.input);
        expect(result.kind).toBe('paired');
        expect(f.calls).toHaveLength(2);
        expect(f.writes).toHaveLength(0);
      },
    },),
    it({
      name: 'retains a cached empty pairing as fallback with its exact historical findings',
      fn: async () => {
        const f = fixture();
        const initial = await prepareBlockPairing(f.input);
        if (initial.kind !== 'paired') throw new Error('expected cacheable initial pairing');
        f.stored.set(blockPairingQuestion({ pair: f.input.pair, modelIds: roster }).key, { pairs: [], findings: ['historical unresolved parent'], });
        const result = await prepareBlockPairing(f.input);
        expect(result.kind).toBe('fallback');
        expect(result.findings).toEqual(['historical unresolved parent']);
        expect(f.calls).toHaveLength(2);
        expect(f.writes).toHaveLength(1);
      },
    },),
    ...[
      { name: 'no agreed relations', reply: '{"pairs":[]}', writes: 1, },
      { name: 'no usable voice', reply: '{"noPairs":true}', writes: 0, },
      { name: 'heard but invalid block indexes', reply: '{"pairs":[{"source":9,"target":0}]}', writes: 0, },
    ].map(test => it({
      name: `names ${test.name} as queried fallback rather than implicit correspondence`,
      fn: async () => {
        const f = fixture({ reply: test.reply });
        const result = await prepareBlockPairing(f.input);
        expect(result.kind).toBe('fallback');
        expect(result.findings.join(' ')).toContain('fell back to scoring');
        expect(f.writes).toHaveLength(test.writes);
        // ASKED, NOT IMPLICIT: the roster was put the question and settled nothing.
        expect(f.calls).toHaveLength(2);
      },
    },)),
    it({
      name: 'does not cache a paired parent whose archive still has unclaimed blocks',
      fn: async () => {
        const f = fixture({ targetText: 'The cat slept.\n\nShe loves boxes.\n\nAn additional note about quilts.', reply: complete });
        const result = await prepareBlockPairing(f.input);
        expect(result.kind).toBe('paired');
        expect(result.findings.join(' ')).toContain('unresolved, not cached');
        expect(f.writes).toHaveLength(0);
      },
    },),
    it({
      name: 'preserves the conservative non-decline when a source block remains unplaced',
      fn: async () => {
        const f = fixture({ reply: '{"pairs":[{"source":0,"target":0}]}' });
        const result = await prepareBlockPairing(f.input);
        expect(result.kind).toBe('paired');
        expect(result.findings.join(' ')).not.toContain('unresolved, not cached');
        expect(f.writes).toHaveLength(1);
      },
    },),
    it({
      name: 'preserves structural singletons without model calls or cache writes',
      fn: async () => {
        const f = fixture({ sourceText: '## 猫', targetText: '## Cat' });
        const result = await prepareBlockPairing(f.input);
        expect(result).toEqual({ kind: 'implicit', findings: [], definitionPairs: [] });
        expect(f.calls).toHaveLength(0);
        expect(f.writes).toHaveLength(0);
      },
    },),
    ...(['source', 'target'] as const).map(side => it({
      name: `does not buy a question for an empty ${side} side`,
      fn: async () => {
        const f = fixture();
        const pair = { ...f.input.pair, [side]: { ...f.input.pair[side], nodes: [], text: '', startOffset: 0, endOffset: 0 } };
        expect((await prepareBlockPairing({ ...f.input, pair })).kind).toBe('empty');
        expect(f.calls).toHaveLength(0);
        expect(f.writes).toHaveLength(0);
      },
    },)),
    it({
      name: 'preserves crossing definition labels while supplying the existing empty body pairing',
      fn: async () => {
        const f = fixture({ sourceText: '[^1]: 猫。\n\n[^2]: 盒子。', targetText: '[^b]: Box.\n\n[^a]: Cat.' });
        const initial = await prepareBlockPairing(f.input);
        if (initial.kind !== 'paired') throw new Error('expected cacheable definition pairing');
        f.stored.set(blockPairingQuestion({ pair: f.input.pair, modelIds: roster }).key, { pairs: [{ source: 0, target: 1 }, { source: 1, target: 0 }], findings: [] });
        const result = await prepareBlockPairing(f.input);
        if (result.kind !== 'paired') throw new Error('expected explicit definition-separated map entry');
        expect(result.pairs).toEqual([]);
        expect(result.definitionPairs).toEqual([{ sourceLabel: '1', targetLabel: 'a' }, { sourceLabel: '2', targetLabel: 'b' }]);
        expect(result.findings.join(' ')).toContain('footnote definitions cross');
        expect(f.calls).toHaveLength(2);
      },
    },),
    it({
      name: 'keeps after-media transcript ownership on cold and warm parent preparation',
      fn: async () => {
        const pathToken = ['$', '{path}'].join('');
        const marker = `<PhotoScroll photos={['${pathToken}/photos/letter.webp']} />`;
        const sourceText = `About the cat.\n\n${marker}\n\nRemember the cat.`;
        const targetText = `About the cat.\n\n${marker}\n\n<details>\n<summary>Letter</summary>\n> Translated letter.\n</details>\n\nRemember the cat.`;
        const f = fixture({ sourceText, targetText, reply: '{"pairs":[{"source":0,"target":0},{"source":1,"target":1},{"source":2,"target":4}]}' });
        const cold = await prepareBlockPairing(f.input);
        if (cold.kind !== 'paired') throw new Error('expected media-owned pairing');
        expect(cold.pairs.filter(pair => pair.source === 1).map(pair => pair.target)).toEqual([1, 2, 3]);
        const warm = await prepareBlockPairing(f.input);
        if (warm.kind !== 'paired') throw new Error('expected resumed media-owned pairing');
        expect(warm.pairs).toEqual(cold.pairs);
        expect(warm.findings).toEqual(cold.findings);
        expect(f.calls).toHaveLength(2);
        expect(f.writes).toHaveLength(1);
      },
    },),
    it({
      name: 'CLAIMS the after-media transcript on a cached pairing stored without it, so a record an older build wrote '
        + 'before media claims were part of what it kept resumes with the claims and says so, asking nobody',
      fn: async () => {
        const pathToken = ['$', '{path}'].join('');
        const marker = `<PhotoScroll photos={['${pathToken}/photos/letter.webp']} />`;
        const sourceText = `About the cat.\n\n${marker}\n\nRemember the cat.`;
        const targetText = `About the cat.\n\n${marker}\n\n<details>\n<summary>Letter</summary>\n> Translated letter.\n</details>\n\nRemember the cat.`;
        const reply = '{"pairs":[{"source":0,"target":0},{"source":1,"target":1},{"source":2,"target":4}]}';
        const coldRun = fixture({ sourceText, targetText, reply });
        const cold = await prepareBlockPairing(coldRun.input);
        if (cold.kind !== 'paired') throw new Error('expected media-owned pairing');
        const f = fixture({ sourceText, targetText, reply });
        const { key } = blockPairingQuestion({ pair: f.input.pair, pictureContext: '', modelIds: roster });
        const storedFinding = 'block-pairing section 7 paired 3 of 3 original and 3 of 5 translation blocks across 3 relations, from 2 usable voices of 2 heard';
        f.stored.set(key, {
          pairs: [{ source: 0, target: 0 }, { source: 1, target: 1 }, { source: 2, target: 4 }],
          findings: [storedFinding],
        });
        const warm = await prepareBlockPairing(f.input);
        if (warm.kind !== 'paired') throw new Error('expected resumed media-owned pairing');
        expect(warm.pairs).toEqual(cold.pairs);
        expect(warm.findings).toEqual([
          storedFinding,
          'block-pairing media-adjacent source 1 claims target 2',
          'block-pairing media-adjacent source 1 claims target 3',
        ]);
        expect(f.calls).toHaveLength(0);
        expect(f.writes).toHaveLength(0);
      },
    },),
    it({
      name: 'propagates persistence failure instead of returning a completed preparation',
      fn: async () => {
        const f = fixture();
        const failure = new Error('fixture persistence failed');
        let caught: unknown;
        try {
          await prepareBlockPairing({ ...f.input, pairingCache: {
            resumed: f.stored,
            persist: async () => {
              throw failure;
            },
          } });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBe(failure);
      },
    },),
    it({
      name: 'ASKS AGAIN UNDER ANOTHER ROSTER (ledger X13): a pairing one bench settled is not resumed for a bench '
        + 'that never answered it, such as the full bench after a dry reading paired the section on a subset',
      fn: async () => {
        const f = fixture();
        await prepareBlockPairing(f.input);
        const [first] = f.writes;
        if (first === undefined) throw new Error('expected the first roster to persist its pairing');
        await prepareBlockPairing({
          ...f.input,
          modelIds: [SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
        });
        expect({
          calls: f.calls.length,
          writes: f.writes.length,
          keysDiffer: f.writes[1]?.key !== first.key,
        }).toEqual({
          calls: 4,
          writes: 2,
          keysDiffer: true,
        });
      },
    },),
    it({
      name: 'honors cancellation before acquisition without writing cache state',
      fn: async () => {
        const f = fixture();
        const failure = new Error('fixture canceled');
        let caught: unknown;
        try {
          await prepareBlockPairing({ ...f.input, signal: AbortSignal.abort(failure) });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBe(failure);
        expect(f.writes).toHaveLength(0);
      },
    },),
  ],
},);
