# @monochromatic-dev/zip-writer-core

Minimal STORE-only ZIP archive writer.
Zero runtime dependencies.
Write-only by design (no parser,
 no decompressor,
 no streaming reader).
Runs anywhere a modern JavaScript runtime exists:
 browsers,
 Node,
 Bun,
 Deno,
 Cloudflare Workers.

## Why

The PKZIP format has a small useful core (local file header,
 central directory,
 end-of-central-directory) and a large historical periphery (Zip64,
 deflate,
 encryption,
 multi-volume,
 data descriptors,
 ZipCrypto,
 AES,
 EOS marker,
 manifold extra fields).
 For workloads that just need to bundle a handful of files with predictable content,
 the periphery is dead weight that drags in transitive dependencies (deflate libraries,
 polyfills) and parser attack surface (zip-slip,
 decompression bombs,
 malformed-header crashes).

This package implements only the core.
What you give it goes out as bytes;
 nothing reads bytes back in.
The result is verified as a valid ZIP by the system `unzip` and `7z` tools.

## Install

```sh
pnpm add @monochromatic-dev/zip-writer-core
```

## Usage

```ts
import { ZipWriter, } from '@monochromatic-dev/zip-writer-core';

const zip = new ZipWriter();
zip.add('manifest.json', JSON.stringify({ version: 1, },),);
zip.add('data/blob.bin', new Uint8Array([1, 2, 3,],),);
const bytes = zip.build();
// `bytes` is a Uint8Array; write it to disk, return it from a Response, etc.
```

For reproducible output (deterministic byte-for-byte,
 useful in builds):

```ts
const zip = new ZipWriter({ modifiedAt: new Date('2024-01-01T00:00:00Z',), },);
```

The CRC-32 and DOS time helpers are exported for callers that need them directly:

```ts
import {
  crc32,
  dosDateTime,
} from '@monochromatic-dev/zip-writer-core';
```

## Supported

- Storing files uncompressed (compression method 0)
- UTF-8 filenames (general purpose bit 11)
- DOS modification timestamps (1980 epoch,
   second granularity)
- Path validation (no NUL,
   no backslash,
   no leading slash,
   no `..`)
- Insertion-order preservation
- Up to 65 535 entries and 4 GiB total

## Not supported

- Compression (deflate,
   bzip2,
   LZMA,
   etc.):
   the writer always uses STORE
- Zip64 (archives over 4 GiB or with over 65 535 entries)
- Encryption (ZipCrypto,
   AES)
- Streaming output:
   the result is built in a single pre-allocated buffer
- Reading or modifying existing archives
- Spanning,
   splitting,
   multi-volume archives
- Data descriptors,
   comments,
   extra fields beyond the basics

If you need any of the above,
 use [JSZip](https://stuk.github.io/jszip/),
 [fflate](https://github.com/101arrowz/fflate),
 or [client-zip](https://github.com/Touffy/client-zip).

## Format reference

PKWARE APPNOTE.
txt v6.3.10:

- Section 4.3:
   local file header and file data
- Section 4.4:
   central directory record
- Section 4.5:
   end of central directory record
