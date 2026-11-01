/**
 Tests for reading the one tally a probe over a single region leaves.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  EMPTY_INTRODUCED_DEFECT_REPORT,
  type IntroducedDefectReport,
} from '@monochromatic-dev/translation-repair-adjudication';
import { singleRegionTally, } from '../../dist/final/node/index.mjs';

/**
 Report of a probe that heard two of three probers about one nap region.
 */
const NAP_REPORT: IntroducedDefectReport = {
  regions: [
    {
      envelopeId: 'envelope/nap',
      issueIds: ['adjudicated/nap',],
      corroborated: 1,
      removalCorroborated: 0,
      contradicted: 0,
      unanchored: 0,
      preExisting: 0,
      noneFound: 1,
      uncertain: 0,
      claims: [],
    },
  ],
  heardProbers: 2,
  configuredProbers: 3,
  findings: [],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: singleRegionTally.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS the one tally the report holds',
          fn: async () => {
            expect(singleRegionTally({ report: NAP_REPORT, },),).toEqual(NAP_REPORT.regions[0],);
          },
        },),

        it({
          name: 'THROWS an unreachable error naming the state when the report holds no tally',
          fn: async () => {
            /**
             What reading a tally from the report of a probe that never ran raised.
             */
            const refusal = caught(function act(): unknown {
              return singleRegionTally({ report: EMPTY_INTRODUCED_DEFECT_REPORT, },);
            },);

            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(
              'Error: unreachable: the probe answered with no region tally although it was given one region, '
                + 'and it screens every region it is given',
            );
          },
        },),
      ],
    },),
  ],
},);
