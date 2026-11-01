/**
 Read-only real-Git queries the observation gatherer uses.

 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { contentOf, } from './ledger-fixture.ts';
import type { ContentState, } from './invariant-model-fixture.ts';
import {
  runBytes,
  runProcess,
} from './process-fixture.ts';
import {
  realGit,
  type ScenarioRepository,
} from './repository-fixture.ts';

//region Types

/**
 One history commit.
 */
export type HistoryCommit = Readonly<{
  /**
   Commit ID.
   */
  oid: string;
  /**
   First parent, absent for a root commit.
   */
  parent?: string;
  /**
   Full message.
   */
  message: string;
}>;

//endregion Types

//region Git reads

/**
 Reads the bytes of a path in a commit.

 @param repository - scenario repository

 @param commit - commit ID

 @param path - repository path

 @returns bytes, absent when the tree lacks the path

 @example
 ```ts
 await treeBytes({ repository, commit: 'HEAD', path: 'a.txt' });
 ```
 */
export async function treeBytes({
  repository,
  commit,
  path,
}: Readonly<{
  repository: ScenarioRepository;
  commit: string;
  path: string;
}>,): Promise<Buffer | 'absent'> {
  /**
   `ls-tree` record naming the blob.
   */
  const listed = await realGit({
    repository,
    args: [
      'ls-tree',
      '-z',
      commit,
      '--',
      path,
    ],
  },);
  /**
   Blob ID from `<mode> <type> <oid>\t<path>`.
   */
  const oid = listed.split('\t',)[0]
    ?.split(' ',)[2];
  if ((listed === '') || (oid === undefined))
    return 'absent';
  return await runBytes({
    command: repository.realGit,
    args: [
      'cat-file',
      'blob',
      oid,
    ],
    cwd: repository.worktree,
    env: repository.realEnv,
  },);
}

/**
 Converts a tree read to a content state.

 @param bytes - bytes or `absent`

 @returns content state

 @example
 ```ts
 stateOf('absent'); // => { state: 'absent' }
 ```
 */
export function stateOf(bytes: Buffer | 'absent',): ContentState {
  return bytes === 'absent' ? contentOf() : contentOf(bytes,);
}

/**
 Reads every commit reachable from local branches.

 @param repository - scenario repository

 @returns commits, each once

 @example
 ```ts
 await readHistory(repository);
 ```
 */
export async function readHistory(repository: ScenarioRepository,): Promise<readonly HistoryCommit[]> {
  /**
   Records separated by RS, fields by NUL.
   */
  const text = await realGit({
    repository,
    args: [
      'log',
      '--branches',
      '--format=%H%x00%P%x00%B%x1e',
    ],
  },);
  return text
    .split('\u001E',)
    .map(function trimRecord(record,) {
      return record.startsWith('\n',) ? record.slice(1,) : record;
    },)
    .filter(function nonEmpty(record,) {
      return record !== '';
    },)
    .map(function parseRecord(record,): HistoryCommit {
      /**
       Commit, parent list, and message fields.
       */
      const [oid = '', parents = '', message = '',] = record.split('\0',);
      /**
       First parent ID.
       */
      const [parent,] = parents.split(' ',);
      return {
        oid,
        ...((parent === undefined) || (parent === '') ? {} : { parent, }),
        message,
      };
    },);
}

/**
 Reports whether one commit is an ancestor of, or equal to, a ref.

 @param repository - scenario repository

 @param gitDir - repository to ask, the scenario worktree when absent

 @param oid - commit ID

 @param ref - target ref

 @returns ancestry

 @example
 ```ts
 await isAncestor({ repository, oid, ref: 'refs/heads/main' });
 ```
 */
export async function isAncestor({
  repository,
  gitDir,
  oid,
  ref,
}: Readonly<{
  repository: ScenarioRepository;
  gitDir?: string;
  oid: string;
  ref: string;
}>,): Promise<boolean> {
  /**
   Settled `merge-base --is-ancestor`.
   */
  const outcome = await runProcess({
    command: repository.realGit,
    args: [
      ...(gitDir === undefined ? [] : [`--git-dir=${gitDir}`,]),
      'merge-base',
      '--is-ancestor',
      oid,
      ref,
    ],
    cwd: repository.worktree,
    env: repository.realEnv,
  },);
  return outcome.exitCode === 0;
}

/**
 Lists paths a commit changes against its first parent.

 @param repository - scenario repository

 @param commit - history commit

 @returns changed paths without rename detection

 @example
 ```ts
 await changedPaths({ repository, commit });
 ```
 */
export async function changedPaths({
  repository,
  commit,
}: Readonly<{
  repository: ScenarioRepository;
  commit: HistoryCommit;
}>,): Promise<readonly string[]> {
  /**
   NUL-separated path list.
   */
  const text = await realGit({
    repository,
    args: commit.parent === undefined
      ? [
        'diff-tree',
        '--root',
        '--no-commit-id',
        '--name-only',
        '--no-renames',
        '-r',
        '-z',
        commit.oid,
      ]
      : [
        'diff-tree',
        '--no-commit-id',
        '--name-only',
        '--no-renames',
        '-r',
        '-z',
        commit.parent,
        commit.oid,
      ],
  },);
  return text.split('\0',)
    .filter(function nonEmpty(path,) {
      return path !== '';
    },);
}

/**
 Lists the first-parent chain from a commit back to an older commit, inclusive of both ends.

 @param repository - scenario repository

 @param newest - newest commit

 @param oldest - oldest commit

 @returns commit IDs newest first, bounded to keep merge probing finite

 @example
 ```ts
 await firstParentChain({ repository, newest: parent, oldest: headBefore });
 ```
 */
export async function firstParentChain({
  repository,
  newest,
  oldest,
}: Readonly<{
  repository: ScenarioRepository;
  newest: string;
  oldest: string;
}>,): Promise<readonly string[]> {
  /**
   Commits strictly between the ends plus the newest.
   */
  const text = await realGit({
    repository,
    args: [
      'rev-list',
      '--first-parent',
      '--max-count=64',
      `${oldest}..${newest}`,
    ],
  },);
  return [
    ...text.split('\n',)
      .filter(function nonEmpty(line,) {
        return line !== '';
      },),
    oldest,
  ];
}

/**
 Three-way merges captured bytes onto the landed parent's bytes from one base.

 @param repository - scenario repository

 @param current - landed parent's bytes

 @param base - candidate preparation base's bytes

 @param other - captured bytes

 @returns merged bytes, or `conflict`

 @example
 ```ts
 await mergeOnto({ repository, current, base, other });
 ```
 */
export async function mergeOnto({
  repository,
  current,
  base,
  other,
}: Readonly<{
  repository: ScenarioRepository;
  current: Buffer;
  base: Buffer;
  other: Buffer;
}>,): Promise<Buffer | 'conflict'> {
  /**
   Scratch directory for the three inputs.
   */
  const directory = await mkdtemp(join(
    repository.root,
    'merge-',
  ),);
  /**
   Input paths.
   */
  const files = {
    current: join(
      directory,
      'current',
    ),
    base: join(
      directory,
      'base',
    ),
    other: join(
      directory,
      'other',
    ),
  };
  await Promise.all([
    writeFile(
      files.current,
      current,
    ),
    writeFile(
      files.base,
      base,
    ),
    writeFile(
      files.other,
      other,
    ),
  ],);
  /**
   Merge result; exit 0 means clean.
   */
  const merged = await runProcess({
    command: repository.realGit,
    args: [
      'merge-file',
      '-p',
      '--quiet',
      files.current,
      files.base,
      files.other,
    ],
    cwd: directory,
    env: repository.realEnv,
  },);
  /**
   Exact merged bytes; text capture is safe because merge-file refuses binary input.
   */
  const bytes = Buffer.from(merged.stdout,);
  await rm(
    directory,
    {
      recursive: true,
      force: true,
    },
  );
  return merged.exitCode === 0 ? bytes : 'conflict';
}

//endregion Git reads
