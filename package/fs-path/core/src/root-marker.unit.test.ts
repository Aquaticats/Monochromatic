/**
 Marker semantics as data tables over the in-memory adapter.

 Every case declares a tree, a start directory, and the root it expects
 (or none), and runs `findRoot` through the same seam the runtime
 adapters fill. Nothing here touches a disk; the real-filesystem cases
 live in `root-discovery.unit.test.ts`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMemoryRootFilesystem,
  directoryNamed,
  fileNamed,
  findRoot,
  GIT_REPOSITORY,
  type MemoryTree,
  MISE_MONOREPO,
  packageNamed,
  PNPM_WORKSPACE,
  type RootMarker,
  RootNotFoundError,
} from '@monochromatic-dev/fs-path-core';

//region Table runner

/**
 Expected outcome marker for a walk that must reject.
 */
const NONE = 'none';

/**
 One row of a marker table.
 */
type MarkerCase = {
  /**
   Test name.
   */
  readonly name: string;

  /**
   Tree the walk runs over.
   */
  readonly tree: MemoryTree;

  /**
   Start directory; defaults to the deepest fixture directory.
   */
  readonly cwd?: string;

  /**
   Root the walk must return, or {@link NONE} when it must reject.
   */
  readonly expected: string;
};

/**
 Deepest directory of every fixture, so each walk climbs two levels
 before reaching `/repo`.
 */
const DEEP = '/repo/a/b';

/**
 Turns one table row into a test.

 @param marker - marker under test

 @param row - tree, start directory, and expectation

 @returns test descriptor

 @example
 ```ts
 children: MISE_CASES.map(function toTest(row) { return markerCase({ marker: MISE_MONOREPO, row }); })
 ```
 */
function markerCase({
  marker,
  row,
}: {
  readonly marker: RootMarker;
  readonly row: MarkerCase;
},): ReturnType<typeof it> {
  return it({
    name: row.name,
    fn: async () => {
      /**
       Adapter over the row's tree.
       */
      const fs = createMemoryRootFilesystem(row.tree,);
      /**
       Start directory for this row.
       */
      const cwd = row.cwd ?? DEEP;
      if (row.expected === NONE) {
        /**
         Rejection captured for inspection.
         */
        let caught: unknown;
        try {
          await findRoot({
            cwd,
            fs,
            marker,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(RootNotFoundError,);
        expect((caught as RootNotFoundError).marker,).toBe(marker.name,);
        expect((caught as RootNotFoundError).startDir,).toBe(cwd,);
        return;
      }
      expect(await findRoot({
        cwd,
        fs,
        marker,
      },),).toBe(row.expected,);
    },
  },);
}

//endregion Table runner

//region mise monorepo

/**
 Tree with one `mise.toml` at `/repo` holding the given text.

 @param text - full `mise.toml` content

 @returns tree with the deep directory present

 @example
 ```ts
 miseTree('[monorepo]\n');
 ```
 */
function miseTree(text: string,): MemoryTree {
  return {
    directories: [DEEP,],
    files: { '/repo/mise.toml': text, },
  };
}

/**
 `[monorepo]` header shapes the marker must accept or refuse.
 */
const MISE_CASES: readonly MarkerCase[] = [
  {
    expected: '/repo',
    name: 'matches a [monorepo] table between other tables',
    tree: miseTree('[tools]\nnode = "24"\n\n[monorepo]\nroot = true\n',),
  },
  {
    expected: '/repo',
    name: 'matches [monorepo] on the first line',
    tree: miseTree('[monorepo]\nroot = true\n',),
  },
  {
    expected: '/repo',
    name: 'matches [monorepo] on the last line without a trailing newline',
    tree: miseTree('[tools]\nnode = "24"\n\n[monorepo]',),
  },
  {
    expected: '/repo',
    name: 'matches through CRLF line endings',
    tree: miseTree('[tools]\r\nnode = "24"\r\n\r\n[monorepo]\r\nroot = true\r\n',),
  },
  {
    expected: '/repo',
    name: 'matches [monorepo] followed by a comment',
    tree: miseTree('[monorepo] # workspace root\nroot = true\n',),
  },
  {
    expected: '/repo',
    name: 'matches [monorepo] followed by a comment without a space',
    tree: miseTree('[monorepo]# workspace root\n',),
  },
  {
    expected: '/repo',
    name: 'matches [monorepo] with trailing spaces',
    tree: miseTree('[monorepo]   \n',),
  },
  {
    expected: '/repo',
    name: 'matches an indented [monorepo] header',
    tree: miseTree('  [monorepo]\n',),
  },
  {
    expected: NONE,
    name: 'refuses [monorepo] quoted inside a value',
    tree: miseTree('description = "[monorepo] later"\n',),
  },
  {
    expected: NONE,
    name: 'refuses [monorepo] followed by anything but a comment',
    tree: miseTree('[monorepo] extra\n',),
  },
  {
    expected: NONE,
    name: 'refuses a table whose name merely starts with monorepo',
    tree: miseTree('[monorepository]\n',),
  },
  {
    expected: NONE,
    name: 'refuses a mise.toml without the table',
    tree: miseTree('[tools]\nnode = "latest"\n',),
  },
  {
    expected: NONE,
    name: 'refuses an empty mise.toml',
    tree: miseTree('',),
  },
  {
    expected: NONE,
    name: 'rejects when no ancestor holds a mise.toml',
    tree: { directories: [DEEP,], },
  },
  {
    expected: NONE,
    name: 'rejects when mise.toml is a directory',
    tree: { directories: [DEEP, '/repo/mise.toml',], },
  },
  {
    expected: '/repo',
    name: 'skips a nearer mise.toml without the table',
    tree: {
      directories: [DEEP,],
      files: {
        '/repo/a/mise.toml': '[tools]\nnode = "24"\n',
        '/repo/mise.toml': '[monorepo]\n',
      },
    },
  },
  {
    expected: '/repo/a',
    name: 'returns the nearest ancestor with the table',
    tree: {
      directories: [DEEP,],
      files: {
        '/repo/a/mise.toml': '[monorepo]\n',
        '/repo/mise.toml': '[monorepo]\n',
      },
    },
  },
  {
    expected: '/repo',
    name: 'accepts the start directory itself',
    cwd: '/repo',
    tree: miseTree('[monorepo]\n',),
  },
];

//endregion mise monorepo

//region Git repository

/**
 Tree with a valid Git administrative directory at the given path plus
 any extra entries.

 @param gitDirectory - administrative directory path

 @param head - HEAD text

 @param extra - entries merged over the valid directory

 @returns tree with the deep directory present

 @example
 ```ts
 gitTree({ gitDirectory: '/repo/.git' });
 ```
 */
function gitTree({
  gitDirectory = '/repo/.git',
  head = 'ref: refs/heads/main\n',
  extra = {},
}: {
  readonly gitDirectory?: string;
  readonly head?: string;
  readonly extra?: MemoryTree;
} = {},): MemoryTree {
  return {
    directories: [
      DEEP,
      `${gitDirectory}/objects`,
      `${gitDirectory}/refs`,
      ...(extra.directories ?? []),
    ],
    files: {
      [`${gitDirectory}/HEAD`]: head,
      ...extra.files,
    },
    links: { ...extra.links, },
  };
}

/**
 Forty hexadecimal characters, a SHA-1 object id.
 */
const SHA_ONE = 'a'.repeat(40,);

/**
 Sixty-four hexadecimal characters, a SHA-256 object id.
 */
const SHA_TWO = 'b'.repeat(64,);

/**
 Gitfile payload one character over Git's accepted size.
 */
const OVERSIZED_GITFILE = `gitdir: ${'x'.repeat(1_048_577,)}`;

/**
 Git marker shapes the marker must accept or refuse.
 */
const GIT_CASES: readonly MarkerCase[] = [
  {
    expected: '/repo',
    name: 'matches a .git directory with HEAD, objects, and refs',
    tree: gitTree(),
  },
  {
    expected: '/repo',
    name: 'accepts a detached SHA-1 HEAD',
    tree: gitTree({ head: `${SHA_ONE}\n`, },),
  },
  {
    expected: '/repo',
    name: 'accepts a detached SHA-256 HEAD',
    tree: gitTree({ head: `${SHA_TWO}\n`, },),
  },
  {
    expected: '/repo',
    name: 'accepts a symbolic HEAD with CRLF',
    tree: gitTree({ head: 'ref: refs/heads/main\r\n', },),
  },
  {
    expected: '/repo',
    name: 'accepts a symbolic HEAD without a space after ref:',
    tree: gitTree({ head: 'ref:refs/heads/main\n', },),
  },
  {
    expected: NONE,
    name: 'refuses a symbolic HEAD outside the refs namespace',
    tree: gitTree({ head: 'ref: heads/main\n', },),
  },
  {
    expected: NONE,
    name: 'refuses an empty HEAD',
    tree: gitTree({ head: '', },),
  },
  {
    expected: NONE,
    name: 'refuses a detached HEAD of the wrong length',
    tree: gitTree({ head: `${'a'.repeat(39,)}\n`, },),
  },
  {
    expected: NONE,
    name: 'refuses a detached HEAD with a non-hexadecimal character',
    tree: gitTree({ head: `${'a'.repeat(39,)}g\n`, },),
  },
  {
    expected: '/repo',
    name: 'accepts a dangling symbolic-link HEAD into the refs namespace',
    tree: {
      directories: [DEEP, '/repo/.git/objects', '/repo/.git/refs',],
      links: { '/repo/.git/HEAD': 'refs/heads/missing', },
    },
  },
  {
    expected: NONE,
    name: 'refuses a symbolic-link HEAD outside the refs namespace',
    tree: {
      directories: [DEEP, '/repo/.git/objects', '/repo/.git/refs',],
      links: { '/repo/.git/HEAD': 'elsewhere/main', },
    },
  },
  {
    expected: NONE,
    name: 'refuses a HEAD that is a directory',
    tree: { directories: [DEEP, '/repo/.git/objects', '/repo/.git/refs', '/repo/.git/HEAD',], },
  },
  {
    expected: NONE,
    name: 'refuses a .git directory without HEAD',
    tree: { directories: [DEEP, '/repo/.git/objects', '/repo/.git/refs',], },
  },
  {
    expected: NONE,
    name: 'refuses a .git directory without objects',
    tree: {
      directories: [DEEP, '/repo/.git/refs',],
      files: { '/repo/.git/HEAD': 'ref: refs/heads/main\n', },
    },
  },
  {
    expected: NONE,
    name: 'refuses a .git directory without refs',
    tree: {
      directories: [DEEP, '/repo/.git/objects',],
      files: { '/repo/.git/HEAD': 'ref: refs/heads/main\n', },
    },
  },
  {
    expected: NONE,
    name: 'refuses an empty .git directory',
    tree: { directories: [DEEP, '/repo/.git',], },
  },
  {
    expected: NONE,
    name: 'rejects when no ancestor holds .git',
    tree: { directories: [DEEP,], },
  },
  {
    expected: '/repo',
    name: 'follows a gitfile with a relative target',
    tree: gitTree({
      extra: { files: { '/repo/.git': 'gitdir: .git-target\n', }, },
      gitDirectory: '/repo/.git-target',
    },),
  },
  {
    expected: '/repo',
    name: 'follows a gitfile with an absolute target',
    tree: gitTree({
      extra: { files: { '/repo/.git': 'gitdir: /store/gitdir\n', }, },
      gitDirectory: '/store/gitdir',
    },),
  },
  {
    expected: '/repo',
    name: 'follows a gitfile with CRLF',
    tree: gitTree({
      extra: { files: { '/repo/.git': 'gitdir: .git-target\r\n', }, },
      gitDirectory: '/repo/.git-target',
    },),
  },
  {
    expected: NONE,
    name: 'refuses a gitfile with the wrong prefix',
    tree: gitTree({
      extra: { files: { '/repo/.git': 'not-a-gitdir: .git-target\n', }, },
      gitDirectory: '/repo/.git-target',
    },),
  },
  {
    expected: NONE,
    name: 'refuses a gitfile whose target contains NUL',
    tree: gitTree({
      extra: { files: { '/repo/.git': 'gitdir: .git-target\0suffix\n', }, },
      gitDirectory: '/repo/.git-target',
    },),
  },
  {
    expected: NONE,
    name: 'refuses a gitfile with an empty target',
    tree: gitTree({
      extra: { files: { '/repo/.git': 'gitdir: \n', }, },
      gitDirectory: '/repo/.git-target',
    },),
  },
  {
    expected: NONE,
    name: 'refuses an oversized gitfile',
    tree: gitTree({
      extra: { files: { '/repo/.git': OVERSIZED_GITFILE, }, },
      gitDirectory: '/repo/.git-target',
    },),
  },
  {
    expected: NONE,
    name: 'refuses a gitfile whose target is missing',
    tree: {
      directories: [DEEP,],
      files: { '/repo/.git': 'gitdir: missing-target\n', },
    },
  },
  {
    expected: NONE,
    name: 'refuses a gitfile whose target is a file',
    tree: {
      directories: [DEEP,],
      files: {
        '/repo/.git': 'gitdir: target\n',
        '/repo/target': '',
      },
    },
  },
  {
    expected: '/repo',
    name: 'follows a linked-worktree gitfile through a relative commondir',
    tree: {
      directories: [DEEP, '/repo/.git-common/objects', '/repo/.git-common/refs',],
      files: {
        '/repo/.git': 'gitdir: .git-worktree\n',
        '/repo/.git-worktree/commondir': '../.git-common\n',
        '/repo/.git-worktree/HEAD': 'ref: refs/heads/main\n',
      },
    },
  },
  {
    expected: '/repo',
    name: 'follows a linked-worktree gitfile through an absolute commondir',
    tree: {
      directories: [DEEP, '/store/common/objects', '/store/common/refs',],
      files: {
        '/repo/.git': 'gitdir: .git-worktree\n',
        '/repo/.git-worktree/commondir': '/store/common\n',
        '/repo/.git-worktree/HEAD': 'ref: refs/heads/main\n',
      },
    },
  },
  {
    expected: NONE,
    name: 'refuses a commondir that is a directory',
    tree: {
      directories: [DEEP, '/repo/.git/commondir', '/repo/.git/objects', '/repo/.git/refs',],
      files: { '/repo/.git/HEAD': 'ref: refs/heads/main\n', },
    },
  },
  {
    expected: NONE,
    name: 'refuses an empty commondir',
    tree: gitTree({ extra: { files: { '/repo/.git/commondir': '\n', }, }, },),
  },
  {
    expected: NONE,
    name: 'refuses a commondir containing NUL',
    tree: gitTree({ extra: { files: { '/repo/.git/commondir': '../x\0y\n', }, }, },),
  },
  {
    expected: NONE,
    name: 'refuses a commondir whose target lacks objects and refs',
    tree: gitTree({ extra: { directories: ['/repo/empty-common',], files: { '/repo/.git/commondir': '../empty-common\n', }, }, },),
  },
  {
    expected: '/repo',
    name: 'skips an invalid nearer marker and finds the valid outer root',
    tree: gitTree({ extra: { directories: ['/repo/a/.git',], }, },),
  },
  {
    expected: '/repo/a',
    name: 'returns the nearest valid marker',
    tree: gitTree({ extra: gitTree({ gitDirectory: '/repo/a/.git', },), },),
  },
  {
    expected: '/repo',
    name: 'follows a .git symbolic link to a valid directory',
    tree: gitTree({
      extra: { links: { '/repo/.git': '/store/gitdir', }, },
      gitDirectory: '/store/gitdir',
    },),
  },
];

//endregion Git repository

//region pnpm workspace and factories

/**
 File and directory marker shapes.
 */
const PNPM_CASES: readonly MarkerCase[] = [
  {
    expected: '/repo',
    name: 'matches a pnpm-workspace.yaml file',
    tree: {
      directories: [DEEP,],
      files: { '/repo/pnpm-workspace.yaml': 'packages:\n  - pkg\n', },
    },
  },
  {
    expected: NONE,
    name: 'refuses a directory named pnpm-workspace.yaml',
    tree: { directories: [DEEP, '/repo/pnpm-workspace.yaml',], },
  },
  {
    expected: NONE,
    name: 'rejects when no ancestor holds the manifest',
    tree: { directories: [DEEP,], },
  },
  {
    expected: '/repo/a',
    name: 'returns the nearest manifest',
    tree: {
      directories: [DEEP,],
      files: {
        '/repo/a/pnpm-workspace.yaml': '',
        '/repo/pnpm-workspace.yaml': '',
      },
    },
  },
];

/**
 Shapes for `fileNamed('deno.json')`.
 */
const FILE_NAMED_CASES: readonly MarkerCase[] = [
  {
    expected: '/repo',
    name: 'matches the named file',
    tree: {
      directories: [DEEP,],
      files: { '/repo/deno.json': '{}', },
    },
  },
  {
    expected: '/repo',
    name: 'matches the named file through a symbolic link',
    tree: {
      directories: [DEEP,],
      files: { '/store/deno.json': '{}', },
      links: { '/repo/deno.json': '/store/deno.json', },
    },
  },
  {
    expected: NONE,
    name: 'refuses a directory with the file name',
    tree: { directories: [DEEP, '/repo/deno.json',], },
  },
];

/**
 Shapes for `directoryNamed('node_modules')`.
 */
const DIRECTORY_NAMED_CASES: readonly MarkerCase[] = [
  {
    expected: '/repo',
    name: 'matches the named directory',
    tree: { directories: [DEEP, '/repo/node_modules',], },
  },
  {
    expected: '/repo/a',
    name: 'matches the nearest named directory',
    tree: { directories: [DEEP, '/repo/a/node_modules', '/repo/node_modules',], },
  },
  {
    expected: NONE,
    name: 'refuses a file with the directory name',
    tree: {
      directories: [DEEP,],
      files: { '/repo/node_modules': '', },
    },
  },
];

/**
 Package name looked for by the `packageNamed` cases.
 */
const PACKAGE = '@scope/pkg';

/**
 Shapes for `packageNamed('@scope/pkg')`.
 */
const PACKAGE_NAMED_CASES: readonly MarkerCase[] = [
  {
    expected: '/repo/a',
    name: 'matches the manifest declaring the name',
    tree: {
      directories: [DEEP,],
      files: { '/repo/a/package.json': `{ "name": "${PACKAGE}" }`, },
    },
  },
  {
    expected: '/repo',
    name: 'walks past a manifest with another name',
    tree: {
      directories: [DEEP,],
      files: {
        '/repo/a/package.json': '{ "name": "@scope/other" }',
        '/repo/package.json': `{ "name": "${PACKAGE}" }`,
      },
    },
  },
  {
    expected: '/repo',
    name: 'walks past a manifest without a name',
    tree: {
      directories: [DEEP,],
      files: {
        '/repo/a/package.json': '{ "private": true }',
        '/repo/package.json': `{ "name": "${PACKAGE}" }`,
      },
    },
  },
  {
    expected: '/repo',
    name: 'walks past an unparsable manifest',
    tree: {
      directories: [DEEP,],
      files: {
        '/repo/a/package.json': '{ not json',
        '/repo/package.json': `{ "name": "${PACKAGE}" }`,
      },
    },
  },
  {
    expected: NONE,
    name: 'rejects when no ancestor declares the name',
    tree: {
      directories: [DEEP,],
      files: { '/repo/package.json': '{ "name": "@scope/other" }', },
    },
  },
  {
    expected: NONE,
    name: 'rejects when package.json is a directory',
    tree: { directories: [DEEP, '/repo/package.json',], },
  },
];

//endregion pnpm workspace and factories

await describe({
  name: '',
  children: [
    describe({
      name: 'MISE_MONOREPO',
      children: MISE_CASES.map(function toTest(row,): ReturnType<typeof it> {
        return markerCase({
          marker: MISE_MONOREPO,
          row,
        },);
      },),
    },),
    describe({
      name: 'GIT_REPOSITORY',
      children: GIT_CASES.map(function toTest(row,): ReturnType<typeof it> {
        return markerCase({
          marker: GIT_REPOSITORY,
          row,
        },);
      },),
    },),
    describe({
      name: 'PNPM_WORKSPACE',
      children: PNPM_CASES.map(function toTest(row,): ReturnType<typeof it> {
        return markerCase({
          marker: PNPM_WORKSPACE,
          row,
        },);
      },),
    },),
    describe({
      name: fileNamed.name,
      children: FILE_NAMED_CASES.map(function toTest(row,): ReturnType<typeof it> {
        return markerCase({
          marker: fileNamed('deno.json',),
          row,
        },);
      },),
    },),
    describe({
      name: directoryNamed.name,
      children: DIRECTORY_NAMED_CASES.map(function toTest(row,): ReturnType<typeof it> {
        return markerCase({
          marker: directoryNamed('node_modules',),
          row,
        },);
      },),
    },),
    describe({
      name: packageNamed.name,
      children: PACKAGE_NAMED_CASES.map(function toTest(row,): ReturnType<typeof it> {
        return markerCase({
          marker: packageNamed(PACKAGE,),
          row,
        },);
      },),
    },),
    describe({
      name: 'marker names',
      children: [
        it({
          name: 'presets and factories carry distinct, stable names',
          fn: async () => {
            expect(MISE_MONOREPO.name,).toBe('mise monorepo',);
            expect(GIT_REPOSITORY.name,).toBe('git repository',);
            expect(PNPM_WORKSPACE.name,).toBe('file pnpm-workspace.yaml',);
            expect(fileNamed('deno.json',).name,).toBe('file deno.json',);
            expect(directoryNamed('node_modules',).name,).toBe('directory node_modules',);
            expect(packageNamed(PACKAGE,).name,).toBe(`package ${PACKAGE}`,);
          },
        },),
      ],
    },),
  ],
},);
