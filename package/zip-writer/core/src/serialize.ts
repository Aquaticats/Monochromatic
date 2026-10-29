/**
 Top-level serializer for ZIP archives.
 
 Computes per-entry offsets, writes local file headers (delegated to
 `headers.ts`), the central directory (delegated), and the
 end-of-central-directory record.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  BYTES_UINT16,
  BYTES_UINT32,
  CDH_FIXED_SIZE,
  EOCD_FIXED_SIZE,
  EOCD_SIGNATURE,
  LFH_FIXED_SIZE,
  MAX_UINT16,
  MAX_UINT32,
} from './constants.ts';
import {
  writeCentralDirectory,
  writeLocalFileHeaders,
} from './headers.ts';
import type {
  Positioned,
  ZipEntry,
} from './types.ts';

export type { ZipEntry, } from './types.ts';

/**
 Compute byte offsets for each entry's local file header and the start of
 the central directory.
 
 @param entries - Iterable of entries in the order they appear in the archive
 
 @returns Positioned entries plus computed offsets and totals
 
 @throws When the archive would exceed legacy ZIP size limits
 */
function computeOffsets(
  entries: Iterable<ZipEntry>,
): {
  positioned: Positioned[];
  cdStart: number;
  cdSize: number;
  totalSize: number;
} {
  /**
   Accumulates entries paired with their computed local file header offsets.
   */
  const positioned: Positioned[] = [];
  /**
   Running byte position marking the next local file header.
   */
  let cursor = 0;
  for (const entry of entries) {
    positioned.push({
      entry,
      lfhOffset: cursor,
    },);
    cursor += LFH_FIXED_SIZE
      + entry
        .nameBytes
        .length
      + entry
        .content
        .length;
  }
  /**
   Cursor frozen at the point the central directory begins.
   */
  const cdStart = cursor;
  /**
   Running total of central directory header bytes.
   */
  let cdSize = 0;
  for (const { entry, } of positioned)
    cdSize += CDH_FIXED_SIZE
      + entry
        .nameBytes
        .length;
  /**
   Final archive size used to allocate the output buffer.
   */
  const totalSize = cdStart + cdSize
    + EOCD_FIXED_SIZE;
  if ((cdStart > MAX_UINT32) || (cdSize > MAX_UINT32)
    || (totalSize > MAX_UINT32)) {
    throw new Error(
      `zip-writer: archive too large for legacy ZIP (${totalSize} bytes, max ${MAX_UINT32}); Zip64 not supported`,
    );
  }
  /**
   Aggregated computation result; named binding so the function matches the helper-shape allowlist.
   */
  const result = {
    positioned,
    cdStart,
    cdSize,
    totalSize,
  };
  return result;
}

/**
 Write the end of central directory record, terminating the archive.
 
 @param view - DataView over the archive buffer
 
 @param entryCount - Number of entries in the central directory
 
 @param cdSize - Total size of the central directory in bytes
 
 @param cdStart - Offset of the start of the central directory
 
 @param startOffset - Cursor where the EOCD record begins
 
 @returns Cursor position after the last write
 
 @mutates view - `view.setUint16` and `view.setUint32` write end-record fields
 */
function writeEndOfCentralDirectory(
  {
    view,
    entryCount,
    cdSize,
    cdStart,
    startOffset,
  }: ForeignBorrowed<Readonly<{
    view: DataView;
    entryCount: number;
    cdSize: number;
    cdStart: number;
    startOffset: number;
  }>>,
): number {
  /**
   Local cursor tracking each successive little-endian write.
   */
  let offset = startOffset;
  view.setUint32(
    offset,
    EOCD_SIGNATURE,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint16(
    offset,
    0,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    0,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    entryCount,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    entryCount,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint32(
    offset,
    cdSize,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint32(
    offset,
    cdStart,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint16(
    offset,
    0,
    true,
  );
  offset += BYTES_UINT16;
  return offset;
}

/**
 Serialize an ordered set of entries into a STORE-only ZIP byte sequence.
 
 Layout: local file headers and data, then central directory headers,
 then end-of-central-directory record. Computed in two passes: first the
 total length and per-entry offsets, then the actual write into a
 pre-allocated buffer.
 
 @param entries - Map of path to entry, in insertion order
 
 @returns Newly allocated `Uint8Array` containing the archive
 
 @throws When the archive would exceed legacy ZIP limits (≥ 65 535
   entries or ≥ 4 GiB total) since Zip64 is not implemented
 
 @example
 ```ts
 const entries = new Map<string, ZipEntry>();
 entries.set('hello.txt', { nameBytes, content, crc, modified, },);
 const bytes = serializeEntries(entries,);
 ```
 */
export function serializeEntries(entries: ReadonlyMap<string, ZipEntry>,): Uint8Array {
  if (entries.size
    > MAX_UINT16) {
    throw new Error(
      `zip-writer: too many entries (${entries.size}, max ${MAX_UINT16}); Zip64 not supported`,
    );
  }

  /**
   Offsets and totals derived in one pass over the entries.
   */
  const {
    positioned,
    cdStart,
    cdSize,
    totalSize,
  } = computeOffsets(entries.values(),);

  /**
   Output byte buffer sized to the precomputed total.
   */
  const buffer = new Uint8Array(totalSize,);
  /**
   Little-endian view over the output buffer for numeric writes.
   */
  const view = new DataView(buffer.buffer,);

  /**
   Cursor after the local file headers; central directory begins here.
   */
  const lfhEnd = writeLocalFileHeaders({
    view,
    buffer,
    positioned,
    startOffset: 0,
  },);
  /**
   Cursor after the central directory; EOCD record begins here.
   */
  const cdEnd = writeCentralDirectory({
    view,
    buffer,
    positioned,
    startOffset: lfhEnd,
  },);
  /**
   Final cursor used to assert the write neither overshot nor undershot the buffer.
   */
  const eocdEnd = writeEndOfCentralDirectory({
    view,
    entryCount: entries.size,
    cdSize,
    cdStart,
    startOffset: cdEnd,
  },);

  if (eocdEnd !== totalSize)
    throw new Error(`zip-writer: internal write mismatch (${eocdEnd} / ${totalSize})`,);
  return buffer;
}
