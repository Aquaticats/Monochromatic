/**
 The scratch root of a latency run: the copy of the wrapper that runs, the empty home and the tools the repository hooks need.
 */
import {
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  symlink,
} from 'node:fs/promises';
import {
  basename,
  join,
  resolve,
} from 'node:path';

/**
 Permission bits of the copied wrapper binary: executable by everyone, writable by its owner.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Make the scratch root of one run, outside the repository so no fixture is ever committed.

 @param scratchParent - private directory that holds the root

 @param binary - release wrapper to copy, so the measurement never races a rebuild of the original

 @param tools - executables the hooks of the fixtures need on PATH, such as `git-lfs`, linked into the root's `tools`

 @returns the root: `bin` holds the wrapper copy and a `git` link to it, `home` is empty, `tools` holds the links

 @example
 ```ts
 const root = await makeRoot({ scratchParent, binary, tools: [] });
 ```
 */
export async function makeRoot({
  scratchParent,
  binary,
  tools,
}: {
  readonly scratchParent: string;
  readonly binary: string;
  readonly tools: readonly string[];
}): Promise<string> {
  /**
   The root of this run.
   */
  const root = await mkdtemp(join(
    scratchParent,
    'native-perf-',
  ));
  await Promise.all([
    'bin',
    'home',
    'tools',
  ].map(function makeDirectory(name) {
    return mkdir(join(
      root,
      name,
    ));
  }));
  await Promise.all(tools.map(function linkTool(tool) {
    return symlink(
      resolve(tool),
      join(
        root,
        'tools',
        basename(tool),
      ),
    );
  }));
  /**
   Copy of the release wrapper.
   */
  const wrapperBinary = join(
    root,
    'bin',
    'cli-git-native',
  );
  await copyFile(
    binary,
    wrapperBinary,
  );
  await chmod(
    wrapperBinary,
    EXECUTABLE_MODE,
  );
  await symlink(
    wrapperBinary,
    join(
      root,
      'bin',
      'git',
    ),
  );
  return root;
}
