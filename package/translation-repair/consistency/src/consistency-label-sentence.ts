import { ConsistencyInvariantError, } from './consistency-invariant-error.ts';

//region Consistency label sentence
// THE SENTENCE THAT NAMES THE LABELS A REPLY MAY USE (the lead's ruling of 2026-10-15). The sheets gave a judge only a
// placeholder, `<the label of the place you choose>`, and the labels appeared only in the headers of the fenced blocks, so a
// judge had to guess how to write one: seven replies of the run of TianqiChen66625 wrote `Occurrence 3`, `3` or `Candidate 2`.
// Every sheet that asks for a label now lists the exact labels, in the order it shows them, and says how to write one. The
// response format of the ask lists the same labels as an `enum` where the format is built per ask.

/**
 The sentence that lists the labels a reply may use and says how to write one.

 @param labels - labels the sheet shows, in the order it shows them

 @param fields - phrases that name where in the reply a label is written, as the reply shape spells them

 @returns One sentence for the end of the sheet, before the reply shape

 @throws {@link ConsistencyInvariantError} when no field is named, since every sheet that asks for a label names where it
 goes and the sentence would otherwise read "in ;"

 @example
 ```ts
 const sentence = labelsSentenceOf({ labels: ['OCCURRENCE 3', 'OCCURRENCE 4',], fields: ['"choice"',], },);
 ```
 */
export function labelsSentenceOf(
  {
    labels,
    fields,
  }: {
    readonly labels: readonly string[];
    readonly fields: readonly string[];
  },
): string {
  /**
   The last field, which closes the phrase.
   */
  const last = fields.at(-1,);
  if (last === undefined)
    throw new ConsistencyInvariantError({ invariant: 'a sheet asks for a label and names no field it goes in', },);
  /**
   The fields as a phrase: one, or all but the last separated by commas and the last after "and".
   */
  const where = (fields.length > 1)
    ? `${fields.slice(
      0,
      -1,
    )
      .join(', ',)} and ${last}`
    : last;
  return `The labels you may write are exactly: ${
    labels.join(', ',)
  }. Write each one as shown, with its word and its number, in ${where}; a number alone is not a label.`;
}

//endregion Consistency label sentence
