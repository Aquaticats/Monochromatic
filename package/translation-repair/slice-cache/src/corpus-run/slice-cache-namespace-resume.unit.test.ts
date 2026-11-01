/**
 Tests for the slice cache's resume read: what recomputes and what surfaces.

 A HALF-WRITTEN FILE IS RECOMPUTED AND SAID (ledger A11), since the slice is
 bought again either way; any other fault surfaces, since persisting is atomic
 now and a file that will not read under that is a fault worth seeing.

 FIXTURES ARE CAT-THEMED, over a throwaway cache directory.

 @module
 */

import {
  truncate,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  loadNamespacedSlices,
  REPAIR_SLICE_NAMESPACE,
  sliceFileName,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Slice cache resume tests

/**
 Size of a cache file Node's `readFile` refuses before it reads a byte: one
 past the largest it reads (2 GiB less one), held sparse so it takes no disk.
 Unlike a mode that forbids reading, it refuses whoever runs the suite, the
 superuser included.
 */
const UNREADABLE_SIZE = 2 ** 31;

await describe({
  name: loadNamespacedSlices.name,
  children: [
    it({
      name: 'RECOMPUTES a cache file whose JSON is no record, and SURFACES the fault where the file will '
        + 'not read',
      fn: async () => {
        // Cache directory holding one file whose JSON is a number and one
        // too large to read.
        await using dir = await scratchDir({ prefix: 'slice-cache-namespace-', },);
        await writeFile(
          join(dir.path, sliceFileName({ key: 'k1', namespace: REPAIR_SLICE_NAMESPACE, },),),
          '5',
          'utf8',
        );
        const loaded = await loadNamespacedSlices({
          dir: dir.path,
          namespace: REPAIR_SLICE_NAMESPACE,
          isValue: function isText(value: unknown,): value is string {
            return (typeof value) === 'string';
          },
        },);
        expect(loaded.size,).toBe(0,);

        await writeFile(
          join(dir.path, sliceFileName({ key: 'k2', namespace: REPAIR_SLICE_NAMESPACE, },),),
          '5',
          'utf8',
        );
        await truncate(
          join(dir.path, sliceFileName({ key: 'k2', namespace: REPAIR_SLICE_NAMESPACE, },),),
          UNREADABLE_SIZE,
        );
        /**
         What the load raised, read for the read fault it carries.
         */
        const refusal: unknown = await rejectionOf(async function load(): Promise<unknown> {
          return await loadNamespacedSlices({
            dir: dir.path,
            namespace: REPAIR_SLICE_NAMESPACE,
            isValue: function isText(value: unknown,): value is string {
              return (typeof value) === 'string';
            },
          },);
        },);
        expect(String(refusal,),).toBe(
          `RangeError [ERR_FS_FILE_TOO_LARGE]: File size (${String(UNREADABLE_SIZE,)}) is greater than 2 GiB`,
        );
      },
    },),

    it({
      name: 'SURFACES a fault that is not a half-written file, here a guard that raises on a file that '
        + 'reads, rather than recomputing it',
      fn: async () => {
        await using dir = await scratchDir({ prefix: 'slice-cache-namespace-', },);
        await writeFile(
          join(dir.path, sliceFileName({ key: 'k1', namespace: REPAIR_SLICE_NAMESPACE, },),),
          '5',
          'utf8',
        );
        /**
         What the load raised, read for its class and its whole wording.
         */
        const refusal: unknown = await rejectionOf(async function load(): Promise<unknown> {
          return await loadNamespacedSlices({
            dir: dir.path,
            namespace: REPAIR_SLICE_NAMESPACE,
            isValue: function isTornGuard(value: unknown,): value is string {
              throw new RangeError(`the cat knocked the guard over while it was handed ${typeof value}`,);
            },
          },);
        },);
        expect(refusal,).toBeInstanceOf(RangeError,);
        expect(String(refusal,),).toBe('RangeError: the cat knocked the guard over while it was handed undefined',);
      },
    },),
  ],
},);

//endregion Slice cache resume tests
