import type { TranslateSliceRecord, } from './translate-document-contract.ts';

//region Translate unheard
// What a settled record MEANS when the producing stage heard no translator, and
// the one place that says so.
//
// The driver keeps the archive's wording for such a slice and deliberately does
// not cache it, and `translateLaneWordings` reports it as the incumbent
// standing by default rather than as a decision. Both of those rest on a
// property nothing checked: that the record's own text really is the
// incumbent's, and that it does not claim to have changed anything.
//
// A stage that heard nobody and returned a change is a defect in the stage, and
// it would travel: the lane's shipped index set is built from `changed`, so the
// document would carry a replacement while the wording ledger reported that
// nobody produced one. That is the two-axis contradiction the ledger exists to
// make impossible, arriving from the other side.

/**
 Whether the producing stage behind this record heard nobody.

 ONE DEFINITION, because three places ask: the driver deciding whether to
 cache, the wording builder deciding what the outcome is, and
 `assertUnheardKeptIncumbent`.
 Two spellings of it would eventually disagree, and the slice that fell between
 them would be cached as a decision nobody made.

 @param record - settled slice record

 @returns Whether no translator answered for this slice

 @example
 ```ts
 if (heardNobody({ record, },)) l.warn('no translator answered',);
 ```
 */
export function heardNobody(
  { record, }: { readonly record: TranslateSliceRecord; },
): boolean {
  return record.stageResult
    .heardTranslators
    === 0;
}

/**
 Names a cached record refused for having heard nobody.

 TAGGED LIKE ITS SIBLINGS (`translate-heard-no-translator`,
 `translate-discarded-contradictory-slice`) rather than written as a sentence,
 because findings are read in bulk: an untagged one cannot be counted across a
 corpus, so a build that started refusing every cached slice would look like a
 run that simply had no cache.

 @param sliceIndex - slice being asked again

 @returns Finding naming the slice and why it was recomputed

 @example
 ```ts
 findings.push(unheardCacheDiscardFinding({ sliceIndex, },),);
 ```
 */
export function unheardCacheDiscardFinding(
  { sliceIndex, }: { readonly sliceIndex: number; },
): string {
  return `translate-discarded-unheard-slice chunk ${
    String(sliceIndex,)
  }; cached record heard no translator, which this driver never caches, so it was written by another `
    + 'build and the slice was asked again rather than settled on a wording nobody produced';
}

/**
 Raised when a stage that heard nobody reports having produced something.

 @example
 ```ts
 throw new TranslateUnheardError({ message: 'slice 4 heard nobody and reports a change', },);
 ```
 */
export class TranslateUnheardError extends Error {
  /**
   Builds the error naming the slice and what it claimed.

   @param message - what the record claims that hearing nobody rules out

   @example
   ```ts
   throw new TranslateUnheardError({ message: 'slice 4 heard nobody and reports a change', },);
   ```
   */
  constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'TranslateUnheardError';
  }
}

/**
 Refuses a record that heard nobody and did not leave the archive alone.

 @param sliceIndex - slice this record settles, for the message

 @param record - settled record, checked only when its stage heard nobody

 @param incumbentText - archive wording of this slice, read from the
 preparation rather than from the record, since the record is what is under
 suspicion

 @throws {@link TranslateUnheardError} when the record carries wording other
 than the archive's, or claims a change, without a voice behind it

 @example
 ```ts
 assertUnheardKeptIncumbent({ sliceIndex, record, incumbentText, },);
 ```
 */
export function assertUnheardKeptIncumbent(
  {
    sliceIndex,
    record,
    incumbentText,
  }: {
    readonly sliceIndex: number;
    readonly record: TranslateSliceRecord;
    readonly incumbentText: string;
  },
): void {
  if (!heardNobody({ record, },))
    return;
  if (record.outputText !== incumbentText) {
    throw new TranslateUnheardError({
      message: `slice ${String(sliceIndex,)} heard no translator and carries wording that is not the `
        + 'archive\'s, so a rendering nobody produced would be written',
    },);
  }
  if (record.changed) {
    throw new TranslateUnheardError({
      message: `slice ${String(sliceIndex,)} heard no translator and reports a change, which would put `
        + 'it in the shipped set while the wording ledger reports that nobody produced one',
    },);
  }
}

//endregion Translate unheard
