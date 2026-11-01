import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  lockFileName,
  manifestFileName,
} from './constants.ts';
import {
  buildCargoMetadataArguments,
  changedWatchedPaths,
  isWatchedPath,
  mergeWatchSets,
  parseCargoMetadata,
  type WatchSet,
} from './watch-paths.ts';

/**
 Absolute root of the fixture repository.
 */
const root = '/work';

/**
 A watch set with a file and a directory whose name is a prefix of a sibling's.
 */
const watch: WatchSet = {
  files: ['package/x/Cargo.toml'],
  directories: ['package/a'],
};

/**
 Build the answer of `cargo metadata` for a set of packages.

 @param packages - name, source and absolute directory of each package; a null source is a path package
 @param workspaceRoot - absolute workspace root
 @returns the JSON text of the answer
 */
function metadata({
  packages,
  workspaceRoot,
}: {
  readonly packages: readonly { readonly name: string; readonly source?: string; readonly directory: string }[];
  readonly workspaceRoot: string;
}): string {
  return JSON.stringify({
    packages: packages.map(function entryOf(entry) {
      return {
        name: entry.name,
        // A path package has no source, which cargo metadata writes as null.
        source: entry.source ?? null,
        manifest_path: join(entry.directory, manifestFileName)
      };
    }),
    workspace_root: workspaceRoot,
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: isWatchedPath.name,
      children: [
        it({
          name: 'matches a watched file only by its whole path',
          fn: async () => {
            expect(isWatchedPath({ path: 'package/x/Cargo.toml', watch })).toBe(true);
            expect(isWatchedPath({ path: `package/x/${manifestFileName}.bak`, watch })).toBe(false);
            expect(isWatchedPath({ path: 'package/x/Cargo.tom', watch })).toBe(false);
            expect(isWatchedPath({ path: 'package/x', watch })).toBe(false);
          },
        }),
        it({
          name: 'matches a watched directory, what lies below it and nothing that merely starts with its name',
          fn: async () => {
            const [directory = ''] = watch.directories;
            expect(isWatchedPath({ path: directory, watch })).toBe(true);
            expect(isWatchedPath({ path: `${directory}/src/lib.rs`, watch })).toBe(true);
            expect(isWatchedPath({ path: `${directory}/a/b/c`, watch })).toBe(true);
            expect(isWatchedPath({ path: `${directory}b/src/lib.rs`, watch })).toBe(false);
            expect(isWatchedPath({ path: `${directory}.fuzz/Cargo.toml`, watch })).toBe(false);
            expect(isWatchedPath({ path: `x/${directory}/lib.rs`, watch })).toBe(false);
          },
        }),
        it({
          name: 'matches nothing in an empty watch set',
          fn: async () => {
            expect(isWatchedPath({ path: 'a', watch: { files: [], directories: [] } })).toBe(false);
          },
        }),
      ],
    }),
    describe({
      name: changedWatchedPaths.name,
      children: [
        it({
          name: 'keeps the watched paths among the changed ones, each once, in the order given',
          fn: async () => {
            expect(changedWatchedPaths({
              changed: ['docs/a.md', 'package/a/src/lib.rs', 'package/ab/lib.rs', 'package/x/Cargo.toml', 'package/a/src/lib.rs'],
              watch,
            })).toEqual(['package/a/src/lib.rs', 'package/x/Cargo.toml']);
          },
        }),
        it({
          name: 'finds nothing when no changed path is watched',
          fn: async () => {
            expect(changedWatchedPaths({ changed: ['docs/a.md', 'package/ab/lib.rs'], watch })).toEqual([]);
            expect(changedWatchedPaths({ changed: [], watch })).toEqual([]);
          },
        }),
        it({
          name: 'counts a renamed file by both of its paths, so a file moved out of a watched directory is found',
          fn: async () => {
            const oldPath = 'package/a/src/old.rs';
            const newPath = 'elsewhere/new.rs';
            expect(changedWatchedPaths({ changed: [newPath, oldPath], watch })).toEqual([oldPath]);
          },
        }),
      ],
    }),
    describe({
      name: mergeWatchSets.name,
      children: [
        it({
          name: 'joins the sets, each path once, in order of first appearance',
          fn: async () => {
            expect(mergeWatchSets([
              { files: ['f1', 'f2'], directories: ['d1'] },
              { files: ['f2', 'f3'], directories: ['d2', 'd1'] },
            ])).toEqual({ files: ['f1', 'f2', 'f3'], directories: ['d1', 'd2'] });
            expect(mergeWatchSets([])).toEqual({ files: [], directories: [] });
          },
        }),
      ],
    }),
    describe({
      name: buildCargoMetadataArguments.name,
      children: [
        it({
          name: 'reads one manifest as JSON without letting cargo rewrite a lockfile',
          fn: async () => {
            const manifest = `package/a/${manifestFileName}`;
            const args = buildCargoMetadataArguments(manifest);
            expect(args).toContain('--locked');
            expect(args).toContain(`--manifest-path=${manifest}`);
            expect(args[0]).toBe('metadata');
          },
        }),
        it({
          name: 'keeps a manifest path that starts with a hyphen inside its option',
          fn: async () => {
            const args = buildCargoMetadataArguments('-x/Cargo.toml');
            expect(args).toContain('--manifest-path=-x/Cargo.toml');
            expect(args).not.toContain('-x/Cargo.toml');
          },
        }),
      ],
    }),
    describe({
      name: parseCargoMetadata.name,
      children: [
        it({
          name: 'watches the directory of every path package and the manifest and lockfile of the workspace root',
          fn: async () => {
            const result = parseCargoMetadata({
              root,
              body: metadata({
                workspaceRoot: join(root, 'package/a'),
                packages: [
                  { name: 'a', directory: join(root, 'package/a') },
                  { name: 'dep', directory: join(root, 'package/dep') },
                  { name: 'serde', source: 'registry+https://github.com/rust-lang/crates.io-index', directory: '/home/.cargo/serde' },
                ],
              }),
            });
            expect(result).toEqual({
              found: true,
              value: {
                files: [`package/a/${manifestFileName}`, `package/a/${lockFileName}`],
                directories: ['package/a', 'package/dep'],
              },
            });
          },
        }),
        it({
          name: 'leaves out every package that comes from a registry or a git source',
          fn: async () => {
            const result = parseCargoMetadata({
              root,
              body: metadata({
                workspaceRoot: join(root, 'package/a'),
                packages: [
                  { name: 'a', directory: join(root, 'package/a') },
                  { name: 'g', source: 'git+https://example.invalid/g', directory: join(root, 'package/g') },
                ],
              }),
            });
            expect(result.found ? result.value.directories : []).toEqual(['package/a']);
          },
        }),
        it({
          name: 'leaves out a path package outside the repository, which a comparison of the repository cannot see',
          fn: async () => {
            const result = parseCargoMetadata({
              root,
              body: metadata({
                workspaceRoot: join(root, 'package/a'),
                packages: [
                  { name: 'a', directory: join(root, 'package/a') },
                  { name: 'far', directory: '/elsewhere/far' },
                  { name: 'sibling', directory: `${root}-sibling/crate` },
                ],
              }),
            });
            expect(result.found ? result.value.directories : []).toEqual(['package/a']);
          },
        }),
        it({
          name: 'lists a directory once when two packages share it, and keeps the repository root as a root manifest',
          fn: async () => {
            const result = parseCargoMetadata({
              root,
              body: metadata({
                workspaceRoot: root,
                packages: [
                  { name: 'a', directory: join(root, 'package/a') },
                  { name: 'a2', directory: join(root, 'package/a') },
                ],
              }),
            });
            expect(result).toEqual({
              found: true,
              value: { files: [manifestFileName, lockFileName], directories: ['package/a'] },
            });
          },
        }),
        it({
          name: 'ignores a workspace root outside the repository',
          fn: async () => {
            const result = parseCargoMetadata({
              root,
              body: metadata({
                workspaceRoot: '/elsewhere',
                packages: [{ name: 'a', directory: join(root, 'package/a') }],
              }),
            });
            expect(result.found ? result.value.files : ['not found']).toEqual([]);
          },
        }),
        it({
          name: 'refuses a path package at the repository root, whose directory is every file',
          fn: async () => {
            const result = parseCargoMetadata({
              root,
              body: metadata({ workspaceRoot: root, packages: [{ name: 'rootpkg', directory: root }] }),
            });
            expect(result.found).toBe(false);
            expect(result.found ? '' : result.reason).toContain('rootpkg');
          },
        }),
        ...[
          ['text that is not JSON', 'not json'],
          ['an answer that is not an object', '[]'],
          ['an answer without packages', '{"workspace_root":"/work"}'],
          ['an answer without a workspace root', '{"packages":[]}'],
          ['a path package without a manifest path', '{"packages":[{"name":"a","source":null}],"workspace_root":"/work"}'],
        ].map(function malformedCase([label = '', body = '']) {
          return it({
            name: `reports a miss for ${label}`,
            fn: async () => {
              expect(parseCargoMetadata({ root, body }).found).toBe(false);
            },
          });
        }),
      ],
    }),
  ],
});
