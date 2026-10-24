import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Calls and builders the search reads
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) as the control
// of the cases of the check. Each imaginary file holds one position the search reads, by itself, so the file is listed
// because of that position and no other; the check reports nothing for any of them. A change of the check that starts
// to report one of these positions fails here, and so does a change of the search that stops listing one, which keeps
// the check from whitelisting a position the search does not read.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The import of the contest builder that opens a file.
 */
const CONTEST_IMPORT = `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`;

/**
 The import of the whole contest module that opens a file.
 */
const CONTEST_NAMESPACE = `import * as contest from '${CONTEST_SPECIFIER}';`;

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'read\', verdicts: [], asksArchive: false, }';

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
 The positions the search reads, each in a file of its own.
 */
const READ_POSITIONS: readonly ReadPosition[] = [
  {
    name: 'read-called',
    lines: [
      CONTEST_IMPORT,
      `export const CALLED = contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-constructed',
    lines: [
      CONTEST_IMPORT,
      `export const CONSTRUCTED = new contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-tagged',
    lines: [
      CONTEST_IMPORT,
      'export const TAGGED = contestResponseFormat`format`;',
    ],
  },
  {
    name: 'read-optional',
    lines: [
      CONTEST_IMPORT,
      `export const OPTIONAL = contestResponseFormat?.(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-applied',
    lines: [
      CONTEST_IMPORT,
      `export const APPLIED = contestResponseFormat.apply(null, [${ARGUMENTS},],);`,
    ],
  },
  {
    name: 'read-call-method',
    lines: [
      CONTEST_IMPORT,
      `export const CALLED_METHOD = contestResponseFormat.call(null, ${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-bound',
    lines: [
      CONTEST_IMPORT,
      `export const BOUND = contestResponseFormat.bind(null,)(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-typeof-type',
    lines: [
      CONTEST_IMPORT,
      'export type Made = typeof contestResponseFormat;',
    ],
  },
  {
    name: 'read-alias',
    lines: [
      CONTEST_IMPORT,
      'const alias = contestResponseFormat;',
      `export const ALIASED = alias(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-casted-alias',
    lines: [
      CONTEST_IMPORT,
      'const casted = contestResponseFormat as Maker;',
      `export const CASTED = casted(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-namespace-member',
    lines: [
      CONTEST_NAMESPACE,
      `export const NAMED = contest.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-namespace-literal',
    lines: [
      CONTEST_NAMESPACE,
      `export const LITERAL = contest['contestResponseFormat'](${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-namespace-taken',
    lines: [
      CONTEST_NAMESPACE,
      'const { contestResponseFormat: taken, } = contest;',
      `export const TAKEN = taken(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-load-held',
    lines: [
      `const lazy = await import('${CONTEST_SPECIFIER}');`,
      `export const LAZY = lazy.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-load-taken',
    lines: [
      `const { contestResponseFormat: lazyFormat, } = await import('${CONTEST_SPECIFIER}');`,
      `export const LAZY_TAKEN = lazyFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-required',
    lines: [
      `const required = require('${CONTEST_SPECIFIER}');`,
      `export const REQUIRED = required.contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-load-in-place',
    lines: [`export const IN_PLACE = (await import('${CONTEST_SPECIFIER}')).contestResponseFormat(${ARGUMENTS},);`,],
  },
  {
    name: 'read-handed-result',
    lines: [
      CONTEST_IMPORT,
      `export const SENT = send({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
    ],
  },
  {
    name: 'read-held-result',
    lines: [
      CONTEST_IMPORT,
      'export function holds() {',
      `  const format = contestResponseFormat(${ARGUMENTS},);`,
      '  return send(format,);',
      '}',
      'export function returnsOther() {',
      "  const format = 'unrelated';",
      '  return format;',
      '}',
    ],
  },
  {
    name: 'read-tested-result',
    lines: [
      CONTEST_IMPORT,
      'export function tests() {',
      `  return contestResponseFormat(${ARGUMENTS},) ? 'yes' : 'no';`,
      '}',
    ],
  },
  {
    name: 'read-sequenced-result',
    lines: [
      CONTEST_IMPORT,
      'export function droppedBySequence() {',
      `  return (contestResponseFormat(${ARGUMENTS},), 1);`,
      '}',
    ],
  },
  {
    name: 'read-wrapper',
    lines: [
      CONTEST_IMPORT,
      'export function wrapper() {',
      `  return contestResponseFormat(${ARGUMENTS},);`,
      '}',
    ],
  },
];

/**
 The positions the search reads through another file: a builder exported again under another name, by a star, or as the
 default, each with the file that calls it through that file.
 */
const READ_THROUGH_FILES: readonly ReadPosition[] = [
  {
    name: 'read-re-exported-stage',
    lines: [`export { contestResponseFormat as renamedFormat, } from '${CONTEST_SPECIFIER}';`,],
  },
  {
    name: 'read-re-exported-caller',
    lines: [
      "import { renamedFormat, } from './read-re-exported-stage.ts';",
      `export const RENAMED = renamedFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-star-exported-stage',
    lines: [`export * from '${CONTEST_SPECIFIER}';`,],
  },
  {
    name: 'read-star-exported-caller',
    lines: [
      "import { contestResponseFormat, } from './read-star-exported-stage.ts';",
      `export const STARRED = contestResponseFormat(${ARGUMENTS},);`,
    ],
  },
  {
    name: 'read-default-exported-stage',
    lines: [
      CONTEST_IMPORT,
      'export default contestResponseFormat;',
    ],
  },
  {
    name: 'read-default-exported-caller',
    lines: [
      "import made from './read-default-exported-stage.ts';",
      `export const DEFAULTED = made(${ARGUMENTS},);`,
    ],
  },
];

/**
 Cases of the positions the search reads, for which the check reports nothing.
 */
export const WHITELIST_READ_CASES: readonly ShapeCase[] = [
  {
    name: 'LISTS EACH FILE THAT HOLDS A POSITION THE SEARCH READS BY ITSELF AND REPORTS NO PLACE IN IT: a builder '
      + 'called, constructed, tagged, called with a question mark, applied, bound, called by its call method, aliased, '
      + 'cast, named in a type only (listed by nothing, reported by nothing), read through a '
      + 'namespace by a member, a literal key or a pattern, through a load held, taken apart, required or read in '
      + 'place, its result handed to a call, held, tested or dropped by a sequence, a wrapper, and a builder exported '
      + 'again, by a star or as the default, listed through the file that calls it',
    files: [
      ...READ_POSITIONS.map(function fileOfPosition({
        name,
        lines,
      },) {
        return {
          file: `imaginary/src/${name}-stage.ts`,
          text: textOf(lines,),
        };
      },),
      ...READ_THROUGH_FILES.map(function fileOfThrough({
        name,
        lines,
      },) {
        return {
          file: `imaginary/src/${name}.ts`,
          text: textOf(lines,),
        };
      },),
      {
        file: 'imaginary/src/read-wrapper-caller.ts',
        text: textOf([
          'import { wrapper, } from \'./read-wrapper-stage.ts\';',
          '',
          'export const CALLED = wrapper();',
        ],),
      },
    ],
    found: [
      'imaginary/src/read-alias-stage.ts',
      'imaginary/src/read-applied-stage.ts',
      'imaginary/src/read-bound-stage.ts',
      'imaginary/src/read-call-method-stage.ts',
      'imaginary/src/read-called-stage.ts',
      'imaginary/src/read-casted-alias-stage.ts',
      'imaginary/src/read-constructed-stage.ts',
      'imaginary/src/read-default-exported-caller.ts',
      'imaginary/src/read-handed-result-stage.ts',
      'imaginary/src/read-held-result-stage.ts',
      'imaginary/src/read-load-held-stage.ts',
      'imaginary/src/read-load-in-place-stage.ts',
      'imaginary/src/read-load-taken-stage.ts',
      'imaginary/src/read-namespace-literal-stage.ts',
      'imaginary/src/read-namespace-member-stage.ts',
      'imaginary/src/read-namespace-taken-stage.ts',
      'imaginary/src/read-optional-stage.ts',
      'imaginary/src/read-re-exported-caller.ts',
      'imaginary/src/read-required-stage.ts',
      'imaginary/src/read-sequenced-result-stage.ts',
      'imaginary/src/read-star-exported-caller.ts',
      'imaginary/src/read-tagged-stage.ts',
      'imaginary/src/read-tested-result-stage.ts',
      'imaginary/src/read-wrapper-caller.ts',
      'imaginary/src/read-wrapper-stage.ts',
    ],
    builders: ['wrapper',],
    spots: [],
  },
];

//endregion Calls and builders the search reads
