/**
 Tests for the package entry point's export surface.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this package wraps node-style callback functions, so its fixtures and wrapped-function declarations implement the callback pattern the tests exercise; see DECISION.callback-capture.md. */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidInputError,
  pify,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    it({
      name: 'exports the pify factory and the input error class',
      fn: async () => {
        expect(typeof pify,).toBe('function',);
        expect(typeof InvalidInputError,).toBe('function',);
      },
    },),

    it({
      name: 'promisifies a function through the package entry point',
      fn: async () => {
        const promisified = pify({
          input: function greet(name: string, callback: (error: unknown, value: unknown) => void): void {
            callback(null, `hello ${name}`,);
          },
        },);
        expect(await promisified({
          args: ['entry',],
        },),).toBe('hello entry',);
      },
    },),

    it({
      name: 'promisifies a module through the package entry point',
      fn: async () => {
        const pified = pify({
          input: {
            greet(name: string, callback: (error: unknown, value: unknown) => void): void {
              callback(null, `hello ${name}`,);
            },
          },
        },);
        expect(await pified.greet({
          args: ['entry',],
        },),).toBe('hello entry',);
      },
    },),
  ],
},);
