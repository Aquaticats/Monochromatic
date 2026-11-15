/**
 How a review asks git a question, and what it does when git answers with a failure: the type a test replaces, the error a failure
 is answered with, and the one function every question goes through.

 The questions themselves live beside the reading they are for; this module holds only the seam, so a module that asks git can use
 it without depending on the module that reads a review's material.

 @module
 */

import type { CommandOutput, } from './run-command.ts';

/**
 How this module asks git a question, which a test replaces with the answers it expects.
 */
export type GitQuestion = (request: {
  readonly words: readonly string[];
  readonly folder: string;
}) => Promise<CommandOutput>;

/**
 A git question a review could not get an answer to.
 */
export class ReviewGitError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewGitError';
}

/**
 Asks git one question in the checkout and answers with what it printed.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the question is about

 @param words - the question's words

 @returns what git printed

 @throws ReviewGitError when git did not answer with a success

 @example
 ```ts
 const shown = await outputOfGit({ git, checkoutFolder: '/repo', words: ['rev-parse', 'main'], },);
 ```
 */
export async function outputOfGit({
  git,
  checkoutFolder,
  words,
}: {
  readonly git: GitQuestion;
  readonly checkoutFolder: string;
  readonly words: readonly string[];
}): Promise<string> {
  /**
   What git answered.
   */
  const answered = await git({
    words,
    folder: checkoutFolder,
  },);
  if (answered.status !== 0) {
    throw new ReviewGitError(
      `git ${words.join(' ',)} in ${checkoutFolder} answered status ${String(answered.status,)}: ${answered.stderr
        .trim()}`,
    );
  }
  return answered.stdout;
}
