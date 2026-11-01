import { treeNodes, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  parseMdxBody,
  requireMdxRefusal,
  withMdxExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { withoutHtmlComments, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Formulas the original does not write
// LEDGER X22 (2026-09-29): the site compiles every page with remark-math, so
// text between dollar signs is a formula there, and the strict parse now reads
// it the same way (`parse-mdx.ts`). A translate candidate in XingZ616 set a
// TeX command between dollar signs where the original has no formula, and
// nothing refused it; two dollar amounts in one paragraph form a formula the
// same way. A candidate forming more formulas than its original is refused
// before any judge reads it. Comments are cut first, as the site turns them
// into JSX comments that hold nothing it renders. THE FLOOR STANDS ASIDE where
// the strict grammar refuses either text, since what the site would render
// cannot then be read, and a refusal of that grammar is its own finding.

/**
 Node types the site renders as a formula.
 */
const FORMULA_TYPES: ReadonlySet<string> = new Set([
  'inlineMath',
  'math',
],);

/**
 What the site's grammar makes of a passage's formulas: how many it forms, or
 that the strict grammar refuses the passage, so nothing can be counted.
 */
type FormulaReading =
  | {
    readonly kind: 'read';

    /**
     Formulas the passage forms.
     */
    readonly count: number;
  }
    | {
      readonly kind: 'refused';
    };

/**
 Whether a node is one the site renders as a formula.

 @param type - node's type, read as a plain string since the formula types
 come from the math grammar's own additions to mdast

 @returns Whether it is inline or block math

 @example
 ```ts
 isFormula({ type: 'inlineMath', },); // true
 ```
 */
function isFormula({ type, }: { readonly type: string; },): boolean {
  return FORMULA_TYPES.has(type,);
}

/**
 How many formulas a text forms under the site's grammar.

 @param text - passage to read

 @returns The count, or a refusal where the strict grammar refuses the passage

 @throws {@link Error} any failure other than the grammar's refusal, which is
 not this floor's to read

 @example
 ```ts
 formulaReading({ text: 'The cat grew $2t$ grams.', },); // { kind: 'read', count: 1 }
 ```
 */
function formulaReading({ text, }: { readonly text: string; },): FormulaReading {
  try {
    return {
      kind: 'read',
      count: treeNodes({
        root: withMdxExhaustionRefused({
          parse: parseMdxBody,
          input: { body: withoutHtmlComments({ text, },), },
        },),
      },)
        .filter(isFormula,)
        .length,
    };
  }
  catch (error) {
    requireMdxRefusal({ error, },);
    return { kind: 'refused', };
  }
}

/**
 Findings for a candidate forming more formulas than its original.

 @internal

 @param sourceText - original slice

 @param candidateText - candidate under validation

 @returns One finding telling the writer what to repair, or none

 @example
 ```ts
 const findings = addedFormulaFindings({ sourceText: '猫', candidateText: 'The cat costs $5 and $10.', },);
 ```
 */
export function addedFormulaFindings(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Formulas the candidate forms.
   */
  const candidate = formulaReading({ text: candidateText, },);
  if ((candidate.kind === 'refused') || (candidate.count === 0))
    return [];
  /**
   Formulas the original forms.
   */
  const source = formulaReading({ text: sourceText, },);
  if ((source.kind === 'refused') || (candidate.count <= source.count))
    return [];
  /**
   Finding telling the writer what to repair.
   */
  const finding = 'Your translation writes a formula the ORIGINAL does not: the site renders any text between two '
    + 'dollar signs in one paragraph as mathematics. Put a backslash before each dollar sign the ORIGINAL means as '
    + 'money, or name the currency in words, and set no formula the ORIGINAL does not set.';
  return [finding,];
}

//endregion Formulas the original does not write
