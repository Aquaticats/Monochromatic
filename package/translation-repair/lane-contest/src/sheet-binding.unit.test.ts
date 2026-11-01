/**
 Tests for the check that refuses a graded sheet scored against the wrong
 manifest.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type { GradingCandidate, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  assertSheetMatchesManifest,
  buildSampleManifest,
  computeDrawDigest,
  requireSheetSeed,
  type SampleManifest,
  SheetBindingError,
} from '../dist/final/node/index.mjs';

/**
 Builds one drawn candidate carrying only what the manifest reads.

 @param issueId - adjudicated identity

 @returns Candidate the manifest records

 @example
 ```ts
 const candidate = catCandidate({ issueId: 'adjudicated/nap', },);
 ```
 */
function catCandidate(
  { issueId, }: { readonly issueId: string; },
): GradingCandidate {
  return {
    entryId: 'Kitten',
    band: 'small',
    issueId,
    category: 'fluency/grammar',
    severity: 'major',
    summary: 'the tense is wrong',
    sourceAnchor: 'quoted',
    sourceQuotes: [],
    targetQuotes: [],
  };
}

/**
 Manifest of a one-item draw, digest included.

 @param issueId - issue drawn at position one

 @returns Manifest as the draw writes it

 @example
 ```ts
 const manifest = catManifest({ issueId: 'adjudicated/nap', },);
 ```
 */
function catManifest(
  { issueId, }: { readonly issueId: string; },
): SampleManifest {
  return buildSampleManifest({
    sample: [catCandidate({ issueId, },),],
    seed: 'cat-seed',
    corpusSha: 'sha/1',
    generation: CAT_GENERATION,
  },);
}

/**
 A recorded generation, so every fixture manifest says which pipeline settled
 the pool it was drawn from.
 */
const CAT_GENERATION = {
  kind: 'recorded',
  digest: 'sha256-tree-v1:tabbycafe',
  entries: 3,
} as const;

/**
 Graded sheet every case names, as an operator's flag would.
 */
const SHEET_PATH = '/runs/grading-sheet-cat-seed.md';

/**
 Draw manifest every case names.
 */
const MANIFEST_PATH = '/runs/sample-manifest-cat-seed.json';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertSheetMatchesManifest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports a DIGEST binding when both sides carry the same one, '
            + 'which is the only state that proves the grades and the manifest '
            + 'describe the same items rather than the same file names',
          fn: async () => {
            /** Manifest of the draw under test. */
            const manifest = catManifest({ issueId: 'adjudicated/nap', },);

            expect(assertSheetMatchesManifest({
              identity: {
                seed: 'cat-seed',
                corpusSha: 'sha/1',
                drawDigest: manifest.drawDigest ?? '',
              },
              manifest,
              sheetLabel: 'repair sheet',
              sheetPath: SHEET_PATH,
              manifestPath: MANIFEST_PATH,
            },),).toBe('digest',);
          },
        },),

        it({
          name: 'REFUSES a sheet whose digest differs while its seed and corpus '
            + 'pin agree. That is exactly the case the digest exists for: the draw '
            + 'is deterministic in its seed but not in its pool, so redrawing the '
            + 'same seed after another entry settles names different issues at the '
            + 'same positions, and a positional join would mislabel every verdict '
            + 'without erroring',
          fn: async () => {
            /** Manifest of the draw actually taken. */
            const manifest = catManifest({ issueId: 'adjudicated/nap', },);

            /**
             What scoresAnotherDraw raised, read for its class as well as its wording.
             */
            const refusalOfScoresAnotherDraw = caught(function scoresAnotherDraw() {
              assertSheetMatchesManifest({
                identity: {
                  seed: 'cat-seed',
                  corpusSha: 'sha/1',
                  drawDigest: computeDrawDigest({
                    seed: 'cat-seed',
                    corpusSha: 'sha/1',
                    items: [
                      {
                        position: 1,
                        entryId: 'Kitten',
                        issueId: 'adjudicated/chase',
                      },
                    ],
                  },),
                },
                manifest,
                sheetLabel: 'repair sheet',
                sheetPath: SHEET_PATH,
                manifestPath: MANIFEST_PATH,
              },);
            },);

            expect(refusalOfScoresAnotherDraw,).toBeInstanceOf(SheetBindingError,);
            expect(refusalOfScoresAnotherDraw,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusalOfScoresAnotherDraw,),).toBe(
              `SheetBindingError: repair sheet ${SHEET_PATH} and manifest ${MANIFEST_PATH} carry different draw `
                + 'digests while agreeing on seed and corpus pin. The draw is deterministic in its seed but not in its '
                + 'pool, so the same seed drawn after another entry settled names a different set of items at the '
                + 'same positions. Name the manifest this sheet was drawn with.',
            );
          },
        },),

        it({
          name: 'reports a HEADER-ONLY binding when NEITHER side carries a digest, '
            + 'rather than refusing it. Round three was drawn before the binding '
            + 'existed and a final draw refuses to overwrite itself, so refusing '
            + 'would strand hours of grading that nothing can reproduce',
          fn: async () => {
            expect(assertSheetMatchesManifest({
              identity: {
                seed: 'cat-seed',
                corpusSha: 'sha/1',
                drawDigest: '',
              },
              manifest: {
                seed: 'cat-seed',
                corpusSha: 'sha/1',
                generation: CAT_GENERATION,
                items: [
                  {
                    position: 1,
                    entryId: 'Kitten',
                    issueId: 'adjudicated/nap',
                  },
                ],
              },
              sheetLabel: 'detection sheet',
              sheetPath: SHEET_PATH,
              manifestPath: MANIFEST_PATH,
            },),).toBe('header-only',);
          },
        },),

        it({
          name: 'REFUSES a pair where only ONE side carries a digest, in both '
            + 'directions. One draw writes sheet and manifest in the same instant '
            + 'and always computes a digest now, so a one-sided pair was assembled '
            + 'from two draws or lost a digest; treating it as legacy would let '
            + 'the older file\'s absence excuse the newer file\'s presence and '
            + 'reopen the very join this check closes',
          fn: async () => {
            /** Manifest of a bound draw. */
            const bound = catManifest({ issueId: 'adjudicated/nap', },);

            /** Manifest of an unbound draw carrying the same items. */
            const unbound = {
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              generation: CAT_GENERATION,
              items: [
                {
                  position: 1,
                  entryId: 'Kitten',
                  issueId: 'adjudicated/nap',
                },
              ],
            };

            /**
             What legacySheetWithBoundManifest raised, read for its class as well as its wording.
             */
            const refusalOfLegacySheetWithBoundManifest = caught(function legacySheetWithBoundManifest() {
              assertSheetMatchesManifest({
                identity: {
                  seed: 'cat-seed',
                  corpusSha: 'sha/1',
                  drawDigest: '',
                },
                manifest: bound,
                sheetLabel: 'detection sheet',
                sheetPath: SHEET_PATH,
                manifestPath: MANIFEST_PATH,
              },);
            },);

            expect(refusalOfLegacySheetWithBoundManifest,).toBeInstanceOf(SheetBindingError,);
            expect(String(refusalOfLegacySheetWithBoundManifest,),).toBe(
              `SheetBindingError: detection sheet ${SHEET_PATH} and manifest ${MANIFEST_PATH} disagree about whether `
                + 'this draw is bound: the manifest carries a draw digest and the sheet carries none. One draw writes '
                + 'both in the same instant, so this pair was assembled from two different draws, or the sheet lost '
                + 'its digest. Name a sheet and a manifest that one draw wrote.',
            );

            /**
             What boundSheetWithLegacyManifest raised, read for its class as well as its wording.
             */
            const refusalOfBoundSheetWithLegacyManifest = caught(function boundSheetWithLegacyManifest() {
              assertSheetMatchesManifest({
                identity: {
                  seed: 'cat-seed',
                  corpusSha: 'sha/1',
                  drawDigest: bound.drawDigest ?? '',
                },
                manifest: unbound,
                sheetLabel: 'detection sheet',
                sheetPath: SHEET_PATH,
                manifestPath: MANIFEST_PATH,
              },);
            },);

            expect(refusalOfBoundSheetWithLegacyManifest,).toBeInstanceOf(SheetBindingError,);
            expect(String(refusalOfBoundSheetWithLegacyManifest,),).toBe(
              `SheetBindingError: detection sheet ${SHEET_PATH} and manifest ${MANIFEST_PATH} disagree about whether `
                + 'this draw is bound: the sheet carries a draw digest and the manifest carries none. One draw writes '
                + 'both in the same instant, so this pair was assembled from two different draws, or the manifest lost '
                + 'its digest. Name a sheet and a manifest that one draw wrote.',
            );
          },
        },),

        it({
          name: 'refuses a mismatched seed and a mismatched corpus pin, since one '
            + 'names a different draw and the other names different document text '
            + 'under the same ids',
          fn: async () => {
            /** Manifest of the draw under test. */
            const manifest = catManifest({ issueId: 'adjudicated/nap', },);

            /**
             What scoresAnotherSeed raised, read for its class as well as its wording.
             */
            const refusalOfScoresAnotherSeed = caught(function scoresAnotherSeed() {
              assertSheetMatchesManifest({
                identity: {
                  seed: 'other-seed',
                  corpusSha: 'sha/1',
                  drawDigest: '',
                },
                manifest,
                sheetLabel: 'detection sheet',
                sheetPath: SHEET_PATH,
                manifestPath: MANIFEST_PATH,
              },);
            },);

            expect(refusalOfScoresAnotherSeed,).toBeInstanceOf(SheetBindingError,);
            expect(String(refusalOfScoresAnotherSeed,),).toBe(
              `SheetBindingError: detection sheet ${SHEET_PATH} and manifest ${MANIFEST_PATH} belong to different `
                + 'draws: their seeds differ. Item counts can match across unrelated draws of the same size, so '
                + 'position is not evidence they describe the same items. Name a sheet and a manifest that one draw '
                + 'wrote.',
            );

            /**
             What scoresAnotherPin raised, read for its class as well as its wording.
             */
            const refusalOfScoresAnotherPin = caught(function scoresAnotherPin() {
              assertSheetMatchesManifest({
                identity: {
                  seed: 'cat-seed',
                  corpusSha: 'sha/2',
                  drawDigest: '',
                },
                manifest,
                sheetLabel: 'detection sheet',
                sheetPath: SHEET_PATH,
                manifestPath: MANIFEST_PATH,
              },);
            },);

            expect(refusalOfScoresAnotherPin,).toBeInstanceOf(SheetBindingError,);
            expect(String(refusalOfScoresAnotherPin,),).toBe(
              `SheetBindingError: detection sheet ${SHEET_PATH} and manifest ${MANIFEST_PATH} were produced against `
                + 'different corpus commits. The same entry can carry different text at two commits, so the grades '
                + 'and the artifacts would be about different documents. Name a sheet and a manifest produced against '
                + 'one corpus commit.',
            );
          },
        },),
      ],
    },),

    describe({
      name: requireSheetSeed.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns the seed a sheet declares',
          fn: async () => {
            expect(requireSheetSeed({
              identity: {
                seed: 'cat-seed',
                corpusSha: 'sha/1',
                drawDigest: '',
              },
              sheetLabel: 'detection sheet',
              sheetPath: SHEET_PATH,
            },),).toBe('cat-seed',);
          },
        },),

        it({
          name: 'THROWS on a sheet declaring no seed, rather than falling back to '
            + 'the current default. Every sheet the formatters write carries the '
            + 'header, so a file without one cannot be placed, and defaulting it '
            + 'would resolve its pre-grades and manifest under whichever round is '
            + 'being worked on now: the mispairing the binding exists to stop, '
            + 'arriving through the back door',
          fn: async () => {
            /**
             What scoresAnUnplaceableSheet raised, read for its class as well as its wording.
             */
            const refusalOfScoresAnUnplaceableSheet = caught(function scoresAnUnplaceableSheet() {
              requireSheetSeed({
                identity: {
                  seed: '',
                  corpusSha: 'sha/1',
                  drawDigest: '',
                },
                sheetLabel: 'detection sheet',
                sheetPath: SHEET_PATH,
              },);
            },);

            expect(refusalOfScoresAnUnplaceableSheet,).toBeInstanceOf(SheetBindingError,);
            expect(String(refusalOfScoresAnUnplaceableSheet,),).toBe(
              `SheetBindingError: detection sheet ${SHEET_PATH} declares no draw seed, so nothing can say which draw `
                + 'it came from. Every sheet the formatters write carries a "Draw seed: " header; a file without one '
                + 'cannot be paired with a manifest or with pre-grades except by guessing. Name a sheet a draw wrote.',
            );
          },
        },),
      ],
    },),

    describe({
      name: computeDrawDigest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'changes when any item identity changes, because that is the only '
            + 'thing standing between a positional join and a silent mislabelling',
          fn: async () => {
            /** Digest of the draw as taken. */
            const original = computeDrawDigest({
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              items: [
                {
                  position: 1,
                  entryId: 'Kitten',
                  issueId: 'adjudicated/nap',
                },
              ],
            },);

            expect(computeDrawDigest({
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              items: [
                {
                  position: 1,
                  entryId: 'Mittens',
                  issueId: 'adjudicated/nap',
                },
              ],
            },),).not.toBe(original,);
          },
        },),

        it({
          name: 'distinguishes two draws whose fields concatenate alike, so an '
            + 'entry id containing the separator a delimiter-joined encoding would '
            + 'use cannot impersonate a different pair of items',
          fn: async () => {
            expect(computeDrawDigest({
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              items: [
                {
                  position: 1,
                  entryId: 'Kitten|adjudicated',
                  issueId: 'nap',
                },
              ],
            },),).not.toBe(computeDrawDigest({
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              items: [
                {
                  position: 1,
                  entryId: 'Kitten',
                  issueId: 'adjudicated|nap',
                },
              ],
            },),);
          },
        },),

        it({
          name: 'depends on ORDER, since the sheets render items in manifest order '
            + 'and two draws holding the same issues at swapped positions would '
            + 'otherwise prove nothing about which grade belongs to which issue',
          fn: async () => {
            expect(computeDrawDigest({
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              items: [
                {
                  position: 1,
                  entryId: 'Kitten',
                  issueId: 'adjudicated/nap',
                },
                {
                  position: 2,
                  entryId: 'Kitten',
                  issueId: 'adjudicated/chase',
                },
              ],
            },),).not.toBe(computeDrawDigest({
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              items: [
                {
                  position: 1,
                  entryId: 'Kitten',
                  issueId: 'adjudicated/chase',
                },
                {
                  position: 2,
                  entryId: 'Kitten',
                  issueId: 'adjudicated/nap',
                },
              ],
            },),);
          },
        },),
      ],
    },),
  ],
},);
