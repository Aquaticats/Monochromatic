import {
  HOLDER,
  storingCase,
} from './claude-schema-derived-container-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names made from a function's parameters through the parameters of a callback
// A function written as an argument of a call takes its parameters from the call: a callback of `forEach`, `map` or
// `then` gets the elements of the value the method is called on. When the callee or another argument of the call
// reads a name made from a parameter of the enclosing function, the parameters of the callback are made from it, and a
// value stored through them in an object the builder call reads is stored from the parameter all the same. What stays
// unread, and is pinned silent here: a callback declared apart and handed over by its name, which is a flow between
// two functions, and a call that reads nothing made from a parameter.

/**
 Cases of a value stored through the parameters of a callback.
 */
export const DERIVED_CALLBACK_CASES: readonly ShapeCase[] = [
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT AN ARROW CALLBACK OF forEach ON ITS PARAMETER '
      + 'STORES THE ELEMENT IN',
    lines: [
      HOLDER,
      '  args.forEach((value: any,) => { holder.a = value; },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A FUNCTION EXPRESSION CALLBACK OF forEach ON ITS '
      + 'PARAMETER STORES THE ELEMENT IN',
    lines: [
      HOLDER,
      '  args.forEach(function (value: any,) { holder.a = value; },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK OF map ON ITS PARAMETER STORES THE '
      + 'ELEMENT IN, as the value of an assignment',
    lines: [
      HOLDER,
      '  args.map((value: any,) => (holder.a = value),);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A LIST A CALLBACK OF forEach ON ITS PARAMETER PUSHES THE '
      + 'ELEMENT TO',
    lines: [
      '  const holder: any[] = [];',
      '  args.forEach((value: any,) => { holder.push(value,); },);',
    ],
    argument: 'holder[0]',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK OF then ON A PROMISE OF ITS PARAMETER '
      + 'STORES THE VALUE IN',
    lines: [
      HOLDER,
      '  await Promise.resolve(args,).then((value: any,) => { holder.a = value; },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK OF forEach WITH A DESTRUCTURED '
      + 'PARAMETER STORES A FIELD OF THE ELEMENT IN',
    lines: [
      HOLDER,
      '  args.forEach(({ a, }: any,) => { holder.a = a; },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A LIST A CALLBACK ON THE ENTRIES OF ITS PARAMETER PUSHES A '
      + 'VALUE TO',
    lines: [
      '  const holder: any[] = [];',
      '  Object.entries(args,).forEach(([, value,]: any,) => { holder.push(value,); },);',
    ],
    argument: 'holder[0]',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A LIST A CALLBACK ON A COPY OF ITS PARAMETER PUSHES THE '
      + 'ELEMENT TO',
    lines: [
      '  const copy = [...args,];',
      '  const holder: any[] = [];',
      '  copy.forEach((value: any,) => { holder.push(value,); },);',
    ],
    argument: 'holder[0]',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK IN A CALLBACK STORES THE VALUE IN',
    lines: [
      HOLDER,
      '  args.forEach((row: any,) => { row.forEach((value: any,) => { holder.a = value; },); },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK STORES ITS PARAMETER IN, when only an '
      + 'argument of the call after the callback reads the parameter of the function',
    lines: [
      HOLDER,
      '  [1,].forEach((value: any,) => { holder.a = value; }, args,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK OF reduce STORES ITS SECOND PARAMETER '
      + 'IN, a later parameter of the callback counts',
    lines: [
      HOLDER,
      '  args.reduce((total: any, value: any,) => { holder.a = value; return total; }, 0,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK UNDER A TYPE ASSERTION STORES THE '
      + 'ELEMENT IN',
    lines: [
      HOLDER,
      '  args.forEach(((value: any,) => { holder.a = value; }) as any,);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK OF A CONSTRUCTOR CALL STORES ITS '
      + 'ARGUMENT IN, the call of new counts as a call',
    lines: [
      HOLDER,
      '  const Runner: any = class { constructor(input: any, run: any,) { run(input,); } };',
      '  new Runner(args, (value: any,) => { holder.a = value; },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK STORES THE ELEMENT OF ANOTHER '
      + 'VALUE IN, a call that reads nothing made from a parameter',
    lines: [
      HOLDER,
      '  [1,].forEach((value: any,) => { holder.a = value; },);',
      '  void args;',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK STORES ITS OWN PARAMETER IN, '
      + 'when the call reads nothing made from a parameter and only the body of the callback does',
    lines: [
      HOLDER,
      '  [1,].forEach((value: any,) => { void args; holder.a = value; },);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK DECLARED APART STORES THE '
      + 'ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      '  const put = (value: any,): void => { holder.a = value; };',
      '  args.forEach(put,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
];

//endregion Names made from a function's parameters through the parameters of a callback
