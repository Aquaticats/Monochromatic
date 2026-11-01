/**
 Tests the two page guards as steps of the publisher: which checks they name, which refusals mark their failure, what
 each reads, and where the regions a carried passage lost are written (B1181).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CarriedEvidenceLostError,
  PageFootnoteIntegrityError,
  pageGuardSteps,
} from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 One archive slice both lanes left as it was.
 */
const ROWS: readonly SourceShippedRow[] = [{
  incumbentText: 'The cat sleeps in warm sunlight.\n',
  sourceText: '猫在阳光下睡觉。\n',
},];

/**
 A carried passage whose evidence is a region the archive slice does not hold.
 */
const LOST_PASSAGE = {
  position: 1,
  sliceIndex: 3,
  sourceText: '猫在水边等。',
  evidence: ['the cat waits by the water',],
};

/**
 Builds the two steps over the one-slice archive, with the page and the carried passages a case chooses.

 @param pageText - page as it ships
 @param carried - carried passages the page must still carry the evidence of
 @param messages - lines the step logger collects

 @returns Footnote step, then carried-evidence step

 @example
 ```ts
 const [footnote, carriedEvidence,] = stepsOver({ pageText: ROWS[0].incumbentText, carried: [], messages: [], },);
 ```
 */
function stepsOver(
  {
    pageText,
    carried,
    messages,
  }: {
    readonly pageText: string;
    readonly carried: Parameters<typeof pageGuardSteps>[0]['carried'];
    readonly messages: string[];
  },
): ReturnType<typeof pageGuardSteps> {
  return pageGuardSteps({
    entryId: 'BookshopCat',
    pageText,
    archiveText: sourceShippedArchive({ rows: ROWS, },),
    artifact: sourceShippedArtifact({ rows: ROWS, },),
    slices: sourceShippedSlices({ rows: ROWS, },),
    carried,
    l: capturingLogger({ messages, },),
  },);
}

await describe({
  name: pageGuardSteps.name,
  children: [
    it({
      name: 'LISTS THE FOOTNOTE INTEGRITY STEP, THEN THE CARRIED-EVIDENCE STEP, each marked by its own refusal',
      fn: async () => {
        const steps = stepsOver({
          pageText: ROWS[0]?.incumbentText ?? '',
          carried: [],
          messages: [],
        },);
        expect(steps.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['page-footnote-integrity', 'carried-evidence',],);
        expect(steps.map(function refusalOf({ refusal, },): unknown {
          return refusal;
        },),).toEqual([PageFootnoteIntegrityError, CarriedEvidenceLostError,],);
      },
    },),
    it({
      name: 'RUNS THE FOOTNOTE STEP OVER THE PAGE TEXT IT IS HANDED, not over the artifact, and raises its refusal for a '
        + 'definition the archive did not carry',
      fn: async () => {
        const [footnote,] = stepsOver({
          pageText: `${ROWS[0]?.incumbentText ?? ''}\n[^3]: That is, a sunbeam.\n`,
          carried: [],
          messages: [],
        },);
        const refusal = caught(function act(): unknown {
          return footnote?.run();
        },);
        expect(refusal,).toBeInstanceOf(PageFootnoteIntegrityError,);
      },
    },),
    it({
      name: 'WRITES THE REGIONS A CARRIED PASSAGE LOST TO THE LOG, one line each, and raises its refusal, since the '
        + 'defect\'s own message names slices and counts only',
      fn: async () => {
        /**
         Lines the step logger collected.
         */
        const messages: string[] = [];
        const [, carriedEvidence,] = stepsOver({
          pageText: ROWS[0]?.incumbentText ?? '',
          carried: [LOST_PASSAGE,],
          messages,
        },);
        const refusal = caught(function act(): unknown {
          return carriedEvidence?.run();
        },);
        expect(refusal,).toBeInstanceOf(CarriedEvidenceLostError,);
        expect(messages.some(function namesTheRegion(line,): boolean {
          return line.includes('entry BookshopCat: carried-insertion-evidence-lost slice 3',)
            && line.includes('the cat waits by the water',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'WRITES NOTHING TO THE LOG for a page that keeps the evidence of its carried passage',
      fn: async () => {
        /**
         Lines the step logger collected.
         */
        const messages: string[] = [];
        const [, carriedEvidence,] = stepsOver({
          pageText: ROWS[0]?.incumbentText ?? '',
          carried: [{
            ...LOST_PASSAGE,
            evidence: ['cat sleeps in warm sunlight',],
          },],
          messages,
        },);
        expect(() => carriedEvidence?.run(),).not.toThrow();
        expect(messages,).toEqual([],);
      },
    },),
  ],
},);
