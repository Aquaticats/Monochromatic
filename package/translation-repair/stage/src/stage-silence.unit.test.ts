/**
 Tests for the one spelling of a silent stage and the question the caches ask.

 A settlement reached while a stage heard nobody was once cached, because the
 quorum gather reports shortfall as a finding and no cache read it. The cases
 hold the producer's spelling and the consumers' reading together.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  refinementIsResumable,
  silentStageLabel,
  silentStagesOf,
  silentStagesOfRecord,
  silentStagesOfSettlement,
  STAGE_QUORUM_UNMET_PREFIX,
  stageQuorumUnmetFinding,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: stageQuorumUnmetFinding.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPELLS the finding the way the caches read it, prefix first and shortfall closed',
          fn: async () => {
            /** Finding a critic stage two short of six would leave. */
            const finding = stageQuorumUnmetFinding({ shortfall: 'critic 2/6', },);
            expect(finding,).toBe('stage-quorum-unmet (critic 2/6)',);
            expect(finding.startsWith(STAGE_QUORUM_UNMET_PREFIX,),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: silentStagesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES nothing for findings that carry no shortfall, lost voices included',
          fn: async () => {
            expect(silentStagesOf({ findings: [], },),).toStrictEqual([],);
            expect(silentStagesOf({
              findings: [
                'stage-voice-lost (critic hf:zai-org/GLM-5.3-Flash)',
                'stage-short (critic 5/6)',
              ],
            },),).toStrictEqual([],);
          },
        },),
        it({
          name: 'NAMES every silent stage and nothing else, in order, wherever the finding sits',
          fn: async () => {
            expect(silentStagesOf({
              findings: [
                stageQuorumUnmetFinding({ shortfall: 'critic 0/6', },),
                'stage-voice-lost (checker qwen3.8-max)',
                stageQuorumUnmetFinding({ shortfall: 'checker 1/3', },),
              ],
            },),).toStrictEqual([
              'stage-quorum-unmet (critic 0/6)',
              'stage-quorum-unmet (checker 1/3)',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: silentStageLabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPELLS heard against asked, and where the stage ran when it ran once for each of several places',
          fn: async () => {
            expect(silentStageLabel({ stage: 'critic', heard: 2, asked: 6, },),).toBe('critic 2/6',);
            expect(silentStageLabel({
              stage: 'panel',
              heard: 0,
              asked: 6,
              where: 'cluster/abc',
            },),).toBe('panel 0/6 (cluster/abc)',);
          },
        },),
      ],
    },),

    describe({
      name: silentStagesOfRecord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A SILENT STAGE THE RECORD CARRIES AS DATA even when every finding about it was relocated, '
            + 'which the XingZ6018 slice 85 record had: its panel findings began `panel-packet (cluster/...): `',
          fn: async () => {
            /**
             The relocated finding the XingZ6018 record held, which the prefix reader misses.
             */
            const relocated = 'panel-packet (cluster/140d): stage-quorum-unmet (panel 0/6)';
            expect(silentStagesOf({ findings: [relocated,], },),).toStrictEqual([],);
            expect(silentStagesOfRecord({
              record: {
                findings: [relocated,],
                silentStages: ['panel 0/6 (cluster/140d)',],
              },
            },),).toStrictEqual(['panel 0/6 (cluster/140d)',],);
          },
        },),
        it({
          name: 'NAMES THE DATA FIRST, then the stages that left an unrelocated finding, and nothing for a stage that heard',
          fn: async () => {
            expect(silentStagesOfRecord({
              record: {
                findings: [stageQuorumUnmetFinding({ shortfall: 'checker 1/3', },), 'stage-voice-lost (editor x)',],
                silentStages: ['panel 0/6 (cluster/a)',],
              },
            },),).toStrictEqual(['panel 0/6 (cluster/a)', 'stage-quorum-unmet (checker 1/3)',],);
            expect(silentStagesOfRecord({ record: { findings: [], silentStages: [], }, },),).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: silentStagesOfSettlement.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the silence the embedded outcome carries and the silence the refinement\'s own findings carry, '
            + 'in that order',
          fn: async () => {
            expect(silentStagesOfSettlement({
              settlement: {
                outcome: { findings: [], silentStages: ['panel 0/6 (cluster/a)',], },
                findings: [stageQuorumUnmetFinding({ shortfall: 'rewriter 0/3', },),],
              },
            },),).toStrictEqual(['panel 0/6 (cluster/a)', 'stage-quorum-unmet (rewriter 0/3)',],);
          },
        },),
      ],
    },),

    describe({
      name: refinementIsResumable.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ANSWERS true only when neither the outcome nor the refinement names a silent stage',
          fn: async () => {
            /**
             An outcome that heard every stage.
             */
            const heard = { findings: [], silentStages: [], };
            expect(refinementIsResumable({ settlement: { outcome: heard, findings: [], }, },),).toBe(true,);
            expect(refinementIsResumable({
              settlement: { outcome: { ...heard, silentStages: ['panel 0/6 (cluster/a)',], }, findings: [], },
            },),).toBe(false,);
            expect(refinementIsResumable({
              settlement: {
                outcome: { ...heard, findings: [stageQuorumUnmetFinding({ shortfall: 'critic 0/6', },),], },
                findings: [],
              },
            },),).toBe(false,);
            expect(refinementIsResumable({
              settlement: { outcome: heard, findings: [stageQuorumUnmetFinding({ shortfall: 'rewriter 0/3', },),], },
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
