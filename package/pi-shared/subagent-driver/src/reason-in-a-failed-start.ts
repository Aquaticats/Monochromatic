/**
 What a failed start's one line says: the reason it printed, not the last line of a stack.

 Node ends an uncaught error with its own version, so the last line of a failed start was `Node.js v26.13.0` for every failure that
 mattered: ten starts of a batch on 2026-11-11 said nothing else, and why each of them ended is still a guess.

 @module
 */

import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 How much of a reason one line carries. A reason long enough to bury the run's name is not one a reader scans; the whole output is
 kept beside the line for the rest.
 */
export const ENTRY_REASON_CHARACTERS: number = 200;

/**
 How Node starts the line it ends an uncaught error with, which names Node and never the failure.
 */
const NODE_VERSION_START = 'Node.js v';

/**
 Says whether a line names a place in a file, which Node prints above an uncaught error and which says nothing about why it failed.

 @param line - one line of what a start printed

 @returns whether it is only a place, like `file:///pkg/src/x.ts:58`

 @example
 ```ts
 const place = isAPlaceLine('file:///pkg/src/x.ts:58');
 ```
 */
function isAPlaceLine(line: string,): boolean {
  if ((line.includes(' ',)) || (line.includes('\t',))) {
    return false;
  }
  /**
   Where the line's last colon is, above the number a place ends with.
   */
  const colon = line.lastIndexOf(':',);
  /**
   Whether the number a place ends with is all the line leaves after a colon.
   */
  const endsWithANumber = (colon > 0)
    && (colon < (line.length - 1))
    && isDigitsOnly(line.slice(colon + 1,),);
  return endsWithANumber;
}

/**
 Says whether one line of a failed start's output could be the reason it failed.

 @param line - one line of what a start printed

 @returns whether the line stands on its own as a reason

 @example
 ```ts
 const reason = standsOnItsOwn('GitCommandFailedError: git refused');
 ```
 */
function standsOnItsOwn(line: string,): boolean {
  /**
   The line without the whitespace it ends with, which a stack's frames do not change.
   */
  const trimmed = line.trimEnd();
  return (trimmed !== '')
    && (!trimmed.startsWith(NODE_VERSION_START,))
    && (!isAPlaceLine(trimmed,))
    && (trimmed === trimmed.trimStart());
}

/**
 Reads the reason a failed start printed.

 The reason is the last line that stands on its own: Node's stack frames, the code and caret it quotes, its version line and the
 place it names above the error are all dropped. A start that printed nothing but those answers with no reason, and the line says so
 instead of pretending otherwise.

 @param output - what the failed start printed on both streams

 @returns the reason, cut to {@link ENTRY_REASON_CHARACTERS}, or no reason when it printed none

 @example
 ```ts
 const { reason } = reasonInFailedStart({ output: 'Error: no prompt\n    at x\nNode.js v26.13.0\n', },);
 ```
 */
export function reasonInFailedStart({ output }: { readonly output: string },): { readonly reason?: string } {
  /**
   The reason, which is the last line that stood on its own.
   */
  const reason = output.split('\n',)
    .map(function withoutTrailingWhitespace(line,) {
      return line.trimEnd();
    },)
    .filter(standsOnItsOwn,)
    .at(-1,);
  return ((reason === undefined) || (reason === ''))
    ? {}
    : { reason: reason.slice(
      0,
      ENTRY_REASON_CHARACTERS,
    ), };
}
