import { PassThrough, } from 'node:stream';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  createSquareCheckboxTheme,
  isPromptCancellation,
} from '../dist/final/node/index.mjs';

await describe({
  name: createSquareCheckboxTheme.name,
  children: [
    it({
      name: 'uses square checkbox indicators in ordinary and security themes',
      fn: async () => {
        /**
         Non-TTY output makes native color policy return plain glyph text.
         */
        const output = new PassThrough();
        /**
         Ordinary square theme.
         */
        const ordinary = createSquareCheckboxTheme({
          security: false,
          output,
        },);
        /**
         Security square theme.
         */
        const security = createSquareCheckboxTheme({
          security: true,
          output,
        },);

        expect(ordinary.icon.checked,).toBe('☑',);
        expect(ordinary.icon.unchecked,).toBe('☐',);
        expect(security.icon.checked,).toBe('☑',);
        expect(security.icon.unchecked,).toBe('☐',);
        expect(security.style.message('SECURITY',),).toBe('SECURITY',);
      },
    },),
    it({
      name: 'recognizes only documented Inquirer cancellation error',
      fn: async () => {
        /**
         Error carrying documented Inquirer cancellation name.
         */
        const cancellation = new Error('User force closed the prompt');
        cancellation.name = 'ExitPromptError';
        expect(isPromptCancellation(cancellation,),).toBe(true,);
        expect(
          isPromptCancellation(new Error('other',),),
        ).toBe(false,);
        expect(isPromptCancellation('ExitPromptError',),).toBe(false,);
      },
    },),
  ],
},);
