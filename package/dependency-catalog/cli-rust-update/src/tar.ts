/**
 Streaming reader for the ustar archive inside the crates.io database dump.

 A tar archive is a sequence of 512-byte header blocks, each followed by the
 entry's bytes padded to a 512-byte boundary, and ends with zero blocks. The
 reader takes decompressed chunks as they arrive, hands each wanted entry's
 bytes to a handler, skips the rest, and stops as soon as the caller has what
 it needs, so a caller reading only `crates.csv` and `versions.csv` never
 decompresses the multi-gigabyte tables after them.

 @module
 */

import { DumpFormatError, } from './csv.ts';

//region Layout

/**
 Size of a header block and of the padding unit.
 */
const BLOCK_SIZE = 512;

/**
 Byte range of the entry name in a ustar header.
 */
const NAME_FIELD = {
  start: 0,
  end: 100,
} as const;

/**
 Byte range of the octal entry size in a ustar header.
 */
const SIZE_FIELD = {
  start: 124,
  end: 136,
} as const;

/**
 Offset of the entry type flag in a ustar header.
 */
const TYPE_OFFSET = 156;

/**
 Byte range of the name prefix in a ustar header.
 */
const PREFIX_FIELD = {
  start: 345,
  end: 500,
} as const;

/**
 High bit that marks a base-256 size, used only for entries over 8 GiB.
 */
const BASE256_FLAG = 0x80;

/**
 Radix of the ustar size field.
 */
const OCTAL = 8;

/**
 Type flag of a regular file (`0`).
 */
const REGULAR_FILE_TYPE = 0x30;

/**
 Type flag old tar writers use for a regular file (NUL).
 */
const LEGACY_REGULAR_FILE_TYPE = 0x00;

/**
 Type flags of regular files.
 */
const REGULAR_FILE_TYPES: ReadonlySet<number> = new Set([
  REGULAR_FILE_TYPE,
  LEGACY_REGULAR_FILE_TYPE,
],);

//endregion Layout

//region Entries

/**
 One archive entry's header.

 @example
 ```ts
 const entry: TarEntry = { name: '2026-10-29-020020/data/crates.csv', size: 1322357069, regularFile: true, };
 ```
 */
export type TarEntry = {
  readonly name: string;
  readonly size: number;
  readonly regularFile: boolean;
};

/**
 Receives the bytes of one wanted entry.

 @example
 ```ts
 const handler: TarEntryHandler = { push: function push(bytes) {}, end: function end() {}, };
 ```
 */
export type TarEntryHandler = {
  readonly push: (bytes: Uint8Array,) => void;
  readonly end: () => void;
};

/**
 Answer to an entry header that skips the entry's bytes.
 */
export const SKIP_ENTRY: unique symbol = Symbol('tar: skip this entry',);

/**
 Answer to an entry header that ends reading before the entry.
 */
export const STOP_ARCHIVE: unique symbol = Symbol('tar: stop reading the archive',);

/**
 Decides what to do with an entry, given its header.

 @example
 ```ts
 const decide: TarEntryDecision = function decide(entry) { return SKIP_ENTRY; };
 ```
 */
export type TarEntryDecision = (entry: TarEntry,) => TarEntryHandler | typeof SKIP_ENTRY | typeof STOP_ARCHIVE;

//endregion Entries

//region Reader

/**
 Mutable reader state, kept in one object so the reader needs no function-level `let`.
 */
type TarState = {
  readonly header: Uint8Array;
  headerFill: number;
  remaining: number;
  padding: number;
  handler: TarEntryHandler | typeof SKIP_ENTRY;
  finished: boolean;
};

/**
 Text of a NUL-terminated header field.

 @param header - header block

 @param field - byte range

 @returns field text up to the first NUL

 @example
 ```ts
 headerText({ header, field: NAME_FIELD, });
 ```
 */
function headerText({
  header,
  field,
}: {
  readonly header: Uint8Array;
  readonly field: {
    readonly start: number;
    readonly end: number
  }
},): string {
  /**
   Field bytes.
   */
  const bytes = header.subarray(
    field.start,
    field.end,
  );
  /**
   Position of the terminating NUL, or the field end.
   */
  const nul = bytes.indexOf(0,);
  return new TextDecoder().decode(nul === (-1) ? bytes : bytes.subarray(
    0,
    nul,
  ),);
}

/**
 Entry described by a full header block.

 @param header - header block

 @returns entry header

 @throws DumpFormatError when the size uses the base-256 form or is not octal

 @example
 ```ts
 parseHeader(header);
 ```
 */
function parseHeader(header: Uint8Array,): TarEntry {
  if (((header[SIZE_FIELD.start] ?? 0) & BASE256_FLAG) !== 0)
    throw new DumpFormatError('a crates.io dump entry is larger than 8 GiB (base-256 tar size); the reader supports only octal sizes',);
  /**
   Octal size text.
   */
  const sizeText = headerText({
    header,
    field: SIZE_FIELD,
  },)
    .trim();
  /**
   Entry size.
   */
  const size = Number.parseInt(
    sizeText === '' ? '0' : sizeText,
    OCTAL,
  );
  if (Number.isNaN(size,))
    throw new DumpFormatError(`a crates.io dump entry has a size field that is not octal: ${JSON.stringify(sizeText,)}`,);
  /**
   Name prefix for long paths.
   */
  const prefix = headerText({
    header,
    field: PREFIX_FIELD,
  },);
  /**
   Name field.
   */
  const name = headerText({
    header,
    field: NAME_FIELD,
  },);
  return {
    name: prefix === '' ? name : `${prefix}/${name}`,
    size,
    regularFile: REGULAR_FILE_TYPES.has(header[TYPE_OFFSET] ?? 0,),
  };
}

/**
 Creates a push reader over decompressed tar bytes.

 @param decide - chooses, per entry, a handler, {@link SKIP_ENTRY}, or {@link STOP_ARCHIVE}

 @returns a `push` that takes the next chunk and returns false once reading should stop

 @throws DumpFormatError from `push` when a header is malformed

 @example
 ```ts
 const reader = createTarReader({ decide: function decide() { return SKIP_ENTRY; }, });
 for await (const chunk of stream) { if (!reader.push(chunk)) break; }
 ```
 */
export function createTarReader({ decide, }: { readonly decide: TarEntryDecision; },): { readonly push: (chunk: Uint8Array,) => boolean; } {
  /**
   Reader state.
   */
  const state: TarState = {
    header: new Uint8Array(BLOCK_SIZE,),
    headerFill: 0,
    remaining: 0,
    padding: 0,
    handler: SKIP_ENTRY,
    finished: false,
  };

  /**
   Consumes header bytes from `chunk` starting at `offset`.

   @param chunk - current chunk

   @param offset - first unread byte

   @returns next unread offset

   @example
   ```ts
   readHeader({ chunk, offset: 0, });
   ```
   */
  function readHeader({
    chunk,
    offset,
  }: {
    readonly chunk: Uint8Array;
    readonly offset: number
  },): number {
    /**
     Bytes of the header still missing.
     */
    const take = Math.min(
      BLOCK_SIZE - state.headerFill,
      chunk.length - offset,
    );
    state.header
      .set(
      chunk.subarray(
        offset,
        offset + take,
      ),
      state.headerFill,
      );
    state.headerFill += take;
    if (state.headerFill < BLOCK_SIZE)
      return offset + take;
    state.headerFill = 0;
    if (state.header
      .every(function isZero(byte: number,): boolean {
        return byte === 0;
      },)) {
      state.finished = true;
      return chunk.length;
    }
    /**
     Entry this header announces.
     */
    const entry = parseHeader(state.header,);
    /**
     Caller's decision; non-file entries (directories, pax headers) are always skipped.
     */
    const decision = entry.regularFile ? decide(entry,) : SKIP_ENTRY;
    if (decision === STOP_ARCHIVE) {
      state.finished = true;
      return chunk.length;
    }
    state.handler = decision;
    state.remaining = entry.size;
    state.padding = (BLOCK_SIZE - (entry.size % BLOCK_SIZE)) % BLOCK_SIZE;
    if ((entry.size === 0) && (decision !== SKIP_ENTRY))
      decision.end();
    return offset + take;
  }

  /**
   Consumes entry bytes from `chunk` starting at `offset`.

   @param chunk - current chunk

   @param offset - first unread byte

   @returns next unread offset

   @example
   ```ts
   readBody({ chunk, offset, });
   ```
   */
  function readBody({
    chunk,
    offset,
  }: {
    readonly chunk: Uint8Array;
    readonly offset: number
  },): number {
    /**
     Entry bytes available in this chunk.
     */
    const take = Math.min(
      state.remaining,
      chunk.length - offset,
    );
    if (state.handler !== SKIP_ENTRY)
      state.handler
        .push(chunk.subarray(
          offset,
          offset + take,
        ),);
    state.remaining -= take;
    if ((state.remaining === 0) && (state.handler !== SKIP_ENTRY))
      state.handler
        .end();
    return offset + take;
  }

  return {
    push: function push(chunk: Uint8Array,): boolean {
      /**
       Read position in the chunk.
       */
      const cursor = { offset: 0, };
      while ((cursor.offset < chunk.length) && (!state.finished)) {
        if (state.remaining > 0)
          cursor.offset = readBody({
            chunk,
            offset: cursor.offset,
          },);
        else if (state.padding > 0) {
          /**
           Padding bytes available in this chunk.
           */
          const skip = Math.min(
            state.padding,
            chunk.length - cursor.offset,
          );
          state.padding -= skip;
          cursor.offset += skip;
        }
        else
          cursor.offset = readHeader({
            chunk,
            offset: cursor.offset,
          },);
      }
      return !state.finished;
    },
  };
}

//endregion Reader
