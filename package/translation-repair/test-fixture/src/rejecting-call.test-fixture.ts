//region Rejecting call
// RUNS A CALL EXPECTED TO REFUSE, handing back whatever it threw rather than
// asserting its shape, for cases that inspect the refusal themselves.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several corpus-run tests kept their own
// copy of this runner; all now import it from here.

/**
 Runs a call that must refuse and hands back what it threw.

 @param act - call expected to reject

 @returns Whatever it rejected with, unchanged

 @throws Error when the call resolved instead of rejecting

 @example
 ```ts
 const refusal = await rejectionOf(async function overNothing() { ... },);
 ```
 */
export async function rejectionOf(act: () => Promise<unknown>,): Promise<unknown> {
  try {
    await act();
  }
  catch (error) {
    return error;
  }
  throw new Error(
    `Expected ${(act.name === '') ? 'the call' : act.name} to refuse, but it returned`,
  );
}

//endregion Rejecting call
