import {
  DERIVED_ARGUMENTS,
  derivedCase,
  sendsFrom,
} from './claude-schema-derived-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names made from a function's parameters through objects, class fields and pattern defaults
// A function that stores a value made from a parameter in an object it holds, and builds from that object, takes the
// arguments of the schema from its callers all the same. The first name of the object is then a name made from the
// parameter: the object an assignment stores a value in (a field, an index, a field of a field), the list or the map
// that a method which stores its argument is called on (`push`, `set`), the first argument of `Object.assign` and of
// the methods that define a property, the name of a class or of a function declared inside whose body reads the
// parameter, and the names a pattern default binds. What stays unread, and is pinned silent here: a value handed to a
// function or to a method that is not one of those, which may store it in an object it receives, a value stored
// through another name for the object (`const alias = holder; alias.a = args`), a value stored in a field of
// `this`, and a value that one function stores in an object of the module and another function builds from.

/**
 The line that declares a local object a case stores a value in.
 */
export const HOLDER = '  const holder: any = {};';

/**
 A case where the parameter is stored in a local object by the lines given, and the builder call reads the object.

 @param name - name of the case

 @param lines - lines that declare the object and store the parameter in it

 @param argument - what the builder call reads

 @param isListed - whether the caller of the function is listed

 @returns Case

 @example
 ```ts
 const shapeCase = storingCase({ name: 'x', lines: [HOLDER,], argument: 'holder.a', isListed: true, },);
 ```
 */
export function storingCase(
  {
    name,
    lines,
    argument,
    isListed,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly argument: string;
    readonly isListed: boolean;
  },
): ShapeCase {
  return derivedCase({
    name,
    signature: 'mk(args: any,)',
    body: [
      ...lines,
      sendsFrom({ argument, },),
    ],
    call: `mk(${DERIVED_ARGUMENTS},)`,
    isListed,
  },);
}

/**
 A case where a method stores the parameter in the list, the set or the map it is called on.

 @param method - method called on the object

 @param declaration - line that declares the object

 @param call - call that stores the parameter

 @param argument - what the builder call reads

 @returns Case that lists the caller

 @example
 ```ts
 const shapeCase = methodCase({
   method: 'push',
   declaration: HOLDER,
   call: '  holder.push(args,);',
   argument: 'holder',
 },);
 ```
 */
function methodCase(
  {
    method,
    declaration,
    call,
    argument,
  }: {
    readonly method: string;
    readonly declaration: string;
    readonly call: string;
    readonly argument: string;
  },
): ShapeCase {
  return storingCase({
    name: `LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT ITS PARAMETER WAS STORED IN BY \`${method}\``,
    lines: [
      declaration,
      call,
    ],
    argument,
    isListed: true,
  },);
}

/**
 Cases of names made from a parameter through an object, a list, a map, a class field or a pattern default.
 */
export const DERIVED_CONTAINER_CASES: readonly ShapeCase[] = [
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A FIELD OF A LOCAL OBJECT THAT ITS PARAMETER WAS ASSIGNED '
      + 'TO',
    lines: [
      HOLDER,
      '  holder.a = args;',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A FIELD OF A FIELD THAT ITS PARAMETER WAS ASSIGNED TO, '
      + 'under a type assertion',
    lines: [
      HOLDER,
      '  (holder as any).a.b = args;',
    ],
    argument: 'holder.a.b',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN ELEMENT OF A LIST THAT ITS PARAMETER WAS ASSIGNED TO',
    lines: [
      '  const holder: any[] = [];',
      '  holder[0] = args;',
    ],
    argument: 'holder[0]',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A PATTERN ASSIGNMENT STORES ITS PARAMETER IN, a '
      + 'member as the target of a pattern',
    lines: [
      HOLDER,
      '  ({ a: holder.x } = { a: args, });',
    ],
    argument: 'holder.x',
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FOR-OF HEAD STORES THE ELEMENTS OF ITS '
      + 'PARAMETER IN, a member as the target of the head',
    signature: 'mk(args: any[],)',
    body: [
      HOLDER,
      '  for (holder.k of args) {',
      '    return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(holder.k,), },);',
      '  }',
    ],
    call: `mk([${DERIVED_ARGUMENTS},],)`,
    isListed: true,
  },),
  methodCase({
    method: 'push',
    declaration: '  const holder: any[] = [];',
    call: '  holder.push(args,);',
    argument: 'holder[0]',
  },),
  methodCase({
    method: 'unshift',
    declaration: '  const holder: any[] = [];',
    call: '  holder.unshift(args,);',
    argument: 'holder[0]',
  },),
  methodCase({
    method: 'splice',
    declaration: '  const holder: any[] = [];',
    call: '  holder.splice(0, 0, args,);',
    argument: 'holder[0]',
  },),
  methodCase({
    method: 'fill',
    declaration: '  const holder: any[] = [0,];',
    call: '  holder.fill(args,);',
    argument: 'holder[0]',
  },),
  methodCase({
    method: 'set',
    declaration: '  const holder = new Map<string, any>();',
    call: '  holder.set(\'a\', args,);',
    argument: 'holder.get(\'a\',)',
  },),
  methodCase({
    method: 'add',
    declaration: '  const holder = new Set<any>();',
    call: '  holder.add(args,);',
    argument: '[...holder,][0]',
  },),
  methodCase({
    method: 'append',
    declaration: '  const holder = new Headers();',
    call: '  holder.append(\'a\', args,);',
    argument: 'holder.get(\'a\',)',
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.assign` COPIES A FIELD MADE FROM ITS '
      + 'PARAMETER INTO',
    lines: [
      HOLDER,
      '  Object.assign(holder, { a: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.defineProperty` GIVES A VALUE MADE FROM '
      + 'ITS PARAMETER',
    lines: [
      HOLDER,
      '  Object.defineProperty(holder, \'a\', { value: args, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT `Object.defineProperties` GIVES A VALUE MADE FROM '
      + 'ITS PARAMETER',
    lines: [
      HOLDER,
      '  Object.defineProperties(holder, { a: { value: args, }, },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A CLASS WHOSE FIELD IS MADE FROM ITS PARAMETER',
    lines: [
      '  class Box {',
      '    a: any = args;',
      '  }',
    ],
    argument: 'new Box().a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE CALL OF A FUNCTION DECLARED INSIDE IT THAT RETURNS ITS '
      + 'PARAMETER',
    lines: [
      '  function inner() {',
      '    return args;',
      '  }',
    ],
    argument: 'inner()',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE CALL OF AN ASYNC FUNCTION DECLARED INSIDE IT THAT '
      + 'RETURNS ITS PARAMETER',
    lines: [
      '  async function inner() {',
      '    return args;',
      '  }',
    ],
    argument: 'await inner()',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE VALUE A GENERATOR DECLARED INSIDE IT YIELDS FROM ITS '
      + 'PARAMETER',
    lines: [
      '  function* inner() {',
      '    yield args;',
      '  }',
    ],
    argument: 'inner().next().value',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME THE DEFAULT OF AN OBJECT PATTERN GIVES ITS PARAMETER',
    lines: ['  const { a = args, } = {} as any;',],
    argument: 'a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME THE DEFAULT OF AN ARRAY PATTERN GIVES ITS PARAMETER',
    lines: ['  const [a = args,] = [] as any[];',],
    argument: 'a',
    isListed: true,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A LOCAL FUNCTION STORES ITS PARAMETER '
      + 'IN, a limit of the reading',
    lines: [
      HOLDER,
      '  const store = (target: any, value: any,): void => { target.a = value; };',
      '  store(holder, args,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A METHOD OUTSIDE THE LIST STORES ITS '
      + 'PARAMETER IN, a limit of the reading',
    lines: [
      '  const holder = { put(value: any,): void { holder.a = value; }, a: undefined as any, };',
      '  holder.put(args,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT ITS PARAMETER WAS STORED IN THROUGH '
      + 'ANOTHER NAME FOR THE OBJECT, a limit of the reading',
    lines: [
      HOLDER,
      '  const alias = holder;',
      '  alias.a = args;',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT NO NAME OF ITS PARAMETERS WAS STORED IN',
    lines: [
      HOLDER,
      '  holder.a = 1;',
      '  void args;',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
];

//endregion Names made from a function's parameters through objects, class fields and pattern defaults
