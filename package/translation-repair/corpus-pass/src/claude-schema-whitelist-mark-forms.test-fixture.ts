import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Marks around a builder or a load, a call used as a key, and loads of modules that carry no builder
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) as the second
// control of the cases of the check, in the form of `claude-schema-whitelist-read-forms.test-fixture.ts`: each
// imaginary file holds one position the search reads by itself, here a builder under a cast, a non-null mark, a
// satisfies mark or parentheses, a load under each mark or called optionally, a call used as the key of a member, and,
// listed by nothing, loads of modules that carry no builder in every form. The check reports nothing for any of them.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The import of the contest builder that opens a file.
 */
const CONTEST_IMPORT = `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`;

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'mark\', verdicts: [], asksArchive: false, }';

/**
 One position the search reads, as the lines of the file that holds it.
 */
type ReadPosition = {
  /**
   Name of the file, without its directory and extension.
   */
  readonly name: string;

  /**
   Lines of the file.
   */
  readonly lines: readonly string[];
};

/**
 The positions the search reads under a mark, each in a file of its own.
 */
const MARK_POSITIONS: readonly ReadPosition[] = [
  {
    name: 'read-callee-cast',
    lines: [
      CONTEST_IMPORT,
      `export const CASTED_CALLEE = (contestResponseFormat as Maker)(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-callee-non-null',
    lines: [
      CONTEST_IMPORT,
      `export const NON_NULL_CALLEE = contestResponseFormat!(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-callee-satisfies',
    lines: [
      CONTEST_IMPORT,
      `export const SATISFIED_CALLEE = (contestResponseFormat satisfies Maker)(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-callee-parenthesized',
    lines: [
      CONTEST_IMPORT,
      `export const PARENTHESIZED_CALLEE = (contestResponseFormat)(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-computed-key',
    lines: [
      CONTEST_IMPORT,
      'export function keyed(table: Table,) {',
      `  return table[contestResponseFormat(${ARGUMENTS},)];`,
      '}',
    ],
  },
  {
    name: 'read-load-cast',
    lines: [
      `const lazy = (await import('${CONTEST_SPECIFIER}')) as Maker;`,
      `export const LAZY_CAST = lazy.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-load-non-null',
    lines: [
      `const lazy = (await import('${CONTEST_SPECIFIER}'))!;`,
      `export const LAZY_NON_NULL = lazy.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-load-satisfies',
    lines: [
      `const lazy = (await import('${CONTEST_SPECIFIER}')) satisfies Maker;`,
      `export const LAZY_SATISFIED = lazy.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-load-assertion',
    lines: [
      `const lazy = <Maker>(await import('${CONTEST_SPECIFIER}'));`,
      `export const LAZY_ASSERTED = lazy.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-load-optional-call',
    lines: [
      `const required = require?.('${CONTEST_SPECIFIER}');`,
      `export const REQUIRED_OPTIONAL = required.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-other-loads',
    lines: [
      'export const LATER = import(\'node:fs\').then((fs,) => fs,);',
      'void import(\'node:os\');',
      'export const REQUIRED_OTHER = [require(\'node:path\'),];',
    ],
  },
];

/**
 Cases of the marks the search reads through, for which the check reports nothing.
 */
export const WHITELIST_MARK_CASES: readonly ShapeCase[] = [
  {
    name: 'LISTS EACH FILE THAT HOLDS A BUILDER OR A LOAD UNDER A MARK THE SEARCH READS THROUGH AND REPORTS NO PLACE '
      + 'IN IT: a cast, a non-null mark, a satisfies mark, parentheses, a call used as the key of a member, a load '
      + 'under each mark or called optionally, and loads of modules that carry no builder, which nothing lists',
    files: MARK_POSITIONS.map(function fileOfPosition({
      name,
      lines,
    },) {
      return {
        file: `imaginary/src/${name}-stage.ts`,
        text: textOf(lines,),
      };
    },),
    found: [
      'imaginary/src/read-callee-cast-stage.ts',
      'imaginary/src/read-callee-non-null-stage.ts',
      'imaginary/src/read-callee-parenthesized-stage.ts',
      'imaginary/src/read-callee-satisfies-stage.ts',
      'imaginary/src/read-computed-key-stage.ts',
      'imaginary/src/read-load-assertion-stage.ts',
      'imaginary/src/read-load-cast-stage.ts',
      'imaginary/src/read-load-non-null-stage.ts',
      'imaginary/src/read-load-optional-call-stage.ts',
      'imaginary/src/read-load-satisfies-stage.ts',
    ],
    builders: [],
    spots: [],
  },
];

//endregion Marks around a builder or a load, a call used as a key, and loads of modules that carry no builder
