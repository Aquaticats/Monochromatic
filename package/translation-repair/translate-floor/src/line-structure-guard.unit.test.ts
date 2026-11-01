/**
 Tests for the fault that names a flattened line-structured rendering.

 WHAT THESE PIN is the shape the corpus measurement forced. The recorded
 prescription was a line-count check against the original; measured over the
 211 line-structured slices of the pinned corpus, the archive's own English
 matches its Chinese line for line on only 115, and 80 of the 96 that differ
 carry MORE lines, because an English rendering of Chinese verse legitimately
 expands. So the check names a SHORTFALL and nothing else, and the case that
 proves it is the one accepting a longer rendering.

 The blind spot has a test of its own rather than a comment, so a later
 instrument that closes it fails here and has to say so.

 Fixtures take the original in Simplified Chinese as every source in this
 corpus is. Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  compareLineCounts,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

/**
 One original whose six lines each stand as a unit.
 */
const ORIGINAL = [
  '猫醒了。',
  '太阳很暖。',
  '',
  '它数鸟。',
  '又数一遍。',
  '',
  '门开了。',
  '它不动。',
].join('\n',);

/**
 Rendering that kept every line apart, as the rule asks.
 */
const KEPT_APART = [
  'The cat wakes.',
  'The sun is warm.',
  '',
  'She counts birds.',
  'She counts again.',
  '',
  'A door swings.',
  'She does not move.',
].join('\n',);

/**
 Rendering that merged each pair into one line, which is the fault.
 */
const MERGED = [
  'The cat wakes. The sun is warm.',
  '',
  'She counts birds. She counts again.',
  '',
  'A door swings. She does not move.',
].join('\n',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: compareLineCounts.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'SAYS NOTHING ABOUT AN UNGOVERNED SLICE, even one whose rendering merged every line. Prose '
              + 'has no line-per-unit rule to break, and the semantic wrap still runs there, so a finding '
              + 'here would refuse renderings the pipeline goes on to wrap correctly',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: false,
              sourceText: ORIGINAL,
              candidateText: MERGED,
            },).length,).toBe(0,);
          },
        },),

        it({
          name:
            'NAMES A GOVERNED RENDERING THAT MERGED LINES, which is the whole fault. The wrap used to '
              + 'paper over this and could not: it splits at semantic boundaries and never joins, so over '
              + 'the 116 governed slices with multi-line blocks, wrapping a flattened passage returned only '
              + '3 exactly and 290 of 740 original lines',
          fn: async () => {
            const found = compareLineCounts({
              lineStructured: true,
              sourceText: ORIGINAL,
              candidateText: MERGED,
            },);

            expect(found.length,).toBe(1,);
          },
        },),

        it({
          name:
            'COUNTS BOTH SIDES IN THE FINDING and restates the rule, because a send-back is the model\'s '
              + 'only instruction on its second turn: it is told what it owes, what it wrote, and that the '
              + 'wording it chose is not what is being refused',
          fn: async () => {
            const [finding,] = compareLineCounts({
              lineStructured: true,
              sourceText: ORIGINAL,
              candidateText: MERGED,
            },);

            expect(finding?.includes('3 lines',),).toBe(true,);
            expect(finding?.includes('6',),).toBe(true,);
            expect(finding?.includes('LINE-STRUCTURED',),).toBe(true,);
          },
        },),

        it({
          name:
            'ACCEPTS A RENDERING CARRYING MORE LINES THAN ITS ORIGINAL, and this case is why the check '
              + 'is not an equality. 80 of the 211 governed slices in the pinned corpus have English '
              + 'carrying more lines than the Chinese; an equality check would send back nearly half of '
              + 'every governed rendering, on text nobody faulted',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: ORIGINAL,
              candidateText: `${KEPT_APART}\nShe sleeps once more.`,
            },).length,).toBe(0,);
          },
        },),

        it({
          name: 'ACCEPTS A RENDERING THAT KEPT EVERY LINE APART, which is what the rule asks for',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: ORIGINAL,
              candidateText: KEPT_APART,
            },).length,).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS NO COMMENT LINE AS A LINE OWED, since a translator\'s note is no line of the passage (ledger F-7: '
            + 'an archive that kept one of two notes was refused as merging), and names one line as "1 line"',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: `<!-- 这里的猫是橘猫 -->\n${ORIGINAL}\n<!-- (本段为第一人称) -->`,
              candidateText: `<!-- (First person.) -->\n${KEPT_APART}`,
            },).length,).toBe(0,);
            const [finding,] = compareLineCounts({
              lineStructured: true,
              sourceText: '猫醒了。\n太阳很暖。',
              candidateText: 'The cat wakes and the sun is warm.',
            },);
            expect(finding,).toContain('carries 1 line of content',);
          },
        },),

        it({
          name:
            'REFUSES A RENDERING THAT REPEATS A LINE the original carries once, and accepts a refrain the '
              + 'original itself repeats (class seventy-four, 2026-09-21): a bilingual attribution '
              + 'rendered once from its Chinese and once from its own English is the same line twice; the finding '
              + 'says "once" for an original that repeats no line, not "1 times" (ledger B98)',
          fn: async () => {
            /** Original quoting a film line in Chinese with its English beside it, attribution likewise. */
            const bilingual = '> 愿每只猫都能找到属于它的阳光。\n>\n> May every cat find a sunbeam that’s all its own.\n>\n'
              + '> 出自《猫的世界》\n>\n> From *The Cat Show*';
            /** Findings on a rendering carrying the attribution twice. */
            const doubled = compareLineCounts({
              lineStructured: true,
              sourceText: bilingual,
              candidateText: '> May every cat find its own sunshine.\n>\n'
                + '> May every cat find a sunbeam that’s all its own.\n>\n> From *The Cat Show*\n>\n> From *The Cat Show*',
            },);
            expect(doubled.length,).toBe(1,);
            expect(doubled[0],).toContain('repeats the line',);
            expect(doubled[0],).toContain('From *The Cat Show*',);
            expect(doubled[0],).toContain('2 times where the ORIGINAL repeats no line more than once.',);
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: '猫醒了。\n太阳很暖。\n猫醒了。',
              candidateText: 'The cat wakes.\nThe sun is warm.\nThe cat wakes.',
            },).length,).toBe(0,);
            expect(compareLineCounts({
              lineStructured: false,
              sourceText: bilingual,
              candidateText: '> From *The Cat Show*\n>\n> From *The Cat Show*',
            },).length,).toBe(0,);
          },
        },),

        it({
          name: 'SAYS an original holding no content line has no line to repeat, where the finding read "no line '
            + 'more than 0 times" for it, while an original repeating nothing still reads "once"',
          fn: async () => {
            /** Findings on a rendering repeating a line against an original that is one comment. */
            const againstNoLine = compareLineCounts({
              lineStructured: true,
              sourceText: '<!-- 这里的猫是橘猫 -->',
              candidateText: 'The cat naps.\nThe cat naps.',
            },);
            expect(againstNoLine,).toEqual([
              'This slice is LINE-STRUCTURED and your rendering repeats the line `The cat naps.` 2 times where '
                + 'the ORIGINAL has no line to repeat. A Chinese line and its own English beside it are one line to '
                + 'render, not two. Drop the repeat, keeping the wording you chose.',
            ],);
            /** Findings on the same rendering against an original whose one line stands once. */
            const againstOneLine = compareLineCounts({
              lineStructured: true,
              sourceText: '猫在睡觉。',
              candidateText: 'The cat naps.\nThe cat naps.',
            },);
            expect(againstOneLine,).toEqual([
              'This slice is LINE-STRUCTURED and your rendering repeats the line `The cat naps.` 2 times where '
                + 'the ORIGINAL repeats no line more than once. A Chinese line and its own English beside it are one '
                + 'line to render, not two. Drop the repeat, keeping the wording you chose.',
            ],);
          },
        },),

        it({
          name: 'READS A LINE WHOSE LETTERS ARE ALL ACCENTED as the original\'s own English beside its Han line, so a '
            + 'rendering owes the pair one line; ASCII letters alone saw no English in "Å" and refused the rendering '
            + 'for merging two lines (ledger B18)',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: '猫住在北方的小村。\nÅ',
              candidateText: 'The cat lived in a small northern village: Å.',
            },).length,).toBe(0,);
          },
        },),

        it({
          name: 'CHECKS THE PAGE BOUND where one was given, faulting a rendering whose block holding the pair\'s '
            + 'English carries more lines than the page\'s block holding it, and finds nothing without the page',
          fn: async () => {
            /**
             Original quoting a line in Chinese with its own English beside it.
             */
            const sourceText = '> 愿每只猫都能找到属于它的阳光。\n> May every cat find a sunbeam.';
            /**
             Rendering carrying the pair as two lines, a second wording above the original's English.
             */
            const candidateText = '> May every cat find its own sunshine.\n> May every cat find a sunbeam.';
            expect(compareLineCounts({
              lineStructured: true,
              sourceText,
              candidateText,
              pageText: '> May every cat find a sunbeam.',
            },),).toEqual([
              'This slice is LINE-STRUCTURED and the ORIGINAL gives the line `May every cat find a sunbeam.` twice, '
                + 'once in Chinese and once in English directly beside it; that pair is ONE line whose English is '
                + 'already its rendering, and the EXISTING TRANSLATION carries the block holding it as 1 line. '
                + 'Yours carries 2. Drop the second rendering of the pair (the Chinese line, or a second English '
                + 'wording of it), keeping the wording you chose elsewhere.',
            ],);
            expect(compareLineCounts({
              lineStructured: true,
              sourceText,
              candidateText,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'COUNTS NO PAIR where the line beside the Han one carries no letter at all, numbers '
            + 'being neither tongue, so merging the two is a fault the pairing never excused',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: '猫住在北方的小村。\n12345',
              candidateText: 'The cat lived in a small northern village. 12345',
            },),).toEqual([
              'This slice is LINE-STRUCTURED: every line stands as its own unit, so your rendering owes one line '
                + 'per line of the ORIGINAL and may never merge two into one. Yours carries 1 line of content where '
                + 'the ORIGINAL has 2. Put back the line breaks you merged, keeping the wording you chose.',
            ],);
          },
        },),

        it({
          name: 'COUNTS THE PAIR in whatever order the two languages stand, the English first here',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: 'Å\n猫住在北方的小村。',
              candidateText: 'Å. The cat lived in a small northern village.',
            },).length,).toBe(0,);
          },
        },),

        it({
          name:
            'IGNORES BLANK LINES ON BOTH SIDES, since they separate blocks rather than carry text. A '
              + 'rendering that writes a different number of them has merged nothing, and faulting it would '
              + 'name a difference the line rule never spoke about',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: ORIGINAL,
              candidateText: KEPT_APART.replaceAll('\n\n', '\n\n\n',),
            },).length,).toBe(0,);
          },
        },),

        it({
          name:
            'IGNORES A LINE THAT CARRIES ONLY A QUOTE MARKER, since a bare \'>\' separates quoted blocks '
              + 'rather than carrying text; an original writing three of them and a rendering writing one '
              + 'have merged nothing (class forty-seven)',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: '> 猫醒了。\n>\n> 太阳很暖。\n>\n> 它数鸟。',
              candidateText: '> The cat wakes.\n> The sun is warm.\n> She counts birds.',
            },).length,).toBe(0,);
          },
        },),

        it({
          name:
            'OWES NO SEPARATE LINE FOR A HAN LINE WHOSE ENGLISH STANDS BESIDE IT in the original, since '
              + 'that line is already rendered by its neighbour and carrying both would quote it twice; '
              + 'the prose lines around the pair are still owed one each (class forty-seven)',
          fn: async () => {
            /**
             Bilingual farewell: each Chinese line stands beside its English original.
             */
            const bilingual = [
              '> 愿你的每个午觉都晒得到太阳。',
              '>',
              '> May every nap of yours find the sun.',
              '>',
              '> 出自《猫的世界》',
              '>',
              '> From *The Cat Show*',
              '',
              '好了，猫，睡吧。',
              '',
              '条目贡献：[Tabby](https://example.org/tabby)',
            ].join('\n',);
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: bilingual,
              candidateText: [
                '> May every nap of yours find the sun.',
                '>',
                '> From *The Cat Show*',
                '',
                'All right, cat, sleep now.',
                '',
                'Contributor for this entry: [Tabby](https://example.org/tabby)',
              ].join('\n',),
            },).length,).toBe(0,);
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: bilingual,
              candidateText: [
                '> May every nap of yours find the sun.',
                '>',
                '> From *The Cat Show*',
                '',
                'All right, cat, sleep now. Contributor for this entry: [Tabby](https://example.org/tabby)',
              ].join('\n',),
            },).length,).toBe(1,);
          },
        },),

        it({
          name:
            'ACCEPTS A RENDERING THAT MERGED IN ONE BLOCK AND SPLIT IN ANOTHER, which is this check\'s '
              + 'one blind spot, pinned here rather than left in a comment. The count is over the whole '
              + 'slice, so the two cancel. Closing it needs per-block alignment, a larger instrument than '
              + 'the flattening this was built to catch, and an instrument that does close it should fail '
              + 'this case and say so',
          fn: async () => {
            expect(compareLineCounts({
              lineStructured: true,
              sourceText: ORIGINAL,
              candidateText: [
                'The cat wakes. The sun is warm.',
                '',
                'She counts birds.',
                'She counts again.',
                '',
                'A door swings.',
                'She does',
                'not move.',
              ].join('\n',),
            },).length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: `${validateTranslatedSlice.name} on a line-structured slice`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'REFUSES A FLATTENED RENDERING THROUGH THE STRUCTURAL GUARD, which is the wiring the skip '
              + 'depends on. Skipping the wrap without this would reopen the hole the audit measured: verse '
              + 'lines separated by a single newline sit inside ONE block, so a producer that merged them '
              + 'passed a guard comparing blocks and atoms, and the wrap silently papered over it after',
          fn: async () => {
            const validation = validateTranslatedSlice({
              sourceText: ORIGINAL,
              candidateText: MERGED,
              lineStructured: true,
            },);

            expect(validation.kind,).toBe('invalid',);
            expect(
              (validation.kind === 'invalid')
                && validation.findings.some(function namesTheRule(finding,): boolean {
                  return finding.includes('LINE-STRUCTURED',);
                },),
            ).toBe(true,);
          },
        },),

        it({
          name:
            'LEAVES THE CHECK OFF WHERE A CALLER SAID NOTHING, since the decision is a union over the '
              + 'slice AND its enclosing chunk and the slice half alone covers 55 slices where the union '
              + 'covers 211. A guard defaulting the other way would guess at that from the slice, which is '
              + 'the reading measured to be wrong',
          fn: async () => {
            expect(validateTranslatedSlice({
              sourceText: ORIGINAL,
              candidateText: MERGED,
            },).kind,).toBe('valid',);
          },
        },),
      ],
    },),
  ],
},);
