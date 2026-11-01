/**
 What a judging sheet says about tense candidates, and how a case finds the
 block that says it, so a sheet case asserts the whole block rather than a
 phrase of it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

//region Tense sheet fixture

/**
 The explanation every tense block opens with, as the sheets print it.
 */
export const TENSE_EXPLANATION: string = [
  'These sentences match present-tense shapes in a memorial page about a person who has died.',
  'Check each against the house tense rule. A candidate may be a trait, a standing fact or a quotation',
  'the rule leaves alone, so it is a question for you and not a finding against the text.',
].join('\n',);

/**
 Paragraphs of one message that name tense candidates, each a whole block of a
 fenced sheet, which holds no blank line of its own.

 @param content - one rendered message

 @returns Every paragraph naming tense candidates, in the order the sheet prints them

 @example
 ```ts
 const blocks = tenseBlocksOf({ content: user.content, },);
 ```
 */
export function tenseBlocksOf({ content, }: { readonly content: string; },): readonly string[] {
  return content
    .split('\n\n',)
    .filter(function namesTenseCandidates(paragraph,): boolean {
      return paragraph.includes('TENSE CANDIDATES',);
    },);
}

/**
 Labels of the tense groups each slice record carries, so a round trip case
 states which texts a writer found a candidate in without restating the groups.

 @param slices - records a writer returned, each with the finding or without

 @returns Per record the labels of its groups in recorded order, an empty list for a record without the finding

 @example
 ```ts
 const labels = tenseLabelsOf({ slices: recorded, },);
 ```
 */
export function tenseLabelsOf(
  { slices, }: {
    readonly slices: readonly {
      readonly tenseCandidates?: readonly { readonly label: string; }[];
    }[];
  },
): readonly (readonly string[])[] {
  return slices.map(function labelsOfSlice({ tenseCandidates = [], },): readonly string[] {
    return tenseCandidates.map(function labelOf({ label, },): string {
      return label;
    },);
  },);
}

//endregion Tense sheet fixture
