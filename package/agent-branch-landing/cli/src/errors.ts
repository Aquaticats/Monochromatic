/**
 What these commands throw when the caller or the checkout is wrong, which is the one thing they do not report as a line: a
 landing that could not run is a mistake to fix, not a state to read.

 @module
 */

/**
 A landing was asked for with something it cannot work with: an option it does not know, a list entry that names no branch or no
 message, a folder that is not a repository, a worktree on a detached HEAD, or a merge already open where a merge was wanted.
 */
export class LandingConfigurationError extends Error {
  /**
   What is wrong and what to change, in the caller's own words.

   @param message - the sentence a caller reads
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'LandingConfigurationError';
  }
}
