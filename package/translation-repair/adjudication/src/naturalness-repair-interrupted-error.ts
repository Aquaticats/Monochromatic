//region Naturalness repair interruption

/**
 Operational stop when current naturalness evidence cannot form another unique correction task.

 This is incomplete work,
 not quality verdict and not publication authorization.

 @example
 ```ts
 throw new NaturalnessRepairInterruptedError({ reason: 'contributor-structure', });
 ```
 */
export class NaturalnessRepairInterruptedError extends Error {
  /**
   Declares message safe to forward because reason is closed operation vocabulary.
   */
  readonly messageNamesOnly: true = true;

  /**
   Constructs operation-only interruption without candidate wording.

   @param reason - named stage state preventing unique next correction

   @example
   ```ts
   new NaturalnessRepairInterruptedError({ reason: 'contributor-structure', });
   ```
   */
  public constructor(
    {
      reason,
    }: {
      readonly reason: 'contributor-structure';
    },
  ) {
    super(`naturalness repair interrupted: ${reason}`,);
    this.name = 'NaturalnessRepairInterruptedError';
  }
}

//endregion Naturalness repair interruption
