/**
 Tests for `caught`, which holds what a call threw so more than one thing can
 be asserted about it.
 
 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

/**
 Refusal class standing in for the custom errors real callers assert about.
 */
class WhiskerError extends Error {
  /**
   Names the class in stack traces the way a real custom error does.
   */
  public override readonly name = 'WhiskerError';
}

await describe({
  name: caught.name,
  children: [
    it({
      name: 'HANDS BACK THE THROWN VALUE UNCHANGED, which is the whole point: the class and the '
        + 'wording can then be asserted separately, and an assertion naming only a message passes '
        + 'just as happily when the wrong type is thrown',
      fn: async function returnsWhatWasThrown() {
        const refusal = caught(function refusesTheBowl() {
          throw new WhiskerError('the bowl is empty',);
        },);

        expect(refusal,).toBeInstanceOf(WhiskerError,);
        expect((refusal as Error).message,).toContain('the bowl is empty',);
      },
    },),

    it({
      name: 'HANDS BACK A THROWN NON-ERROR TOO, unwrapped, because a call under test is not obliged '
        + 'to throw an Error and a helper that assumed one would hide exactly that',
      fn: async function returnsNonErrors() {
        expect(caught(function throwsAString() {
          // oxlint-disable-next-line eslint/no-throw-literal, typescript/only-throw-error -- the fixture IS a thrown non-Error, which is the whole case: a helper that assumed an Error would hide exactly this
          throw 'no bowl at all';
        },),).toBe('no bowl at all',);
      },
    },),

    it({
      name: 'REFUSES A CALL THAT RETURNS, naming the call, rather than handing back undefined for a '
        + 'later matcher to complain about in terms that never mention throwing',
      fn: async function refusesAReturn() {
        expect(function callsOneThatReturns() {
          caught(function fillsTheBowl() {
            return 'full';
          },);
        },).toThrow('Expected fillsTheBowl to throw, but it returned',);
      },
    },),

    it({
      name: 'SAYS "the call" FOR AN ANONYMOUS ONE, so the refusal still reads as a sentence when the '
        + 'caller passed an arrow with no name to borrow',
      fn: async function namesAnonymousCalls() {
        expect(function callsAnAnonymousOne() {
          caught(() => 'full',);
        },).toThrow('Expected the call to throw, but it returned',);
      },
    },),
  ],
},);
