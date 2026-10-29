/**
 The text of the artifact parse refusal a read of a recorded consistency
 value threw, for the cases that pin each reader's refusals whole.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { caught, } from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  ARTIFACT_SCHEMA_VERSION_V21,
  type ArtifactConsistencyCarry,
  assertCarriesRecount,
  type TwoLaneArtifactGeneration,
} from '../../dist/final/node/index.mjs';

//region Consistency refusal text

/**
 Text of the artifact parse refusal a read threw.

 @param read - read that must refuse

 @returns Refusal as text, class name first

 @throws {@link Error} when the read threw something other than an artifact parse refusal

 @example
 ```ts
 const text = refusalTextOf({ read: function readsABrokenBallot() { return parseConsistencyBallot({ value: 3, path: 'cat.ballot', },); }, },);
 ```
 */
export function refusalTextOf({ read, }: { readonly read: () => unknown; },): string {
  /**
   What the read threw.
   */
  const refusal = caught(read,);
  if (!(refusal instanceof ArtifactParseError)) {
    throw new Error(
      `Expected the read to throw an ArtifactParseError, but it threw ${
        Error.isError(refusal,) ? `${refusal.name}: ${refusal.message}` : typeof refusal
      }`,
    );
  }
  return String(refusal,);
}

/**
 Text of the refusal the recount of the artifact's carries raises for one carry, read under the path `cat.carries`.

 @param carry - carry as recorded

 @param generation - generation the section was written under, the one the pass writes by default

 @returns Refusal as text, class name first

 @throws {@link Error} when the recount threw something other than an artifact parse refusal, or nothing

 @example
 ```ts
 const text = recountRefusalOf({ carry, },);
 ```
 */
export function recountRefusalOf(
  {
    carry,
    generation = ARTIFACT_SCHEMA_VERSION_V21,
  }: {
    readonly carry: ArtifactConsistencyCarry;
    readonly generation?: TwoLaneArtifactGeneration;
  },
): string {
  return refusalTextOf({
    read: function recounts(): void {
      assertCarriesRecount({
        carries: [carry,],
        path: 'cat.carries',
        generation,
      },);
    },
  },);
}

//endregion Consistency refusal text
