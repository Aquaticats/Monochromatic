/**
 One `rm` command whose paths are not written in the command text.
 
 @module
 */

//region Types

/**
 One `rm` whose paths are not written in the command text.
 */
type LiteralRmPathFinding = {
  /**
   Command line the `rm` was read from, as written.
   */
  readonly command: string;
  /**
   Word whose spelling makes the path unreadable,
   or the program word when another program supplies the paths.
   */
  readonly word: string;
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

export type { LiteralRmPathFinding, };
