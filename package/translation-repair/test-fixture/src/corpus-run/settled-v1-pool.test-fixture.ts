/**
 Settled version 1 artifacts written into a throwaway runs directory, for the
 cases of the draw command that read a pool.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

//region Settled version 1 pool
// An artifact is a regular file named `<entry id>.json` under the `artifacts`
// directory of a runs directory, recording the commit and the built pipeline
// its pass ran under, the accepted count its pass tallied, and its issues.

/**
 Hex digits in a git commit id.
 */
const COMMIT_HEX_LENGTH = 40;

/**
 Hex digits in a tree digest.
 */
const DIGEST_HEX_LENGTH = 64;

/**
 Commit every fixture artifact records as the tip its pass started under.
 */
const FIXTURE_TIP: string = 'a'.repeat(COMMIT_HEX_LENGTH,);

/**
 Digest every fixture artifact records as the pipeline its pass executed.
 */
const FIXTURE_DIGEST: string = `sha256-tree-v1:${'c'.repeat(DIGEST_HEX_LENGTH,)}`;

/**
 Builds one accepted issue record, in the shape the settled parser takes.

 @param issueId - identity of the adjudicated issue

 @param repairRecorded - whether the record carries the repair its pass recorded, which an artifact written before repair recording lacks

 @returns Record shaped as an artifact carries one

 @example
 ```ts
 const record = acceptedIssueRecord({ issueId: 'adjudicated/purr', repairRecorded: false, },);
 ```
 */
function acceptedIssueRecord(
  {
    issueId,
    repairRecorded,
  }: {
    readonly issueId: string;
    readonly repairRecorded: boolean;
  },
): unknown {
  return {
    ...(repairRecorded
      ? {
        repairDisposition: 'shipped',
        repairRegions: [],
        refined: false,
      }
      : {}),
    sliceIndex: 0,
    resolved: false,
    issue: {
      issueId,
      status: 'accepted',
      severity: 'minor',
      claims: [
        {
          claimId: 'claim/whisker',
          claim: {
            category: 'accuracy/omission',
            severity: 'minor',
            summary: 'A purr is dropped from the greeting.',
            spans: [
              {
                side: 'source',
                nodeId: 'block/0',
                quotedText: 'the cat purred',
              },
              {
                side: 'target',
                nodeId: 'block/0',
                quotedText: '',
              },
            ],
          },
        },
      ],
      tallies: {},
    },
  };
}

/**
 Writes one settled artifact whose issues are all accepted.

 @param runsDir - throwaway runs directory

 @param entryId - corpus entry the artifact is for, which names its file

 @param issueIds - identities of the accepted issues it records

 @param repairRecorded - whether its records carry the repair their pass recorded

 @example
 ```ts
 await writeSettledArtifact({ runsDir, entryId: 'mittens', issueIds: ['adjudicated/purr',], repairRecorded: false, },);
 ```
 */
export async function writeSettledArtifact(
  {
    runsDir,
    entryId,
    issueIds,
    repairRecorded,
  }: {
    readonly runsDir: string;
    readonly entryId: string;
    readonly issueIds: readonly string[];
    readonly repairRecorded: boolean;
  },
): Promise<void> {
  /**
   Directory every settled artifact of the run sits in.
   */
  const artifactsDir = join(
    runsDir,
    'artifacts',
  );
  await mkdir(
    artifactsDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      artifactsDir,
      `${entryId}.json`,
    ),
    JSON.stringify({
      id: entryId,
      tip: FIXTURE_TIP,
      pipelineDigest: FIXTURE_DIGEST,
      corpusSha: 'sha/1',
      status: 'repaired',
      durationMs: 1,
      acceptedCount: issueIds.length,
      issues: issueIds.map(function recordOf(issueId,): unknown {
        return acceptedIssueRecord({
          issueId,
          repairRecorded,
        },);
      },),
    },),
    'utf8',
  );
}

//endregion Settled version 1 pool
