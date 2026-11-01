/**
 Shared internal types for the ZIP writer.
 
 @module
 */

import type { DosDateTime, } from './dos-time.ts';

/**
 Internal record of an added file used by the serializer.
 */
export type ZipEntry = {
  /**
   UTF-8 encoded path bytes.
   */
  readonly nameBytes: Uint8Array;

  /**
   Raw file content bytes (stored uncompressed).
   */
  readonly content: Uint8Array;

  /**
   CRC-32 of `content`, precomputed at add time.
   */
  readonly crc: number;

  /**
   DOS modification timestamp.
   */
  readonly modified: DosDateTime;
};

/**
 Per-entry positioning bookkeeping for the offset-computation pass.
 */
export type Positioned = {
  /**
   The entry to be written.
   */
  readonly entry: ZipEntry;

  /**
   Byte offset of the entry's local file header within the archive.
   */
  readonly lfhOffset: number;
};
