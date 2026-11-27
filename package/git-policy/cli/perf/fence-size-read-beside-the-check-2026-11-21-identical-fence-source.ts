/**
 The source the 2026-11-21 identical-fence control adds into: a small repository whose `.gitattributes` gives a text rule,
 so the size proof decides the paths of the worktree the add writes.

 The fixture is built from nothing but files this module writes, so a later run of the same script measures a tree of the same
 shape without a tar of anything.
 */
import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { run, } from './fence-size-read-beside-the-check-2026-11-21-commands.ts';
import {
  carriageReturnEvery,
  filesPerKind,
  HOME,
  RUNTIME,
  ROOT,
  settleSeconds,
  SOURCE,
} from './fence-size-read-beside-the-check-2026-11-21-identical-fence-paths.ts';

/**
 The environment every fixture-building command takes: a home and a runtime directory of the fixture's own, and no system
 configuration.

 @returns the environment of one command of this control

 @example
 ```ts
 fixtureEnvironment(); // { ...process.env, HOME: <the fixture's home>, ... }
 ```
 */
export function fixtureEnvironment(): NodeJS.ProcessEnv {
  return {
    ...process.env,
    HOME,
    XDG_RUNTIME_DIR: RUNTIME,
    GIT_CONFIG_NOSYSTEM: '1',
  };
}

/**
 Writes the fixture's files, once each, at the same time: no file reads another, and the fixture is not timed.
 */
async function writeTheFixtureFiles(): Promise<void> {
  await Promise.all([...Array.from({ length: filesPerKind, },)
    .keys(),].map(async function writeTheFilesOf(index) {
    // A carriage return in some files, so the text rule and the size proof both matter.
      await writeFile(
        join(
          SOURCE,
          `file-${String(index,)}.txt`,
        ),
        (index % carriageReturnEvery) === 0 ? `line\r\n${String(index,)}\r\n` : `line\n${String(index,)}\n`,
      );
      await writeFile(
        join(
          SOURCE,
          `blob-${String(index,)}.bin`,
        ),
        Buffer.from([
          index,
          0,
          255,
          index,
        ],),
      );
    },));
}

/**
 Builds the fixture: a source whose attributes give a text rule, committed, with nothing staged.

 @example
 ```ts
 await buildFixture();
 ```
 */
export async function buildFixture(): Promise<void> {
  await rm(
    ROOT,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    HOME,
    { recursive: true, },
  );
  await mkdir(
    RUNTIME,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  await mkdir(
    SOURCE,
    { recursive: true, },
  );
  await writeFile(
    join(
      SOURCE,
      '.gitattributes',
    ),
    '*.txt text eol=lf\n*.bin -text\n',
  );
  await writeTheFixtureFiles();
  await writeFile(
    join(
      SOURCE,
      'plain.txt',
    ),
    'no rule touches this one\n',
  );
  /**
   The environment of every fixture-building command.
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
  const treeOfTheCommit = (await run({
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
      treeOfTheCommit,
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
  // A fresh index's stat fields are racy for a moment, and the check refuses the source until they settle.
  await run({ args: [
    'sleep',
    String(settleSeconds,),
  ], },);
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
   What the source's own status says, which is what a fixture that stayed dirty prints.
   */
  const status = (await run({
    args: [
      'git',
      'status',
      '--porcelain',
    ],
    cwd: SOURCE,
    env,
  },)).trim();
  if (status !== '') console.log(`fixture stays dirty: ${status}`,);
}
