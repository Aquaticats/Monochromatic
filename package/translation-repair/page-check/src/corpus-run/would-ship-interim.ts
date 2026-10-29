//region Would-ship interim
// THE ONE FIELD A SOURCE COMPOSED FOR AN INTERIM PAGE CARRIES BEYOND A SETTLED ARTIFACT'S, split from
// `would-ship-text.ts` at its line cap. `interim-page.ts` composes it and `would-ship-text.ts` reads it.

/**
 The slices an interim page ships without a settled wording, for a source composed while its entry is still open.

 A settled artifact carries none, which reads as an entry every slice of which the stages decided. An entry stopped dry
 in the middle of its page (the owner's answer of 2026-10-29, "Ship interim, resumable") carries the slices no
 consolidation record settled, and each of those reads as the archive's text, or as the original under its note where
 the archive has none, whatever the lane texts, the contest and the consolidation say of it.

 @example
 ```ts
 const carrier: InterimCarrier = { interim: { unsettledSlices: new Set([2, 3,],), }, };
 ```
 */
export type InterimCarrier = {
  /**
   The slices nothing settled, present only on a source composed for an interim page.
   */
  readonly interim?: {
    /**
     Indices of the slices that read as the archive's text or the original.
     */
    readonly unsettledSlices: ReadonlySet<number>;
  };
};

/**
 Whether a slice of an interim page is one nothing settled.

 @param artifact - source being read, which carries the slices only when composed for an interim page

 @param sliceIndex - slice the reading is for

 @returns Whether the slice reads as the archive's text or the original, false for a settled artifact

 @example
 ```ts
 if (isUnsettledSlice({ artifact, sliceIndex: 3, },)) return archiveStands;
 ```
 */
export function isUnsettledSlice(
  {
    artifact,
    sliceIndex,
  }: {
    readonly artifact: InterimCarrier;
    readonly sliceIndex: number;
  },
): boolean {
  /**
   The slices nothing settled, absent for a settled artifact.
   */
  const { interim, } = artifact;
  if (interim === undefined)
    return false;
  return interim.unsettledSlices
    .has(sliceIndex,);
}

//endregion Would-ship interim
