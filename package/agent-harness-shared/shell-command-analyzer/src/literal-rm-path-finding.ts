/**
 One deletion command whose target this repository's literal-path rule forbids.
 
 A finding names the command as written,
 the word that broke the rule,
 which check fired and why,
 and, for a script file, the file and the line its command starts on.
 
 @module
 */

//region Types

/**
 Which check a finding came from.
 */
type LiteralRmPathCheck =
  | 'another-program-supplies-the-paths'
    | 'find-deletes-the-paths-it-found'
    | 'shell-computes-the-path'
    | 'target-collapses-to-a-broader-path';

/**
 One deletion whose target is not written as the rule requires.
 */
type LiteralRmPathFinding = {
  /**
   Command line the deletion was read from, as written.
   */
  readonly command: string;
  /**
   Word whose spelling or value broke the rule,
   or the program word when another program supplies the paths.
   */
  readonly word: string;
  /**
   Which check fired.
   */
  readonly check: LiteralRmPathCheck;
  /**
   Why that check fired, in words a refusal prints after the word.
   */
  readonly detail?: string;
  /**
   File the finding was read from, when the source is a file's whole text.
   */
  readonly file?: string;
  /**
   Line of the finding in {@link file}, when the reader can locate it.
   */
  readonly line?: number;
};

//endregion Types

export type {
  LiteralRmPathCheck,
  LiteralRmPathFinding,
};
