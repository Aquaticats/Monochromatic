/**
 Reports the `rm` violations a file-editing tool call would add to a shell script.
 
 Both agent harnesses reach the same decision here:
 a write replaces the file, an edit changes part of it,
 and either one is judged by the text the call leaves on disk against the text
 the file holds now.
 Keeping that decision in one place is what makes two harnesses refuse the same
 changes.
 
 @module
 */

import { homedir, } from 'node:os';

import { literalRmPathFindingsAddedToScriptFile, } from './literal-rm-paths-added-to-script-file.ts';
import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';
import {
  textAfterEdits,
  type TextEdit,
} from './text-after-edits.ts';
import { textBeforeAFileChange, } from './text-before-a-file-change.ts';

//region Public reads

/**
 Reports the violations a write call would add to a script file.
 
 @param path - file the call writes
 
 @param content - text the call leaves in the file
 
 @returns the violations the call adds, empty when it adds none
 
 @example
 ```ts
 await literalRmPathFindingsAWriteWouldAdd({
   path: '/repo/clean.sh',
   content: 'rm -rf "$d"\n',
 });
 ```
 */
async function literalRmPathFindingsAWriteWouldAdd({
  path,
  content,
}: {
  readonly path: string;
  readonly content: string;
},): Promise<readonly LiteralRmPathFinding[]> {
  return literalRmPathFindingsAddedToScriptFile({
    path,
    before: await textBeforeAFileChange({ path, },),
    after: content,
  },);
}

/**
 Reports the violations an edit call would add to a script file.
 
 @param path - file the call edits
 
 @param edits - replacements the call makes, in order
 
 @returns the violations the call adds, empty when it adds none
 
 @example
 ```ts
 await literalRmPathFindingsAnEditWouldAdd({
   path: '/repo/clean.sh',
   edits: [{ oldText: 'LITERAL', newText: '"$d"' }],
 });
 ```
 */
async function literalRmPathFindingsAnEditWouldAdd({
  path,
  edits,
}: {
  readonly path: string;
  readonly edits: readonly TextEdit[];
},): Promise<readonly LiteralRmPathFinding[]> {
  /**
   Text the file holds before the call.
   */
  const before = await textBeforeAFileChange({ path, },);
  return literalRmPathFindingsAddedToScriptFile({
    path,
    before,
    after: textAfterEdits({
      text: before,
      edits,
    },),
  },);
}

//endregion Public reads

export {
  literalRmPathFindingsAnEditWouldAdd,
  literalRmPathFindingsAWriteWouldAdd,
};
