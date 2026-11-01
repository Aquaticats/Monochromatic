/**
 Tests for the naturalness shortfall of a settled artifact, reported as a defect and no longer a stop (ledger B1207).

 `assertFinalNaturalnessComplete` threw before the page was written, and the entry ended with no page and no artifact.
 The owner's rule is "no matter what, we must ship", so the refusal is collected as the defect `final-naturalness`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NaturalnessCompletenessError, } from '@monochromatic-dev/translation-repair-adjudication';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import type { SettledArtifact, } from '@monochromatic-dev/translation-repair-artifact';
import {
  assertFinalNaturalnessComplete,
  finalNaturalnessDefects,
} from '../../dist/final/node/index.mjs';

/**
 Wraps one polish record in the minimal artifact the completeness guard reads.

 @param polish - polish record of the one consolidated body slice, absent for none

 @param selectionKind - how the artifact records its lane contest

 @returns Artifact whose consolidated body slice carries that polish

 @example
 ```ts
 const artifact = artifactCarrying({ polish: undefined, selectionKind: 'contested', },);
 ```
 */
function artifactCarrying(
  {
    polish,
    selectionKind,
  }: {
    readonly polish: unknown;
    readonly selectionKind: string;
  },
): SettledArtifact {
  // oxlint-disable-next-line unicorn/prefer-structured-clone -- JSON semantics are the point: persistence writes JSON, so an undefined-valued key must drop.
  return JSON.parse(JSON.stringify({
    laneSelection: {
      kind: selectionKind,
      slices: [{
        sliceIndex: 1,
        verdict: { kind: 'lane-won', lane: 'translate', },
        ballots: [],
        usable: 2,
      },],
    },
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 1,
        terminal: 'gate-kept-standing',
        shipped: { kind: 'unchanged', },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
        polish,
      },],
    },
  },),) as SettledArtifact;
}

await describe({
  name: finalNaturalnessDefects.name,
  concurrency: 1,
  children: [
    it({
      name: 'REPORTS A BODY SLICE WITHOUT A POLISH RECORD AS THE DEFECT final-naturalness, naming the slice',
      fn: async () => {
        expect(finalNaturalnessDefects({
          artifact: artifactCarrying({ polish: undefined, selectionKind: 'contested', },),
        },),).toEqual([{
          check: 'final-naturalness',
          message: 'slice 1 did not meet absolute naturalness floor',
        },],);
      },
    },),

    it({
      name: 'REPORTS NOTHING FOR AN UNENDORSED STANDING THAT SHIPPED WITH ITS FINDING, the control',
      fn: async () => {
        expect(finalNaturalnessDefects({
          artifact: artifactCarrying({
            polish: { kind: 'not-run', reason: 'unsafe-baseline', },
            selectionKind: 'contested',
          },),
        },),).toEqual([],);
      },
    },),

    it({
      name: 'REPORTS A REVIEW THE ARTIFACT READER CANNOT PARSE AS NO APPROVAL, the defect final-naturalness naming the '
        + 'slice (B1241), where the parser\'s ArtifactParseError left the guard, stopped the entry on every attempt and '
        + 'shipped it interim for ever',
      fn: async () => {
        /**
         What the guard made of a settled polish whose review is no review.
         */
        const defects = finalNaturalnessDefects({
          artifact: artifactCarrying({
            polish: {
              kind: 'settled',
              text: 'The cat naps.',
              review: { not: 'a review', },
            },
            selectionKind: 'contested',
          },),
        },);
        expect(defects,).toEqual([{
          check: 'final-naturalness',
          message: 'slice 1 did not meet absolute naturalness floor',
        },],);
      },
    },),

    it({
      name: 'KEEPS THE PARSER\'S REFUSAL AS THE CAUSE of the naturalness refusal it raises for an unreadable review, so '
        + 'the log can name what the parser said',
      fn: async () => {
        /**
         What the guard threw.
         */
        let caught: unknown;
        try {
          assertFinalNaturalnessComplete({
            artifact: artifactCarrying({
              polish: {
                kind: 'settled',
                text: 'The cat naps.',
                review: { not: 'a review', },
              },
              selectionKind: 'contested',
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NaturalnessCompletenessError,);
        expect(caught instanceof Error ? caught.cause : undefined,).toBeInstanceOf(ArtifactParseError,);
      },
    },),

    it({
      name: 'STILL THROWS A FAULT THE GUARD RAISES OTHER THAN ITS REFUSAL: an artifact whose contest never ran',
      fn: async () => {
        expect(() => finalNaturalnessDefects({
          artifact: artifactCarrying({ polish: undefined, selectionKind: 'not-run', },),
        },),).toThrow('unreachable',);
      },
    },),
  ],
},);
