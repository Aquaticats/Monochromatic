/**
 Tests for the document-scale repetition check: what it names, and the three
 things it must NOT name.

 WHY IT EXISTS: the introduced-defect probe compares one edited
 region against itself, so `lintong`'s duplicated farewell was invisible to it
 at any setting. The duplication lives in no single region, only in the
 assembled document. This check reads the whole document against the archive
 the artifact now stores, which needs no model, no roster and no quota.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  findIntroducedRepetitions,
  repetitionFindings,
  whitespaceTokensOf,
} from '../dist/final/node/index.mjs';

/**
 A passage long enough to be reported, standing in for the farewell the repair
 lane said twice.
 */
const PASSAGE = 'do come back and visit the tabby by the gate again soon';

/**
 A passage LONGER than the twelve-word window, so it spans several of them.

 Every word is at least five letters, so each window clears the content-word
 threshold on its own and the merge rather than the filter is what the test
 measures.
 */
const LONG_PASSAGE =
  'afternoon sunlight arrived across weathered floorboards beneath sleeping tabby kittens breathing gently while distant harbour clocks counted quiet hours';

/**
 Twelve words exactly, so it is one window and cannot merge with itself.
 */
const FIRST_PASSAGE =
  'weathered floorboards beneath sleeping tabby kittens breathing gently through morning harbour sunlight';

/**
 {@inheritDoc FIRST_PASSAGE}

 Shares no twelve-word window with {@link FIRST_PASSAGE}, so any merge between
 the two would have come from adjacency rather than from occurrence.
 */
const SECOND_PASSAGE =
  'lanterns glimmered against darkened rooftops wherever autumn evenings settled quietly across sleeping courtyards';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: findIntroducedRepetitions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES a passage the shipped document says twice and the archive said once',
          fn: async () => {
            const findings = findIntroducedRepetitions({
              archiveText: `The kitten dozes. ${PASSAGE}. The end.`,
              shippedText: `The kitten dozes. ${PASSAGE}. And again: ${PASSAGE}. The end.`,
            },);
            expect(findings.length,).toBe(1,);
            expect(findings[0]?.archiveCount,).toBe(1,);
            expect(findings[0]?.shippedCount,).toBe(2,);
            // The reported phrase carries whatever punctuation sits on its tokens,
            // since words are whitespace-separated, so it is checked against the
            // shipped text rather than against the bare passage.
            expect(`The kitten dozes. ${PASSAGE}. And again: ${PASSAGE}. The end.`,)
              .toContain(findings[0]?.phrase ?? 'nothing',);
          },
        },),
        it({
          name: 'IGNORES repetition the archive already carried, which is the author\'s and not ours',
          fn: async () => {
            // A refrain repeated on purpose. A check that fired on this would fire
            // on every poem and every list in the corpus.
            const findings = findIntroducedRepetitions({
              archiveText: `${PASSAGE}. The kitten dozes. ${PASSAGE}.`,
              shippedText: `${PASSAGE}. The kitten naps. ${PASSAGE}.`,
            },);
            expect(findings.length,).toBe(0,);
          },
        },),
        it({
          name: 'KEEPS a phrase that is a character substring of a longer finding across a word boundary, '
            + 'since `at the garden lantern` is no part of `cat the garden lanterns`',
          fn: async () => {
            // Both phrases carry two content words, which the content rule asks
            // for, and the shorter sits inside the longer only as characters.
            const findings = findIntroducedRepetitions({
              archiveText: 'The kitten dozes. The end.',
              shippedText: 'Then cat the garden lanterns were burning bright again. So cat the garden lanterns '
                + 'were burning bright today. He stood at the garden lantern before dawn. She waited at the '
                + 'garden lantern after dusk.',
            },);
            expect(findings.map(function phraseOf(finding,): string {
              return finding.phrase;
            },)
              .toSorted(),).toStrictEqual([
                'at the garden lantern',
                'cat the garden lanterns were burning bright',
              ],);
          },
        },),
        it({
          name: 'REPORTS THE LONGEST FORM rather than every substring of it',
          fn: async () => {
            // A repeated eleven-word passage also repeats as eight four-word ones,
            // and reporting those would bury the finding inside itself.
            const findings = findIntroducedRepetitions({
              archiveText: `${PASSAGE}.`,
              shippedText: `${PASSAGE}. ${PASSAGE}.`,
            },);
            expect(findings.length,).toBe(1,);
          },
        },),
        it({
          name: 'is UNAFFECTED by rewrapping, since shipped text is wrapped semantically',
          fn: async () => {
            /**
             The same words, broken across lines the way the wrapper would.
             */
            const wrapped = PASSAGE.split(' ',)
              .join('\n',);
            const findings = findIntroducedRepetitions({
              archiveText: `${PASSAGE}.`,
              shippedText: `${wrapped}. ${wrapped}.`,
            },);
            expect(findings.length,).toBe(1,);
            expect(findings[0]?.shippedCount,).toBe(2,);
          },
        },),
        it({
          name: 'names wording the pipeline INVENTED twice, which the archive never had',
          fn: async () => {
            const findings = findIntroducedRepetitions({
              archiveText: 'The kitten dozes on the windowsill.',
              shippedText: `The kitten dozes. ${PASSAGE}. ${PASSAGE}.`,
            },);
            expect(findings.length,).toBe(1,);
            expect(findings[0]?.archiveCount,).toBe(0,);
          },
        },),
        it({
          name: 'renders a finding NAMING THE SHAPE and never the passage itself',
          fn: async () => {
            // Findings are counted and compared across runs. A passage of prose
            // inside one would make every tally depend on the text it happened to
            // find, so the finding carries the shape and the counts only.
            const rendered = repetitionFindings({
              archiveText: `The kitten dozes. ${PASSAGE}.`,
              shippedText: `The kitten dozes. ${PASSAGE}. Again: ${PASSAGE}.`,
            },);
            expect(rendered.length,).toBe(1,);
            expect(rendered[0],).toContain('introduced-repetition',);
            expect(rendered[0],).toContain('archive 1',);
            expect(rendered[0],).toContain('shipped 2',);
            expect(rendered[0],).not.toContain('tabby',);
          },
        },),
        it({
          name: 'IGNORES a repeated run of function words, which any two paragraphs may share',
          fn: async () => {
            // Six words, none longer than four letters. Measured on the settled
            // artifacts, findings of this shape were the false positives: the
            // documented damage carries three substantial words, these carry none.
            const thin = 'and so it was that the';
            const findings = findIntroducedRepetitions({
              archiveText: `A cat sat. ${thin} day ended.`,
              shippedText: `A cat sat. ${thin} day ended. ${thin} night came.`,
            },);
            expect(findings.length,).toBe(0,);
          },
        },),
        it({
          name: 'IGNORES a repeated run of function words as long as the longest window, which is read apart from '
            + 'the shorter ones and filtered there too',
          fn: async () => {
            // Twelve words, none longer than four letters: one window of the
            // longest length, which the growing step would report whole had its
            // own filter let it through.
            const thin = 'and so it was that the cat sat on the mat as';
            expect(thin.split(' ',).length,).toBe(12,);
            const findings = findIntroducedRepetitions({
              archiveText: `A cat sat. ${thin} day ended.`,
              shippedText: `A cat sat. ${thin} day ended. ${thin} night came.`,
            },);
            expect(findings.length,).toBe(0,);
          },
        },),
        it({
          name: 'REPORTS ONE LONG DUPLICATION ONCE, naming its whole length, '
            + 'rather than once per window position. Growth stops at twelve words, '
            + 'so a longer passage spans many windows of exactly that length and '
            + 'the containment rule cannot merge them: they are all the same '
            + 'length, so no one of them contains another. A measurement found an '
            + '877-word duplication arriving as 866 findings, which made every '
            + 'corpus aggregate over this token a statement about one slice',
          fn: async () => {
            const findings = findIntroducedRepetitions({
              archiveText: 'The kitten dozes quietly.',
              shippedText: `${LONG_PASSAGE} ${LONG_PASSAGE}`,
            },);

            expect(findings.length,).toBe(1,);
            expect(findings[0]?.phrase
              .split(' ',)
              .length,).toBe(LONG_PASSAGE.split(' ',).length,);
            expect(findings[0]?.shippedCount,).toBe(2,);
            expect(findings[0]?.archiveCount,).toBe(0,);
          },
        },),

        it({
          name: 'REPORTS A PASSAGE SAID THREE TIMES ONCE TOO, carrying the count '
            + 'rather than repeating the finding. The walk reaches a passage again '
            + 'at each of its own occurrences, so reporting on arrival would '
            + 'reintroduce the same over-counting one level down',
          fn: async () => {
            const findings = findIntroducedRepetitions({
              archiveText: 'The kitten dozes quietly.',
              shippedText: `${LONG_PASSAGE} ${LONG_PASSAGE} ${LONG_PASSAGE}`,
            },);

            expect(findings.length,).toBe(1,);
            expect(findings[0]?.shippedCount,).toBe(3,);
          },
        },),

        it({
          name: 'KEEPS TWO SEPARATE REPEATS SEPARATE even where they abut, which '
            + 'is what makes merging on occurrences rather than on adjacency '
            + 'load-bearing: two passages sitting next to each other are not '
            + 'evidence of one passage, and a merge rule reading only the output '
            + 'order would invent a span the document never said',
          fn: async () => {
            const findings = findIntroducedRepetitions({
              archiveText: 'The kitten dozes quietly.',
              // FIRST occurrence puts them side by side, SECOND separates them, so
              // every window straddling the junction occurs exactly once and no
              // run can cross it.
              shippedText:
                `${FIRST_PASSAGE} ${SECOND_PASSAGE} The kitten dozes quietly. ${FIRST_PASSAGE} `
                  + `Another sentence entirely, unrelated. ${SECOND_PASSAGE}`,
            },);

            expect(findings.length,).toBe(2,);
            expect(findings.map(function toLength(found,): number {
              return found.phrase
                .split(' ',)
                .length;
            },),).toStrictEqual([
              FIRST_PASSAGE.split(' ',).length,
              SECOND_PASSAGE.split(' ',).length,
            ],);
          },
        },),

        it({
          name: 'stays quiet on an untouched document',
          fn: async () => {
            const findings = findIntroducedRepetitions({
              archiveText: `The kitten dozes. ${PASSAGE}.`,
              shippedText: `The kitten dozes. ${PASSAGE}.`,
            },);
            expect(findings.length,).toBe(0,);
          },
        },),
      ],
    },),

    // Pinned before the tokenizer stopped rebuilding each token a character at
    // a time (ledger B29), so the index-slicing rewrite is read against what the
    // accumulator returned, a letter beyond the first plane included.
    describe({
      name: whitespaceTokensOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS AT every whitespace run, leading and trailing ones included, and keeps punctuation on its token',
          fn: async () => {
            expect(whitespaceTokensOf({ text: '  the cat,\n\tnaps.  ', },),).toEqual(['the', 'cat,', 'naps.',],);
            expect(whitespaceTokensOf({ text: 'cat\u{3000}naps', },),).toEqual(['cat', 'naps',],);
            expect(whitespaceTokensOf({ text: 'cat\u{A0}naps', },),).toEqual(['cat', 'naps',],);
          },
        },),
        it({
          name: 'KEEPS a letter beyond the first plane whole inside its token',
          fn: async () => {
            expect(whitespaceTokensOf({ text: 'cat\u{1D49C} naps', },),).toEqual(['cat\u{1D49C}', 'naps',],);
            expect(whitespaceTokensOf({ text: '\u{1D49C}\u{1D49C} \u{1D49C}', },),).toEqual(['\u{1D49C}\u{1D49C}', '\u{1D49C}',],);
          },
        },),
        it({
          name: 'READS NO TOKEN in empty or blank text',
          fn: async () => {
            expect(whitespaceTokensOf({ text: '', },),).toEqual([],);
            expect(whitespaceTokensOf({ text: ' \n ', },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
