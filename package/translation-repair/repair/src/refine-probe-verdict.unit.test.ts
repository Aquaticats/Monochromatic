/**
 Tests for what the damage probe's report decides for a naturalness rewrite.

 The verdict has three exits: an admitted claim rolls the rewrite back, a
 round short of its quorum rolls it back under its own wording, and a round
 with neither lets the rewrite ship. The settler's cases reach these only
 through a whole settlement; these hold each exit, each plural, and the order
 between the first two against the function directly.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stageQuorumUnmetFinding, } from '@monochromatic-dev/translation-repair-stage';
import type {
  IntroducedDefectReport,
  RegionDefectTally,
} from '@monochromatic-dev/translation-repair-adjudication';
import { refineProbeVerdict, } from '../dist/final/node/index.mjs';

/**
 Tally of one replaced region carrying the given admitted claims and no other
 outcome.

 @param added - claims of added damage the screen bore out

 @param dropped - claims of dropped content the screen bore out

 @returns Region tally with every other count at zero

 @example
 ```ts
 const tally = regionTally({ added: 1, dropped: 0, },);
 ```
 */
function regionTally(
  {
    added,
    dropped,
  }: {
    readonly added: number;
    readonly dropped: number;
  },
): RegionDefectTally {
  return {
    envelopeId: 'paragraph/sunbeam',
    issueIds: [],
    corroborated: added,
    removalCorroborated: dropped,
    contradicted: 0,
    unanchored: 0,
    preExisting: 0,
    noneFound: 0,
    uncertain: 0,
    claims: [],
  };
}

/**
 Report of a probe that heard its probers, with the given regions and
 findings.

 @param regions - screened tally per replaced region

 @param heardProbers - probers whose reply arrived and validated

 @param configuredProbers - probers asked

 @param findings - wire irregularities and the stage's own findings

 @returns Probe report

 @example
 ```ts
 const report = probeReport({ regions: [], heardProbers: 3, configuredProbers: 3, findings: [], },);
 ```
 */
function probeReport(
  {
    regions,
    heardProbers,
    configuredProbers,
    findings,
  }: {
    readonly regions: readonly RegionDefectTally[];
    readonly heardProbers: number;
    readonly configuredProbers: number;
    readonly findings: readonly string[];
  },
): IntroducedDefectReport {
  return {
    regions,
    heardProbers,
    configuredProbers,
    findings,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: refineProbeVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS a rewrite whose probe heard its quorum and admitted no claim, whatever regions it read',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [regionTally({ added: 0, dropped: 0, },),],
                heardProbers: 3,
                configuredProbers: 3,
                findings: [],
              },),
            },),).toEqual({ kind: 'kept', },);
          },
        },),

        it({
          name: 'KEEPS a rewrite the probe read no region of, since a report with no region admits nothing',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [],
                heardProbers: 2,
                configuredProbers: 3,
                findings: [],
              },),
            },),).toEqual({ kind: 'kept', },);
          },
        },),

        it({
          name: 'KEEPS a rewrite whose bench was short but heard its own quorum, since the stage left no '
            + 'quorum-unmet finding and its other findings are not silence',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [],
                heardProbers: 2,
                configuredProbers: 5,
                findings: [
                  'stage-short-bench (introduced-defect-probe reachable 2 of 5, quorum 2)',
                  'stage-voice-lost (introduced-defect-probe cat-prober)',
                ],
              },),
            },),).toEqual({ kind: 'kept', },);
          },
        },),

        it({
          name: 'ROLLS BACK on one admitted added-damage claim, naming the zero removal claims in the plural',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [regionTally({ added: 1, dropped: 0, },),],
                heardProbers: 3,
                configuredProbers: 3,
                findings: [],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-rolled-back-by-probe (1 added-damage and 0 removal claims admitted against the rewrite)',
            },);
          },
        },),

        it({
          name: 'ROLLS BACK on one admitted removal claim, naming it in the singular',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [regionTally({ added: 0, dropped: 1, },),],
                heardProbers: 3,
                configuredProbers: 3,
                findings: [],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-rolled-back-by-probe (0 added-damage and 1 removal claim admitted against the rewrite)',
            },);
          },
        },),

        it({
          name: 'ROLLS BACK on two admitted removal claims, naming them in the plural',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [regionTally({ added: 0, dropped: 2, },),],
                heardProbers: 3,
                configuredProbers: 3,
                findings: [],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-rolled-back-by-probe (0 added-damage and 2 removal claims admitted against the rewrite)',
            },);
          },
        },),

        it({
          name: 'SUMS admitted claims across every region the report holds, in both directions',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [
                  regionTally({ added: 1, dropped: 0, },),
                  regionTally({ added: 2, dropped: 1, },),
                  regionTally({ added: 0, dropped: 0, },),
                ],
                heardProbers: 3,
                configuredProbers: 3,
                findings: [],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-rolled-back-by-probe (3 added-damage and 1 removal claim admitted against the rewrite)',
            },);
          },
        },),

        it({
          name: 'ROLLS BACK a round that heard no prober of three as unheard, naming the probers in the plural',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [],
                heardProbers: 0,
                configuredProbers: 3,
                findings: [stageQuorumUnmetFinding({ shortfall: 'introduced-defect-probe 0/3', },),],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-probe-unheard (0 of 3 probers heard)',
            },);
          },
        },),

        it({
          name: 'ROLLS BACK a round that heard no prober of one as unheard, naming the prober in the singular',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [],
                heardProbers: 0,
                configuredProbers: 1,
                findings: [stageQuorumUnmetFinding({ shortfall: 'introduced-defect-probe 0/1', },),],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-probe-unheard (0 of 1 prober heard)',
            },);
          },
        },),

        it({
          name: 'ROLLS BACK a round short of its quorum on a short bench, whatever else its findings hold',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [regionTally({ added: 0, dropped: 0, },),],
                heardProbers: 1,
                configuredProbers: 5,
                findings: [
                  'stage-short-bench (introduced-defect-probe reachable 2 of 5, quorum 2)',
                  stageQuorumUnmetFinding({ shortfall: 'introduced-defect-probe 1/2', },),
                ],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-probe-unheard (1 of 5 probers heard)',
            },);
          },
        },),

        it({
          name: 'NAMES the admitted claim and not the unheard round when a round short of its quorum also admitted '
            + 'one, since the settler attaches no report to a rollback and the claim would reach no finding',
          fn: async () => {
            expect(refineProbeVerdict({
              report: probeReport({
                regions: [regionTally({ added: 1, dropped: 1, },),],
                heardProbers: 1,
                configuredProbers: 3,
                findings: [stageQuorumUnmetFinding({ shortfall: 'introduced-defect-probe 1/3', },),],
              },),
            },),).toEqual({
              kind: 'rolled-back',
              finding: 'refine-rolled-back-by-probe (1 added-damage and 1 removal claim admitted against the rewrite)',
            },);
          },
        },),
      ],
    },),
  ],
},);
