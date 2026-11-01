/**
 Tests for the generation census over a settled directory.

 Each classification the census can answer gets one file, so the guard that
 builds sentences from these answers is shown the whole alphabet: a declared
 generation, an unversioned one, a version the build cannot read, and the two
 shapes that are not artifacts at all. The directory also carries the two
 entries the listing must skip.

 Fixtures are cat-themed invention.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  censusBySchema,
  type SchemaCensusRow,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Writes one file per classification, plus a directory and a suffix-less file
 the listing skips, into a caller-owned directory.

 @param dir - case-owned directory to write into

 @returns Nothing; the files are the effect

 @example
 ```ts
 await censusDirectory({ dir: scratch.path, },);
 ```
 */
async function censusDirectory({ dir: artifactsDir, }: { readonly dir: string; },): Promise<void> {
  await writeFile(join(artifactsDir, 'declared.json',), JSON.stringify({ artifactSchemaVersion: 4, },), 'utf8',);
  await writeFile(join(artifactsDir, 'unversioned.json',), JSON.stringify({ sliceCount: 0, },), 'utf8',);
  await writeFile(
    join(artifactsDir, 'unreadable.json',),
    JSON.stringify({ artifactSchemaVersion: 'four', },),
    'utf8',
  );
  await writeFile(join(artifactsDir, 'not-json.json',), 'not json {', 'utf8',);
  await writeFile(join(artifactsDir, 'array.json',), '[1]', 'utf8',);
  await mkdir(join(artifactsDir, 'directory.json',),);
  await writeFile(join(artifactsDir, 'notes.txt',), 'not an artifact', 'utf8',);
}

/**
 Row for one entry, or a throw naming the entry the census dropped.

 @param rows - census as returned

 @param entryId - entry wanted

 @returns Its row

 @throws {@link Error} when the census carries no row for it

 @example
 ```ts
 const row = rowFor({ rows, entryId: 'declared', },);
 ```
 */
function rowFor(
  {
    rows,
    entryId,
  }: {
    readonly rows: readonly SchemaCensusRow[];
    readonly entryId: string;
  },
): SchemaCensusRow {
  /**
   Row under that id, absent when the census skipped it.
   */
  const row = rows.find(function isWanted(candidate,): boolean {
    return candidate.entryId === entryId;
  },);
  if (row === undefined)
    throw new Error(`the census carries no row for ${entryId}`,);
  return row;
}

await describe({
  name: censusBySchema.name,
  children: [
    it({
      name: 'REFUSES to skip a file that is not JSON, classifying it malformed with the parser\'s reason, '
        + 'since a census that ignores part of the directory reports the directory fine',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-schema-census-', },);
        await censusDirectory({ dir: scratch.path, },);
        const row = rowFor({
          rows: await censusBySchema({ artifactsDir: scratch.path, },),
          entryId: 'not-json',
        },);

        expect(row.classification.kind,).toBe('malformed',);
        if (row.classification.kind === 'malformed')
          expect(row.classification.reason.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'classifies JSON that is not a record as malformed too, apart from every generation',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-schema-census-', },);
        await censusDirectory({ dir: scratch.path, },);
        const row = rowFor({
          rows: await censusBySchema({ artifactsDir: scratch.path, },),
          entryId: 'array',
        },);

        expect(row.classification.kind,).toBe('malformed',);
      },
    },),

    it({
      name: 'reads a declared generation with its number',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-schema-census-', },);
        await censusDirectory({ dir: scratch.path, },);
        const row = rowFor({
          rows: await censusBySchema({ artifactsDir: scratch.path, },),
          entryId: 'declared',
        },);

        expect(row.classification,).toEqual({
          kind: 'declared',
          version: 4,
        },);
      },
    },),

    it({
      name: 'reads a record with no version field as unversioned',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-schema-census-', },);
        await censusDirectory({ dir: scratch.path, },);
        const row = rowFor({
          rows: await censusBySchema({ artifactsDir: scratch.path, },),
          entryId: 'unversioned',
        },);

        expect(row.classification,).toEqual({ kind: 'unversioned', },);
      },
    },),

    it({
      name: 'classifies a version that is not a count as unreadable rather than throwing out of the census',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-schema-census-', },);
        await censusDirectory({ dir: scratch.path, },);
        const row = rowFor({
          rows: await censusBySchema({ artifactsDir: scratch.path, },),
          entryId: 'unreadable',
        },);

        expect(row.classification.kind,).toBe('unreadable-version',);
      },
    },),

    it({
      name: 'lists exactly the regular artifacts, in name order, skipping the directory and the suffix-less file',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-schema-census-', },);
        await censusDirectory({ dir: scratch.path, },);
        const rows = await censusBySchema({ artifactsDir: scratch.path, },);

        expect(rows.map(function toId(row,): string {
          return row.entryId;
        },),).toEqual([
          'array',
          'declared',
          'not-json',
          'unreadable',
          'unversioned',
        ],);
      },
    },),
  ],
},);
