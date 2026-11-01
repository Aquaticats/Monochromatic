/**
 Tests for whether a run left anything to verify, and how it says so.

 THIS IS THE WHOLE POINT. `verify-published.ts` answered an absent
 artifacts directory with an empty list and printed the absence on stderr, so
 a directory that was never a run printed the same stdout summary as a run
 whose every page agreed, and left the same exit code behind. Anything using
 the check as a gate passed the run that was never examined.

 THREE POPULATIONS HAVE TO STAY APART, and no two of them may collapse: a run
 that is not there, a run that settled nothing, and a run with entries to
 check. The first two are both "nothing verified" and the third is not, but
 the first two still differ in what an operator does next, so the reason
 rides along in the verdict rather than being thrown away.

 THE REASON IS A FILESYSTEM CODE, NOT A CLASS NAME. `errorName` answers
 `Error` for every filesystem failure, so the report used to separate
 "pointed at the wrong directory" from "cannot read this directory" not at
 all. Two cases here pin `ENOENT` against `ENOTDIR` for exactly that.

 AN ABSENT PUBLISHED TREE IS DELIBERATELY NOT SILENCE. Beside real artifacts
 it means every settled entry was never published, which is this check's most
 serious finding, so it stays checkable with an empty tree.

 DISPOSABLE FIXTURES ONLY: every case writes into its own `scratchDir`
 directory, and nothing here reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type DirectoryReading,
  filesystemReason,
  namesIn,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  publishedEntryIds,
  settledEntryIds,
  whatThereIsToVerify,
} from '../../dist/final/node/index.mjs';
import {
  scratchDir,
  scratchDirWith,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Fixed tree directory a run publishes under, as `publish-fixed.ts` names it.
 */
const FIXED_TREE = 'fixed';

/**
 People directory inside that tree.
 */
const PEOPLE = 'people';

/**
 Artifacts directory a run settles into.
 */
const ARTIFACTS = 'artifacts';

/**
 Writes a run directory holding exactly these artifact file names.

 @param names - file names to write, verbatim, so a case can write something
 that is not an artifact at all

 @returns Run directory holding them under its artifacts directory, removed
 when its `await using` scope ends

 @example
 ```ts
 await using settled = await runSettling({ names: ['Mittens.json',], },);
 const runsDir = settled.runsDir;
 ```
 */
async function runSettling(
  { names, }: { readonly names: readonly string[]; },
): Promise<{ readonly runsDir: string; } & AsyncDisposable> {
  // Disposable root for this case.
  return await scratchDirWith({
    prefix: 'published-tree-listing-',
    setup: async function seeded({ path: runsDir, },): Promise<{ readonly runsDir: string; }> {
      await mkdir(
        join(
          runsDir,
          ARTIFACTS,
        ),
        { recursive: true, },
      );
      await Promise.all(names.map(async function writeOne(name,): Promise<void> {
        await writeFile(
          join(
            runsDir,
            ARTIFACTS,
            name,
          ),
          '{}',
          'utf8',
        );
      },),);
      return { runsDir, };
    },
  },);
}

/**
 Writes a run directory whose published tree holds exactly these entries.

 @param runsDir - run directory to publish into

 @param entryIds - entries to write a page for under the people directory

 @example
 ```ts
 await publishInto({ runsDir, entryIds: ['Mittens',], },);
 ```
 */
async function publishInto(
  {
    runsDir,
    entryIds,
  }: {
    readonly runsDir: string;
    readonly entryIds: readonly string[];
  },
): Promise<void> {
  await Promise.all(entryIds.map(async function makeOne(entryId,): Promise<void> {
    /**
     The entry's directory in the published tree.
     */
    const entryDir = join(
      runsDir,
      FIXED_TREE,
      PEOPLE,
      entryId,
    );
    await mkdir(
      entryDir,
      { recursive: true, },
    );
    await writeFile(
      join(
        entryDir,
        'page.en.md',
      ),
      'A cat naps.\n',
      'utf8',
    );
  },),);
}

/**
 Names read off a listing, or a marker saying it was not readable.

 Keeps every case's assertion one line, and fails loudly rather than
 silently reading `[]` off a refusal, which is the defect under test.

 @param reading - what a listing returned

 @returns Its names, or a marker naming the refusal

 @example
 ```ts
 expect(namesOf({ reading, },),).toEqual(['Mittens',],);
 ```
 */
function namesOf(
  { reading, }: { readonly reading: DirectoryReading; },
): readonly string[] {
  if (reading.kind === 'unreadable')
    return [`UNREADABLE ${reading.reason}`,];
  return reading.names;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: namesIn.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a directory that is there, which is the control every '
            + 'other case departs from',
          fn: async () => {
            await using settled = await runSettling({ names: ['Mittens.json',], },);
            const dir = settled.runsDir;
            expect(namesOf({ reading: await namesIn({
              dir: join(
                dir,
                ARTIFACTS,
              ),
              kind: 'file',
            },), },),)
              .toEqual(['Mittens.json',],);
          },
        },),

        it({
          name: 'REFUSES a directory that is not there, naming ENOENT rather than '
            + 'returning an empty listing a caller reads as a clean one',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'published-tree-listing-', },);
            const reading = await namesIn({
              dir: join(
                scratch.path,
                'nowhere',
              ),
              kind: 'directory',
            },);
            expect(reading.kind,).toBe('unreadable',);
            expect((reading.kind === 'unreadable') ? reading.reason : '',).toBe('ENOENT',);
          },
        },),

        it({
          name: 'REFUSES a path that is a file with ENOTDIR, which is a different '
            + 'operator action from ENOENT and used to read as the same Error',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'published-tree-listing-', },);
            const dir = scratch.path;
            const file = join(
              dir,
              'not-a-directory',
            );
            await writeFile(
              file,
              'the cat sat on the keyboard\n',
              'utf8',
            );
            const reading = await namesIn({
              dir: file,
              kind: 'directory',
            },);
            expect((reading.kind === 'unreadable') ? reading.reason : '',).toBe('ENOTDIR',);
          },
        },),
      ],
    },),

    describe({
      name: filesystemReason.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the filesystem code when the caught value carries one',
          fn: async () => {
            expect(filesystemReason({ error: Object.assign(
              new Error('unread',),
              { code: 'EACCES', },
            ), },),).toBe('EACCES',);
          },
        },),

        it({
          name: 'FALLS BACK to the class name when the caught value carries no code',
          fn: async () => {
            expect(filesystemReason({ error: new TypeError('no code here',), },),)
              .toBe('TypeError',);
          },
        },),

        it({
          name: 'FALLS BACK for a thrown value that is not an Error at all, since '
            + 'a catch binding may hold anything',
          fn: async () => {
            expect(filesystemReason({ error: 'a string nobody should have thrown', },),)
              .toContain('not an Error',);
          },
        },),
      ],
    },),

    describe({
      name: settledEntryIds.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS artifact ids sorted, dropping a file that is not one',
          fn: async () => {
            await using settled = await runSettling({ names: [
              'Whiskers.json',
              'Mittens.json',
              'notes.txt',
            ], },);
            const { runsDir, } = settled;
            expect(namesOf({ reading: await settledEntryIds({ runsDir, },), },),)
              .toEqual([
                'Mittens',
                'Whiskers',
              ],);
          },
        },),

        it({
          name: 'SKIPS a directory and a symlink named like an artifact, which the census and the scheduler never '
            + 'count as settled either, so verifying and republishing never look for their pages (ledger B64)',
          fn: async () => {
            await using settled = await runSettling({ names: ['Mittens.json',], },);
            const { runsDir, } = settled;
            await mkdir(join(
              runsDir,
              ARTIFACTS,
              'Tabby.json',
            ),);
            await symlink(
              'Mittens.json',
              join(
                runsDir,
                ARTIFACTS,
                'Siamese.json',
              ),
            );
            expect(namesOf({ reading: await settledEntryIds({ runsDir, },), },),)
              .toEqual(['Mittens',],);
          },
        },),

        it({
          name: 'REFUSES a run directory with no artifacts directory, rather than '
            + 'reporting a run that settled nothing',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'published-tree-listing-', },);
            const reading = await settledEntryIds({ runsDir: scratch.path, },);
            expect((reading.kind === 'unreadable') ? reading.reason : '',).toBe('ENOENT',);
          },
        },),
      ],
    },),

    describe({
      name: publishedEntryIds.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS published entries sorted',
          fn: async () => {
            await using settled = await runSettling({ names: [], },);
            const { runsDir, } = settled;
            await publishInto({
              runsDir,
              entryIds: [
                'Whiskers',
                'Mittens',
              ],
            },);
            expect(namesOf({ reading: await publishedEntryIds({ runsDir, },), },),)
              .toEqual([
                'Mittens',
                'Whiskers',
              ],);
          },
        },),

        it({
          name: 'SKIPS an entry directory whose page is gone, so the entry reads as unpublished rather than as '
            + 'a page that fails to read (ledger A16b)',
          fn: async () => {
            await using settled = await runSettling({ names: [], },);
            const { runsDir, } = settled;
            await publishInto({
              runsDir,
              entryIds: ['Mittens',],
            },);
            await mkdir(
              join(
                runsDir,
                FIXED_TREE,
                PEOPLE,
                'Whiskers',
              ),
              { recursive: true, },
            );
            expect(namesOf({ reading: await publishedEntryIds({ runsDir, },), },),)
              .toEqual(['Mittens',],);
          },
        },),

        it({
          name: 'SKIPS a file and a symlink in the people directory, which no pass writes there, rather than raising '
            + 'ENOTDIR out of the verifier or counting a page that lives under another name (ledger B65)',
          fn: async () => {
            await using settled = await runSettling({ names: [], },);
            const { runsDir, } = settled;
            await publishInto({
              runsDir,
              entryIds: [
                'Mittens',
                'Whiskers',
              ],
            },);

            /**
             The published tree's people directory.
             */
            const peopleDir = join(
              runsDir,
              FIXED_TREE,
              PEOPLE,
            );
            await writeFile(
              join(
                peopleDir,
                'Tabby',
              ),
              'a note the cat left\n',
              'utf8',
            );
            await symlink(
              'Whiskers',
              join(
                peopleDir,
                'Siamese',
              ),
            );
            expect(namesOf({ reading: await publishedEntryIds({ runsDir, },), },),)
              .toEqual([
                'Mittens',
                'Whiskers',
              ],);
          },
        },),

        it({
          name: 'REFUSES a run directory that published nothing at all',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'published-tree-listing-', },);
            const reading = await publishedEntryIds({ runsDir: scratch.path, },);
            expect((reading.kind === 'unreadable') ? reading.reason : '',).toBe('ENOENT',);
          },
        },),
      ],
    },),

    describe({
      name: whatThereIsToVerify.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a directory that is not a run, carrying the reason into '
            + 'the verdict so the report can name ENOENT',
          fn: async () => {
            const verdict = whatThereIsToVerify({
              settled: {
                kind: 'unreadable',
                reason: 'ENOENT',
              },
              published: {
                kind: 'unreadable',
                reason: 'ENOENT',
              },
            },);
            expect(verdict.kind,).toBe('nothing-verified',);
            expect((verdict.kind === 'nothing-verified') ? verdict.why : '',)
              .toContain('ENOENT',);
          },
        },),

        it({
          name: 'REFUSES a run that settled no entry, which is the original defect itself: an '
            + 'empty run used to report exactly what a perfect run reports',
          fn: async () => {
            const verdict = whatThereIsToVerify({
              settled: {
                kind: 'read',
                names: [],
              },
              published: {
                kind: 'read',
                names: [],
              },
            },);
            expect(verdict.kind,).toBe('nothing-verified',);
          },
        },),

        it({
          name: 'ACCEPTS a run with entries to check, carrying both sides through',
          fn: async () => {
            const verdict = whatThereIsToVerify({
              settled: {
                kind: 'read',
                names: ['Mittens',],
              },
              published: {
                kind: 'read',
                names: ['Mittens',],
              },
            },);
            expect(verdict.kind,).toBe('checkable',);
            expect((verdict.kind === 'checkable') ? verdict.published : [],)
              .toEqual(['Mittens',],);
          },
        },),

        it({
          name: 'ACCEPTS real artifacts with no published tree as an empty tree, '
            + 'because every settled entry being unpublished is a finding rather '
            + 'than a silence',
          fn: async () => {
            const verdict = whatThereIsToVerify({
              settled: {
                kind: 'read',
                names: ['Mittens',],
              },
              published: {
                kind: 'unreadable',
                reason: 'ENOENT',
              },
            },);
            expect(verdict.kind,).toBe('checkable',);
            expect((verdict.kind === 'checkable') ? verdict.published : ['unset',],)
              .toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
