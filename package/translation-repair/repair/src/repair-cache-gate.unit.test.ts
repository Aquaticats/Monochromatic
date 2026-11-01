/**
 Tests for why a settled repair slice may not be cached, said as reasons the
 driver's warn line prints.

 A slice no critic was heard on, and one where a stage fell short of quorum,
 must not be kept, since a resumed run would read either as "examined and
 found nothing to change". Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stageQuorumUnmetFinding, } from '@monochromatic-dev/translation-repair-stage';
import {
  cacheRefusalLine,
  cacheRefusalsOf,
  frontMatterRepairOutcome,
} from '../dist/final/node/index.mjs';

/**
 Settled outcome the cases vary, heard by three critics and carrying no
 finding of its own.
 */
const HEARD = {
  ...frontMatterRepairOutcome({
    sliceIndex: 4,
    targetText: 'The kitten naps on the windowsill.',
  },),
  heardCritics: 3,
  findings: [],
  silentStages: [],
};

/**
 Settled outcome whose repair patch won, so the slice's text changed, while the
 introduced-defect probe closed short of quorum (zheermao21 slice 3 had this
 shape: `changedSliceIndices` held it and the log still said it shipped
 unchanged).
 */
const CHANGED_BUT_PROBE_SHORT = {
  ...HEARD,
  changed: true,
  repairedText: 'The kitten naps on the sunny windowsill.',
  findings: [stageQuorumUnmetFinding({ shortfall: 'introduced-defect-probe 1/3', },),],
};

await describe({
  name: '',
  children: [
    describe({
      name: cacheRefusalsOf.name,
      children: [
        it({
          name: 'REFUSES a slice no critic was heard on, whatever else its findings say',
          fn: async () => {
            expect(cacheRefusalsOf({
              outcome: {
                ...HEARD,
                heardCritics: 0,
                findings: [stageQuorumUnmetFinding({ shortfall: 'checker 1/3', },),],
              },
            },),).toStrictEqual(['no critic was heard',],);
          },
        },),
        it({
          name: 'NAMES every stage that fell short of quorum, in order, and not a stage that only lost '
            + 'a voice',
          fn: async () => {
            expect(cacheRefusalsOf({
              outcome: {
                ...HEARD,
                findings: [
                  stageQuorumUnmetFinding({ shortfall: 'panel 1/3', },),
                  'stage-voice-lost (editor tabby-large)',
                  stageQuorumUnmetFinding({ shortfall: 'checker 1/3', },),
                ],
              },
            },),).toStrictEqual([
              'a stage heard fewer than quorum (stage-quorum-unmet (panel 1/3))',
              'a stage heard fewer than quorum (stage-quorum-unmet (checker 1/3))',
            ],);
          },
        },),
        it({
          name: 'REFUSES A SLICE WHOSE PANEL HEARD NOBODY when the only finding about it was relocated, as the '
            + 'XingZ6018 slice 85 record was: 12 needs-human issues, heard critics, and five findings that began '
            + '`panel-packet (cluster/...): ` so no prefix reader saw them',
          fn: async () => {
            expect(cacheRefusalsOf({
              outcome: {
                ...HEARD,
                heardCritics: 4,
                findings: [
                  'panel-packet (cluster/140d): stage-quorum-unmet (panel 0/6)',
                  'panel-packet (cluster/445d): stage-quorum-unmet (panel 0/5)',
                ],
                silentStages: ['panel 0/6 (cluster/140d)', 'panel 0/5 (cluster/445d)',],
              },
            },),).toStrictEqual([
              'a stage heard fewer than quorum (panel 0/6 (cluster/140d))',
              'a stage heard fewer than quorum (panel 0/5 (cluster/445d))',
            ],);
          },
        },),
        it({
          name: 'ADMITS a slice every stage heard to quorum',
          fn: async () => {
            expect(cacheRefusalsOf({ outcome: HEARD, },),).toStrictEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: cacheRefusalLine.name,
      children: [
        it({
          name: 'DOES NOT SAY A CHANGED SLICE SHIPS UNCHANGED when a probe closed short of quorum',
          fn: async () => {
            /**
             Warn line a changed, quorum-unmet settlement prints.
             */
            const line = cacheRefusalLine({
              sliceIndex: 3,
              refusals: cacheRefusalsOf({ outcome: CHANGED_BUT_PROBE_SHORT, },),
              changed: CHANGED_BUT_PROBE_SHORT.changed,
            },);
            expect(line,).not.toContain('ships unchanged',);
            expect(line,).toContain('stage-quorum-unmet (introduced-defect-probe 1/3)',);
            expect(line,).toContain('NOT cached',);
            expect(line,).toContain('the slice\'s text changed',);
          },
        },),
        it({
          name: 'SAYS AN UNCHANGED SLICE DID NOT CHANGE, and still that nothing is cached',
          fn: async () => {
            /**
             Warn line a settlement no critic was heard on prints.
             */
            const line = cacheRefusalLine({
              sliceIndex: 4,
              refusals: cacheRefusalsOf({ outcome: { ...HEARD, heardCritics: 0, }, },),
              changed: false,
            },);
            expect(line,).toContain('chunk 4: no critic was heard',);
            expect(line,).toContain('NOT cached',);
            expect(line,).toContain('the slice\'s text did not change',);
            expect(line,).not.toContain('the slice\'s text changed',);
          },
        },),
      ],
    },),
  ],
},);
