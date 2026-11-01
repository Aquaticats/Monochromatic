//region Pass entry contract
// Shapes shared by corpus scheduling and one-entry settlement.

/**
 Whether one entry reached its artifact.

 @example
 ```ts
 const outcome: EntryOutcome = { kind: 'settled', };
 ```
 */
export type EntryOutcome = {
  /**
   Artifact was written after all stages and publication completed.
   */
  readonly kind: 'settled';
} | {
  /**
   Operational failure or hard ceiling may resume from newly cached work.
   */
  readonly kind: 'resumable-failure';
} | {
  /**
   Work remains incomplete but same invocation must not start whole entry again.
   */
  readonly kind: 'stopped';
} | {
  /**
   The archive's note says the whole page is the author's own English, so the
   pipeline declined to repair it (the owner's rule of 2026-09-08); a decline
   record stands where the artifact would, and no pass attempts it again.
   */
  readonly kind: 'declined';
};

/**
 One eligible corpus pair with its text loaded.

 @example
 ```ts
 const entry: CorpusPair = { id: 'CatEntry', sourceText: '猫。', targetText: 'Cat.' };
 ```
 */
export type CorpusPair = {
  /**
   Person entry id.
   */
  readonly id: string;

  /**
   Original Chinese page text.
   */
  readonly sourceText: string;

  /**
   English archive page text.
   */
  readonly targetText: string;
};

//endregion Pass entry contract
