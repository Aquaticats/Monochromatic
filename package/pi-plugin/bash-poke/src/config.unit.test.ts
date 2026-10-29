/**
 Tests for settings loading in the built bash-poke artifact.

 @module
 */

import { mkdir, mkdtemp, readFile, rm, writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { dirname, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BashPokeConfigError,
  configPathForHome,
  DEFAULT_SETTINGS,
  loadSettings,
  parseSettingsJson,
  readSettingsFile,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Disposable home directory used by settings tests.
 */
type TempHome = {
  /**
   Absolute home path the `.pi` tree is built below.
   */
  readonly path: string;

  /**
   Removal hook run when async disposal completes.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable home directory.
 
 @returns home removed when async disposal completes
 
 @example
 ```ts
 await using home = await createTempHome();
 ```
 */
async function createTempHome(): Promise<TempHome> {
  /**
   Absolute home root unique to the current test.
   */
  const path = await mkdtemp(join(scratchRoot(), 'bash-poke-settings-', ), );
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { recursive: true, force: true, }, );
    },
  };
}

/**
 Writes a settings document into a disposable home directory.
 
 @param home - home directory holding the `.pi` tree
 
 @param text - raw document text
 
 @returns absolute settings path
 
 @example
 ```ts
 await writeSettings({ home: '/tmp/home', text: '{}', },);
 ```
 */
async function writeSettings(
  {
    home,
    text,
  }: {
    readonly home: string;
    readonly text: string;
  },
): Promise<string> {
  /**
   Absolute settings path inside that home.
   */
  const path = configPathForHome({ home, }, );
  await mkdir(dirname(path, ), { recursive: true, }, );
  await writeFile(path, text, 'utf8', );
  return path;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region configPathForHome

    describe({
      name: configPathForHome.name,
      children: [
        it({
          name: 'points below the global Pi extension directory',
          fn: async () => {
            expect(configPathForHome({ home: '/home/user', }, ), )
              .toBe(join('/home/user', '.pi', 'agent', 'extensions', 'pi-bash-poke.json', ), );
          },
        }, ),
      ],
    }, ),

    //endregion configPathForHome

    //region readSettingsFile

    describe({
      name: readSettingsFile.name,
      children: [
        it({
          name: 'reports absence without throwing',
          fn: async () => {
            await using home = await createTempHome();
            const read = await readSettingsFile({
              path: configPathForHome({ home: home.path, }, ),
            }, );
            expect(read.state, ).toBe('absent');
          },
        }, ),
        it({
          name: 'returns the text of a present file',
          fn: async () => {
            await using home = await createTempHome();
            const path = await writeSettings({ home: home.path, text: '{"killGraceMs":5}', }, );
            const read = await readSettingsFile({ path, }, );
            expect(read.state, ).toBe('present');
            if (read.state !== 'present')
              throw new Error('expected a present read state');
            expect(read.text, ).toBe('{"killGraceMs":5}');
          },
        }, ),
        it({
          name: 'reports an unreadable path as a fault rather than absence',
          fn: async () => {
            await using home = await createTempHome();
            // A directory at the settings path cannot be read as a file, which
            // is a different situation from a file nobody created.
            const path = configPathForHome({ home: home.path, }, );
            await mkdir(path, { recursive: true, }, );
            const read = await readSettingsFile({ path, }, );
            expect(read.state, ).toBe('unreadable');
            if (read.state !== 'unreadable')
              throw new Error('expected an unreadable read state');
            expect(read.failure.length > 0, ).toBe(true);
          },
        }, ),
      ],
    }, ),

    //endregion readSettingsFile

    //region parseSettingsJson

    describe({
      name: parseSettingsJson.name,
      children: [
        it({
          name: 'decodes valid JSON',
          fn: async () => {
            const parsed = parseSettingsJson({ text: '{"a":1}', }, );
            expect(parsed.state, ).toBe('parsed');
            if (parsed.state !== 'parsed')
              throw new Error('expected a parsed state');
            expect(parsed.value, ).toEqual({ a: 1, });
          },
        }, ),
        it({
          name: 'reports invalid JSON without throwing',
          fn: async () => {
            const parsed = parseSettingsJson({ text: '{', }, );
            expect(parsed.state, ).toBe('invalid');
            if (parsed.state !== 'invalid')
              throw new Error('expected an invalid state');
            expect(parsed.failure.length > 0, ).toBe(true);
          },
        }, ),
      ],
    }, ),

    //endregion parseSettingsJson

    //region loadSettings

    describe({
      name: loadSettings.name,
      children: [
        it({
          name: 'returns defaults when no settings file exists',
          fn: async () => {
            await using home = await createTempHome();
            expect(await loadSettings({ home: home.path, }, ), ).toEqual(DEFAULT_SETTINGS);
          },
        }, ),
        it({
          name: 'merges a partial file over the defaults',
          fn: async () => {
            await using home = await createTempHome();
            await writeSettings({ home: home.path, text: '{"pokeTailChars":400}', }, );
            const settings = await loadSettings({ home: home.path, }, );
            expect(settings.pokeTailChars, ).toBe(400);
            expect(settings.pokeHeadChars, ).toBe(DEFAULT_SETTINGS.pokeHeadChars);
            expect(settings.pokeInstruction, ).toBe(DEFAULT_SETTINGS.pokeInstruction);
          },
        }, ),
        it({
          name: 'throws on invalid JSON and names the file',
          fn: async () => {
            await using home = await createTempHome();
            const path = await writeSettings({ home: home.path, text: 'not json', }, );
            let caught: unknown;
            try {
              await loadSettings({ home: home.path, }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught, ).toBeInstanceOf(BashPokeConfigError);
            expect((caught as Error).message, ).toContain(path);
            expect((caught as Error).message, ).toContain('invalid JSON');
          },
        }, ),
        it({
          name: 'throws on an unknown key',
          fn: async () => {
            await using home = await createTempHome();
            await writeSettings({ home: home.path, text: '{"retryOnError":true}', }, );
            let caught: unknown;
            try {
              await loadSettings({ home: home.path, }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect((caught as Error).message, ).toContain('unknown settings');
            expect((caught as Error).message, ).toContain('retryOnError');
          },
        }, ),
        it({
          name: 'throws when the path cannot be read as a file',
          fn: async () => {
            await using home = await createTempHome();
            await mkdir(configPathForHome({ home: home.path, }, ), { recursive: true, }, );
            let caught: unknown;
            try {
              await loadSettings({ home: home.path, }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught, ).toBeInstanceOf(BashPokeConfigError);
            expect((caught as Error).message, ).toContain('unreadable');
          },
        }, ),
        it({
          name: 'reads back exactly what was written',
          fn: async () => {
            await using home = await createTempHome();
            const path = await writeSettings({
              home: home.path,
              text: '{"progressWidget":false}',
            }, );
            expect(await readFile(path, 'utf8', ), ).toBe('{"progressWidget":false}');
            expect((await loadSettings({ home: home.path, }, )).progressWidget, ).toBe(false);
          },
        }, ),
      ],
    }, ),

    //endregion loadSettings
  ],
}, );
