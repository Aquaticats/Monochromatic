import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  dirname,
  join,
  resolve,
} from 'node:path';

import { scratchSegments } from './constants.ts';

/**
 Programs the release and wait commands start; the fixture shadows every one of them, so a test can never reach a
 real `cargo`, `git`, `gh`, `curl`, `mise` or `systemd-run`.
 */
export const stubbedPrograms: readonly string[] = [
  'cargo',
  'mise',
  'systemd-run',
  'git',
  'gh',
  'curl',
];

/**
 Permission bits of a stub executable: readable and executable by everyone, writable by the owner.
 */
const executableMode = 0o755;

/**
 Permission bits of the scratch directory: the owner only, as trust checks require.
 */
const privateDirectoryMode = 0o700;

/**
 Source of every stub: it appends its call to the log, then prints and exits as the fixture's files say.
 Written as CommonJS because the file has no extension and no package scope.
 */
const stubSource = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const name = path.basename(process.argv[1]);
const args = process.argv.slice(2);
const home = path.dirname(process.argv[1]);
const key = name === 'gh' ? name + '.' + args[0] + '-' + args[1] : name;
fs.appendFileSync(path.join(home, '..', 'calls.jsonl'), JSON.stringify({ program: name, args, cwd: process.cwd() }) + '\\n');
const out = path.join(home, key + '.out');
if (fs.existsSync(out)) process.stdout.write(fs.readFileSync(out, 'utf8'));
const status = path.join(home, key + '.status');
process.exitCode = fs.existsSync(status) ? Number(fs.readFileSync(status, 'utf8')) : 0;
`;

/**
 One recorded call of a stubbed program.
 */
export type RecordedCall = {
  readonly program: string;
  readonly args: readonly string[];
  readonly cwd: string;
};

/**
 Disposable directory with stubbed programs, a call log and room for a fixture repository.
 Declare it with `await using`, and the directory is removed when the scope ends, even when the body throws.
 */
export type StubFixture = AsyncDisposable & {
  /**
   Real path of the fixture directory, which is the repository root of the test.
   */
  readonly directory: string;
  /**
   Set what a stub prints (`<key>.out`) or the status it exits with (`<key>.status`); the key of `gh` is
   `gh.<command>-<subcommand>`, the key of the others is their name.
   */
  readonly script: (input: {
    readonly key: string;
    readonly stdout?: string;
    readonly status?: number
  }) => Promise<void>;
  /**
   Write a file of the fixture repository, creating its directories.
   */
  readonly write: (input: {
    readonly path: string;
    readonly text: string
  }) => Promise<void>;
  /**
   Read a file of the fixture repository.
   */
  readonly read: (path: string) => Promise<string>;
  /**
   Calls the stubs recorded so far, in order.
   */
  readonly calls: () => Promise<readonly RecordedCall[]>;
  /**
   Environment for a child process that finds the stubs before any real program.
   */
  readonly environment: () => NodeJS.ProcessEnv;
};

/**
 Check that a parsed log line has the shape of a recorded call.

 @param value - parsed JSON

 @returns true when it has a program, an argument list and a working directory

 @example
 ```ts
 isRecordedCall({ program: 'cargo', args: [], cwd: '/r' });
 ```
 */
function isRecordedCall(value: unknown): value is RecordedCall {
  return ((typeof value) === 'object') && (value !== null)
    && ('program' in value)
    && ((typeof value.program) === 'string')
    && ('args' in value)
    && Array.isArray(value.args)
    && ('cwd' in value)
    && ((typeof value.cwd) === 'string');
}

/**
 Create the fixture under the scratch directory, with every stub in place.

 @returns the fixture; declare it with `await using` so the directory is removed afterwards

 @example
 ```ts
 await using fixture = await createStubFixture();
 ```
 */
export async function createStubFixture(): Promise<StubFixture> {
  /**
   Scratch directory of agent runs, which is never the system temporary directory.
   */
  const scratch = join(
    homedir(),
    ...scratchSegments
  );
  await mkdir(
    scratch,
    {
      recursive: true,
      mode: privateDirectoryMode
    }
  );
  /**
   Fresh fixture directory.
   */
  const directory = await realpath(
    await mkdtemp(join(
      scratch,
      'crate-release-'
    )),
  );
  /**
   Directory of the stub executables.
   */
  const stubDirectory = join(
    directory,
    'stubs'
  );
  await mkdir(stubDirectory);
  /**
   Path of the call log, which starts empty so a read never meets a missing file.
   */
  const callLogPath = join(
    directory,
    'calls.jsonl'
  );
  await writeFile(
    callLogPath,
    ''
  );
  await Promise.all(stubbedPrograms.map(async function writeStub(program) {
    /**
     Path of one stub.
     */
    const path = join(
      stubDirectory,
      program
    );
    await writeFile(
      path,
      stubSource
    );
    await chmod(
      path,
      executableMode
    );
  }));
  return {
    directory,
    script: async function script({
      key,
      stdout,
      status,
    }) {
      if (stdout !== undefined)
        await writeFile(
          join(
            stubDirectory,
            `${key}.out`
          ),
          stdout
        );
      if (status !== undefined)
        await writeFile(
          join(
            stubDirectory,
            `${key}.status`
          ),
          String(status)
        );
    },
    write: async function write({
      path,
      text,
    }) {
      /**
       Absolute path inside the fixture.
       */
      const target = resolve(
        directory,
        path
      );
      await mkdir(
        dirname(target),
        { recursive: true }
      );
      await writeFile(
        target,
        text
      );
    },
    read: async function read(path) {
      return await readFile(
        resolve(
          directory,
          path
        ),
        'utf8'
      );
    },
    calls: async function calls() {
      return (await readFile(
        callLogPath,
        'utf8'
      ))
        .split('\n')
        .filter(function nonEmpty(line) {
          return line !== '';
        })
        .map(function parse(line): unknown {
          return JSON.parse(line);
        })
        .filter(isRecordedCall);
    },
    environment: function environment() {
      return {
        ...process.env,
        PATH: `${stubDirectory}:${process.env
          .PATH
          ?? ''}`
      };
    },
    [Symbol.asyncDispose]: async function dispose() {
      await rm(
        directory,
        {
          recursive: true,
          force: true
        }
      );
    },
  };
}
