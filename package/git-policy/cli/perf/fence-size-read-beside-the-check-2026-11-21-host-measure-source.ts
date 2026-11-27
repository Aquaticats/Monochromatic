/**
 The fixture the 2026-11-21 host measurement adds into: this repository's tracked tree, committed, with no policy running
 over it, and the collection pass that brings its loose objects back to the shape the recorded rounds started from.

 Every Git command here runs with a home of the fixture's own and no system configuration, so a measurement never reads or
 writes the account's Git configuration.
 */
import {
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { run, } from './fence-size-read-beside-the-check-2026-11-21-commands.ts';
import {
  DESTS,
  hostRuntimeDirectory,
  PLAIN_HOME,
  SOURCE,
  TAR,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-paths.ts';

/**
 The environment every fixture-building Git run takes: its own home and no system configuration.

 @returns the environment of one fixture-building Git run

 @example
 ```ts
 fixtureEnvironment(); // { ...process.env, HOME: <the fixture's home>, ... }
 ```
 */
export function fixtureEnvironment(): NodeJS.ProcessEnv {
  return {
    ...process.env,
    HOME: PLAIN_HOME,
    XDG_RUNTIME_DIR: hostRuntimeDirectory,
    GIT_CONFIG_NOSYSTEM: '1',
  };
}

/**
 Removes the fixture source and the destinations left from an earlier run.

 @example
 ```ts
 await unbuild();
 ```
 */
export async function unbuild(): Promise<void> {
  await rm(
    SOURCE,
    {
      recursive: true,
      force: true,
    },
  );
  await rm(
    DESTS,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    SOURCE,
    { recursive: true, },
  );
}

/**
 What the source's `git status --porcelain` said, one entry per line.

 @param env - the environment the fixture's Git runs take

 @returns the entries that hold something
 */
async function sourceStatus({ env, }: { readonly env: NodeJS.ProcessEnv; }): Promise<readonly string[]> {
  return (await run({
    args: [
      'git',
      'status',
      '--porcelain',
    ],
    cwd: SOURCE,
    env,
  },)).trim()
    .split('\n',)
    .filter(function holdsSomething(line) {
      return line !== '';
    },);
}

/**
 Prints what `git count-objects -v` says about the fixture source.

 @param env - the environment the fixture's Git runs take
 */
async function printObjectCounts({ env, }: { readonly env: NodeJS.ProcessEnv; }): Promise<void> {
  console.log((await run({
    args: [
      'git',
      'count-objects',
      '-v',
    ],
    cwd: SOURCE,
    env,
  },)).trim(),);
}

/**
 Builds the fixture source: this repository's tracked tree, committed, with no policy to run over it.

 @example
 ```ts
 await fixture();
 ```
 */
export async function fixture(): Promise<void> {
  await mkdir(
    PLAIN_HOME,
    { recursive: true, },
  );
  await unbuild();
  await mkdir(
    DESTS,
    { recursive: true, },
  );
  await run({
    args: [
      'tar',
      '--extract',
      '--file',
      TAR,
      '--directory',
      SOURCE,
    ],
  },);
  /**
   The source's attributes file, whose LFS rules this fixture has no store for.
   */
  const attributes = join(
    SOURCE,
    '.gitattributes',
  );
  /**
   What the attributes file holds without the LFS rules.
   */
  const kept = (await readFile(
    attributes,
    'utf8',
  ))
    .split('\n',)
    .filter(function withoutLfs(line) {
      return !line.includes('filter=lfs',);
    },)
    .join('\n',);
  await writeFile(
    attributes,
    kept,
  );
  // No policy runs on the fixture's own commits, so a measurement never pays a scanner over this tree.
  await writeFile(
    join(
      SOURCE,
      'monochromatic-git-policy-cli.config.jsonc',
    ),
    '{\n  "policies": {}\n}\n',
  );
  /**
   The environment of every fixture-building Git run.
   */
  const env = fixtureEnvironment();
  await run({
    args: [
      'git',
      'init',
      '--quiet',
      '--initial-branch=main',
    ],
    cwd: SOURCE,
    env,
  },);
  await run({
    args: [
      'git',
      'config',
      'user.name',
      'Fixture',
    ],
    cwd: SOURCE,
    env,
  },);
  await run({
    args: [
      'git',
      'config',
      'user.email',
      'fixture@example.invalid',
    ],
    cwd: SOURCE,
    env,
  },);
  await run({
    args: [
      'git',
      'config',
      'commit.gpgsign',
      'false',
    ],
    cwd: SOURCE,
    env,
  },);
  await run({
    args: [
      'git',
      'add',
      '--all',
      '--no-enforce-bulk-add',
    ],
    cwd: SOURCE,
    env,
  },);
  /**
   The tree the fixture's one commit holds.
   */
  const tree = (await run({
    args: [
      'git',
      'write-tree',
    ],
    cwd: SOURCE,
    env,
  },)).trim();
  /**
   The fixture's one commit.
   */
  const commit = (await run({
    args: [
      'git',
      'commit-tree',
      tree,
      '-m',
      'fixture',
    ],
    cwd: SOURCE,
    env,
  },)).trim();
  await run({
    args: [
      'git',
      'update-ref',
      'refs/heads/main',
      commit,
    ],
    cwd: SOURCE,
    env,
  },);
  await run({
    args: [
      'git',
      'update-index',
      '--really-refresh',
    ],
    cwd: SOURCE,
    env,
  },);
  /**
   The source's staged changes, which a fresh index can show for a moment.
   */
  const status = await sourceStatus({ env, },);
  /**
   How many paths the fixture's commit holds.
   */
  const tracked = (await run({
    args: [
      'git',
      'ls-files',
    ],
    cwd: SOURCE,
    env,
  },)).trim()
    .split('\n',)
    .length;
  console.log(JSON.stringify({
    phase: 'fixture',
    tracked,
    statusLines: status.length,
    commit,
  },),);
  await printObjectCounts({ env, },);
}

/**
 Collects the fixture's loose objects, so the next round starts from the shape its adds were recorded from.

 @example
 ```ts
 await gc();
 ```
 */
export async function gc(): Promise<void> {
  /**
   The environment of every fixture-building Git run.
   */
  const env = fixtureEnvironment();
  await run({
    args: [
      'git',
      'gc',
      '--quiet',
    ],
    cwd: SOURCE,
    env,
  },);
  await run({
    args: [
      'git',
      'update-index',
      '--really-refresh',
    ],
    cwd: SOURCE,
    env,
  },);
  /**
   The source's staged changes, which a fresh index can show for a moment.
   */
  const status = await sourceStatus({ env, },);
  console.log(JSON.stringify({
    phase: 'gc',
    statusLines: status.length,
  },),);
  await printObjectCounts({ env, },);
}
