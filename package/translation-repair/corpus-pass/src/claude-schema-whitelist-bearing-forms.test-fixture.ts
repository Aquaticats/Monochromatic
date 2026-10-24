import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Which modules carry a builder, and loads in a file that names none
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) to hold the set
// of modules that carry a builder: a module that declares one, exports one by name, exports everything of a module that
// carries one, exports such a module as a namespace, exports an object that holds one, or exports a namespace of a
// module that became a carrier a pass earlier. A path literal that names such a module outside a load is reported; one
// that names a module that carries none is not. The second case holds a load whose path is no single literal in a file
// that names no module at all, which the check reads all the same.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The character that opens a substitution in a template, which an imaginary file's text spells out beside a brace.
 */
const DOLLAR = '$';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'bearing\', verdicts: [], asksArchive: false, }';

/**
 Cases of the modules that carry a builder, and of loads whose path is no literal.
 */
export const WHITELIST_BEARING_CASES: readonly ShapeCase[] = [
  {
    name: 'READS A MODULE AS CARRYING A BUILDER WHEN IT DECLARES ONE, EXPORTS ONE BY NAME, EXPORTS EVERYTHING OF A '
      + 'MODULE THAT CARRIES ONE, EXPORTS THAT MODULE AS A NAMESPACE, EXPORTS AN OBJECT THAT HOLDS ONE, OR EXPORTS A '
      + 'NAMESPACE OF A MODULE THAT CARRIES ONE ONLY AFTER A PASS, and reports a path literal that names any of them '
      + 'outside a load, and none that names a module that carries none',
    files: [
      {
        file: 'imaginary/src/bearing-named.ts',
        text: textOf([
          `export { contestResponseFormat as renamedFormat, } from '${CONTEST_SPECIFIER}';`,
        ],),
      },
      {
        file: 'imaginary/src/bearing-star.ts',
        text: textOf([
          `export * from '${CONTEST_SPECIFIER}';`,
        ],),
      },
      {
        file: 'imaginary/src/bearing-namespace.ts',
        text: textOf([
          `export * as contestModule from '${CONTEST_SPECIFIER}';`,
        ],),
      },
      {
        file: 'imaginary/src/bearing-object.ts',
        text: textOf([
          `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,
          '',
          'export const kit = {',
          '  make() {',
          `    return contestResponseFormat(${ARGUMENTS},);`,
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/bearing-seed.ts',
        text: textOf([
          'function hidden() {',
          '  return { type: \'json_schema\', };',
          '}',
          '',
          'export const NOT_A_FUNCTION = 1;',
        ],),
      },
      {
        file: 'imaginary/src/bearing-second.ts',
        text: textOf([
          'import * as barrel from \'./bearing-namespace.ts\';',
          '',
          'export { barrel, };',
        ],),
      },
      {
        file: 'imaginary/src/bearing-plain.ts',
        text: textOf([
          'export const PLAIN = 1;',
        ],),
      },
      {
        file: 'imaginary/src/bearing-consumer.ts',
        text: textOf([
          'export const PATHS = [',
          '  \'./bearing-named.ts\',',
          '  \'./bearing-star.ts\',',
          '  \'./bearing-namespace.ts\',',
          '  \'./bearing-object.ts\',',
          '  \'./bearing-seed.ts\',',
          '  \'./bearing-second.ts\',',
          '  \'./bearing-plain.ts\',',
          '];',
          '',
          'import plain from \'./bearing-plain.ts\';',
          '',
          'export const P = plain.value;',
        ],),
      },
    ],
    found: [
      'imaginary/src/bearing-object.ts',
      'imaginary/src/bearing-seed.ts',
    ],
    builders: [
      'hidden',
      'kit.make',
    ],
    spots: [
      {
        file: 'imaginary/src/bearing-consumer.ts',
        line: 2,
        shape: 'module-path-outside-a-load',
        position: 'ArrayExpression.elements',
      },
      {
        file: 'imaginary/src/bearing-consumer.ts',
        line: 3,
        shape: 'module-path-outside-a-load',
        position: 'ArrayExpression.elements',
      },
      {
        file: 'imaginary/src/bearing-consumer.ts',
        line: 4,
        shape: 'module-path-outside-a-load',
        position: 'ArrayExpression.elements',
      },
      {
        file: 'imaginary/src/bearing-consumer.ts',
        line: 5,
        shape: 'module-path-outside-a-load',
        position: 'ArrayExpression.elements',
      },
      {
        file: 'imaginary/src/bearing-consumer.ts',
        line: 6,
        shape: 'module-path-outside-a-load',
        position: 'ArrayExpression.elements',
      },
      {
        file: 'imaginary/src/bearing-consumer.ts',
        line: 7,
        shape: 'module-path-outside-a-load',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    name: 'REPORTS A LOAD WHOSE PATH IS NO SINGLE LITERAL IN A FILE THAT NAMES NO MODULE AT ALL, a name, a joined path '
      + 'and a template with a substitution, since such a load can reach any module',
    files: [
      {
        file: 'imaginary/src/computed-only-stage.ts',
        text: textOf([
          'export async function load(path: string,) {',
          '  return await import(path);',
          '}',
          '',
          'export function take(name: string,) {',
          '  return require(name);',
          '}',
          '',
          'export async function templated(name: string,) {',
          `  return await import(\`./${  DOLLAR  }{name}.ts\`);`,
          '}',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/computed-only-stage.ts',
        line: 2,
        shape: 'computed-load-path',
        position: 'path is not one literal',
      },
      {
        file: 'imaginary/src/computed-only-stage.ts',
        line: 6,
        shape: 'computed-load-path',
        position: 'path is not one literal',
      },
      {
        file: 'imaginary/src/computed-only-stage.ts',
        line: 10,
        shape: 'computed-load-path',
        position: 'path is not one literal',
      },
    ],
  },
];

//endregion Which modules carry a builder, and loads in a file that names none
