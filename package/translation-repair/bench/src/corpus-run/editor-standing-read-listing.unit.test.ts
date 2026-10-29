/**
 Tests for which files the editor standing read lists as artifacts under one
 named directory, and what it says when it can list neither layout.

 AN ARTIFACT IS A REGULAR FILE named like one (ledger B64): a directory or a
 symlink named like an artifact is skipped. The nested layout, an archive's
 own `artifacts/` directory, is read first, and the archive itself only where
 that directory cannot be listed.

 Every directory is a scratch directory the case makes. Fixtures are
 cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { standingArtifactPaths, } from '../../dist/final/node/index.mjs';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Fixtures

/**
 Makes one archive holding the named files in the directory given, beside a
 directory and a symlink named like an artifact.

 @param under - directory inside the archive the files go in, empty for the
 archive itself

 @param names - artifact file names written as empty objects

 @returns Archive, removed on dispose

 @example
 ```ts
 await using made = await archiveHolding({ under: 'artifacts', names: ['Mittens.json',], },);
 ```
 */
async function archiveHolding(
  {
    under,
    names,
  }: {
    readonly under: string;
    readonly names: readonly string[];
  },
): Promise<AsyncDisposable & { readonly archive: string; }> {
  return await scratchDirWith({
    prefix: 'editor-standing-listing-',
    setup: async function seeded({ path: archive, },): Promise<{ readonly archive: string; }> {
      /**
       Directory the files and the decoys go in.
       */
      const holder = join(
        archive,
        under,
      );
      await mkdir(
        join(
          holder,
          'Tabby.json',
        ),
        { recursive: true, },
      );
      await Promise.all(names.map(async function written(name,): Promise<void> {
        await writeFile(
          join(
            holder,
            name,
          ),
          '{}',
          'utf8',
        );
      },),);
      await symlink(
        'Tabby.json',
        join(
          holder,
          'Siamese.json',
        ),
      );
      return { archive, };
    },
  },);
}

//endregion Fixtures

await describe({
  name: standingArtifactPaths.name,
  children: [
    it({
      name: 'LISTS THE REGULAR FILES IN AN ARCHIVE\'S artifacts DIRECTORY, two of them, skipping a directory and a '
        + 'symlink named like an artifact, and says nothing',
      fn: async () => {
        await using made = await archiveHolding({ under: 'artifacts', names: ['Mittens.json', 'Pepper.json',], },);

        /**
         What the listing found.
         */
        const listing = await standingArtifactPaths({ path: made.archive, },);

        expect({
          paths: listing.paths.toSorted(),
          said: listing.said,
        },).toEqual({
          paths: [
            join(made.archive, 'artifacts', 'Mittens.json',),
            join(made.archive, 'artifacts', 'Pepper.json',),
          ],
          said: [],
        },);
      },
    },),

    it({
      name: 'LISTS THE REGULAR FILE IN AN ARCHIVE THAT IS ITSELF THE ARTIFACTS DIRECTORY, where it holds no '
        + 'artifacts directory, and says nothing',
      fn: async () => {
        await using made = await archiveHolding({ under: '', names: ['Mittens.json',], },);

        expect(await standingArtifactPaths({ path: made.archive, },),).toEqual({
          paths: [join(made.archive, 'Mittens.json',),],
          said: [],
        },);
      },
    },),

    it({
      name: 'READS AN artifacts DIRECTORY HOLDING NO ARTIFACT AS EMPTY, without reading the archive beside it, '
        + 'and says nothing',
      fn: async () => {
        await using made = await archiveHolding({ under: '', names: ['Mittens.json',], },);
        await mkdir(join(made.archive, 'artifacts',),);

        expect(await standingArtifactPaths({ path: made.archive, },),).toEqual({
          paths: [],
          said: [],
        },);
      },
    },),

    it({
      name: 'SAYS WHY NEITHER LAYOUT COULD BE LISTED where the named directory is not there, and lists nothing',
      fn: async () => {
        await using made = await archiveHolding({ under: '', names: [], },);

        /**
         A directory nobody made, inside the scratch archive.
         */
        const missing = join(
          made.archive,
          'never-written',
        );

        expect(await standingArtifactPaths({ path: missing, },),).toEqual({
          paths: [],
          said: [
            `editor-standing-read: no artifacts under ${missing} (ENOENT on artifacts/, ENOENT on the directory itself)`,
          ],
        },);
      },
    },),

    it({
      name: 'SAYS WHY NEITHER LAYOUT COULD BE LISTED where the named path is a file, naming the system\'s own code '
        + 'for each',
      fn: async () => {
        await using made = await archiveHolding({ under: '', names: ['Mittens.json',], },);

        /**
         A regular file named where a directory belongs.
         */
        const file = join(
          made.archive,
          'Mittens.json',
        );

        expect(await standingArtifactPaths({ path: file, },),).toEqual({
          paths: [],
          said: [
            `editor-standing-read: no artifacts under ${file} (ENOTDIR on artifacts/, ENOTDIR on the directory itself)`,
          ],
        },);
      },
    },),
  ],
},);
