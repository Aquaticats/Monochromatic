import { methodFormatLinesOf, } from './claude-schema-builder-forms.test-fixture.ts';
import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Forms the search follows to a builder: a load inside a function, a load read in place, and a call method
// What `claude-schemas.unit.test.ts` feeds the builder search to show three readings that reach a builder through more
// than a name bound at the top of a file: a dynamic `import()` or a `require` of a literal path bound to a name inside
// a function body (the first binding of a name in the file wins); a member read off such a load in place, as in
// `(await import(path)).name` and `require(path).name`; and `.call`, `.apply` and `.bind` read off a builder, which
// are the builder. Each case is a few imaginary files, with the contest builder (`contestResponseFormat`) beside them,
// and the files and builders the search must list.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const CONTEST_ARGUMENTS = '{ schemaName: \'followed\', verdicts: [], asksArchive: false, }';

/**
 Cases of a load bound to a name inside a function body.
 */
export const BINDING_IN_FUNCTION_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A BUILDER BROUGHT IN INSIDE A FUNCTION BY A DYNAMIC import() OR A require OF A LITERAL PATH, whole '
      + 'or destructured with a new name',
    files: [
      {
        file: 'imaginary/src/nested-dynamic-destructured-stage.ts',
        text: textOf([
          'export async function lazyStage() {',
          `  const { contestResponseFormat: lazyFormat, } = await import('${CONTEST_SPECIFIER}');`,
          `  return send(lazyFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/nested-dynamic-whole-stage.ts',
        text: textOf([
          'export async function wholeStage() {',
          `  const contest = await import('${CONTEST_SPECIFIER}');`,
          `  return send(contest.contestResponseFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/nested-required-stage.ts',
        text: textOf([
          'export function requiredStage() {',
          `  const required = require('${CONTEST_SPECIFIER}');`,
          `  return send(required.contestResponseFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/nested-dynamic-destructured-stage.ts',
      'imaginary/src/nested-dynamic-whole-stage.ts',
      'imaginary/src/nested-required-stage.ts',
    ],
    builders: [],
  },
  {
    name: 'READS THE FIRST BINDING OF A NAME IN A FILE, whichever function binds it, so a name bound to a module of '
      + 'the sources that does not export the builder first stands for nothing',
    files: [
      {
        file: 'imaginary/src/other-kit.ts',
        text: textOf(['export const other = 1;',],),
      },
      {
        file: 'imaginary/src/two-bindings-stage.ts',
        text: textOf([
          'export async function firstStage() {',
          '  const kit = await import(\'./other-kit.ts\');',
          `  return send(kit.contestResponseFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
          '',
          'export async function secondStage() {',
          `  const kit = await import('${CONTEST_SPECIFIER}');`,
          `  return send(kit.contestResponseFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'READS THE BINDING OF THE TOP OF A FILE BEFORE A BINDING IN A FUNCTION, so a function that binds the name '
      + 'again does not change what the name stands for',
    files: [
      {
        file: 'imaginary/src/other-top-kit.ts',
        text: textOf(['export const other = 1;',],),
      },
      {
        file: 'imaginary/src/top-wins-stage.ts',
        text: textOf([
          'import * as kit from \'./other-top-kit.ts\';',
          '',
          'export async function laterStage() {',
          `  const kit = await import('${CONTEST_SPECIFIER}');`,
          `  return send(kit.contestResponseFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
];

/**
 Cases of a member read off a load in place.
 */
export const LOAD_IN_PLACE_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A MEMBER READ OFF A LOAD IN PLACE, as in `(await import(path)).name(...)` and `require(path).name(...)`',
    files: [
      {
        file: 'imaginary/src/in-place-dynamic-stage.ts',
        text: textOf([
          'export async function inPlaceDynamicStage() {',
          `  return send((await import('${CONTEST_SPECIFIER}')).contestResponseFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/in-place-required-stage.ts',
        text: textOf([
          'export function inPlaceRequiredStage() {',
          `  return send(require('${CONTEST_SPECIFIER}').contestResponseFormat(${CONTEST_ARGUMENTS},),);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/in-place-constant-stage.ts',
        text: textOf([
          `const make = (await import('${CONTEST_SPECIFIER}')).contestResponseFormat;`,
          '',
          `export const IN_PLACE_CONSTANT_RESPONSE_FORMAT = make(${CONTEST_ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/in-place-constant-stage.ts',
      'imaginary/src/in-place-dynamic-stage.ts',
      'imaginary/src/in-place-required-stage.ts',
    ],
    builders: [],
  },
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS THE CALL OF A MEMBER READ OFF A LOAD IN PLACE, and the file that calls the '
      + 'wrapper',
    files: [
      {
        file: 'imaginary/src/in-place-wrapper-wire.ts',
        text: textOf([
          'export async function inPlaceWrapperResponseFormat(schemaName: string,) {',
          `  return (await import('${CONTEST_SPECIFIER}')).contestResponseFormat({`,
          '    schemaName,',
          '    verdicts: [],',
          '    asksArchive: false,',
          '  },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/in-place-wrapper-gate-stage.ts',
        text: textOf([
          'export const IN_PLACE_WRAPPER_GATE_RESPONSE_FORMAT = await inPlaceWrapperResponseFormat(\'in_place\',);',
        ],),
      },
    ],
    found: [
      'imaginary/src/in-place-wrapper-gate-stage.ts',
      'imaginary/src/in-place-wrapper-wire.ts',
    ],
    builders: ['inPlaceWrapperResponseFormat',],
  },
];

/**
 Cases of `.call`, `.apply` and `.bind` read off a builder.
 */
export const CALL_METHOD_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS .call, .apply AND .bind READ OFF A BUILDER AS THE BUILDER, whether the file imports it, takes it '
      + 'from a namespace or never declares it',
    files: [
      {
        file: 'imaginary/src/method-free-stage.ts',
        text: textOf([
          `export const CALLED = contestResponseFormat.call(null, ${CONTEST_ARGUMENTS},);`,
        ],),
      },
      {
        file: 'imaginary/src/method-applied-stage.ts',
        text: textOf([
          `export const APPLIED = contestResponseFormat.apply(null, [${CONTEST_ARGUMENTS},],);`,
        ],),
      },
      {
        file: 'imaginary/src/method-bound-stage.ts',
        text: textOf([
          `export const BOUND = contestResponseFormat.bind(null, ${CONTEST_ARGUMENTS},);`,
        ],),
      },
      {
        file: 'imaginary/src/method-imported-stage.ts',
        text: textOf([
          `import { contestResponseFormat as formatOf, } from '${CONTEST_SPECIFIER}';`,
          '',
          `export const IMPORTED_CALLED = formatOf.call(null, ${CONTEST_ARGUMENTS},);`,
        ],),
      },
      {
        file: 'imaginary/src/method-namespace-stage.ts',
        text: textOf([
          `import * as contest from '${CONTEST_SPECIFIER}';`,
          '',
          `export const NAMESPACE_APPLIED = contest.contestResponseFormat.apply(null, [${CONTEST_ARGUMENTS},],);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/method-applied-stage.ts',
      'imaginary/src/method-bound-stage.ts',
      'imaginary/src/method-free-stage.ts',
      'imaginary/src/method-imported-stage.ts',
      'imaginary/src/method-namespace-stage.ts',
    ],
    builders: [],
  },
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS THE CALL OF A BUILDER THROUGH .call OR .apply, and the file that calls the '
      + 'wrapper',
    files: [
      {
        file: 'imaginary/src/method-wrapper-wire.ts',
        text: textOf([
          'export function calledResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat.call(null, { schemaName, verdicts: [], asksArchive: false, },);',
          '}',
          '',
          'export function appliedResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat.apply(null, [{ schemaName, verdicts: [], asksArchive: false, },],);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/method-wrapper-gate-stage.ts',
        text: textOf([
          'export const CALLED_GATE_RESPONSE_FORMAT = calledResponseFormat(\'called_gate\',);',
          'export const APPLIED_GATE_RESPONSE_FORMAT = appliedResponseFormat(\'applied_gate\',);',
        ],),
      },
    ],
    found: [
      'imaginary/src/method-wrapper-gate-stage.ts',
      'imaginary/src/method-wrapper-wire.ts',
    ],
    builders: [
      'appliedResponseFormat',
      'calledResponseFormat',
    ],
  },
  {
    name: 'READS A MEMBER NAMED call OF AN OBJECT A FILE DECLARES AS THAT MEMBER, not as the object, so a builder '
      + 'written as `kit.call` is the member',
    files: [
      {
        file: 'imaginary/src/call-member-wire.ts',
        text: textOf([
          'export const callKit = {',
          '  call(moods: readonly string[],) {',
          '    return {',
          ...methodFormatLinesOf({ name: 'call_mood', },),
          '    };',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/call-member-gate-stage.ts',
        text: textOf([
          'import { callKit, } from \'./call-member-wire.ts\';',
          '',
          'export const CALL_MEMBER_GATE_RESPONSE_FORMAT = callKit.call([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/call-member-gate-stage.ts',
      'imaginary/src/call-member-wire.ts',
    ],
    builders: ['callKit.call',],
  },
];

//endregion Forms the search follows to a builder: a load inside a function, a load read in place, and a call method
