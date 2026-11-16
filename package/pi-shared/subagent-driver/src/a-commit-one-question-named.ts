/**
 The kind a commit has once one git question has named it, which every later question is asked of, and which a revision that may
 move does not have.

 A branch moves on as soon as anyone commits to it, and this machine merges into its integration many times an hour, so a question
 asked of a name can be answered about another commit than the one a generation is about. The type here makes that a compile error:
 a function that asks git about a commit takes this type, and a name is only a string.
 */

/**
 The mark a commit carries once a git question has named it.
 */
declare const NAMED_BY_ONE_GIT_QUESTION: unique symbol;

/**
 A commit one git question named, by its full name, which every later question about it is asked of.
 */
export type APinnedCommit = string & {
  readonly [NAMED_BY_ONE_GIT_QUESTION]: 'one git question named this commit';
};

/**
 Reads a commit id git itself printed as a commit no question has to resolve again.

 @param commit - the commit id, as git printed it

 @returns the commit, which every later question about it is asked of

 @example
 ```ts
 const pinned = aCommitOneQuestionNamed({ commit: 'a'.repeat(40), },);
 ```
 */
export function aCommitOneQuestionNamed({ commit, }: { readonly commit: string },): APinnedCommit {
  // The brand is a compile-time claim and nothing more: git printed this commit id, and no runtime value changes here. The type
  // guard the linter asks for cannot exist, because a string holds no evidence of which question named it, and every later
  // question takes this type so that a name that may move cannot be passed to one.
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- this is the one place git's own answer becomes the type every later git question takes
  return commit as APinnedCommit;
}
