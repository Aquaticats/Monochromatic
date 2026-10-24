import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Calls and builders the search reads
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) as the control
// of the cases of the check: one imaginary file that holds a builder and a module that carries one in every position
// the search reads, and the check reports nothing for any of them. A change of the check that starts to report one of
// these positions fails here.

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
const ARGUMENTS = '{ schemaName: \'read\', verdicts: [], asksArchive: false, }';

/**
 Cases of the positions the search reads, for which the check reports nothing.
 */
export const WHITELIST_READ_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS NO CALL THE SEARCH READS: a call that is the whole of a return, handed to a call as a result, '
      + 'held by a top-level constant, tested, dropped by a sequence, or held by a local that no return reads, and '
      + 'no module path or builder the search follows',
    files: [
      {
        file: 'imaginary/src/read-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          `import * as contest from '${CONTEST_SPECIFIER}';`,
          `export { contestResponseFormat as renamedFormat, } from '${CONTEST_SPECIFIER}';`,
          `export * from '${CONTEST_SPECIFIER}';`,
          '',
          'export const alias = contestResponseFormat;',
          'export const casted = contestResponseFormat as Maker;',
          `export const FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          `export const NAMED = contest.contestResponseFormat(${ARGUMENTS},);`,
          `export const LITERAL = contest['contestResponseFormat'](${ARGUMENTS},);`,
          'export const TAGGED = contestResponseFormat`format`;',
          `export const OPTIONAL = contestResponseFormat?.(${ARGUMENTS},);`,
          `export const APPLIED = contestResponseFormat.apply(null, [${ARGUMENTS},],);`,
          `export const BOUND = contestResponseFormat.bind(null,)(${ARGUMENTS},);`,
          `export const CONSTRUCTED = new contestResponseFormat(${ARGUMENTS},);`,
          'export const { contestResponseFormat: taken, } = contest;',
          `export const WRAPPED = { schema: contestResponseFormat(${ARGUMENTS},), };`,
          'export type Made = typeof contestResponseFormat;',
          '',
          'export function sends() {',
          `  return send({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
          '',
          'export function wrapper() {',
          `  return contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'export function tests() {',
          `  return contestResponseFormat(${ARGUMENTS},) ? 'yes' : 'no';`,
          '}',
          '',
          'export function droppedBySequence() {',
          `  return (contestResponseFormat(${ARGUMENTS},), 1);`,
          '}',
          '',
          'export function holds() {',
          `  const format = contestResponseFormat(${ARGUMENTS},);`,
          '  return send(format,);',
          '}',
          '',
          'export function returnsOther() {',
          '  const format = 1;',
          '  return format;',
          '}',
          '',
          'export { contest, };',
          '',
          'export default contestResponseFormat;',
          '',
          `const lazy = await import('${CONTEST_SPECIFIER}');`,
          `const { contestResponseFormat: lazyFormat, } = await import('${CONTEST_SPECIFIER}');`,
          `const required = require('${CONTEST_SPECIFIER}');`,
          '',
          `export const LAZY = lazy.contestResponseFormat(${ARGUMENTS},);`,
          `export const LAZY_TAKEN = lazyFormat(${ARGUMENTS},);`,
          `export const REQUIRED = required.contestResponseFormat(${ARGUMENTS},);`,
          `export const IN_PLACE = (await import('${CONTEST_SPECIFIER}')).contestResponseFormat(${ARGUMENTS},);`,
        ],),
      },
      {
        file: 'imaginary/src/read-caller.ts',
        text: textOf([
          'import { wrapper, } from \'./read-stage.ts\';',
          '',
          'export const CALLED = wrapper();',
        ],),
      },
    ],
    found: [
      'imaginary/src/read-caller.ts',
      'imaginary/src/read-stage.ts',
    ],
    builders: ['wrapper',],
    spots: [],
  },
];

//endregion Calls and builders the search reads
