import {
  HOLDER,
  storingCase,
} from './claude-schema-derived-container-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names made from a parameter through the parts of a declaration outside its body
// A class or a function declared inside a function holds what its whole declaration reads, not only what its body
// reads: the superclass of a class, the default of a parameter and the default inside a destructured parameter take
// the value of the parameter all the same. A call that stores in its first argument is `Object.assign` or a method of
// `Object` that defines a property, and no method of another object that has the same name; a call of the other
// functions that define a property, like `Reflect.defineProperty`, is outside the list and is pinned silent, and so is
// a storing function that a constant holds under another name, or that a pattern takes out of `Object`.

/**
 Cases of the parts of a declaration outside its body, and of the calls that are not in the list of storing calls.
 */
export const DERIVED_DECLARATION_CASES: readonly ShapeCase[] = [
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN INSTANCE OF A CLASS THAT EXTENDS ITS PARAMETER, the '
      + 'superclass of a class counts, and the body of the class reads nothing',
    lines: ['  class Box extends (args as any) {}',],
    argument: 'new Box()',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE CALL OF A FUNCTION DECLARED INSIDE IT WHOSE PARAMETER '
      + 'TAKES ITS PARAMETER AS A DEFAULT, the default of a parameter counts, and the body reads nothing',
    lines: [
      '  function inner(value: any = args,) {',
      '    return 1;',
      '  }',
    ],
    argument: 'inner()',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE CALL OF A FUNCTION DECLARED INSIDE IT WHOSE DESTRUCTURED '
      + 'PARAMETER TAKES ITS PARAMETER AS A DEFAULT, and the body reads nothing',
    lines: [
      '  function inner({ a = args, }: any = {},) {',
      '    return 1;',
      '  }',
    ],
    argument: 'inner()',
    isListed: true,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A METHOD OF ANOTHER OBJECT NAMED assign '
      + 'COPIES ITS PARAMETER INTO, only `Object` has the storing statics',
    lines: [
      HOLDER,
      '  const other: any = { assign(target: any, source: any,): void { target.a = source.a; }, };',
      '  other.assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Reflect.defineProperty` GIVES A VALUE MADE '
      + 'FROM ITS PARAMETER, a call outside the list and a limit of the reading',
    lines: [
      HOLDER,
      '  Reflect.defineProperty(holder, \'a\', { value: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CONSTANT THAT HOLDS Object.assign COPIES '
      + 'ITS PARAMETER INTO, another name for a storing function and a limit of the reading',
    lines: [
      HOLDER,
      '  const copy = Object.assign;',
      '  copy(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT assign TAKEN OUT OF Object BY A PATTERN '
      + 'COPIES ITS PARAMETER INTO, another name for a storing function and a limit of the reading',
    lines: [
      HOLDER,
      '  const { assign, } = Object;',
      '  assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
];

//endregion Names made from a parameter through the parts of a declaration outside its body
