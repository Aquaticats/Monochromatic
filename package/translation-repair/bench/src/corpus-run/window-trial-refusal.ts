//region Window trial refusal
// A DRAWN SLICE THE TRIAL CANNOT TRY, stated by the class so the pick's warning
// can repeat it. The pick catches whatever the arms raise, provider failures
// included, and a message of another class is named and not quoted, so the two
// refusals this trial makes on its own input need a class that declares its
// sentence safe: the sentence is built from an entry id and a slice index alone.

/**
 Draw the trial was handed that no arm can be run over.

 @example
 ```ts
 throw new TrialSliceRefusalError({ entryId: 'Mittens', sliceIndex: 99, kind: 'slice-missing', },);
 ```
 */
export class TrialSliceRefusalError extends Error {
  /**
   Declares this message safe to forward: it names an entry and a slice index.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal naming the entry, the slice and which of the two inputs
   was missing.

   @param entryId - entry the slice was drawn from

   @param sliceIndex - slice the draw named

   @param kind - `slice-missing` where the prepared slices do not carry the
   index, `window-empty` where its neighbouring original holds no text

   @example
   ```ts
   new TrialSliceRefusalError({ entryId: 'Mittens', sliceIndex: 0, kind: 'window-empty', },);
   ```
   */
  public constructor(
    {
      entryId,
      sliceIndex,
      kind,
    }: {
      readonly entryId: string;
      readonly sliceIndex: number;
      readonly kind: 'slice-missing' | 'window-empty';
    },
  ) {
    super(
      (kind === 'slice-missing')
        ? `${entryId} has no slice ${String(sliceIndex,)}; the draw and the preparation disagree, which means they `
          + 'were made from different text'
        : `${entryId}/${String(sliceIndex,)} has no neighbouring section carrying text, so its wide arm would be `
          + 'its narrow arm and the pair would report a false null',
    );
    this.name = 'TrialSliceRefusalError';
  }
}

//endregion Window trial refusal
