import { textCase, } from './claude-schema-text-call-forms.test-fixture.ts';
import { TAKES_LOADER, } from './claude-schema-text-pin-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Reads of the loader and of the binding that call nothing and are reported all the same
// `process.getBuiltinModule` and `process.binding` are reported wherever they are read and are no direct call, because
// the function can be called with a name the search cannot read. The reading does not tell a read that calls nothing
// from one that does, so it reports an assignment to the member, a `delete` of it, its `name`, a read of `binding`
// through `call` with a name that makes no code, and a member named `getBuiltinModule` of any object named `process`
// (`ctx.process.getBuiltinModule`). Each is an over-report that fails closed, and each is pinned here as reported, so a
// change that stops reporting one is a decision and not an accident.

/**
 What a report says of `process.binding` read where it is no direct call.
 */
const TAKES_BINDING = 'binding of Node taken out of a call runs text the search cannot read';

/**
 Cases of the over-reports of the loader of modules and of the binding of Node.
 */
export const LOADER_OVERREAD_CASES: readonly ShapeCase[] = [
  textCase({
    name: 'REPORTS AN ASSIGNMENT TO process.getBuiltinModule, an over-report that fails closed',
    lines: ['(process as any).getBuiltinModule ??= () => undefined;',],
    expected: [
      {
        line: 1,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'REPORTS A delete OF process.getBuiltinModule, an over-report that fails closed',
    lines: ['delete (process as any).getBuiltinModule;',],
    expected: [
      {
        line: 1,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'REPORTS THE name OF process.getBuiltinModule, an over-report that fails closed',
    lines: ['export const named = process.getBuiltinModule.name;',],
    expected: [
      {
        line: 1,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'REPORTS process.binding READ BY call WITH A NAME THAT MAKES NO CODE, an over-report that fails closed',
    lines: ['export const read = process.binding.call(process, \'fs\',);',],
    expected: [
      {
        line: 1,
        position: TAKES_BINDING,
      },
    ],
  },),
  textCase({
    name: 'REPORTS A MEMBER NAMED getBuiltinModule OF ANY OBJECT NAMED process, an over-report that fails closed',
    lines: [
      'declare const ctx: { readonly process: { readonly getBuiltinModule: unknown; }; };',
      'export const held = [ctx.process.getBuiltinModule,];',
    ],
    expected: [
      {
        line: 2,
        position: TAKES_LOADER,
      },
    ],
  },),
];

//endregion Reads of the loader and of the binding that call nothing and are reported all the same
