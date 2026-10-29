/**
 Tests for the block-coverage check on carved slices.

 THE FAILURE IT EXISTS FOR is silent: a block that reached no slice leaves the
 document, and no range disagrees with itself, so `assertSpanContiguity` sees
 nothing wrong. Every case here is therefore built by carving blocks AWAY from
 a pair rather than by malforming one, which is what production did.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assertSliceCoverage,
  SliceCoverageError,
  type SliceCoverageFault,
} from '../dist/final/node/index.mjs';

/**
 One block of a document, as the parser emits them.
 */
type Block = {
  readonly id: string;
  readonly zone: 'body';
  readonly kind: 'paragraph';
  readonly text: string;
  readonly startOffset: number;
  readonly endOffset: number;
  readonly contentHash: string;
};

/**
 Builds a document node standing in for one block.

 @param id - block id, unique within its side

 @param at - start offset, with the node one character long

 @returns Node shaped as the parser emits them

 @example
 ```ts
 const node = blockNode({ id: 'block/0', at: 0, },);
 ```
 */
function blockNode(
  {
    id,
    at,
  }: {
    readonly id: string;
    readonly at: number;
  },
): Block {
  return {
    id,
    zone: 'body',
    kind: 'paragraph',
    text: 'The cat naps.',
    startOffset: at,
    endOffset: at + 1,
    contentHash: id,
  };
}

/**
 One side of a chunk, as slicing produces it.
 */
type Side = {
  readonly sliceIndex: number;
  readonly nodes: readonly Block[];
  readonly startOffset: number;
  readonly endOffset: number;
  readonly text: string;
};

/**
 Builds one side of a chunk from a run of blocks.

 @param nodes - blocks this side carries

 @returns Chunk side shaped as slicing produces it

 @example
 ```ts
 const side = chunkSide({ nodes: [FIRST_ORIGINAL,], },);
 ```
 */
function chunkSide({ nodes, }: { readonly nodes: readonly Block[]; },): Side {
  return {
    sliceIndex: 0,
    nodes,
    startOffset: nodes.at(0,)?.startOffset ?? 0,
    endOffset: nodes.at(-1,)?.endOffset ?? 0,
    text: 'The cat naps.',
  };
}

/**
 First original of the pair.
 */
const FIRST_ORIGINAL = blockNode({
  id: 'block/0',
  at: 0,
},);

/**
 Second original of the pair.
 */
const SECOND_ORIGINAL = blockNode({
  id: 'block/1',
  at: 2,
},);

/**
 Third original, the one production lost.
 */
const THIRD_ORIGINAL = blockNode({
  id: 'block/2',
  at: 4,
},);

/**
 First rendering, which the first two originals share.
 */
const FIRST_RENDERING = blockNode({
  id: 'block/0',
  at: 0,
},);

/**
 Second rendering.
 */
const SECOND_RENDERING = blockNode({
  id: 'block/1',
  at: 2,
},);

/**
 Reads the fault a carving's refusal names, after checking it is a coverage
 refusal at all.

 THE FAULT RATHER THAN THE CLASS ALONE, since every check here throws the
 same class: a case asserting the class passes whichever check fired.

 @param carve - call that should refuse

 @returns Fault the refusal carries

 @example
 ```ts
 const fault = coverageFault({ carve: function carve() { assertSliceCoverage({ pair: PAIR, carved: [], },); }, },);
 ```
 */
function coverageFault({ carve, }: { readonly carve: () => void; },): SliceCoverageFault {
  /**
   What the carving threw.
   */
  const refusal = caught(carve,);
  expect(refusal,).toBeInstanceOf(SliceCoverageError,);
  return (refusal as SliceCoverageError).fault;
}

/**
 The pair every carving here was carved from.
 */
const PAIR = {
  source: chunkSide({
    nodes: [
      FIRST_ORIGINAL,
      SECOND_ORIGINAL,
      THIRD_ORIGINAL,
    ],
  },),
  target: chunkSide({
    nodes: [
      FIRST_RENDERING,
      SECOND_RENDERING,
    ],
  },),
};

await describe({
  name: assertSliceCoverage.name,
  children: [
    it({
      name: 'ACCEPTS a carving that places every block exactly once',
      fn: async () => {
        assertSliceCoverage({
          pair: PAIR,
          carved: [
            {
              source: chunkSide({
                nodes: [
                  FIRST_ORIGINAL,
                  SECOND_ORIGINAL,
                ],
              },),
              target: chunkSide({ nodes: [FIRST_RENDERING,], },),
            },
            {
              source: chunkSide({ nodes: [THIRD_ORIGINAL,], },),
              target: chunkSide({ nodes: [SECOND_RENDERING,], },),
            },
          ],
        },);
      },
    },),
    it({
      name: 'REFUSES a carving that leaves an original in no slice at all',
      fn: async () => {
        // THE PRODUCTION FAILURE, reduced: the last original is simply absent
        // from every slice. Nothing else about the carving is wrong, which is
        // why no other check in the pipeline can see it.
        expect(coverageFault({
          carve: function losesTheLastBlock() {
            assertSliceCoverage({
              pair: PAIR,
              carved: [
                {
                  source: chunkSide({
                    nodes: [
                      FIRST_ORIGINAL,
                      SECOND_ORIGINAL,
                    ],
                  },),
                  target: chunkSide({
                    nodes: [
                      FIRST_RENDERING,
                      SECOND_RENDERING,
                    ],
                  },),
                },
              ],
            },);
          },
        },),).toEqual({
          kind: 'placement',
          sliceIndex: 0,
          side: 'source',
          placement: {
            kind: 'missing',
            missing: ['block/2',],
            expected: 3,
          },
        },);
      },
    },),
    it({
      name: 'REFUSES a carving that places one block into two slices',
      fn: async () => {
        // A REPEAT IS NOT MERELY UNTIDY: the grouper measures characters per
        // step, so a block counted twice inflates a run past its budget and
        // cuts the document somewhere it should not.
        expect(coverageFault({
          carve: function repeatsABlock() {
            assertSliceCoverage({
              pair: PAIR,
              carved: [
                {
                  source: chunkSide({
                    nodes: [
                      FIRST_ORIGINAL,
                      SECOND_ORIGINAL,
                    ],
                  },),
                  target: chunkSide({ nodes: [FIRST_RENDERING,], },),
                },
                {
                  source: chunkSide({
                    nodes: [
                      SECOND_ORIGINAL,
                      THIRD_ORIGINAL,
                    ],
                  },),
                  target: chunkSide({ nodes: [SECOND_RENDERING,], },),
                },
              ],
            },);
          },
        },),).toEqual({
          kind: 'placement',
          sliceIndex: 0,
          side: 'source',
          placement: {
            kind: 'repeated',
            repeated: ['block/1',],
          },
        },);
      },
    },),
    it({
      name: 'REFUSES a carving that reorders the blocks it places',
      fn: async () => {
        // OUT OF ORDER means one slice gathers text from two places in the
        // document, which reads as a coherent passage and is not one.
        expect(coverageFault({
          carve: function reordersBlocks() {
            assertSliceCoverage({
              pair: PAIR,
              carved: [
                {
                  source: chunkSide({
                    nodes: [
                      THIRD_ORIGINAL,
                      FIRST_ORIGINAL,
                    ],
                  },),
                  target: chunkSide({ nodes: [FIRST_RENDERING,], },),
                },
                {
                  source: chunkSide({ nodes: [SECOND_ORIGINAL,], },),
                  target: chunkSide({ nodes: [SECOND_RENDERING,], },),
                },
              ],
            },);
          },
        },),).toEqual({
          kind: 'placement',
          sliceIndex: 0,
          side: 'source',
          placement: {
            kind: 'out-of-order',
            placed: [
              'block/2',
              'block/0',
              'block/1',
            ],
          },
        },);
      },
    },),
    it({
      name: 'REFUSES a rendering dropped while the originals are whole',
      fn: async () => {
        // CHECKED PER SIDE. A carving can keep every original and still lose a
        // rendering, and losing one deletes shipped English just as surely.
        expect(coverageFault({
          carve: function losesARendering() {
            assertSliceCoverage({
              pair: PAIR,
              carved: [
                {
                  source: chunkSide({
                    nodes: [
                      FIRST_ORIGINAL,
                      SECOND_ORIGINAL,
                      THIRD_ORIGINAL,
                    ],
                  },),
                  target: chunkSide({ nodes: [FIRST_RENDERING,], },),
                },
              ],
            },);
          },
        },),).toEqual({
          kind: 'placement',
          sliceIndex: 0,
          side: 'target',
          placement: {
            kind: 'missing',
            missing: ['block/1',],
            expected: 2,
          },
        },);
      },
    },),
    it({
      name: 'REFUSES a carving that places translation blocks the pairing declined, and words it with the '
        + 'ids, since two parts of the preparation then disagree about where those blocks went',
      fn: async () => {
        /**
         Refusal of a carving placing both renderings the pairing declined.
         */
        const refusal = caught(function placesDeclined() {
          assertSliceCoverage({
            pair: PAIR,
            carved: [
              {
                source: chunkSide({
                  nodes: [
                    FIRST_ORIGINAL,
                    SECOND_ORIGINAL,
                    THIRD_ORIGINAL,
                  ],
                },),
                target: chunkSide({
                  nodes: [
                    FIRST_RENDERING,
                    SECOND_RENDERING,
                  ],
                },),
              },
            ],
            declined: new Set([
              'block/0',
              'block/1',
            ],),
          },);
        },);
        expect(refusal,).toBeInstanceOf(SliceCoverageError,);
        expect((refusal as SliceCoverageError).fault,).toEqual({
          kind: 'declined-reached',
          sliceIndex: 0,
          contradicted: [
            'block/0',
            'block/1',
          ],
        },);
        expect((refusal as SliceCoverageError).message,)
          .toBe('slicing chunk 0: target 2 declined blocks reached a slice: block/0, block/1',);
      },
    },),
    it({
      name: 'REFUSES a carving that places blocks the archive\'s note sealed, on either side, and names the side, '
        + 'since a sealed block ships as it stands and reaches no slice by the owner\'s rule of 2026-09-08',
      fn: async () => {
        /**
         Carving that places every block of the pair once.
         */
        const whole = [
          {
            source: chunkSide({
              nodes: [
                FIRST_ORIGINAL,
                SECOND_ORIGINAL,
                THIRD_ORIGINAL,
              ],
            },),
            target: chunkSide({
              nodes: [
                FIRST_RENDERING,
                SECOND_RENDERING,
              ],
            },),
          },
        ];

        /**
         Refusal when two sealed originals reach the slice.
         */
        const sourceRefusal = caught(function placesSealedOriginals() {
          assertSliceCoverage({
            pair: PAIR,
            carved: whole,
            sealed: {
              source: new Set([
                'block/0',
                'block/1',
              ],),
              target: new Set<string>(),
            },
          },);
        },);
        expect(sourceRefusal,).toBeInstanceOf(SliceCoverageError,);
        expect((sourceRefusal as SliceCoverageError).message,)
          .toBe('slicing chunk 0: source 2 sealed blocks reached a slice: block/0, block/1',);
        expect(coverageFault({
          carve: function placesSealedRenderings() {
            assertSliceCoverage({
              pair: PAIR,
              carved: whole,
              sealed: {
                source: new Set<string>(),
                target: new Set([
                  'block/0',
                  'block/1',
                ],),
              },
            },);
          },
        },),).toEqual({
          kind: 'sealed-reached',
          sliceIndex: 0,
          side: 'target',
          contradicted: [
            'block/0',
            'block/1',
          ],
        },);
      },
    },),
    it({
      name: 'WORDS its refusal from the fault alone, naming positional block ids and never wording',
      fn: async () => {
        const error = new SliceCoverageError({
          fault: {
            kind: 'placement',
            sliceIndex: 3,
            side: 'source',
            placement: {
              kind: 'missing',
              missing: ['block/6',],
              expected: 7,
            },
          },
        },);
        expect(error.message,).toBe('slicing chunk 3: source 1 of 7 blocks reached no slice: block/6',);
        expect(error.messageNamesOnly,).toBe(true,);
      },
    },),
  ],
},);
