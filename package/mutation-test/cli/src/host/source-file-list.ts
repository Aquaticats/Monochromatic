/**
 Host-side list of the repository files a shard container copies into its
 work tree.
 
 Copying the whole checkout fails once ignored build output and nested
 agent worktrees push it past the container's work-tree tmpfs, so the host
 asks git which files are source: tracked files plus untracked files that
 are not ignored (uncommitted edits stay under test).
 
 @example
 ```ts
 const list = await listSourceFiles({ repoRoot });
 ```
 */

import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for the source file list.
 */
const l = tagged({ tag: 'mutation-test', },);

/**
 Computes the NUL-separated list of source files under a repository root.
 
 Tracked files deleted in the working tree stay listed, because git reports
 the index; the copy step tolerates missing entries. NUL separation keeps
 paths holding spaces or newlines intact. Paths that are not valid UTF-8 are
 lossy in the decoded output and end up missing from the copy.
 
 @param options - Repository root to list.
 
 @returns Repository-root-relative paths, each followed by one NUL byte;
 empty when the repository holds no source files.
 
 @throws SubprocessError when `git ls-files` fails, for example when the root
 is not a git repository.
 
 @example
 ```ts
 const list = await listSourceFiles({ repoRoot: '/repo' });
 ```
 */
export async function listSourceFiles(options: {
  readonly repoRoot: string;
},): Promise<string> {
  /**
   Logger scoped to this listing.
   */
  const rl = tagged({
    tag: listSourceFiles.name,
    l,
  },);
  rl.debug(`listing source files under ${options.repoRoot}`,);
  /**
   Subprocess result holding git's NUL-separated paths.
   */
  const result = await spawn(
    'git',
    [
      '-C',
      options.repoRoot,
      'ls-files',
      '-z',
      '--cached',
      '--others',
      '--exclude-standard',
    ],
    { cwd: options.repoRoot, },
  );
  rl.info(`${String(result.stdout
    .split('\0',)
    .length - 1,)} source files listed under ${options.repoRoot}`,);
  return result.stdout;
}
