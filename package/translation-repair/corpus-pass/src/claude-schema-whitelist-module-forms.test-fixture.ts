import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Modules that carry a builder in positions the search does not read
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) to hold the
// first and the second total of the check for the modules that carry a builder: a namespace or a loaded module used as
// a value, a load whose names the search cannot bind, a path literal outside an import, an export from and a load, and
// a builder inside a namespace, an enum, a parameter property or an export assignment. Each case is one imaginary file
// that holds one place on each line, with the files the search lists and the places the check reports, by file, line,
// shape and position.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'module\', verdicts: [], asksArchive: false, }';

/**
 Cases of a module that carries a builder used as a value, a path literal outside a load, and a builder in a construct
 the search does not read.
 */
export const WHITELIST_MODULE_CASES: readonly ShapeCase[] = [
  {
    name: 'LEAVES UNREAD A NAMESPACE USED AS A VALUE, as the object of a member computed from a name, an argument, a '
      + 'property, a spread or a rest, and a default import that reads a member of nothing, and the check reports '
      + 'each place',
    files: [
      {
        file: 'imaginary/src/module-value-stage.ts',
        text: textOf([
          `import * as contest from '${CONTEST_SPECIFIER}';`,
          `import contestDefault from '${CONTEST_SPECIFIER}';`,
          '',
          'const key = \'contestResponseFormat\';',
          `export const computed = contest[key](${ARGUMENTS},);`,
          'export const values = Object.values(contest,);',
          'export const registered = register(contest,);',
          'export const kit = { contest, };',
          'export const spread = { ...contest, };',
          'export const { ...all } = contest;',
          `export const fromDefault = contestDefault.contestResponseFormat(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/module-value-stage.ts',
        line: 5,
        shape: 'module-binding-outside-a-read-position',
        position: 'member computed from a name',
      },
      {
        file: 'imaginary/src/module-value-stage.ts',
        line: 6,
        shape: 'module-binding-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/module-value-stage.ts',
        line: 7,
        shape: 'module-binding-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/module-value-stage.ts',
        line: 8,
        shape: 'module-binding-outside-a-read-position',
        position: 'Property.value',
      },
      {
        file: 'imaginary/src/module-value-stage.ts',
        line: 9,
        shape: 'module-binding-outside-a-read-position',
        position: 'SpreadElement.argument',
      },
      {
        file: 'imaginary/src/module-value-stage.ts',
        line: 10,
        shape: 'module-binding-outside-a-read-position',
        position: 'VariableDeclarator.init',
      },
      {
        file: 'imaginary/src/module-value-stage.ts',
        line: 11,
        shape: 'module-binding-outside-a-read-position',
        position: 'member of a default the module does not export',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD A LOADED MODULE USED AS A VALUE OR TAKEN APART BY A PATTERN THE SEARCH CANNOT BIND, stored in '
      + 'an object, handed to a call, taken apart with a rest, a computed key or a nested pattern, and the check '
      + 'reports each place',
    files: [
      {
        file: 'imaginary/src/load-value-stage.ts',
        text: textOf([
          `const mod = await import('${CONTEST_SPECIFIER}');`,
          'export const kit = { mod, };',
          'export const registered = register(mod,);',
          `const { contestResponseFormat: take, ...rest } = await import('${CONTEST_SPECIFIER}');`,
          `const { ['contestResponseFormat']: computed, } = await import('${CONTEST_SPECIFIER}');`,
          `const { contestResponseFormat: { name, }, } = await import('${CONTEST_SPECIFIER}');`,
          `export const CALLED = mod.contestResponseFormat(${ARGUMENTS},);`,
          `const {} = await import('${CONTEST_SPECIFIER}');`,
          `const [first,] = await import('${CONTEST_SPECIFIER}');`,
        ],),
      },
    ],
    found: ['imaginary/src/load-value-stage.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/load-value-stage.ts',
        line: 2,
        shape: 'module-binding-outside-a-read-position',
        position: 'Property.value',
      },
      {
        file: 'imaginary/src/load-value-stage.ts',
        line: 3,
        shape: 'module-binding-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/load-value-stage.ts',
        line: 4,
        shape: 'load-in-unfollowed-form',
        position: 'VariableDeclarator.init',
      },
      {
        file: 'imaginary/src/load-value-stage.ts',
        line: 5,
        shape: 'load-in-unfollowed-form',
        position: 'VariableDeclarator.init',
      },
      {
        file: 'imaginary/src/load-value-stage.ts',
        line: 6,
        shape: 'load-in-unfollowed-form',
        position: 'VariableDeclarator.init',
      },
      {
        file: 'imaginary/src/load-value-stage.ts',
        line: 8,
        shape: 'load-in-unfollowed-form',
        position: 'VariableDeclarator.init',
      },
      {
        file: 'imaginary/src/load-value-stage.ts',
        line: 9,
        shape: 'load-in-unfollowed-form',
        position: 'VariableDeclarator.init',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD A PATH LITERAL THAT NAMES A MODULE CARRYING A BUILDER OUTSIDE AN IMPORT, AN EXPORT FROM AND A '
      + 'LOAD, as an argument of a call or of new, a constant, or the module of an import-equals declaration, and '
      + 'the check reports each',
    files: [
      {
        file: 'imaginary/src/path-literal-stage.ts',
        text: textOf([
          'import { createRequire, } from \'node:module\';',
          '',
          `export const viaCreate = createRequire(import.meta.url,)('${CONTEST_SPECIFIER}');`,
          `export const resolved = require.resolve('${CONTEST_SPECIFIER}');`,
          `export const meta = import.meta.resolve('${CONTEST_SPECIFIER}');`,
          `export const url = new URL('${CONTEST_SPECIFIER}', import.meta.url,);`,
          `export const kept = '${CONTEST_SPECIFIER}';`,
          `import equalled = require('${CONTEST_SPECIFIER}');`,
          `export type Typed = typeof import('${CONTEST_SPECIFIER}');`,
          `export const templated = \`${CONTEST_SPECIFIER}\`;`,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/path-literal-stage.ts',
        line: 3,
        shape: 'module-path-outside-a-load',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/path-literal-stage.ts',
        line: 4,
        shape: 'module-path-outside-a-load',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/path-literal-stage.ts',
        line: 5,
        shape: 'module-path-outside-a-load',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/path-literal-stage.ts',
        line: 6,
        shape: 'module-path-outside-a-load',
        position: 'NewExpression.arguments',
      },
      {
        file: 'imaginary/src/path-literal-stage.ts',
        line: 7,
        shape: 'module-path-outside-a-load',
        position: 'VariableDeclarator.init',
      },
      {
        file: 'imaginary/src/path-literal-stage.ts',
        line: 8,
        shape: 'module-path-outside-a-load',
        position: 'TSExternalModuleReference.expression',
      },
      {
        file: 'imaginary/src/path-literal-stage.ts',
        line: 10,
        shape: 'module-path-outside-a-load',
        position: 'VariableDeclarator.init',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD A BUILDER INSIDE A NAMESPACE, AN ENUM INITIALIZER, A PARAMETER PROPERTY OR AN EXPORT '
      + 'ASSIGNMENT, OR BROUGHT IN BY AN IMPORT-EQUALS DECLARATION, which the search leaves out with the types, and '
      + 'the check reports each',
    files: [
      {
        file: 'imaginary/src/construct-stage.ts',
        text: textOf([
          `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,
          '',
          'namespace Holder {',
          `  export const FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'enum Choice {',
          `  First = contestResponseFormat(${ARGUMENTS},).length,`,
          '}',
          '',
          'export class Params {',
          '  constructor(private readonly make = contestResponseFormat,) {}',
          '}',
          '',
          'export = contestResponseFormat;',
        ],),
      },
      {
        file: 'imaginary/src/equals-stage.ts',
        text: textOf([
          `import * as contest from '${CONTEST_SPECIFIER}';`,
          '',
          'import made = contest.contestResponseFormat;',
          '',
          'namespace Carried {',
          '  export const KEPT = contest;',
          '}',
          '',
          `export const CALLED = made(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/construct-stage.ts',
        line: 4,
        shape: 'builder-in-typescript-value-construct',
        position: 'TSModuleDeclaration',
      },
      {
        file: 'imaginary/src/construct-stage.ts',
        line: 8,
        shape: 'builder-in-typescript-value-construct',
        position: 'TSEnumDeclaration',
      },
      {
        file: 'imaginary/src/construct-stage.ts',
        line: 12,
        shape: 'builder-in-typescript-value-construct',
        position: 'TSParameterProperty',
      },
      {
        file: 'imaginary/src/construct-stage.ts',
        line: 15,
        shape: 'builder-in-typescript-value-construct',
        position: 'TSExportAssignment',
      },
      {
        file: 'imaginary/src/equals-stage.ts',
        line: 3,
        shape: 'builder-in-typescript-value-construct',
        position: 'TSImportEqualsDeclaration',
      },
      {
        file: 'imaginary/src/equals-stage.ts',
        line: 6,
        shape: 'builder-in-typescript-value-construct',
        position: 'TSModuleDeclaration',
      },
    ],
  },
];

//endregion Modules that carry a builder in positions the search does not read
