import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Names that are a use of a builder and names that are not
// What `claude-schemas.unit.test.ts` feeds the builder search to show which occurrences of a builder's name are a use:
// a call, a call through an optional chain and a tagged template are; a name that is only part of a longer name, a
// comment, a string, a property key and a `typeof` are not. Each case is a few imaginary files, with the contest
// builder (`contestResponseFormat`) beside them, and the files the search must list.

/**
 Cases of a name that is no use of a builder: part of a longer name, in a comment, in a string with a quote of
 another kind against it, a property key of the same name, and a `typeof`.
 */
export const NOT_A_USE_CASES: readonly FormCase[] = [
  {
    name: 'NAMES NOTHING IN A NAME THAT ONLY HOLDS A BUILDER\'S NAME, before it, after it, with a digit, `_` or '
      + '`$` against it',
    files: [
      {
        file: 'imaginary/src/longer-names.ts',
        text: textOf([
          'export const LONGER = contestResponseFormatMore({ schemaName: \'a\', },);',
          'export const REBUILT = rebuildContestResponseFormat({ schemaName: \'b\', },);',
          'export const NUMBERED = contestResponseFormat2({ schemaName: \'c\', },);',
          'export const UNDERSCORED = _contestResponseFormat({ schemaName: \'d\', },);',
          'export const DOLLARED = $contestResponseFormat({ schemaName: \'e\', },);',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'NAMES NOTHING IN A COMMENT, on a line of its own, in a block or in the documentation of a declaration',
    files: [
      {
        file: 'imaginary/src/comment-lines.ts',
        text: textOf([
          '// const format = contestResponseFormat({ schemaName: \'x\', verdicts: [], asksArchive: false, },);',
          '  // contestResponseFormat({ schemaName: \'y\', verdicts: [], asksArchive: false, },);',
          '/* contestResponseFormat({ schemaName: \'z\', verdicts: [], asksArchive: false, },); */',
          '/**',
          ' The format comes from contestResponseFormat(...).',
          ' */',
          'export const NOTHING = 6;',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'NAMES NOTHING IN A STRING WITH A QUOTE OF ANOTHER KIND AGAINST THE NAME, as in `"call \'name\' here"`',
    files: [
      {
        file: 'imaginary/src/quoted-neighbours.ts',
        text: textOf([
          'export const SINGLE_IN_DOUBLE = "call \'contestResponseFormat\' here";',
          'export const BACKTICK_IN_SINGLE = \'call `contestResponseFormat` here\';',
          'export const DOUBLE_IN_SINGLE = \'a"contestResponseFormat() b\';',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'NAMES NOTHING IN A PROPERTY KEY OF THE SAME NAME, written plainly or in quotes',
    files: [
      {
        file: 'imaginary/src/key-names.ts',
        text: textOf(['export const KEYS = { contestResponseFormat: 1, \'contestResponseFormat\': 2, };',],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'NAMES NOTHING IN A TYPEOF, whether it stands in a type or in an expression',
    files: [
      {
        file: 'imaginary/src/typeof-value.ts',
        text: textOf([
          'export type Maker = typeof contestResponseFormat;',
          'export const IS_FUNCTION = typeof contestResponseFormat === \'function\';',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
];

/**
 Cases of a call the search reads as a use although its syntax is not a plain call: a tagged template and a call
 through an optional chain.
 */
export const CALL_SHAPE_CASES: readonly FormCase[] = [
  {
    name: 'READS A TAGGED TEMPLATE AS A CALL, so a file that writes a builder\'s name before a template is a use',
    files: [
      {
        file: 'imaginary/src/tagged-gate-stage.ts',
        text: textOf(['export const TAGGED_GATE_RESPONSE_FORMAT = contestResponseFormat`tagged`;',],),
      },
    ],
    found: ['imaginary/src/tagged-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'READS AN OPTIONAL CALL OF A BUILDER AS A CALL, as in `contestResponseFormat?.(...)`',
    files: [
      {
        file: 'imaginary/src/optional-gate-stage.ts',
        text: textOf([
          'export const OPTIONAL_GATE_RESPONSE_FORMAT = contestResponseFormat?.({',
          '  schemaName: \'optional_gate\',',
          '  verdicts: [],',
          '  asksArchive: false,',
          '},);',
        ],),
      },
    ],
    found: ['imaginary/src/optional-gate-stage.ts',],
    builders: [],
  },
];

//endregion Names that are a use of a builder and names that are not
