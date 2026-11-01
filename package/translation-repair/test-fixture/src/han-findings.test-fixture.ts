//region Han findings
// THE HAN FLOORS' FINDINGS AS A MODEL READS THEM, built once for the tests
// that compare whole verdicts (the Han title floor's and the Han residue
// floor's), so a wording change is made in one place and every such test
// reads it. Test support, not package source.

/**
 Finding refusing a title the original brackets in 《》 left in Han.

 @param title - title between the marks

 @returns The finding, word for word

 @example
 ```ts
 hanTitleFinding({ title: '猫猫摇篮曲', },);
 ```
 */
export function hanTitleFinding({ title, }: { readonly title: string; },): string {
  return `Your translation leaves the title 《${title}》 in Han: a work the ORIGINAL names is called by its English `
    + 'title on the page, the official English title where one exists and a translation of the title where none '
    + 'does, and the Han never stands alone as the name.';
}

/**
 Finding refusing Han left standing in a candidate's English.

 @param runs - refused runs, in the order the finding names them

 @returns The finding, word for word

 @example
 ```ts
 hanResidueFinding({ runs: ['小橘子',], },);
 ```
 */
export function hanResidueFinding({ runs, }: { readonly runs: readonly string[]; },): string {
  /**
   Runs as the finding quotes them.
   */
  const named = runs
    .map(function quoted(run,): string {
      return `"${run}"`;
    },)
    .join(', ',);
  return `Your translation leaves Han standing in its English text: ${named}. Render each in English: a person's `
    + 'handle is romanized as it is read, with its literal meaning in parentheses the first time it appears; a '
    + 'term or a title is translated; a line the ORIGINAL gives both in Chinese and in English is carried once, in '
    + 'English. The Han may follow the English in parentheses and never stands in its place.';
}

//endregion Han findings
