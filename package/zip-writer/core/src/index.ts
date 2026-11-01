/**
 Minimal STORE-only ZIP writer.
 
 Implements the subset of the PKZIP/APPNOTE.txt specification needed to
 serialize a small number of stored files into a single archive: local
 file headers, central directory headers, end of central directory
 record. No compression, no Zip64, no encryption, no streaming, no
 reading.
 
 Why custom: the API is three calls (`new`, `add`, `build`) and writes
 only stored files. The write-only design has minimal attack surface:
 there is no parser, so no zip-slip, no decompression bombs, no
 malformed-header crashes from untrusted input. Zero runtime
 dependencies; runtime-neutral (browser, Node, Bun, Deno, Workers).
 
 Reference: PKWARE APPNOTE.txt v6.3.10 sections 4.3 (local file header
 + file data), 4.4 (central directory record), 4.5 (end of central
 directory record).
 
 @module @monochromatic-dev/zip-writer-core
 */

import {
  MAX_UINT16,
  MAX_UINT32,
} from './constants.ts';
import { crc32, } from './crc32.ts';
import {
  type DosDateTime,
  dosDateTime,
} from './dos-time.ts';
import { validatePath, } from './path.ts';
import {
  serializeEntries,
  type ZipEntry,
} from './serialize.ts';

export { crc32, } from './crc32.ts';
export {
  type DosDateTime,
  dosDateTime,
} from './dos-time.ts';

/**
 Reusable text encoder for filename and string content conversion.
 */
const TEXT_ENCODER = new TextEncoder();

/* oxlint-disable no-restricted-syntax/no-class -- ZipWriter is consumed via `new ZipWriter()` by package/figma/to-penpot (src/index.ts:1534); migrating to a factory requires a coordinated call-site update tracked in doc/migration/no-class.md, out of scope for this package-local lint sweep */
/**
 Builder for STORE-only ZIP archives. Add files with {@link ZipWriter.add}
 then call {@link ZipWriter.build} to produce the final byte sequence.
 
 Not thread-safe. Each instance produces a single archive; create a new
 instance for the next archive.
 
 @example
 ```ts
 const zip = new ZipWriter();
 zip.add('manifest.json', JSON.stringify({ version: 1, },),);
 zip.add('data/blob.bin', new Uint8Array([1, 2, 3,],),);
 const bytes = zip.build();
 ```
 */
export class ZipWriter {
  /**
   Insertion-ordered map of path to entry. Map gives duplicate detection.
   */
  readonly #entries = new Map<string, ZipEntry>();

  /**
   Default modification time applied to every added file.
   */
  readonly #defaultModified: DosDateTime;

  /**
   Create a new writer. Pass `modifiedAt` to override the default
   modification timestamp for entries (defaults to the current time);
   pass a fixed `Date` to produce reproducible output.
   */
  constructor({ modifiedAt = new Date(), }: Readonly<{ modifiedAt?: Date; }> = {},) {
    this.#defaultModified = dosDateTime(modifiedAt,);
  }

  /**
   Add a file to the archive.
   
   @param path - Path inside the archive (forward-slash delimited)
   
   @param content - File contents; strings are encoded as UTF-8
   
   @throws When `path` is invalid (see {@link validatePath}) or when an
     entry already exists at the same path
   */
  add(
    path: string,
    content: string | Uint8Array,
  ): void {
    validatePath(path,);
    if (this.#entries
      .has(path,))
      throw new Error(`zip-writer: duplicate entry at \`${path}\``,);
    /**
     UTF-8 path encoding shared between the size check and the stored entry.
     */
    const nameBytes = TEXT_ENCODER.encode(path,);
    if (nameBytes.length
      > MAX_UINT16) {
      throw new Error(
        `zip-writer: path too long when UTF-8 encoded (${nameBytes.length} bytes, max ${MAX_UINT16}): ${path}`,
      );
    }
    /**
     Raw byte view of the content so the size check works for both inputs.
     */
    const data = ((typeof content) === 'string')
      ? TEXT_ENCODER.encode(content,)
      : content;
    if (data.length
      > MAX_UINT32) {
      throw new Error(
        `zip-writer: file too large for legacy ZIP (${data.length} bytes, max ${MAX_UINT32}): ${path}`,
      );
    }
    this.#entries
      .set(
      path,
      {
        nameBytes,
        content: data,
        crc: crc32(data,),
        modified: this.#defaultModified,
      },
      );
  }

  /**
   Number of entries currently in the writer.
   
   @returns Count of entries added so far
   */
  get size(): number {
    return this.#entries
      .size;
  }

  /**
   Serialize all added files into a single ZIP byte sequence.
   
   @returns Newly allocated `Uint8Array` containing the archive
   
   @throws When the resulting archive would exceed legacy ZIP limits
     (≥ 65 535 entries or ≥ 4 GiB total) since Zip64 is not implemented
   */
  build(): Uint8Array {
    return serializeEntries(this.#entries,);
  }
}
/* oxlint-enable no-restricted-syntax/no-class */
