import { declaredNameRefusalFinding, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { quoteLossRefusalFinding, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { alignmentRefusalFinding, } from './translate-alignment.ts';
import type { TranslateSliceRecord, } from './translate-document-contract.ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Translate alignment refusals
// The sentence a reader gets for every slice whose replacement the alignment
// guard refused.
//
// Split out of the driver with the assembly it belongs to: it reads settled
// records and builds prose, which is reporting rather than translating, and the
// driver was over its line budget with both.

/**
 Names what the alignment guard measured, for callers building a report.

 DERIVED RATHER THAN STORED. The sentence names a slice by its index, and a
 settled record is keyed by what the models were asked, which since translate
 version 2 excludes the index. The same record can therefore be resumed at a
 different position, so the only trustworthy index is the one the record
 carries after the driver stamps it, which is the one this reads.

 @param records - settled slice records

 @returns Refusal findings in the order the slices appear

 @throws {@link UnhandledMemberInvariantError} when a member of `TranslateDisposition` has no branch here, which the compiler rules out, since a record that is neither a refusal nor a stage result has no finding

 @example
 ```ts
 const refusals = alignmentRefusals({ records: result.slices, },);
 ```
 */
export function alignmentRefusals(
  { records, }: { readonly records: readonly TranslateSliceRecord[]; },
): readonly string[] {
  return records
    .flatMap(function toFinding(record,): readonly string[] {
      if (record.disposition === 'refused-alignment') {
        return [alignmentRefusalFinding({
          sliceIndex: record.sliceIndex,
          assessment: record.alignment,
        },),];
      }

      // A QUOTE-LOSS REFUSAL IS NAMED TOO, and named differently. Both keep the
      // archive and both are worth counting, but a run whose refusals are all
      // one kind is a different run from one whose refusals are all the other,
      // and a single label would hide that. The counts are the ones the guard
      // compared, stored with the refusal: recounting the record's whole
      // archive counted a held-out transcript's quotes as the judged part's
      // (ledger B42).
      if (record.disposition === 'refused-quote-loss') {
        return [quoteLossRefusalFinding({
          sliceIndex: record.sliceIndex,
          quotedPassages: record.quotedPassages,
        },),];
      }
      // A DROPPED DECLARED NAME IS NAMED TOO, and named separately for the same
      // reason: a run whose refusals are all names is a different run from one
      // whose refusals are all alignment, and one label would hide that.
      if (record.disposition === 'refused-declared-name') {
        return [declaredNameRefusalFinding({
          sliceIndex: record.sliceIndex,
          dropped: record.droppedDeclaredNames,
        },),];
      }
      // A STAGE RESULT TAKEN AS IT STANDS refused nothing, so it has no
      // finding. It is named rather than left to the end of the chain, so a
      // disposition added without a finding fails to compile here instead of
      // passing as one that refused nothing.
      if (record.disposition === 'stage-result')
        return [];
      throw new UnhandledMemberInvariantError({
        what: 'translate slice disposition',
        member: record.disposition,
      },);
    },);
}

//endregion Translate alignment refusals
