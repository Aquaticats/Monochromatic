/**
 * Hex dump generator for binary file display.
 *
 * Produces output in the classic `xxd`/`hexdump -C` format:
 * offset, hex bytes grouped by 8, ASCII representation.
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';

/**
 * Hexadecimal radix for `toString` and `padStart`.
 */
const HEX_RADIX = 16;

/**
 * Width of the offset column in hex characters.
 */
const OFFSET_WIDTH = HEX_RADIX / 2;

/**
 * Number of bytes displayed per hex dump line.
 */
const BYTES_PER_LINE = HEX_RADIX;

/**
 * Group boundary for inserting an extra space between hex columns.
 */
const GROUP_BOUNDARY = OFFSET_WIDTH;

/**
 * Multiplier to derive max dump size from bytes per line.
 */
const DUMP_LINE_COUNT = 1_024;

/**
 * Maximum bytes to include in the dump before truncating.
 */
export const HEX_DUMP_MAX_BYTES: number = BYTES_PER_LINE * DUMP_LINE_COUNT;

/**
 * First printable ASCII code point (space).
 */
const ASCII_PRINTABLE_START = 0x20;

/**
 * Last printable ASCII code point (tilde).
 */
const ASCII_PRINTABLE_END = 0x7E;

/**
 * Generates a hex dump string from a binary buffer.
 * Output is truncated to {@link MAX_DUMP_BYTES} with a summary footer.
 *
 * @param buffer - raw file contents (may be a prefix when `totalSize` is provided)
 *
 * @param totalSize - actual file size for the truncation message when
 * `buffer` is a partial read; defaults to `buffer.length`
 *
 * @returns formatted hex dump string
 *
 * @example
 * ```ts
 * const dump = generateHexDump({ buffer: Buffer.from([0x48, 0x65, 0x6C, 0x6C, 0x6F]) });
 * // "00000000  48 65 6c 6c 6f                                    |Hello|"
 * ```
 */
export function generateHexDump(
  {
    buffer,
    totalSize,
  }: {
    readonly buffer: Buffer;
    readonly totalSize?: number;
  },
): string {
  /**
   * Original file size; differs from buffer length when caller pre-truncated.
   */
  const fullSize = totalSize ?? buffer
    .length;
  /**
   * Capped output length so very large buffers do not produce unbounded dumps.
   */
  const limit = Math.min(
    buffer.length,
    HEX_DUMP_MAX_BYTES,
  );
  /**
   * Accumulator joined with newlines as the final return.
   */
  const lines: string[] = [];

  for (let offset = 0; offset < limit; offset += BYTES_PER_LINE) {
    /**
     * Padded offset shown at the start of each dump row.
     */
    const offsetHex = offset.toString(HEX_RADIX,)
      .padStart(
      OFFSET_WIDTH,
      '0',
    );
    /**
     * Clamps the final row when the buffer is not a multiple of the row width.
     */
    const end = Math.min(
      offset + BYTES_PER_LINE,
      limit,
    );
    /**
     * Slice covering this row; up to {@link BYTES_PER_LINE} bytes.
     */
    const chunk = buffer.subarray(
      offset,
      end,
    );

    /**
     * Per-byte hex strings joined with spaces below.
     */
    const hexParts: string[] = [];
    /**
     * ASCII gutter built byte-by-byte alongside the hex parts.
     */
    let ascii = '';

    for (let i = 0; i < BYTES_PER_LINE; i++) {
      if (i === GROUP_BOUNDARY)
        hexParts.push('',);
      if (i < chunk
        .length) {
        /**
         * {@link nonNullishOrThrow} replaces the `!` operator banned by AGENTS.md.
         */
        const byte = nonNullishOrThrow(chunk[i],);
        hexParts.push(byte.toString(HEX_RADIX,)
          .padStart(
          2,
          '0',
        ),);
        ascii += ((byte >= ASCII_PRINTABLE_START) && (byte <= ASCII_PRINTABLE_END))
          ? String.fromCodePoint(byte,)
          : '.';
      }
      else {
        hexParts.push('  ',);
      }
    }

    lines.push(`${offsetHex}  ${hexParts.join(' ',)}  |${ascii}|`,);
  }

  if (fullSize > limit) {
    lines.push('',);
    lines.push(
      `... truncated (showing ${limit.toLocaleString()} of ${fullSize.toLocaleString()} bytes)`,
    );
  }

  return lines.join('\n',);
}
