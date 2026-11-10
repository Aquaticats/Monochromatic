/**
 Exhausted structured-review diagnostics shared by concrete callers.
 
 @module
 */

//region Helpers -- primitive text joining without caller methods

/**
 Join primitive diagnostic text without invoking methods on caller arrays.
 
 @param values - primitive strings to join
 
 @param separator - text inserted between values
 
 @returns joined text
 
 @example
 ```ts
 joinText({ values: ['a', 'b'], separator: ', ' });
 ```
 */
function joinText(
  {
    values,
    separator,
  }: {
    readonly values: readonly string[];
    readonly separator: string;
  },
): string {
  /**
   Joined primitive output.
   */
  const output = { value: '', };
  for (const value of values) {
    output.value = output.value === ''
      ? value
      : `${output.value}${separator}${value}`;
  }
  return output.value;
}

//endregion Helpers -- primitive text joining without caller methods

//region Public API -- exhausted-reviewer error with complete attempt audit

/**
 Exhausted structured reviewer error with complete attempt audit.
 
 @example
 ```ts
 throw new ReviewUnavailableError({
   attemptedCandidateIdentities: ['a/one'],
   diagnostics: ['timeout'],
 });
 ```
 */
class ReviewUnavailableError extends Error {
  /**
   Candidate identities whose transports started.
   */
  readonly attemptedCandidateIdentities: readonly string[];
  /**
   Normalized transport, parsing, and selection diagnostics.
   */
  readonly diagnostics: readonly string[];

  /**
   Create exhausted-reviewer diagnostic.
   
   @param attemptedCandidateIdentities - candidates whose transports started
   
   @param diagnostics - normalized failure details
   
   @param cause - terminal lower-level failure
   
   @example
   ```ts
   new ReviewUnavailableError({
     attemptedCandidateIdentities: ['a/one'],
     diagnostics: ['timeout'],
   });
   ```
   */
  constructor(
    {
      attemptedCandidateIdentities,
      diagnostics,
      cause,
    }: {
      readonly attemptedCandidateIdentities: readonly string[];
      readonly diagnostics: readonly string[];
      readonly cause?: unknown;
    },
  ) {
    super(
      `Structured review unavailable after attempts by ${joinText({
        values: attemptedCandidateIdentities,
        separator: ', ',
      },)}: ${joinText({
        values: diagnostics,
        separator: '; ',
      },)}`,
      ...(cause === undefined ? [] : [{ cause, },]),
    );
    this.name = 'ReviewUnavailableError';
    this.attemptedCandidateIdentities = [...attemptedCandidateIdentities,];
    this.diagnostics = [...diagnostics,];
  }
}

//endregion Public API -- exhausted-reviewer error with complete attempt audit

export { ReviewUnavailableError, };
