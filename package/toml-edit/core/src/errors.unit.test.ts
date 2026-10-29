import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  TomlEditError,
  TomlImmutableNodeError,
  TomlPathNotFoundError,
  TomlSpliceUnavailableError,
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'public TOML error classes',
  children: [
    it({
      name: 'retain their names and shared base type',
      fn: async () => {
        const cases = [
          [new TomlEditError('base',), 'TomlEditError',],
          [new TomlPathNotFoundError('missing',), 'TomlPathNotFoundError',],
          [new TomlSpliceUnavailableError('mode',), 'TomlSpliceUnavailableError',],
          [new TomlTypeError('value',), 'TomlTypeError',],
          [new TomlImmutableNodeError('node',), 'TomlImmutableNodeError',],
        ] as const;
        for (const [error, name,] of cases) {
          expect(error.name,).toBe(name,);
          expect(error,).toBeInstanceOf(TomlEditError,);
        }
      },
    },),
    it({
      name: 'retains the supplied cause',
      fn: async () => {
        const cause = new Error('origin',);
        const error = new TomlEditError('failed', { cause, },);
        expect(error.cause,).toBe(cause,);
        expect(error.message,).toBe('failed',);
      },
    },),
  ],
},);
