//region Translation repair interruption

/**
 Operational stop when an absent passage cannot form another unique repair task.

 This preserves latest findings for resumed stage work.
 It is not quality verdict and does not authorize publication.

 ONE REASON REMAINS (B1181, B1183). `carried-evidence-lost` and `page-footnote-integrity` were verdicts on the page that
 ships, thrown where the owner's rule is to ship with the defect reported, and are now publish defects. A stage that
 heard no voice is an outage; the pass ships the interim page for it and keeps the entry open.

 @example
 ```ts
 throw new TranslationRepairInterruptedError({
   reason: 'provider-unavailable',
   findings: [],
 },);
 ```
 */
export class TranslationRepairInterruptedError extends Error {
  /**
   Declares message safe because reason is closed operation vocabulary.
   */
  readonly messageNamesOnly: true = true;

  /**
   Named stage state preventing unique continuation.
   */
  public readonly reason: 'provider-unavailable';

  /**
   Latest stage findings available when repair paused.
   */
  public readonly findings: readonly string[];

  /**
   Constructs operation-only interruption without source or candidate wording.

   @param reason - named stage state preventing unique continuation

   @param findings - latest structured evidence for resumed repair

   @example
   ```ts
   new TranslationRepairInterruptedError({
     reason: 'provider-unavailable',
     findings: [],
   },);
   ```
   */
  public constructor(
    {
      reason,
      findings,
    }: {
      readonly reason: 'provider-unavailable';
      readonly findings: readonly string[];
    },
  ) {
    super(`translation repair interrupted: ${reason}`,);
    this.name = 'TranslationRepairInterruptedError';
    this.reason = reason;
    this.findings = findings;
  }
}

//endregion Translation repair interruption
