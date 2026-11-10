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
  wait,
  withTimeout,
} from '@monochromatic-dev/async-time-core/ts';

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

/**
 Characters a buffered one-shot runner holds before it ends the child, which is
 Node's documented default `maxBuffer` for `execFile`.
 */
const BUFFER_LIMIT_CHARACTERS = 1024 * 1024;

/**
 Output the chatty launcher writes, comfortably over that bound.
 */
const CHATTY_OUTPUT_CHARACTERS = 2 * BUFFER_LIMIT_CHARACTERS;

/**
 Characters one write of the chatty launcher carries.
 */
const CHATTY_CHUNK_CHARACTERS = 64 * 1024;

/**
 Pause between those writes in milliseconds, so the writes span several
 event-loop turns and an output bound the launcher's parent enforces gets the
 chance to act on a child that is still writing.
 */
const CHATTY_CHUNK_PAUSE_MILLISECONDS = 10;

/**
 How long the held-open launcher runs before it exits, in milliseconds.
 */
const HELD_OPEN_MILLISECONDS = 2000;

/**
 Deadline for the chatty launcher to record that it reached its last write.
 */
const SURVIVAL_DEADLINE_MILLISECONDS = 5000;

/**
 Pause between reads of what the stand-in recorded, in milliseconds.
 */
const POLL_MILLISECONDS = 20;

/**
 Shell lines a stand-in runs to write more than a buffered runner holds, in
 chunks with a pause between them, and then record that it reached the end.
 */
const CHATTY_LAUNCHER_LINES: readonly string[] = [
  'written=0',
  `while [ "$written" -lt ${String(CHATTY_OUTPUT_CHARACTERS)} ]; do`,
  `  dd if=/dev/zero bs=${String(CHATTY_CHUNK_CHARACTERS)} count=1 2>/dev/null | tr '\\0' x`,
  `  written=$((written + ${String(CHATTY_CHUNK_CHARACTERS)}))`,
  `  sleep ${String(CHATTY_CHUNK_PAUSE_MILLISECONDS / 1000)}`,
  'done',
  `printf 'survived\\n' >> "$KEY_HELPER_FAKE_ARGUMENTS"`,
];

/**
 Line the chatty launcher appends once every write succeeded.
 */
const SURVIVAL_LINE = 'survived';

/**
 Read the stand-in's recorded argument vector until it holds a given line, or
 the deadline passes.
 
 @param readArguments - Read of what the stand-in recorded so far
 
 @param expected - Line the recording must hold
 
 @param deadlineMilliseconds - Longest the case waits for it
 
 @returns The recorded vector, with or without the expected line
 
 @example
 ```ts
 await recordedUntil({ readArguments: fake.readArguments, expected: 'survived', deadlineMilliseconds: 5000 });
 ```
 */
async function recordedUntil({
  readArguments,
  expected,
  deadlineMilliseconds,
}: {
  readonly readArguments: () => Promise<readonly string[]>;
  readonly expected: string;
  readonly deadlineMilliseconds: number;
}): Promise<readonly string[]> {
  return await withTimeout({
    promise: (async function pollUntilRecorded(): Promise<readonly string[]> {
      /**
       Longest the poll keeps reading before the deadline above ends it.
       */
      const attempts = Math.ceil(deadlineMilliseconds / POLL_MILLISECONDS);
      for (let attempt = 0; attempt < attempts; attempt += 1) {
        /**
         What the stand-in has recorded so far.
         */
        const recorded = await readArguments();
        if (recorded.includes(expected)) {
          return recorded;
        }
        await wait(POLL_MILLISECONDS,);
      }
      return await readArguments();
    })(),
    ms: deadlineMilliseconds,
    label: `the launcher recording '${expected}'`,
  },);
}

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
        it({
          name: 'returns before a launcher that keeps running has exited',
          fn: async () => {
            using _held = await installFakeCommand({
              name: 'kstart',
              exitCode: 0,
              standardError: '',
              extraLines: [`sleep ${String(HELD_OPEN_MILLISECONDS / 1000)}`,],
            });
            /**
             When the launch was asked for, so the case can tell a launcher
             left running from one waited for.
             */
            const startedAt = performance.now();
            await launchNewInstance({ desktopFileName: '', resourceClass: SOME_APP, });
            expect(performance.now() - startedAt).toBeLessThan(HELD_OPEN_MILLISECONDS / 2);
          },
        }),
        it({
          name: 'keeps a launcher alive that writes more than a buffered runner holds',
          fn: async () => {
            using fake = await installFakeCommand({
              name: 'kstart',
              exitCode: 0,
              standardError: '',
              extraLines: CHATTY_LAUNCHER_LINES,
            });
            await launchNewInstance({ desktopFileName: '', resourceClass: SOME_APP, });
            /**
             What the launcher recorded: its arguments, then the line it appends
             only when every write of its output succeeded.
             */
            const recorded = await recordedUntil({
              readArguments: fake.readArguments,
              expected: SURVIVAL_LINE,
              deadlineMilliseconds: SURVIVAL_DEADLINE_MILLISECONDS,
            });
            expect(recorded).toEqual([SOME_APP, SURVIVAL_LINE,]);
          },
        }),
      ],
    }),
  ],
});
