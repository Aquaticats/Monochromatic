/**
 Tests for caught-value stack diagnostics.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { caughtValueStack, } from '@monochromatic-dev/caught-value-core';

await describe({
  name: caughtValueStack.name,
  children: [
    it({
      name: 'returns Error stack text when available',
      fn: async () => {
        /**
         Error carrying deterministic stack text.
         */
        const error = new Error('offline',);
        error.stack = 'stack details';
        expect(caughtValueStack(error,),).toBe('stack details',);
      },
    },),
    it({
      name: 'falls back to Error message when stack is absent',
      fn: async () => {
        /**
         Error with stack explicitly absent.
         */
        const error = new Error('offline',);
        delete error.stack;
        expect(caughtValueStack(error,),).toBe('offline',);
      },
    },),
    it({
      name: 'preserves caller-defined non-Error text',
      fn: async () => {
        /**
         Thrown object carrying diagnostic conversion text.
         */
        const thrownValue = {
          /**
           Supplies caller-defined diagnostic text.
           
           @returns caller-defined text.
           */
          toString(): string {
            return 'provider details';
          },
        };
        expect(caughtValueStack(thrownValue,),).toBe('provider details',);
      },
    },),
  ],
},);
