import {
  HOLDER,
  storingCase,
} from './claude-schema-derived-container-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names made from a function's parameters through a storing method of a built-in object, however it is called
// `Object.assign(holder, { a: args })` stores in its first argument, and so do `Object.defineProperty`,
// `Object.defineProperties` and `Reflect.set`. The call is the same when the method is called through `call` or
// `apply`, through `Reflect.apply` with the arguments in an array, with the arguments spread from an array written in
// place, as the last operand of a comma, or through a chain of global objects (`globalThis.globalThis.Object`,
// `window.self.Object`). What stays unread, and is pinned silent here: a method bound and called after, the arguments
// of a spread that is no array written in place, and the arguments of `Reflect.apply` that are no array written in
// place.

/**
 Cases of a value stored through a storing method of a built-in object called another way.
 */
export const DERIVED_STATIC_CASES: readonly ShapeCase[] = [
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS CALLED THROUGH `call` WITH A '
      + 'FIELD MADE FROM ITS PARAMETER',
    lines: [
      HOLDER,
      '  Object.assign.call(null, holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS CALLED THROUGH `apply` WITH A '
      + 'FIELD MADE FROM ITS PARAMETER',
    lines: [
      HOLDER,
      '  Object.assign.apply(null, [holder, { a: args, },],);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS CALLED THROUGH `Reflect.apply` '
      + 'WITH A FIELD MADE FROM ITS PARAMETER',
    lines: [
      HOLDER,
      '  Reflect.apply(Object.assign, null, [holder, { a: args, },],);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS CALLED WITH ITS ARGUMENTS '
      + 'SPREAD FROM AN ARRAY WRITTEN IN PLACE',
    lines: [
      HOLDER,
      '  Object.assign(...([holder, { a: args, },] as [any, any,]),);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS THE LAST OPERAND OF A COMMA '
      + 'THE CALL IS MADE ON',
    lines: [
      HOLDER,
      '  (0, Object.assign)(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS READ OFF A CHAIN OF GLOBAL '
      + 'OBJECTS',
    lines: [
      HOLDER,
      '  (globalThis as any).globalThis.Object.assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS READ OFF `window.self`',
    lines: [
      HOLDER,
      '  (globalThis as any).window.self.Object.assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS WRITTEN WITH A STRING KEY '
      + 'UNDER A TYPE MARK AND CALLED THROUGH `call`',
    lines: [
      HOLDER,
      '  (Object as any)[\'assign\'].call(null, holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.defineProperty` IS CALLED THROUGH `call`',
    lines: [
      HOLDER,
      '  Object.defineProperty.call(null, holder, \'a\', { value: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Reflect.set` GIVES A VALUE MADE FROM ITS '
      + 'PARAMETER',
    lines: [
      HOLDER,
      '  Reflect.set(holder, \'a\', args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A METHOD NAMED `assign` OF ANOTHER OBJECT '
      + 'IS CALLED THROUGH `call` WITH A FIELD MADE FROM ITS PARAMETER',
    lines: [
      HOLDER,
      '  const other = { assign(): void {}, };',
      '  other.assign.call(null, holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS BOUND AND CALLED '
      + 'AFTER, a limit of the reading',
    lines: [
      HOLDER,
      '  Object.assign.bind(null,)(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` IS CALLED WITH ARGUMENTS '
      + 'SPREAD FROM A NAME, a limit of the reading',
    lines: [
      HOLDER,
      '  const pair = [holder, { a: args, },] as [any, any,];',
      '  Object.assign(...pair,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Reflect.apply` HANDS `Object.assign` ITS '
      + 'ARGUMENTS IN A NAME, a limit of the reading',
    lines: [
      HOLDER,
      '  const pair = [holder, { a: args, },];',
      '  Reflect.apply(Object.assign, null, pair,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
];

//endregion Names made from a function's parameters through a storing method of a built-in object, however it is called
