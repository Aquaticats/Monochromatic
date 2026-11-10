/**
 Folding of curly apostrophes to the ASCII apostrophe every phrase list in this
 package is written in.
 
 `UserPromptSubmit` and `Stop` payloads arrive with whatever the user's input
 method typed, so `that’s wrong` and `that's wrong` are the same claim. One
 definition of the fold keeps a future addition (another apostrophe codepoint,
 say U+02BC) from having to be made in every reader.
 
 @module
 */

/**
 Unicode left single quotation mark (`U+2018`).
 */
const LEFT_SINGLE_QUOTE = '‘';

/**
 Unicode right single quotation mark (`U+2019`), the shape most keyboards and
 autocorrect produce.
 */
const RIGHT_SINGLE_QUOTE = '’';

/**
 Folds curly single quotation marks to ASCII apostrophes so phrase lookups only
 need the straight-apostrophe variant of each entry.
 
 @param text - input text
 
 @returns text with curly single quotes replaced by `'`
 
 @example
 ```ts
 normaliseApostrophes('that’s wrong'); // => "that's wrong"
 ```
 */
function normaliseApostrophes(text: string,): string {
  return text
    .replaceAll(
    LEFT_SINGLE_QUOTE,
    "'",
    )
    .replaceAll(
    RIGHT_SINGLE_QUOTE,
    "'",
    );
}

export {
  LEFT_SINGLE_QUOTE,
  normaliseApostrophes,
  RIGHT_SINGLE_QUOTE,
};
