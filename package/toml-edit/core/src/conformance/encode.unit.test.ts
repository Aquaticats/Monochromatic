/**
 Encoder-adapter boundary tests for malformed tagged scalars.

 The toml-test encoder builds `tomlSet` values from untrusted stdin JSON, so a
 tagged date or time whose payload is not a TOML lexeme must reject the whole
 input instead of writing that text into the document.

 @module
 */

import { spawnSync, } from 'node:child_process';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 The encoder adapter entry point, run exactly as the toml-test runner runs it.
 */
const ENCODER = fileURLToPath(new URL('encode.ts', import.meta.url,),);

/**
 Run the encoder adapter over `input` on stdin.
 
 @param input - Tagged JSON text the adapter reads.
 
 @returns Exit status, stdout, and stderr of the adapter process.
 
 @example
 ```ts
 runEncoder({ input: '{"a":{"type":"bool","value":"true"}}', }); // { status: 0, stdout: 'a = true\n', stderr: '' }
 ```
 */
function runEncoder({ input, }: { readonly input: string; },): {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
} {
  const result = spawnSync(process.execPath, [ENCODER,], {
    input,
    encoding: 'utf8',
  },);
  return {
    status: result.status ?? (-1),
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

await describe({
  name: 'conformance encoder wrapper rejection',
  children: [
    it({
      name: 'emits a well-formed tagged local date',
      fn: async () => {
        const result = runEncoder({ input: '{"d":{"type":"date-local","value":"2026-01-01"}}', },);
        expect(result.status,)
          .toBe(0,);
        expect(result.stdout,)
          .toBe('d = 2026-01-01\n',);
      },
    },),
    it({
      name: 'rejects a tagged local date carrying a comment',
      fn: async () => {
        const result = runEncoder({ input: '{"d":{"type":"date-local","value":"2026-01-01 # x"}}', },);
        expect(result.status,)
          .toBe(1,);
        expect(result.stdout,)
          .toBe('',);
        expect(result.stderr,)
          .toContain('local-date',);
        expect(result.stderr,)
          .toContain('2026-01-01 # x',);
      },
    },),
    it({
      name: 'rejects a tagged local date-time that carries an offset',
      fn: async () => {
        const result = runEncoder({
          input: '{"d":{"type":"datetime-local","value":"2026-05-14T10:00:00Z"}}',
        },);
        expect(result.status,)
          .toBe(1,);
        expect(result.stdout,)
          .toBe('',);
        expect(result.stderr,)
          .toContain('local-date-time',);
      },
    },),
    it({
      name: 'rejects a tagged local time that carries a newline',
      fn: async () => {
        const result = runEncoder({
          input: String.raw`{"d":{"type":"time-local","value":"10:00:00\nbad = 1"}}`,
        },);
        expect(result.status,)
          .toBe(1,);
        expect(result.stdout,)
          .toBe('',);
        expect(result.stderr,)
          .toContain('local-time',);
      },
    },),
    it({
      name: 'encodes a table whose only key is named tomlKind',
      fn: async () => {
        const result = runEncoder({ input: '{"t":{"tomlKind":{"type":"string","value":"integer"}}}', },);
        expect(result.status,)
          .toBe(0,);
        expect(result.stdout,)
          .toBe('t = { tomlKind = "integer", }\n',);
      },
    },),
  ],
},);
