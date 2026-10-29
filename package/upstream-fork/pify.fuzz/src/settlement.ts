/**
 Comparable descriptions of promise settlements, so the oracle can compare
 two implementations' outcomes structurally.
 
 @module
 */

//region Types

/**
 One settlement description: how the promise settled and what it settled
 with. Rejections with `Error` reasons are described by name and message;
 every other reason (upstream `pify` rejects with raw callback value arrays)
 is carried raw for structural comparison.
 */
export type Settlement = {
  /**
   How the promise settled.
   */
  readonly tag: 'resolved' | 'rejected';
  /**
   Resolution value, or the rejection reason's comparable description.
   */
  readonly value: unknown;
};

//endregion Types

//region Settlement

/**
 Describes one promise's settlement for structural comparison.
 
 @param promise - Promise to observe.
 
 @returns Settlement description once the promise settles.
 
 @example
 ```ts
 const settlement = await describeSettlement(pified({ args: [], }));
 settlement.tag; // => 'resolved'
 ```
 */
export async function describeSettlement(promise: Promise<unknown>,): Promise<Settlement> {
  try {
    return {
      tag: 'resolved',
      value: await promise,
    };
  }
  catch (reason) {
    return Error.isError(reason,)
      ? {
        tag: 'rejected',
        value: {
          errorName: reason.name,
          message: reason.message,
        },
      }
      : {
        tag: 'rejected',
        value: reason,
      };
  }
}

//endregion Settlement
