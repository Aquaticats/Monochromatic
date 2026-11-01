//region Translation repair interruption

/**
 Operational stop when an absent passage cannot form another unique repair task.

 This preserves latest findings for resumed stage work.
 It is not quality verdict and does not authorize publication.

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
  public readonly reason:
    | 'carried-evidence-lost'
      | 'page-footnote-integrity'
      | 'provider-unavailable';

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
     reason: 'carried-evidence-lost',
     findings: [],
   },);
   ```
   */
  public constructor(
    {
      reason,
      findings,
    }: {
      readonly reason:
        | 'carried-evidence-lost'
          | 'page-footnote-integrity'
          | 'provider-unavailable';
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
