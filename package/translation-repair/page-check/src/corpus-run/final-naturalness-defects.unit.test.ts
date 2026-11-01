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

import type { SettledArtifact, } from '@monochromatic-dev/translation-repair-artifact';
import { finalNaturalnessDefects, } from '../../dist/final/node/index.mjs';

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
      name: 'STILL THROWS A FAULT THE GUARD RAISES OTHER THAN ITS REFUSAL: an artifact whose contest never ran',
      fn: async () => {
        expect(() => finalNaturalnessDefects({
          artifact: artifactCarrying({ polish: undefined, selectionKind: 'not-run', },),
        },),).toThrow('unreachable',);
      },
    },),
  ],
},);
