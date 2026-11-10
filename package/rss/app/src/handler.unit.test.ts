/**
 Regression coverage for the ignore endpoint storing whatever a client sent:
 it enforced neither the documented body shape nor any size bound, so any
 caller could append arbitrary bytes to the JSONL file.
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ignore, } from './handler.ts';

/**
 File the endpoint appends records to, asserted on disk after each request.
 */
const RECORD_FILE = 'api.jsonl';

/**
 Link carried by the accepted fixture body.
 */
const FIXTURE_LINK = 'https://example.com/1';

/**
 Body size no ignore request may exceed; far above the documented record and
 far below the megabytes an abusing client used to append.
 */
const OVERSIZED_BODY_BYTES = 1_048_576;

/**
 Disposable ignore directory, removed when the enclosing scope exits.
 */
type DisposableDirectory = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates an ignore directory outside the checkout so a test never writes, or
 leaves a record in, the directory the server uses.
 
 @returns async-disposable directory handle
 */
async function makeIgnoreDirectory(): Promise<DisposableDirectory> {
  const path = await mkdtemp(join(scratchRoot(), 'rss-ignore-',),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 Builds a POST request the endpoint accepts as an ignore record.
 
 @param body - text sent as the request body
 
 @returns request carrying that body
 */
function postIgnore(body: string,): Request {
  return new Request(
    'http://localhost/api/ignore/new',
    {
      method: 'POST',
      body,
    },
  );
}

/**
 Sentinel for a record file the endpoint never created, distinct from an
 existing empty file.
 */
const RECORD_FILE_ABSENT = Symbol('record file missing on disk',);

/**
 Reads the record file's text, or {@link RECORD_FILE_ABSENT} when the endpoint
 created no file.
 
 @param directory - ignore directory under test
 
 @returns raw JSONL text, or the absent sentinel
 
 @throws when the file exists but cannot be read
 */
async function readRecordText(
  directory: string,
): Promise<string | typeof RECORD_FILE_ABSENT> {
  try {
    return await readFile(
      join(
        directory,
        RECORD_FILE,
      ),
      'utf8',
    );
  }
  catch (error) {
    if (Error.isError(error,)
      && ('code' in error)
      && (error.code === 'ENOENT'))
      return RECORD_FILE_ABSENT;
    throw error;
  }
}

/**
 Splits raw JSONL text into the records it holds.
 
 @param text - raw JSONL text, or {@link RECORD_FILE_ABSENT} for a missing file
 
 @returns parsed records, in file order
 */
function parseRecords(text: string | typeof RECORD_FILE_ABSENT,): unknown[] {
  if (text === RECORD_FILE_ABSENT)
    return [];
  return text
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.length > 0;
    },)
    .map(function parseLine(line,) {
      return JSON.parse(line,) as unknown;
    },);
}

await describe({
  name: ignore.name,
  concurrency: 1,
  children: [
    it({
      name: 'appends one parseable record for a documented link body',
      fn: async () => {
        await using directory = await makeIgnoreDirectory();

        const response = await ignore({
          request: postIgnore(JSON.stringify({ link: FIXTURE_LINK, },),),
          directory: directory.path,
        },);

        const records = parseRecords(await readRecordText(directory.path,),);
        expect(response.status,).toBe(200,);
        expect(records,).toEqual([{ link: FIXTURE_LINK, },],);
      },
    },),
    it({
      name: 'writes one record per line across repeated appends',
      fn: async () => {
        await using directory = await makeIgnoreDirectory();
        const body = JSON.stringify({ link: FIXTURE_LINK, },);

        await ignore({
          request: postIgnore(body,),
          directory: directory.path,
        },);
        await ignore({
          request: postIgnore(body,),
          directory: directory.path,
        },);

        const text = await readRecordText(directory.path,);
        expect(text,).toBe(`${body}\n${body}`,);
      },
    },),
    it({
      name: 'appends the metadata body the reader sends for an unresolvable link',
      fn: async () => {
        await using directory = await makeIgnoreDirectory();
        const metadataOuterHtml = '<div class="feed__metadata">One</div>';

        const response = await ignore({
          request: postIgnore(JSON.stringify({ metadataOuterHtml, },),),
          directory: directory.path,
        },);

        const records = parseRecords(await readRecordText(directory.path,),);
        expect(response.status,).toBe(200,);
        expect(records,).toEqual([{ metadataOuterHtml, },],);
      },
    },),
    it({
      name: 'rejects a body that is not JSON without creating the record file',
      fn: async () => {
        await using directory = await makeIgnoreDirectory();

        const response = await ignore({
          request: postIgnore('not json at all',),
          directory: directory.path,
        },);

        expect(response.status,).toBe(400,);
        expect(await readRecordText(directory.path,),).toBe(RECORD_FILE_ABSENT,);
      },
    },),
    it({
      name: 'rejects JSON without a documented field without creating the record file',
      fn: async () => {
        await using directory = await makeIgnoreDirectory();

        const response = await ignore({
          request: postIgnore(JSON.stringify({ unrelated: FIXTURE_LINK, },),),
          directory: directory.path,
        },);

        expect(response.status,).toBe(400,);
        expect(await readRecordText(directory.path,),).toBe(RECORD_FILE_ABSENT,);
      },
    },),
    it({
      name: 'rejects an oversized body without creating the record file',
      fn: async () => {
        await using directory = await makeIgnoreDirectory();

        const response = await ignore({
          request: postIgnore('x'.repeat(OVERSIZED_BODY_BYTES,),),
          directory: directory.path,
        },);

        expect(response.status,).toBe(400,);
        expect(await readRecordText(directory.path,),).toBe(RECORD_FILE_ABSENT,);
      },
    },),
  ],
},);
