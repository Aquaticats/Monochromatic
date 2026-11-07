/**
 A disposable repository, a recording stand-in for `gh`, and the real daily update run over them.

 The update's own commands (`toolchain:update`, `cargo-shadow update` and the rest) are replaced by small scripts that
 move a lock the way the real ones do, and two crates and one TypeScript package are checked through real `mise` tasks.
 Everything else is real: `mise`, `git`, the process groups, the run lock, the pending file and the `gh` call path.
 The repository has no remote and the stand-in `gh` is first on `PATH`, so no real issue can be opened.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmod,
  mkdir,
  mkdtempDisposable,
  readFile,
  realpath,
  stat,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { text, } from 'node:stream/consumers';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 Upper bound that stops a broken fixture from hanging verification.
 */
const WHOLE_PATH_TIMEOUT_MS = 180_000;

/**
 Folder of the daily update's TypeScript source, which the fixture runs directly as the unit does.
 */
const SOURCE_DIRECTORY = join(import.meta.dirname,);

/**
 A call the stand-in `gh` received.
 */
export type RecordedGhCall = {
  /**
   Arguments after `gh`.
   */
  readonly args: readonly string[];
  /**
   Text sent on standard input.
   */
  readonly stdin: string;
  /**
   The environment values the update promises to set, as `gh` saw them.
   */
  readonly environment: Readonly<Record<string, string>>;
};

/**
 A desktop notification the stand-in `notify-send` received.
 */
export type RecordedNotification = {
  /**
   Arguments after `notify-send`: the options, the title and the text.
   */
  readonly args: readonly string[];
  /**
   The session bus address the notifier was given.
   */
  readonly bus: string;
};

/**
 What the whole run left behind.
 */
export type WholePathResult = {
  /**
   Exit status of the daily update.
   */
  readonly exitCode: number;
  /**
   Everything the update printed on standard error, which is its journal.
   */
  readonly journal: string;
  /**
   Calls the stand-in `gh` received, in order.
   */
  readonly ghCalls: readonly RecordedGhCall[];
  /**
   Desktop notifications the stand-in `notify-send` received, in order.
   */
  readonly notifications: readonly RecordedNotification[];
  /**
   Commit subjects of the repository, newest first.
   */
  readonly commitSubjects: readonly string[];
  /**
   Text of the pending issues file, empty when the run wrote none.
   */
  readonly pendingText: string;
};

/**
 Options of the whole-path run.
 */
export type WholePathOptions = {
  /**
   Exit status the stand-in `gh` gives to every call; zero for a working GitHub.
   */
  readonly ghExitCode: number;
};

/**
 The fake `gh`: records its calls and answers like an empty tracker.
 */
const FAKE_GH = `#!/usr/bin/env node
const { appendFileSync, readFileSync } = require('node:fs');
const stdin = process.stdin.isTTY ? '' : readFileSync(0, 'utf8');
const names = ['GIT_ASKPASS', 'GIT_TERMINAL_PROMPT', 'GH_PROMPT_DISABLED'];
const environment = Object.fromEntries(names.map((name) => [name, process.env[name] === undefined ? '(unset)' : process.env[name]]));
appendFileSync(process.env.FIXTURE_GH_LOG, JSON.stringify({ args: process.argv.slice(2), stdin, environment }) + '\\n');
const exitCode = Number(process.env.FIXTURE_GH_EXIT ?? '0');
if (exitCode !== 0) { process.stderr.write('error connecting to api.github.com\\n'); process.exit(exitCode); }
const verb = process.argv[3];
if (verb === 'list') process.stdout.write('[]');
if (verb === 'create') process.stdout.write('https://github.com/example/fixture/issues/1\\n');
`;

/**
 The fake `notify-send`: records the call and its bus address and shows nothing, so no run of the fixture can reach the
 owner's desktop.
 */
const FAKE_NOTIFY_SEND = `#!/usr/bin/env node
const { appendFileSync } = require('node:fs');
appendFileSync(process.env.FIXTURE_NOTIFY_LOG, JSON.stringify({ args: process.argv.slice(2), bus: process.env.DBUS_SESSION_BUS_ADDRESS ?? '(unset)' }) + '\\n');
`;

/**
 The fake `cargo-shadow`: its `update` moves zbus in the two crates' locks, as the real update did on 2026-11-07.
 */
const FAKE_CARGO_SHADOW = `#!/usr/bin/env node
const { readFileSync, writeFileSync } = require('node:fs');
if (process.argv[2] === 'update') {
  for (const lock of ['package/a/broken-crate/Cargo.lock', 'package/c/red-crate/Cargo.lock']) {
    writeFileSync(lock, readFileSync(lock, 'utf8').replace('version = "5.19.0"', 'version = "6.2.1"'));
  }
}
`;

/**
 The fake `pnpm`: nothing to install in the fixture.
 */
const FAKE_PNPM = `#!/usr/bin/env node
`;

/**
 Script that replaces one text with another in a file, standing in for an update that moves a lock.
 */
const MOVE_SCRIPT = `import { readFileSync, writeFileSync } from 'node:fs';
const [file, from, to] = process.argv.slice(2);
writeFileSync(file, readFileSync(file, 'utf8').replace(from, to));
`;

/**
 Script that stands in for a crate's Clippy: it fails when the lock names zbus 6, or always when asked.
 */
const CLIPPY_SCRIPT = `import { readFileSync } from 'node:fs';
const [code, always] = process.argv.slice(2);
const lock = readFileSync('Cargo.lock', 'utf8');
if (always === 'always' || lock.includes('version = "6.2.1"')) {
  console.log('warning: unused import');
  console.log('');
  console.log('error[' + code + ']: failed to resolve: could not find \`blocking\` in \`zbus\`');
  console.log(' --> src/session.rs:12:11');
  process.exit(101);
}
`;

/**
 A Cargo lock text with zbus at 5.19.0.
 */
const CARGO_LOCK = '[[package]]\nname = "zbus"\nversion = "5.19.0"\n\n[[package]]\nname = "serde"\nversion = "1.0.200"\n';

/**
 Root `mise.toml`: the update's task names, each a script that moves a lock or does nothing.
 */
const ROOT_MISE = `monorepo_root = true

[monorepo]
config_roots = ["package/*/*"]

[settings]
experimental = true
task.monorepo_depth = 3

[tasks."toolchain:update"]
run = "node steps/move.ts toolchain.lock.json nightly-1 nightly-2"
[tasks."toolchain:install"]
run = "node -e 0"
[tasks."toolchain:update:global"]
run = "node steps/move.ts $FIXTURE_GLOBAL_LOCK nightly-1 nightly-2"
[tasks."toolchain:install:global"]
run = "node -e 0"
[tasks."deps:update"]
run = "node -e 0"
[tasks."catalog:tighten"]
run = "node -e 0"
[tasks."sync:files"]
run = "node -e 0"
[tasks."check:files"]
run = "node -e 0"
`;

/**
 Writes a file, making its folder first.

 @param path - absolute path

 @param content - text

 @example
 ```ts
 await put({ path: '/r/a/b', content: 'x', },);
 ```
 */
export async function put({
  path,
  content,
}: {
  readonly path: string;
  readonly content: string;
},): Promise<void> {
  await mkdir(dirname(path,), { recursive: true, },);
  await writeFile(path, content,);
}

/**
 Runs a program to its end and returns its status and output.

 @param program - program name or path

 @param args - arguments

 @param directory - working directory

 @param environment - process environment

 @returns the exit status and the text of both output streams

 @throws Error when the program runs past the fixture's bound

 @example
 ```ts
 const { exitCode, } = await run({ program: 'git', args: ['status',], directory: '/r', environment: {}, },);
 ```
 */
export async function run({
  program,
  args,
  directory,
  environment,
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly directory: string;
  readonly environment: NodeJS.ProcessEnv;
},): Promise<{ readonly exitCode: number; readonly stdout: string; readonly stderr: string; }> {
  /**
   The process, stopped when it runs past the bound.
   */
  const child = spawn(program, [...args,], {
    cwd: directory,
    env: environment,
    timeout: WHOLE_PATH_TIMEOUT_MS,
    stdio: [
      'ignore',
      'pipe',
      'pipe',
    ],
  },);
  // Reading both streams to their end while waiting for the exit keeps a full pipe from stalling the child.
  const [stdout, stderr,] = await Promise.all([
    text(child.stdout,),
    text(child.stderr,),
    once(child, 'close',),
  ],);
  return {
    exitCode: child.exitCode ?? -1,
    stdout,
    stderr,
  };
}

/**
 Writes an executable with the given content into a folder.

 @param directory - folder

 @param name - file name

 @param content - script text starting with a shebang

 @example
 ```ts
 await putExecutable({ directory: '/r/bin', name: 'gh', content: '#!/usr/bin/env node\n', },);
 ```
 */
export async function putExecutable({
  directory,
  name,
  content,
}: {
  readonly directory: string;
  readonly name: string;
  readonly content: string;
},): Promise<void> {
  /**
   Absolute path of the executable.
   */
  const path = join(directory, name,);
  await put({ path, content, },);
  await chmod(path, 0o755,);
}

/**
 Lays out the repository: update tasks, two crates, one TypeScript package, and a first commit.

 @param root - absolute path of the repository

 @param environment - process environment for `git`

 @example
 ```ts
 await layOutRepository({ root: '/r', environment: process.env, },);
 ```
 */
export async function layOutRepository({
  root,
  environment,
}: {
  readonly root: string;
  readonly environment: NodeJS.ProcessEnv;
},): Promise<void> {
  await Promise.all([
    put({ path: join(root, 'mise.toml',), content: ROOT_MISE, },),
    put({ path: join(root, '.gitignore',), content: '.cache/\n', },),
    put({ path: join(root, 'toolchain.lock.json',), content: '{"tools":{"rust":{"version":"nightly-1"}}}\n', },),
    put({ path: join(root, 'steps', 'move.ts',), content: MOVE_SCRIPT, },),
    put({ path: join(root, 'steps', 'clippy.ts',), content: CLIPPY_SCRIPT, },),
    put({
      path: join(root, 'package', 'a', 'broken-crate', 'mise.toml',),
      content: '[tasks."lint:clippy"]\nrun = "node ../../../steps/clippy.ts E0433"\n',
    },),
    put({ path: join(root, 'package', 'a', 'broken-crate', 'Cargo.lock',), content: CARGO_LOCK, },),
    put({
      path: join(root, 'package', 'c', 'red-crate', 'mise.toml',),
      content: '[tasks."lint:clippy"]\nrun = "node ../../../steps/clippy.ts E0277 always"\n',
    },),
    put({ path: join(root, 'package', 'c', 'red-crate', 'Cargo.lock',), content: CARGO_LOCK, },),
    put({
      path: join(root, 'package', 'e', 'ts-package', 'mise.toml',),
      content: '[tasks."lint:types"]\nrun = "node -e 0"\n',
    },),
  ],);
  /**
   Everything the first commit holds, named one by one because the repository's Git wrapper rejects bulk staging.
   */
  const committed = [
    'mise.toml',
    '.gitignore',
    'toolchain.lock.json',
    'steps',
    'package',
  ];
  for (const args of [
    ['init', '--quiet',],
    ['add', '--', ...committed,],
    ['commit', '--quiet', '--message', 'fixture: first commit', '--', ...committed,],
  ]) {
    /**
     How the `git` step ended.
     */
    const step = await run({
      program: 'git',
      args,
      directory: root,
      environment,
    },);
    if (step.exitCode !== 0)
      throw new Error(`git ${args.join(' ',)} exited with status ${step.exitCode}: ${step.stderr}`,);
  }
}

/**
 Resolves the directory of an executable that is on the caller's `PATH`.

 @param name - executable name

 @returns the real directory that holds it

 @throws Error when it is not on `PATH`

 @example
 ```ts
 const miseDirectory = await directoryOf('mise',);
 ```
 */
export async function directoryOf(name: string,): Promise<string> {
  for (const directory of (process.env.PATH ?? '').split(delimiter,)) {
    /**
     Metadata of the candidate, absent when the folder has no such file.
     */
    const metadata = await stat(join(directory, name,), { throwIfNoEntry: false, },);
    if (metadata?.isFile() === true)
      return dirname(await realpath(join(directory, name,),),);
  }
  throw new Error(`${name} is required on PATH for the whole-path test`,);
}

/**
 Runs the real daily update over the disposable repository with the stand-in `gh`.

 @param options - what the stand-in `gh` answers

 @returns what the run printed, which commits it made, what `gh` was asked, and what it kept for later

 @throws Error when `mise` or `node` is not on `PATH` or a step of the fixture fails

 @example
 ```ts
 const result = await runWholePath({ ghExitCode: 0, },);
 ```
 */
export async function runWholePath({
  ghExitCode,
}: WholePathOptions,): Promise<WholePathResult> {
  await using disposable = await mkdtempDisposable(join(scratchRoot(), 'daily-update-whole-path-',),);
  /**
   Canonical path of the fixture root.
   */
  const base = await realpath(disposable.path,);
  /**
   Repository, home and the folder of stand-in programs.
   */
  const root = join(base, 'repository',);
  const bin = join(base, 'bin',);
  const ghLog = join(base, 'gh-calls.jsonl',);
  const notifyLog = join(base, 'notifications.jsonl',);
  const globalLock = join(base, 'global', 'toolchain.lock.json',);
  /**
   Environment of every process: the stand-ins first on `PATH`, fixture-owned homes, no ambient credentials.
   */
  const environment: NodeJS.ProcessEnv = {
    PATH: [
      bin,
      '/usr/bin',
      '/bin',
    ].join(delimiter,),
    HOME: join(base, 'home',),
    XDG_CONFIG_HOME: join(base, 'config',),
    XDG_CACHE_HOME: join(base, 'cache',),
    XDG_DATA_HOME: join(base, 'data',),
    XDG_STATE_HOME: join(base, 'state',),
    GIT_CONFIG_GLOBAL: join(base, 'gitconfig',),
    GIT_AUTHOR_NAME: 'Fixture',
    GIT_AUTHOR_EMAIL: 'fixture@example.invalid',
    GIT_COMMITTER_NAME: 'Fixture',
    GIT_COMMITTER_EMAIL: 'fixture@example.invalid',
    MISE_TRUSTED_CONFIG_PATHS: base,
    MISE_CEILING_PATHS: dirname(base,),
    MISE_TASK_RUN_AUTO_INSTALL: 'false',
    MISE_YES: '1',
    NO_COLOR: '1',
    // A session bus address that leads nowhere, so even a notifier that was not the stand-in could show nothing.
    DBUS_SESSION_BUS_ADDRESS: `unix:path=${join(base, 'no-session-bus',)}`,
    FIXTURE_NOTIFY_LOG: notifyLog,
    FIXTURE_GH_LOG: ghLog,
    FIXTURE_GH_EXIT: String(ghExitCode,),
    FIXTURE_GLOBAL_LOCK: globalLock,
  };
  // Only the program folders the fixture names are on `PATH`: the folder that holds `mise` also holds the repository's
  // Git wrapper, which needs a user service manager that a disposable repository must not start.
  await mkdir(bin, { recursive: true, },);
  await Promise.all([
    symlink(await realpath(join(await directoryOf('mise',), 'mise',),), join(bin, 'mise',),),
    symlink(process.execPath, join(bin, 'node',),),
  ],);
  await Promise.all([
    putExecutable({ directory: bin, name: 'gh', content: FAKE_GH, },),
    putExecutable({ directory: bin, name: 'notify-send', content: FAKE_NOTIFY_SEND, },),
    putExecutable({ directory: bin, name: 'cargo-shadow', content: FAKE_CARGO_SHADOW, },),
    putExecutable({ directory: bin, name: 'pnpm', content: FAKE_PNPM, },),
    put({ path: globalLock, content: '{"tools":{"rust":{"version":"nightly-1"}}}\n', },),
    put({ path: ghLog, content: '', },),
    put({ path: notifyLog, content: '', },),
  ],);
  await layOutRepository({ root, environment, },);
  /**
   The daily update, run the way the task runs it.
   */
  const update = await run({
    program: process.execPath,
    args: [
      join(SOURCE_DIRECTORY, 'daily-update.ts',),
      '--repository',
      root,
      '--global-lock',
      globalLock,
    ],
    directory: root,
    environment,
  },);
  /**
   The repository's commit subjects.
   */
  const log = await run({
    program: 'git',
    args: [
      'log',
      '--format=%s',
    ],
    directory: root,
    environment,
  },);
  /**
   Text of the pending file, empty when absent.
   */
  const pendingText = await readFile(join(root, '.cache', 'toolchain-daily-update-pending-issues.json',), 'utf8',)
    .catch(function absent() {
      return '';
    },);
  return {
    exitCode: update.exitCode,
    journal: update.stderr,
    ghCalls: (await readFile(ghLog, 'utf8',))
      .split('\n',)
      .filter(function nonEmpty(line,) {
        return line !== '';
      },)
      .map(function parse(line,): RecordedGhCall {
        return JSON.parse(line,) as RecordedGhCall;
      },),
    notifications: (await readFile(notifyLog, 'utf8',))
      .split('\n',)
      .filter(function nonEmpty(line,) {
        return line !== '';
      },)
      .map(function parse(line,): RecordedNotification {
        return JSON.parse(line,) as RecordedNotification;
      },),
    commitSubjects: log.stdout.split('\n',)
      .filter(function nonEmpty(line,) {
        return line !== '';
      },),
    pendingText,
  };
}
