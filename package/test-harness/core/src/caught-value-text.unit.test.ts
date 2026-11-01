/**
 Tests for caught-value diagnostic text.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core';

await describe({
  name: caughtValueText.name,
  children: [
    it({
      name: 'returns Error message text',
      fn: async () => {
        /**
         Error carrying expected message.
         */
        const error = new Error('offline',);
        expect(caughtValueText(error,),).toBe('offline',);
      },
    },),
    it({
      name: 'preserves primitive thrown value text',
      fn: async () => {
        expect(caughtValueText('plain failure',),).toBe('plain failure',);
        expect(caughtValueText(404,),).toBe('404',);
      },
    },),
    it({
      name: 'preserves caller-defined object text',
      fn: async () => {
        /**
         Number of caller conversion hooks invoked.
         */
        let conversionCount = 0;
        /**
         Thrown object carrying diagnostic conversion text.
         */
        const thrownValue = {
          /**
           Supplies diagnostic text and records observable invocation.
           
           @returns caller-defined diagnostic text.
           */
          toString(): string {
            conversionCount++;
            return 'provider details';
          },
        };
        expect(caughtValueText(thrownValue,),).toBe('provider details',);
        expect(conversionCount,).toBe(1,);
      },
    },),
    it({
      name: 'reads caller-defined Error message getter',
      fn: async () => {
        /**
         Error whose message is supplied by a getter.
         */
        const error = new Error('replaced by getter',);
        /**
         Number of message getter invocations.
         */
        let messageReadCount = 0;
        Object.defineProperty(
          error,
          'message',
          {
            configurable: true,
            /**
             Supplies diagnostic message and records observable access.
             
             @returns getter-provided message.
             */
            get(): string {
              messageReadCount++;
              return 'getter details';
            },
          },
        );
        expect(caughtValueText(error,),).toBe('getter details',);
        expect(messageReadCount,).toBe(1,);
      },
    },),
  ],
},);
