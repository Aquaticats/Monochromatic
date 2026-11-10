/**
 The plugin archive is reproducible: two builds of the same inputs produce the
 same bytes, because every member is stamped with one fixed instant instead of
 the moment the build happened to run.

 The committed `islands-black.jar` is what users install, so a byte that moves
 without an input change turns every build into a diff the committer has to
 discard by hand.

 @module
 */

import { execFile, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import {
  cp,
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { dosDateTime, } from '@monochromatic-dev/zip-writer-core/ts';

import { ARCHIVE_MODIFIED_AT, } from '../dist/final/node/index.mjs';

/**
 Build script under test, run over a fixture copy of the package's archive roots.
 */
const BUILD_SCRIPT_URL = new URL('../src/build.ts', import.meta.url,);

/**
 Package root the archive roots are copied from.
 */
const PACKAGE_ROOT_URL = new URL('../', import.meta.url,);

/**
 Directories the build packs, copied into each fixture.
 */
const ARCHIVE_ROOTS = [
  'META-INF',
  'theme',
  'scheme',
] as const;

/**
 Signature opening a local file header.
 */
const LOCAL_FILE_HEADER_SIGNATURE = 0x04_03_4B_50;

/**
 Bytes a local file header spends before its name, extra field and content.
 */
const LOCAL_FILE_HEADER_FIXED_SIZE = 30;

/**
 Offset of the modification time word inside a local file header.
 */
const MODIFICATION_TIME_OFFSET = 10;

/**
 Offset of the modification date word inside a local file header.
 */
const MODIFICATION_DATE_OFFSET = 12;

/**
 Offset of the compressed size inside a local file header.
 */
const COMPRESSED_SIZE_OFFSET = 18;

/**
 Offset of the file name length inside a local file header.
 */
const NAME_LENGTH_OFFSET = 26;

/**
 Offset of the extra field length inside a local file header.
 */
const EXTRA_LENGTH_OFFSET = 28;

/**
 Header words are little endian, per APPNOTE.txt.
 */
const HEADER_VIEW_IS_LITTLE_ENDIAN = true;

/**
 Bytes of one SHA-256 digest, in hexadecimal digits.
 */
const SHA256_HEX_LENGTH = 64;

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

const execFileAsync = promisifyExecFile(execFile,);

type Fixture = Readonly<{
  path: string;
  [Symbol.asyncDispose]: () => Promise<void>;
}>;

type HeaderTimestamp = Readonly<{
  time: number;
  date: number;
}>;

/**
 Creates a disposable copy of the package's archive roots.

 @returns Path of a fixture directory holding `META-INF`, `theme` and `scheme`

 @example
 ```ts
 await using fixture = await makeFixture();
 ```
 */
async function makeFixture(): Promise<Fixture> {
  const path = await mkdtemp(join(scratchRoot(), 'islands-black-reproducible-',),);
  await Promise.all(ARCHIVE_ROOTS.map(async function copyRoot(root: string,): Promise<void> {
    await cp(
      new URL(root, PACKAGE_ROOT_URL,),
      join(
        path,
        root,
      ),
      { recursive: true, },
    );
  },),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Runs the build script over a fixture and returns the archive it wrote.

 @param cwd - Fixture directory the script reads and writes in

 @returns Bytes of the archive the script produced

 @example
 ```ts
 const bytes = await buildArchive({ cwd: fixture.path, });
 ```
 */
async function buildArchive({ cwd, }: Readonly<{ cwd: string; }>,): Promise<Uint8Array> {
  await execFileAsync(
    process.execPath,
    [BUILD_SCRIPT_URL.pathname,],
    { cwd, },
  );
  return readFile(
    join(cwd, 'islands-black.jar',),
  );
}

/**
 Digests an archive.

 @param bytes - Archive bytes

 @returns Lowercase hexadecimal SHA-256

 @example
 ```ts
 const digest = sha256({ bytes, });
 ```
 */
function sha256({ bytes, }: Readonly<{ bytes: Uint8Array; }>,): string {
  return createHash('sha256',)
    .update(bytes,)
    .digest('hex',);
}

/**
 Reads one unsigned word of a local file header.

 @param view - Little-endian view over the archive bytes

 @param offset - Byte offset of the word

 @param width - Word width, in bytes

 @returns The word read at that offset

 @example
 ```ts
 const time = headerWord({ view, offset: MODIFICATION_TIME_OFFSET, width: 2, });
 ```
 */
function headerWord(
  {
    view,
    offset,
    width,
  }: Readonly<{
    view: DataView;
    offset: number;
    width: 2 | 4;
  }>,
): number {
  return width === 2
    ? view.getUint16(
      offset,
      HEADER_VIEW_IS_LITTLE_ENDIAN,
    )
    : view.getUint32(
      offset,
      HEADER_VIEW_IS_LITTLE_ENDIAN,
    );
}

/**
 Reads the modification words of every local file header the archive carries.

 Walks the local headers from offset zero, stepping over each name, extra field
 and stored content, which is all this writer emits before the central
 directory.

 @param bytes - Archive bytes

 @returns One timestamp per member, in archive order

 @example
 ```ts
 const stamps = memberTimestamps({ bytes, });
 ```
 */
function memberTimestamps({ bytes, }: Readonly<{ bytes: Uint8Array; }>,): readonly HeaderTimestamp[] {
  const view = new DataView(
    bytes.buffer,
    bytes.byteOffset,
    bytes.byteLength,
  );
  const timestamps: HeaderTimestamp[] = [];
  let offset = 0;
  while (headerWord({
    view,
    offset,
    width: 4,
  },) === LOCAL_FILE_HEADER_SIGNATURE) {
    timestamps.push({
      time: headerWord({
        view,
        offset: offset + MODIFICATION_TIME_OFFSET,
        width: 2,
      },),
      date: headerWord({
        view,
        offset: offset + MODIFICATION_DATE_OFFSET,
        width: 2,
      },),
    },);
    offset += LOCAL_FILE_HEADER_FIXED_SIZE
      + headerWord({
        view,
        offset: offset + NAME_LENGTH_OFFSET,
        width: 2,
      },)
      + headerWord({
        view,
        offset: offset + EXTRA_LENGTH_OFFSET,
        width: 2,
      },)
      + headerWord({
        view,
        offset: offset + COMPRESSED_SIZE_OFFSET,
        width: 4,
      },);
  }
  return timestamps;
}

/**
 The modification words every member must carry.

 @returns The archive's fixed instant, encoded the way the ZIP writer encodes it

 @example
 ```ts
 const expected = expectedTimestamp();
 ```
 */
function expectedTimestamp(): HeaderTimestamp {
  const encoded = dosDateTime(ARCHIVE_MODIFIED_AT,);
  return {
    time: encoded.time,
    date: encoded.date,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'islands-black.jar',
      children: [
        it({
          name: 'is byte-identical when the same inputs are built twice',
          fn: async () => {
            await using fixture = await makeFixture();
            const first = await buildArchive({ cwd: fixture.path, },);
            const second = await buildArchive({ cwd: fixture.path, },);
            expect(sha256({ bytes: first, },).length,).toBe(SHA256_HEX_LENGTH,);
            expect(sha256({ bytes: second, },),).toBe(sha256({ bytes: first, },),);
          },
        },),
        it({
          name: 'stamps every member with the archive\'s fixed modification time',
          fn: async () => {
            await using fixture = await makeFixture();
            const timestamps = memberTimestamps({ bytes: await buildArchive({ cwd: fixture.path, },), },);
            expect(timestamps.length,).toBeGreaterThan(0,);
            expect(timestamps,).toEqual(timestamps.map(function toFixed() {
              return expectedTimestamp();
            },),);
          },
        },),
      ],
    },),
  ],
},);
