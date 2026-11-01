import { readFile, readdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { translateSliceInput, } from '@monochromatic-dev/translation-repair-translate-lane';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type PairedSectionRecord,
  pairingQuestionKey,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { createSyntheticClient, } from '@monochromatic-dev/translation-repair-provider';
import { assertPipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { preparePassEntry, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

const roster = [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER] as const;
const l = tagged({ tag: 'pass-footnote-lifecycle-test' });
const sourceText = 'Alpha[^10]  \nBeta[^200]  \nGamma.\n\n[^10]: First source note.\n\n[^200]: Second source note.';
const archiveBody = 'Alpha archive[^2]  \nBeta archive[^1]  \nGamma archive.';
const archiveDefinitions = '[^2]: First archive note.\n\n[^1]: Second archive note.';
const crossedDefinitions = '[^1]: Second archive note.\n\n[^2]: First archive note.';
const finalText = 'Alpha archive[^10]  \nBeta archive[^200]  \nGamma archive.\n\n[^10]: First archive note.\n\n[^200]: Second archive note.';
const directPairs = [{ source: 0, target: 0 }, { source: 1, target: 1 }, { source: 2, target: 2 }];
const crossedPairs = [{ source: 0, target: 0 }, { source: 1, target: 2 }, { source: 2, target: 1 }];

function cacheKey(targetText: string): string {
  const originals = parseDocument({ text: sourceText }).nodes;
  const targets = parseDocument({ text: targetText }).nodes;
  // The roster that answers is part of the key (ledger X13), and one encoder builds it (ledger X15).
  return pairingQuestionKey({
    question: 'block',
    sourceTexts: originals.map(node => node.text),
    targetTexts: targets.map(node => node.text),
    pictureContext: '',
    modelIds: roster,
  });
}

await describe({
  name: 'actual pass footnote preparation lifecycle',
  children: [{ protectedOriginal: false, crossed: false }, { protectedOriginal: true, crossed: false }, { protectedOriginal: false, crossed: true }]
    .map(({ protectedOriginal, crossed }) => it({
      name: protectedOriginal ? 'retains the original preparation and cache identity when relabeling is withheld'
        : crossed ? 'reorders after forced elimination while refusing to cache the incomplete initial agreement'
          : 'reprepares changed target text with fresh cache keys and current node metadata, then reuses both warm recipes',
      fn: async () => {
        await using owned = await scratchDir({ prefix: 'footnote-pass-lifecycle-' });
        const entryCacheDir = owned.path;
        // Model I/O uses only the injected transport, and the preparation reads
        // nothing from outside the pipeline (ledger X19).
        const calls: { readonly phase: string; readonly body: string; }[] = [];
        const archiveText = `${archiveBody}\n\n${protectedOriginal ? '<!-- 以下内容原文为英文 -->\n\n' : ''}${crossed ? crossedDefinitions : archiveDefinitions}`;
        const expectedText = protectedOriginal ? archiveText : finalText;
        const client = createSyntheticClient({ apiKey: 'fixture-key', transport: async exchange => {
          const body = exchange.bodyJson ?? '';
          const payload = JSON.parse(body) as { readonly messages: readonly { readonly content: string; }[]; };
          const text = payload.messages.map(message => message.content).join('\n');
          const initial = text.includes('[^2]: First archive note.');
          const changed = text.includes('[^10]: First archive note.');
          expect(initial || changed).toBe(true);
          expect(initial && changed).toBe(false);
          calls.push({ phase: initial ? 'initial' : 'changed', body });
          const content = JSON.stringify({ pairs: initial && crossed ? crossedPairs : directPairs });
          return { status: 200, bodyText: `data: ${JSON.stringify({ choices: [{ index: 0, delta: { content } }] })}\n\ndata: [DONE]\n\n` };
        } });
        const generation = `sha256-tree-v1:${'f'.repeat(64)}`;
        assertPipelineDigest(generation);
        const input = { client, entryId: 'invented-footnote-entry', entryCacheDir, pipelineDigest: generation,
          modelIds: roster, sourceText, targetText: archiveText, signal: new AbortController().signal, exchangeTimeoutMs: HANG_STOP_MS, l,
          outsideReads: NO_OUTSIDE_READS, stageSettings: SCRIPTED_STAGE_SETTINGS };
        const cold = await preparePassEntry(input);
        expect(cold.prepared.targetText).toBe(expectedText);
        expect(calls.map(call => call.phase)).toEqual(protectedOriginal ? ['initial', 'initial'] : ['initial', 'initial', 'changed', 'changed']);
        expect(cold.prepared.unclaimedTargetBlocks).toEqual([]);
        expect(cold.footnoteDefinitionPairs).toEqual(protectedOriginal
          ? [{ sourceLabel: '10', targetLabel: '2' }, { sourceLabel: '200', targetLabel: '1' }]
          : [{ sourceLabel: '10', targetLabel: '10' }, { sourceLabel: '200', targetLabel: '200' }]);
        const currentDocument = parseDocument({ text: expectedText });
        const oldDocument = parseDocument({ text: archiveText });
        expect(currentDocument.footnoteGraph.findings).toEqual([]);
        const targetNodes = cold.prepared.slices.flatMap(slice => slice.target.nodes);
        for (const node of targetNodes) {
          expect(node.text).toBe(expectedText.slice(node.startOffset, node.endOffset));
          expect(node.contentHash).toBe(hashContent({ content: node.text }));
          expect(node).toEqual(currentDocument.nodes.find(expected => expected.id === node.id));
          if (!protectedOriginal)
            expect(node.contentHash).not.toBe(oldDocument.nodes.find(previous => previous.id === node.id)?.contentHash);
        }
        if (protectedOriginal) {
          expect(cold.prepared.archiveOriginalSpans).toHaveLength(1);
          expect(cold.prepared.alignmentFindings.some(finding => finding.includes('whole operation withheld'))).toBe(true);
        } else {
          expect(targetNodes.map(node => node.id)).toEqual(currentDocument.nodes.map(node => node.id));
          expect(currentDocument.documentHash).not.toBe(oldDocument.documentHash);
          const oracle = prepareDocumentPair({ sourceText, targetText: expectedText, frontMatterAuthority: cold.prepared.frontMatterAuthority ?? 'rendered',
            blockPairings: new Map([[0, directPairs]]), sealArchiveOriginal: true });
          expect(cold.prepared.slices).toEqual(oracle.slices);
          expect([...cold.prepared.lineStructuredSliceIndices]).toEqual([...oracle.lineStructuredSliceIndices]);
          for (const slice of cold.prepared.slices) {
            const projected = translateSliceInput({ slice, prepared: cold.prepared });
            expect(projected.archiveText).toBe(slice.target.text);
            expect(projected.stageInput.lineStructured).toBe(cold.prepared.lineStructuredSliceIndices.has(slice.target.sliceIndex));
          }
        }
        const cacheFiles = (await readdir(entryCacheDir)).filter(file => file.startsWith('pairing.') && file.endsWith('.json'));
        const cached = new Map(
          await Promise.all(cacheFiles.map(async file => {
            const envelope = JSON.parse(await readFile(join(entryCacheDir, file), 'utf8')) as { readonly cacheKey: string; readonly record: PairedSectionRecord; };
            expect(file).toBe(`pairing.${envelope.cacheKey}.json`);
            return [envelope.cacheKey, envelope.record] as const;
          })),
        );
        expect((await readFile(join(entryCacheDir, 'pairing-generation.txt'), 'utf8')).trim()).toBe(generation);
        const initialKey = cacheKey(archiveText);
        const changedKey = cacheKey(finalText);
        expect(initialKey).not.toBe(changedKey);
        expect([...cached.keys()].toSorted()).toEqual(protectedOriginal ? [initialKey] : crossed ? [changedKey] : [initialKey, changedKey].toSorted());
        expect(cached.get(initialKey)?.pairs).toEqual(crossed ? undefined : directPairs);
        if (!protectedOriginal)
          expect(cached.get(changedKey)?.pairs).toEqual(directPairs);
        const beforeWarm = calls.length;
        const warm = await preparePassEntry(input);
        expect(calls).toHaveLength(beforeWarm + (crossed ? 2 : 0));
        if (crossed)
          expect(calls.slice(beforeWarm).map(call => call.phase)).toEqual(['initial', 'initial']);
        expect(warm.prepared).toEqual(cold.prepared);
        expect(warm.footnoteDefinitionPairs).toEqual(cold.footnoteDefinitionPairs);
      },
      timeout: HANG_STOP_MS,
    })),
});
