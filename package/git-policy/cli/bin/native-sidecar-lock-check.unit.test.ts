import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  basename,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeProblem,
  lockedPackages,
  miseTaskOf,
  refusalLine,
  sidecarDirectories,
  staleDependencies,
  type SidecarLockProblem,
} from './native-sidecar-lock-check.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Create the scratch parent and one disposable tree under it.

 @param name - prefix of the tree's directory name

 @returns the tree, removed when the caller's scope ends

 @example
 ```ts
 await using tree = await scratchTree({ name: 'sidecar-locks-test-' });
 ```
 */
async function scratchTree({ name, }: {
  readonly name: string;
},) {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  return mkdtempDisposable(join(
    scratchParent,
    name,
  ));
}

/**
 Write a package directory holding named siblings, of which the named crates hold a manifest.

 @param root - tree receiving the package

 @param names - the package directory and its siblings

 @param crates - the siblings that hold a `Cargo.toml`

 @returns the package directory
 */
async function packageWith({
  root,
  names,
  crates,
}: {
  readonly root: string;
  readonly names: readonly string[];
  readonly crates: readonly string[];
},): Promise<string> {
  await Promise.all(names.map(function created(name) {
    return mkdir(join(
      root,
      name,
    ));
  }));
  await Promise.all(crates.map(function manifest(name) {
    return writeFile(
      join(
        root,
        name,
        'Cargo.toml',
      ),
      '[package]\nname = "sidecar"\nversion = "0.0.0"\n',
    );
  }));
  return join(
    root,
    'cli',
  );
}

await describe({
  name: 'the sidecar lock check',
  children: [
    describe({
      name: lockedPackages.name,
      children: [
        it({
          name: 'sorts the packages by name and version and counts a repeated pair once',
          fn: async () => {
            const document = {
              packages: [
                { name: 'serde', version: '1.0.200' },
                { name: 'clap', version: '4.5.0' },
                { name: 'serde', version: '1.0.200' },
              ],
            };
            expect(lockedPackages({ document, })).toEqual([
              { name: 'clap', version: '4.5.0' },
              { name: 'serde', version: '1.0.200' },
            ]);
          },
        }),
        it({
          name: 'refuses a document that is not the shape cargo documents',
          fn: async () => {
            expect(() => lockedPackages({ document: { packages: 'serde', }, })).toThrow();
            expect(() => lockedPackages({ document: null, })).toThrow();
            expect(() => lockedPackages({ document: { packages: [{ name: 'serde', },], }, })).toThrow();
          },
        }),
      ],
    }),
    describe({
      name: staleDependencies.name,
      children: [
        it({
          name: 'finds nothing when the sidecar locks every package at the same version',
          fn: async () => {
            const subject = [
              { name: 'clap', version: '4.5.0', },
              { name: 'serde', version: '1.0.200', },
            ];
            expect(staleDependencies({
              subject,
              sidecar: [...subject, { name: 'libfuzzer-sys', version: '0.4.13', },],
            })).toEqual([]);
          },
        }),
        it({
          name: 'names both versions when the sidecar locks an older one',
          fn: async () => {
            const stale = staleDependencies({
              subject: [{ name: 'serde', version: '1.0.200', },],
              sidecar: [{ name: 'serde', version: '1.0.199', },],
            });
            expect(stale).toEqual([{ name: 'serde', expected: '1.0.200', found: ['1.0.199'], },]);
          },
        }),
        it({
          name: 'names a package the sidecar does not lock at all, which is a dependency this package gained',
          fn: async () => {
            const stale = staleDependencies({
              subject: [{ name: 'zstd', version: '0.13.0', },],
              sidecar: [{ name: 'serde', version: '1.0.199', },],
            });
            expect(stale).toEqual([{ name: 'zstd', expected: '0.13.0', },]);
          },
        }),
        it({
          name: 'accepts a second version of the same crate beside the wanted one, which a sidecar dependency may need',
          fn: async () => {
            const stale = staleDependencies({
              subject: [{ name: 'libc', version: '0.2.191', },],
              sidecar: [{ name: 'libc', version: '0.2.180', }, { name: 'libc', version: '0.2.191', },],
            });
            expect(stale).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: sidecarDirectories.name,
      children: [
        it({
          name: 'names every sibling of the package that holds a manifest, and nothing else',
          fn: async () => {
            await using tree = await scratchTree({ name: 'sidecar-locks-test-', });
            const packageDirectory = await packageWith({
              root: tree.path,
              names: ['cli', 'cli.fuzz', 'cli.bench', 'cli.notes', 'other'],
              crates: ['cli.fuzz', 'cli.bench'],
            });
            const sidecars = await sidecarDirectories({ packageDirectory, });
            expect(sidecars.map(function name(directory) {
              return basename(directory);
            })).toEqual(['cli.bench', 'cli.fuzz']);
          },
        }),
        it({
          name: 'gives no sidecar for a package whose neighbours hold no manifest',
          fn: async () => {
            await using tree = await scratchTree({ name: 'sidecar-locks-empty-test-', });
            const packageDirectory = await packageWith({
              root: tree.path,
              names: ['cli', 'cli.notes'],
              crates: [],
            });
            expect(await sidecarDirectories({ packageDirectory, })).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: describeProblem.name,
      children: [
        it({
          name: 'names the sidecar, both versions and the task that records them',
          fn: async () => {
            const problem: SidecarLockProblem = {
              sidecar: 'cli.fuzz',
              directory: '/repo/package/git-policy/cli.fuzz',
              refusal: '',
              stale: [{ name: 'serde', expected: '1.0.200', found: ['1.0.199'], },],
            };
            const described = describeProblem({
              problem,
              packageDirectory: '/repo/package/git-policy/cli',
            });
            expect(described).toContain('cli.fuzz: its Cargo.lock no longer agrees with cli/Cargo.lock');
            expect(described).toContain('serde: cli locks 1.0.200, where it locks 1.0.199');
            expect(described).toContain('`mise run //package/git-policy/cli.fuzz:lock:members`');
          },
        }),
        it({
          name: 'names cargo\'s refusal when cargo would not read the lock at all',
          fn: async () => {
            const problem: SidecarLockProblem = {
              sidecar: 'cli.fuzz',
              directory: '/repo/package/git-policy/cli.fuzz',
              refusal: 'error: the lock file needs to be updated but --locked was passed',
              stale: [],
            };
            const described = describeProblem({
              problem,
              packageDirectory: '/repo/package/git-policy/cli',
            });
            expect(described).toContain('cargo refused the lock: error: the lock file needs to be updated but --locked was passed');
            expect(described).toContain('`mise run //package/git-policy/cli.fuzz:lock:members`');
          },
        }),
      ],
    }),
    describe({
      name: refusalLine.name,
      children: [
        it({
          name: 'names cargo\'s error after the warnings and lock waits it prints first',
          fn: async () => {
            expect(refusalLine({ stderr: 'warning: waiting\nBlocking waiting for file lock on package cache\nerror: the lock file needs to be updated but --locked was passed\n' })).toBe('error: the lock file needs to be updated but --locked was passed');
          },
        }),
        it({
          name: 'falls back to the first line cargo wrote when it named no error',
          fn: async () => {
            expect(refusalLine({ stderr: '\n something went wrong\n' })).toBe('something went wrong');
            expect(refusalLine({ stderr: '' })).toBe('');
          },
        }),
      ],
    }),
    describe({
      name: miseTaskOf.name,
      children: [
        it({
          name: 'names the task from the repository-relative package path',
          fn: async () => {
            expect(miseTaskOf({ directory: '/repo/package/git-policy/cli.fuzz', })).toBe('mise run //package/git-policy/cli.fuzz');
          },
        }),
      ],
    }),
  ],
});
