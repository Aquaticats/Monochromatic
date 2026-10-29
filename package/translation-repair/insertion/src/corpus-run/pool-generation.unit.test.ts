/**
 Tests for naming the one built pipeline a draw's pool was settled under.

 THE FUNCTION IS KEYED ON THE KEPT NAMES, NOT ON THE LOOKUP, and that is the
 property most worth pinning. `EligibleEntries.digestByEntry` answers for every
 entry the pool ADMITTED, while a draw keeps a subset of those; reading the
 lookup's values directly would let an entry the draw never touched decide the
 pool's generation, or turn a clean single-generation draw into the
 two-generation refusal the "REFUSES a pool spanning two builds" case pins.
 Every case here therefore hands over a lookup wider than the kept names.

 THE LOOKUP KEYS ENTRY IDS AND THE DRAW KEEPS FILE NAMES. This file's
 fixtures used to key the lookup by file name too, agreeing with a lookup
 that read the kept name as it came, while the real lookup keys the census's
 entry ids; so every real draw's manifest recorded no generation at all
 (ledger B63). The fixtures now key by entry id, and one case drives the
 real census.

 THE COUNT AND THE DIGEST ARE COUNTED OVER DIFFERENT SETS, deliberately. The
 digest comes from kept entries that recorded one, and `entries` comes from
 every kept name. A pool where half the artifacts predate digest recording
 still has one generation and still offered the sample its full width, so
 collapsing the two counts would understate the pool a graded sheet was drawn
 from. A manifest that recorded no generation is the gap this closes, and that
 asymmetry is the closing.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactFileName,
  artifactFileNameOf,
  censusByGeneration,
  type EligibleEntries,
  keepEligible,
  listArtifactFiles,
  selectEligible,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import { poolGeneration, } from '../../dist/final/node/index.mjs';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { UNASKED_REPOSITORY, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

//region Pool generation tests

/**
 Entry one household's draw kept.
 */
const WHISKERS = 'whiskers';

/**
 Second entry of that draw.
 */
const MITTENS = 'mittens';

/**
 Third entry of that draw.
 */
const SAFFRON = 'saffron';

/**
 Entry the pool admitted and the draw did NOT keep.

 Present in every lookup this file builds, so any case that starts reading the lookup
 rather than the kept names fails on the extra generation it introduces.
 */
const UNDRAWN = 'pepperbox';

/**
 Built pipeline most of these artifacts were settled under.
 */
const SETTLED_UNDER = 'c4f9e1a7b2d6';

/**
 A different built pipeline, which a pool may not span.
 */
const OTHER_BUILD = '8e3a0d5c17bf';

/**
 Third build, carried only by the entry no draw kept.
 */
const UNDRAWN_BUILD = 'fa27b9046e3d';

/**
 Why a pool that recorded nothing cannot name a generation.
 */
const NOTHING_RECORDED = 'no kept entry recorded a pipeline digest';

/**
 Why a pool spanning builds cannot name one either.
 */
const TWO_GENERATIONS = 'pool holds 2 generations, which the pool guard should have refused';

/**
 Artifact file names the whole-household draw kept, as a draw hands them over.
 */
const WHOLE_HOUSEHOLD: readonly ArtifactFileName[] = [
  WHISKERS,
  MITTENS,
  SAFFRON,
].map(function toName(entryId,): ArtifactFileName {
  return artifactFileNameOf({ entryId, },);
},);

/**
 How many that comes to, which is what a manifest reports as the pool width.
 */
const HOUSEHOLD_SIZE = WHOLE_HOUSEHOLD.length;

/**
 Builds an eligibility result carrying one digest lookup and nothing else the
 function reads.

 The other fields are filled with what an empty pool would carry rather than
 with the kept names, precisely because `poolGeneration` must not consult
 them: a fixture that agreed with the kept names could not tell a reader of
 `entryIds` from a reader of `names`.

 @param digests - what each admitted entry recorded, keyed by entry id as the
 census keys it

 @returns Eligibility result shaped for this function's one question

 @example
 ```ts
 const eligible = pooled({ digests: [[WHISKERS, SETTLED_UNDER,],], },);
 ```
 */
function pooled(
  {
    digests,
  }: {
    readonly digests: readonly (readonly [
      string,
      string,
    ])[];
  },
): EligibleEntries {
  return {
    entryIds: [],
    excludedIds: [],
    malformedIds: [],
    tipByEntry: new Map(),
    digestByEntry: new Map(digests,),
    selection: { kind: 'all-generations', },
    report: [],
  };
}

/**
 Commit the directory's artifacts record, a full object id so they place.
 */
const PLACED_TIP = 'a1b2c3d4e5f60718293a4b5c6d7e8f9012345678';

/**
 Built pipeline the directory's artifacts record, in the scheme this build
 reads.
 */
const PLACED_DIGEST = `sha256-tree-v1:${'0123456789abcdef'.repeat(4,)}`;

/**
 Makes one throwaway artifacts directory holding a placed artifact for each
 entry id, as a pass writes them.

 @param entryIds - entries to settle

 @returns Directory, removed on dispose

 @example
 ```ts
 await using settled = await settledArtifacts({ entryIds: ['whiskers',], },);
 ```
 */
async function settledArtifacts(
  { entryIds, }: { readonly entryIds: readonly string[]; },
): Promise<AsyncDisposable & { readonly artifactsDir: string; }> {
  // Throwaway artifacts directory, never a real run.
  return await scratchDirWith({
    prefix: 'pool-generation-',
    setup: async function seeded({ path: artifactsDir, },): Promise<{ readonly artifactsDir: string; }> {
      await Promise.all(entryIds.map(async function settleOne(entryId,): Promise<void> {
        await writeFile(
          join(
            artifactsDir,
            `${entryId}.json`,
          ),
          JSON.stringify({
            id: entryId,
            tip: PLACED_TIP,
            pipelineDigest: PLACED_DIGEST,
          },),
          'utf8',
        );
      },),);

      return { artifactsDir, };
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: poolGeneration.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the one build every kept entry recorded, and how many were offered',
          fn: async () => {
            expect(poolGeneration({
              eligible: pooled({
                digests: [
                  [
                    WHISKERS,
                    SETTLED_UNDER,
                  ],
                  [
                    MITTENS,
                    SETTLED_UNDER,
                  ],
                  [
                    SAFFRON,
                    SETTLED_UNDER,
                  ],
                  [
                    UNDRAWN,
                    UNDRAWN_BUILD,
                  ],
                ],
              },),
              names: WHOLE_HOUSEHOLD,
            },),).toEqual({
              kind: 'recorded',
              digest: SETTLED_UNDER,
              entries: HOUSEHOLD_SIZE,
            },);
          },
        },),
        it({
          name: 'COUNTS every kept name, including ones that recorded no build',
          fn: async () => {
            // Only one of the three kept artifacts recorded a digest. The pool still
            // has one generation and still offered three entries, so the count must
            // not shrink to the one that happened to be tagged.
            expect(poolGeneration({
              eligible: pooled({
                digests: [
                  [
                    MITTENS,
                    SETTLED_UNDER,
                  ],
                  [
                    UNDRAWN,
                    UNDRAWN_BUILD,
                  ],
                ],
              },),
              names: WHOLE_HOUSEHOLD,
            },),).toEqual({
              kind: 'recorded',
              digest: SETTLED_UNDER,
              entries: HOUSEHOLD_SIZE,
            },);
          },
        },),
        it({
          name: 'REFUSES to name a build when no kept entry recorded one',
          fn: async () => {
            // The lookup is not empty: it holds a build for an entry this draw did
            // not keep. Reading values rather than kept names would report that one
            // as the pool's generation.
            expect(poolGeneration({
              eligible: pooled({
                digests: [
                  [
                    UNDRAWN,
                    UNDRAWN_BUILD,
                  ],
                ],
              },),
              names: WHOLE_HOUSEHOLD,
            },),).toEqual({
              kind: 'unrecorded',
              reason: NOTHING_RECORDED,
            },);
          },
        },),
        it({
          name: 'REFUSES to name a build for a draw that kept nothing',
          fn: async () => {
            expect(poolGeneration({
              eligible: pooled({
                digests: [
                  [
                    WHISKERS,
                    SETTLED_UNDER,
                  ],
                ],
              },),
              names: [],
            },),).toEqual({
              kind: 'unrecorded',
              reason: NOTHING_RECORDED,
            },);
          },
        },),
        it({
          name: 'REFUSES a pool spanning two builds, and says the guard should have caught it',
          fn: async () => {
            expect(poolGeneration({
              eligible: pooled({
                digests: [
                  [
                    WHISKERS,
                    SETTLED_UNDER,
                  ],
                  [
                    MITTENS,
                    OTHER_BUILD,
                  ],
                  [
                    SAFFRON,
                    SETTLED_UNDER,
                  ],
                ],
              },),
              names: WHOLE_HOUSEHOLD,
            },),).toEqual({
              kind: 'unrecorded',
              reason: TWO_GENERATIONS,
            },);
          },
        },),
        it({
          name: 'IGNORES an undrawn entry that recorded a build of its own',
          fn: async () => {
            // Same lookup as the two-generation case except the second build sits on
            // the entry nobody kept. A reader of the lookup refuses this pool; a
            // reader of the kept names names its one build, which is correct.
            expect(poolGeneration({
              eligible: pooled({
                digests: [
                  [
                    WHISKERS,
                    SETTLED_UNDER,
                  ],
                  [
                    MITTENS,
                    SETTLED_UNDER,
                  ],
                  [
                    SAFFRON,
                    SETTLED_UNDER,
                  ],
                  [
                    UNDRAWN,
                    OTHER_BUILD,
                  ],
                ],
              },),
              names: WHOLE_HOUSEHOLD,
            },),).toEqual({
              kind: 'recorded',
              digest: SETTLED_UNDER,
              entries: HOUSEHOLD_SIZE,
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${poolGeneration.name} over a census`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the build of a pool resolved from real artifacts, whose lookup keys entry ids while the '
            + 'draw keeps file names (ledger B63)',
          fn: async () => {
            /**
             Entries the directory settles, every one of which the draw keeps.
             */
            const entryIds = [
              'whiskers',
              'mittens',
            ];
            await using settled = await settledArtifacts({ entryIds, },);

            /**
             The directory as a draw lists it.
             */
            const listed = await listArtifactFiles({ artifactsDir: settled.artifactsDir, },);

            /**
             The pool as the draw resolves it, from one census of that listing.
             */
            const eligible = await selectEligible({
              census: await censusByGeneration({
                artifactsDir: settled.artifactsDir,
                names: listed,
              },),
              pooledDeliberately: false,
              repository: UNASKED_REPOSITORY,
            },);

            expect(poolGeneration({
              eligible,
              names: keepEligible({
                names: listed,
                eligible,
              },),
            },),).toEqual({
              kind: 'recorded',
              digest: PLACED_DIGEST,
              entries: entryIds.length,
            },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Pool generation tests
