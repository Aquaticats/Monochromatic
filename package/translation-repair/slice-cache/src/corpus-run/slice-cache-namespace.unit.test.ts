/**
 Tests for how three lanes share one cache directory without deleting each
 other's files.

 WHY THIS FILE EXISTS. The repair lane's namespace is defined by SUBTRACTION:
 it owns every file whose name is not claimed by a listed prefix. So a new
 lane that invents a prefix and forgets to register it is silently adopted by
 the repair lane, whose discard then deletes files it does not own while
 logging that it discarded its own. That error has now cost four times, most
 recently `picture.`, which was added to the store on 2026-08-19 and not to
 the list: opening the repair cache removed a picture reading and reported
 "discarding 1 cached slices".

 THE FIRST TEST WALKS THE PACKAGE'S OWN LIST, `EVERY_SLICE_NAMESPACE`, rather
 than a copy of it. It used to keep a copy, and the copy drifted exactly the
 way the registration it guards had: `contest.` and `pairing.` were missing
 from both, so a repair-lane generation change deleted an entry's contest
 ballots and its whole block pairing while reporting that it discarded its own
 slices. A guard maintained by hand fails the same way as the thing it guards,
 so it now reads the same array the store derives its claims from. The rest
 pin the containment in both directions, since a namespace that claims too
 much is as wrong as one that claims too little.

 THE `openNamespacedCache` SUITE PINS WHEN AND HOW A LANE RESTAMPS ITS
 MARKER: only when the marker does not already name the generation opening,
 and then by replacing the file, never by rewriting it in place. A marker
 cut short reads as another generation, and the open that reads it discards
 every slice the lane holds in that directory.

 @module
 */

import {
  chmod,
  open,
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  belongsToNamespace,
  EVERY_SLICE_NAMESPACE,
  isSliceFileName,
  openNamespacedCache,
  PICTURE_READING_NAMESPACE,
  REPAIR_SLICE_NAMESPACE,
  type SliceNamespace,
  TRANSLATE_SLICE_NAMESPACE,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 A file name in a namespace, built the way the store builds one.

 @param namespace - lane whose prefix it carries

 @param key - cache key standing in for a hash

 @returns Name as it would sit on disk

 @example
 ```ts
 const name = fileIn({ namespace: PICTURE_READING_NAMESPACE, key: 'abc', },);
 ```
 */
function fileIn(
  {
    namespace,
    key,
  }: {
    readonly namespace: SliceNamespace;
    readonly key: string;
  },
): string {
  return `${namespace.prefix}${key}.json`;
}

/**
 Accepts a stored text, the only value the open cases store.

 @param value - stored record

 @returns Whether it is text

 @example
 ```ts
 await openNamespacedCache({ dir, generation, namespace, isValue: isText, },);
 ```
 */
function isText(value: unknown,): value is string {
  return (typeof value) === 'string';
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: belongsToNamespace.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CLAIMS EVERY PREFIXED NAMESPACE THIS PACKAGE DEFINES, so a lane added without '
            + 'registering its prefix fails here rather than silently in production. The repair lane '
            + 'is defined by subtraction, so an unregistered prefix is adopted by it and deleted on '
            + 'the next generation change, which is the error this list has cost four times',
          fn: async () => {
            for (const namespace of EVERY_SLICE_NAMESPACE) {
              if (namespace.prefix === '')
                continue;

              /**
               A file of this lane's, offered to the lane defined by subtraction.
               */
              const name = fileIn({
                namespace,
                key: 'whatever-hash',
              },);

              expect(belongsToNamespace({
                name,
                namespace: REPAIR_SLICE_NAMESPACE,
              },),).toBe(false,);
              expect(belongsToNamespace({
                name,
                namespace,
              },),).toBe(true,);
            }
          },
        },),

        it({
          name: 'REFUSES A PICTURE READING TO THE REPAIR LANE, which is the exact file the repair '
            + 'lane deleted before its prefix was registered',
          fn: async () => {
            /**
             Name a stored picture reading carries.
             */
            const name = fileIn({
              namespace: PICTURE_READING_NAMESPACE,
              key: 'picture-hash-aaa',
            },);

            expect(belongsToNamespace({
              name,
              namespace: REPAIR_SLICE_NAMESPACE,
            },),).toBe(false,);
            expect(belongsToNamespace({
              name,
              namespace: TRANSLATE_SLICE_NAMESPACE,
            },),).toBe(false,);
            expect(belongsToNamespace({
              name,
              namespace: PICTURE_READING_NAMESPACE,
            },),).toBe(true,);
          },
        },),

        it({
          name: 'GIVES THE REPAIR LANE AN UNPREFIXED FILE, since it owns the names already on disk '
            + 'from before any lane had a prefix. A namespace that claims too little would strand '
            + 'every slice settled before the split',
          fn: async () => {
            expect(belongsToNamespace({
              name: 'plain-hash.json',
              namespace: REPAIR_SLICE_NAMESPACE,
            },),).toBe(true,);
            expect(belongsToNamespace({
              name: 'plain-hash.json',
              namespace: PICTURE_READING_NAMESPACE,
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A GENERATION MARKER TO EVERY LANE, since a marker is not a cached slice and '
            + 'a discard that swept one would erase the stamp it is about to compare against',
          fn: async () => {
            for (const namespace of EVERY_SLICE_NAMESPACE)
              for (const marker of EVERY_SLICE_NAMESPACE.map(function toMarker(one,): string {
                return one.marker;
              },))
                expect(belongsToNamespace({
                  name: marker,
                  namespace,
                },),).toBe(false,);
          },
        },),

        it({
          name: 'GIVES EVERY LANE A DISTINCT MARKER FILE, so one lane restamping its generation '
            + 'cannot retire another lane whose work is still current',
          fn: async () => {
            /**
             Marker file name per lane, which must be as distinct as the prefixes.
             */
            const markers = EVERY_SLICE_NAMESPACE.map(function toMarker(one,): string {
              return one.marker;
            },);

            expect(new Set(markers,).size,).toBe(markers.length,);
          },
        },),
      ],
    },),

    describe({
      name: isSliceFileName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS every lane\'s slice file, the control the refusals depart from',
          fn: async () => {
            for (const namespace of EVERY_SLICE_NAMESPACE)
              expect(isSliceFileName({
                name: fileIn({
                  namespace,
                  key: 'abc',
                },),
              },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES every lane\'s generation marker and a slice still being written under its temporary name '
            + '(ledger B65)',
          fn: async () => {
            for (const namespace of EVERY_SLICE_NAMESPACE) {
              expect(isSliceFileName({ name: namespace.marker, },),).toBe(false,);
              expect(isSliceFileName({
                name: `${
              fileIn({
                namespace,
                key: 'abc',
              },)
            }.4242.partial`,
              },),).toBe(false,);
            }
          },
        },),
      ],
    },),

    describe({
      name: openNamespacedCache.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RESTAMPS its marker by replacing the file and leaves nothing beside it, so a reader that opened the '
            + 'marker before a reopen under another generation still reads the earlier generation whole, where a '
            + 'restamp in place showed that reader the later bytes',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'slice-cache-namespace-', },);
            const dir = scratch.path;
            await openNamespacedCache({
              dir,
              generation: 'nap-3',
              namespace: REPAIR_SLICE_NAMESPACE,
              isValue: isText,
            },);
            /**
             Reader holding the marker open since before the restamp.
             */
            await using earlier = await open(
              join(
                dir,
                REPAIR_SLICE_NAMESPACE.marker,
              ),
              'r',
            );
            await openNamespacedCache({
              dir,
              generation: 'nap-4',
              namespace: REPAIR_SLICE_NAMESPACE,
              isValue: isText,
            },);
            expect(await earlier.readFile('utf8',),).toBe('nap-3\n',);
            expect(await readFile(
              join(
                dir,
                REPAIR_SLICE_NAMESPACE.marker,
              ),
              'utf8',
            ),).toBe('nap-4\n',);
            expect(await readdir(dir,),).toEqual([REPAIR_SLICE_NAMESPACE.marker,],);
          },
        },),
        it({
          name: 'WRITES NOTHING on a reopen under the generation its marker already names, so the reopen resumes its '
            + 'slice from a marker and a directory it may only read, where the restamp of every open needed the '
            + 'marker writable and emptied it when the write was refused',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'slice-cache-namespace-', },);
            const dir = scratch.path;
            /**
             Cache as the run that filled it left it, holding one slice.
             */
            const filled = await openNamespacedCache({
              dir,
              generation: 'nap-3',
              namespace: REPAIR_SLICE_NAMESPACE,
              isValue: isText,
            },);
            await filled.persist({
              key: 'whiskers',
              serialized: JSON.stringify('purr',),
            },);
            // Read and traverse only, on the marker and on the directory: a
            // superuser is not held by either, so run as root this case
            // passes whatever the open writes.
            await chmod(
              join(
                dir,
                REPAIR_SLICE_NAMESPACE.marker,
              ),
              0o400,
            );
            await chmod(
              dir,
              0o500,
            );
            /**
             Puts the directory back so the scratch directory can be removed,
             however the reopen ends.
             */
            await using _writable = {
              [Symbol.asyncDispose]: async () => {
                await chmod(
                  dir,
                  0o700,
                );
              },
            };
            /**
             Cache as a later run under the same generation opens it.
             */
            const reopened = await openNamespacedCache({
              dir,
              generation: 'nap-3',
              namespace: REPAIR_SLICE_NAMESPACE,
              isValue: isText,
            },);
            expect([...reopened.resumed,],).toEqual([['whiskers', 'purr',],],);
          },
        },),
      ],
    },),
  ],
},);
