//region Consistency marks sentence
// The marks are chosen against every text a call shows, so a text that holds a run of them lengthens them for that
// call. The sentence naming them is therefore a value of the call and opens the user turn; the rules in the system
// prompt are the same text on every call and so are written once into the cache.

/**
 The sentence that tells the judge or the writer which marks enclose the repeated wording, for the head of a user turn.

 @param marks - marks chosen against every shown text, so none of them occurs inside one

 @param shown - what the sheet calls the Chinese text the marks sit in

 @returns One sentence naming both marks

 @example
 ```ts
 const sentence = marksSentenceOf({ marks: { open: '⟦', close: '⟧', }, shown: 'paragraph', },);
 ```
 */
export function marksSentenceOf(
  {
    marks,
    shown,
  }: {
    readonly marks: {
      readonly open: string;
      readonly close: string;
    };
    readonly shown: 'ORIGINAL' | 'paragraph';
  },
): string {
  return `In each ${shown} the repeated wording is enclosed between ${marks.open} and ${marks.close}; `
    + 'the marks are not part of the text.';
}

//endregion Consistency marks sentence
