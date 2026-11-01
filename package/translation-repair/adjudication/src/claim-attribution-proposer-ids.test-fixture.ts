import type { ClaimAttribution, } from '../dist/final/node/index.mjs';

//region Claim attribution proposer ids
// MODEL IDS OF EVERY PROPOSER BEHIND ONE ATTRIBUTED CLAIM, for cases
// counting which critics are credited with raising something.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The critic-stage-attribution and
// repair-translation tests kept their own copy of this projection; both now
// import it from here.

/**
 Model ids of every proposer behind one attributed claim.

 @param attribution - one claim's recorded attribution

 @returns Proposer model ids, one per proposer

 @example
 ```ts
 const ids = proposerIdsOf(attribution,);
 ```
 */
export function proposerIdsOf(attribution: ClaimAttribution,): readonly string[] {
  return attribution.proposers
    .map(function toModelId(proposer,) {
      return proposer.modelId;
    },);
}

//endregion Claim attribution proposer ids
