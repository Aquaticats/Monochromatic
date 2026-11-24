/**
 The failure one literal-`rm`-path verification reports.
 
 Kept apart from the session and the assertions so both can throw it without
 importing each other.
 
 @module
 */

//region Types

/**
 A verification run that cannot answer its question.
 */
class LiteralRmVerificationError extends Error {
  /**
   Name printed with the message.
   */
  override readonly name = 'LiteralRmVerificationError';
}

//endregion Types

//region Constructors

/**
 Reports one mismatch between what Pi answered and what the guardrail promises.
 
 @param problem - what differed, in plain words
 
 @returns the error to throw
 
 @example
 ```typescript
 throw mismatch('expected a refusal');
 ```
 */
function mismatch(problem: string,): LiteralRmVerificationError {
  return new LiteralRmVerificationError(problem,);
}

//endregion Constructors

export {
  LiteralRmVerificationError,
  mismatch,
};
