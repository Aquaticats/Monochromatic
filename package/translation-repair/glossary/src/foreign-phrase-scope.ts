//region Foreign phrase scope
// LEDGER S20. The critic and editor sheets keep a phrase the ORIGINAL writes
// in another language in its wording with its meaning alongside, and the
// critic reports its absence as policy/foreign-phrase-gloss. A Japanese title
// or handle is such a phrase by the letter of that rule, yet the house title
// and handle rules let its English form stand alone, the original wording at
// most after it in parentheses; so a correct *The Spider's Thread* could be
// reported for dropping 蜘蛛の糸. Both sheets carry this one sentence.

/**
 Sentence taking names and titles out of the foreign-phrase rule.

 @example
 ```ts
 const rule = `- Keep any phrase ... alone. ${FOREIGN_PHRASE_NAME_TITLE_SCOPE}`;
 ```
 */
export const FOREIGN_PHRASE_NAME_TITLE_SCOPE: string =
  'A name or the title of a work is no such phrase: it follows the house name and title rules, where its English '
    + 'form stands and the original wording may follow it in parentheses.';

//endregion Foreign phrase scope
