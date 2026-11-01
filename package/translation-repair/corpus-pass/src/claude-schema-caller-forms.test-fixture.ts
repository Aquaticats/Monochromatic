import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Which files are the callers of a builder
// What `claude-schemas.unit.test.ts` feeds the caller search: a builder, imaginary files that use it, and the callers
// the search must list for it. The file that declares a builder and the producer list use it without being its callers.

/**
 One builder, the files searched and the paths the search lists as the callers of the builder.
 */
type CallerCase = Pick<FormCase, 'files' | 'name'> & {
  /**
   Name of the builder whose callers are listed.
   */
  readonly builder: string;

  /**
   Paths of the callers, sorted.
   */
  readonly callers: readonly string[];
};

/**
 The cases of callers.
 */
export const CALLER_CASES: readonly CallerCase[] = [
  {
    name: 'LISTS AS A CALLER NEITHER THE FILE THAT DECLARES THE BUILDER NOR THE PRODUCER LIST, though both use it',
    files: [
      {
        file: 'imaginary/src/own-wire.ts',
        text: textOf([
          'export function ownResponseFormat(schemaName: string,) {',
          '  return { type: \'json_schema\', schema: { title: schemaName, }, };',
          '}',
          '',
          'export const OWN_SAMPLE = ownResponseFormat(\'own\',);',
        ],),
      },
      {
        file: 'imaginary/src/own-caller.ts',
        text: textOf([
          'import { ownResponseFormat, } from \'./own-wire.ts\';',
          '',
          'export const OWN = ownResponseFormat(\'own_caller\',);',
        ],),
      },
      {
        file: 'corpus-pass/src/claude-schema-producers.ts',
        text: textOf([
          'import { ownResponseFormat, } from \'../../imaginary/src/own-wire.ts\';',
          '',
          'export const SAMPLE = ownResponseFormat(\'listed\',);',
        ],),
      },
    ],
    builder: 'ownResponseFormat',
    callers: ['imaginary/src/own-caller.ts',],
  },
];

//endregion Which files are the callers of a builder
