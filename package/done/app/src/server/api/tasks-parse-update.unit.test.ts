/**
 Tests for the task update body parser.

 The parser stands between an untrusted HTTP body and the merge that writes a
 task, so a body it does not understand must reach the caller as {@link INVALID}
 rather than as an update with no fields, which would report success while
 changing nothing.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  INVALID,
  parseTaskUpdateInput,
} from '../../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: parseTaskUpdateInput.name,
      children: [
        //region Record bodies are parsed field by field

        it({
          name: 'parses a record body carrying a title',
          fn: async () => {
            expect(parseTaskUpdateInput({ title: 'ship it', },),).toEqual({ title: 'ship it', },);
          },
        },),

        it({
          name: 'reads a record body with no fields as an update that changes nothing',
          fn: async () => {
            expect(parseTaskUpdateInput({},),).toEqual({},);
          },
        },),

        //endregion Record bodies are parsed field by field

        //region Array bodies are rejected, not read as empty updates

        it({
          name: 'rejects an array body instead of reading it as an update with no fields',
          fn: async () => {
            expect(parseTaskUpdateInput([],),).toBe(INVALID,);
          },
        },),

        it({
          name: 'rejects an array body whose elements look like field values',
          fn: async () => {
            expect(parseTaskUpdateInput(['ship it',],),).toBe(INVALID,);
          },
        },),

        //endregion Array bodies are rejected, not read as empty updates
      ],
    },),
  ],
},);
