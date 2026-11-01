/**
 Tests that the structural floor still floors when the page refuses the strict
 grammar.

 WHAT THIS FILE EXISTS TO STOP, measured on the sixth consolidation bed rather
 than imagined. A slice boundary fell between an opening `details` tag and its
 closing tag, so the page span carried no closing tag and the strict MDX
 grammar refused it. The floor then had no block list to compare against, fell
 back to the original alone, and a 164-character rendering passed against a
 3875-character page. Both lanes had carried that page whole; only the third
 rendering dropped it, and it shipped.

 A CHECK THAT CANNOT RUN MUST NOT ANSWER YES. The page side now downgrades to
 plain markdown, which reads the same span as a paragraph followed by an html
 block, and the floor refuses the one-paragraph candidate on its own evidence.

 WHY THE FIXTURE IS UNBALANCED ON PURPOSE. A WELL-FORMED `details` element
 parses under the strict grammar perfectly well, so a tidy fixture proves
 nothing here: it would pass before the fix and after it. Only a span cut
 through an element reproduces the refusal.
 SINCE 2026-09-07 A SPAN CUT BETWEEN A CONTAINER'S TAGS READS STRICTLY: the
 lone tag is masked before the parse and carried as a `container-tag` atom
 (`mask-container-tags.ts`), because `container-extents.ts` cuts every
 container whose blocks fall in different slices exactly that way, and the
 Huasheng pass stopped on it. The relaxed path is therefore exercised here by
 a page the grammar refuses for another reason, an inline tag torn from its
 closer on the same line, and the cut-container page has its own cases.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type SliceValidation,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Chinese standing in for the note, which is all this slice's original is.
 */
const SOURCE_TEXT = '（注：内容可能引起不适）';

/**
 Page whose span was cut between the opening tag and its close, so the strict
 grammar has a complaint no producer could answer.
 */
const CUT_PAGE = `(Warning: this account may be upsetting.)

<details>
<summary>The cat's requests</summary>
> Feed the birds at dawn.
> Leave the window open.`;

/**
 Same page, closed properly, which the strict grammar reads without help.
 */
const WHOLE_PAGE = `${CUT_PAGE}
</details>`;

/**
 Page torn through an inline element, which no mask reads for the grammar:
 the `summary` opener has content on its line and no closer anywhere.
 */
const TORN_PAGE = `(Warning: this account may be upsetting.)

<summary>The cat's requests
> Feed the birds at dawn.
> Leave the window open.`;

/**
 Candidate carrying the note and nothing that followed it.
 */
const NOTE_ONLY = '(Note: this account may be upsetting; please be aware before reading.)';

/**
 Pulls the findings out of a verdict that must be a refusal.

 NARROWS BY THROWING rather than by optional chaining, so a verdict that
 unexpectedly passed fails the case where it happened instead of silently
 comparing an empty string.

 @param verdict - what the floor answered

 @returns Findings joined, one per line

 @throws {@link Error} when the verdict was not a refusal

 @example
 ```ts
 const findings = findingsOf({ verdict, },);
 ```
 */
function findingsOf({ verdict, }: { readonly verdict: SliceValidation; },): string {
  if (verdict.kind !== 'invalid')
    throw new Error(`expected a refusal, got ${verdict.kind}`,);

  return verdict.findings.join('\n',);
}

//endregion Fixtures

await describe({
  name: 'the floor holds on an unparseable page',
  children: [
    it({
      name: 'REFUSES a candidate that drops a page the strict grammar cannot read',
      fn: async () => {
        // Before the page side downgraded, this returned valid: the floor had
        // no blocks, and the candidate matched the one-block original.
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: NOTE_ONLY,
          pageText: TORN_PAGE,
        },);

        expect(verdict.kind,).toBe('invalid',);
        expect(findingsOf({ verdict, },),).toContain('PAGE AS IT STANDS',);
      },
    },),

    it({
      name: 'TELLS the producer what the relaxed reading found, not that parsing failed',
      fn: async () => {
        // The finding has to be actionable. A producer cannot fix the archive's
        // grammar, but it can carry the blocks the page has.
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: NOTE_ONLY,
          pageText: TORN_PAGE,
        },);

        expect(findingsOf({ verdict, },),).toContain('html',);
      },
    },),

    it({
      name: 'REFUSES the same drop when the page parses strictly, which is the positive control',
      fn: async () => {
        // Proves the "REFUSES a candidate that drops a page the strict grammar
        // cannot read" case is about the DOWNGRADE and not about the
        // floor in general: a well-formed page has always been floored on.
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: NOTE_ONLY,
          pageText: WHOLE_PAGE,
        },);

        expect(verdict.kind,).toBe('invalid',);
      },
    },),

    it({
      name: 'REFUSES even a faithful candidate here, at the candidate parse rather than the floor',
      fn: async () => {
        // NOT AN OVERSIGHT, and worth pinning. A candidate that reproduces the
        // cut page exactly is refused because the CANDIDATE side stays strict,
        // which is the older rule and the right one: text we cannot parse is
        // text we should not splice into a page.
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: TORN_PAGE,
          pageText: TORN_PAGE,
        },);

        expect(verdict.kind,).toBe('invalid',);
        expect(findingsOf({ verdict, },),).toContain('could not be parsed as Markdown',);
      },
    },),

    it({
      name: 'NAMES the grammar behind a pass, so a downgrade is never silent',
      fn: async () => {
        // A pass carries no findings to inspect, so without this the reader of
        // a valid verdict cannot tell a strict reading from a relaxed one.
        const relaxed = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: TORN_PAGE,
          pageText: TORN_PAGE,
        },);
        const strict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: NOTE_ONLY,
          pageText: '',
        },);

        // The relaxed case is a refusal for other reasons; what matters is that
        // a pass on a strictly read page says so.
        expect(relaxed.kind,).toBe('invalid',);
        expect(strict.kind,).toBe('valid',);
        expect(strict.kind === 'valid' ? strict.pageGrammar : 'not-valid',).toBe('absent',);
      },
    },),

    it({
      name: 'READS a page cut between a container\'s tags STRICTLY, refusing a candidate that drops '
        + 'the lone tag and passing one that carries it (Huasheng, 2026-09-07)',
      fn: async () => {
        // The cut page was the fail-closed case until 2026-09-07; it is now the
        // ordinary case, since the extents cut every split container this way.
        const dropped = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: NOTE_ONLY,
          pageText: CUT_PAGE,
        },);
        const carried = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: CUT_PAGE,
          pageText: CUT_PAGE,
        },);

        expect(dropped.kind,).toBe('invalid',);
        expect(findingsOf({ verdict: dropped, },),).toContain('container-tag <details>',);
        expect(findingsOf({ verdict: dropped, },),).not.toContain('html',);
        expect(carried.kind,).toBe('valid',);
        expect(carried.kind === 'valid' ? carried.pageGrammar : 'not-valid',).toBe('strict',);
      },
    },),

    it({
      name: 'REFUSES a well-formed candidate at a cut slice still, since a closed element is a '
        + 'different block from the opener the page owns',
      fn: async () => {
        // The page owns the opener alone and reads as three blocks with a tag
        // atom; a candidate that closes the element reads as one flow element,
        // which is not what the page has. The incumbent stays.
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: WHOLE_PAGE,
          pageText: CUT_PAGE,
        },);

        expect(verdict.kind,).toBe('invalid',);
        expect(findingsOf({ verdict, },),).toContain('mdxJsxFlowElement',);
      },
    },),
  ],
},);
