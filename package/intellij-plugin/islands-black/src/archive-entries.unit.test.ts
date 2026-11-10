/**
 The plugin JAR's archive listing, against throwaway trees under the shared
 scratch root: name ordering, separator normalization, recursion, and the
 refusal of inputs a JAR cannot carry.

 The build script itself is not exercised here; it only composes these helpers
 with the ZIP writer, and the committed JAR is the evidence that composition
 runs.

 @module
 */

import {
  type Dirent,
  mkdirSync,
  writeFileSync,
} from 'node:fs';
import {
  mkdtemp,
  readdir,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
  relative,
  sep,
} from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  compareDirentNames,
  listArchiveEntries,
  toArchivePath,
} from '../dist/final/node/index.mjs';

/**
 File positions the fixture tree is created in, in creation order.

 Creation order is the reverse of the order the archive must end up in, so a
 listing that returned filesystem order instead of name order fails.
 */
const FIXTURE_FILES_CREATION_ORDER = [
  'zeta.txt',
  'beta.txt',
  join('alpha', 'second.txt',),
  join('alpha', 'first.txt',),
] as const;

/**
 Archive-relative positions the fixture tree must be listed in, in order.
 */
const EXPECTED_ARCHIVE_PATHS = [
  'alpha/first.txt',
  'alpha/second.txt',
  'beta.txt',
  'zeta.txt',
] as const;

/**
 Names whose comparison order the fixture dirents exercise.
 */
const ORDERED_NAMES = ['alpha.txt', 'beta.txt',] as const;

/**
 Path separator the running platform's `sep` is not.
 */
const WINDOWS_SEPARATOR = '\\';

type Fixture = Readonly<{
  path: string;
  [Symbol.asyncDispose]: () => Promise<void>;
}>;

/**
 Creates a disposable directory removed when the test body returns.

 @returns Path of an empty fixture directory under the scratch root

 @example
 ```ts
 await using fixture = await makeFixture();
 ```
 */
async function makeFixture(): Promise<Fixture> {
  const path = await mkdtemp(join(scratchRoot(), 'islands-black-archive-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Assembles the nested fixture tree, writing each file's position as its content.

 @param root - Directory that becomes the fixture root

 @returns The same root path, for use as the listing input

 @example
 ```ts
 const root = await writeFixtureTree({ root: fixture.path, });
 ```
 */
async function writeFixtureTree({ root, }: Readonly<{ root: string; }>,): Promise<string> {
  // Created one file at a time, on purpose: the filesystem then hands these
  // entries back in creation order, so a listing that forgets to sort is
  // failing here rather than passing by luck.
  for (const relativePath of FIXTURE_FILES_CREATION_ORDER) {
    const diskPath = join(
      root,
      relativePath,
    );
    mkdirSync(
      dirname(diskPath,),
      { recursive: true, },
    );
    writeFileSync(
      diskPath,
      relativePath,
      'utf8',
    );
  }
  return root;
}

/**
 Assembles a flat tree of plain files whose names order the dirent comparisons.

 @param root - Directory that receives one file per ordered name

 @returns Directory entries of the fixture, in filesystem order

 @example
 ```ts
 const dirents = await writeOrderedDirents({ root: fixture.path, });
 ```
 */
async function writeOrderedDirents({ root, }: Readonly<{ root: string; }>,): Promise<readonly Dirent[]> {
  await Promise.all(ORDERED_NAMES.map(async function writeNamedFile(name: string,): Promise<void> {
    await writeFile(
      join(root, name,),
      name,
      'utf8',
    );
  },),);
  return readdir(
    root,
    { withFileTypes: true, },
  );
}

/**
 Picks one fixture dirent by name.

 @param dirents - Directory entries the fixture returned

 @param name - Entry name the test depends on

 @returns The matching entry

 @throws When the fixture did not produce the entry

 @example
 ```ts
 const alpha = direntNamed({ dirents, name: 'alpha.txt', });
 ```
 */
function direntNamed(
  {
    dirents,
    name,
  }: Readonly<{
    dirents: readonly Readonly<Dirent>[];
    name: string;
  }>,
): Readonly<Dirent> {
  const found = dirents.find(function byName(dirent: Readonly<Dirent>,): boolean {
    return dirent.name === name;
  },);
  if (found === undefined)
    throw new Error(`fixture directory did not produce ${name}`,);
  return found;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: toArchivePath.name,
      children: [
        it({
          name: 'replaces this platform\'s separator with a forward slash',
          fn: async () => {
            const diskPath = join(
              'META-INF',
              'plugin.xml',
            );
            expect(toArchivePath({ diskPath, },),).toBe('META-INF/plugin.xml',);
          },
        },),
        it({
          name: 'replaces a Windows separator on a host that uses another one',
          fn: async () => {
            expect(
              toArchivePath({
                diskPath: ['theme', 'islands_black.theme.json',].join(WINDOWS_SEPARATOR,),
                pathSeparator: WINDOWS_SEPARATOR,
              },),
            ).toBe('theme/islands_black.theme.json',);
          },
        },),
        it({
          name: 'drops a leading current-directory segment',
          fn: async () => {
            expect(
              toArchivePath({
                diskPath: ['.', 'scheme', 'Islands_Black.xml',].join(sep,),
                pathSeparator: sep,
              },),
            ).toBe('scheme/Islands_Black.xml',);
          },
        },),
        it({
          name: 'leaves a forward-slashed nested path unchanged',
          fn: async () => {
            expect(
              toArchivePath({
                diskPath: 'a/b/c.xml',
                pathSeparator: '/',
              },),
            ).toBe('a/b/c.xml',);
          },
        },),
      ],
    },),
    describe({
      name: compareDirentNames.name,
      children: [
        it({
          name: 'orders the earlier name first',
          fn: async () => {
            await using fixture = await makeFixture();
            const dirents = await writeOrderedDirents({ root: fixture.path, },);
            expect(
              compareDirentNames({
                left: direntNamed({
                  dirents,
                  name: 'alpha.txt',
                },),
                right: direntNamed({
                  dirents,
                  name: 'beta.txt',
                },),
              },),
            ).toBe(-1,);
          },
        },),
        it({
          name: 'orders the later name second',
          fn: async () => {
            await using fixture = await makeFixture();
            const dirents = await writeOrderedDirents({ root: fixture.path, },);
            expect(
              compareDirentNames({
                left: direntNamed({
                  dirents,
                  name: 'beta.txt',
                },),
                right: direntNamed({
                  dirents,
                  name: 'alpha.txt',
                },),
              },),
            ).toBe(1,);
          },
        },),
        it({
          name: 'treats one name as equal to itself',
          fn: async () => {
            await using fixture = await makeFixture();
            const dirents = await writeOrderedDirents({ root: fixture.path, },);
            const alpha = direntNamed({
              dirents,
              name: 'alpha.txt',
            },);
            expect(
              compareDirentNames({
                left: alpha,
                right: alpha,
              },),
            ).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: listArchiveEntries.name,
      children: [
        it({
          name: 'walks nested directories in name order, not creation order',
          fn: async () => {
            await using fixture = await makeFixture();
            const root = await writeFixtureTree({ root: fixture.path, },);
            const entries = await listArchiveEntries({ root, },);
            expect(
              entries.map(function toPosition(entry,) {
                return relative(
                  root,
                  entry.diskPath,
                )
                  .split(sep,)
                  .join('/',);
              },),
            ).toEqual([...EXPECTED_ARCHIVE_PATHS,],);
          },
        },),
        it({
          name: 'names every entry with forward slashes only',
          fn: async () => {
            await using fixture = await makeFixture();
            const root = await writeFixtureTree({ root: fixture.path, },);
            const entries = await listArchiveEntries({ root, },);
            expect(
              entries.filter(function holdsBackslash(entry,) {
                return entry.archivePath.includes(WINDOWS_SEPARATOR,);
              },).length,
            ).toBe(0,);
          },
        },),
        it({
          name: 'returns nothing for a directory holding no files',
          fn: async () => {
            await using fixture = await makeFixture();
            expect(await listArchiveEntries({ root: fixture.path, },),).toEqual([],);
          },
        },),
        it({
          name: 'refuses a symbolic link instead of dropping it',
          skip: process.platform === 'win32',
          fn: async () => {
            await using fixture = await makeFixture();
            const target = join(fixture.path, 'target.txt',);
            await writeFile(target, 'target', 'utf8',);
            await symlink(target, join(fixture.path, 'link.txt',),);
            /** Captured refusal. */
            let caught: unknown;
            try {
              await listArchiveEntries({ root: fixture.path, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect(String(caught,),).toContain('link.txt',);
          },
        },),
      ],
    },),
  ],
},);
