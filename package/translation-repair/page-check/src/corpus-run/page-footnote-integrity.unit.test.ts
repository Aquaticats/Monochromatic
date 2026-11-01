/**
 Tests the footnote check of the page that ships, which reports the defects the page introduces over its archive and
 never stops the entry (owner: "no matter what, we must ship"; a page-agreement check ships with the defect reported).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertPageFootnotesIntact,
  PageFootnoteIntegrityError,
} from '../../dist/final/node/index.mjs';

/**
 Archive wording with no footnote at all, as two paragraphs.
 */
const ARCHIVE = 'The cat sleeps in warm sunlight.\n\nThe cat naps on the sill.\n';

/**
 The archive with a definition the translate lane wrote at its tail, while no paragraph refers to it: the page of
 XingZ6023, whose slice 44 kept an archive without the reference and whose slice 115 shipped the definition.
 */
const ORPHANED = `${ARCHIVE}\n[^3]: That is, a sunbeam.\n`;

await describe({
  name: assertPageFootnotesIntact.name,
  children: [
    it({
      name: 'ACCEPTS a page whose footnote graph is as whole as the archive\'s',
      fn: async () => {
        expect(() => assertPageFootnotesIntact({
          entryId: 'BookshopCat',
          pageText: 'The cat naps in warm sunlight.[^1]\n\n[^1]: A note.\n',
          targetText: 'The cat sleeps in warm sunlight.[^1]\n\n[^1]: A note.\n',
        },),).not.toThrow();
      },
    },),
    it({
      name: 'REPORTS A DEFINITION NOTHING REFERS TO that the archive did not carry, as a refusal the publish step '
        + 'turns into a reported defect, naming the entry and the finding (XingZ6023: `orphan-definition gfm 3`)',
      fn: async () => {
        const refusal = caught(function act(): unknown {
          return assertPageFootnotesIntact({
            entryId: 'BookshopCat',
            pageText: ORPHANED,
            targetText: ARCHIVE,
          },);
        },);
        if (!(refusal instanceof PageFootnoteIntegrityError))
          throw new Error('expected the orphan definition to be refused as a footnote integrity defect',);
        expect(refusal.findings,).toEqual(['page-footnote-orphan-definition gfm 3',],);
        expect(refusal.message,).toContain('entry BookshopCat',);
        expect(refusal.message,).toContain('orphan-definition gfm 3',);
        expect(refusal.messageNamesOnly,).toBe(true,);
      },
    },),
    it({
      name: 'REPORTS A REFERENCE WITH NO DEFINITION the archive never carried (the nineteenth hakureico pass of '
        + '2026-09-09 shipped “Mayday”[^1] with no [^1] definition anywhere)',
      fn: async () => {
        const refusal = caught(function act(): unknown {
          return assertPageFootnotesIntact({
            entryId: 'BookshopCat',
            pageText: 'The cat sleeps in warm sunlight[^1].\n\nThe cat naps on the sill.\n',
            targetText: ARCHIVE,
          },);
        },);
        if (!(refusal instanceof PageFootnoteIntegrityError))
          throw new Error('expected the unresolved reference to be refused as a footnote integrity defect',);
        expect(refusal.findings,).toEqual(['page-footnote-unresolved-reference gfm 1',],);
      },
    },),
    it({
      name: 'ACCEPTS a page that carries the same dangling reference the archive already carried, since a defect the '
        + 'archive brought is never blamed on the page',
      fn: async () => {
        expect(() => assertPageFootnotesIntact({
          entryId: 'BookshopCat',
          pageText: 'The cat naps in warm sunlight[^1].\n',
          targetText: 'The cat sleeps in warm sunlight[^1].\n',
        },),).not.toThrow();
      },
    },),
    it({
      name: 'READS THE ORIGINAL AN UNFILLED ANCHOR SHIPS when it is part of the text it is handed, since the page '
        + 'that ships carries it and the reader meets the page',
      fn: async () => {
        const refusal = caught(function act(): unknown {
          return assertPageFootnotesIntact({
            entryId: 'BookshopCat',
            pageText: `${ARCHIVE}\n> This passage has not been translated yet. The original text follows.\n\n她说[^7]\n`,
            targetText: ARCHIVE,
          },);
        },);
        if (!(refusal instanceof PageFootnoteIntegrityError))
          throw new Error('expected the original\'s unresolved reference to be refused as a footnote integrity defect',);
        expect(refusal.findings,).toEqual(['page-footnote-unresolved-reference gfm 7',],);
      },
    },),
  ],
},);
