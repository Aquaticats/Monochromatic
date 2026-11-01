/**
 Settled-artifact directories for the score runners' cases.

 A runs directory holds an `artifacts` directory of one JSON file per entry,
 each stamped with the pipeline commit and built pipeline the pool partitions
 by. Every body written here carries one shared stamp, so a directory is a
 single generation unless a body names its own.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '../roster-seats.test-fixture.ts';

/**
 Hex digits in a pipeline commit.
 */
const COMMIT_HEX_LENGTH = 40;

/**
 Hex digits in a tree digest.
 */
const DIGEST_HEX_LENGTH = 64;

/**
 Pipeline commit every fixture artifact carries unless its body names one.
 */
const SHARED_COMMIT: string = 'f'.repeat(COMMIT_HEX_LENGTH,);

/**
 Built pipeline every fixture artifact carries unless its body names one.
 */
const SHARED_PIPELINE: string = `sha256-tree-v1:${'f'.repeat(DIGEST_HEX_LENGTH,)}`;

/**
 An entry that carries attribution and settled two issues, one resting on a
 critic that repeated itself and one on two critics.
 */
const WHISKERS_ARTIFACT: Readonly<Record<string, unknown>> = {
  artifactSchemaVersion: 1,
  id: 'Whiskers',
  chunkCritics: [
    {
      chunkIndex: 0,
      heardCriticIds: [
        SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
      ],
      claimAttributions: [
        {
          claimId: 'issue/nap',
          proposers: [
            {
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              emissionCount: 2,
            },
          ],
        },
        {
          claimId: 'issue/chase',
          proposers: [
            {
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              emissionCount: 1,
            },
            {
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              emissionCount: 1,
            },
          ],
        },
      ],
    },
  ],
  issues: [
    {
      chunkIndex: 0,
      issue: {
        status: 'accepted',
        claims: [{ claimId: 'issue/nap', },],
      },
    },
    {
      chunkIndex: 0,
      issue: {
        status: 'accepted',
        claims: [{ claimId: 'issue/chase', },],
      },
    },
  ],
};

/**
 An entry settled before attribution existed.
 */
export const MITTENS_ARTIFACT: Readonly<Record<string, unknown>> = {
  artifactSchemaVersion: 1,
  id: 'Mittens',
  issues: [],
};

/**
 Artifacts of one run: the attributed entry and the one settled before
 attribution existed.
 */
export const ATTRIBUTED_CAT_ARTIFACTS: Readonly<Record<string, Readonly<Record<string, unknown>>>> = {
  'Whiskers.json': WHISKERS_ARTIFACT,
  'Mittens.json': MITTENS_ARTIFACT,
};

/**
 Writes artifacts into the `artifacts` directory of a runs directory.

 @param runsDir - runs directory the case owns

 @param artifacts - file name to body; an object body gets the shared stamp
 unless it names its own, and a string body is written as it is, so a case
 can leave a file truncated

 @example
 ```ts
 await writeScoreArtifacts({ runsDir: scratch.path, artifacts: { 'Whiskers.json': { id: 'Whiskers', }, }, },);
 ```
 */
export async function writeScoreArtifacts(
  {
    runsDir,
    artifacts,
  }: {
    readonly runsDir: string;
    readonly artifacts: Readonly<Record<string, Readonly<Record<string, unknown>> | string>>;
  },
): Promise<void> {
  /**
   Directory the pass writes entries into.
   */
  const artifactsDir = join(
    runsDir,
    'artifacts',
  );
  await mkdir(
    artifactsDir,
    { recursive: true, },
  );
  await Promise.all(Object
    .entries(artifacts,)
    .map(async function writeOne([name, body,],): Promise<void> {
      await writeFile(
        join(
          artifactsDir,
          name,
        ),
        ((typeof body) === 'string')
          ? body
          : JSON.stringify({
            tip: SHARED_COMMIT,
            pipelineDigest: SHARED_PIPELINE,
            ...body,
          },),
        'utf8',
      );
    },),);
}

/**
 An entry whose three attributed claims each ended differently: accepted
 on one critic's word, rejected on another's, and left to a human where both
 had proposed it.
 */
export const BISCUIT_ARTIFACT: Readonly<Record<string, unknown>> = {
  artifactSchemaVersion: 1,
  id: 'Biscuit',
  chunkCritics: [
    {
      chunkIndex: 0,
      heardCriticIds: [
        SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        SEAT_SYNTHETIC_VISION_WITHHELD,
      ],
      claimAttributions: [
        {
          claimId: 'issue/nap',
          proposers: [
            {
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              emissionCount: 1,
            },
          ],
        },
        {
          claimId: 'issue/chase',
          proposers: [
            {
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              emissionCount: 1,
            },
          ],
        },
        {
          claimId: 'issue/purr',
          proposers: [
            {
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              emissionCount: 1,
            },
            {
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              emissionCount: 1,
            },
          ],
        },
      ],
    },
  ],
  issues: [
    {
      chunkIndex: 0,
      issue: {
        status: 'accepted',
        claims: [{ claimId: 'issue/nap', },],
      },
    },
    {
      chunkIndex: 0,
      issue: {
        status: 'rejected',
        claims: [{ claimId: 'issue/chase', },],
      },
    },
    {
      chunkIndex: 0,
      issue: {
        status: 'needs-human',
        claims: [{ claimId: 'issue/purr', },],
      },
    },
  ],
};

/**
 An entry settled before attribution existed whose accepted issue names a claim
 no proposer was ever recorded for.
 */
export const POUNCE_ARTIFACT: Readonly<Record<string, unknown>> = {
  artifactSchemaVersion: 1,
  id: 'Pounce',
  issues: [
    {
      chunkIndex: 0,
      issue: {
        status: 'accepted',
        claims: [{ claimId: 'issue/old', },],
      },
    },
  ],
};

/**
 An entry that records attribution of a claim only, so no issue names an
 attributed claim.

 @param claims - claims the entry attributes, each proposed by the listed critics

 @param issues - issue records, as the artifact carries them

 @returns Artifact body for an entry called Tuft

 @example
 ```ts
 const body = tuftArtifact({ claims: [], issues: [], },);
 ```
 */
export function tuftArtifact(
  {
    claims,
    issues,
  }: {
    readonly claims: readonly {
      readonly claimId: string;
      readonly proposers: readonly string[];
    }[];
    readonly issues: readonly {
      readonly status: string;
      readonly claimIds: readonly string[];
    }[];
  },
): Readonly<Record<string, unknown>> {
  return {
    artifactSchemaVersion: 1,
    id: 'Tuft',
    chunkCritics: [
      {
        chunkIndex: 0,
        heardCriticIds: [
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_SYNTHETIC_VISION_WITHHELD,
        ],
        claimAttributions: claims.map(function toAttribution({
          claimId,
          proposers,
        },): Readonly<Record<string, unknown>> {
          return {
            claimId,
            proposers: proposers.map(function toProposer(modelId,): Readonly<Record<string, unknown>> {
              return {
                modelId,
                emissionCount: 1,
              };
            },),
          };
        },),
      },
    ],
    issues: issues.map(function toIssue({
      status,
      claimIds,
    },): Readonly<Record<string, unknown>> {
      return {
        chunkIndex: 0,
        issue: {
          status,
          claims: claimIds.map(function toClaim(claimId,): Readonly<Record<string, unknown>> {
            return { claimId, };
          },),
        },
      };
    },),
  };
}
