import { methodFormatLinesOf, } from './claude-schema-builder-forms.test-fixture.ts';
import {
  callOf,
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Names a file gives a builder, and the rules that decide which function a name is
// What `claude-schemas.unit.test.ts` feeds the builder search to show how a file's own names lead to a builder: a
// dynamic `import()` or `require` of a literal path, a destructuring, a second name; and what decides that a name is
// the builder or is not: a function of the same name in another module, a specifier that names no file of the
// sources, a name the file never declares or imports, a parameter of the builder's name, and a path that is no literal.
// Each case is a few imaginary files, with the contest builder (`contestResponseFormat`) beside them, and the files
// and builders the search must list.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 Cases in which a file brings a builder in by a dynamic import, a `require`, a destructuring or a second name.
 */
export const BINDING_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A BUILDER BROUGHT IN BY A DYNAMIC import() OR A require OF A LITERAL PATH, whole or '
      + 'destructured with a new name',
    files: [
      {
        file: 'imaginary/src/dynamic-whole-gate-stage.ts',
        text: textOf([
          `const contest = await import('${CONTEST_SPECIFIER}');`,
          '',
          callOf('contest.contestResponseFormat',),
        ],),
      },
      {
        file: 'imaginary/src/dynamic-destructured-gate-stage.ts',
        text: textOf([
          `const { contestResponseFormat: lazyFormat, } = await import('${CONTEST_SPECIFIER}');`,
          '',
          callOf('lazyFormat',),
        ],),
      },
      {
        file: 'imaginary/src/required-whole-gate-stage.ts',
        text: textOf([
          `const contest = require('${CONTEST_SPECIFIER}');`,
          '',
          callOf('contest.contestResponseFormat',),
        ],),
      },
      {
        file: 'imaginary/src/required-destructured-gate-stage.ts',
        text: textOf([
          `const { contestResponseFormat: requiredFormat, } = require('${CONTEST_SPECIFIER}');`,
          '',
          callOf('requiredFormat',),
        ],),
      },
    ],
    found: [
      'imaginary/src/dynamic-destructured-gate-stage.ts',
      'imaginary/src/dynamic-whole-gate-stage.ts',
      'imaginary/src/required-destructured-gate-stage.ts',
      'imaginary/src/required-whole-gate-stage.ts',
    ],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER BROUGHT IN BY A DESTRUCTURED import() WITHOUT A NEW NAME, as in `const { name } = '
      + 'await import(...)`',
    files: [
      {
        file: 'imaginary/src/lazy-gate-stage.ts',
        text: textOf([
          `const { contestResponseFormat, } = await import('${CONTEST_SPECIFIER}');`,
          '',
          callOf('contestResponseFormat',),
        ],),
      },
    ],
    found: ['imaginary/src/lazy-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER THAT A FILE ASSIGNS TO A SECOND NAME, once or twice',
    files: [
      {
        file: 'imaginary/src/second-name-gate-stage.ts',
        text: textOf([
          `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,
          '',
          'const makeFormat = contestResponseFormat;',
          'const makeAgain = makeFormat;',
          '',
          callOf('makeAgain',),
        ],),
      },
    ],
    found: ['imaginary/src/second-name-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER THAT IS A MEMBER OF AN OBJECT AND IS TAKEN OUT BY DESTRUCTURING, with a new name or '
      + 'without',
    files: [
      {
        file: 'imaginary/src/taken-kit-wire.ts',
        text: textOf([
          'export const takenKit = {',
          '  takenResponseFormat(moods: readonly string[],) {',
          '    return {',
          ...methodFormatLinesOf({ name: 'taken_mood', },),
          '    };',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/taken-gate-stage.ts',
        text: textOf([
          'import { takenKit, } from \'./taken-kit-wire.ts\';',
          '',
          'const { takenResponseFormat: makeTaken, } = takenKit;',
          '',
          'export const TAKEN_GATE_RESPONSE_FORMAT = makeTaken([\'wags\',],);',
        ],),
      },
      {
        file: 'imaginary/src/taken-plain-gate-stage.ts',
        text: textOf([
          'import { takenKit, } from \'./taken-kit-wire.ts\';',
          '',
          'const { takenResponseFormat, } = takenKit;',
          '',
          'export const TAKEN_PLAIN_GATE_RESPONSE_FORMAT = takenResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/taken-gate-stage.ts',
      'imaginary/src/taken-kit-wire.ts',
      'imaginary/src/taken-plain-gate-stage.ts',
    ],
    builders: ['takenKit.takenResponseFormat',],
  },
];

/**
 Cases of what decides that a name is the builder or is not: a function of the same name in another module or in the
 file itself, a specifier that names no file, a name the file neither declares nor imports, a dotted name of that
 kind, a parameter of the builder's name, and a path that is no literal.
 */
export const NAME_RULE_CASES: readonly FormCase[] = [
  {
    name: 'READS A FUNCTION OF THE BUILDER\'S NAME THAT A FILE DECLARES OR IMPORTS FROM ANOTHER MODULE AS THAT '
      + 'FUNCTION AND NOT AS THE BUILDER',
    files: [
      {
        file: 'imaginary/src/same-name-wire.ts',
        text: textOf([
          'export function contestResponseFormat(label: string,) {',
          '  return label;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/same-name-importer.ts',
        text: textOf([
          'import { contestResponseFormat, } from \'./same-name-wire.ts\';',
          '',
          'export const LABEL = contestResponseFormat(\'label\',);',
        ],),
      },
      {
        file: 'imaginary/src/same-name-own.ts',
        text: textOf([
          'function contestResponseFormat(label: string,) {',
          '  return label;',
          '}',
          '',
          'export const OWN_LABEL = contestResponseFormat(\'own\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'READS A NAME IMPORTED FROM A SPECIFIER THAT NAMES NO FILE OF THE SOURCES AS THE BUILDER OF THAT NAME',
    files: [
      {
        file: 'imaginary/src/outside-gate-stage.ts',
        text: textOf([
          'import { contestResponseFormat, } from \'some-package-outside-the-family\';',
          '',
          callOf('contestResponseFormat',),
        ],),
      },
      {
        file: 'imaginary/src/outside-namespace-gate-stage.ts',
        text: textOf([
          'import * as outside from \'some-package-outside-the-family\';',
          '',
          callOf('outside.contestResponseFormat',),
        ],),
      },
    ],
    found: [
      'imaginary/src/outside-gate-stage.ts',
      'imaginary/src/outside-namespace-gate-stage.ts',
    ],
    builders: [],
  },
  {
    name: 'READS A DOTTED NAME THE FILE NEITHER DECLARES NOR IMPORTS AS THE BUILDER OF THAT NAME, when the builder '
      + 'is a member of an object',
    files: [
      {
        file: 'imaginary/src/free-kit-wire.ts',
        text: textOf([
          'export const freeKit = {',
          '  freeResponseFormat(moods: readonly string[],) {',
          '    return {',
          ...methodFormatLinesOf({ name: 'free_mood', },),
          '    };',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/free-kit-gate-stage.ts',
        text: textOf(['export const FREE_KIT_RESPONSE_FORMAT = freeKit.freeResponseFormat([\'wags\',],);',],),
      },
    ],
    found: [
      'imaginary/src/free-kit-gate-stage.ts',
      'imaginary/src/free-kit-wire.ts',
    ],
    builders: ['freeKit.freeResponseFormat',],
  },
  {
    name: 'READS A PARAMETER OF THE BUILDER\'S NAME AS THE BUILDER, which the search does not tell apart and so '
      + 'over-reads',
    files: [
      {
        file: 'imaginary/src/shadowing.ts',
        text: textOf([
          'export function shadowing(contestResponseFormat: (label: string,) => string,) {',
          '  return contestResponseFormat(\'label\',);',
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/shadowing.ts',],
    builders: ['shadowing',],
  },
  {
    name: 'FOLLOWS NO NAME FROM AN import() OR A require WHOSE PATH IS NOT A LITERAL, so a builder reached that '
      + 'way is not found',
    files: [
      {
        file: 'imaginary/src/computed-path-gate-stage.ts',
        text: textOf([
          'const path = \'../../lane-contest/src/contest-ballot-wire.ts\';',
          'const contest = await import(path);',
          'const required = require(path);',
          '',
          callOf('contest.contestResponseFormat',),
          callOf('required.contestResponseFormat',),
        ],),
      },
    ],
    found: [],
    builders: [],
  },
];

//endregion Names a file gives a builder, and the rules that decide which function a name is
