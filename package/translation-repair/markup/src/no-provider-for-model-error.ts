//region No provider for model
// Its own module so the router's re-ask can raise it without importing the
// router that imports the re-ask (ledger P9).

/**
 Refusal raised when no provider can take one call at all, or, on a hinted
 re-ask, when no provider other than the one it was hinted away from can.

 @example
 ```ts
 throw new NoProviderForModelError({ modelId, reason: 'no provider serves this model', },);
 ```
 */
export class NoProviderForModelError extends Error {
  /**
   Declares this message safe to forward: it names a model and which of the routing outcomes it hit.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model and why nowhere could take it.

   @param modelId - model the call was addressed to

   @param reason - what the router decided, verbatim

   @example
   ```ts
   new NoProviderForModelError({ modelId: 'minimax-m3', reason: 'no provider serves this model', },);
   ```
   */
  public constructor(
    {
      modelId,
      reason,
    }: {
      readonly modelId: string;
      readonly reason: string;
    },
  ) {
    super(`no provider can take ${modelId}: ${reason}`,);
    this.name = 'NoProviderForModelError';
  }
}

//endregion No provider for model
