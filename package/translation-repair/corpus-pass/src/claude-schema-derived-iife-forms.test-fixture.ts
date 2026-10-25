import {
  HOLDER,
  storingCase,
} from './claude-schema-derived-container-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names made from a function's parameters through the parameters of a function called where it is written
// A function written as the callee of a call (`((value) => { holder.a = value; })(args)`) takes its parameters from the
// arguments of the call, as a function written as an argument takes them from the call it sits in. When an argument
// reads a name made from a parameter of the enclosing function, the parameters of the called function are made from it,
// and a value stored through them in an object the builder call reads is stored from the parameter all the same. The
// function may be an arrow, a function expression or an async function, written under parentheses, a type mark or a
// unary operator, and a function called inside a function called at once is read the same way. A call whose arguments
// read nothing made from a parameter binds nothing.

/**
 Cases of a value stored through the parameters of a function that is called where it is written.
 */
export const DERIVED_IIFE_CASES: readonly ShapeCase[] = [
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT AN ARROW CALLED AT ONCE WITH ITS PARAMETER STORES '
      + 'THE ARGUMENT IN',
    lines: [
      HOLDER,
      '  ((value: any,) => { holder.a = value; })(args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION EXPRESSION CALLED AT ONCE WITH ITS '
      + 'PARAMETER STORES THE ARGUMENT IN',
    lines: [
      HOLDER,
      '  (function (value: any,) { holder.a = value; })(args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT AN ASYNC ARROW CALLED AT ONCE AND AWAITED STORES '
      + 'THE ARGUMENT IN',
    lines: [
      HOLDER,
      '  await (async (value: any,) => { holder.a = value; })(args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION CALLED AT ONCE UNDER A TYPE ASSERTION '
      + 'STORES THE ARGUMENT IN',
    lines: [
      HOLDER,
      '  (((value: any,) => { holder.a = value; }) as any)(args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION EXPRESSION CALLED AT ONCE BEHIND A '
      + 'UNARY OPERATOR STORES THE ARGUMENT IN',
    lines: [
      HOLDER,
      '  void function (value: any,) { holder.a = value; }(args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION CALLED AT ONCE WITH A DESTRUCTURED '
      + 'PARAMETER STORES A FIELD OF THE ARGUMENT IN',
    lines: [
      HOLDER,
      '  (({ a, }: any,) => { holder.a = a; })(args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION CALLED AT ONCE STORES ITS SECOND '
      + 'PARAMETER IN, when only the second argument reads the parameter of the function',
    lines: [
      HOLDER,
      '  ((first: any, second: any,) => { holder.a = second; })(1, args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A LIST A FUNCTION CALLED AT ONCE PUSHES THE ARGUMENT TO',
    lines: [
      '  const holder: any[] = [];',
      '  ((value: any,) => { holder.push(value,); })(args,);',
    ],
    argument: 'holder[0]',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION CALLED AT ONCE IN A FUNCTION CALLED AT '
      + 'ONCE STORES THE ARGUMENT IN',
    lines: [
      HOLDER,
      '  ((outer: any,) => { ((inner: any,) => { holder.a = inner; })(outer,); })(args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION CALLED AT ONCE STORES ANOTHER '
      + 'VALUE IN, a call whose arguments read nothing made from a parameter',
    lines: [
      HOLDER,
      '  ((value: any,) => { holder.a = value; })(1,);',
      '  void args;',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION CALLED AT ONCE STORES ITS FIRST '
      + 'PARAMETER IN, when only the second argument reads the parameter of the function, since the link does not tell '
      + 'the arguments apart (an over-read that fails closed)',
    lines: [
      HOLDER,
      '  ((first: any, second: any,) => { void second; holder.a = first; })(1, args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
];

//endregion Names made from a function's parameters through the parameters of a function called where it is written
