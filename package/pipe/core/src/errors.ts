/**
 Error thrown when runtime pipeline arguments skip a function key and then provide a later one.
 
 Plain JavaScript callers, or TypeScript callers using assertions, can bypass the overload checks.
 This error preserves the invariant that later provided steps are never silently ignored.
 
 @example
 ```ts
 const error = new PipeStepGapError(1);
 error.name; // "PipeStepGapError"
 ```
 */
export class PipeStepGapError extends Error {
  /**
   Creates a gap error for the first missing zero-based step index.
   
   @param firstGapIndex - zero-based slot of first missing step key; reported one-based so the
   message names the absent `fnN` rather than its array offset
   */
  public constructor(firstGapIndex: number,) {
    super(
      `Pipeline step gap before fn${String(firstGapIndex + 1,)}; provide contiguous function keys from fn1.`,
    );
    this.name = 'PipeStepGapError';
  }
}

/**
 Error thrown when runtime pipeline arguments include a step beyond `fn9`.
 
 TypeScript overloads reject `fn10`, but plain JavaScript callers and assertions can bypass them.
 This error prevents unsupported steps from being silently ignored.
 
 @example
 ```ts
 const error = new PipeStepOverflowError();
 error.name; // "PipeStepOverflowError"
 ```
 */
export class PipeStepOverflowError extends Error {
  /**
   Creates an overflow error for unsupported `fn10` input.
   */
  public constructor() {
    super('Pipeline supports fn1 through fn9; fn10 was provided.',);
    this.name = 'PipeStepOverflowError';
  }
}
