/**
 Tests for the new-instance launcher: `selectLaunchCommand` (override table
 first, case-insensitive, then the desktop file, then the bare resource class,
 then nothing) and `launchNewInstance`, which runs the chosen command and
 reports a launcher that failed.

 The launcher is a stand-in `kstart` shell script first on `PATH`, so no
 application is started.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  flushLogs,
  launchNewInstance,
  NoAppIdentityError,
  selectLaunchCommand,
} from '../dist/final/node/testing.mjs';

import {
  capturedStandardError,
  installFakeCommand,
} from './fake-command-fixture.ts';

/**
 Resource class with no override entry, so the launcher is `kstart`.
 */
const SOME_APP = 'someapp';

/**
 Exit status the stand-in launcher reports when it is meant to fail.
 */
const FAILURE_EXIT_CODE = 3;

/**
 Text the stand-in launcher writes to stderr when it fails.
 */
const FAILURE_TEXT = 'kstart: no such desktop file';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: selectLaunchCommand.name,
      children: [
        it({
          name: 'uses the ghostty override for a fresh process',
          fn: async () => {
            expect(selectLaunchCommand({ desktopFileName: '', resourceClass: 'ghostty' })).toEqual({
              cmd: 'ghostty',
              args: ['--gtk-single-instance=false'],
            });
          },
        }),
        it({
          name: 'matches the override by the reverse-DNS resource class too',
          fn: async () => {
            expect(
              selectLaunchCommand({ desktopFileName: '', resourceClass: 'com.mitchellh.ghostty' }),
            ).toEqual({ cmd: 'ghostty', args: ['--gtk-single-instance=false'] });
          },
        }),
        it({
          name: 'matches overrides case-insensitively',
          fn: async () => {
            expect(selectLaunchCommand({ desktopFileName: '', resourceClass: 'Firefox' })).toEqual({
              cmd: 'firefox',
              args: ['--new-window'],
            });
          },
        }),
        it({
          name: 'falls back to the desktop file via kstart',
          fn: async () => {
            expect(
              selectLaunchCommand({ desktopFileName: 'org.kde.konsole', resourceClass: 'konsole' }),
            ).toEqual({ cmd: 'kstart', args: ['--application', 'org.kde.konsole'] });
          },
        }),
        it({
          name: 'falls back to the bare resource class via kstart',
          fn: async () => {
            expect(
              selectLaunchCommand({ desktopFileName: '', resourceClass: SOME_APP }),
            ).toEqual({ cmd: 'kstart', args: [SOME_APP] });
          },
        }),
        it({
          name: 'throws NoAppIdentityError when no app identity is provided',
          fn: async () => {
            expect(() => selectLaunchCommand({ desktopFileName: '', resourceClass: '' })).toThrow(
              NoAppIdentityError,
            );
          },
        }),
      ],
    }),
    describe({
      name: launchNewInstance.name,
      children: [
        it({
          name: 'runs the selected launcher with the resource class',
          fn: async () => {
            using fake = await installFakeCommand({
              name: 'kstart',
              exitCode: 0,
              standardError: '',
            });
            await launchNewInstance({ desktopFileName: '', resourceClass: SOME_APP, });
            expect(await fake.readArguments()).toEqual([SOME_APP]);
          },
        }),
        it({
          name: 'logs a launcher that exited nonzero, with its status and stderr',
          fn: async ctx => {
            using fake = await installFakeCommand({
              name: 'kstart',
              exitCode: FAILURE_EXIT_CODE,
              standardError: FAILURE_TEXT,
            });
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await launchNewInstance({ desktopFileName: '', resourceClass: SOME_APP, });
            await flushLogs();
            /** Log text the failed launch produced. */
            const reported = capturedStandardError(standardError)
              .filter(function namesLaunchFailure(line: string): boolean {
                return line.includes(`exited with code ${String(FAILURE_EXIT_CODE)}`);
              });
            expect(reported).toHaveLength(1);
            expect(reported[0]).toContain('kstart');
            expect(reported[0]).toContain(FAILURE_TEXT);
            expect(await fake.readArguments()).toEqual([SOME_APP]);
          },
        }),
        it({
          name: 'logs nothing when the launcher succeeds',
          fn: async ctx => {
            using _fake = await installFakeCommand({
              name: 'kstart',
              exitCode: 0,
              standardError: '',
            });
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await launchNewInstance({ desktopFileName: '', resourceClass: SOME_APP, });
            await flushLogs();
            /** Log lines naming this launcher's failure. */
            const reported = capturedStandardError(standardError)
              .filter(function namesLaunchFailure(line: string): boolean {
                return line.includes('launch failed');
              });
            expect(reported).toEqual([]);
          },
        }),
      ],
    }),
  ],
});
