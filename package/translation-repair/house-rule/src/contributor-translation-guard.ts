import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { droppedContributorNameForms, } from './contributor-name-authority.ts';

//region Contributor translation guard

/**
 Privacy-safe complete-form contributor authority finding.
 */
export const CONTRIBUTOR_AUTHORITY_FINDING = 'Target-authoritative contributor identity from PAGE AS IT STANDS is missing or changed.';

/**
 Finds privacy-safe target contributor authority defect in one candidate.

 @param texts - target-authoritative archive then proposed replacement

 @returns Empty when authority survives,
 otherwise one non-identifying correction finding

 @example
 ```ts
 const findings = contributorAuthorityFindings({ texts: [pageText, candidateText,], },);
 ```
 */
export function contributorAuthorityFindings(
  { texts, }: { readonly texts: readonly string[]; },
): readonly string[] {
  /**
   Target-authoritative archive slice.
   */
  const pageText = nonNullishOrThrow(texts[0],);
  /**
   Proposed replacement slice.
   */
  const candidateText = nonNullishOrThrow(texts[1],);
  /**
   Target-authoritative forms candidate lost.
   */
  const dropped = droppedContributorNameForms({
    archiveText: pageText,
    candidateText,
  },);
  return (dropped.length === 0)
    ? []
    : [CONTRIBUTOR_AUTHORITY_FINDING,];
}

//endregion Contributor translation guard
