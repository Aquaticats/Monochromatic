/**
 The lines of a feed that a writer has finished, from a file it may still be writing.

 A reader of a run's feed reads a file another process appends to, so the text it sees can end in the middle of the line being
 written: the write crosses a page boundary, or the reader asks before the last write has landed, and the text ends inside a JSON
 string. Parsing that fragment throws `Unterminated string in JSON`, and a reader that takes the throw for damage reports a failed
 run where the run is only still writing.

 One line is only a record once its line feed is there, so the reader drops the fragment after the last line feed and parses what
 the writer finished.

 @module
 */

/**
 The byte that ends a feed's record.
 */
const LINE_FEED = '\n';

/**
 Reads the lines of a feed that the writer has finished.

 @param shown - the feed's text as it was read

 @returns every whole line, in order, and nothing of a line still being written

 @example
 ```ts
 const lines = wholeLinesOf({ shown: '{"kind":"turn_start"}\n{"kind":"sett', },);
 ```
 */
export function wholeLinesOf({ shown, }: { readonly shown: string },): readonly string[] {
  /**
   Every piece between line feeds, the last of which is the line the writer has not ended yet: empty when the text ends with its
   own line feed, and the fragment of a record otherwise.
   */
  const pieces = shown.split(LINE_FEED,);
  // That last piece is no record in either case, so it is what a reader leaves alone.
  return pieces.slice(
    0,
    -1,
  );
}
