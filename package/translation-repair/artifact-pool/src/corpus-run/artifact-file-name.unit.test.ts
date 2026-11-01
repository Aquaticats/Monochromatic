/**
 Tests for what names an artifact, read one way by every reader.

 AN ARTIFACT IS A REGULAR FILE NAMED `<entry id>.json`. Two readers skipped
 the regular-file check the census and the scheduler apply, so a directory or
 a symlink named like an artifact became a settled entry there (ledger B64).
 These cases moved here with the lister from `artifact-placement.ts`, and
 cover the name predicate, the entry id, and the listing that reports an
 absent directory rather than raising.

 DISPOSABLE FIXTURES ONLY: every case writes into its own `scratchDir`
 directory, removed when the case ends, and nothing here reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  artifactFileNameOf,
  artifactFilesIn,
  entryIdOfArtifact,
  isArtifactFileName,
  listArtifactFiles,
} from '../../dist/final/node/index.mjs';
import { scratchDirOfEmptyRecords, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Artifact file name tests

/**
 Writes one disposable directory holding exactly these files, each `{}`.

 @param names - file names to write, verbatim, so a case can write something
 that is not an artifact at all

 @returns Directory holding them, removed on dispose

 @example
 ```ts
 await using fixture = await directoryHolding({ names: ['Mittens.json',], },);
 ```
 */
async function directoryHolding(
  { names, }: { readonly names: readonly string[]; },
): Promise<AsyncDisposable & { readonly dir: string; }> {
  return await scratchDirOfEmptyRecords({
    prefix: 'artifact-file-name-',
    names,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isArtifactFileName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a name ending in the artifact suffix',
          fn: async () => {
            expect(isArtifactFileName('Mittens.json',),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a name with another suffix, and the temporary name an atomic write renames from',
          fn: async () => {
            expect(isArtifactFileName('notes.txt',),).toBe(false,);
            expect(isArtifactFileName('Mittens.json.partial',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: entryIdOfArtifact.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the entry id as the name without its suffix',
          fn: async () => {
            expect(entryIdOfArtifact({ name: 'Mittens.json', },),).toBe('Mittens',);
          },
        },),

        it({
          name: 'REMOVES ONE suffix only, so a doubled suffix keeps the inner one as part of the id',
          fn: async () => {
            expect(entryIdOfArtifact({ name: 'Mittens.json.json', },),).toBe('Mittens.json',);
          },
        },),

        it({
          name: 'READS the empty id off a name that is nothing but the suffix, which readers then refuse',
          fn: async () => {
            expect(entryIdOfArtifact({ name: '.json', },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: artifactFileNameOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES an entry\'s artifact as the id and the suffix, which the entry id reads back',
          fn: async () => {
            const name = artifactFileNameOf({ entryId: 'Mittens', },);
            expect(name,).toBe('Mittens.json',);
            expect(entryIdOfArtifact({ name, },),).toBe('Mittens',);
          },
        },),
      ],
    },),

    describe({
      name: listArtifactFiles.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'lists every regular file named like an artifact, which is the control the skipping cases depart '
            + 'from',
          fn: async () => {
            await using fixture = await directoryHolding({ names: [
              'Mittens.json',
              'Tabby.json',
            ], },);
            expect([...await listArtifactFiles({ artifactsDir: fixture.dir, },),].toSorted(),)
              .toEqual([
                'Mittens.json',
                'Tabby.json',
              ],);
          },
        },),

        it({
          name: 'SKIPS a file with another suffix, and a temporary file an interrupted write left behind',
          fn: async () => {
            await using fixture = await directoryHolding({ names: [
              'Mittens.json',
              'notes.txt',
              'Tabby.json.partial',
            ], },);
            expect([...await listArtifactFiles({ artifactsDir: fixture.dir, },),],)
              .toEqual(['Mittens.json',],);
          },
        },),

        it({
          name: 'SKIPS a directory named like an artifact, which otherwise reached the read and threw EISDIR out '
            + 'of the whole census',
          fn: async () => {
            await using fixture = await directoryHolding({ names: ['Mittens.json',], },);
            await mkdir(join(
              fixture.dir,
              'backup.json',
            ),);
            expect([...await listArtifactFiles({ artifactsDir: fixture.dir, },),],)
              .toEqual(['Mittens.json',],);
          },
        },),

        it({
          name: 'SKIPS a symbolic link, which was followed wherever it pointed and could duplicate another '
            + 'artifact under a second identity or leave the directory entirely',
          fn: async () => {
            await using fixture = await directoryHolding({ names: ['Mittens.json',], },);
            await symlink(
              join(
                fixture.dir,
                'Mittens.json',
              ),
              join(
                fixture.dir,
                'Mittens-again.json',
              ),
            );
            expect([...await listArtifactFiles({ artifactsDir: fixture.dir, },),],)
              .toEqual(['Mittens.json',],);
          },
        },),

        it({
          name: 'lists nothing for a directory holding nothing, rather than refusing a run that has settled no '
            + 'entry yet',
          fn: async () => {
            await using fixture = await directoryHolding({ names: [], },);
            expect([...await listArtifactFiles({ artifactsDir: fixture.dir, },),],).toEqual([],);
          },
        },),

        it({
          name: 'RAISES for a directory that is not there, leaving the caller to decide whether absence is an '
            + 'answer',
          fn: async () => {
            await using fixture = await directoryHolding({ names: [], },);
            await expect(listArtifactFiles({
              artifactsDir: join(
                fixture.dir,
                'nowhere',
              ),
            },),).rejects.toThrow('ENOENT',);
          },
        },),
      ],
    },),

    describe({
      name: artifactFilesIn.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the artifacts of a directory that is there, skipping what is not one',
          fn: async () => {
            await using fixture = await directoryHolding({ names: [
              'Mittens.json',
              'notes.txt',
            ], },);
            await mkdir(join(
              fixture.dir,
              'Tabby.json',
            ),);
            expect(await artifactFilesIn({ dir: fixture.dir, },),).toEqual({
              kind: 'read',
              names: ['Mittens.json',],
            },);
          },
        },),

        it({
          name: 'REPORTS a directory that is not there as unreadable with ENOENT, rather than as an empty '
            + 'listing a caller reads as a run that settled nothing',
          fn: async () => {
            await using fixture = await directoryHolding({ names: [], },);
            expect(await artifactFilesIn({
              dir: join(
                fixture.dir,
                'nowhere',
              ),
            },),).toEqual({
              kind: 'unreadable',
              reason: 'ENOENT',
            },);
          },
        },),

        it({
          name: 'REPORTS a path that is a file as unreadable with ENOTDIR, a different operator action from '
            + 'ENOENT',
          fn: async () => {
            await using fixture = await directoryHolding({ names: ['Mittens.json',], },);
            expect(await artifactFilesIn({
              dir: join(
                fixture.dir,
                'Mittens.json',
              ),
            },),).toEqual({
              kind: 'unreadable',
              reason: 'ENOTDIR',
            },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Artifact file name tests
