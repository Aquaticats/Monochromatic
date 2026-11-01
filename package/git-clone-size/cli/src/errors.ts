/**
 Operational error for git-clone-size.
 
 This is raised ONLY for operational failures (a spawned process the tool
 itself could not run, an unreadable local path). It is never thrown to mean
 "cannot estimate": the estimator never refuses. When a probe fails or a
 budget trips, that folds into a wider range and lower confidence instead of
 an error.
 
 @module
 */

/**
 Operational error raised when git-clone-size cannot run an underlying
 operation. Distinct from estimation uncertainty, which is expressed as a
 range, never an error.
 
 @example
 ```ts
 throw new CloneSizeError({ message: 'git not found on PATH' });
 ```
 */
export class CloneSizeError extends Error {
  /**
   Stable name for instanceof-free discrimination and log lines.
   */
  public override readonly name = 'CloneSizeError';

  /**
   @param message - human-readable operational failure description
   
   @param cause - underlying error, when wrapping a lower-level throw
   */
  public constructor(
    {
      message,
      cause,
    }: {
      readonly message: string;
      readonly cause?: unknown
    },
  ) {
    super(
      message,
      cause === undefined ? undefined : { cause, },
    );
  }
}
