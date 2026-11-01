/**
 Tests for the lines the probe sensitivity runner prints.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  IntroducedDefectReport,
  RegionDefectTally,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  type SensitivityArm,
  sensitivityArmLines,
  sensitivityNotes,
  sensitivityOpening,
} from '../../dist/final/node/index.mjs';

/**
 Arm under the withheld list, asking about a nap edit.
 */
const NAP_ARM: SensitivityArm = {
  region: {
    envelopeId: 'envelope/nap',
    issueIds: [],
    before: 'The cat is doing the sleeping, and she wakes at dusk.',
    editorAfter: 'The cat sleeps.',
  },
  expectation: 'damage-omission',
  list: 'withheld',
  issue: 'prior',
  issues: [],
  disclosure: 'withheld',
  editKind: 'accuracy-repair',
  baselineText: 'The cat is doing the sleeping, and she wakes at dusk.',
};

/**
 Tally whose counts all differ, so a count printed under the wrong name shows.
 */
const COUNTED_TALLY: RegionDefectTally = {
  envelopeId: 'envelope/nap',
  issueIds: [],
  corroborated: 1,
  removalCorroborated: 2,
  contradicted: 3,
  unanchored: 4,
  preExisting: 5,
  noneFound: 6,
  uncertain: 7,
  claims: [],
};

/**
 Report of a probe that heard two of three probers.
 */
const HEARD_REPORT: IntroducedDefectReport = {
  regions: [COUNTED_TALLY,],
  heardProbers: 2,
  configuredProbers: 3,
  findings: [],
};

/**
 What the arm line reads for the counted tally and the heard report.
 */
const COUNTED_LINE = 'SENSITIVITY envelope/nap list=withheld issue=prior expected=damage-omission heard=2/3 '
  + 'corroborated=1 removal=2 contradicted=3 unanchored=4 preExisting=5 noneFound=6 uncertain=7';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sensitivityOpening.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS one arm follows in the singular',
          fn: async () => {
            expect(sensitivityOpening({
              production: 'withheld',
              armCount: 1,
            },),).toBe('SENSITIVITY production sends list=withheld; 1 arm follows',);
          },
        },),

        it({
          name: 'SAYS several arms follow in the plural',
          fn: async () => {
            expect(sensitivityOpening({
              production: 'rendered',
              armCount: 3,
            },),).toBe('SENSITIVITY production sends list=rendered; 3 arms follow',);
          },
        },),

        it({
          name: 'SAYS no arm in the plural, zero taking the many form',
          fn: async () => {
            expect(sensitivityOpening({
              production: 'none',
              armCount: 0,
            },),).toBe('SENSITIVITY production sends list=none; 0 arms follow',);
          },
        },),
      ],
    },),

    describe({
      name: sensitivityArmLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS the one tally line with every count under its own name when no claim was raised',
          fn: async () => {
            expect(sensitivityArmLines({
              arm: NAP_ARM,
              report: HEARD_REPORT,
              tally: COUNTED_TALLY,
            },),).toEqual([COUNTED_LINE,],);
          },
        },),

        it({
          name: 'PRINTS one indented claim line per claim after the tally line, in the order the claims came',
          fn: async () => {
            expect(sensitivityArmLines({
              arm: NAP_ARM,
              report: HEARD_REPORT,
              tally: {
                ...COUNTED_TALLY,
                claims: [
                  {
                    modelId: 'cat-house/tabbyscribe-2',
                    category: 'accuracy/omission',
                    severity: 'major',
                    evidence: '',
                    omittedText: 'she wakes at dusk',
                    reason: 'the clause is gone',
                    admissibility: 'removal-corroborated',
                  },
                  {
                    modelId: 'cat-house/mouser-mini',
                    category: 'style/awkward-phrasing',
                    severity: 'minor',
                    evidence: 'The cat sleeps.',
                    omittedText: '',
                    reason: 'stiff',
                    admissibility: 'contradicted',
                  },
                ],
              },
            },),).toEqual([
              COUNTED_LINE,
              '  claim removal-corroborated (accuracy/omission/major)',
              '  claim contradicted (style/awkward-phrasing/minor)',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: sensitivityNotes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CLOSES the first note on the list production sends and leaves the other two unchanged by it',
          fn: async () => {
            /**
             Notes printed when production sends the rendered list.
             */
            const rendered = sensitivityNotes({ production: 'rendered', },);

            /**
             Notes printed when production sends the withheld list.
             */
            const withheld = sensitivityNotes({ production: 'withheld', },);

            expect(rendered,).toEqual([
              'NOTE compare each accuracy region\'s three lines. list=none against list=withheld differ only '
                + 'in the deterministic screen, which dismisses a claim restating the prior issue; '
                + 'list=withheld against list=rendered differ only in the prompt, and rendered is the prompt '
                + 'production abandoned because it silenced the stage. A region that reports damage with '
                + 'list=none and goes quiet with list=withheld is one whose claims merely restate the prior '
                + 'issue; one that goes quiet only with list=rendered is one the rendered prompt silences, '
                + 'and its zeros would mean nothing in a run that rendered. Production sends list=rendered.',
              'NOTE the refinement/* lines test the naturalness framing under production\'s list. Its '
                + 'control is refinement/clean: a claim there means the probe reads mere rephrasing as '
                + 'damage, which would flag every refinement the lane ever ships.',
              'NOTE the deletion/* lines vary what the issue list SAYS, under both lists that carry one. '
                + 'With list=rendered the prober reads the label; with list=withheld only the screen does. '
                + 'deletion/unlabelled and deletion/mislabelled delete identical source-supported text; a '
                + 'gap between them under list=rendered measures how far a false accepted issue can talk '
                + 'the probe out of seeing real damage, and under list=withheld how far the screen dismisses '
                + 'a real claim as restating it. deletion/licensed is the negative control, where silence '
                + 'is correct.',
            ],);
            expect(withheld[0]?.endsWith('Production sends list=withheld.',),).toBe(true,);
            expect(withheld.slice(1,),).toEqual(rendered.slice(1,),);
          },
        },),
      ],
    },),
  ],
},);
