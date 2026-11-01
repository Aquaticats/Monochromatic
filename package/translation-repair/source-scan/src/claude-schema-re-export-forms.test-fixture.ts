import { formatLinesOf, } from './claude-schema-builder-forms.test-fixture.ts';
import {
  callOf,
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Modules that re-export a builder
// What `claude-schemas.unit.test.ts` feeds the builder search to show how it follows a builder from the module that
// declares it to the file that calls it through a module that exports it again: by name, under another name, with
// `export *`, as a namespace, through a list that exports an import, and, for a default export, through
// `export { default } from` and through `export default name;`. Each case is a few imaginary files, with the contest
// builder (`contestResponseFormat`) beside the first group, and the files and builders the search must list.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 Cases in which a file reaches the contest builder through a module that re-exports it.
 */
export const RE_EXPORT_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A BUILDER THROUGH A MODULE THAT RE-EXPORTS IT BY NAME, and the file that imports it from there',
    files: [
      {
        file: 'imaginary/src/named-barrel.ts',
        text: textOf([`export { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,],),
      },
      {
        file: 'imaginary/src/named-barrel-gate-stage.ts',
        text: textOf([
          'import { contestResponseFormat, } from \'./named-barrel.ts\';',
          '',
          callOf('contestResponseFormat',),
        ],),
      },
    ],
    found: ['imaginary/src/named-barrel-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER THROUGH A MODULE THAT RE-EXPORTS IT UNDER ANOTHER NAME, and the file that imports '
      + 'that name',
    files: [
      {
        file: 'imaginary/src/renaming-barrel.ts',
        text: textOf([`export { contestResponseFormat as formatFor, } from '${CONTEST_SPECIFIER}';`,],),
      },
      {
        file: 'imaginary/src/renaming-barrel-gate-stage.ts',
        text: textOf([
          'import { formatFor, } from \'./renaming-barrel.ts\';',
          '',
          callOf('formatFor',),
        ],),
      },
    ],
    found: ['imaginary/src/renaming-barrel-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER THROUGH A MODULE THAT RE-EXPORTS EVERYTHING, as in `export * from`, even through two '
      + 'of them',
    files: [
      {
        file: 'imaginary/src/star-barrel.ts',
        text: textOf([`export * from '${CONTEST_SPECIFIER}';`,],),
      },
      {
        file: 'imaginary/src/outer-star-barrel.ts',
        text: textOf(['export * from \'./star-barrel.ts\';',],),
      },
      {
        file: 'imaginary/src/star-barrel-gate-stage.ts',
        text: textOf([
          'import { contestResponseFormat, } from \'./outer-star-barrel.ts\';',
          '',
          callOf('contestResponseFormat',),
        ],),
      },
    ],
    found: ['imaginary/src/star-barrel-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER THROUGH A MODULE THAT RE-EXPORTS A MODULE AS A NAMESPACE, as in `export * as contest '
      + 'from`',
    files: [
      {
        file: 'imaginary/src/namespace-barrel.ts',
        text: textOf([`export * as contest from '${CONTEST_SPECIFIER}';`,],),
      },
      {
        file: 'imaginary/src/namespace-barrel-gate-stage.ts',
        text: textOf([
          'import { contest, } from \'./namespace-barrel.ts\';',
          '',
          callOf('contest.contestResponseFormat',),
        ],),
      },
    ],
    found: ['imaginary/src/namespace-barrel-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER THROUGH A LIST THAT EXPORTS AN IMPORT UNDER ANOTHER NAME, and the file that imports '
      + 'it from there',
    files: [
      {
        file: 'imaginary/src/list-barrel.ts',
        text: textOf([
          `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`,
          '',
          'export { contestResponseFormat as listedFormat, };',
        ],),
      },
      {
        file: 'imaginary/src/list-barrel-gate-stage.ts',
        text: textOf([
          'import { listedFormat, } from \'./list-barrel.ts\';',
          '',
          callOf('listedFormat',),
        ],),
      },
    ],
    found: ['imaginary/src/list-barrel-gate-stage.ts',],
    builders: [],
  },
];

/**
 Cases in which a file reaches a builder declared as the default export of its module.
 */
export const DEFAULT_EXPORT_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A DEFAULT EXPORT THROUGH A MODULE THAT RE-EXPORTS IT, as in `export { default } from`, '
      + 'whatever the importer calls it',
    files: [
      {
        file: 'imaginary/src/default-source-wire.ts',
        text: textOf([
          'export default function defaultSourceResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...formatLinesOf({ name: 'default_source_mood', },),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/default-barrel.ts',
        text: textOf(['export { default, } from \'./default-source-wire.ts\';',],),
      },
      {
        file: 'imaginary/src/default-renaming-barrel.ts',
        text: textOf(['export { default as renamedDefault, } from \'./default-source-wire.ts\';',],),
      },
      {
        file: 'imaginary/src/default-barrel-gate-stage.ts',
        text: textOf([
          'import fromBarrel from \'./default-barrel.ts\';',
          'import { renamedDefault, } from \'./default-renaming-barrel.ts\';',
          '',
          'export const FROM_BARREL_RESPONSE_FORMAT = fromBarrel([\'wags\',],);',
          'export const RENAMED_RESPONSE_FORMAT = renamedDefault([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/default-barrel-gate-stage.ts',
      'imaginary/src/default-source-wire.ts',
    ],
    builders: ['defaultSourceResponseFormat',],
  },
  {
    name: 'FOLLOWS A DEFAULT EXPORT WRITTEN AS `export default name;` AFTER THE FUNCTION IS DECLARED, and the '
      + 'file that imports it under a name of its own',
    files: [
      {
        file: 'imaginary/src/listed-default-wire.ts',
        text: textOf([
          'function listedDefaultResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...formatLinesOf({ name: 'listed_default_mood', },),
          '  };',
          '}',
          '',
          'export default listedDefaultResponseFormat;',
        ],),
      },
      {
        file: 'imaginary/src/listed-default-gate-stage.ts',
        text: textOf([
          'import makeListed from \'./listed-default-wire.ts\';',
          '',
          'export const LISTED_DEFAULT_RESPONSE_FORMAT = makeListed([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/listed-default-gate-stage.ts',
      'imaginary/src/listed-default-wire.ts',
    ],
    builders: ['listedDefaultResponseFormat',],
  },
];

//endregion Modules that re-export a builder
