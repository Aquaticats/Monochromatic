import { expect, } from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

//region Rendering glossary candidate kinds
// WHETHER A LIST OF CANDIDATES VALIDATES AS INVALID OR AS VALID, in list
// order, against a rendering or community glossary guard's own candidates.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. community-glossary-fandom and every
// rendering-glossary-* class test (circumstance, idiom, owner-forms,
// phrasing, school-life, study, wording) kept their own copy of these two
// assertions; all now import them from here.

/**
 Kind each candidate validates as, in list order.

 @param candidates - candidates to validate

 @returns Each candidate's validation kind

 @example
 ```ts
 const kinds = kindsOf({ candidates: REFUSED_CANDIDATES, },);
 ```
 */
function kindsOf(
  { candidates, }: { readonly candidates: readonly Parameters<typeof validateTranslatedSlice>[0][]; },
): readonly string[] {
  return candidates.map(function kindOf(candidate,): string {
    return validateTranslatedSlice(candidate,)
      .kind;
  },);
}

/**
 Asserts every candidate validates as invalid, in list order.

 @param candidates - candidates expected to validate as invalid

 @example
 ```ts
 expectAllRefused({ candidates: REFUSED_CANDIDATES, },);
 ```
 */
export function expectAllRefused(
  { candidates, }: { readonly candidates: readonly Parameters<typeof validateTranslatedSlice>[0][]; },
): void {
  expect(kindsOf({ candidates, },),)
    .toEqual(candidates.map(function invalid(): string {
      return 'invalid';
    },),);
}

/**
 Asserts every candidate validates as valid, in list order.

 @param candidates - candidates expected to validate as valid

 @example
 ```ts
 expectAllAccepted({ candidates: ACCEPTED_CANDIDATES, },);
 ```
 */
export function expectAllAccepted(
  { candidates, }: { readonly candidates: readonly Parameters<typeof validateTranslatedSlice>[0][]; },
): void {
  expect(kindsOf({ candidates, },),)
    .toEqual(candidates.map(function valid(): string {
      return 'valid';
    },),);
}

//endregion Rendering glossary candidate kinds
