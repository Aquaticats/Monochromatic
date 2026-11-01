import { textCase, } from './claude-schema-text-call-forms.test-fixture.ts';
import { TAKES_LOADER, } from './claude-schema-text-pin-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The loader of modules handed to a call as an argument
// `process.getBuiltinModule` is told as the callee of a call by the string it is called with. The same function handed
// to a call as an argument (`Reflect.apply(process.getBuiltinModule, process, ['vm'])`, a function of the caller's own)
// is called by that function with a name the search cannot read, so it is a read where it is no direct call and is
// reported where it sits. A call of the loader made with the loader as its callee is the direct call.

/**
 What a report says of a place that builds code from the `Function` constructor.
 */
const BUILDS_FUNCTION = 'Function builds code from text the search cannot read';

/**
 Cases of the loader of modules handed to a call.
 */
export const TEXT_ARGUMENT_CASES: readonly ShapeCase[] = [
  textCase({
    name: 'REPORTS process.getBuiltinModule HANDED TO Reflect.apply AS THE FUNCTION IT CALLS, since the call that '
      + 'holds it as an argument is not a direct call of it',
    lines: ['export const sent = Reflect.apply(process.getBuiltinModule, process, [\'vm\',],);',],
    expected: [
      {
        line: 1,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'REPORTS process.getBuiltinModule HANDED TO A FUNCTION OF THE FILE AS AN ARGUMENT',
    lines: [
      'declare function take(value: unknown,): void;',
      'take(process.getBuiltinModule,);',
    ],
    expected: [
      {
        line: 2,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'REPORTS process.getBuiltinModule HANDED TO call OF THE CALL MEMBER OF Function.prototype, the constructor '
      + 'and the loader both',
    lines: [
      'export const called = Function.prototype.call.call(process.getBuiltinModule, process, \'vm\',);',
    ],
    expected: [
      {
        line: 1,
        position: BUILDS_FUNCTION,
      },
      {
        line: 1,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'ACCEPTS process.getBuiltinModule CALLED DIRECTLY WITH A NAME OF A MODULE THAT MAKES NO CODE, while the same '
      + 'function handed to a call is reported',
    lines: [
      'export const direct = process.getBuiltinModule(\'node:fs\',);',
    ],
    expected: [],
  },),
];

//endregion The loader of modules handed to a call as an argument
