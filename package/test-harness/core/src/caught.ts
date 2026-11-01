//region Capturing a refusal
// Holding what a call threw, so more than one thing can be asserted about it.

/**
 Runs a call that must refuse and hands back what it threw.
 
 WHY THIS EXISTS RATHER THAN A SECOND EXPECTATION ON `toThrow`. That matcher
 takes ONE expectation, a message, a pattern, or a class, so a test wanting
 BOTH the class and the wording cannot ask for them in a single call. Asking
 twice on one bound matcher does not work either: chai rebinds the assertion
 subject to the error it caught, so the second call asserts against that error
 rather than the function and reports "expected SomeError... to be a function".
 Holding the thrown value lets each fact be asserted on its own with ordinary
 matchers, and an assertion that names only a message passes just as happily
 when the wrong type is thrown.
 
 REFUSES A CALL THAT RETURNS rather than handing back `undefined`. Every use of
 this is a test asserting a refusal, so a call that returned is the failure
 itself, and saying so here reads better than an `undefined` some later matcher
 complains about in terms that do not mention throwing.
 
 SYNCHRONOUS ONLY, deliberately. A rejected promise needs no equivalent:
 `expect(promise).rejects` awaits afresh on every matcher call, so asserting a
 class and then a message against the same promise already works.
 
 @param act - call expected to throw
 
 @returns Whatever it threw, unchanged and unwrapped
 
 @throws Error when `act` returns instead of throwing
 
 @example
 ```ts
 const refusal = caught(function readsAFutureVersion() {
   readArtifactSchemaVersion({ artifact, path, },);
 },);
 
 expect(refusal,).toBeInstanceOf(ArtifactParseError,);
 expect((refusal as Error).message,).toContain('this reader knows',);
 ```
 */
export function caught(act: () => unknown,): unknown {
  try {
    act();
  }
  catch (error) {
    return error;
  }
  throw new Error(
    `Expected ${(act.name === '') ? 'the call' : act.name} to throw, but it returned`,
  );
}

//endregion Capturing a refusal
