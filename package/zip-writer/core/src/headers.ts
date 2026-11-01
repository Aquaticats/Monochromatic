/**
 Local file header and central directory header writers.
 
 Reference: PKWARE APPNOTE.txt v6.3.10 sections 4.3 (local file header)
 and 4.4 (central directory record).
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  BYTES_UINT16,
  BYTES_UINT32,
  CDH_SIGNATURE,
  COMPRESSION_STORE,
  FLAG_UTF8_FILENAME,
  LFH_SIGNATURE,
  UNIX_FILE_MODE_DEFAULT,
  VERSION_MADE_BY,
  VERSION_NEEDED,
} from './constants.ts';
import type {
  Positioned,
  ZipEntry,
} from './types.ts';

/**
 Write a single local file header followed by its raw file data.
 
 @param view - DataView over the archive buffer
 
 @param buffer - Backing byte buffer (used for raw-byte writes)
 
 @param entry - Entry to write
 
 @param startOffset - Cursor at which to begin writing
 
 @returns Cursor position after the header and data
 
 @mutates view - `view.setUint16` and `view.setUint32` write ZIP header fields
 
 @mutates buffer - `buffer.set` writes filename and file-content bytes
 */
function writeOneLocalFileHeader(
  {
    view,
    buffer,
    entry,
    startOffset,
  }: ForeignBorrowed<Readonly<{
    view: DataView;
    buffer: Uint8Array;
    entry: ZipEntry;
    startOffset: number;
  }>>,
): number {
  /**
   Local cursor tracking each successive little-endian write.
   */
  let offset = startOffset;
  view.setUint32(
    offset,
    LFH_SIGNATURE,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint16(
    offset,
    VERSION_NEEDED,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    FLAG_UTF8_FILENAME,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    COMPRESSION_STORE,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    entry.modified
      .time,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    entry.modified
      .date,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint32(
    offset,
    entry.crc,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint32(
    offset,
    entry.content
      .length,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint32(
    offset,
    entry.content
      .length,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint16(
    offset,
    entry.nameBytes
      .length,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    0,
    true,
  );
  offset += BYTES_UINT16;
  buffer.set(
    entry.nameBytes,
    offset,
  );
  offset += entry.nameBytes
    .length;
  buffer.set(
    entry.content,
    offset,
  );
  offset += entry.content
    .length;
  return offset;
}

/**
 Write all local file headers (and their data) into the archive buffer.
 
 @param view - DataView over the archive buffer
 
 @param buffer - Backing byte buffer (used for raw-byte writes)
 
 @param positioned - Entries with their pre-computed LFH offsets
 
 @param startOffset - Cursor at which to begin writing (always 0 for ZIP layout)
 
 @returns Cursor position after the last write
 
 @mutates view - `writeOneLocalFileHeader` delegates `view.setUint16` and `view.setUint32` writes
 
 @mutates buffer - `writeOneLocalFileHeader` delegates filename and content writes to `buffer.set`
 
 @example
 ```ts
 const lfhEnd = writeLocalFileHeaders({ view, buffer, positioned, startOffset: 0, },);
 ```
 */
export function writeLocalFileHeaders(
  {
    view,
    buffer,
    positioned,
    startOffset,
  }: ForeignBorrowed<Readonly<{
    view: DataView;
    buffer: Uint8Array;
    positioned: readonly Positioned[];
    startOffset: number;
  }>>,
): number {
  /**
   Running cursor advancing through successive local file headers.
   */
  let offset = startOffset;
  for (const { entry, } of positioned) {
    offset = writeOneLocalFileHeader({
      view,
      buffer,
      entry,
      startOffset: offset,
    },);
  }
  return offset;
}

/**
 Write a single central directory file header.
 
 @param view - DataView over the archive buffer
 
 @param buffer - Backing byte buffer (used for raw filename bytes)
 
 @param entry - Entry whose central directory header is being written
 
 @param lfhOffset - Pre-computed offset of the entry's local file header
 
 @param startOffset - Cursor at which to begin writing
 
 @returns Cursor position after the header
 
 @mutates view - `view.setUint16` and `view.setUint32` write directory fields
 
 @mutates buffer - `buffer.set` writes filename bytes
 */
function writeOneCentralDirectoryHeader(
  {
    view,
    buffer,
    entry,
    lfhOffset,
    startOffset,
  }: ForeignBorrowed<Readonly<{
    view: DataView;
    buffer: Uint8Array;
    entry: ZipEntry;
    lfhOffset: number;
    startOffset: number;
  }>>,
): number {
  /**
   Local cursor tracking each successive little-endian write.
   */
  let offset = startOffset;
  view.setUint32(
    offset,
    CDH_SIGNATURE,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint16(
    offset,
    VERSION_MADE_BY,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    VERSION_NEEDED,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    FLAG_UTF8_FILENAME,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    COMPRESSION_STORE,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    entry.modified
      .time,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint16(
    offset,
    entry.modified
      .date,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint32(
    offset,
    entry.crc,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint32(
    offset,
    entry.content
      .length,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint32(
    offset,
    entry.content
      .length,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint16(
    offset,
    entry.nameBytes
      .length,
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
    0,
    true,
  );
  offset += BYTES_UINT16;
  view.setUint32(
    offset,
    UNIX_FILE_MODE_DEFAULT,
    true,
  );
  offset += BYTES_UINT32;
  view.setUint32(
    offset,
    lfhOffset,
    true,
  );
  offset += BYTES_UINT32;
  buffer.set(
    entry.nameBytes,
    offset,
  );
  offset += entry.nameBytes
    .length;
  return offset;
}

/**
 Write all central directory file headers into the archive buffer.
 
 @param view - DataView over the archive buffer
 
 @param buffer - Backing byte buffer (used for raw filename bytes)
 
 @param positioned - Entries with their pre-computed LFH offsets
 
 @param startOffset - Cursor where the central directory begins
 
 @returns Cursor position after the last write
 
 @mutates view - `writeOneCentralDirectoryHeader` delegates `view.setUint16` and `view.setUint32` writes
 
 @mutates buffer - `writeOneCentralDirectoryHeader` delegates filename writes to `buffer.set`
 
 @example
 ```ts
 const cdEnd = writeCentralDirectory({ view, buffer, positioned, startOffset: lfhEnd, },);
 ```
 */
export function writeCentralDirectory(
  {
    view,
    buffer,
    positioned,
    startOffset,
  }: ForeignBorrowed<Readonly<{
    view: DataView;
    buffer: Uint8Array;
    positioned: readonly Positioned[];
    startOffset: number;
  }>>,
): number {
  /**
   Running cursor advancing through successive central directory headers.
   */
  let offset = startOffset;
  for (const {
    entry,
    lfhOffset,
  } of positioned) {
    offset = writeOneCentralDirectoryHeader({
      view,
      buffer,
      entry,
      lfhOffset,
      startOffset: offset,
    },);
  }
  return offset;
}
