/**
 Tests the trusted readings `reader-measure` holds a candidate to: which
 files of a runs directory are picture readings the pipeline corroborated,
 and which of them a picture belongs to.

 THE RECORDS DO NOT NAME THEIR PICTURE (the cache key hashes the bytes with the
 roster that was asked), so a picture is matched to the record whose readings
 its deterministic reading overlaps at the corroboration share or more, the
 way the 2026-09-08 instrument did.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { mkdir, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  matchRecord,
  readTrustedRecords,
} from '../../dist/final/node/index.mjs';

/**
 A transcription of a notice about a lost cat.
 */
const NOTICE = 'Lost tabby cat called Mittens, last seen by the old well. Please call 555 0134.';

/**
 A transcription of a menu.
 */
const MENU = 'Tuna on Monday, salmon on Tuesday, chicken on Wednesday, a saucer of cream on Friday.';

/**
 A record as the cache file holds it: the key it was stored under, then the record.

 @param record - stored reading

 @returns File content

 @example
 ```ts
 const content = stored({ record: { kind: 'no-text', characters: 0, }, },);
 ```
 */
function stored({ record, }: { readonly record: unknown; },): string {
  return JSON.stringify({
    cacheKey: 'stand-in-key',
    record,
  },);
}

/**
 A corroborated picture reading as the cache stores it.

 @param text - what both readers carried

 @returns File content

 @example
 ```ts
 const content = corroborated({ text: NOTICE, },);
 ```
 */
function corroborated({ text, }: { readonly text: string; },): string {
  return stored({
    record: {
      kind: 'corroborated',
      overlap: 0.9,
      readings: [
        {
          modelId: 'seated-one',
          text,
        },
        {
          modelId: 'seated-two',
          text,
        },
      ],
    },
  },);
}

await describe({
  name: 'reader measure records',
  children: [
    describe({
      name: readTrustedRecords.name,
      children: [
        it({
          name: 'READS THE CORROBORATED PICTURE READINGS OF EVERY DIRECTORY NAMED, at any depth, and nothing '
            + 'else: not an unavailable verdict that kept no readings, not another lane\'s slice, not a file that is no JSON',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-reader-records-', },);
            const entry = join(scratch.path, 'run', 'Tabby_01',);
            await mkdir(entry, { recursive: true, },);
            await writeFile(join(entry, 'picture.aaa.json',), corroborated({ text: NOTICE, },),);
            await writeFile(join(entry, 'picture.bbb.json',), corroborated({ text: MENU, },),);
            await writeFile(join(entry, 'picture.ccc.json',), stored({ record: { kind: 'unavailable', reason: 'one-reader-only', perReader: [], transient: false, }, },),);
            await writeFile(join(entry, 'translate.ddd.json',), corroborated({ text: NOTICE, },),);
            await writeFile(join(entry, 'picture.eee.json',), '{not json',);
            await writeFile(join(entry, 'picture.fff.json',), stored({ record: { kind: 'corroborated', overlap: 0.9, readings: [{ modelId: 'seated-one', text: NOTICE, },], }, },),);

            const records = await readTrustedRecords({ dirs: [scratch.path,], },);
            expect(records.map(function textOf(record,): string {
              return record[0]?.text ?? '';
            },).toSorted(),).toEqual([MENU, NOTICE,].toSorted(),);
          },
        },),

        it({
          name: 'KEEPS THE READINGS OF A PICTURE THE READERS DISAGREED ABOUT, since the seated band must include the '
            + 'pictures the seated readers did not agree on',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-reader-records-', },);
            await writeFile(join(scratch.path, 'picture.aaa.json',), stored({
              record: {
                kind: 'unavailable',
                reason: 'readers-disagree',
                transient: false,
                perReader: ['', '',],
                overlap: 0.1,
                readings: [
                  {
                    modelId: 'seated-one',
                    text: NOTICE,
                  },
                  {
                    modelId: 'seated-two',
                    text: MENU,
                  },
                ],
              },
            },),);

            const records = await readTrustedRecords({ dirs: [scratch.path,], },);
            expect(records.length,).toBe(1,);
            expect(records[0]?.map(function modelOf(reading,): string {
              return reading.modelId;
            },),).toEqual(['seated-one', 'seated-two',],);
          },
        },),

        it({
          name: 'KEEPS ONE RECORD FOR READINGS STORED UNDER MORE THAN ONE KEY, since a picture is stored once per roster '
            + 'that read it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-reader-records-', },);
            const first = join(scratch.path, 'one',);
            const second = join(scratch.path, 'two',);
            await mkdir(first, { recursive: true, },);
            await mkdir(second, { recursive: true, },);
            await writeFile(join(first, 'picture.aaa.json',), corroborated({ text: NOTICE, },),);
            await writeFile(join(second, 'picture.zzz.json',), corroborated({ text: NOTICE, },),);

            expect((await readTrustedRecords({ dirs: [first, second,], },)).length,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: matchRecord.name,
      children: [
        it({
          name: 'MATCHES A PICTURE TO THE RECORD ITS DETERMINISTIC READING OVERLAPS, and to none when no record reaches '
            + 'the corroboration share',
          fn: async () => {
            await Promise.resolve();
            /**
             Two records: the notice's and the menu's.
             */
            const records = [
              [
                {
                  modelId: 'seated-one',
                  text: NOTICE,
                },
                {
                  modelId: 'seated-two',
                  text: NOTICE,
                },
              ],
              [
                {
                  modelId: 'seated-one',
                  text: MENU,
                },
                {
                  modelId: 'seated-two',
                  text: MENU,
                },
              ],
            ];
            /**
             Match for a noisy reading of the menu.
             */
            const matched = matchRecord({ ocrText: 'Tuna on Monday, salmon on Tuesday, chicken on Wednesday', records, },);
            expect(matched.kind,).toBe('matched',);
            if (matched.kind !== 'matched')
              throw new Error('matched by construction',);
            expect(matched.record[0]?.text,).toBe(MENU,);
            expect(matchRecord({ ocrText: 'Zyx wvu qj kk zz yy xx.', records, },).kind,).toBe('unmatched',);
          },
        },),
      ],
    },),
  ],
},);
