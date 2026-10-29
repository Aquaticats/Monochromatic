/**
 Tests for reading one settled artifact into the shape a draw samples from.

 THIS READER FEEDS THE PRECISION GATE, which is what makes its refusals worth
 more than its happy path. Every issue it hands back becomes a row a human
 grades, and the rate computed from those rows gets quoted as the pipeline's
 precision. An entry that joins the pool unverified does not produce a wrong
 number visibly; it produces a slightly short denominator, silently, and the
 rate over it still looks like a measurement.

 SO THE CASES ARE ALL REFUSALS, and one of them is about ORDER. An artifact can
 be both substituted and short at once, and which refusal comes out decides
 what an operator goes looking for. Provenance is checked first on purpose: a
 file that is not the entry the pool admitted has to be reported as the wrong
 file, not as a file with the wrong number of issues in it.

 THE HAPPY PATH READS AN INVENTED SOURCE. `loadEntry` bands an entry by
 reading its source at the pinned corpus commit, and a run reads the unlicensed
 clone; a unit suite that required the clone would pass on one machine and
 fail everywhere else, so the load cases hand `loadEntry` a reader returning a
 fixture page (ledger T8). Every guard this file tests runs BEFORE that read,
 so none of them needs one.

 FIXTURES ARE INVENTED AND CAT-THEMED, and every one is written into a
 throwaway directory that is removed when the case ends.

 @module
 */

import {
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  classifyBand,
  sourceBytesOf,
} from '@monochromatic-dev/translation-repair-house-rule';
import {
  ArtifactProvenanceError,
  type EligibleEntries,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  DrawReconcileError,
  loadEntry,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Draw entry load tests

/**
 Entry these fixtures describe.
 */
const ENTRY_ID = 'whiskers';

/**
 File that entry's artifact is written to.
 */
const ARTIFACT_NAME = `${ENTRY_ID}.json`;

/**
 Commit the pool recorded for this entry.
 */
const POOL_TIP = 'c4f9e1a7b2d6';

/**
 Built pipeline the pool recorded for this entry.
 */
const POOL_DIGEST = '8e3a0d5c17bf';

/**
 A different commit, standing in for a file the pool never admitted.
 */
const OTHER_TIP = 'fa27b9046e3d';

/**
 Clone and commit the draw is handed, invented, so a read at any other pin
 shows in what the reader was asked for.
 */
const DRAW_PIN = {
  cloneDir: '/cats/corpus/clone',
  commitSha: 'feedface'.repeat(5,),
} as const;

/**
 Builds one accepted issue record, in the shape the settled parser takes.

 @param issueId - identity of the adjudicated issue

 @returns Record shaped as an artifact carries one

 @example
 ```ts
 const record = acceptedRecord({ issueId: 'adjudicated/purr', },);
 ```
 */
function acceptedRecord(
  { issueId, }: { readonly issueId: string; },
): unknown {
  return {
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
 Builds the eligibility result the pool would hand this reader.

 @param tip - commit the pool recorded for this entry

 @returns Pool carrying that one entry

 @example
 ```ts
 const eligible = pooled({ tip: POOL_TIP, },);
 ```
 */
function pooled(
  { tip, }: { readonly tip: string; },
): EligibleEntries {
  return {
    entryIds: [ENTRY_ID,],
    excludedIds: [],
    malformedIds: [],
    tipByEntry: new Map([[
      ENTRY_ID,
      tip,
    ],],),
    digestByEntry: new Map([[
      ENTRY_ID,
      POOL_DIGEST,
    ],],),
    selection: {
      kind: 'single-generation',
      digest: POOL_DIGEST,
    },
    report: [],
  };
}

/**
 Writes one artifact into a throwaway directory and reads it back.

 @param artifact - whole artifact value, valid or not

 @param eligible - pool to check the bytes against

 @param readSource - source reader for a load that gets past every refusal;
 the refusal cases leave it out and get one that refuses any read, so a
 refusal that read the corpus first would say so rather than read the clone

 @returns What `loadEntry` made of it

 @throws Whatever `loadEntry` refuses with, which is the point of most cases

 @example
 ```ts
 await loadingFrom({ artifact, eligible: pooled({ tip: POOL_TIP, },), },);
 ```
 */
async function loadingFrom(
  {
    artifact,
    eligible,
    readSource,
  }: {
    readonly artifact: unknown;
    readonly eligible: EligibleEntries;
    readonly readSource?: Parameters<typeof loadEntry>[0]['readSource'];
  },
): Promise<Awaited<ReturnType<typeof loadEntry>>> {
  await using scratch = await scratchDir({ prefix: 'whiskers-draw-entry-', },);

  await writeFile(
    join(
      scratch.path,
      ARTIFACT_NAME,
    ),
    JSON.stringify(artifact,),
    'utf8',
  );
  return await loadEntry({
    artifactsDir: scratch.path,
    name: ARTIFACT_NAME,
    eligible,
    pin: DRAW_PIN,
    readSource: readSource ?? async function refuseCorpusRead(): Promise<string> {
      throw new Error('a refusal case read a corpus page before refusing',);
    },
  },);
}

/**
 Invented source standing in for the entry's zh page at the pin.
 */
const SOURCE_PAGE = '猫猫在窗台上睡觉。\n';

await describe({
  name: loadEntry.name,
  children: [
    it({
      name: 'REFUSES a file whose bytes are not the entry the pool admitted',
      fn: async () => {
        await expect(loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: OTHER_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 1,
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: pooled({ tip: POOL_TIP, },),
        },),).rejects.toThrow(ArtifactProvenanceError,);
      },
    },),
    it({
      name: 'REFUSES an artifact recording no accepted count, since nothing can confirm the population',
      fn: async () => {
        await expect(loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: POOL_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: pooled({ tip: POOL_TIP, },),
        },),).rejects.toThrow('records no numeric acceptedCount',);
      },
    },),
    it({
      name: 'REFUSES a count recorded as something other than a number',
      fn: async () => {
        // The reconcile used to run only when the field happened to be a
        // number, so the one shape it could not check was the shape most likely
        // to be wrong.
        await expect(loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: POOL_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 'one',
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: pooled({ tip: POOL_TIP, },),
        },),).rejects.toThrow('records no numeric acceptedCount',);
      },
    },),
    it({
      name: 'REFUSES a count that disagrees with what parsing found, naming both numbers',
      fn: async () => {
        await expect(loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: POOL_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 4,
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: pooled({ tip: POOL_TIP, },),
        },),).rejects.toThrow('acceptedCount 4 != parsed 1',);
      },
    },),
    it({
      name: 'REFUSES as DrawReconcileError, a marked class, so the boundary prints the entry and both '
        + 'counts instead of the class name alone',
      fn: async () => {
        await expect(loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: POOL_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 4,
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: pooled({ tip: POOL_TIP, },),
        },),).rejects.toThrow(DrawReconcileError,);
      },
    },),
    it({
      name: 'REPORTS a substituted file as substituted, not as one with the wrong count',
      fn: async () => {
        // Both wrong at once. Provenance runs first on purpose: an operator
        // told the count is short goes looking for missing issues, when the
        // file in front of them is not the entry they think it is.
        await expect(loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: OTHER_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 9,
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: pooled({ tip: POOL_TIP, },),
        },),).rejects.toThrow(ArtifactProvenanceError,);
      },
    },),
    it({
      name: 'BANDS THE ENTRY BY THE SOURCE IT READS and flattens every accepted issue into a candidate, carrying a '
        + 'recorded repair and leaving an unrecorded one absent (ledger T8)',
      fn: async () => {
        /**
         Corpus paths the reader was asked for.
         */
        const asked: string[] = [];
        const loaded = await loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: POOL_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 2,
            issues: [
              acceptedRecord({ issueId: 'adjudicated/purr', },),
              {
                ...(acceptedRecord({ issueId: 'adjudicated/hiss', },) as Record<string, unknown>),
                repairDisposition: 'shipped',
                repairRegions: [],
                refined: false,
              },
            ],
          },
          eligible: pooled({ tip: POOL_TIP, },),
          readSource: async function readSource({ relPath, },): Promise<string> {
            asked.push(relPath,);
            return SOURCE_PAGE;
          },
        },);
        expect(asked,).toStrictEqual([`people/${ENTRY_ID}/page.md`,],);
        expect(loaded.id,).toBe(ENTRY_ID,);
        expect(loaded.band,).toBe(classifyBand({ sourceBytes: sourceBytesOf({ text: SOURCE_PAGE, },), },),);
        expect(loaded.candidates.map(function carriesRepair(candidate,): boolean {
          return 'repair' in candidate;
        },),).toStrictEqual([false, true,],);
      },
    },),
    it({
      name: 'READS THE ENTRY\'S SOURCE AT THE PIN IT IS HANDED, never at one of its own',
      fn: async () => {
        /**
         Pins the reader was asked to read at, with the path asked for.
         */
        const asked: unknown[] = [];
        await loadingFrom({
          artifact: {
            id: ENTRY_ID,
            tip: POOL_TIP,
            pipelineDigest: POOL_DIGEST,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 1,
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: pooled({ tip: POOL_TIP, },),
          readSource: async function readSource({
            pin,
            relPath,
          },): Promise<string> {
            asked.push({
              pin,
              relPath,
            },);
            return SOURCE_PAGE;
          },
        },);
        expect(asked,).toEqual([
          {
            pin: DRAW_PIN,
            relPath: `people/${ENTRY_ID}/page.md`,
          },
        ],);
      },
    },),
    it({
      name: 'READS AN ARTIFACT WITH NO TIP OR DIGEST as observing neither, which a pool that recorded neither admits '
        + '(ledger T8)',
      fn: async () => {
        const loaded = await loadingFrom({
          artifact: {
            id: ENTRY_ID,
            corpusSha: 'sha/1',
            status: 'repaired',
            durationMs: 1,
            acceptedCount: 1,
            issues: [acceptedRecord({ issueId: 'adjudicated/purr', },),],
          },
          eligible: {
            ...pooled({ tip: POOL_TIP, },),
            tipByEntry: new Map(),
            digestByEntry: new Map(),
          },
          readSource: async function readSource(): Promise<string> {
            return SOURCE_PAGE;
          },
        },);
        expect(loaded.candidates,).toHaveLength(1,);
      },
    },),
  ],
},);

//endregion Draw entry load tests
