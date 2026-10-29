import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  normalizeMessage,
  SEVERITY_MAP,
} from './nvim-client.ts';

//region SEVERITY_MAP: maps vim.diagnostic.severity codes to human-readable labels

await describe({
  name: '',
  children: [
    describe({
      name: 'SEVERITY_MAP',
      children: [
        it({
          name: 'maps 1 to ERROR',
          fn: async () => {
            expect(SEVERITY_MAP[1],).toBe('ERROR',);
          },
        },),
        it({
          name: 'maps 2 to WARN',
          fn: async () => {
            expect(SEVERITY_MAP[2],).toBe('WARN',);
          },
        },),
        it({
          name: 'maps 3 to INFO',
          fn: async () => {
            expect(SEVERITY_MAP[3],).toBe('INFO',);
          },
        },),
        it({
          name: 'maps 4 to HINT',
          fn: async () => {
            expect(SEVERITY_MAP[4],).toBe('HINT',);
          },
        },),
        it({
          name: 'returns undefined for unknown severity codes',
          fn: async () => {
            expect(SEVERITY_MAP[0],).toBeUndefined();
            expect(SEVERITY_MAP[5],).toBeUndefined();
          },
        },),
        it({
          name: 'contains exactly 4 entries',
          fn: async () => {
            expect(Object.keys(SEVERITY_MAP,),).toHaveLength(4,);
          },
        },),
      ],
    },),

    //endregion SEVERITY_MAP

    //region normalizeMessage: reformats embedded help text from LSP diagnostics

    describe({
      name: normalizeMessage.name,
      children: [
        it({
          name: 'reformats embedded help text inline',
          fn: async () => {
            expect(
              normalizeMessage(
                'Empty exports do nothing in module files\nhelp: Remove this empty export.',
              ),
            )
              .toBe(
                'Empty exports do nothing in module files (help: Remove this empty export.)',
              );
          },
        },),
        it({
          name: 'passes through message without help text unchanged',
          fn: async () => {
            expect(
              normalizeMessage("Type 'string' is not assignable to type 'number'.",),
            )
              .toBe("Type 'string' is not assignable to type 'number'.",);
          },
        },),
        it({
          name: 'handles message that is just help text prefix without content',
          fn: async () => {
            expect(normalizeMessage('Error\nhelp: ',),)
              .toBe('Error (help: )',);
          },
        },),
        it({
          name: 'only reformats the first help occurrence',
          fn: async () => {
            expect(normalizeMessage('Error\nhelp: Fix this\nhelp: Also this',),)
              .toBe('Error (help: Fix this\nhelp: Also this)',);
          },
        },),
        it({
          name: 'does not match help without preceding newline',
          fn: async () => {
            expect(normalizeMessage('See help: section for details',),)
              .toBe('See help: section for details',);
          },
        },),
      ],
    },),
    //endregion normalizeMessage
  ],
},);
