/**
 Fixture repositories that have the shape of the person's own main worktree, and the page cache of a fixture's files.

 The benchmark in `native-perf.ts` measures a commit at the size the person commits at: a reflink copy of a prepared
 repository with tens of thousands of tracked files, a multi-megabyte index and a pack of hundreds of thousands of objects,
 with the hooks and the refreshed index such a worktree has.
 */
import { execFile } from 'node:child_process';
import {
  copyFile,
  readdir,
} from 'node:fs/promises';
import { promisify } from 'node:util';
import { join } from 'node:path';

import {
  environment,
  repository,
  setup,
} from './native-perf-git.ts';
import { runCommand } from './native-verification-process.ts';

/**
 The real Git every fixture setup command runs.
 */
const REAL_GIT = '/usr/bin/git';

/**
 Names of the hook files a main-shaped fixture takes from the repository whose hooks it imitates; the sample hooks that
 `git init` writes are never copied.
 */
const SHAPED_HOOKS = [
  'post-checkout',
  'post-commit',
  'post-merge',
  'pre-push',
] as const;

/**
 One repository that is a reflink copy of a prepared repository, so a fixture has the shape of the person's own main
 worktree: the same tracked files, the same index size, the same object store and refs.

 The copy shares disk blocks with the prepared repository until a file changes, which makes four fixtures cost metadata
 only. The prepared repository is never changed.

 @param root - scratch root that holds `bin` and `home`

 @param name - directory name inside the scratch root

 @param shape - prepared repository to copy, a disposable clone without any remote

 @param hooksFrom - Git directory whose `post-checkout`, `post-commit`, `post-merge` and `pre-push` hooks the copy gets,
   or absent for a copy without hooks

 @returns the repository directory

 @throws Error when the prepared repository has a remote, because a fixture a commit could push from is not disposable

 @example
 ```ts
 const directory = await shapedRepository({ root, name: 'direct-repo', shape });
 ```
 */
export async function shapedRepository({
  root,
  name,
  shape,
  hooksFrom,
}: {
  readonly root: string;
  readonly name: string;
  readonly shape: string;
  readonly hooksFrom?: string;
}): Promise<string> {
  /**
   Directory of the new repository inside the scratch root.
   */
  const directory = join(
    root,
    name,
  );
  /**
   Remotes of the prepared repository, which must be none.
   */
  const remotes = await runCommand({
    command: REAL_GIT,
    args: [
      'remote',
    ],
    capture: true,
    cwd: shape,
    env: environment({
      root,
      wrapped: false,
    }),
  });
  if (remotes.stdout
    .trim()
    !== '') {
    throw new Error(`the shape repository ${shape} has remotes (${remotes.stdout
      .trim()}); remove them before it is copied`);
  }
  await runCommand({
    command: 'cp',
    args: [
      '--archive',
      '--reflink=always',
      shape,
      directory,
    ],
    capture: true,
  });
  await setup({
    root,
    directory,
    arguments_: [
      'config',
      'core.untrackedcache',
      'true',
    ],
  });
  // A copy has new inodes and change times, so every index entry reads as changed until Git refreshes the index; the person's
  // index has been refreshed by their own `git status` and editors long before a commit, and a fixture that was not would
  // make every commit re-read all the files (measured: about four seconds of the wrapped signed commit on 20,740 files).
  // Refreshed here, outside any timing, with the untracked cache the person's configuration keeps.
  await setup({
    root,
    directory,
    arguments_: [
      'update-index',
      '--refresh',
      '--really-refresh',
    ],
  });
  await setup({
    root,
    directory,
    arguments_: [
      'status',
      '--porcelain',
    ],
  });
  if (hooksFrom !== undefined) {
    /**
     Hook files of the imitated repository, of which only the policy hooks are copied.
     */
    const present = new Set(await readdir(hooksFrom));
    await Promise.all(SHAPED_HOOKS
      .filter(function exists(hook) {
        return present.has(hook);
      })
      .map(function copyHook(hook) {
        return copyFile(
          join(
            hooksFrom,
            hook,
          ),
          join(
            directory,
            '.git',
            'hooks',
            hook,
          ),
        );
      }));
  }
  return directory;
}

/**
 Drop the cached pages of a repository's pack files, pack indexes, index and packed refs, so the next command reads them from
 the disk as it does when the host's memory pressure has evicted them.

 This is the cache state of the fixture's own files and no load on the host: `dd` with `iflag=nocache` and `count=0` only asks
 the kernel to forget clean pages of the one file (`posix_fadvise` with `POSIX_FADV_DONTNEED`), reads nothing and writes
 nothing. It is applied to both arms of a pair in the same way, so the difference of a pair is the reading the wrapper adds.

 @param directory - repository whose files are evicted

 @example
 ```ts
 await evictRepositoryCache(directory);
 ```
 */
export async function evictRepositoryCache(directory: string): Promise<void> {
  /**
   The pack directory of the repository.
   */
  const packs = join(
    directory,
    '.git',
    'objects',
    'pack',
  );
  /**
   Every file whose pages are dropped.
   */
  const files = [
    ...(await readdir(packs)).map(function inPacks(name) {
      return join(
        packs,
        name,
      );
    }),
    join(
      directory,
      '.git',
      'index',
    ),
    join(
      directory,
      '.git',
      'packed-refs',
    ),
  ];
  // One file at a time: the calls are cheap, and a failed one (a file that does not exist) is not worth the others' time.
  await Array.fromAsync(
    files,
    async function evict(file) {
      await runCommand({
        command: 'dd',
        args: [
          `if=${file}`,
          'iflag=nocache',
          'count=0',
          'status=none',
        ],
        capture: true,
        allowFailure: true,
      });
    },
  );
}

/**
 Make one fixture repository: the thirty-file repository of the forwarded-command scenarios, or a reflink copy of the prepared
 shape repository when one is given.

 @param root - scratch root that holds `bin` and `home`

 @param name - directory name inside the scratch root

 @param shape - prepared repository to copy, or absent for the thirty-file repository

 @param hooksFrom - Git directory whose policy hooks a shaped copy gets

 @returns the repository directory

 @example
 ```ts
 const directory = await makeFixture({ root, name: 'direct-repo' });
 ```
 */
export function makeFixture({
  root,
  name,
  shape,
  hooksFrom,
}: {
  readonly root: string;
  readonly name: string;
  readonly shape?: string;
  readonly hooksFrom?: string;
}): Promise<string> {
  return shape === undefined
    ? repository({
      root,
      name,
    })
    : shapedRepository({
      root,
      name,
      shape,
      ...(hooksFrom === undefined ? {} : { hooksFrom, }),
    });
}

/**
 Runs a command without a shell and resolves with its standard output.
 */
const run = promisify(execFile);

/**
 The first tracked text files of a repository, the files a commit of a chosen size edits.

 Only Markdown, TypeScript and Rust sources count: they are text, so an appended line is an ordinary change, and a
 repository holds many of them.

 @param directory - repository whose index names the files

 @param count - how many files to return

 @returns repository-relative paths in index order

 @throws Error when the repository tracks fewer such files than asked for

 @example
 ```ts
 const names = await trackedTextFiles({ directory, count: 50 });
 ```
 */
export async function trackedTextFiles({
  directory,
  count,
}: {
  readonly directory: string;
  readonly count: number;
}): Promise<readonly string[]> {
  /**
   Names the index holds that end in a text-source extension, NUL separated.
   */
  const { stdout, } = await run(
    REAL_GIT,
    [
      '-C',
      directory,
      'ls-files',
      '-z',
      '--',
      '*.md',
      '*.ts',
      '*.rs',
      '*.txt',
    ],
    { maxBuffer: 256 * 1024 * 1024, },
  );
  /**
   Every such name.
   */
  const names = stdout.split('\0')
    .filter(Boolean);
  if (names.length < count) {
    throw new Error(`the repository tracks ${String(names.length)} text files, fewer than the ${String(count)} asked for`);
  }
  return names.slice(
    0,
    count,
  );
}
