/**
 Tests for naming a caught value.

 The cases that matter are the ones a `catch` binding actually sees, which is
 anything at all: this exists because asserting a caught value to `Error` is a
 claim nobody checked, and the throw that breaks that assumption is the throw
 whose report someone is reading.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  errorName,
  failureName,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: errorName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES a plain Error by its class',
          fn: async () => {
            expect(errorName({ error: new Error('a cat sat on the keyboard',), },),).toBe('Error',);
          },
        },),
        it({
          name: 'NAMES a subclass by ITS name rather than by Error, which is the whole point of asking',
          fn: async () => {
            /**
             Error class standing in for the pipeline's named ones.
             */
            class TabbyMissingError extends Error {
              /**
               Names itself the way the pipeline's error classes do.

               @example
               ```ts
               throw new TabbyMissingError();
               ```
               */
              constructor() {
                super('no tabby',);
                this.name = 'TabbyMissingError';
              }
            }

            expect(errorName({ error: new TabbyMissingError(), },),).toBe('TabbyMissingError',);
          },
        },),
        it({
          name: 'REFUSES to invent a name for a thrown string, which is legal JavaScript and carries none',
          fn: async () => {
            expect(errorName({ error: 'mittens', },),).toBe('a thrown value that is not an Error',);
          },
        },),
        it({
          name:
            'REFUSES to invent one for null, the case a `error.name` read would crash on rather than '
              + 'merely mis-report',
          fn: async () => {
            expect(errorName({ error: null, },),).toBe('a thrown value that is not an Error',);
          },
        },),
        it({
          name: 'REFUSES to invent one for an object that merely LOOKS like an error by having a name',
          fn: async () => {
            expect(errorName({ error: { name: 'NotReallyAnError', }, },),)
              .toBe('a thrown value that is not an Error',);
          },
        },),
      ],
    },),

    describe({
      name: failureName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES a filesystem code, which tells a reader what to do where a class name does not',
          fn: async () => {
            expect(failureName({
              error: Object.assign(
                new Error('ENOENT: no such file or directory, open /runs/whiskerfield/ledger',),
                { code: 'ENOENT', },
              ),
            },),).toBe('ENOENT',);
          },
        },),
        it({
          name: 'REFUSES to report the message, which quotes a path that can name a person',
          fn: async () => {
            expect(failureName({
              error: Object.assign(
                new Error('EACCES: permission denied, open /runs/Bixbyfluff/ledger/000001.json',),
                { code: 'EACCES', },
              ),
            },).includes('Bixbyfluff',),).toBe(false,);
          },
        },),
        it({
          name: 'FALLS BACK to the class where there is no code, since most errors carry none',
          fn: async () => {
            expect(failureName({ error: new RangeError('out of range',), },),).toBe('RangeError',);
          },
        },),
        it({
          name: 'FALLS BACK where a code is present but is not a string, which no reader could print',
          fn: async () => {
            expect(failureName({
              error: Object.assign(
                new RangeError('out of range',),
                { code: 13, },
              ),
            },),).toBe('RangeError',);
          },
        },),
        it({
          name: 'NAMES a thrown value that is not an Error, which carries neither code nor class',
          fn: async () => {
            expect(failureName({ error: 'mittens', },),).toBe('a thrown value that is not an Error',);
          },
        },),
      ],
    },),
  ],
},);
