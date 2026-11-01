/**
 Tests for the draw manifest, the only record of which issue sat at which
 sheet position.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { GradingCandidate, } from '@monochromatic-dev/translation-repair-house-rule';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  buildSampleManifest,
  parseSampleManifest,
} from '../dist/final/node/index.mjs';

/**
 Builds one drawn candidate carrying only what the manifest reads.

 @param issueId - adjudicated identity

 @param entryId - corpus entry it came from

 @returns Candidate the manifest records

 @example
 ```ts
 const candidate = catCandidate({ issueId: 'adjudicated/nap', },);
 ```
 */
function catCandidate(
  {
    issueId,
    entryId = 'Kitten',
  }: {
    readonly issueId: string;
    readonly entryId?: string;
  },
): GradingCandidate {
  return {
    entryId,
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
 A recorded generation, so every fixture manifest says which pipeline settled
 the pool it was drawn from.
 */
const CAT_GENERATION = {
  kind: 'recorded',
  digest: 'sha256-tree-v1:tabbycafe',
  entries: 3,
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildSampleManifest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'numbers items from one in draw order, matching how both sheets '
            + 'render them, since the position IS the join key a grader returns',
          fn: async () => {
            const manifest = buildSampleManifest({
              sample: [
                catCandidate({ issueId: 'adjudicated/nap', },),
                catCandidate({
                  issueId: 'adjudicated/chase',
                  entryId: 'Mittens',
                },),
              ],
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              generation: CAT_GENERATION,
            },);
            expect(manifest.items,).toHaveLength(2,);
            expect(manifest.items[0]?.position,).toBe(1,);
            expect(manifest.items[0]?.issueId,).toBe('adjudicated/nap',);
            expect(manifest.items[1]?.position,).toBe(2,);
            expect(manifest.items[1]?.entryId,).toBe('Mittens',);
          },
        },),

        it({
          name: 'records the seed and corpus commit, so a manifest cannot be '
            + 'silently paired with another draw\'s sheets',
          fn: async () => {
            const manifest = buildSampleManifest({
              sample: [],
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              generation: CAT_GENERATION,
            },);
            expect(manifest.seed,).toBe('cat-seed',);
            expect(manifest.corpusSha,).toBe('sha/1',);
          },
        },),
      ],
    },),

    describe({
      name: parseSampleManifest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A MANIFEST THAT PREDATES THE GENERATION FIELD as unrecorded, never as a guess: '
            + 'a sample attributed to a pipeline nobody checked is worse evidence than one that says '
            + 'it does not know, because only the second stops a reader quoting it',
          fn: async () => {
            /**
             A manifest as it was written before the field existed.
             */
            const older = buildSampleManifest({
              sample: [catCandidate({ issueId: 'adjudicated/nap', },),],
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              generation: CAT_GENERATION,
            },);

            /**
             The same manifest with the generation removed, which is exactly what
             an older draw wrote.

             Destructured rather than deleted from a clone, so the field's absence
             is a fact about this binding rather than a mutation a reader has to
             trace.
             */
            const {
              generation: _dropped,
              ...withoutGeneration
            } = older;

            /**
             What the parser makes of it.
             */
            const parsed = parseSampleManifest({ value: withoutGeneration, },);

            expect(parsed.generation.kind,).toBe('unrecorded',);
            if (parsed.generation.kind !== 'unrecorded')
              throw new Error('unrecorded by construction',);
            expect(parsed.generation.reason.length,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'READS A MANIFEST WHOSE GENERATION SAYS IT IS UNRECORDED as unrecorded, carrying the '
            + 'reason it gave',
          fn: async () => {
            const parsed = parseSampleManifest({
              value: buildSampleManifest({
                sample: [catCandidate({ issueId: 'adjudicated/nap', },),],
                seed: 'cat-seed',
                corpusSha: 'sha/1',
                generation: {
                  kind: 'unrecorded',
                  reason: 'the draw named no pipeline',
                },
              },),
            },);
            expect(parsed.generation.kind,).toBe('unrecorded',);
            if (parsed.generation.kind !== 'unrecorded')
              throw new Error('unrecorded by construction',);
            expect(parsed.generation.reason,).toBe('the draw named no pipeline',);
          },
        },),

        it({
          name: 'round-trips what the draw wrote, which is the only property the '
            + 'join depends on',
          fn: async () => {
            /** Manifest as the draw builds it. */
            const built = buildSampleManifest({
              sample: [catCandidate({ issueId: 'adjudicated/nap', },),],
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              generation: CAT_GENERATION,
            },);

            /**
             Manifest text exactly as the draw writes it beside the sheets.

             Serialized and re-read rather than cloned, for the same reason the
             provenance suite is: the file is the boundary under test, and a deep
             clone would keep what a file drops.
             */
            const onDisk = JSON.stringify(built,);

            expect(
              parseSampleManifest({ value: JSON.parse(onDisk,), },),
            ).toEqual(built,);
          },
        },),

        it({
          name: 'throws rather than skipping a malformed item, because dropping '
            + 'one shifts every later position and turns the join into a '
            + 'mislabelling instead of a gap anyone would notice',
          fn: async () => {
            expect(function readsMalformed() {
              parseSampleManifest({
                value: {
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
                    },
                  ],
                },
              },);
            },).toThrow(ArtifactParseError,);
          },
        },),

        it({
          name: 'RECOMPUTES the digest from the items rather than trusting the one '
            + 'written beside them. A stored digest that is never checked against '
            + 'its own contents proves only that two files carry the same string, '
            + 'so editing the items and leaving the digest alone would still match '
            + 'a sheet carrying the stale value',
          fn: async () => {
            /** Manifest as the draw builds it. */
            const built = buildSampleManifest({
              sample: [catCandidate({ issueId: 'adjudicated/nap', },),],
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              generation: CAT_GENERATION,
            },);

            expect(function readsTamperedItems() {
              parseSampleManifest({
                value: {
                  ...built,
                  items: [
                    {
                      position: 1,
                      entryId: 'Kitten',
                      issueId: 'adjudicated/chase',
                    },
                  ],
                },
              },);
            },).toThrow(ArtifactParseError,);
          },
        },),

        it({
          name: 'reads a manifest carrying NO digest, since every draw taken '
            + 'before the binding existed reads that way and a final sheet cannot '
            + 'be redrawn to acquire one',
          fn: async () => {
            expect(
              parseSampleManifest({
                value: {
                  seed: 'cat-seed',
                  corpusSha: 'sha/1',
                  items: [
                    {
                      position: 1,
                      entryId: 'Kitten',
                      issueId: 'adjudicated/nap',
                    },
                  ],
                },
              },).drawDigest,
            ).toBe(undefined,);
          },
        },),

        it({
          name: 'THROWS on a position that disagrees with where its item sits. '
            + 'Both scorers read grades by array order and take the issue id from '
            + 'the same index, so a manifest recording another order describes one '
            + 'join while the code performs a different one',
          fn: async () => {
            expect(function readsShuffledPositions() {
              parseSampleManifest({
                value: {
                  seed: 'cat-seed',
                  corpusSha: 'sha/1',
                  items: [
                    {
                      position: 2,
                      entryId: 'Kitten',
                      issueId: 'adjudicated/nap',
                    },
                    {
                      position: 1,
                      entryId: 'Kitten',
                      issueId: 'adjudicated/chase',
                    },
                  ],
                },
              },);
            },).toThrow(ArtifactParseError,);
          },
        },),
      ],
    },),
  ],
},);
