import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ParseError, } from 'toml-eslint-parser';

import {
  parseTomlEdit,
  tomlGetValue,
  tomlStringify,
  TomlEditError,
} from '@monochromatic-dev/toml-edit-core';

/**
 Capture expected parse failures without treating unexpected exceptions as valid.

 @param source - Invalid TOML fixture chosen for a distinct failure path.

 @returns Package error carrying the parser cause when one exists.

 @throws Error when parsing succeeds unexpectedly.

 @example
 ```ts
 parseFailure('key = ');
 ```
 */
function parseFailure(source: string,): TomlEditError {
  try {
    parseTomlEdit({ source, },);
  }
  catch (error) {
    if (error instanceof TomlEditError)
      return error;
    throw error;
  }
  throw new Error('Expected a TOML parse failure',);
}

await describe({
  name: parseTomlEdit.name,
  children: [
    it({
      name: 'rejects bare carriage returns, including a trailing one',
      fn: async () => {
        for (const source of ['a = 1\rb = 2\n', 'a = 1\r', 'a = """x\ry"""\n']) {
          const error = parseFailure(source,);
          expect(error.message,).toBe('Failed to parse TOML: a bare carriage return is not allowed; CR must be part of CRLF',);
          expect(error.cause,).toBe(undefined,);
        }
      },
    },),
    it({
      name: 'normalizes CRLF before storing and emitting the document',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a = 1\r\nb = 2\r\n', },);
        expect(edit.source,).toBe('a = 1\nb = 2\n',);
        expect(tomlStringify({ edit, },),).toBe(edit.source,);
      },
    },),
    it({
      name: 'retains parser rejection as the cause of a package error',
      fn: async () => {
        const error = parseFailure('a = \n',);
        expect(error.message,).toBe('Failed to parse TOML: Unspecified values are invalid',);
        expect(error.cause,).toBeInstanceOf(ParseError,);
      },
    },),
    it({
      name: 'forwards the selected TOML version to the parser',
      fn: async () => {
        const source = 'key = {foo="value"\n  , bar="value"}\n';
        expect(() => parseTomlEdit({ source, tomlVersion: '1.0', },),)
          .toThrow(TomlEditError,);
        const edit = parseTomlEdit({ source, tomlVersion: '1.1', },);
        const defaultEdit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['key', 'bar',], },),).toBe('value',);
        expect(tomlGetValue({ edit: defaultEdit, path: ['key', 'bar',], },),).toBe('value',);
      },
    },),
  ],
},);
