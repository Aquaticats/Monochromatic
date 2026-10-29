/**
 Tests for the tense candidates a consolidation record keeps for the wording it
 shipped.

 Every case asserts the whole record or the whole group list, because the defect
 this guards against is a finding recorded for wording that did not ship, under
 a label the judges never read, or for an entry that records no death, and each
 of those changes the record rather than one field. Fixtures are cat-themed
 invention; no corpus text appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactConsolidateSlice,
  withShippedTenseCandidates,
} from '../dist/final/node/index.mjs';

/**
 Record of a slice that shipped a wrapped consolidation.

 @param text - wording the slice shipped

 @returns Record whose shipped wording is that text

 @example
 ```ts
 const slice = consolidatedWith({ text: 'The cat napped.', },);
 ```
 */
function consolidatedWith({ text, }: { readonly text: string; },): ArtifactConsolidateSlice {
  return {
    sliceIndex: 0,
    terminal: 'consolidated',
    shipped: {
      kind: 'consolidated',
      text,
    },
    rewrapped: false,
    demoted: false,
    verdicts: [],
    gate: { kind: 'not-asked', },
  };
}

//region Consolidation shipped tense tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: withShippedTenseCandidates.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS the candidates of a shipped consolidation under the label the gate gave that text',
          fn: async () => {
            expect(withShippedTenseCandidates({
              slice: consolidatedWith({ text: 'Mittens naps by the window.', }),
              lifeEnded: true,
              subjectNames: ['Mittens',],
            },),).toEqual({
              ...consolidatedWith({ text: 'Mittens naps by the window.', }),
              tenseCandidates: [
                {
                  label: 'CANDIDATE "consolidated"',
                  found: [
                    {
                      sentence: 'Mittens naps by the window.',
                      matched: 'Mittens naps',
                      rule: 's-ending-verb',
                    },
                  ],
                },
              ],
            },);
          },
        },),

        it({
          name: 'RECORDS the candidates of an incumbent that stood in, under the archive label, since that wording is the archive\'s',
          fn: async () => {
            /**
             Record of a slice whose kept standing was the incumbent.
             */
            const kept: ArtifactConsolidateSlice = {
              ...consolidatedWith({ text: 'She naps by the window.', }),
              terminal: 'gate-kept-standing',
              shipped: {
                kind: 'incumbent',
                text: 'She naps by the window.',
              },
            };
            expect(withShippedTenseCandidates({
              slice: kept,
              lifeEnded: true,
            },),).toEqual({
              ...kept,
              tenseCandidates: [
                {
                  label: 'ARCHIVE RENDERING',
                  found: [
                    {
                      sentence: 'She naps by the window.',
                      matched: 'She naps',
                      rule: 's-ending-verb',
                    },
                  ],
                },
              ],
            },);
          },
        },),

        it({
          name: 'READS an alias the translation declares as the subject of a shipped sentence',
          fn: async () => {
            expect(withShippedTenseCandidates({
              slice: consolidatedWith({ text: 'Whiskers naps by the window.', }),
              lifeEnded: true,
              subjectNames: [
                'Mittens',
                'Whiskers',
              ],
            },).tenseCandidates?.length,).toBe(1,);
          },
        },),

        it({
          name: 'LEAVES THE RECORD AS IT STANDS where the slice ships no wording of its own, since the contest record holds the candidates of the lanes and the archive',
          fn: async () => {
            /**
             Records of slices that ship the archive and the contest's wording.
             */
            const unshipped: readonly ArtifactConsolidateSlice[] = [
              {
                ...consolidatedWith({ text: 'She naps by the window.', }),
                terminal: 'incumbent-only',
                shipped: { kind: 'unchanged', },
              },
              {
                ...consolidatedWith({ text: 'She naps by the window.', }),
                terminal: 'no-standing-text',
                shipped: { kind: 'archive', },
              },
            ];
            for (const slice of unshipped) {
              expect(withShippedTenseCandidates({
                slice,
                lifeEnded: true,
              },),).toBe(slice,);
            }
          },
        },),

        it({
          name: 'LEAVES THE RECORD AS IT STANDS for an entry that records no death, and for wording with no candidate',
          fn: async () => {
            /**
             Record whose wording holds a present-tense sentence.
             */
            const present = consolidatedWith({ text: 'She naps by the window.', });
            /**
             Record whose wording holds none.
             */
            const past = consolidatedWith({ text: 'She napped by the window.', });
            expect(withShippedTenseCandidates({
              slice: present,
              lifeEnded: false,
            },),).toBe(present,);
            expect(withShippedTenseCandidates({
              slice: past,
              lifeEnded: true,
            },),).toBe(past,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Consolidation shipped tense tests
