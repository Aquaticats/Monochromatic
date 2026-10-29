import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { readBypassStatePath, } from '../dist/final/node/tunnel-bypass.mjs';

/**
 Valid persisted state shared by shape mutations.
 */
const VALID_STATE = {
  version: 2,
  interfaceName: 'wgtest',
  mark: 100,
  table: 52_000,
  preference: 50,
  ownerId: 'owner',
  routes: [{
    proto: '-4',
    tokens: [
      'default',
      'dev',
      'eth0',
      'table',
      '52000',
      'proto',
      '201',
    ],
  },],
} as const;

/**
 Invalid persisted-state fixture.
 */
type InvalidStateCase = {
  readonly name: string;
  readonly value: unknown;
};

/**
 Invalid field shapes rejected before route mutation.
 */
const INVALID_STATE_CASES: readonly InvalidStateCase[] = [
  {
    name: 'rejects non-object state',
    value: [],
  },
  {
    name: 'rejects unsupported version',
    value: { ...VALID_STATE, version: 1, },
  },
  {
    name: 'rejects empty interface name',
    value: { ...VALID_STATE, interfaceName: '', },
  },
  {
    name: 'rejects slash in interface name',
    value: { ...VALID_STATE, interfaceName: 'wg/test', },
  },
  {
    name: 'rejects overlong interface name',
    value: { ...VALID_STATE, interfaceName: 'wireguard-test-01', },
  },
  {
    name: 'rejects zero mark',
    value: { ...VALID_STATE, mark: 0, },
  },
  {
    name: 'rejects negative mark',
    value: { ...VALID_STATE, mark: -1, },
  },
  {
    name: 'rejects fractional mark',
    value: { ...VALID_STATE, mark: 1 / 2, },
  },
  {
    name: 'rejects mark beyond unsigned integer range',
    value: { ...VALID_STATE, mark: 0x1_00_00_00_00, },
  },
  {
    name: 'rejects main table identity',
    value: { ...VALID_STATE, table: 0, },
  },
  {
    name: 'rejects fractional table',
    value: { ...VALID_STATE, table: 52_000 + (1 / 2), },
  },
  {
    name: 'rejects reserved zero preference',
    value: { ...VALID_STATE, preference: 0, },
  },
  {
    name: 'rejects fractional preference',
    value: { ...VALID_STATE, preference: 50 + (1 / 2), },
  },
  {
    name: 'rejects empty owner identity',
    value: { ...VALID_STATE, ownerId: '', },
  },
  {
    name: 'rejects absent route fingerprints',
    value: {
      version: VALID_STATE.version,
      interfaceName: VALID_STATE.interfaceName,
      mark: VALID_STATE.mark,
      table: VALID_STATE.table,
      preference: VALID_STATE.preference,
      ownerId: VALID_STATE.ownerId,
    },
  },
  {
    name: 'rejects invalid route family',
    value: { ...VALID_STATE, routes: [{ proto: '-5', tokens: ['default',], },], },
  },
  {
    name: 'rejects empty route tokens',
    value: { ...VALID_STATE, routes: [{ proto: '-4', tokens: [], },], },
  },
  {
    name: 'rejects non-string route token',
    value: { ...VALID_STATE, routes: [{ proto: '-6', tokens: ['default', 1,], },], },
  },
];

/**
 Disposable state fixture directory.
 */
type TempDir = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates disposable state fixture directory.
 
 @returns Directory removed recursively on disposal.
 
 @example
 ```ts
 await using directory = await makeTempDir();
 ```
 */
async function makeTempDir(): Promise<TempDir> {
  /**
   Fresh operating-system temporary path.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    'wg-quicker-state-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Captures asynchronous failure.
 
 @param operation - Operation expected to reject.
 
 @returns Rejection value.
 
 @example
 ```ts
 await captureRejected({ operation: async () => { throw new Error('fixture'); } });
 ```
 */
async function captureRejected(
  { operation, }: { readonly operation: () => Promise<unknown>; },
): Promise<unknown> {
  try {
    await operation();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('Expected operation to reject.',);
}

await describe({
  name: readBypassStatePath.name,
  children: [
    it({
      name: 'reads complete bounded state',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Valid state path.
         */
        const path = join(directory.path, 'valid.json',);
        await writeFile(
          path,
          JSON.stringify(VALID_STATE,),
        );
        await expect(readBypassStatePath({ path, },),).resolves.toEqual(VALID_STATE,);
      },
    },),
    it({
      name: 'rejects absent state path',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Missing path carrying diagnostic identity.
         */
        const path = join(directory.path, 'absent.json',);
        /**
         Missing-state rejection.
         */
        const error = await captureRejected({
          operation: async function readAbsentState(): Promise<unknown> {
            return await readBypassStatePath({ path, },);
          },
        },);
        expect(String(error,),).toContain('BypassStateError',);
        expect(String(error,),).toContain(path,);
      },
    },),
    it({
      name: 'wraps malformed JSON as bypass state error',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Malformed state path.
         */
        const path = join(directory.path, 'malformed.json',);
        await writeFile(
          path,
          '{',
        );
        /**
         Parse rejection carrying state error identity.
         */
        const error = await captureRejected({
          operation: async function readMalformedState(): Promise<unknown> {
            return await readBypassStatePath({ path, },);
          },
        },);
        expect(String(error,),).toContain('BypassStateError',);
        expect(String(error,),).toContain(path,);
      },
    },),
    ...INVALID_STATE_CASES.map(function invalidStateTest(fixture,): ReturnType<typeof it> {
      return it({
        name: fixture.name,
        fn: async () => {
          await using directory = await makeTempDir();
          /**
           Invalid shape state path.
           */
          const path = join(directory.path, 'invalid.json',);
          await writeFile(
            path,
            JSON.stringify(fixture.value,),
          );
          /**
           Shape validation rejection.
           */
          const error = await captureRejected({
            operation: async function readInvalidState(): Promise<unknown> {
              return await readBypassStatePath({ path, },);
            },
          },);
          expect(String(error,),).toContain('BypassStateError',);
          expect(String(error,),).toContain(path,);
        },
      },);
    },),
  ],
},);
