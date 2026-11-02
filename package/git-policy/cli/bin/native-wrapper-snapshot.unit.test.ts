/**
 Guards for the shared wrapper snapshot: a path dependency the manifests link but a snapshot does not copy fails
 here in moments, instead of after an image build has vendored and compiled for minutes.
 */
import {
  existsSync,
  readFileSync,
} from 'node:fs';
import { mkdtempDisposable, } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { copyEndToEndSnapshot, } from './native-e2e-snapshot.ts';
import { copySourceSnapshot, } from './native-test-snapshot.ts';
import {
  copyWrapperSnapshot,
  pathCrates,
  subjectCrate,
  wrapperCrates,
  wrapperSnapshotEntries,
} from './native-wrapper-snapshot.ts';

/**
 Directory holding every package, three levels above this script.
 */
const packageRoot = resolve(
  import.meta.dirname,
  '../../..',
);

/**
 Text between a path dependency's crate name and its relative path, as file-enforcer writes it.
 */
const pathDependencyMarker = ' = { path = "../../';

/**
 Characters a crate name may hold: ASCII letters, digits, underscores and hyphens.
 */
const crateNameCharacters = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-';

/**
 Whether a character may appear in a crate name.

 @param character - one character of a manifest line

 @returns true when a crate name may contain it
 */
function isCrateNameCharacter(character: string): boolean {
  return crateNameCharacters.includes(character);
}

/**
 Crate a manifest line links by path, read by scanning the line instead of matching a pattern, so a hostile line
 cannot cost more than one pass.
 A path dependency line reads `name = { path = "../../<category>/<crate>", ... }`; a comment line starts with
 `#`, which no crate name contains, so it never matches.

 @param line - one line of a Cargo manifest

 @returns the linked crate's path below the packages directory, no `linked` when the line is no path dependency
 */
function pathDependencyOf(line: string): { readonly linked?: string; } {
  /**
   Where the crate name ends.
   */
  const markerAt = line.indexOf(pathDependencyMarker);
  if ((markerAt <= 0) || (!Array.from(line.slice(0, markerAt)).every(isCrateNameCharacter)))
    return {};
  /**
   The text after the marker, which holds the path and then the closing quote.
   */
  const afterMarker = line.slice(markerAt + pathDependencyMarker.length);
  /**
   Where the path ends.
   */
  const quoteAt = afterMarker.indexOf('"');
  return quoteAt > 0 ? { linked: afterMarker.slice(0, quoteAt) } : {};
}

/**
 Every crate the tested crate links by path, found by reading the manifests, never the lists under test.

 @returns the linked crates below the packages directory, sorted, each once
 */
function linkedCrates(): readonly string[] {
  /**
   Crates reached so far, starting from the tested crate.
   */
  const reached = new Set<string>();
  /**
   Crates whose manifests are still to be read.
   */
  const pending: string[] = [subjectCrate];
  while (pending.length > 0) {
    /**
     Crate whose manifest is read now.
     */
    const crate = pending.pop() ?? subjectCrate;
    /**
     Lines of that crate's manifest.
     */
    const lines = readFileSync(join(packageRoot, crate, 'Cargo.toml'), 'utf8').split('\n');
    for (const line of lines) {
      /**
       Linked crate named by this line, if it is a path dependency.
       */
      const { linked, } = pathDependencyOf(line);
      if ((linked !== undefined) && (!reached.has(linked))) {
        reached.add(linked);
        pending.push(linked);
      }
    }
  }
  return [...reached].toSorted();
}

await describe({
  name: wrapperSnapshotEntries.name,
  children: [
    it({
      name: 'lists exactly the path crates the manifests link, transitively',
      fn: async () => {
        expect(linkedCrates()).toEqual([...pathCrates].toSorted());
      },
    }),
    it({
      name: 'copies the manifest, lockfile and sources of the tested crate and every linked crate',
      fn: async () => {
        /**
         Context the entries are computed for; nothing is copied.
         */
        const context = join(
          scratchRoot(),
          'wrapper-snapshot-entries',
        );
        /**
         Destinations of the entries.
         */
        const destinations = new Set(wrapperSnapshotEntries({ context, })
          .map(function destinationOf(entry) {
            return entry.to;
          }));
        for (const crate of [subjectCrate, ...linkedCrates()]) {
          for (const name of ['Cargo.toml', 'Cargo.lock',]) {
            expect(destinations).toContain(join(context, 'package', crate, name));
          }
          expect(destinations.has(join(context, 'package', crate, 'src'))
            || destinations.has(join(context, 'package', crate, 'src/native'))).toBe(true);
        }
      },
    }),
    it({
      name: 'names only sources that exist in the repository',
      fn: async () => {
        for (const entry of wrapperSnapshotEntries({ context: scratchRoot(), }))
          expect(existsSync(entry.from)).toBe(true);
        expect(wrapperCrates).toEqual([subjectCrate, ...pathCrates]);
      },
    }),
  ],
});

await describe({
  name: copyWrapperSnapshot.name,
  children: [
    {
      harness: 'gate',
      copy: copySourceSnapshot,
    },
    {
      harness: 'end-to-end suite',
      copy: copyEndToEndSnapshot,
    },
  ].map(function snapshotTest({
    harness,
    copy,
  }) {
    return it({
      name: `delivers the manifest of every linked crate in the snapshot of the ${harness}`,
      fn: async () => {
        /**
         Disposable build context receiving the snapshot.
         */
        await using context = await mkdtempDisposable(join(
          scratchRoot(),
          'wrapper-snapshot-',
        ));
        await copy({ context: context.path, });
        for (const crate of [subjectCrate, ...linkedCrates()]) {
          /**
           Where the snapshot must hold the crate's manifest.
           */
          const manifest = join(context.path, 'package', crate, 'Cargo.toml');
          expect(existsSync(manifest)).toBe(true);
        }
      },
    });
  }),
});
