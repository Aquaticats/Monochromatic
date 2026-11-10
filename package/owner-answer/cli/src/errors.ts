/**
 Records the owner's answers in the questions file,
 and takes the answered questions out of its open list.

 The answers come as one block of Markdown,
 so the owner's own words are never retyped from a summary.
 Everything the block replaces is named,
 and a name that matches nothing stops the run before the file is written.

 @module
 */

/**
 Failure vocabulary for the owner-answer command.
 */
export class OwnerAnswerError extends Error {
  /**
   Creates a named failure.

   @param message - Text naming the input and the remedy.

   @example
   ```ts
   throw new OwnerAnswerError('the answers file names no record block');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'OwnerAnswerError';
  }
}

/**
 Reports command invocation misuse, which exits with status two.
 */
export class CliInvocationError extends OwnerAnswerError {
  /**
   Creates an invocation failure.

   @param message - Text naming the flag or argument to correct.

   @example
   ```ts
   throw new CliInvocationError('--answers FILE is required');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'CliInvocationError';
  }
}

/**
 Reports an answers file that does not match the schema,
 or a questions file whose named parts are not there.
 */
export class AnswersError extends OwnerAnswerError {
  /**
   Creates an answers failure.

   @param message - Text naming the part and what was expected.

   @example
   ```ts
   throw new AnswersError('no section starts with: ### A question');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'AnswersError';
  }
}
