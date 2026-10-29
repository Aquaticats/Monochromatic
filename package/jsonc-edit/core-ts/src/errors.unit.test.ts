/**
 Unit tests for the error classes: each carries the right `name`, is an
 instance of its class, and exposes its structured fields (offset, path).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type StringJsonc,
  jsoncDelete,
  jsoncGetValue,
  JsoncParseError,
  JsoncPathNotFoundError,
  JsoncTypeError,
  parseJsonc,
  parseJsoncEdit,
} from '../dist/final/neutral/index.mjs';

const asJsonc = (source: string,): StringJsonc => source as StringJsonc;

const capture = (fn: () => void,): unknown => {
  try {
    fn();
  }
  catch (error: unknown) {
    return error;
  }
  return undefined;
};

await describe({
  name: 'errors',
  children: [
    it({
      name: 'JsoncParseError carries its name and the failure offset',
      fn: async () => {
        const error = capture(() => {
          parseJsonc({ source: asJsonc('{',), },);
        },);
        expect(error,).toBeInstanceOf(JsoncParseError,);
        expect(error,).toHaveProperty('name', 'JsoncParseError',);
        expect(error,).toHaveProperty('offset', 0,);
      },
    },),
    it({
      name: 'JsoncPathNotFoundError carries its name and the unresolved path',
      fn: async () => {
        const error = capture(() => {
          jsoncGetValue({ state: parseJsoncEdit({ source: asJsonc('{ "a": 1 }',), },), path: ['nope',], },);
        },);
        expect(error,).toBeInstanceOf(JsoncPathNotFoundError,);
        expect(error,).toHaveProperty('name', 'JsoncPathNotFoundError',);
        expect(error,).toHaveProperty('path[0]', 'nope',);
      },
    },),
    it({
      name: 'JsoncTypeError carries its name',
      fn: async () => {
        const error = capture(() => {
          jsoncDelete({ state: parseJsoncEdit({ source: asJsonc('{ "a": 1 }',), },), path: [], },);
        },);
        expect(error,).toBeInstanceOf(JsoncTypeError,);
        expect(error,).toHaveProperty('name', 'JsoncTypeError',);
      },
    },),
  ],
},);
