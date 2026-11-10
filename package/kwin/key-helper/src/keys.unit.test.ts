/**
 Tests for the injection half of the package: `keysToEvdev`, the combo-to-evdev
 translation that is the injection security boundary, and `sendKeys`, which
 runs `ydotool` and reports a run that failed.

 `ydotool` is a stand-in shell script first on `PATH`, so nothing is injected
 into the owner's session and no `ydotoold` is needed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  flushLogs,
  keysToEvdev,
  sendKeys,
  UnknownKeyError,
} from '../dist/final/node/testing.mjs';

import {
  capturedStandardError,
  installFakeCommand,
} from './fake-command-fixture.ts';

//region Fixtures
// The combo every case injects, the tokens it must become, and the failure a
// stand-in `ydotool` reports, so a case states only what it varies.

/**
 Combo whose tokens are known, used by the `sendKeys` cases.
 */
const COMBO = 'ctrl+w';

/**
 Tokens `ydotool key` receives for {@link COMBO}.
 */
const COMBO_TOKENS: readonly string[] = [
  'key',
  '29:1',
  '17:1',
  '17:0',
  '29:0',
];

/**
 Exit status `ydotool` reports when `ydotoold` is not running.
 */
const FAILURE_EXIT_CODE = 2;

/**
 Text `ydotool` writes to stderr when `ydotoold` is not running.
 */
const FAILURE_TEXT = 'failed to connect to ydotoold: No such file or directory';

/**
 Lowercase `Object.prototype` names, which a plain index would answer.
 */
const INHERITED_NAMES: readonly string[] = [
  'constructor',
  '__proto__',
  'hasOwnProperty',
];

//endregion Fixtures

//region Cases
// Both exported halves: the translation that turns a combo into evdev tokens
// or rejects it, and the runner that reports what `ydotool` did.

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: keysToEvdev.name,
      children: [
        it({
          name: 'presses left-to-right then releases right-to-left',
          fn: async () => {
            expect(keysToEvdev(COMBO)).toEqual(['29:1', '17:1', '17:0', '29:0']);
          },
        }),
        it({
          name: 'handles a single key',
          fn: async () => {
            expect(keysToEvdev('w')).toEqual(['17:1', '17:0']);
          },
        }),
        it({
          name: 'is case-insensitive',
          fn: async () => {
            expect(keysToEvdev('CTRL+W')).toEqual(keysToEvdev(COMBO));
          },
        }),
        it({
          name: 'nests releases for a three-key combo',
          fn: async () => {
            expect(keysToEvdev('ctrl+shift+t')).toEqual([
              '29:1',
              '42:1',
              '20:1',
              '20:0',
              '42:0',
              '29:0',
            ]);
          },
        }),
        it({
          name: 'throws UnknownKeyError on an unmapped token',
          fn: async () => {
            expect(() => keysToEvdev('ctrl+hyper')).toThrow(UnknownKeyError);
          },
        }),
        it({
          name: 'throws on an empty combo',
          fn: async () => {
            expect(() => keysToEvdev('')).toThrow(UnknownKeyError);
          },
        }),
        it({
          name: 'rejects a shell-injection-shaped token instead of forwarding it',
          fn: async () => {
            expect(() => keysToEvdev('ctrl; rm -rf /')).toThrow(UnknownKeyError);
          },
        }),
        ...INHERITED_NAMES.map(function mapInheritedName(name: string) {
          return it({
            name: `rejects the Object.prototype name ${name} instead of translating it`,
            fn: async () => {
              expect(() => keysToEvdev(name)).toThrow(UnknownKeyError);
            },
          });
        }),
      ],
    }),
    describe({
      name: sendKeys.name,
      children: [
        it({
          name: 'runs ydotool key with the translated combo',
          fn: async () => {
            using fake = await installFakeCommand({
              name: 'ydotool',
              exitCode: 0,
              standardError: '',
            });
            await sendKeys(COMBO);
            expect(await fake.readArguments()).toEqual(COMBO_TOKENS);
          },
        }),
        it({
          name: 'logs a ydotool that exited nonzero, with its status and stderr',
          fn: async ctx => {
            using fake = await installFakeCommand({
              name: 'ydotool',
              exitCode: FAILURE_EXIT_CODE,
              standardError: FAILURE_TEXT,
            });
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await sendKeys(COMBO);
            await flushLogs();
            /** Log text the failed injection produced. */
            const reported = capturedStandardError(standardError)
              .filter(function namesInjectionFailure(line: string): boolean {
                return line.includes(`exited with code ${String(FAILURE_EXIT_CODE)}`);
              });
            expect(reported).toHaveLength(1);
            expect(reported[0]).toContain('ydotool');
            expect(reported[0]).toContain(FAILURE_TEXT);
            expect(await fake.readArguments()).toEqual(COMBO_TOKENS);
          },
        }),
        it({
          name: 'logs nothing when ydotool succeeds',
          fn: async ctx => {
            using _fake = await installFakeCommand({
              name: 'ydotool',
              exitCode: 0,
              standardError: '',
            });
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await sendKeys(COMBO);
            await flushLogs();
            /** Log lines naming an injection failure. */
            const reported = capturedStandardError(standardError)
              .filter(function namesInjectionFailure(line: string): boolean {
                return line.includes('ydotool/key error');
              });
            expect(reported).toEqual([]);
          },
        }),
        it({
          name: 'logs an unknown token instead of running ydotool',
          fn: async ctx => {
            using fake = await installFakeCommand({
              name: 'ydotool',
              exitCode: 0,
              standardError: '',
            });
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await sendKeys('ctrl+hyper');
            await flushLogs();
            expect(await fake.readArguments()).toEqual([]);
            /** Log text the unknown token produced. */
            const reported = capturedStandardError(standardError)
              .filter(function namesUnknownKey(line: string): boolean {
                return line.includes('unknown key: hyper');
              });
            expect(reported).toHaveLength(1);
          },
        }),
      ],
    }),
  ],
});

//endregion Cases
