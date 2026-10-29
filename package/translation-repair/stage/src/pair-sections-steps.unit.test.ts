/**
 Tests for turning a roster's section pairing into alignment steps, and for
 the insertion boundary two paired neighbours pin between them.

 THIS IS WHERE THE PROVEN ANCHOR COMES FROM. Measured over the pinned corpus,
 the deterministic aligner never proves one: all 11 of its unpaired source
 sections come back `ambiguous`, every anchor comes back `may-pair`, and the
 insertion path emits nothing at all. A pairing that names the sections either
 side of a gap says where the gap is.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { sectionPairingToSteps, } from '../dist/final/node/index.mjs';

/**
 Four original sections, headed in Chinese as the corpus heads them.
 */
const SOURCE_HEADINGS = [
  '## 第一节',
  '## 第二节',
  '## 第三节',
  '## 第四节',
];

/**
 Four translation sections, headed in English, sharing no token with any of
 them. This is the corpus condition: `headingAffinity` is token overlap and
 reads 0.00 across the whole grid, which is why a model is asked at all.
 */
const TARGET_HEADINGS = [
  '## Naps',
  '## Birds',
  '## Boxes',
  '## Sunbeams',
];

/**
 Builds steps over the four-by-four fixture.

 @param pairs - correspondences a roster agreed on

 @returns Steps in the order the aligner emits them

 @example
 ```ts
 const steps = stepsFor([{ source: 0, target: 0, },],);
 ```
 */
function stepsFor(pairs: readonly { readonly source: number; readonly target: number; }[],) {
  return sectionPairingToSteps({
    pairs,
    sourceHeadings: SOURCE_HEADINGS,
    targetHeadings: TARGET_HEADINGS,
  },);
}

/**
 Reads one step's anchor, which only a `source-only` step carries.

 @param steps - what the conversion produced

 @param sourceIndex - original section to read

 @returns That section's anchor

 @throws Error when the step is not an unpaired original, since a case asking
 for an anchor on a paired section is asking the wrong question

 @example
 ```ts
 const anchor = anchorAt({ steps, sourceIndex: 2, },);
 ```
 */
function anchorAt(
  {
    steps,
    sourceIndex,
  }: {
    readonly steps: ReturnType<typeof stepsFor>;
    readonly sourceIndex: number;
  },
) {
  /**
   Decision made about that original section.
   */
  const step = steps.find(function names(candidate,): boolean {
    return (candidate.kind === 'source-only') && (candidate.sourceIndex === sourceIndex);
  },);
  if (step?.kind !== 'source-only')
    throw new Error(`original section ${String(sourceIndex,)} is not unpaired`,);
  return step.anchor;
}

await describe({
  name: sectionPairingToSteps.name,
  children: [
    it({
      name: 'EMITS one step per original then every unclaimed translation, matching what the '
        + 'deterministic aligner emits, so nothing downstream learns a model was involved',
      fn: async () => {
        const steps = stepsFor([
          {
            source: 0,
            target: 0,
          },
          {
            source: 1,
            target: 1,
          },
          {
            source: 2,
            target: 2,
          },
          {
            source: 3,
            target: 3,
          },
        ],);
        expect(steps.length,).toBe(SOURCE_HEADINGS.length,);
        expect(steps.every(function isPaired(step,): boolean {
          return step.kind === 'paired';
        },),).toBe(true,);
      },
    },),

    it({
      name: 'REPORTS the affinity the two headings actually share rather than inventing one, which '
        + 'on this corpus is 0.00 and is the measurement that explains why a model was asked',
      fn: async () => {
        const [first,] = stepsFor([{
          source: 0,
          target: 0,
        },],);
        if (first?.kind !== 'paired')
          throw new Error('first step should be a pairing',);
        expect(first.affinity,).toBe(0,);
      },
    },),

    it({
      name: 'CARRIES every translation nobody claimed as its own step, after the originals, so a '
        + 'section the archive added on its own is still accounted for',
      fn: async () => {
        const steps = stepsFor([{
          source: 0,
          target: 0,
        },],);

        /** Translation sections the conversion emitted on their own. */
        const targetOnly = steps.filter(function isTargetOnly(step,): boolean {
          return step.kind === 'target-only';
        },);
        expect(targetOnly.length,).toBe(3,);

        // AFTER EVERY ORIGINAL DECISION, which is the order the deterministic
        // aligner emits and which `placeInsertions` walks assuming.
        expect(steps.findIndex(function isTargetOnly(step,): boolean {
          return step.kind === 'target-only';
        },),).toBe(SOURCE_HEADINGS.length,);
      },
    },),

    it({
      name: 'NAMES the reason `roster-unpaired` on both sides, which is neither of either scorer '
        + 'reasons: those describe a table of optimal paths and this describes a reading',
      fn: async () => {
        const steps = stepsFor([{
          source: 0,
          target: 0,
        },],);
        expect(steps.every(function namesTheRoster(step,): boolean {
          return (step.kind === 'paired') || (step.reason === 'roster-unpaired');
        },),).toBe(true,);
      },
    },),

    it({
      name: 'PROVES the boundary when the paired sections either side of a gap take ADJACENT '
        + 'translations, since there is then exactly one place the missing rendering can go',
      fn: async () => {
        const steps = stepsFor([
          {
            source: 0,
            target: 0,
          },
          {
            source: 2,
            target: 1,
          },
          {
            source: 3,
            target: 2,
          },
        ],);
        expect(anchorAt({
          steps,
          sourceIndex: 1,
        },),).toEqual({
          kind: 'proven',
          beforeTargetIndex: 1,
        },);
      },
    },),

    it({
      name: 'REFUSES to choose when unclaimed translations lie between the neighbours, listing '
        + 'every boundary rather than picking one, because putting real content under the wrong '
        + 'heading is a misfiling nothing downstream can undo',
      fn: async () => {
        const steps = stepsFor([
          {
            source: 0,
            target: 0,
          },
          {
            source: 2,
            target: 3,
          },
        ],);
        expect(anchorAt({
          steps,
          sourceIndex: 1,
        },),).toEqual({
          kind: 'several-boundaries',
          boundaries: [
            1,
            2,
            3,
          ],
        },);
      },
    },),

    it({
      name: 'PROVES the boundary at the very FRONT for an original nothing precedes, rather than '
        + 'defaulting to the start of the file where front matter may sit',
      fn: async () => {
        const steps = stepsFor([
          {
            source: 1,
            target: 0,
          },
          {
            source: 2,
            target: 1,
          },
        ],);
        expect(anchorAt({
          steps,
          sourceIndex: 0,
        },),).toEqual({
          kind: 'proven',
          beforeTargetIndex: 0,
        },);
      },
    },),

    it({
      name: 'PROVES the boundary PAST the last translation for an original nothing follows and '
        + 'nothing is left unclaimed after, which is where a page whose closing sections were '
        + 'never rendered needs them written. Measured live on `XingZ60`, whose last two original '
        + 'sections anchor there',
      fn: async () => {
        const steps = stepsFor([
          {
            source: 0,
            target: 1,
          },
          {
            source: 1,
            target: 2,
          },
          {
            source: 2,
            target: 3,
          },
        ],);
        expect(anchorAt({
          steps,
          sourceIndex: 3,
        },),).toEqual({
          kind: 'proven',
          beforeTargetIndex: TARGET_HEADINGS.length,
        },);
      },
    },),

    it({
      name: 'REFUSES to choose at the END when a translation after the last paired one went '
        + 'unclaimed, since the missing rendering could sit either side of it',
      fn: async () => {
        const steps = stepsFor([
          {
            source: 0,
            target: 0,
          },
          {
            source: 1,
            target: 1,
          },
          {
            source: 2,
            target: 2,
          },
        ],);
        expect(anchorAt({
          steps,
          sourceIndex: 3,
        },),).toEqual({
          kind: 'several-boundaries',
          boundaries: [
            3,
            4,
          ],
        },);
      },
    },),

    it({
      name: 'REFUSES every boundary when the pairing is EMPTY, so a roster that agreed on nothing '
        + 'cannot be read as a roster that placed everything at the top of the page',
      fn: async () => {
        const steps = stepsFor([],);
        expect(anchorAt({
          steps,
          sourceIndex: 0,
        },).kind,).toBe('several-boundaries',);
        expect(steps.filter(function isSourceOnly(step,): boolean {
          return step.kind === 'source-only';
        },).length,).toBe(SOURCE_HEADINGS.length,);
        expect(steps.filter(function isTargetOnly(step,): boolean {
          return step.kind === 'target-only';
        },).length,).toBe(TARGET_HEADINGS.length,);
      },
    },),
  ],
},);
