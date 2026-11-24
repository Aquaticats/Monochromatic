/**
 Applies a tool call's text edits so a reader can judge the file that results.
 
 A guard that reads only the new text of an edit lets two edits split one
 violation between them,
 because neither half spells it.
 Reading the whole file after the edits is the only text the tool will leave
 on disk,
 so that is the text a guard judges.
 
 @module
 */

//region Types

/**
 One text replacement, in the shape both agent harnesses report it.
 */
type TextEdit = {
  /**
   Text the call replaces.
   */
  readonly oldText: string;
  /**
   Text the call puts in its place.
   */
  readonly newText: string;
  /**
   Whether every occurrence is replaced instead of the first one.
   */
  readonly replaceAll?: boolean;
};

//endregion Types

//region Application

/**
 Applies one text edit the way the tool that reported it does.
 
 A replacement whose old text is absent leaves the file unchanged,
 which is what a call that fails its own uniqueness check leaves behind.
 
 @param text - file text before this edit
 
 @param edit - replacement to apply
 
 @returns file text after this edit
 
 @example
 ```ts
 textAfterOneEdit({ text: 'rm -rf "$d"\n', edit: { oldText: '"$d"', newText: '/literal' } });
 ```
 */
function textAfterOneEdit({
  text,
  edit,
}: {
  readonly text: string;
  readonly edit: TextEdit;
},): string {
  if (edit.replaceAll === true)
    return text.replaceAll(
      edit.oldText,
      edit.newText,
    );
  /**
   Offset of the first occurrence of the replaced text, or -1 when it is absent.
   */
  const occurrenceOffset = text.indexOf(edit.oldText,);
  if (occurrenceOffset === (-1))
    return text;
  return text.slice(
    0,
    occurrenceOffset,
  ) + edit.newText
    + text.slice(occurrenceOffset
      + edit.oldText
        .length,);
}

/**
 Applies a tool call's text edits in the order it reports them.
 
 @param text - file text before the call, or empty text for a file the call creates
 
 @param edits - replacements the call makes, in order
 
 @returns file text the call leaves on disk
 
 @example
 ```ts
 textAfterEdits({ text: 'a\n', edits: [{ oldText: 'a', newText: 'b' }] }); // 'b\n'
 ```
 */
function textAfterEdits({
  text,
  edits,
}: {
  readonly text: string;
  readonly edits: readonly TextEdit[];
},): string {
  return edits
    .reduce(
    function applyEdit(
      current,
      edit,
    ): string {
      return textAfterOneEdit({
        text: current,
        edit,
      },);
    },
    text,
    );
}

//endregion Application

export {
  textAfterEdits,
  textAfterOneEdit,
};
export type { TextEdit, };
