/**
 Tests for naturalness eligibility: which paragraphs of a repaired slice the
 lane may rewrite, and the reason recorded for every skip.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseDocument,
  selectRefinableParagraphs,
} from '../dist/final/node/index.mjs';

/**
 Prose long enough to clear the minimum, on one physical line.
 */
const LONG_PROSE =
  'The cat naps on the warm windowsill every afternoon, and when the light moves across the floor she follows it without hurry, settling again wherever it lands.';

/**
 Eligibility verdicts over one fixture document.

 @param text - slice text to parse and judge

 @returns Verdict per block in document order

 @example
 ```ts
 const verdicts = judge(LONG_PROSE,);
 ```
 */
function judge(text: string,) {
  return selectRefinableParagraphs({ document: parseDocument({ text, },), },);
}

/**
 Reason recorded for the first block, or `eligible` when it passed.

 @param text - slice text to parse and judge

 @returns Reason string for the first block

 @example
 ```ts
 expect(firstReason('## Heading',),).toBe('not-a-paragraph',);
 ```
 */
function firstReason(text: string,): string {
  /**
   Verdict for the first block of the fixture.
   */
  const [verdict,] = judge(text,);
  if (verdict === undefined)
    throw new Error('fixture produced no block',);
  return verdict.eligible ? 'eligible' : verdict.reason;
}

await describe({
  name: selectRefinableParagraphs.name,
  children: [
    it({
      name: 'admits ordinary single-line prose above the minimum length',
      fn: async () => {
        expect(firstReason(LONG_PROSE,),).toBe('eligible',);
      },
    },),

    it({
      name: 'skips headings, since only paragraphs are eligible and a new block '
        + 'kind stays ineligible until someone decides otherwise',
      fn: async () => {
        expect(firstReason(`## ${LONG_PROSE}`,),).toBe('not-a-paragraph',);
      },
    },),

    it({
      name: 'ADMITS a soft-wrapped paragraph, because a source wrap renders as '
        + 'a space and carries no authored structure. 811 of 2067 prose '
        + 'paragraphs at the pinned corpus commit carry an internal newline '
        + 'and only 29 carry a hard break, so refusing them all cost 782 '
        + 'ordinary paragraphs to protect 29',
      fn: async () => {
        /** The same prose broken across two physical lines. */
        const wrapped = LONG_PROSE.replace(
          ', and when',
          ',\nand when',
        );
        expect(firstReason(wrapped,),).toBe('eligible',);
      },
    },),

    it({
      name: 'skips a paragraph whose line ends in two spaces, which is Markdown '
        + 'for an AUTHORED break and is what verse uses. This is the case the '
        + 'newline rule was really protecting, and the only one it needed to',
      fn: async () => {
        /** The same prose with a hard break where the wrap was. */
        const broken = LONG_PROSE.replace(
          ', and when',
          ',  \nand when',
        );
        expect(firstReason(broken,),).toBe('hard-break',);
      },
    },),

    it({
      name: 'skips a paragraph whose line ends in a backslash, the other '
        + 'Markdown spelling of a hard break, so the rule does not depend on '
        + 'which spelling an author reached for',
      fn: async () => {
        /** The same prose using the backslash spelling. */
        const broken = LONG_PROSE.replace(
          ', and when',
          ',\\\nand when',
        );
        expect(firstReason(broken,),).toBe('hard-break',);
      },
    },),

    it({
      name: 'does not read a TRAILING hard-break marker as authored structure, '
        + 'since a break after the last line separates the paragraph from what '
        + 'follows rather than dividing it',
      fn: async () => {
        expect(firstReason(`${LONG_PROSE}  `,),).toBe('eligible',);
      },
    },),

    it({
      name: 'skips a paragraph carrying markup, because a break element needs '
        + 'no newline and the single-line rule alone would miss it',
      fn: async () => {
        expect(firstReason(`${LONG_PROSE}<br />`,),).toBe('carries-markup',);
      },
    },),

    it({
      name: 'skips prose below the minimum and above the maximum length',
      fn: async () => {
        expect(firstReason('The cat naps.',),).toBe('too-short',);
        expect(
          firstReason(LONG_PROSE.repeat(10,),),
        ).toBe('too-long',);
      },
    },),

    it({
      name: 'ADMITS SHORT BODY PROSE when final polish disables repair-lane length floor',
      fn: async () => {
        const verdicts = selectRefinableParagraphs({
          document: parseDocument({ text: 'She too had once faced life positively.', },),
          minimumChars: 0,
        },);
        expect(verdicts.at(0,)?.eligible,).toBe(true,);
      },
    },),

    it({
      name: 'disqualifies every block of a slice whose parse was degraded, not '
        + 'just the block that caused it',
      fn: async () => {
        /**
         Prose preceded by a comment, which the tolerant parser masks and
         reports as a finding while leaving the prose itself intact.
         */
        const degraded = `<!-- a note to translators -->\n\n${LONG_PROSE}`;

        /** Verdicts across the degraded slice. */
        const verdicts = judge(degraded,);
        expect(verdicts.length,).toBeGreaterThan(0,);
        expect(
          verdicts.every(function isSkipped(verdict,) {
            return !verdict.eligible;
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'reports a verdict for every block so the lane yield is explainable',
      fn: async () => {
        /** Mixed slice: a heading, eligible prose, and a short paragraph. */
        const mixed = `## Introduction\n\n${LONG_PROSE}\n\nThe bowl stays full.\n`;

        /** Verdicts across the mixed slice. */
        const verdicts = judge(mixed,);
        expect(verdicts.length,).toBe(3,);
        expect(
          verdicts.filter(function isEligible(verdict,) {
            return verdict.eligible;
          },).length,
        ).toBe(1,);
      },
    },),

    it({
      name: 'keeps a slice eligible when its only finding is a blanked '
        + 'invisible line, because that finding records a REPAIR rather than a '
        + 'loss: masking restores the paragraph break a byte-order mark had '
        + 'welded shut, making the tree a MORE faithful account of the bytes. '
        + 'Counting findings instead of naming their kinds disqualified a slice '
        + 'for having been repaired',
      fn: async () => {
        /**
         Two paragraphs a mark had welded, with the break restored by masking.
         */
        const welded = `${LONG_PROSE}\n\u{FEFF}\n${LONG_PROSE}\n`;

        expect(
          parseDocument({ text: welded, },)
            .parseFindings
            .map(function toKind(finding,) {
              return finding.kind;
            },),
        ).toEqual(['invisible-line-masked',],);
        expect(
          judge(welded,).filter(function isEligible(verdict,) {
            return verdict.eligible;
          },).length,
        ).toBe(2,);
      },
    },),

    it({
      name: 'still disqualifies a slice whose comment was blanked, since those '
        + 'bytes really are absent from the tree. The distinction is between '
        + 'kinds of finding, not a blanket exemption from the rule',
      fn: async () => {
        expect(
          judge(`${LONG_PROSE}\n\n<!-- note -->\n\n${LONG_PROSE}\n`,)
            .filter(function isEligible(verdict,) {
              return verdict.eligible;
            },).length,
        ).toBe(0,);
      },
    },),
  ],
},);
