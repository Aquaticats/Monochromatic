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
  join,
  resolve,
} from 'node:path';

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
 */
export type StubFixture = {
  /**
   Real path of the fixture directory, which is the repository root of the test.
   */
  readonly directory: string;
  /**
   Directory to put first on `PATH`.
   */
  readonly stubDirectory: string;
  /**
   Set what a stub prints (`<key>.out`) or the status it exits with (`<key>.status`); the key of `gh` is
   `gh.<command>-<subcommand>`, the key of the others is their name.
   */
  readonly script: (input: { readonly key: string; readonly stdout?: string; readonly status?: number }) => Promise<void>;
  /**
   Write a file of the fixture repository, creating its directories.
   */
  readonly write: (input: { readonly path: string; readonly text: string }) => Promise<void>;
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
 Create the fixture under the scratch directory, run the body, and remove the fixture afterwards.

 @param body - test body that receives the fixture
 @returns nothing; the fixture is removed even when the body throws
 */
export async function withStubFixture(body: (fixture: StubFixture) => Promise<void>): Promise<void> {
  /**
   Scratch directory of agent runs, which is never the system temporary directory.
   */
  const scratch = join(homedir(), 'temp', 'agent');
  await mkdir(scratch, { recursive: true, mode: 0o700 });
  /**
   Fresh fixture directory.
   */
  const directory = await realpath(await mkdtemp(join(scratch, 'crate-release-')));
  /**
   Directory of the stub executables.
   */
  const stubDirectory = join(directory, 'stubs');
  await mkdir(stubDirectory);
  try {
    for (const program of stubbedPrograms) {
      /**
       Path of one stub.
       */
      const path = join(stubDirectory, program);
      await writeFile(path, stubSource);
      await chmod(path, 0o755);
    }
    await body({
      directory,
      stubDirectory,
      script: async function script({ key, stdout, status }) {
        if (stdout !== undefined)
          await writeFile(join(stubDirectory, `${key}.out`), stdout);
        if (status !== undefined)
          await writeFile(join(stubDirectory, `${key}.status`), String(status));
      },
      write: async function write({ path, text }) {
        /**
         Absolute path inside the fixture.
         */
        const target = resolve(directory, path);
        await mkdir(join(target, '..'), { recursive: true });
        await writeFile(target, text);
      },
      read: async function read(path) {
        return await readFile(resolve(directory, path), 'utf8');
      },
      calls: async function calls() {
        /**
         Log text, empty when no stub ran.
         */
        const log = await readFile(join(directory, 'calls.jsonl'), 'utf8').catch(function missing() {
          return '';
        });
        return log
          .split('\n')
          .filter(function nonEmpty(line) {
            return line !== '';
          })
          .map(function parse(line) {
            return JSON.parse(line) as RecordedCall;
          });
      },
      environment: function environment() {
        return { ...process.env, PATH: `${stubDirectory}:${process.env['PATH'] ?? ''}` };
      },
    });
  }
  finally {
    await rm(directory, { recursive: true, force: true });
  }
}
