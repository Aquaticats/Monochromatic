/**
 The path one `edit` or `write` call names, whole, for the feed line that keeps only a preview of the call's arguments.

 A model writes a call's arguments as one JSON object, and a `write` that fills a file writes its text first and its path second.
 The feed keeps 700 characters of that object, so a call whose text is longer loses its path, which is the one field a reader of a
 run that changed the wrong file needs (issue 6636). The path itself is short, so it is kept beside the preview rather than inside
 it.

 @module
 */

import { isRecord, } from './read-pi-fields.ts';

//region Constants

/**
 The tools whose path is kept whole: the two that change a file.
 */
const TOOLS_THAT_CHANGE_A_FILE: ReadonlySet<string> = new Set([
  'edit',
  'write',
]);

//endregion Constants

//region The path of a call

/**
 The path one call names, or that it names none.
 */
export type PathOfACall =
  | { readonly path: string }
    | { readonly absent: true };

/**
 Reads the whole path of an `edit` or `write` call.

 @param toolName - the tool the model asked for

 @param input - the call's arguments, of any shape because they come from a model

 @returns the path the call names, whole, or that nothing is named

 @example
 ```ts
 const path = wholePathOfACall({ toolName: 'write', input: { content: 'x', path: '/repo/a.ts' }, });
 ```
 */
export function wholePathOfACall({
  toolName,
  input,
}: {
  readonly toolName: string;
  readonly input: unknown;
},): PathOfACall {
  /**
   Whether the call is one whose path this record keeps.
   */
  const changesAFile = TOOLS_THAT_CHANGE_A_FILE.has(toolName,);
  if (!changesAFile) {
    return { absent: true, };
  }
  if (!isRecord(input,)) {
    return { absent: true, };
  }
  /**
   The path the call names.
   */
  const { path, } = input;
  return ((typeof path) === 'string') ? { path, } : { absent: true, };
}

//endregion The path of a call
