/**
 Guards the archive's spelling of a Han name written inside a sentence. A
 judged polish shipped "Han Lian Yi (Cold Ripples)" over the archive's "Han
 Lianyi", because the house rule for a handle with no declared, archive or
 corpus rendering was read as if it applied, and no floor knew the archive had
 spelled the name. A candidate that writes the name's reading letters under
 another spelling is refused, with the archive's spelling named. Cat-themed
 invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original: a cited author named in a sentence, written in Han.
 */
const SOURCE_TEXT = '在她离世后，雪团子为她写下了[悼文](https://example.invalid/memorial)。';

/**
 Archive spelling that author in its sentence.
 */
const ARCHIVE_TEXT = 'After her departure, Xue Tuanzi wrote an [epitaph](https://example.invalid/memorial) for her.';

/**
 Finding the floor writes for the candidate that regrouped the spelling.
 */
const REGROUPED_FINDING = 'The original names 雪团子 and the page already spells it "Xue Tuanzi"; your translation spells it '
  + '"Xuetuan Zi". Write "Xue Tuanzi" exactly as the page does: the house rule leaves a name the archive has already '
  + 'rendered as the archive spells it.';

/**
 Findings a candidate draws against the original and a page, empty when it
 passes.

 @param candidateText - rendering under the floors

 @param declared - name pairs the front matter declares

 @param pageText - page as it stands, the archive's own wording

 @returns Findings of the composed outcome, empty for a valid candidate

 @example
 ```ts
 const findings = findingsFor({ candidateText: ARCHIVE_TEXT, },);
 ```
 */
function findingsFor(
  {
    candidateText,
    declared = [],
    pageText = ARCHIVE_TEXT,
  }: {
    readonly candidateText: string;
    readonly declared?: readonly { readonly source: string; readonly rendering: string; }[];
    readonly pageText?: string;
  },
): readonly string[] {
  /**
   Composed outcome.
   */
  const outcome = validateTranslatedSlice({
    sourceText: SOURCE_TEXT,
    candidateText,
    pageText,
    lineStructured: false,
    declared,
  },);
  return (outcome.kind === 'invalid') ? outcome.findings : [];
}

await describe({
  name: 'a candidate keeps the spelling the archive gives a Han name in its sentences',
  children: [
    describe({
      name: validateTranslatedSlice.name,
      children: [
        it({
          name: 'REFUSES a candidate that spells the name\'s letters another way, a gloss after it or none',
          fn: async () => {
            expect(findingsFor({
              candidateText: 'After her passing, Xuetuan Zi (Snow Dumpling) wrote an [epitaph](https://example.invalid/memorial) for her.',
            },),).toEqual([REGROUPED_FINDING,],);
            expect(findingsFor({
              candidateText: 'After her passing, Xuetuan Zi wrote an [epitaph](https://example.invalid/memorial) for her.',
            },),).toEqual([REGROUPED_FINDING,],);
          },
        },),
        it({
          name: 'PASSES the archive\'s own spelling, with a meaning after it or without',
          fn: async () => {
            expect(findingsFor({ candidateText: ARCHIVE_TEXT, },),).toEqual([],);
            expect(findingsFor({
              candidateText: 'After her passing, Xue Tuanzi (Snow Dumpling) wrote an [epitaph](https://example.invalid/memorial) for her.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'PASSES a candidate that does not write the name, and a page that spells it two ways',
          fn: async () => {
            expect(findingsFor({
              candidateText: 'After her passing, the cat wrote an [epitaph](https://example.invalid/memorial) for her.',
            },),).toEqual([],);
            expect(findingsFor({
              candidateText: 'After her passing, Xuetuan Zi wrote an [epitaph](https://example.invalid/memorial) for her.',
              pageText: 'After her departure, Xue Tuanzi wrote an [epitaph](https://example.invalid/memorial) for her. '
                + 'Xuetuan Zi stayed.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'PASSES a name the front matter declares, whose declared rendering governs',
          fn: async () => {
            expect(findingsFor({
              candidateText: 'After her passing, Snow Dumpling wrote an [epitaph](https://example.invalid/memorial) for her.',
              declared: [{ source: '雪团子', rendering: 'Snow Dumpling', },],
            },),).toEqual([],);
            expect(findingsFor({
              candidateText: 'After her passing, Xuetuan Zi wrote an [epitaph](https://example.invalid/memorial) for her.',
              declared: [{ source: '雪团子', rendering: 'Xuetuan Zi', },],
            },),).toEqual([],);
          },
        },),
        it({
          name: 'PASSES a page that has no spelling of the name, such as a slice the archive never carried',
          fn: async () => {
            expect(findingsFor({
              candidateText: 'After her passing, Xuetuan Zi wrote an [epitaph](https://example.invalid/memorial) for her.',
              pageText: '',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
