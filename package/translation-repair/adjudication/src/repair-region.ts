import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Repair region
// The text the accuracy stage actually WROTE, kept region-shaped rather than
// issue-shaped.
//
// Detection and repair are separate questions and only the first was ever
// measured. The grading sheet asks whether an accepted issue is a real defect;
// nothing recorded the replacement, so a correct detection carrying a poor
// repair scored as an unqualified success. Round two shows the gap directly:
// four of thirty-seven true positives came back with the grader asking whether
// there was a better way, and all four counted as successes.
//
// The checker stage does not substitute for that measurement. Across the
// thirty-one settled artifacts of the round-two run, 2215 of 2257 accepted
// issues carry `resolved: true`, so checkers confirm 98.1% of repairs. A verdict
// that near-unanimous separates almost nothing, which is why the question
// belongs on a human sheet.
//
// Region-shaped, not issue-shaped, because envelopes merge OVERLAPPING and
// TOUCHING target-side intervals (`deriveEditableEnvelopes` merges on
// `interval.start <= last.end`), so one replacement can serve several accepted
// issues and may fix only some of them. Copying one replacement onto each issue
// as "the repair for this issue" would erase that. Keeping the served issue ids
// on the region lets a record select the regions naming it while still
// disclosing who else the same edit was written for.

/**
 One replacement the accuracy stage applied, with every accepted issue the
 replaced region was cut for.

 @example
 ```ts
 const region: RepairRegion = {
   envelopeId: 'envelope/abcd',
   issueIds: ['adjudicated/one',],
   before: 'The cat is doing the sleeping.',
   editorAfter: 'The cat is asleep.',
 };
 ```
 */
export type RepairRegion = {
  /**
   Envelope this replacement targeted.
   */
  readonly envelopeId: string;

  /**
   Every accepted issue whose target-side evidence contributed to this
   envelope, in issue order. More than one means the replacement is shared
   and cannot be read as written for any single issue.
   */
  readonly issueIds: readonly string[];

  /**
   Envelope content before the replacement; empty at an insertion point.
   */
  readonly before: string;

  /**
   Replacement text of the SELECTED accuracy patch.

   Named for the stage rather than for a model on purpose: the editor
   ensemble's judges pick between candidates and may ship a composite or a
   fallback, so this is what the stage settled on, not what any one editor
   wrote. It is also pre-refinement text; the naturalness lane may rewrite the
   surrounding paragraph afterwards, which `ChunkRepairOutcome.refined`
   records. It is deliberately not re-derived from refined text: envelopes are
   sub-paragraph spans and a whole-paragraph rewrite carries no offset mapping
   back onto them, so any re-derivation would be a guess presented as a record.
   */
  readonly editorAfter: string;
};

/**
 Records every applied operation as a region, carrying the accepted issues its
 envelope serves.

 @param envelopes - editable envelopes the operations were written against

 @param applied - operations that passed the deterministic apply gate

 @returns One region per applied operation, in operation order

 @example
 ```ts
 const regions = collectRepairRegions({
   envelopes,
   applied: editor.patch.applied,
 },);
 ```
 */
export function collectRepairRegions(
  {
    envelopes,
    applied,
  }: {
    readonly envelopes: readonly EditableEnvelope[];
    readonly applied: readonly PatchOperation[];
  },
): readonly RepairRegion[] {
  /**
   Envelopes by identity, so attribution stays linear in the operation count;
   a map, as every table filled by a key is (ledger B77).
   */
  const byId: ReadonlyMap<string, EditableEnvelope> = new Map(envelopes.map(function keyed(envelope,): readonly [
    string,
    EditableEnvelope,
  ] {
    return [
      envelope.envelopeId,
      envelope,
    ];
  },),);

  return applied.map(function toRegion(operation,): RepairRegion {
    /**
     Envelope this operation targets, present because the apply gate rejects
     every operation naming an unknown envelope.
     */
    const envelope = nonNullishOrThrow(byId.get(operation.envelopeId,),);

    return {
      envelopeId: envelope.envelopeId,
      issueIds: envelope.issueIds,
      before: envelope.baseText,
      editorAfter: operation.newText,
    };
  },);
}

//endregion Repair region
