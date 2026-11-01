import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import { DERIVED_ARGUMENTS, } from './claude-schema-derived-forms.test-fixture.ts';
import {
  HOLDER,
  storingCase,
} from './claude-schema-derived-container-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names made from a function's parameters through a member as the target of a pattern or of a call
// A member as the target of a pattern (an element of an array pattern, a rest element, a member with a default in
// an assignment pattern) stores the value in the object the member is read off, and so does a storing method
// written under a type mark (`(holder as any).push(args)`, `holder.push!(args)`) and `Object.assign` or a method that
// defines a property when `Object` is written under a type mark or read off the global object. The first name of the
// object is then a name made from the parameter, as it is for the other places that store
// (`claude-schema-derived-container-forms.test-fixture.ts`). A value that one function stores in an object of the
// module and another function builds from is a flow between two functions, which is not read.

/**
 Cases of a member as the target of a pattern, and of a storing method under a type mark.
 */
export const DERIVED_TARGET_CASES: readonly ShapeCase[] = [
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A MEMBER IN AN ARRAY PATTERN STORES ITS '
      + 'PARAMETER IN',
    lines: [
      HOLDER,
      '  [holder.x] = [args,];',
    ],
    argument: 'holder.x',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT THE REST ELEMENT OF A PATTERN STORES ITS '
      + 'PARAMETER IN',
    lines: [
      HOLDER,
      '  [...holder.rest] = [args,];',
    ],
    argument: 'holder.rest',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A MEMBER WITH A DEFAULT IN A PATTERN STORES ITS '
      + 'PARAMETER IN, the default being no value made from the parameter',
    lines: [
      HOLDER,
      '  ({ b: holder.x = 1, } = { b: args, });',
    ],
    argument: 'holder.x',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT ITS PARAMETER WAS STORED IN BY A METHOD CALLED '
      + 'UNDER A TYPE ASSERTION',
    lines: [
      '  const holder: any[] = [];',
      '  (holder as any).push(args,);',
    ],
    argument: 'holder[0]',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT ITS PARAMETER WAS STORED IN BY A METHOD UNDER A '
      + 'NON-NULL ASSERTION',
    lines: [
      '  const holder: any[] = [];',
      '  holder.push!(args,);',
    ],
    argument: 'holder[0]',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` COPIES A FIELD MADE FROM ITS '
      + 'PARAMETER INTO, with `Object` under a type assertion',
    lines: [
      HOLDER,
      '  (Object as any).assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` COPIES A FIELD MADE FROM ITS '
      + 'PARAMETER INTO, with `Object` read off the global object',
    lines: [
      HOLDER,
      '  globalThis.Object.assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.defineProperty` GIVES A VALUE MADE FROM '
      + 'ITS PARAMETER, with `Object` under a non-null assertion',
    lines: [
      HOLDER,
      '  Object!.defineProperty(holder, \'a\', { value: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `assign` OF ANOTHER OBJECT READ OFF THE '
      + 'GLOBAL OBJECT COPIES ITS PARAMETER INTO, only `Object` has the storing statics',
    lines: [
      HOLDER,
      '  (globalThis as any).Other.assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `assign` OF A MEMBER NAMED Object OF '
      + 'ANOTHER OBJECT COPIES ITS PARAMETER INTO, only `Object` itself has the storing statics',
    lines: [
      HOLDER,
      '  const other: any = {};',
      '  other.Object.assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  {
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT STORES ITS PARAMETER IN AN OBJECT OF THE MODULE THAT ANOTHER '
      + 'FUNCTION BUILDS FROM, a limit of the reading',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/derived-heap-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'const box: any = {};',
          'export function put(args: any,) {',
          '  box.a = args;',
          '}',
          'export async function mk() {',
          '  return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(box.a,), },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/derived-heap-caller.ts',
        text: textOf([
          'import { put, } from \'./derived-heap-stage.ts\';',
          `export const SENT = put(${DERIVED_ARGUMENTS},);`,
        ],),
      },
    ],
    found: ['imaginary/src/derived-heap-stage.ts',],
    builders: [],
    spots: [],
  },
];

//endregion Names made from a function's parameters through a member as the target of a pattern or of a call
