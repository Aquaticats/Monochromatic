/**
 Tests for the lines the probe relabel runner prints.

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
  relabelCaseLines,
  relabelClaimLines,
  relabelCounts,
  relabelGathered,
  relabelNotes,
  relabelRebuilt,
} from '../../dist/final/node/index.mjs';
import { napCase, } from '../relabel-case.test-fixture.ts';

/**
 Tally whose counts all differ, so a count printed under the wrong name shows.
 */
const COUNTED_TALLY: RegionDefectTally = {
  envelopeId: 'envelope/nap-tabby',
  issueIds: ['adjudicated/nap',],
  corroborated: 1,
  removalCorroborated: 2,
  contradicted: 3,
  unanchored: 4,
  preExisting: 5,
  noneFound: 6,
  uncertain: 7,
  claims: [],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: relabelRebuilt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS one damaged region in the singular',
          fn: async () => {
            expect(relabelRebuilt({ count: 1, },),).toBe('RELABEL rebuilt 1 distinct damaged region',);
          },
        },),

        it({
          name: 'SAYS several damaged regions, and none, in the plural',
          fn: async () => {
            expect(relabelRebuilt({ count: 3, },),).toBe('RELABEL rebuilt 3 distinct damaged regions',);
            expect(relabelRebuilt({ count: 0, },),).toBe('RELABEL rebuilt 0 distinct damaged regions',);
          },
        },),
      ],
    },),

    describe({
      name: relabelGathered.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS one control region in the singular',
          fn: async () => {
            expect(relabelGathered({ count: 1, },),).toBe('RELABEL gathered 1 unflagged control region',);
          },
        },),

        it({
          name: 'SAYS several control regions, and none, in the plural',
          fn: async () => {
            expect(relabelGathered({ count: 2, },),).toBe('RELABEL gathered 2 unflagged control regions',);
            expect(relabelGathered({ count: 0, },),).toBe('RELABEL gathered 0 unflagged control regions',);
          },
        },),
      ],
    },),

    describe({
      name: relabelCaseLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HEADS a damaged case with its positions joined by plus, the issues it served and both lengths',
          fn: async () => {
            expect(relabelCaseLines({
              relabelCase: napCase({
                entryId: 'tabby',
                positions: [
                  2,
                  7,
                ],
                recorded: 'corroborated=0 removal=1',
              },),
            },),).toEqual([
              'RELABEL tabby positions=2+7 issuesServed=1 beforeChars=53 afterChars=15',
              '  run-recorded  corroborated=0 removal=1',
            ],);
          },
        },),

        it({
          name: 'HEADS a control case with its positions empty and the run-recorded text it carries',
          fn: async () => {
            expect(relabelCaseLines({
              relabelCase: napCase({
                entryId: 'whiskers',
                positions: [],
                recorded: 'not probed',
              },),
            },),).toEqual([
              'RELABEL whiskers positions= issuesServed=1 beforeChars=53 afterChars=15',
              '  run-recorded  not probed',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: relabelClaimLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS no line when the tally holds no claim',
          fn: async () => {
            expect(relabelClaimLines({ tally: COUNTED_TALLY, },),).toEqual([],);
          },
        },),

        it({
          name: 'PRINTS one line per claim naming the prober, what the screen made of it and its category',
          fn: async () => {
            expect(relabelClaimLines({
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
                    evidence: '',
                    omittedText: '',
                    reason: 'stiff',
                    admissibility: 'unanchored',
                  },
                ],
              },
            },),).toEqual([
              '    cat-house/tabbyscribe-2 removal-corroborated (accuracy/omission)',
              '    cat-house/mouser-mini unanchored (style/awkward-phrasing)',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: relabelCounts.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS the probers heard over those configured and every count under its own name',
          fn: async () => {
            /**
             Report of a probe that heard two of three probers.
             */
            const report: IntroducedDefectReport = {
              regions: [COUNTED_TALLY,],
              heardProbers: 2,
              configuredProbers: 3,
              findings: [],
            };

            expect(relabelCounts({
              report,
              tally: COUNTED_TALLY,
            },),).toBe(
              'heard=2/3 corroborated=1 removal=2 contradicted=3 unanchored=4 preExisting=5 none=6 uncertain=7',
            );
          },
        },),
      ],
    },),

    describe({
      name: relabelNotes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the disclosure production sends and the other one in the first note and leaves the second unchanged',
          fn: async () => {
            /**
             Notes printed when production sends the withheld list.
             */
            const withheld = relabelNotes({
              production: 'withheld',
              other: 'rendered',
            },);

            /**
             Notes printed when production sends the rendered list.
             */
            const rendered = relabelNotes({
              production: 'rendered',
              other: 'withheld',
            },);

            expect(withheld,).toEqual([
              'NOTE production sends issues-withheld. Compare it against issues-rendered on each region: '
                + 'a region that reports damage under one prompt only is one the other prompt talks the probe '
                + 'out of. Compare it against issues-absent: a region that reports damage only with no list '
                + 'known is one whose claims the screen dismisses as restating a prior issue. A region dark '
                + 'under all three exonerates the label and indicts the difficulty of the judgement.',
              'NOTE a control line prints positions= empty. Read the issues-absent arm across controls '
                + 'against the issues-absent arm across damaged regions: similar rates mean the unlabelled '
                + 'prober is re-reporting pre-existing defects and the damaged result proves nothing, and a '
                + 'much lower control rate means the issue list is suppressing real detections.',
            ],);
            expect(rendered[0]?.startsWith(
              'NOTE production sends issues-rendered. Compare it against issues-withheld on each region: ',
            ),).toBe(true,);
            expect(rendered.slice(1,),).toEqual(withheld.slice(1,),);
          },
        },),
      ],
    },),
  ],
},);
