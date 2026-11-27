/**
 What the whole-path run lays out and runs: stand-in programs, the files of the disposable repository, and the
 helpers that write files and run programs to their end.

 @module
 */

import {
  chmod,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { run, } from './daily-update-whole-path-programs-fixture.ts';
import {
  CARGO_LOCK,
  CHECK_SCRIPT,
  CLIPPY_SCRIPT,
  crateManifest,
  MOVE_SCRIPT,
  PNPM_LOCK,
  PNPM_WORKSPACE,
  ROOT_MISE,
  RUST_CATALOG,
  SYNC_SCRIPT,
  TIGHTEN_CARGO_SCRIPT,
} from './daily-update-whole-path-scripts-fixture.ts';

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
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    path,
    content,
  );
}

/**
 Permission bits of a stand-in program: the owner may read, write and run it, others may read and run it.
 */
const EXECUTABLE_MODE = 0o755;

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
  const path = join(
    directory,
    name,
  );
  await put({
    path,
    content,
  },);
  await chmod(
    path,
    EXECUTABLE_MODE,
  );
}

/**
 Runs one `git` step in the repository.

 @param args - arguments after `git`

 @param root - absolute path of the repository

 @param environment - process environment for `git`

 @throws Error when `git` exits with a nonzero status

 @example
 ```ts
 await runGitStep({ args: ['init',], root: '/r', environment, },);
 ```
 */
export async function runGitStep({
  args,
  root,
  environment,
}: {
  readonly args: readonly string[];
  readonly root: string;
  readonly environment: Readonly<Record<string, string>>;
},): Promise<void> {
  /**
   How the `git` step ended.
   */
  const result = await run({
    program: 'git',
    args,
    directory: root,
    environment,
  },);
  if (result.exitCode !== 0)
    throw new Error(`git ${args.join(' ',)} exited with status ${result.exitCode}: ${result.stderr}`,);
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
  readonly environment: Readonly<Record<string, string>>;
},): Promise<void> {
  await Promise.all([
    put({
      path: join(
        root,
        'mise.toml',
      ),
      content: ROOT_MISE,
    },),
    put({
      path: join(
        root,
        '.gitignore',
      ),
      content: '.cache/\n',
    },),
    put({
      path: join(
        root,
        'toolchain.lock.json',
      ),
      content: '{"tools":{"rust":{"version":"nightly-1"}}}\n',
    },),
    put({
      path: join(
        root,
        'pnpm-workspace.yaml',
      ),
      content: PNPM_WORKSPACE,
    },),
    put({
      path: join(
        root,
        'pnpm-lock.yaml',
      ),
      content: PNPM_LOCK,
    },),
    put({
      path: join(
        root,
        'steps',
        'move.ts',
      ),
      content: MOVE_SCRIPT,
    },),
    put({
      path: join(
        root,
        'steps',
        'clippy.ts',
      ),
      content: CLIPPY_SCRIPT,
    },),
    put({
      path: join(
        root,
        'steps',
        'sync.ts',
      ),
      content: SYNC_SCRIPT,
    },),
    put({
      path: join(
        root,
        'steps',
        'check.ts',
      ),
      content: CHECK_SCRIPT,
    },),
    put({
      path: join(
        root,
        'steps',
        'tighten-cargo.ts',
      ),
      content: TIGHTEN_CARGO_SCRIPT,
    },),
    // The Rust catalog and the manifests `sync:files` renders from it: one crate whose lock the raise leaves behind and
    // one whose lock follows it. Both carry the exact pin the catalog ties to the slint release.
    put({
      path: join(
        root,
        'rust-catalog.toml',
      ),
      content: RUST_CATALOG,
    },),
    put({
      path: join(
        root,
        'package',
        'a',
        'broken-crate',
        'Cargo.toml',
      ),
      content: crateManifest({ name: 'broken-crate', },),
    },),
    put({
      path: join(
        root,
        'package',
        'c',
        'red-crate',
        'Cargo.toml',
      ),
      content: crateManifest({ name: 'red-crate', },),
    },),
    put({
      path: join(
        root,
        'package',
        'a',
        'broken-crate',
        'mise.toml',
      ),
      content: '[tasks."lint:clippy"]\nrun = "node ../../../steps/clippy.ts E0433"\n',
    },),
    put({
      path: join(
        root,
        'package',
        'a',
        'broken-crate',
        'Cargo.lock',
      ),
      content: CARGO_LOCK,
    },),
    put({
      path: join(
        root,
        'package',
        'c',
        'red-crate',
        'mise.toml',
      ),
      content: '[tasks."lint:clippy"]\nrun = "node ../../../steps/clippy.ts E0277 always"\n',
    },),
    put({
      path: join(
        root,
        'package',
        'c',
        'red-crate',
        'Cargo.lock',
      ),
      content: CARGO_LOCK,
    },),
    put({
      path: join(
        root,
        'package',
        'e',
        'ts-package',
        'mise.toml',
      ),
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
    'pnpm-workspace.yaml',
    'pnpm-lock.yaml',
    'rust-catalog.toml',
    'steps',
    'package',
  ];
  // Each `git` step needs the one before it, so they run one after another and the first failure ends the layout.
  await runGitStep({
    args: [
      'init',
      '--quiet',
    ],
    root,
    environment,
  },);
  await runGitStep({
    args: [
      'add',
      '--',
      ...committed,
    ],
    root,
    environment,
  },);
  await runGitStep({
    args: [
      'commit',
      '--quiet',
      '--message',
      'fixture: first commit',
      '--',
      ...committed,
    ],
    root,
    environment,
  },);
}
