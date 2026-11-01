import type { AdjudicatedIssue, } from '../dist/final/node/index.mjs';

//region Accepted cat issue
// ONE ACCEPTED ISSUE WITH NO CLAIMS, since nothing under test in these cases
// reads them.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The repair-chunk-verdict, repair-edit-
// stages and repair-record tests kept their own copy of this builder; all
// now import it from here.

/**
 Builds one accepted issue with no claims.

 @param issueId - adjudicated identity

 @returns Issue the report carries

 @example
 ```ts
 const issue = catIssue({ issueId: 'adjudicated/nap', },);
 ```
 */
export function catIssue({ issueId, }: { readonly issueId: string; },): AdjudicatedIssue {
  return {
    issueId,
    status: 'accepted' as const,
    severity: 'major' as const,
    claims: [],
    tallies: {},
  };
}

//endregion Accepted cat issue
