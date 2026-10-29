/**
 Guards ledger A17: a signer's handle the ORIGINAL writes in Han and no
 declared or archive rendering covers is romanized with its literal meaning in
 parentheses (owner, 2026-09-22; the house policy's handle rule). Over the
 stored artifacts, 45 of the 51 such page signers shipped bare everywhere they
 were rendered, since no floor asked a writer for the gloss and the page passes
 cannot invent one; and a handle the archive also left in Han passed the Han
 residue floor, whose page excuse covers it. Cat-themed invention throughout;
 no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original: a line and a signature whose signer writes in Han.
 */
const SOURCE_TEXT = '猫在窗台上睡觉。\n\n——橘猫，2021年';

/**
 Archive that left the signer in Han.
 */
const ARCHIVE_IN_HAN = 'The cat sleeps on the windowsill.\n\n——橘猫, 2021';

/**
 What every signer finding tells the writer to do for this signer: the
 handle's reading, and the house rule's own example rather than a template.
 */
const INSTRUCTION = 'Write Jumao, followed by what 橘猫 means in English in parentheses, as the house rule writes '
  + 'Jinxin (Brocade Heart); the page keeps the meaning at its first appearance and drops it after that.';

/**
 Finding refusing a reading written with no meaning after it.

 @param bare - reading as the candidate writes it

 @returns Finding, word for word

 @example
 ```ts
 unglossedFinding({ bare: 'Jumao', },);
 ```
 */
function unglossedFinding({ bare, }: { readonly bare: string; },): string {
  return `The signature names 橘猫 and your translation writes "${bare}" with no literal meaning. ${INSTRUCTION}`;
}

/**
 Findings a candidate draws against the original and a page, empty when it
 passes.

 @param candidateText - rendering under the floors

 @param pageText - page as it stands, the archive's own wording

 @param declared - name pairs the front matter declares

 @returns Findings of the composed verdict, empty for a valid candidate

 @example
 ```ts
 const findings = findingsFor({ candidateText: 'The cat sleeps.\n\n—— Jumao, 2021', },);
 ```
 */
function findingsFor(
  {
    candidateText,
    pageText = ARCHIVE_IN_HAN,
    declared = [],
  }: {
    readonly candidateText: string;
    readonly pageText?: string;
    readonly declared?: readonly { readonly source: string; readonly rendering: string; }[];
  },
): readonly string[] {
  /**
   Composed verdict.
   */
  const verdict = validateTranslatedSlice({
    sourceText: SOURCE_TEXT,
    candidateText,
    pageText,
    lineStructured: false,
    declared,
  },);
  return (verdict.kind === 'invalid') ? verdict.findings : [];
}

await describe({
  name: 'signer handle floor (ledger A17)',
  children: [
    it({
      name: 'REFUSES THE PINYIN READING WITH NO LITERAL MEANING, and names the reading to gloss',
      fn: async () => {
        expect(findingsFor({
          candidateText: 'The cat sleeps on the windowsill.\n\n—— Jumao, 2021',
        },),).toEqual([unglossedFinding({ bare: 'Jumao', },),],);
      },
    },),
    it({
      name: 'REFUSES A SPACED OR TONE-MARKED READING with no meaning the same way',
      fn: async () => {
        for (const rendering of [
          'Ju Mao',
          'Júmāo',
          'JUMAO',
        ]) {
          expect(findingsFor({
            candidateText: `The cat sleeps on the windowsill.\n\n—— ${rendering}, 2021`,
          },),).toEqual([unglossedFinding({ bare: rendering, },),],);
        }
      },
    },),
    it({
      name: 'REFUSES THE SIGNER LEFT IN HAN though the archive left it in Han too, which the Han residue '
        + 'floor excuses as Han the page already carries',
      fn: async () => {
        expect(findingsFor({
          candidateText: 'The cat sleeps on the windowsill.\n\n——橘猫, 2021',
        },),).toEqual([`The signature names 橘猫 and your translation leaves it in Han. ${INSTRUCTION}`,],);
      },
    },),
    it({
      name: 'REFUSES A SIGNER THAT SHOWS A READER NOTHING, a zero-width space or a Hangul filler among spaces '
        + 'or alone, and asks for the reading with its meaning as it does of a signer left in Han',
      fn: async () => {
        for (const rendering of [
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ]) {
          expect(findingsFor({
            candidateText: `The cat sleeps on the windowsill.\n\n——${rendering}, 2021`,
          },),).toEqual([`The signature names 橘猫 and your translation signs it with nothing a reader can see. ${INSTRUCTION}`,],);
        }
      },
    },),
    it({
      name: 'TAKES NO PAGE SIGNER THAT SHOWS A READER NOTHING for a rendering that governs the signer, so a bare '
        + 'reading is still refused beside it',
      fn: async () => {
        expect(findingsFor({
          candidateText: 'The cat sleeps on the windowsill.\n\n—— Jumao, 2021',
          pageText: 'The cat sleeps on the windowsill.\n\n——\u{200B}, 2021',
        },),).toEqual([unglossedFinding({ bare: 'Jumao', },),],);
      },
    },),
    it({
      name: 'PASSES THE READING WITH ITS MEANING in parentheses',
      fn: async () => {
        expect(findingsFor({
          candidateText: 'The cat sleeps on the windowsill.\n\n—— Jumao (Orange Cat), 2021',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES A PLACEHOLDER IN THE PARENTHESES, the one wrong meaning that can be predicted: a writer '
        + 'echoing the finding\'s own words back would otherwise ship glossed',
      fn: async () => {
        for (const placeholder of [
          'its literal meaning in English',
          'literal meaning',
          'Meaning',
        ]) {
          expect(findingsFor({
            candidateText: `The cat sleeps on the windowsill.\n\n—— Jumao (${placeholder}), 2021`,
          },),).toEqual([
            `The signature names 橘猫 and the parentheses after "Jumao" hold a placeholder, not a meaning. ${INSTRUCTION}`,
          ],);
        }
      },
    },),
    it({
      name: 'LEAVES A SIGNER THE ARCHIVE RENDERS IN LATIN LETTERS to the archive\'s authority, and one a '
        + 'declared pair renders to the declaration',
      fn: async () => {
        expect(findingsFor({
          candidateText: 'The cat sleeps on the windowsill.\n\n—— Marmalade, 2021',
          pageText: 'The cat sleeps on the windowsill.\n\n—— Marmalade, 2021',
        },),).toEqual([],);
        expect(findingsFor({
          candidateText: 'The cat sleeps on the windowsill.\n\n—— Marmalade, 2021',
          pageText: '',
          declared: [{ source: '橘猫', rendering: 'Marmalade', },],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES A SIGNER THE ORIGINAL ITSELF SIGNS IN LATIN LETTERS, which asks for no reading',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫在窗台上睡觉。\n\n——Maomao，2021年',
          candidateText: 'The cat sleeps on the windowsill.\n\n—— Maomao, 2021',
        },),).toEqual({
          kind: 'valid',
          pageGrammar: 'absent',
        },);
      },
    },),
  ],
},);
