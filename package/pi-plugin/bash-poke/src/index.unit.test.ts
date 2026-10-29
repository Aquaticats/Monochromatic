/**
 Tests for the built bash-poke extension entry point.

 @module
 */

import { mkdtemp, mkdir, rm, writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import bashPoke, { constants, } from '../dist/final/node/index.mjs';

import { createFakePi, } from './test-fixtures.ts';

//region Fixtures

/**
 Disposable home directory that owns the `HOME` override for one case.
 */
type DisposableHome = {
  /**
   Absolute home path the extension reads settings below.
   */
  readonly path: string;

  /**
   Restores the previous `HOME` and removes the directory.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable home and points `HOME` at it.
 
 The extension resolves settings through `homedir()`, which reads `HOME`, so a
 case that must control settings has to own that variable for its duration.
 
 @returns home restored and removed on disposal
 
 @example
 ```ts
 await using home = await createDisposableHome();
 ```
 */
async function createDisposableHome(): Promise<DisposableHome> {
  /**
   Absolute home directory unique to the current case.
   */
  const path = await mkdtemp(join(scratchRoot(), 'bash-poke-home-', ), );

  /**
   Value `HOME` held before this case replaced it.
   */
  const previous = process.env.HOME;
  process.env.HOME = path;
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      if (previous === undefined)
        delete process.env.HOME;
      else
        process.env.HOME = previous;
      await rm(path, { recursive: true, force: true, }, );
    },
  };
}

/**
 Writes a settings document into a disposable home.
 
 @param home - home directory holding the `.pi` tree
 
 @param text - raw settings document
 
 @example
 ```ts
 await writeSettingsFile({ home: '/tmp/home', text: '{}', },);
 ```
 */
async function writeSettingsFile(
  {
    home,
    text,
  }: {
    readonly home: string;
    readonly text: string;
  },
): Promise<void> {
  /**
   Directory the global Pi extension settings live in.
   */
  const dir = join(home, '.pi', 'agent', 'extensions', );
  await mkdir(dir, { recursive: true, }, );
  await writeFile(join(dir, 'pi-bash-poke.json', ), text, 'utf8', );
}

//endregion Fixtures

await describe({
  name: bashPoke.name,
  concurrency: 1,
  children: [
    it({
      name: 'registers the renderer and both event handlers with default settings',
      fn: async () => {
        await using home = await createDisposableHome();

        /**
         Fake Pi API recording what the extension registered.
         */
        const fake = createFakePi();
        await bashPoke(fake.api, );
        expect(home.path.length > 0, ).toBe(true);
        expect(fake.renderers.has(constants.POKE_CUSTOM_TYPE, ), ).toBe(true);
        expect([...fake.handlers.keys(), ].toSorted(), )
          .toEqual(['session_start', 'user_bash', ]);
      },
    }, ),
    it({
      name: 'honors a settings file in the home it is given',
      fn: async () => {
        await using home = await createDisposableHome();
        await writeSettingsFile({ home: home.path, text: '{"pokeInstruction":"resume"}', }, );

        /**
         Fake Pi API recording what the extension registered.
         */
        const fake = createFakePi();
        await bashPoke(fake.api, );
        expect(fake.handlers.has('user_bash', ), ).toBe(true);
      },
    }, ),
    it({
      name: 'fails the load rather than registering with an unusable setting',
      fn: async () => {
        await using home = await createDisposableHome();
        await writeSettingsFile({ home: home.path, text: '{"notAKey":1}', }, );

        /**
         Fake Pi API that must never be reached when settings are invalid.
         */
        const fake = createFakePi();
        let caught: unknown;
        try {
          await bashPoke(fake.api, );
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(String(caught, ), ).toContain('unknown settings');
        expect(fake.handlers.size, ).toBe(0);
      },
    }, ),
  ],
}, );
