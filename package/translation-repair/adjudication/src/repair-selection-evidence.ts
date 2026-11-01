import type { SelectEvidence, } from './candidate-select-wire.ts';
import { citedReferenceEvidence, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { declaredNamesEvidence, } from './declared-names-evidence.ts';

/**
 Supplies factual support without making neighboring passages additional translation obligations.

 THE DECLARED NAMES AND THE REFERENCES ARE HERE TOO (ledger S14): the critic
 filed and the panel accepted issues with both on their sheets, and the
 judges choosing among the repairs of those issues had neither, so a repair
 respelling a declared name or stripping a detail a cited page states could
 win.

 @param neighbouringSourceText - local source evidence for interpreting current claims

 @param documentSourceText - same-entry original checking claims outside the local window

 @param identityContext - declared names and handles from both front matters

 @param referenceContext - what the pages the original cites say

 @returns Additional evidence blocks in the measured order

 @example
 ```ts
 const evidence = repairSelectionSourceEvidence({ neighbouringSourceText, documentSourceText });
 ```
 */
export function repairSelectionSourceEvidence(
  {
    neighbouringSourceText,
    documentSourceText,
    identityContext,
    referenceContext,
  }: {
    readonly neighbouringSourceText?: string;
    readonly documentSourceText?: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
  },
): readonly SelectEvidence[] {
  return [
    ...((neighbouringSourceText === undefined) || (neighbouringSourceText === '')
      ? []
      : [{
        label: 'NEARBY ORIGINAL, source evidence for the current passage rather than additional coverage',
        text: neighbouringSourceText,
      },]),
    ...((documentSourceText === undefined) || (documentSourceText === '')
      ? []
      : [{
        label: 'FULL ORIGINAL DOCUMENT, factual evidence only: use it to check current claims without requiring the rest of this document to be translated in this passage',
        text: documentSourceText,
      },]),
    ...declaredNamesEvidence((identityContext === undefined) ? {} : { identityContext, },),
    ...citedReferenceEvidence((referenceContext === undefined) ? {} : { referenceContext, },),
  ];
}
