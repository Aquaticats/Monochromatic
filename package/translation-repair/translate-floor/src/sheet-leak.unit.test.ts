/**
 Guards class forty-four (Mio27, 2026-09-17): a candidate that carries one of
 the sheet's own evidence blocks, copied from the writer's sheet, is refused
 before any judge reads it. Cat-themed invention throughout; no corpus content
 appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  sheetLeakFindings,
  type SliceValidation,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

/**
 Original with a picture component and a rule below it.
 */
const SOURCE = `<PhotoScroll photos={[\n'\${path}/photos/cat1.webp'\n]} />\n\n---`;

/**
 Rendering that copied the sheet's transcript block after the component.
 */
const LEAKED = `${SOURCE}\n\n===== WHAT THE PICTURES HERE SAY =====\nPICTURE cat1.webp\nreader-one:\n[left] 猫在睡觉`;

/**
 Fenced header labels the sheets write, each with the label the finding
 names: the listed head the label carries, or the label itself where no
 listed head covers it.
 */
const FENCED_LABELS: readonly (readonly [string, string,])[] = [
  [
    'WHAT THE JUDGES FOUND, claims to check against the original',
    'WHAT THE JUDGES FOUND',
  ],
  [
    'REQUIRED FINDINGS from independent absolute-quality review',
    'REQUIRED FINDINGS',
  ],
  [
    'PRIOR CORRECTION STRATEGIES THAT FAILED; choose a materially different approach',
    'PRIOR CORRECTION STRATEGIES',
  ],
  [
    'ORIGINAL (Chinese), the standard',
    'ORIGINAL (Chinese)',
  ],
  [
    'SURROUNDING ORIGINAL (Chinese), context only',
    'ORIGINAL (Chinese)',
  ],
  [
    'REJECTED CANDIDATE 1',
    'REJECTED CANDIDATE',
  ],
  [
    'A LABEL NO LIST NAMES YET',
    'A LABEL NO LIST NAMES YET',
  ],
];

/**
 Finding refusing a candidate that carries a sheet block.

 @param label - label the finding names: a listed head, or a fenced line's
 own label where no listed head covers it

 @returns Finding, word for word

 @example
 ```ts
 leakFinding({ label: 'ATTESTED DETAILS', },);
 ```
 */
function leakFinding({ label, }: { readonly label: string; },): string {
  return `Your translation carries the sheet's own "${label}" block, which is evidence shown to you and never part `
    + 'of the passage. Render the ORIGINAL alone and leave every fenced block out.';
}

/**
 Verdict refusing a candidate that carries a sheet block.

 @param label - label the finding names

 @returns Whole verdict

 @example
 ```ts
 leakRefusal({ label: 'ATTESTED DETAILS', },);
 ```
 */
function leakRefusal({ label, }: { readonly label: string; },): SliceValidation {
  return {
    kind: 'invalid',
    findings: [leakFinding({ label, },),],
  };
}

await describe({
  name: 'sheet evidence copied into a candidate (class forty-four)',
  children: [
    ...[
      'WHAT THE PICTURES HERE SAY',
      'ATTESTED DETAILS',
      'CITED REFERENCES',
      'EXISTING TRANSLATION',
    ].map(function refusesLabel(label,) {
      return it({
        name: `REFUSES a candidate carrying the sheet's ${label} block`,
        fn: async () => {
          expect(validateTranslatedSlice({
            sourceText: '猫在睡觉。',
            candidateText: `The cat is sleeping.\n\n===== ${label} =====\nsomething copied`,
          },),).toEqual(leakRefusal({ label, },),);
        },
      },);
    },),
    it({
      name: 'REFUSES the transcript block copied under a picture component, the Mio27 shape',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          pageText: SOURCE,
          candidateText: LEAKED,
        },),).toEqual(leakRefusal({ label: 'WHAT THE PICTURES HERE SAY', },),);
      },
    },),
    ...FENCED_LABELS.map(function refusesFenced([label, named,],) {
      return it({
        name: `REFUSES the fenced "${label}" block, naming "${named}" (ledger F-8 and E7: the label list had fallen `
          + 'behind the sheets)',
        fn: async () => {
          // THE FLOOR ITSELF, not the composed verdict: there the added block
          // is refused by the block comparison first, which says nothing
          // about whether this floor reads the label.
          expect(sheetLeakFindings({
            sourceText: '猫在睡觉。',
            pageText: 'The cat is sleeping.',
            candidateText: `The cat is sleeping.\n\n===== ${label} =====\n- the cat is asleep`,
          },),).toEqual([leakFinding({ label: named, },),],);
        },
      },);
    },),
    it({
      name: 'REFUSES a sheet head copied without its fence, and ACCEPTS a setext underline, a fenced line the '
        + 'original and the page carry, and a fence with no label between its runs',
      fn: async () => {
        expect(sheetLeakFindings({
          sourceText: '猫在睡觉。',
          pageText: '',
          candidateText: 'The cat is sleeping.\n\nWHAT THE JUDGES FOUND: the cat is asleep.',
        },),).toEqual([leakFinding({ label: 'WHAT THE JUDGES FOUND', },),],);
        expect(sheetLeakFindings({
          sourceText: '猫的日记\n=====\n\n猫在睡觉。',
          pageText: '',
          candidateText: 'The Cat\'s Diary\n=====\n\nThe cat is sleeping.',
        },),).toStrictEqual([],);
        expect(sheetLeakFindings({
          sourceText: '===== 猫 =====\n\n猫在睡觉。',
          pageText: '===== Cat =====\n\nThe cat is sleeping.',
          candidateText: '===== Cat =====\n\nThe cat is sleeping.',
        },),).toStrictEqual([],);
        expect(sheetLeakFindings({
          sourceText: '猫在睡觉。',
          pageText: '',
          candidateText: 'The cat is sleeping.\n\n=====   =====\n\nThe cat woke.',
        },),).toStrictEqual([],);
      },
    },),
    it({
      name: 'ACCEPTS a fence whose runs hold only what shows a reader nothing, as it accepts one with no label '
        + 'between them: a zero-width space, a Hangul filler, or both among spaces',
      fn: async () => {
        expect([
          '\u{200B}',
          '\u{3164}',
          ' \u{200B} \u{3164} ',
        ].map(function findingsBetween(filling,) {
          return sheetLeakFindings({
            sourceText: '猫在睡觉。',
            pageText: '',
            candidateText: `The cat is sleeping.\n\n=====${filling}=====\n\nThe cat woke.`,
          },);
        },),).toEqual([
          [],
          [],
          [],
        ],);
      },
    },),
    it({
      name: 'REFUSES the editor sheet\'s region marker and its unfenced line heads copied into a rendering, '
        + 'unless the original or the page carries them (ledger B24)',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫在睡觉。',
          candidateText: 'The cat «REGION 3» is sleeping.',
        },),).toEqual(leakRefusal({ label: '«REGION', },),);
        expect(validateTranslatedSlice({
          sourceText: '猫在睡觉。',
          candidateText: 'CURRENT TEXT: The cat is sleeping.',
        },),).toEqual(leakRefusal({ label: 'CURRENT TEXT:', },),);
        expect(validateTranslatedSlice({
          sourceText: '猫在睡觉。',
          candidateText: 'CONTEXT: ...the cat is sleeping...',
        },),).toEqual(leakRefusal({ label: 'CONTEXT: ...', },),);
        expect(sheetLeakFindings({
          sourceText: '猫在睡觉。',
          pageText: 'CURRENT TEXT: The cat is sleeping.',
          candidateText: 'CURRENT TEXT: The cat is sleeping.',
        },),).toStrictEqual([],);
      },
    },),
    it({
      name: 'ACCEPTS a rendering that leaves the sheet blocks out',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫在睡觉。',
          candidateText: 'The cat is sleeping.',
        },),).toEqual({
          kind: 'valid',
          pageGrammar: 'absent',
        },);
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          pageText: SOURCE,
          candidateText: SOURCE,
        },),).toEqual({
          kind: 'valid',
          pageGrammar: 'strict',
        },);
      },
    },),
  ],
},);
