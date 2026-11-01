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
   Model the call was addressed to.
   */
  readonly modelId: string;

  /**
   What the router decided, verbatim.
   */
  readonly reason: string;

  /**
   Whether the router raised this after an exchange: a call of this request was made and cut at its stream bound or
   refused, and the router then found no provider left to try (B1230). A refusal raised before any call leaves this
   false, and a model refused that way was never asked.
   */
  readonly afterExchange: boolean;

  /**
   Builds failure naming the model and why nowhere could take it.

   @param modelId - model the call was addressed to

   @param reason - what the router decided, verbatim

   @param afterExchange - whether a call of this request was made first; false when absent

   @param cause - refusal this one replaces when the mark is put on it, whose stack and own cause stay reachable
   through it (B1248)

   @example
   ```ts
   new NoProviderForModelError({ modelId: 'minimax-m3', reason: 'no provider serves this model', },);
   ```
   */
  public constructor(
    {
      modelId,
      reason,
      afterExchange = false,
      cause,
    }: {
      readonly modelId: string;
      readonly reason: string;
      readonly afterExchange?: boolean;
      readonly cause?: unknown;
    },
  ) {
    super(
      `no provider can take ${modelId}: ${reason}`,
      (cause === undefined) ? {} : { cause, },
    );
    this.name = 'NoProviderForModelError';
    this.modelId = modelId;
    this.reason = reason;
    this.afterExchange = afterExchange;
  }
}

/**
 Whether an error is the router's refusal of a model it never asked, the only kind whose claim a re-ask gives back.

 @param error - what a re-ask caught

 @returns True for a no-provider refusal raised before any call of the request

 @example
 ```ts
 if (refusedBeforeAnyExchange({ error, },)) claims.release({ modelId, },);
 ```
 */
export function refusedBeforeAnyExchange({ error, }: { readonly error: unknown; },): boolean {
  return (error instanceof NoProviderForModelError) && (!error.afterExchange);
}

/**
 Marks a no-provider refusal as raised after an exchange, leaving any other error as it is.

 @param error - what the router caught or is about to raise after a call of the request was made

 @returns Same refusal marked as raised after an exchange, or the error itself when it is none or is marked

 @example
 ```ts
 throw markedAfterExchange({ error, },);
 ```
 */
export function markedAfterExchange({ error, }: { readonly error: unknown; },): unknown {
  if ((!(error instanceof NoProviderForModelError)) || error.afterExchange)
    return error;
  return new NoProviderForModelError({
    modelId: error.modelId,
    reason: error.reason,
    afterExchange: true,
    cause: error,
  },);
}

//endregion No provider for model
