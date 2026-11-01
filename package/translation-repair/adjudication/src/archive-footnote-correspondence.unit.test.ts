import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  footnoteRelabelOf,
  footnoteRelabelOfDefinitions,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';

await describe({
  name: 'complete footnote correspondence evidence',
  children: [
    it({
      name: 'refuses distinct archive labels claiming equivalent original destinations',
      fn: async () => {
        expect(footnoteRelabelOfDefinitions({ pairs: [
          { sourceLabel: 'Note', targetLabel: 'a' },
          { sourceLabel: 'note', targetLabel: 'b' },
        ] }).kind).toBe('ambiguous');
      },
    }),
    it({
      name: 'retains identity definition evidence without emitting an identity rewrite',
      fn: async () => {
        expect(footnoteRelabelOfDefinitions({ pairs: [
          { sourceLabel: '1', targetLabel: '2' },
          { sourceLabel: '3', targetLabel: '3' },
        ] })).toEqual({ kind: 'relabel', map: [{ from: '2', to: '1' }],
          correspondences: [{ from: '2', to: '1' }, { from: '3', to: '3' }], skipped: [] });
      },
    }),
    it({
      name: 'normalizes identity while retaining the supplied raw label spellings',
      fn: async () => {
        expect(footnoteRelabelOfDefinitions({ pairs: [
          { sourceLabel: 'B', targetLabel: 'b' },
          { sourceLabel: 'a', targetLabel: 'C' },
        ] })).toEqual({ kind: 'relabel', map: [{ from: 'C', to: 'a' }],
          correspondences: [{ from: 'b', to: 'B' }, { from: 'C', to: 'a' }], skipped: [] });
      },
    }),
    it({
      name: 'keeps an unchanged positional reading distinct from having no correspondence',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: '猫[^Note]。', targetText: 'Cat[^note].' });
        expect(footnoteRelabelOf(prepared)).toEqual({ kind: 'unchanged',
          correspondences: [{ from: 'note', to: 'Note' }], skipped: [] });
        expect(footnoteRelabelOf({ slices: [], sourceText: '', targetText: '' })).toEqual({ kind: 'unchanged', correspondences: [], skipped: [] });
      },
    }),
    it({
      name: 'refuses contradictory destinations hidden by case differences',
      fn: async () => {
        expect(footnoteRelabelOfDefinitions({ pairs: [
          { sourceLabel: 'x', targetLabel: 'A' },
          { sourceLabel: 'y', targetLabel: 'a' },
        ] }).kind).toBe('ambiguous');
      },
    }),
  ],
});
