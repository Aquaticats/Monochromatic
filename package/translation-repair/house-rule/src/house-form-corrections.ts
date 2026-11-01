//region House form corrections
// The house rules of FORM, listed once (ledger S14): the corrections a rewrite
// may make without changing what a passage says. The polish gate, the refiner,
// the naturalness review and the refine selection each listed their own, in
// their own words, which is the drift ledger S5 found in the apparatus lists.

/**
 Every house correction of form, as the form the text is brought to.

 @example
 ```ts
 const rule = `Prefer a polish bringing the base to the house form (${HOUSE_FORM_CORRECTIONS}).`;
 ```
 */
export const HOUSE_FORM_CORRECTIONS: string = 'the past tense for the life of a person who has died (never for quoted words), '
  + 'singular they for a TA, English for a word left in Han, chat shorthand spelled out, Canadian spelling, '
  + 'and a month-first date';

/**
 What a house correction of form is, for a sheet asking whether meaning
 changed: nothing.

 WRITTEN AGAINST "Says exactly what the CURRENT text says" (ledger S14), which
 refused a candidate whose only change was the past tense for a life that has
 ended, the correction the house tense rule asks for.

 @example
 ```ts
 const criterion = `Nothing added or dropped. ${HOUSE_FORM_CORRECTION_KEEPS_MEANING}`;
 ```
 */
export const HOUSE_FORM_CORRECTION_KEEPS_MEANING: string = `A change a house rule of form makes (${HOUSE_FORM_CORRECTIONS}) is not a change to what the text says.`;

//endregion House form corrections
