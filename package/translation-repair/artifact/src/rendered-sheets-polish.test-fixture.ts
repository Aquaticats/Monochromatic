/**
 The notes the final polish shows its readers and the sheets of the judges of
 its writers' rewrites, rendered for `rendered-sheets.test-fixture.ts` (ledger
 B805): the claims the repair panel accepted, the note on a disputed archive
 rendering, and the two kinds of question the refine stage asks its judges.

 A judge of a text sees what its writer saw: the polish gate, the polish writer
 and these judges are all given both notes, each written by the builder
 production uses.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { buildCandidateSelectMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  acceptedRepairClaimsNote,
  archiveDisputeNote,
} from '@monochromatic-dev/translation-repair-archive';
import {
  buildRefineSelectionContext,
  type RefineStageMode,
} from '@monochromatic-dev/translation-repair-repair';
import {
  ARCHIVE,
  IDENTITY,
  joined,
  REFERENCES,
  REPAIR,
  type RenderedSheet,
  SOURCE,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

//region Polish notes

/**
 The claims the repair panel accepted for the invented slice, written by the
 builder production uses (ledger B731), which the polish gate, the polish
 writers and the judges of the writers' rewrites are all shown (ledger B805).
 */
export const REPAIR_CLAIMS: string = acceptedRepairClaimsNote({
  claims: ['fluency/grammar minor: the narration is told in the present',],
},);

/**
 The note on the disputed archive rendering of the invented slice, written by
 the builder production uses, which the polish gate, the polish writers and the
 judges of the writers' rewrites are all shown (ledger B805).
 */
export const ARCHIVE_DISPUTE: string = archiveDisputeNote({
  dispute: {
    sliceIndex: 0,
    standIn: REPAIR,
    standInEligible: true,
    acceptedClaims: ['accuracy/mistranslation major: the narration is told in the past, not the present',],
  },
},);

/**
 The kinds of question the refine stage asks its judges, which the polish
 round asks both of.
 */
const POLISH_MODES: readonly {
  readonly name: string;
  readonly mode: RefineStageMode;
}[] = [
  {
    name: 'polish refine slate',
    mode: { kind: 'comparative', },
  },
  {
    name: 'polish refine objection slate',
    mode: {
      kind: 'objection-correction',
      groups: [{
        origin: 'consolidation gate',
        objections: ['The tense shifts mid-paragraph.',],
      },],
    },
  },
];

/**
 The sheets of the judges who choose among the polish writers' rewrites, one
 per kind of question the polish round asks.

 @returns One entry per sheet

 @example
 ```ts
 const sheets = polishSlateSheets();
 ```
 */
export function polishSlateSheets(): readonly RenderedSheet[] {
  return POLISH_MODES.map(function polishSlate(
    {
      name,
      mode,
    },
  ): RenderedSheet {
    return {
      name,
      text: joined({
        messages: buildCandidateSelectMessages({
          ...buildRefineSelectionContext({
            mode,
            sourceText: SOURCE,
            repairedText: ARCHIVE,
            referenceContext: REFERENCES,
            identityContext: IDENTITY,
            acceptedRepairClaimsNote: REPAIR_CLAIMS,
            archiveDisputeNote: ARCHIVE_DISPUTE,
          },),
          rendered: [
            ARCHIVE,
            REPAIR,
          ],
          sourceText: SOURCE,
        },),
      },),
    };
  },);
}

//endregion Polish notes
