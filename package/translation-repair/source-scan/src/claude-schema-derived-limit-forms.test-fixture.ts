import {
  HOLDER,
  storingCase,
} from './claude-schema-derived-container-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Callbacks that reach a call by a way other than being written in it, which stay unread
// The parameters of a function are made from a call when the function is written in the call, as an argument or as the
// callee. A function that reaches the call another way is a flow between two functions that the reading does not
// follow, and each way is pinned here as a limit: held in an array and called with the parameter, declared apart and
// handed over by a member of an object, by a conditional, by a factory's result, by a spread argument, by the member of
// an object written in place, by an array argument of `Reflect.apply`, or by a class field. A callback handed to a
// function that was made from the parameter and stored is no limit: the stored function is a name made from the
// parameter, so the call binds the parameters of the callback.

/**
 The line that declares a callback apart.
 */
const PUT = '  const put = (value: any,): void => { holder.a = value; };';

/**
 Cases of the ways a callback reaches a call that the reading does not follow, and the way it does.
 */
export const DERIVED_LIMIT_CASES: readonly ShapeCase[] = [
  storingCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK HANDED TO A FUNCTION MADE FROM THE '
      + 'PARAMETER AND STORED STORES THE ELEMENT IN, since the stored function is a name made from the parameter',
    lines: [
      HOLDER,
      '  const each = args.forEach.bind(args,);',
      '  each((value: any,) => { holder.a = value; },);',
    ],
    argument: 'holder.a',
    isListed: true,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK HELD IN AN ARRAY AND CALLED '
      + 'WITH THE PARAMETER STORES THE ARGUMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      '  const callbacks = [(value: any,) => { holder.a = value; },];',
      '  callbacks[0](args,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK DECLARED APART AND HANDED OVER '
      + 'BY A MEMBER OF AN OBJECT STORES THE ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      '  const kit = { put: (value: any,): void => { holder.a = value; }, };',
      '  args.forEach(kit.put,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK DECLARED APART AND HANDED OVER '
      + 'BY A CONDITIONAL STORES THE ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      PUT,
      '  args.forEach(args.length > 0 ? put : put,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK THAT A FACTORY RETURNS STORES '
      + 'THE ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      '  const make = () => (value: any,): void => { holder.a = value; };',
      '  args.forEach(make(),);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK HANDED OVER BY A SPREAD '
      + 'ARGUMENT STORES THE ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      PUT,
      '  args.forEach(...[put,],);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A METHOD OF AN OBJECT WRITTEN IN PLACE '
      + 'AND HANDED OVER BY ITS call MEMBER STORES THE ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      '  args.forEach({ call(value: any,) { holder.a = value; }, }.call,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK IN THE ARGUMENT ARRAY OF '
      + 'Reflect.apply STORES THE ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      '  Reflect.apply(Array.prototype.forEach, args, [(value: any,) => { holder.a = value; },],);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
  storingCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM AN OBJECT A CALLBACK THAT IS A CLASS FIELD STORES '
      + 'THE ELEMENT IN, a limit of the reading',
    lines: [
      HOLDER,
      '  class Kit { put = (value: any,): void => { holder.a = value; }; }',
      '  args.forEach(new Kit().put,);',
    ],
    argument: 'holder.a',
    isListed: false,
  },),
];

//endregion Callbacks that reach a call by a way other than being written in it, which stay unread
