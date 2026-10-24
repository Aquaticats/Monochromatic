import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Builders in positions the search does not read
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) to hold the part
// of the second total that reads where a builder itself stands: stored in a class field, handed to a call, used as a
// default, assigned, chosen by a conditional, bound, put in an array, wrapped in a proxy, read for a member that is no
// call method, or aliased by a constant the search does not bind; and an object that holds a builder, used as a value.
// Each case is imaginary files that hold one place on each line, beside the files that call through them, with the
// files the search lists and the places the check reports, by file, line, shape and position.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The import of the contest builder that opens each stage file.
 */
const CONTEST_IMPORT = `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`;

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'whitelist\', verdicts: [], asksArchive: false, }';

/**
 Cases of a builder in a position the search does not read.
 */
export const WHITELIST_CASES: readonly ShapeCase[] = [
  {
    name: 'LEAVES UNREAD A BUILDER STORED, HANDED OVER, DEFAULTED, ASSIGNED OR ALIASED IN A POSITION THE SEARCH '
      + 'DOES NOT READ, so the file that calls what holds it is not listed, and the check reports each position',
    files: [
      {
        file: 'imaginary/src/position-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export class Kit {',
          '  made = contestResponseFormat;',
          '  static memo = memoize(contestResponseFormat,);',
          '}',
          '',
          'export function withDefault(make = contestResponseFormat,) {',
          '  return make;',
          '}',
          '',
          'export function withPatternDefault({ make = contestResponseFormat, } = {},) {',
          '  return make;',
          '}',
          '',
          'export const registry = new Map();',
          'registry.set(\'contest\', contestResponseFormat,);',
          'export default memoize(debounce(contestResponseFormat,),);',
          'export let assigned;',
          'assigned = contestResponseFormat;',
          'export const viaTernary = flag ? contestResponseFormat : other;',
          'export const viaFallback = contestResponseFormat || other;',
          'export const viaBind = contestResponseFormat.bind(null,);',
          'export const inArray = [contestResponseFormat,];',
          'export const proxied = new Proxy(contestResponseFormat, {},);',
          'exports.made = contestResponseFormat;',
          'export const named = contestResponseFormat.name;',
          'export const key = contestResponseFormat[tag];',
          'export const sequenced = (0, contestResponseFormat);',
          'export const getter = { get: () => contestResponseFormat, };',
        ],),
      },
      {
        file: 'imaginary/src/position-shadow-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export function first() {',
          '  const make = other;',
          '  return make;',
          '}',
          '',
          'export function second() {',
          '  const make = contestResponseFormat;',
          '  return make;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/position-caller.ts',
        text: textOf([
          'import { Kit, } from \'./position-stage.ts\';',
          '',
          `export const CALLED = new Kit().made(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/position-shadow-stage.ts',
      'imaginary/src/position-stage.ts',
    ],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/position-stage.ts',
        line: 4,
        shape: 'builder-outside-a-read-position',
        position: 'PropertyDefinition.value',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 5,
        shape: 'builder-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 8,
        shape: 'builder-outside-a-read-position',
        position: 'AssignmentPattern.right',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 12,
        shape: 'builder-outside-a-read-position',
        position: 'AssignmentPattern.right',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 17,
        shape: 'builder-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 18,
        shape: 'builder-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 20,
        shape: 'builder-outside-a-read-position',
        position: 'AssignmentExpression.right',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 21,
        shape: 'builder-outside-a-read-position',
        position: 'ConditionalExpression.consequent',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 22,
        shape: 'builder-outside-a-read-position',
        position: 'LogicalExpression.left',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 23,
        shape: 'builder-outside-a-read-position',
        position: 'VariableDeclarator.init',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 24,
        shape: 'builder-outside-a-read-position',
        position: 'ArrayExpression.elements',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 25,
        shape: 'builder-outside-a-read-position',
        position: 'NewExpression.arguments',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 26,
        shape: 'builder-outside-a-read-position',
        position: 'AssignmentExpression.right',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 27,
        shape: 'builder-outside-a-read-position',
        position: 'member name of a builder',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 28,
        shape: 'builder-outside-a-read-position',
        position: 'member computed from a name',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 29,
        shape: 'builder-outside-a-read-position',
        position: 'SequenceExpression.expressions',
      },
      {
        file: 'imaginary/src/position-stage.ts',
        line: 30,
        shape: 'builder-outside-a-read-position',
        position: 'ArrowFunctionExpression.body',
      },
      {
        file: 'imaginary/src/position-shadow-stage.ts',
        line: 9,
        shape: 'builder-outside-a-read-position',
        position: 'VariableDeclarator.init',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD AN OBJECT THAT HOLDS A BUILDER USED AS A VALUE, handed to a call or stored in an object, '
      + 'and the builder taken out of it and handed to a call, whether the file imports the object or never declares '
      + 'it, and the check reports each, and reports nothing for the object aliased, a member called, aliased or '
      + 'written as a string',
    files: [
      {
        file: 'imaginary/src/carrier-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export const kit = {',
          '  make() {',
          `    return contestResponseFormat(${ARGUMENTS},);`,
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/carrier-caller.ts',
        text: textOf([
          'import { kit, } from \'./carrier-stage.ts\';',
          '',
          'export const SAME = kit;',
          'export const CALLED = kit.make();',
          'export const TAKEN = kit.make;',
          'export const LITERAL = kit[\'make\']();',
          'export const registered = register(kit,);',
          'export const stored = { kit, };',
          'export const handed = register(kit.make,);',
        ],),
      },
      {
        file: 'imaginary/src/carrier-free-caller.ts',
        text: textOf([
          'export const FREE_CALLED = kit.make();',
          'export const FREE_HANDED = register(kit.make,);',
        ],),
      },
    ],
    found: [
      'imaginary/src/carrier-caller.ts',
      'imaginary/src/carrier-free-caller.ts',
      'imaginary/src/carrier-stage.ts',
    ],
    builders: ['kit.make',],
    spots: [
      {
        file: 'imaginary/src/carrier-caller.ts',
        line: 7,
        shape: 'module-binding-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/carrier-caller.ts',
        line: 8,
        shape: 'module-binding-outside-a-read-position',
        position: 'Property.value',
      },
      {
        file: 'imaginary/src/carrier-caller.ts',
        line: 9,
        shape: 'builder-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
      {
        file: 'imaginary/src/carrier-free-caller.ts',
        line: 2,
        shape: 'builder-outside-a-read-position',
        position: 'CallExpression.arguments',
      },
    ],
  },
];

//endregion Builders in positions the search does not read
