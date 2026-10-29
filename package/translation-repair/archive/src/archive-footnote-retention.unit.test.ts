import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { closeFootnoteRelabel, } from '../dist/final/node/index.mjs';

await describe({
  name: 'retained archive footnote labels',
  children: [
    it({
      name: 'rejects empty normalized namespaces before inferring elimination',
      fn: async () => {
        expect(closeFootnoteRelabel({ map: [], archiveLabels: ['1'], originalLabels: [' '] }).kind).toBe('open');
      },
    }),
    it({
      name: 'rejects conflicting normalized archive sources rather than discarding a claim',
      fn: async () => {
        expect(closeFootnoteRelabel({ map: [{ from: 'A', to: 'x' }, { from: 'a', to: 'y' }],
          archiveLabels: ['A', 'z'], originalLabels: ['x', 'y'] }).kind).toBe('open');
      },
    }),
    it({
      name: 'folds repeated equivalent claims without inventing extra correspondence',
      fn: async () => {
        expect(closeFootnoteRelabel({ map: [{ from: 'A', to: 'X' }, { from: 'a', to: 'x' }],
          archiveLabels: ['A'], originalLabels: ['X'] })).toEqual({ kind: 'closed', map: [{ from: 'A', to: 'X' }],
            correspondences: [{ from: 'A', to: 'X' }], eliminated: [], retained: [] });
      },
    }),
    it({
      name: 'reserves original-only labels when selecting an archive retention identifier',
      fn: async () => {
        const correspondences = [{ from: '2', to: '1' }, { from: '3', to: '2' }, { from: '5', to: '4' }];
        expect(closeFootnoteRelabel({ map: correspondences, archiveLabels: ['1', '2', '3', '5'], originalLabels: ['1', '2', '4'] }))
          .toEqual({ kind: 'closed', correspondences, eliminated: [], retained: [{ from: '1', retainedAs: '6' }],
            map: [...correspondences, { from: '1', to: '6' }] });
      },
    }),
    it({
      name: 'displaces a surplus colliding archive label without asserting source correspondence',
      fn: async () => {
        const correspondences = [{ from: '2', to: '1' }, { from: '3', to: '2' }];
        expect(closeFootnoteRelabel({ map: correspondences, archiveLabels: ['1', '2', '3'], originalLabels: ['1', '2'] })).toEqual({
          kind: 'closed',
          correspondences,
          eliminated: [],
          retained: [{ from: '1', retainedAs: '4' }],
          map: [...correspondences, { from: '1', to: '4' }],
        });
      },
    }),
    it({
      name: 'allocates distinct deterministic labels for multiple surplus colliders',
      fn: async () => {
        const correspondences = [{ from: 'a', to: '1' }, { from: 'b', to: '2' }];
        const result = closeFootnoteRelabel({ map: correspondences, archiveLabels: ['1', '2', 'a', 'b'], originalLabels: ['1', '2'] });
        expect(result).toEqual({ kind: 'closed', correspondences, eliminated: [],
          retained: [{ from: '1', retainedAs: '3' }, { from: '2', retainedAs: '4' }],
          map: [...correspondences, { from: '1', to: '3' }, { from: '2', to: '4' }], });
      },
    }),
    it({
      name: 'uses explicit identity correspondence when deciding whether all originals are accounted for',
      fn: async () => {
        const incomplete = [{ from: '2', to: '1' }, { from: '4', to: '3' }];
        const input = { archiveLabels: ['1', '2', '3', '4', '5'], originalLabels: ['1', '3', '5'] };
        expect(closeFootnoteRelabel({ ...input, map: incomplete }).kind).toBe('open');
        const complete = [...incomplete, { from: '5', to: '5' }];
        expect(closeFootnoteRelabel({ ...input, map: complete })).toEqual({ kind: 'closed', correspondences: complete,
          eliminated: [], retained: [{ from: '1', retainedAs: '6' }, { from: '3', retainedAs: '7' }],
          map: [...incomplete, { from: '1', to: '6' }, { from: '3', to: '7' }], });
      },
    }),
    it({
      name: 'keeps existing forced elimination distinct from operational displacement',
      fn: async () => {
        const correspondences = [{ from: '2', to: '1' }, { from: '3', to: '2' }];
        expect(closeFootnoteRelabel({ map: correspondences, archiveLabels: ['1', '2', '3'], originalLabels: ['1', '2', '4'] })).toEqual({
          kind: 'closed', correspondences, eliminated: [{ from: '1', to: '4' }], retained: [],
          map: [...correspondences, { from: '1', to: '4' }],
        });
      },
    }),
    it({
      name: 'recognizes normalization-equivalent identities before allocating a fresh identifier',
      fn: async () => {
        const correspondences = [{ from: 'b', to: 'B' }, { from: 'C', to: 'a' }];
        expect(closeFootnoteRelabel({ map: correspondences, archiveLabels: ['A', 'b', 'C'], originalLabels: ['a', 'B'] })).toEqual({
          kind: 'closed', correspondences, eliminated: [], retained: [{ from: 'A', retainedAs: '1' }],
          map: [{ from: 'C', to: 'a' }, { from: 'A', to: '1' }],
        });
      },
    }),
    ...[
      { name: 'duplicate destinations', map: [{ from: 'a', to: '1' }, { from: 'b', to: '1' }], archiveLabels: ['a', 'b'], originalLabels: ['1'] },
      { name: 'unavailable archive source', map: [{ from: 'missing', to: '1' }], archiveLabels: ['a'], originalLabels: ['1'] },
      { name: 'unavailable original destination', map: [{ from: 'a', to: 'missing' }], archiveLabels: ['a'], originalLabels: ['1'] },
    ].map(test => it({
      name: `refuses ${test.name} rather than constructing a misleading closed map`,
      fn: async () => { expect(closeFootnoteRelabel(test).kind).toBe('open'); },
    })),
  ],
});
