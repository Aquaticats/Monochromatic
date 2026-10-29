import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChunkPair,
  isLineStructured,
} from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type AttestedDetail,
  attestedDetailLines,
} from '@monochromatic-dev/translation-repair-reference';
import { translateSliceInput, } from '../dist/final/node/index.mjs';

function first(prepared: PreparedDocumentPair,): ChunkPair {
  const [slice,] = prepared.slices;
  if (slice === undefined) throw new Error('fixture requires a prepared slice');
  return slice;
}
const pathToken = ['$', '{path}'].join('');
const marker = `<PhotoScroll photos={['${pathToken}/photos/letter.webp']} />`;

await describe({
  name: translateSliceInput.name,
  children: [
    it({
      name: 'preserves ordinary canonical text and omits absent context keys',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '猫睡了。', targetText: 'The cat slept.' });
        const slice = first(prepared);
        const surface = translateSliceInput({ slice, prepared });
        expect(surface).toEqual({ archiveText: slice.target.text, protectedText: '', stageInput: {
          sourceText: slice.source.text, incumbentText: slice.target.text, incumbentKind: 'present', lineStructured: false,
        } });
      },
    },),
    it({
      name: 'holds the existing trailing transcript outside both writing and judging while retaining supplied evidence',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: `猫睡了。\n\n${marker}`, targetText: `The cat slept.\n\n${marker}\n\n> Dear cat, rest well.` });
        expect(prepared.slices).toHaveLength(1);
        const slice = first(prepared);
        const surface = translateSliceInput({ slice, prepared: { ...prepared, identityContext: 'declared name: Cat' },
          neighbouringSourceText: 'Source neighbor', neighbouringIncumbentText: 'Archive neighbor', pictureContext: 'Corroborated picture text' });
        expect(surface.archiveText).toBe(slice.target.text);
        expect(surface.protectedText).toBe('\n\n> Dear cat, rest well.');
        expect(surface.stageInput).toEqual({
          sourceText: slice.source.text, incumbentText: `The cat slept.\n\n${marker}`, incumbentKind: 'present', lineStructured: false,
          identityContext: 'declared name: Cat', neighbouringSourceText: 'Source neighbor', neighbouringIncumbentText: 'Archive neighbor', pictureContext: 'Corroborated picture text',
        });
      },
    },),
    it({
      name: 'does not turn an ordinary trailing English expansion into protected transcript',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: marker, targetText: `${marker}\n\nAn ordinary explanatory sentence.` });
        const slice = first(prepared);
        const surface = translateSliceInput({ slice, prepared });
        expect(surface.protectedText).toBe('');
        expect(surface.stageInput.incumbentText).toBe(slice.target.text);
      },
    },),
    it({
      name: 'JUDGES THE REPAIR LANE\'S TEXT AS THE INCUMBENT where the archive rendering is disputed, so the archive is neither a candidate nor the fallback (class one hundred seven, CuspariaKLSY10 slice 3, 2026-09-24)',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '猫睡了。', targetText: 'The cat slept after swallowing its pills.' });
        const slice = first(prepared);
        const standIn = 'The cat slept.';
        const surface = translateSliceInput({ slice, prepared, archiveStandIn: standIn });
        expect(surface.archiveText).toBe(standIn);
        expect(surface.protectedText).toBe('');
        expect(surface.stageInput.incumbentText).toBe(standIn);
        expect(surface.stageInput.incumbentKind).toBe('present');
      },
    },),
    it({
      name: 'CARRIES THE DISPUTE NOTE into the stage input beside the stand-in, so the translators and judges read why the archive is not shown (class one hundred eight, 2026-09-24)',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '猫睡了。', targetText: 'The cat slept after swallowing its pills.' });
        const slice = first(prepared);
        const surface = translateSliceInput({ slice, prepared, archiveStandIn: 'The cat slept.', archiveDisputeNote: 'ARCHIVE RENDERING DISPUTED: (1) the pills.' });
        expect(surface.stageInput.archiveDisputeNote).toBe('ARCHIVE RENDERING DISPUTED: (1) the pills.');
        expect(translateSliceInput({ slice, prepared }).stageInput.archiveDisputeNote).toBe(undefined);
      },
    },),
    // LEDGER B29: the floor-inputs scan names the translate slice's call to
    // the stage as forwarding this literal, which states the references, the
    // attested lines and the declared names; a mutation batch dropped each and
    // no case noticed.
    it({
      name: 'CARRIES THE CITED REFERENCES, THE ATTESTED LINES AND THE DECLARED NAMES the prepared pair holds into the stage input',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '猫睡了。', targetText: 'The cat slept.' });
        const slice = first(prepared);
        const attested: AttestedDetail = { archiveQuote: 'The cat slept.', reference: 1, referenceQuote: 'the cat slept', voices: 2, heard: 3 };
        const declared = [{ source: '猫', rendering: 'Tabby' }];
        const surface = translateSliceInput({ slice, prepared: { ...prepared, referenceContext: 'Reference 1: the cat slept.',
          attestedDetails: [attested], declaredNamePairs: declared } });
        expect(surface.stageInput.referenceContext).toBe('Reference 1: the cat slept.');
        expect(surface.stageInput.attestedLines).toEqual(attestedDetailLines({ details: [attested] }));
        expect(surface.stageInput.declared).toEqual(declared);
      },
    },),
    it({
      name: 'distinguishes an insertion anchor from an existing blank content span',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '猫睡了。', targetText: 'The cat slept.' });
        const slice = first(prepared);
        const insertion: ChunkPair = { ...slice, target: makeInsertionChunk({ sliceIndex: slice.target.sliceIndex, offset: slice.target.startOffset }) };
        const blank: ChunkPair = { ...slice, target: { ...slice.target, nodes: [], text: '', startOffset: 0, endOffset: 0 } };
        expect(translateSliceInput({ slice: insertion, prepared }).stageInput.incumbentKind).toBe('absent');
        expect(translateSliceInput({ slice: blank, prepared }).stageInput.incumbentKind).toBe('present');
      },
    },),
    it({
      name: 'inherits the actual prepared child flag rather than recomputing it on the smaller source',
      fn: async () => {
        const source = ['猫。', '盒子。', '午后。', '睡了。', '晚安。'];
        const target = ['Cat.', 'Box.', 'Afternoon.', 'Sleep.', 'Good night.'];
        const prepared = prepareDocumentPair({ sourceText: source.join('\n\n'), targetText: target.join('\n\n'), sliceCharBudget: 1,
          blockPairings: new Map([[0, source.map((_, index) => ({ source: index, target: index }))]]) });
        const slice = first(prepared);
        expect(isLineStructured({ text: slice.source.text })).toBe(false);
        expect(prepared.lineStructuredSliceIndices.has(slice.target.sliceIndex)).toBe(true);
        expect(translateSliceInput({ slice, prepared }).stageInput.lineStructured).toBe(true);
      },
    },),
    it({
      name: 'preserves explicitly supplied empty contexts instead of changing their presence',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '猫睡了。', targetText: 'The cat slept.' });
        const surface = translateSliceInput({ slice: first(prepared), prepared: { ...prepared, identityContext: '' },
          neighbouringSourceText: '', neighbouringIncumbentText: '', pictureContext: '' });
        expect(surface.stageInput.identityContext).toBe('');
        expect(surface.stageInput.neighbouringSourceText).toBe('');
        expect(surface.stageInput.neighbouringIncumbentText).toBe('');
        expect(surface.stageInput.pictureContext).toBe('');
      },
    },),
    it({
      name: 'retains metadata syntax without executing a publication policy',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '---\nname: 猫\n---\n\n猫。', targetText: '---\nname: Cat\n---\n\nCat.' });
        const slice = prepared.slices.find(candidate => candidate.syntax === 'front-matter');
        if (slice === undefined) throw new Error('fixture requires metadata-aware preparation');
        expect(translateSliceInput({ slice, prepared }).stageInput.syntax).toBe('front-matter');
      },
    },),
  ],
},);
