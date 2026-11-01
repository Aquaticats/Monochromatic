/**
 Tests for the consistency pass's invariant error: a named class whose message
 says what broke, behind the word the coverage census looks for.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure';
import { exchangeFailureText, } from '@monochromatic-dev/translation-repair-provider-stream';
import { ConsistencyInvariantError, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: ConsistencyInvariantError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS WHAT BROKE, behind the word the coverage census and a reader look for',
          fn: async () => {
            const error = new ConsistencyInvariantError({ invariant: 'a group lists no place', },);
            expect({
              name: error.name,
              message: error.message,
              isError: Error.isError(error,),
            },).toEqual({
              name: 'ConsistencyInvariantError',
              message: 'unreachable: a group lists no place',
              isError: true,
            },);
          },
        },),
        it({
          name: 'RECORDS ITS OWN SENTENCE where a failure is recorded, since every site writes it from positions and '
            + 'vocabulary words and never from page text',
          fn: async () => {
            const error = new ConsistencyInvariantError({ invariant: 'a group lists no place', },);
            expect({
              marked: error.messageNamesOnly,
              stored: exchangeFailureText({ error, },),
              logged: refusalText({ error, },),
            },).toEqual({
              marked: true,
              stored: 'unreachable: a group lists no place',
              logged: 'unreachable: a group lists no place',
            },);
          },
        },),
      ],
    },),
  ],
},);
