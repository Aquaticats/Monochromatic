/**
 {@link tomlSetHeaderComment}: set or clear the file's header comment block.
 
 @module
 */

import type { TomlEditState, } from './types.ts';

/**
 Set the header comment block.
 
 Pass a string (or array of lines) for the new comment; omit `comment`
 to clear an existing header. Each line is emitted as `# <line>` by
 canonical-mode output, preceded by a blank line.
 
 Splice-mode emission does not currently re-flow existing source
 comments; setting a header comment in splice mode affects only
 canonical re-emission via {@link tomlStringify} when the state is canonical.
 
 @returns A fresh {@link TomlEditState} reflecting the change.
 
 @example
 ```ts
 tomlSetHeaderComment({ edit, comment: ' Generated; do not edit', },);
 tomlSetHeaderComment({ edit, },); // clears any existing header
 ```
 */
export function tomlSetHeaderComment(
  {
    edit,
    comment,
  }: {
    readonly edit: TomlEditState;
    readonly comment?: string | readonly string[];
  },
): TomlEditState {
  if (comment === undefined) {
    /**
     Drop the property entirely so the cleared state has no header.
     */
    const {
      headerComment: _cleared,
      ...rest
    } = edit;
    return rest;
  }
  /**
   Normalised payload so the state field carries one representation.
   */
  const headerComment = (typeof comment) === 'string' ? comment : comment.join('\n',);
  return {
    ...edit,
    headerComment,
  };
}
