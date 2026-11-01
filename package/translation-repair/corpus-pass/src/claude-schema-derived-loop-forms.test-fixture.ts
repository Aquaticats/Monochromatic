import {
  DERIVED_ARGUMENTS,
  derivedCase,
} from './claude-schema-derived-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names a loop over a parameter binds, read from the loop and from no other name
// The names made from a function's parameters (`claude-schema-derived-forms.test-fixture.ts`) take in the names the
// head of a `for...in` or a `for...of` loop binds from a value that reads one of them. The cases there read a member of
// the parameter itself in the body, so the name of the head was already one made from the parameter, and a search that
// ignored the loop would have passed them. Each case here builds from the name of the head alone, with the loop over
// the parameter as the only route: a loop over keys, a head that assigns a name declared before the loop, a head that
// assigns a name with `in`, and a head that takes a name out of an array pattern.

/**
 The line that hands the sender a builder call made from the given name, inside a loop.

 @param name - name the builder call reads

 @returns Line of a function body

 @example
 ```ts
 const line = sendsInLoop({ name: 'k', },);
 ```
 */
function sendsInLoop({ name, }: { readonly name: string; },): string {
  return `    return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(${name},), },);`;
}

/**
 Cases of a loop over a parameter whose head is the only route from the parameter to the builder call.
 */
export const DERIVED_LOOP_CASES: readonly ShapeCase[] = [
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE NAME A FOR-IN LOOP OVER ITS PARAMETER BINDS, and from '
      + 'no other name of the parameter',
    signature: 'mk(args: any,)',
    body: [
      '  for (const k in args) {',
      sendsInLoop({ name: 'k', },),
      '  }',
    ],
    call: `mk({ a: ${DERIVED_ARGUMENTS}, },)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME THAT A FOR-OF HEAD ASSIGNS WITHOUT DECLARING IT, a '
      + 'name declared before the loop',
    signature: 'mk(args: any[],)',
    body: [
      '  let k: any;',
      '  for (k of args) {',
      sendsInLoop({ name: 'k', },),
      '  }',
    ],
    call: `mk([${DERIVED_ARGUMENTS},],)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME THAT A FOR-IN HEAD ASSIGNS WITHOUT DECLARING IT, a '
      + 'name declared before the loop',
    signature: 'mk(args: any,)',
    body: [
      '  let k: any;',
      '  for (k in args) {',
      sendsInLoop({ name: 'k', },),
      '  }',
    ],
    call: `mk({ a: ${DERIVED_ARGUMENTS}, },)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME AN ARRAY PATTERN IN A FOR-OF HEAD ASSIGNS, a name '
      + 'declared before the loop',
    signature: 'mk(args: any[][],)',
    body: [
      '  let k: any;',
      '  for ([k] of args) {',
      sendsInLoop({ name: 'k', },),
      '  }',
    ],
    call: `mk([[${DERIVED_ARGUMENTS},],],)`,
    isListed: true,
  },),
];

//endregion Names a loop over a parameter binds, read from the loop and from no other name
