/**
 Tests for settings validation in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BashPokeConfigError,
  DEFAULT_SETTINGS,
  parseSettings,
  readBooleanSetting,
  readNumberSetting,
  readStringSetting,
  SETTING_KEYS,
  SETTING_KEY_SET,
} from '../dist/final/node/index.mjs';

/**
 File name used in diagnostics across these cases.
 */
const FILE_NAME = 'pi-bash-poke.json';

await describe({
  name: '',
  children: [
    //region parseSettings

    describe({
      name: parseSettings.name,
      children: [
        it({
          name: 'applies every default to an empty object',
          fn: async () => {
            expect(parseSettings({ decoded: {}, fileName: FILE_NAME, }, ), )
              .toEqual(DEFAULT_SETTINGS);
          },
        }, ),
        it({
          name: 'honors each configured key',
          fn: async () => {
            const settings = parseSettings({
              decoded: {
                pokeInstruction: 'resume',
                pokeHeadChars: 10,
                pokeTailChars: 20,
                progressWidget: false,
                progressTailLines: 1,
                progressRefreshMs: 5,
                killGraceMs: 7,
              },
              fileName: FILE_NAME,
            }, );
            expect(settings.pokeInstruction, ).toBe('resume');
            expect(settings.pokeHeadChars, ).toBe(10);
            expect(settings.pokeTailChars, ).toBe(20);
            expect(settings.progressWidget, ).toBe(false);
            expect(settings.progressTailLines, ).toBe(1);
            expect(settings.progressRefreshMs, ).toBe(5);
            expect(settings.killGraceMs, ).toBe(7);
          },
        }, ),
        it({
          name: 'accepts zero for every numeric key',
          fn: async () => {
            const settings = parseSettings({
              decoded: {
                pokeHeadChars: 0,
                pokeTailChars: 0,
                progressTailLines: 0,
                progressRefreshMs: 0,
                killGraceMs: 0,
              },
              fileName: FILE_NAME,
            }, );
            expect(settings.pokeTailChars, ).toBe(0);
            expect(settings.killGraceMs, ).toBe(0);
          },
        }, ),
        it({
          name: 'accepts an empty instruction so a poke can carry none',
          fn: async () => {
            const settings = parseSettings({
              decoded: { pokeInstruction: '', },
              fileName: FILE_NAME,
            }, );
            expect(settings.pokeInstruction, ).toBe('');
          },
        }, ),
        it({
          name: 'rejects a non-object document',
          fn: async () => {
            let caught: unknown;
            try {
              parseSettings({ decoded: [], fileName: FILE_NAME, }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught, ).toBeInstanceOf(BashPokeConfigError);
            expect((caught as Error).message, ).toContain('must be a JSON object');
          },
        }, ),
        it({
          name: 'rejects a scalar document',
          fn: async () => {
            let caught: unknown;
            try {
              parseSettings({ decoded: 'nope', fileName: FILE_NAME, }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught, ).toBeInstanceOf(BashPokeConfigError);
          },
        }, ),
        it({
          name: 'names every unknown key it rejects',
          fn: async () => {
            let caught: unknown;
            try {
              parseSettings({
                decoded: { pokeHead: 1, shell: 'zsh', },
                fileName: FILE_NAME,
              }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught, ).toBeInstanceOf(BashPokeConfigError);
            expect((caught as Error).message, ).toContain('pokeHead');
            expect((caught as Error).message, ).toContain('shell');
          },
        }, ),
        it({
          name: 'rejects a fractional or negative number',
          fn: async () => {
            let fractional: unknown;
            try {
              parseSettings({ decoded: { pokeHeadChars: 1.5, }, fileName: FILE_NAME, }, );
            }
            catch (error: unknown) {
              fractional = error;
            }
            expect((fractional as Error).message, ).toContain('whole number');

            let negative: unknown;
            try {
              parseSettings({ decoded: { pokeHeadChars: -1, }, fileName: FILE_NAME, }, );
            }
            catch (error: unknown) {
              negative = error;
            }
            expect((negative as Error).message, ).toContain('must not be negative');
          },
        }, ),
      ],
    }, ),

    //endregion parseSettings

    //region Per-kind readers

    describe({
      name: readStringSetting.name,
      children: [
        it({
          name: 'falls back when absent and rejects a wrong kind',
          fn: async () => {
            expect(
              readStringSetting({
                record: {},
                key: 'pokeInstruction',
                fallback: 'continue',
                fileName: FILE_NAME,
              }, ),
            ).toBe('continue');

            let caught: unknown;
            try {
              readStringSetting({
                record: { pokeInstruction: 3, },
                key: 'pokeInstruction',
                fallback: 'continue',
                fileName: FILE_NAME,
              }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught, ).toBeInstanceOf(BashPokeConfigError);
            expect((caught as Error).message, ).toContain('must be a string');
          },
        }, ),
      ],
    }, ),

    describe({
      name: readNumberSetting.name,
      children: [
        it({
          name: 'falls back when absent and rejects a wrong kind',
          fn: async () => {
            expect(
              readNumberSetting({
                record: {},
                key: 'pokeHeadChars',
                fallback: 2_000,
                fileName: FILE_NAME,
              }, ),
            ).toBe(2_000);

            let caught: unknown;
            try {
              readNumberSetting({
                record: { pokeHeadChars: '2000', },
                key: 'pokeHeadChars',
                fallback: 2_000,
                fileName: FILE_NAME,
              }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect((caught as Error).message, ).toContain('must be a number');
          },
        }, ),
      ],
    }, ),

    describe({
      name: readBooleanSetting.name,
      children: [
        it({
          name: 'falls back when absent and rejects a wrong kind',
          fn: async () => {
            expect(
              readBooleanSetting({
                record: {},
                key: 'progressWidget',
                fallback: true,
                fileName: FILE_NAME,
              }, ),
            ).toBe(true);

            let caught: unknown;
            try {
              readBooleanSetting({
                record: { progressWidget: 'yes', },
                key: 'progressWidget',
                fallback: true,
                fileName: FILE_NAME,
              }, );
            }
            catch (error: unknown) {
              caught = error;
            }
            expect((caught as Error).message, ).toContain('must be a boolean');
          },
        }, ),
      ],
    }, ),

    //endregion Per-kind readers

    //region Key inventory

    describe({
      name: 'SETTING_KEYS',
      children: [
        it({
          name: 'lists every documented key exactly once',
          fn: async () => {
            expect(SETTING_KEYS, ).toHaveLength(8);
            expect(new Set(SETTING_KEYS, ).size, ).toBe(8);
            expect(SETTING_KEY_SET.has('killGraceMs', ), ).toBe(true);
            expect(SETTING_KEY_SET.has('shell', ), ).toBe(false);
          },
        }, ),
        it({
          name: 'gives every key a default',
          fn: async () => {
            for (const key of SETTING_KEYS)
              expect(Object.hasOwn(DEFAULT_SETTINGS, key, ), ).toBe(true);
          },
        }, ),
      ],
    }, ),

    //endregion Key inventory
  ],
}, );
