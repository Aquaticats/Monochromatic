/**
 Tests for the checks both lanes run around assembly.

 These began as a defence against the slice CACHE, which was trusted on its
 index alone. Both lanes now refuse a contradictory cached record where they
 accept it, so what remains here is a backstop: for a defect in a stage nobody
 has changed yet, for a future caller of the exported guard, and for the one
 relation no single slice can see, which is whether the returned document is
 the one its own surviving replacements assemble to.

 Fixtures are invented. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertReplacementsChange,
  AssemblyContractError,
  deriveShippedIndices,
  orderedChangeSets,
} from '../dist/final/node/index.mjs';

/**
 Archive wording of the one slice these cases use.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 One prepared slice pair carrying that wording.
 */
const CAT_SLICES = [{
  source: {
    sliceIndex: 0,
    nodes: [],
    startOffset: 0,
    endOffset: 3,
    text: 'source of the nap',
  },
  target: {
    sliceIndex: 0,
    nodes: [],
    startOffset: 0,
    endOffset: ARCHIVE_NAP.length,
    text: ARCHIVE_NAP,
  },
},];

/**
 Archive document of the net-zero case: three paragraphs, sliced so that one
 paragraph can move ACROSS the join between two slices. That is what makes a
 net-zero reachable rather than hypothetical, since the separator between the
 slices belongs to neither of them.
 */
const ARCHIVE_PARAGRAPHS = 'The cat naps.\n\nThe sill is warm.\n\nThe bird waits.\n';

/**
 Two prepared slices over that document: the first carries two paragraphs and
 the second carries the last one.
 */
const PARAGRAPH_SLICES = [
  {
    source: {
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 3,
      text: 'source of the first two',
    },
    target: {
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 'The cat naps.\n\nThe sill is warm.'.length,
      text: 'The cat naps.\n\nThe sill is warm.',
    },
  },
  {
    source: {
      sliceIndex: 1,
      nodes: [],
      startOffset: 3,
      endOffset: 6,
      text: 'source of the last',
    },
    target: {
      sliceIndex: 1,
      nodes: [],
      startOffset: 'The cat naps.\n\nThe sill is warm.\n\n'.length,
      endOffset: 'The cat naps.\n\nThe sill is warm.\n\nThe bird waits.'.length,
      text: 'The bird waits.',
    },
  },
];

/**
 Wording a lane might replace the one slice with.
 */
const REWRITTEN_NAP = 'A cat dozes in the window.';

/**
 Runs one change-set case and returns whatever it raised.

 @param sliceCount - prepared slices bounding both sets

 @param shipped - slices said to carry a change

 @param withdrawn - slices said to have had one taken back

 @returns Failure raised, or undefined when the case was accepted

 @example
 ```ts
 const caught = changeSetFailure({ sliceCount: 2, shipped: [0,], withdrawn: [0,], },);
 ```
 */
function changeSetFailure(
  {
    sliceCount,
    shipped,
    withdrawn,
  }: {
    readonly sliceCount: number;
    readonly shipped: readonly number[];
    readonly withdrawn: readonly number[];
  },
): unknown {
  try {
    orderedChangeSets({
      sliceCount,
      shipped,
      withdrawn,
    },);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertReplacementsChange.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'passes a replacement that actually differs from the archive wording',
          fn: async () => {
            /**
             Failure the check raised, if any.
             */
            let caught: unknown;
            try {
              assertReplacementsChange({
                slices: CAT_SLICES,
                replacements: [{ sliceIndex: 0, replacementText: 'A cat dozes in the window.', },],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBe(undefined,);
          },
        },),
        it({
          name:
            'REFUSES a replacement that repeats its incumbent verbatim, which a truncated or stale cache file '
              + 'produces and which would otherwise be counted as a slice this lane changed while changing no byte',
          fn: async () => {
            /**
             Failure the check raised.
             */
            let caught: unknown;
            try {
              assertReplacementsChange({
                slices: CAT_SLICES,
                replacements: [{ sliceIndex: 0, replacementText: ARCHIVE_NAP, },],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(AssemblyContractError,);
            expect(String(caught,),).toContain('archive wording',);
          },
        },),
        it({
          name: 'REFUSES a replacement naming a slice this preparation never produced',
          fn: async () => {
            /**
             Failure the check raised.
             */
            let caught: unknown;
            try {
              assertReplacementsChange({
                slices: CAT_SLICES,
                replacements: [{ sliceIndex: 5, replacementText: 'A cat dozes in the window.', },],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(AssemblyContractError,);
          },
        },),
      ],
    },),

    describe({
      name: deriveShippedIndices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'DERIVES the shipped set from the surviving replacements rather than accepting one, which is '
              + 'what makes the two impossible to disagree: a document changed in one slice while a caller '
              + 'named another used to pass, since only emptiness was checked',
          fn: async () => {
            expect(deriveShippedIndices({
              incumbentText: ARCHIVE_NAP,
              assembledText: REWRITTEN_NAP,
              slices: CAT_SLICES,
              survivingReplacements: [{
                sliceIndex: 0,
                replacementText: REWRITTEN_NAP,
              },],
            },),).toEqual([0,],);
            expect(deriveShippedIndices({
              incumbentText: ARCHIVE_NAP,
              assembledText: ARCHIVE_NAP,
              slices: CAT_SLICES,
              survivingReplacements: [],
            },),).toEqual([],);
          },
        },),
        it({
          name:
            'REFUSES a document its own surviving replacements do not reconstruct, which is the check '
              + 'nothing performed before: the returned text and the reported set came from one list by '
              + 'convention rather than by construction',
          fn: async () => {
            /**
             Failure the check raised.
             */
            let caught: unknown;
            try {
              deriveShippedIndices({
                incumbentText: ARCHIVE_NAP,
                assembledText: 'Some third wording nobody proposed.',
                slices: CAT_SLICES,
                survivingReplacements: [{
                  sliceIndex: 0,
                  replacementText: REWRITTEN_NAP,
                },],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(AssemblyContractError,);
            expect(String(caught,),).toContain('surviving replacement assembles to',);
          },
        },),
        it({
          name:
            'REFUSES a document that moved while nothing survived, which is the same contradiction from '
              + 'the other side and would hide a rewrite from every per-slice reader, BY THE RE-SPLICE: no '
              + 'replacements reassemble to the archive itself, so the moved document is not what its zero '
              + 'survivors make',
          fn: async () => {
            /**
             Failure the check raised.
             */
            let caught: unknown;
            try {
              deriveShippedIndices({
                incumbentText: ARCHIVE_NAP,
                assembledText: REWRITTEN_NAP,
                slices: CAT_SLICES,
                survivingReplacements: [],
              },);
            }
            catch (error) {
              caught = error;
            }
            if (!(caught instanceof AssemblyContractError))
              throw new Error('the check raises an assembly contract refusal',);
            expect(caught.fault,).toEqual({
              kind: 'reassembly-differs',
              survivors: 0,
            },);
          },
        },),
        it({
          name:
            'REFUSES a surviving replacement repeating its own incumbent, so the check is sound when '
              + 'called on its own rather than relying on every caller having run the per-slice check '
              + 'before the guard: such a replacement survives assembly untouched and would be named as '
              + 'shipped',
          fn: async () => {
            /**
             Failure the check raised.
             */
            let caught: unknown;
            try {
              deriveShippedIndices({
                incumbentText: ARCHIVE_NAP,
                assembledText: ARCHIVE_NAP,
                slices: CAT_SLICES,
                survivingReplacements: [{
                  sliceIndex: 0,
                  replacementText: ARCHIVE_NAP,
                },],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(AssemblyContractError,);
            expect(String(caught,),).toContain('claims a change and carries the archive wording',);
          },
        },),
        it({
          name:
            'NAMES THE CALL ORDER when it refuses a net-zero set, because that refusal is the one a '
              + 'blameless run can reach: these two replacements each differ from their own slice and '
              + 'reassemble to the archive anyway, which is a legitimate outcome the guard canonicalizes '
              + 'to no survivors. Reaching this message means the guard has not run yet, and the message '
              + 'has to say that rather than describe the document',
          fn: async () => {
            /**
             Failure the check raised.
             */
            let caught: unknown;
            try {
              deriveShippedIndices({
                incumbentText: ARCHIVE_PARAGRAPHS,
                assembledText: ARCHIVE_PARAGRAPHS,
                slices: PARAGRAPH_SLICES,
                survivingReplacements: [
                  {
                    sliceIndex: 0,
                    replacementText: 'The cat naps.',
                  },
                  {
                    sliceIndex: 1,
                    replacementText: 'The sill is warm.\n\nThe bird waits.',
                  },
                ],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(AssemblyContractError,);
            expect(String(caught,),).toContain('guardFootnoteAssembly',);
          },
        },),
      ],
    },),

    describe({
      name: orderedChangeSets.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'puts BOTH sets in document order, which the withdrawn one never was: the guard returns it in the order '
              + 'it took slices back, so two lanes compared slice by slice were being read from lists ordered by '
              + 'different rules',
          fn: async () => {
            /**
             Sets given in the order a guard would produce them.
             */
            const ordered = orderedChangeSets({
              sliceCount: 6,
              shipped: [4, 0, 2,],
              withdrawn: [5, 1,],
            },);
            expect(ordered.shipped,).toEqual([0, 2, 4,],);
            expect(ordered.withdrawn,).toEqual([1, 5,],);
          },
        },),
        it({
          name: 'REFUSES an index outside the prepared slices, in either direction',
          fn: async () => {
            expect(changeSetFailure({
              sliceCount: 2,
              shipped: [2,],
              withdrawn: [],
            },),).toBeInstanceOf(AssemblyContractError,);
            expect(changeSetFailure({
              sliceCount: 2,
              shipped: [],
              withdrawn: [-1,],
            },),).toBeInstanceOf(AssemblyContractError,);
          },
        },),
        it({
          name: 'REFUSES an index that is not a whole number, which no slice can be',
          fn: async () => {
            expect(changeSetFailure({
              sliceCount: 4,
              shipped: [1.5,],
              withdrawn: [],
            },),).toBeInstanceOf(AssemblyContractError,);
          },
        },),
        it({
          name: 'REFUSES a repeat within either set, which would double-count one slice in every rate built on it',
          fn: async () => {
            expect(changeSetFailure({
              sliceCount: 4,
              shipped: [1, 1,],
              withdrawn: [],
            },),).toBeInstanceOf(AssemblyContractError,);
            expect(changeSetFailure({
              sliceCount: 4,
              shipped: [],
              withdrawn: [2, 2,],
            },),).toBeInstanceOf(AssemblyContractError,);
          },
        },),
        it({
          name:
            'reports the REPEAT before the range when a set breaks both, and names an out-of-range index in '
              + 'document order rather than in the order it was listed. Nothing depends on which failure comes '
              + 'first, which is exactly why it is pinned: the checks that need no slice count run first, and '
              + 'the range check reads the sets once they are ascending',
          fn: async () => {
            /**
             Failure a set breaking both the repeat rule and the range rule raised.
             */
            const bothBroken = changeSetFailure({
              sliceCount: 2,
              shipped: [2, 2,],
              withdrawn: [],
            },);
            expect(String(bothBroken,),).toContain('shipped slices repeat',);

            /**
             Failure two out-of-range indices raised, listed high to low.
             */
            const outOfOrder = changeSetFailure({
              sliceCount: 2,
              shipped: [9, 5,],
              withdrawn: [],
            },);
            expect(String(outOfOrder,),).toContain('slice 5 of 2 prepared',);
          },
        },),
        it({
          name:
            'REFUSES a slice named as both shipped and withdrawn, which both lane contracts call impossible '
              + 'by construction and neither checked',
          fn: async () => {
            /**
             Failure the overlap raised.
             */
            const caught = changeSetFailure({
              sliceCount: 4,
              shipped: [1, 3,],
              withdrawn: [3,],
            },);
            expect(caught,).toBeInstanceOf(AssemblyContractError,);
            expect(String(caught,),).toContain('both shipped and withdrawn',);
          },
        },),
        it({
          name: 'WORDS its refusal from the fault alone, as a marked class the boundary may print',
          fn: async () => {
            const error = new AssemblyContractError({
              fault: {
                kind: 'index-beyond-count',
                index: 9,
                sliceCount: 4,
              },
            },);
            expect(error.message,).toBe('change set names slice 9 of 4 prepared',);
            expect(error.messageNamesOnly,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
