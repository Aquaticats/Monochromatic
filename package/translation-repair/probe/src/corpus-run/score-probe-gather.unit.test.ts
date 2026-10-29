/**
 Tests for the reader that gathers every settled artifact's probe readings.

 Each case writes its own artifacts directory and reads what the gather made
 of it: the readings grouped by artifact, the issue index, the rewritten
 slices, the roster coverage and the record counts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { gatherProbeReadings, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { writeScoreArtifacts, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/score-artifacts.test-fixture.ts';
import {
  probeArtifactText,
  probedRecord,
  probeRegion,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/score-probe-artifacts.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 A shipped record of an issue with one flagged region.

 @param issueId - issue the record is about

 @param refined - whether the naturalness lane rewrote the slice

 @returns Record for a fixture artifact

 @example
 ```ts
 const record = flaggedRecord({ issueId: 'adjudicated/nap', refined: false, },);
 ```
 */
function flaggedRecord(
  {
    issueId,
    refined,
  }: {
    readonly issueId: string;
    readonly refined: boolean;
  },
): Readonly<Record<string, unknown>> {
  return probedRecord({
    issueId,
    refined,
    regions: [
      probeRegion({
        envelopeId: `envelope/${issueId}`,
        issueIds: [issueId,],
        corroborated: 2,
      },),
    ],
  },);
}

await describe({
  name: gatherProbeReadings.name,
  concurrency: 1,
  children: [
    it({
      name: 'GATHERS the readings of each artifact in code-point order of its file name, with the issue index, the '
        + 'rewritten slices and the record counts',
      fn: async (ctx) => {
        using _printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-gather-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Whiskers.json': probeArtifactText({
              entryId: 'Whiskers',
              issues: [
                flaggedRecord({ issueId: 'adjudicated/nap', refined: false, },),
                flaggedRecord({ issueId: 'adjudicated/chase', refined: true, },),
              ],
              findings: [
                'editor-candidates (1/3 heard)',
                'refine-candidates (0/1 heard)',
              ],
            },),
            'Mittens.json': probeArtifactText({
              entryId: 'Mittens',
              issues: [
                {
                  sliceIndex: 0,
                  repairDisposition: 'shipped',
                  resolved: true,
                  refined: false,
                  issue: { issueId: 'adjudicated/purr', },
                },
              ],
              findings: [],
            },),
          },
        },);

        /**
         What the gather made of the directory.
         */
        const gathered = await gatherProbeReadings({ artifactsDir: `${scratch.path}/artifacts`, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

        expect(gathered.entries,).toBe(2,);
        expect(gathered.readings.map(function toName({ entryId, },): string {
          return entryId;
        },),).toStrictEqual(['Mittens.json', 'Whiskers.json',],);
        expect(gathered.readings.map(function toCount({ readings, },): number {
          return readings.length;
        },),).toStrictEqual([0, 2,],);
        expect([...gathered.byIssueId.keys(),].toSorted(),).toStrictEqual([
          'adjudicated/chase',
          'adjudicated/nap',
        ],);
        expect([...gathered.refinedIssueIds,],).toStrictEqual(['adjudicated/chase',],);
        expect(gathered.repairShippedRecords,).toBe(3,);
        expect(gathered.repairUnprobedRecords,).toBe(1,);
        expect(gathered.entriesWithRewrites,).toBe(1,);
        expect(gathered.editorRoster,).toStrictEqual({
          offered: 1,
          degraded: 1,
          silent: 0,
        },);
        expect(gathered.refineRoster,).toStrictEqual({
          offered: 1,
          degraded: 1,
          silent: 1,
        },);
        expect(gathered.refinementReadings.map(function toCount({ readings, },): number {
          return readings.length;
        },),).toStrictEqual([0, 0,],);
      },
    },),

    it({
      name: 'REFUSES an artifact whose probe field is present and malformed, naming the file',
      fn: async (ctx) => {
        using _printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-gather-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Whiskers.json': probeArtifactText({
              entryId: 'Whiskers',
              issues: [
                {
                  sliceIndex: 0,
                  repairDisposition: 'shipped',
                  resolved: true,
                  refined: false,
                  issue: { issueId: 'adjudicated/nap', },
                  introducedDefects: 'none',
                },
              ],
              findings: [],
            },),
          },
        },);

        /**
         What gathering raised.
         */
        const refusal = await rejectionOf(async function gathersMalformedProbe(): Promise<void> {
          await gatherProbeReadings({ artifactsDir: `${scratch.path}/artifacts`, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);
        },);

        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          'ArtifactParseError: artifact parse failed at Whiskers.json.lanes.repair.result.issues[0].introducedDefects: '
            + 'expected an object.',
        );
      },
    },),
  ],
},);
