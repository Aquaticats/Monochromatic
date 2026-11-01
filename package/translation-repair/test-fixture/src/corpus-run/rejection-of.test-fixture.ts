//region Rejection of
// WHAT AN ASYNC CALL REJECTED WITH, for a case that asserts the whole error
// rather than a substring of its message.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. `caught` of module-test reads a call that
// throws at once; this reads one that rejects.

/**
 Awaits a promise that is expected to reject and hands back what it rejected
 with.

 @param promise - call expected to reject

 @returns The rejection

 @throws Error when the promise resolves

 @example
 ```ts
 const refusal = await rejectionOf({ promise: reportSliceCost({ line, },), },);
 ```
 */
export async function rejectionOf({ promise, }: { readonly promise: Promise<unknown>; },): Promise<unknown> {
  try {
    await promise;
  }
  catch (error) {
    return error;
  }
  throw new Error('Expected the call to reject, but it resolved',);
}

//endregion Rejection of
