/**
 Tests for the report `score-attribution` prints over a runs directory.

 Each case writes its own runs directory, hands it to `printAttribution` and
 reads the whole of what was printed, the pool's own lines included, since
 those are part of what an operator sees.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { printAttribution, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  ATTRIBUTED_CAT_ARTIFACTS,
  MITTENS_ARTIFACT,
  writeScoreArtifacts,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/score-artifacts.test-fixture.ts';
import { builtPipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/built-pipeline-digest.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 What the pool prints as the pipeline that read it.
 */
const POOL_STAMP = await builtPipelineDigest({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-artifact-pool',), },);

/**
 The two critic rows of the attributed fixture, ordered by model id.
 */
const ATTRIBUTED_CRITIC_ROWS = [
  'hf:Qwen/Qwen3.8-27B                             1       1        1      1       1.00      1.00',
  'hf:openai/gpt-oss-120b                          1       2        3      2       2.00      2.00',
];

/**
 The table header over every critic row.
 */
const TABLE_HEADER = 'CRITIC                                      heard  raised  emitted   hits  raised/ch   hits/ch';

/**
 The closing note of a report over entries that carry attribution.
 */
const SUPPORT_NOTE = 'NOTE sole means an accepted issue rested on exactly one critic, which is '
  + 'legitimate: the reference run had gpt-oss-120b as the sole finder of a planted seed. selfRepeated '
  + 'means one critic emitted the same claim twice, which must never read as agreement. That '
  + 'distinction is what issue 65 asks about duplicates.';

await describe({
  name: printAttribution.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS the source, the population, the critic rows and the support counts of a run',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-attribution-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: ATTRIBUTED_CAT_ARTIFACTS,
        },);

        await printAttribution({ runsDir: scratch.path, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

        expect(printed.lines,).toStrictEqual([
          `SOURCE ${scratch.path}/artifacts`,
          `POOL read by pipeline ${POOL_STAMP}`,
          'POOL 2 entries across 1 pipeline generation',
          'POPULATION eligible=1 ineligible=1 chunks=1',
          `\n${TABLE_HEADER}`,
          ...ATTRIBUTED_CRITIC_ROWS,
          '\nSUPPORT sole=1 multi=1 selfRepeated=1 unattributed=0 partialJoin=0',
          SUPPORT_NOTE,
        ],);
      },
    },),

    it({
      name: 'SAYS NO ENTRY CARRIES ATTRIBUTION and stops, where every entry was settled before it existed',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-attribution-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: { 'Mittens.json': MITTENS_ARTIFACT, },
        },);

        await printAttribution({ runsDir: scratch.path, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

        expect(printed.lines,).toStrictEqual([
          `SOURCE ${scratch.path}/artifacts`,
          `POOL read by pipeline ${POOL_STAMP}`,
          'POOL 1 entry across 1 pipeline generation',
          'POPULATION eligible=0 ineligible=1 chunks=0',
          'NOTE no entry carries attribution yet. Entries settled before it existed record none, and they '
            + 'are excluded rather than counted as critics that raised nothing.',
        ],);
      },
    },),

    it({
      name: 'NAMES the artifact that could not be read before the population, and reports the rest',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-attribution-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            ...ATTRIBUTED_CAT_ARTIFACTS,
            'Biscuit.json': '{"id": "Bisc',
          },
        },);

        await printAttribution({ runsDir: scratch.path, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

        expect(printed.lines.slice(0, 8,),).toStrictEqual([
          `SOURCE ${scratch.path}/artifacts`,
          'POOL malformed Biscuit.json: could not read Biscuit.json as JSON (SyntaxError at byte 12)',
          `POOL read by pipeline ${POOL_STAMP}`,
          'POOL 2 entries across 1 pipeline generation',
          'POOL   1 artifact unreadable, passed through to be reported as malformed: Biscuit',
          'WARNING 1 artifact could not be read and is in NEITHER population this report counts, so every '
            + 'count is over the rest. Named rather than summarized, because a truncated artifact is a '
            + 'different problem from a malformed one:',
          '  Biscuit.json: could not read Biscuit.json as JSON (SyntaxError at byte 12)',
          'POPULATION eligible=1 ineligible=1 chunks=1',
        ],);
      },
    },),

    it({
      name: 'WARNS of a partial join and of an unattributed issue after the support note, each held out of the counts',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-attribution-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Pounce.json': {
              artifactSchemaVersion: 1,
              id: 'Pounce',
              chunkCritics: [
                {
                  chunkIndex: 0,
                  heardCriticIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
                  claimAttributions: [
                    {
                      claimId: 'issue/nap',
                      proposers: [{ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, emissionCount: 1, },],
                    },
                  ],
                },
              ],
              issues: [
                {
                  chunkIndex: 0,
                  issue: {
                    status: 'accepted',
                    claims: [
                      { claimId: 'issue/nap', },
                      { claimId: 'issue/ghost', },
                    ],
                  },
                },
                {
                  chunkIndex: 0,
                  issue: {
                    status: 'accepted',
                    claims: [{ claimId: 'issue/phantom', },],
                  },
                },
              ],
            },
          },
        },);

        await printAttribution({ runsDir: scratch.path, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

        expect(printed.lines.slice(3,),).toStrictEqual([
          'POPULATION eligible=1 ineligible=0 chunks=1',
          `\n${TABLE_HEADER}`,
          'hf:openai/gpt-oss-120b                          1       1        1      0       1.00      0.00',
          '\nSUPPORT sole=0 multi=0 selfRepeated=0 unattributed=1 partialJoin=1',
          SUPPORT_NOTE,
          'WARNING 1 accepted issue joined only SOME of its claims to attribution. It is held out of every '
            + 'other count in this report rather than counted as support, because the unattributed member may '
            + 'have come from a critic that got no credit. A nonzero number here is a defect in the join, not a '
            + 'fact about critics.',
          'WARNING 1 accepted issue on ELIGIBLE entries carries no attribution, meaning a claim id the index '
            + 'does not hold. That is a defect in the join, not a quiet critic.',
        ],);
      },
    },),

    it({
      name: 'REFUSES in its own words a runs directory with no artifacts directory, after naming the source',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-attribution-run-', },);

        /**
         Runs directory no pass ever wrote into.
         */
        const runsDir = join(
          scratch.path,
          'nowhere',
        );

        /**
         What the report raised.
         */
        const refusal = await rejectionOf(async function printsAbsentRun(): Promise<void> {
          await printAttribution({ runsDir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot list ${runsDir}/artifacts (ENOENT): name a runs directory that holds an `
            + '`artifacts` directory with TRANSLATION_REPAIR_RUNS_DIR',
        );
        expect(printed.lines,).toStrictEqual([`SOURCE ${runsDir}/artifacts`,],);
      },
    },),
  ],
},);
