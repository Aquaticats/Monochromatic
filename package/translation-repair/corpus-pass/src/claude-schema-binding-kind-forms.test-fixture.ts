import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Bindings below the top of a file, and members of a builder that are no builder
// What `claude-schemas.unit.test.ts` feeds the builder search to pin three halves of the sentence on bindings inside a
// function or a block, and the members of a builder that are not read as the builder: a name that a nested declaration
// binds to another name (an alias), a declaration written `let` or `var`, a load bound inside an `if` block, and the
// members `name`, `length` and `toString` of a builder, which stand for nothing. Each case is a few imaginary files,
// with the contest builder (`contestResponseFormat`) beside them, and the files and builders the search must list.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const CONTEST_ARGUMENTS = '{ schemaName: \'kind\', verdicts: [], asksArchive: false, }';

/**
 Cases of a binding made inside a function or a block, beyond a `const` that holds a load.
 */
export const BINDING_KIND_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A NAME THAT A CONSTANT INSIDE A FUNCTION BINDS TO ANOTHER NAME, so a function that returns the call '
      + 'of the alias is a wrapper and the file that calls it is listed',
    files: [
      {
        file: 'imaginary/src/nested-alias-wire.ts',
        text: textOf([
          `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,
          '',
          'export function aliasedResponseFormat(schemaName: string,) {',
          '  const make = contestResponseFormat;',
          '  return make({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/nested-alias-gate-stage.ts',
        text: textOf([
          'export const NESTED_ALIAS_GATE_RESPONSE_FORMAT = aliasedResponseFormat(\'nested_alias\',);',
        ],),
      },
    ],
    found: [
      'imaginary/src/nested-alias-gate-stage.ts',
      'imaginary/src/nested-alias-wire.ts',
    ],
    builders: ['aliasedResponseFormat',],
  },
  {
    name: 'FOLLOWS A NAME THAT A DECLARATION WRITTEN let OR var INSIDE A FUNCTION BINDS, not only one written const',
    files: [
      {
        file: 'imaginary/src/nested-let-wire.ts',
        text: textOf([
          `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,
          '',
          'export function letResponseFormat(schemaName: string,) {',
          '  let make = contestResponseFormat;',
          '  return make({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/nested-var-wire.ts',
        text: textOf([
          'export async function varResponseFormat(schemaName: string,) {',
          `  var contest = await import('${CONTEST_SPECIFIER}');`,
          '  return contest.contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/nested-declaration-gate-stage.ts',
        text: textOf([
          'export const LET_GATE_RESPONSE_FORMAT = letResponseFormat(\'let_gate\',);',
          'export const VAR_GATE_RESPONSE_FORMAT = await varResponseFormat(\'var_gate\',);',
        ],),
      },
    ],
    found: [
      'imaginary/src/nested-declaration-gate-stage.ts',
      'imaginary/src/nested-let-wire.ts',
      'imaginary/src/nested-var-wire.ts',
    ],
    builders: [
      'letResponseFormat',
      'varResponseFormat',
    ],
  },
  {
    name: 'FOLLOWS A LOAD BOUND INSIDE AN if BLOCK, not only one bound in the body of a function, so a function that '
      + 'returns the call of the member it takes out is a wrapper',
    files: [
      {
        file: 'imaginary/src/nested-block-wire.ts',
        text: textOf([
          'export async function blockResponseFormat(schemaName: string, flag: boolean,) {',
          '  if (flag) {',
          `    const { contestResponseFormat: make, } = await import('${CONTEST_SPECIFIER}');`,
          '    return make({ schemaName, verdicts: [], asksArchive: false, },);',
          '  }',
          '  return undefined;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/nested-block-gate-stage.ts',
        text: textOf([
          'export const BLOCK_GATE_RESPONSE_FORMAT = await blockResponseFormat(\'block_gate\', true,);',
        ],),
      },
    ],
    found: [
      'imaginary/src/nested-block-gate-stage.ts',
      'imaginary/src/nested-block-wire.ts',
    ],
    builders: ['blockResponseFormat',],
  },
];

/**
 Cases of a member of a builder that is no builder.
 */
export const BUILDER_MEMBER_CASES: readonly FormCase[] = [
  {
    name: 'READS NO USE OF A BUILDER IN ITS MEMBERS name, length AND toString, only .call, .apply AND .bind are the '
      + 'builder, so a function that returns the call of one of them is no wrapper',
    files: [
      {
        file: 'imaginary/src/member-imported-stage.ts',
        text: textOf([
          `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,
          '',
          'export const MEMBER_NAME = contestResponseFormat.name;',
          'export const MEMBER_LENGTH = contestResponseFormat.length;',
          '',
          'export function renderedResponseFormat() {',
          `  return contestResponseFormat.toString(${CONTEST_ARGUMENTS});`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/member-free-stage.ts',
        text: textOf([
          'export const FREE_MEMBER_NAME = contestResponseFormat.name;',
          '',
          'export function freeRenderedResponseFormat() {',
          `  return contestResponseFormat.toString(${CONTEST_ARGUMENTS});`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/member-caller-stage.ts',
        text: textOf([
          'export const RENDERED = renderedResponseFormat();',
          'export const FREE_RENDERED = freeRenderedResponseFormat();',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
];

//endregion Bindings below the top of a file, and members of a builder that are no builder
