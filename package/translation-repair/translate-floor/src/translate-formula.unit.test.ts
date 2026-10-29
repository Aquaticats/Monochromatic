/**
 Guards ledger X22 (2026-09-29): the site compiles every page with
 remark-math, so text between dollar signs is a formula there. A translate
 candidate in XingZ616 wrote a TeX command between dollar signs where the
 original has no formula, and nothing refused it; a pair of dollar amounts in
 one paragraph forms a formula the same way. A candidate forming more
 formulas than the original is refused before any judge reads it; one that
 keeps the original's formula, escapes its dollar signs, or keeps them inside
 a JSX attribute is not.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original with no formula: the kitten's treats cost money.
 */
const TREATS = '小猫的零食要五美元,猫窝要十美元。';

/**
 Original carrying a formula of its own.
 */
const WITH_FORMULA = '小猫每天长大 $w(t) = 2t$ 克。';

/**
 Original carrying a photo list whose paths open JSX template strings.
 */
const PHOTOS = `<PhotoScroll photos={['\${cat}/nap.webp', '\${cat}/bowl.webp']} />\n\n小猫在睡觉。`;

/**
 Finding text the floor opens with, so a case can tell this floor's refusal
 from another floor's.
 */
const FORMULA_FINDING = 'Your translation writes a formula';

/**
 Whether a validation refused the candidate with this floor's finding.

 @param validation - verdict of `validateTranslatedSlice`

 @returns Whether the formula finding is among its findings

 @example
 ```ts
 refusedForFormula({ validation, },);
 ```
 */
function refusedForFormula({ validation, }: {
  readonly validation: ReturnType<typeof validateTranslatedSlice>;
},): boolean {
  return (validation.kind === 'invalid') && validation.findings.some(function isFormula(finding,): boolean {
    return finding.startsWith(FORMULA_FINDING,);
  },);
}

await describe({
  name: 'formulas the original does not write (ledger X22)',
  children: [
    it({
      name: 'REFUSES a pair of dollar amounts in one paragraph, which the site reads as a formula',
      fn: async () => {
        expect(refusedForFormula({
          validation: validateTranslatedSlice({
            sourceText: TREATS,
            candidateText: 'The kitten\'s treats cost $5 and its bed $10.',
          },),
        },),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES a TeX command set between dollar signs where the original has no formula',
      fn: async () => {
        expect(refusedForFormula({
          validation: validateTranslatedSlice({
            sourceText: TREATS,
            candidateText: String.raw`The kitten's treats cost $\text{five dollars}$ and its bed ten.`,
          },),
        },),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES a formula in a paragraph whose HTML comment stands outside it, which the strict grammar '
        + 'would refuse unless the comment is cut first',
      fn: async () => {
        expect(refusedForFormula({
          validation: validateTranslatedSlice({
            sourceText: TREATS,
            candidateText: '<!-- priced by the cat --> The kitten\'s treats cost $5 and its bed $10.',
          },),
        },),).toBe(true,);
      },
    },),
    it({
      name: 'PASSES dollar amounts with each sign escaped, and a single dollar amount',
      fn: async () => {
        expect(refusedForFormula({
          validation: validateTranslatedSlice({
            sourceText: TREATS,
            candidateText: String.raw`The kitten's treats cost \$5 and its bed \$10.`,
          },),
        },),).toBe(false,);
        expect(refusedForFormula({
          validation: validateTranslatedSlice({
            sourceText: TREATS,
            candidateText: 'The kitten\'s treats cost $5, and its bed ten dollars.',
          },),
        },),).toBe(false,);
      },
    },),
    it({
      name: 'PASSES the original\'s own formula kept, and dollar signs inside a JSX attribute',
      fn: async () => {
        expect(refusedForFormula({
          validation: validateTranslatedSlice({
            sourceText: WITH_FORMULA,
            candidateText: 'The kitten grows $w(t) = 2t$ grams a day.',
          },),
        },),).toBe(false,);
        expect(refusedForFormula({
          validation: validateTranslatedSlice({
            sourceText: PHOTOS,
            candidateText: `<PhotoScroll photos={['\${cat}/nap.webp', '\${cat}/bowl.webp']} />\n\nThe kitten is asleep.`,
          },),
        },),).toBe(false,);
      },
    },),
  ],
},);
