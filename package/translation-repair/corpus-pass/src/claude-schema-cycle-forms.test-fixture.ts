import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Names that lead back to themselves
// What `claude-schemas.unit.test.ts` feeds the builder search to show that following a name ends when the name leads
// back to itself: two files that export everything of each other, and two constants that are each other's value. In
// both the name stands for nothing, and the search ends and lists nothing. Each case is a few imaginary files, with
// the contest builder (`contestResponseFormat`) beside them, and the files the search must list.

/**
 Cases of a name followed through a loop.
 */
export const CYCLE_CASES: readonly FormCase[] = [
  {
    name: 'ENDS WHEN TWO FILES EXPORT EVERYTHING OF EACH OTHER, and names nothing for a name neither exports',
    files: [
      {
        file: 'imaginary/src/star-cycle-a.ts',
        text: textOf(['export * from \'./star-cycle-b.ts\';',],),
      },
      {
        file: 'imaginary/src/star-cycle-b.ts',
        text: textOf(['export * from \'./star-cycle-a.ts\';',],),
      },
      {
        file: 'imaginary/src/star-cycle-user.ts',
        text: textOf([
          'import { missing, } from \'./star-cycle-a.ts\';',
          '',
          'export const USED = missing();',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'ENDS WHEN TWO CONSTANTS ARE EACH OTHER\'S VALUE, as in `const a = b; const b = a;`, and names nothing',
    files: [
      {
        file: 'imaginary/src/alias-cycle.ts',
        text: textOf([
          'const first = second;',
          'const second = first;',
          '',
          'export const USED = first();',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
];

//endregion Names that lead back to themselves
