/**
 Tests for the manifest `score-verify` joins to its sheet by position.

 The join is only as good as the manifest's order, so a case holds each shape
 the manifest may take and each refusal that keeps a reordered or malformed
 file from labelling grades wrongly.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RunJsonUnreadableError, } from '@monochromatic-dev/translation-repair-shared-contract';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  parseVerifyManifest,
  readVerifyManifest,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 What parsing a value raised, or the rows it gave.

 @param value - parsed manifest, of any shape

 @returns Refusal

 @example
 ```ts
 const refusal = refusalOf({ value: [], },);
 ```
 */
function refusalOf({ value, }: { readonly value: unknown; },): unknown {
  return caught(function parsesManifest(): unknown {
    return parseVerifyManifest({ value, },);
  },);
}

await describe({
  name: 'score-verify-manifest',
  children: [
    describe({
      name: parseVerifyManifest.name,
      children: [
        it({
          name: 'READS every row, numbering it by its place in the file',
          fn: async () => {
            expect(parseVerifyManifest({
              value: {
                items: [
                  {
                    position: 1,
                    entryId: 'Whiskers',
                    kind: 'control',
                  },
                  {
                    position: 2,
                    entryId: 'Mittens',
                    kind: 'flagged',
                  },
                ],
              },
            },),).toStrictEqual([
              {
                position: 1,
                entryId: 'Whiskers',
                kind: 'control',
              },
              {
                position: 2,
                entryId: 'Mittens',
                kind: 'flagged',
              },
            ],);
          },
        },),

        it({
          name: 'READS a manifest with no row as no rows',
          fn: async () => {
            expect(parseVerifyManifest({ value: { items: [], }, },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'REFUSES a manifest that is not an object, naming the manifest',
          fn: async () => {
            /**
             What parsing a list raised.
             */
            const refusal = refusalOf({ value: [], },);

            expect(refusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusal,),).toBe('ArtifactParseError: artifact parse failed at verify manifest: expected an object.',);
          },
        },),

        it({
          name: 'REFUSES a manifest whose items are not a list, naming the field',
          fn: async () => {
            /**
             What parsing a manifest with a text for its items raised.
             */
            const refusal = refusalOf({ value: { items: 'none', }, },);

            expect(refusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusal,),).toBe(
              'ArtifactParseError: artifact parse failed at verify manifest.items: expected an array.',
            );
          },
        },),

        it({
          name: 'REFUSES a row that is not an object, naming the row',
          fn: async () => {
            /**
             What parsing a manifest with a text for its row raised.
             */
            const refusal = refusalOf({ value: { items: ['Whiskers',], }, },);

            expect(refusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusal,),).toBe(
              'ArtifactParseError: artifact parse failed at verify manifest.items[]: expected an object.',
            );
          },
        },),

        it({
          name: 'REFUSES in its own words a row whose position is not its place, without quoting the position',
          fn: async () => {
            /**
             What parsing a manifest whose second row carries a text for its position
             raised.
             */
            const refusal = refusalOf({
              value: {
                items: [
                  {
                    position: 1,
                    entryId: 'Whiskers',
                    kind: 'control',
                  },
                  {
                    position: 'a long note that names the corpus',
                    entryId: 'Mittens',
                    kind: 'flagged',
                  },
                ],
              },
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: verify manifest item 2 does not carry position 2, so the file is not in '
                + 'sheet order and a positional join would mislabel every grade',
            );
          },
        },),

        it({
          name: 'REFUSES a row with no entry id, naming the field',
          fn: async () => {
            /**
             What parsing a row without an entry id raised.
             */
            const refusal = refusalOf({
              value: {
                items: [
                  {
                    position: 1,
                    kind: 'control',
                  },
                ],
              },
            },);

            expect(refusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusal,),).toBe(
              'ArtifactParseError: artifact parse failed at verify manifest.items[].entryId: expected a string.',
            );
          },
        },),

        it({
          name: 'REFUSES a row with no set, naming the field',
          fn: async () => {
            /**
             What parsing a row without a set raised.
             */
            const refusal = refusalOf({
              value: {
                items: [
                  {
                    position: 1,
                    entryId: 'Whiskers',
                  },
                ],
              },
            },);

            expect(refusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusal,),).toBe(
              'ArtifactParseError: artifact parse failed at verify manifest.items[].kind: expected a string.',
            );
          },
        },),
      ],
    },),

    describe({
      name: readVerifyManifest.name,
      children: [
        it({
          name: 'READS the rows of a manifest file',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-verify-manifest-', },);

            /**
             Manifest file the case wrote.
             */
            const path = join(
              scratch.path,
              'probe-verify-manifest.json',
            );
            await writeFile(
              path,
              JSON.stringify({
                items: [
                  {
                    position: 1,
                    entryId: 'Whiskers',
                    kind: 'control',
                  },
                ],
              },),
              'utf8',
            );

            expect(await readVerifyManifest({ path, },),).toStrictEqual([
              {
                position: 1,
                entryId: 'Whiskers',
                kind: 'control',
              },
            ],);
          },
        },),

        it({
          name: 'REFUSES a manifest file that is not there, naming its base name and ENOENT',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-verify-manifest-', },);

            /**
             What reading a file nobody wrote raised.
             */
            const refusal = await rejectionOf(async function readsAbsentManifest(): Promise<void> {
              await readVerifyManifest({
                path: join(
                  scratch.path,
                  'probe-verify-manifest.json',
                ),
              },);
            },);

            expect(refusal,).toBeInstanceOf(RunJsonUnreadableError,);
            expect(String(refusal,),).toBe(
              'RunJsonUnreadableError: could not read probe-verify-manifest.json as JSON (ENOENT)',
            );
          },
        },),
      ],
    },),
  ],
},);
