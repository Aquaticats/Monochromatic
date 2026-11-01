/**
 Tests for reading one region tally out of a probe artifact.

 THE COUNTS ARE DERIVED AND THE PARSER SAYS SO. The screen computes every
 declared count from the claim list, so the two are one fact written twice,
 and a tally where they disagree is a malformed artifact rather than a
 different answer. The reader refuses it, because downstream the CLAIMS report
 sums the counts while the majority rule reads the claims, and a disagreement
 would make one region report a corroboration and flag nothing.

 ATTRIBUTION ONLY. A claim carries evidence and reasons quoted from corpus
 text; the tally this reader returns carries who said it and how the screen
 judged it, and nothing else, which the control case pins by handing it a
 claim with those fields and reading back the keys.

 `readArtifactProbe` was the only caller and reached this through whole probe
 files, so no case named these refusals before.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseRegionTally, } from '../dist/final/node/index.mjs';

/**
 Path every case reports its refusals under.
 */
const AT = 'whiskers.issues[0].regions[0]';

/**
 Tally whose counts match its two claims, one upheld and one refuted, with
 the text fields a real claim carries on the first.
 */
const MATCHED_TALLY = {
  envelopeId: 'envelope/1',
  issueIds: [
    'issue/1',
    'issue/2',
  ],
  corroborated: 1,
  removalCorroborated: 0,
  contradicted: 1,
  unanchored: 0,
  preExisting: 0,
  noneFound: 2,
  uncertain: 0,
  claims: [
    {
      modelId: 'hf:cat/Cat-A',
      admissibility: 'corroborated',
      evidence: 'the cat naps on the sill',
      reason: 'the line about the sill was dropped',
    },
    {
      modelId: 'hf:cat/Cat-B',
      admissibility: 'contradicted',
    },
  ],
};

/**
 Reads a tally expected to refuse, returning what it said.

 @param value - tally as an artifact would carry it

 @returns Refusal text, or an empty string where it read

 @example
 ```ts
 const said = refusalOf({ value: 7, },);
 ```
 */
function refusalOf({ value, }: { readonly value: unknown; },): string {
  try {
    parseRegionTally({
      value,
      path: AT,
    },);
    return '';
  }
  catch (error) {
    return String(error,);
  }
}

await describe({
  name: parseRegionTally.name,
  children: [
    it({
      name: 'READS a tally whose counts match its claims, carrying each claim '
        + 'as attribution only, which is the control every refusal departs from',
      fn: async () => {
        expect(parseRegionTally({
          value: MATCHED_TALLY,
          path: AT,
        },),)
          .toEqual({
            envelopeId: 'envelope/1',
            issueIds: [
              'issue/1',
              'issue/2',
            ],
            corroborated: 1,
            removalCorroborated: 0,
            contradicted: 1,
            unanchored: 0,
            preExisting: 0,
            noneFound: 2,
            uncertain: 0,
            claims: [
              {
                modelId: 'hf:cat/Cat-A',
                admissibility: 'corroborated',
              },
              {
                modelId: 'hf:cat/Cat-B',
                admissibility: 'contradicted',
              },
            ],
          },);
      },
    },),

    it({
      name: 'READS AN ABSENT preExisting COUNT AS ZERO, since the outcome did '
        + 'not exist when the settled artifacts were written and refusing the '
        + 'field would make every earlier one unreadable',
      fn: async () => {
        /**
         Tally from before the outcome existed.
         */
        const { preExisting: _dropped, ...earlier } = MATCHED_TALLY;

        expect(parseRegionTally({
          value: earlier,
          path: AT,
        },).preExisting,)
          .toBe(0,);
      },
    },),

    it({
      name: 'REFUSES A COUNT THAT DISAGREES WITH THE CLAIM LIST, naming the '
        + 'field, since the CLAIMS report sums the counts while the majority '
        + 'rule reads the claims',
      fn: async () => {
        /**
         What the refusal says.
         */
        const said = refusalOf({
          value: {
            ...MATCHED_TALLY,
            corroborated: 2,
          },
        },);

        expect(said.includes(`${AT}.corroborated`,),).toBe(true,);
        expect(said.includes('to match its claim list',),).toBe(true,);
      },
    },),

    it({
      name: 'HOLDS A DECLARED preExisting COUNT TO THE CLAIMS TOO, so the '
        + 'tolerance for its absence is not a tolerance for its value',
      fn: async () => {
        /**
         What the refusal says.
         */
        const said = refusalOf({
          value: {
            ...MATCHED_TALLY,
            preExisting: 1,
          },
        },);

        expect(said.includes(`${AT}.preExisting`,),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES AN ADMISSIBILITY THE SCREEN CANNOT HAVE WRITTEN rather '
        + 'than counting it as non-upholding, which would silently zero the '
        + 'corroboration every region reports',
      fn: async () => {
        /**
         What the refusal says.
         */
        const said = refusalOf({
          value: {
            ...MATCHED_TALLY,
            claims: [
              {
                modelId: 'hf:cat/Cat-A',
                admissibility: 'plausible',
              },
            ],
          },
        },);

        expect(said.includes('one of corroborated',),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES A CLAIM NAMING NO PROBER, since the majority rule counts '
        + 'distinct probers and a claim without one cannot be counted',
      fn: async () => {
        /**
         What the refusal says.
         */
        const said = refusalOf({
          value: {
            ...MATCHED_TALLY,
            corroborated: 0,
            claims: [{ admissibility: 'contradicted', },],
          },
        },);

        expect(said.includes('modelId',),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES A TALLY THAT IS NOT A RECORD, under the path it was asked '
        + 'to read',
      fn: async () => {
        /**
         What the refusal says.
         */
        const said = refusalOf({ value: 7, },);

        expect(said,).not.toBe('',);
        expect(said.includes(AT,),).toBe(true,);
      },
    },),
  ],
},);
