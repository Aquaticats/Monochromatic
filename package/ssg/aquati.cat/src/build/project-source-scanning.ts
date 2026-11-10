/**
 Linear scanners that turn project source text into the Material Symbols icons
 the site uses.
 
 The icon charset pass needs the `icon('NAME')` call sites of every source file
 with the comments taken out. This module does that with one character walk per
 step, replacing the three regular expressions the pass used: a block comment
 stripper, a line comment stripper, and the call-site matcher. Each walk resumes
 where the previous match ended, so the whole pass stays linear in the length of
 the text and no backtracking state has to be reasoned about.
 
 The two strippers keep the order the pass depends on: block comments are
 removed before line comments, because a line comment marker inside a block
 comment would otherwise eat the line that closes the block comment.
 
 @module
 */

//region Shared character tests

/**
 Opening delimiter of a block comment.
 */
const BLOCK_COMMENT_OPENING = '/*';

/**
 Closing delimiter of a block comment.
 */
const BLOCK_COMMENT_CLOSING = '*/';

//endregion Shared character tests

//region Block comment removal

/**
 Removes every block comment from source text.
 
 Delimiters pair up the way the previous non-greedy pattern paired them: the
 first closing delimiter after an opening `/*` ends the comment, so a comment
 marker inside a comment closes it there. An opening `/*` with no closing
 delimiter after it is not a comment and stays in the text, which is what an
 unterminated block comment leaves the pass with today.
 
 @param text - source text to strip
 
 @returns the text with every closed block comment removed
 
 @example
 ```ts
 // A commented-out call no longer reaches the icon charset.
 const stripped = stripBlockComments(sourceText,);
 ```
 */
export function stripBlockComments(text: string,): string {
  /**
   Text runs between comments, joined once at the end so the walk never rebuilds
   the result character by character.
   */
  const kept: string[] = [];
  for (let cursor = 0;;) {
    /**
     Next opening delimiter at or after the cursor; absent when the text is done.
     */
    const opening = text.indexOf(
      BLOCK_COMMENT_OPENING,
      cursor,
    );
    /**
     First closing delimiter after that opening; absent when the comment never closes.
     */
    const closing = opening === (-1)
      ? (-1)
      : text.indexOf(
        BLOCK_COMMENT_CLOSING,
        opening + BLOCK_COMMENT_OPENING.length,
      );
    if ((opening === (-1)) || (closing === (-1))) {
      kept.push(text.slice(cursor,),);
      return kept.join('',);
    }
    kept.push(
      text.slice(
        cursor,
        opening,
      ),
    );
    cursor = closing + BLOCK_COMMENT_CLOSING.length;
  }
}

//endregion Block comment removal
