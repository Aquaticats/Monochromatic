/**
 Tests for the page-level footnote check: every reference on the page has a definition and every definition has a
 reference, whatever the archive carried.

 The check is not relative to the archive (decision D7 of the XingZ6020 round): `assertPageFootnotesIntact` weighs the
 page against the archive and so says nothing when the archive already lacked the definitions, which is how XingZ6020
 shipped five references with nowhere to land. This check reads the page alone, reports and never refuses.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  assertPageFootnotesResolve,
  PageFootnoteResolutionError,
} from '../../dist/final/node/index.mjs';

/**
 Runs the check and returns what it refused with.

 @param pageText - page to check

 @returns Refusal, or none when the page resolves

 @example
 ```ts
 const refusal = refusalOf({ pageText: 'A cat[^1].\n\n[^1]: Purrs.\n', },);
 ```
 */
function refusalOf(
  { pageText, }: { readonly pageText: string; },
): PageFootnoteResolutionError | 'resolves' {
  try {
    assertPageFootnotesResolve({
      entryId: 'BookshopCat',
      pageText,
    },);
  }
  catch (error) {
    if (error instanceof PageFootnoteResolutionError)
      return error;
    throw error;
  }
  return 'resolves';
}

await describe({
  name: assertPageFootnotesResolve.name,
  children: [
    it({
      name: 'ACCEPTS A PAGE WHOSE REFERENCES AND DEFINITIONS MATCH, the control every refusal departs from',
      fn: async () => {
        expect(refusalOf({ pageText: 'A cat[^1] and a mat[^note].\n\n[^1]: Purrs.\n\n[^note]: Sits.\n', },),)
          .toBe('resolves',);
      },
    },),

    it({
      name: 'NAMES EVERY REFERENCE THAT HAS NO DEFINITION, the shape XingZ6020 shipped: five references and '
        + 'the page ends before any definition',
      fn: async () => {
        const refusal = refusalOf({
          pageText: 'A tabby[^1] kept the shop[^2] company[^4], napped[^5] and purred[^6].\n',
        },);
        expect(refusal,).toBeInstanceOf(PageFootnoteResolutionError,);
        expect((refusal as PageFootnoteResolutionError).unresolved,).toEqual(['1', '2', '4', '5', '6',],);
        expect((refusal as PageFootnoteResolutionError).orphaned,).toEqual([],);
        expect((refusal as PageFootnoteResolutionError).duplicated,).toEqual([],);
        expect((refusal as PageFootnoteResolutionError).message,).toBe(
          'entry BookshopCat: page footnotes do not resolve; references without a definition: 1, 2, 4, 5, 6',
        );
      },
    },),

    it({
      name: 'NAMES EVERY DEFINITION THAT NO REFERENCE ANSWERS, and every identifier defined twice',
      fn: async () => {
        const refusal = refusalOf({
          pageText: 'A cat[^1].\n\n[^1]: Purrs.\n\n[^1]: Purrs again.\n\n[^3]: Sits.\n',
        },);
        expect(refusal,).toBeInstanceOf(PageFootnoteResolutionError,);
        expect((refusal as PageFootnoteResolutionError).unresolved,).toEqual([],);
        expect((refusal as PageFootnoteResolutionError).orphaned,).toEqual(['3',],);
        expect((refusal as PageFootnoteResolutionError).duplicated,).toEqual(['1',],);
        expect((refusal as PageFootnoteResolutionError).message,).toBe(
          'entry BookshopCat: page footnotes do not resolve; definitions without a reference: 3; '
            + 'identifiers defined more than once: 1',
        );
      },
    },),

    it({
      name: 'DECLARES ITS MESSAGE SAFE TO REPEAT, since a reported defect carries it into the run log',
      fn: async () => {
        const refusal = refusalOf({ pageText: 'A cat[^1].\n', },);
        expect((refusal as PageFootnoteResolutionError).messageNamesOnly,).toBe(true,);
      },
    },),
  ],
},);
