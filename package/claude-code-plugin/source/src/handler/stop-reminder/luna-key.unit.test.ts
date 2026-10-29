import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  miseEnvironment,
  readLunaKey,
} from './luna-key.ts';
import {
  LUNA_KEY_ENV,
  LunaKeyMissingError,
} from './luna-situation.ts';

/**
 Runs an async operation and returns the error it threw.

 @param run - operation expected to throw

 @returns the thrown value

 @example
 ```ts
 const error = await caught(async () => { throw new Error('x'); });
 ```
 */
async function caught(run: () => Promise<unknown>,): Promise<unknown> {
  try {
    await run();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  children: [
    describe({
      name: readLunaKey.name,
      children: [
        it({
          name: 'returns the key from the environment without asking mise',
          fn: async () => {
            let asked = 0;

            const key = await readLunaKey({
              env: { [LUNA_KEY_ENV]: 'env-key', },
              cwd: '/anywhere',
              readMiseEnvironment: async () => {
                asked += 1;
                return {};
              },
            },);

            expect(key,).toBe('env-key',);
            expect(asked,).toBe(0,);
          },
        },),
        it({
          name: 'reads mise exactly once when the environment lacks the key',
          fn: async () => {
            const seen: string[] = [];

            const key = await readLunaKey({
              env: {},
              cwd: '/project',
              readMiseEnvironment: async (cwd,) => {
                seen.push(cwd,);
                return { [LUNA_KEY_ENV]: 'mise-key', };
              },
            },);

            expect(key,).toBe('mise-key',);
            expect(seen,).toEqual(['/project',],);
          },
        },),
        it({
          name: 'treats an empty environment value as absent',
          fn: async () => {
            const key = await readLunaKey({
              env: { [LUNA_KEY_ENV]: '', },
              cwd: '/project',
              readMiseEnvironment: async () => ({ [LUNA_KEY_ENV]: 'mise-key', }),
            },);

            expect(key,).toBe('mise-key',);
          },
        },),
        it({
          name: 'throws when mise supplies no key',
          fn: async () => {
            const error = await caught(async () => readLunaKey({
              env: {},
              cwd: '/project',
              readMiseEnvironment: async () => ({}),
            },),);

            expect(error,).toBeInstanceOf(LunaKeyMissingError,);
          },
        },),
        it({
          name: 'throws when mise supplies a non-string or empty key',
          fn: async () => {
            const errors = await Promise.all([7, '', null,].map(async function failsFor(value,) {
              return await caught(async () => readLunaKey({
                env: {},
                cwd: '/project',
                readMiseEnvironment: async () => ({ [LUNA_KEY_ENV]: value, }),
              },),);
            },),);

            expect(errors.every((error,) => error instanceof LunaKeyMissingError),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: miseEnvironment.name,
      children: [
        it({
          name: 'returns the environment mise prints for a directory with no project key',
          fn: async () => {
            const scratch = await mkdtemp(join(scratchRoot(), 'stop-reminder-mise-',),);

            const environment = await miseEnvironment(scratch,);

            expect(environment[LUNA_KEY_ENV],).toBeUndefined();
            expect(Object.keys(environment,).length,).toBeGreaterThan(0,);
          },
          timeout: 10_000,
        },),
        it({
          name: 'throws a missing-key error when the directory does not exist',
          fn: async () => {
            const error = await caught(async () => miseEnvironment(join(scratchRoot(), 'stop-reminder-no-such-dir', 'nested',),),);

            expect(error,).toBeInstanceOf(LunaKeyMissingError,);
          },
          timeout: 10_000,
        },),
      ],
    },),
  ],
},);
