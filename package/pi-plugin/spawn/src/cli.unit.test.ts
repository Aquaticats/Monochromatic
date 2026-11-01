import { spawnSync, } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SESSION_NOT_FOUND_WARNING, } from './cli-core.ts';
import { tempDir, } from './test-support.ts';

/**
 Number of polling attempts used while waiting for detached fake terminal-exec output.
 */
const DETACHED_WRITE_ATTEMPTS = 50;

/**
 Milliseconds between detached fake terminal-exec output checks.
 */
const DETACHED_WRITE_DELAY_MS = 10;

/**
 Source CLI path used for fallback smoke coverage.
 */
const SOURCE_CLI_PATH = fileURLToPath(new URL(
  'cli.ts',
  import.meta.url,
),);

/**
 Directory name used for fake executables in CLI tests.
 */
const FAKE_BIN_DIRNAME = 'bin';

/**
 File name used for fake terminal-exec executable in CLI tests.
 */
const FAKE_TERMINAL_EXEC_NAME = 'terminal-exec';

/**
 Environment variable carrying fake terminal-exec launch record path.
 */
const FAKE_TERMINAL_RECORD_PATH_ENV = 'SPAWN_PI_FAKE_TERMINAL_RECORD_PATH';

/**
 Non-nullish sentinel written when fake terminal-exec receives no spawn identifier.
 */
const FAKE_TERMINAL_SPAWN_ID_ABSENT = '<absent>';

/**
 Executable permission bits for fake terminal-exec.
 */
const EXECUTABLE_PERMISSION = 0o755;

/**
 Captured fake terminal-exec launch record.
 */
type FakeTerminalRecord = {
  /**
   Arguments passed to terminal-exec.
   */
  readonly argv: readonly string[];
  /**
   Working directory observed by fake terminal-exec.
   */
  readonly cwd: string;
  /**
   Spawn identifier inherited by terminal-exec, or sentinel text in unlinked mode.
   */
  readonly spawnId: string;
};

/**
 Writes fake terminal-exec executable that records its launch payload.
 
 @param binDir - directory receiving fake executable.
 
 @returns path to fake terminal-exec executable.
 
 @example
 ```typescript
 const fakeTerminal = writeFakeTerminalExec('/tmp/bin');
 ```
 */
function writeFakeTerminalExec(binDir: string,): string {
  mkdirSync(
    binDir,
    { recursive: true, },
  );

  /**
   Fake terminal-exec executable path.
   */
  const fakeTerminalPath = join(
    binDir,
    FAKE_TERMINAL_EXEC_NAME,
  );

  writeFileSync(
    fakeTerminalPath,
    [
      '#!/usr/bin/env node',
      "import { writeFileSync } from 'node:fs';",
      'writeFileSync(',
      `  process.env.${FAKE_TERMINAL_RECORD_PATH_ENV},`,
      '  JSON.stringify({',
      '    argv: process.argv.slice(2),',
      '    cwd: process.cwd(),',
      `    spawnId: process.env.PI_SPAWN_ID ?? '${FAKE_TERMINAL_SPAWN_ID_ABSENT}',`,
      '  }),',
      ');',
      '',
    ].join('\n',),
  );
  chmodSync(
    fakeTerminalPath,
    EXECUTABLE_PERMISSION,
  );

  return fakeTerminalPath;
}

/**
 Waits for detached fake terminal-exec to write its launch record.
 
 @param recordPath - JSON record path expected from fake terminal-exec.
 
 @returns parsed launch record.
 
 @throws when fake terminal-exec does not write record before polling ends.
 
 @example
 ```typescript
 const record = await readDetachedRecord('/tmp/record.json');
 ```
 */
async function readDetachedRecord(recordPath: string,): Promise<FakeTerminalRecord> {
  for (let attempt = 0; attempt < DETACHED_WRITE_ATTEMPTS; attempt += 1) {
    if (existsSync(recordPath,)) {
      /**
       Raw JSON written by fake terminal-exec.
       */
      const raw = readFileSync(
        recordPath,
        'utf8',
      );
      /**
       Parsed fake terminal-exec launch record.
       */
      const record = JSON.parse(raw,) as FakeTerminalRecord;
      return record;
    }

    // oxlint-disable-next-line eslint/no-await-in-loop -- Sequential polling observes detached child state after each delay; parallel waits would not check the filesystem between attempts.
    await wait(DETACHED_WRITE_DELAY_MS,);
  }

  throw new Error('fake terminal-exec did not write launch record',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'unlinked fallback',
      children: [
        it({
          name: 'warns and launches child Pi without result forwarding when no parent mapping exists',
          fn: async function testUnlinkedFallback() {
            await using dir = await tempDir({ prefix: 'spawn-pi-cli-fallback-', },);

            /**
             Fake executable directory prepended to PATH.
             */
            const fakeBinDir = join(
              dir.path,
              FAKE_BIN_DIRNAME,
            );
            /**
             Captured fake terminal-exec launch record path.
             */
            const recordPath = join(
              dir.path,
              'terminal-record.json',
            );

            writeFakeTerminalExec(fakeBinDir,);

            /**
             Source CLI result when no PID mapping directory exists.
             */
            const result = spawnSync(
              'node',
              [
                SOURCE_CLI_PATH,
                '--cwd',
                dir.path,
                '--extra-arguments',
                '--thinking high',
                'fallback prompt',
              ],
              {
                encoding: 'utf8',
                env: {
                  ...process.env,
                  PATH: `${fakeBinDir}:${process.env.PATH ?? ''}`,
                  PI_CODING_AGENT_DIR: dir.path,
                  [FAKE_TERMINAL_RECORD_PATH_ENV]: recordPath,
                },
              },
            );

            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain(SESSION_NOT_FOUND_WARNING,);
            expect(result.stdout.trim(),).toBe('{"resultForwarding":false}',);

            /**
             Launch record written by detached fake terminal-exec.
             */
            const record = await readDetachedRecord(recordPath,);

            expect(record.cwd,).toBe(dir.path,);
            expect(record.spawnId,).toBe(FAKE_TERMINAL_SPAWN_ID_ABSENT,);
            expect(record.argv,).toEqual([
              '--title=spawn-pi unlinked',
              '--',
              'pi',
              '--thinking',
              'high',
              'fallback prompt',
            ],);
          },
        },),
      ],
    },),
  ],
},);
