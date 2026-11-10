/**
 * pkt-line codec for the git smart-HTTP protocol.
 *
 * pkt-line is git's framing format: a 4-byte hex length prefix
 * (length includes the prefix itself) followed by the payload.
 * The special case `"0000"` is the flush-pkt; `"0001"` is the
 * delim-pkt. Maximum data payload is 65516 bytes (so total length
 * fits in 65520 bytes).
 *
 * Format reference: `Documentation/gitprotocol-pack.txt` in the git
 * source tree, and isomorphic-git's `src/models/GitPktLine.js`.
 *
 * Vendored from isomorphic-git so the server can both encode wire
 * frames (it doesn't expose `mux`/`encode`) and decode incoming
 * `git-receive-pack` and `git-upload-pack` request bodies (it
 * doesn't expose these decoders either; see
 * `server/TROUBLESHOOTING.isomorphic-git.md`).
 */

/**
 * Hexadecimal radix for pkt-len conversion.
 */
const HEX_RADIX = 16;

/**
 * Number of hex digits in the pkt-len prefix.
 */
export const PKT_LEN_BYTES = 4;

/**
 * Largest total pkt-line length, prefix included. gitprotocol-common: "The
 * maximum length of a pkt-line's data component is 65516 bytes. Implementations
 * MUST NOT send pkt-line whose length exceeds 65520 (65516 bytes of payload + 4
 * bytes of length data)".
 */
const MAX_PKT_LEN_TOTAL = 65_520;

/**
 * Largest payload one pkt-line may carry; callers with more bytes must split
 * them across frames, because a longer prefix needs a fifth hex digit and the
 * frame buffer cannot hold it.
 */
export const MAX_PKT_LEN_PAYLOAD = MAX_PKT_LEN_TOTAL - PKT_LEN_BYTES;

/**
 * A flush-pkt is `"0000"`.
 */
const FLUSH_PKT = new TextEncoder().encode('0000',);

/**
 * A delim-pkt is `"0001"`.
 */
const DELIM_PKT = new TextEncoder().encode('0001',);

/**
 * Special pkt-len value indicating flush.
 */
const FLUSH_LEN = 0;

/**
 * Special pkt-len value indicating delim.
 */
const DELIM_LEN = 1;

/**
Lowercase hexadecimal digits, indexed by the value each one denotes.
Each digit of a pkt-len prefix is looked up here so a character outside
the alphabet fails instead of ending the conversion early.
*/
const HEX_DIGITS = '0123456789abcdef';

/**
Value {@link HEX_DIGITS}.indexOf returns for a character outside the alphabet.
*/
const DIGIT_NOT_FOUND = -1;

/**
Parse one 4-byte pkt-len prefix into the frame length it states.

Every decoder in the server reads its frames through this function, because
`Number.parseInt` stops at the first character it cannot read: a prefix such as
`"000z"` would decode as the flush sentinel and silently truncate the rest of
the frame stream instead of reporting the corrupt frame.

@param prefixText - ASCII text of the length prefix, exactly 4 bytes

@returns frame length including the prefix itself

@throws Error quoting the prefix when it is not four hexadecimal digits

@example
```ts
parsePktLenPrefix('0005'); // 5
```
*/
export function parsePktLenPrefix(prefixText: string,): number {
  /**
  Frame length accumulated across the prefix's digits.
  */
  let length = 0;
  for (const character of prefixText.toLowerCase()) {
    /**
    Position of this character in the digit alphabet, or -1 when it is not one.
    */
    const digitValue = HEX_DIGITS.indexOf(character,);
    if (digitValue === DIGIT_NOT_FOUND) {
      throw new Error(
        `pkt-line: length prefix "${prefixText}" is not four hexadecimal digits`,
      );
    }
    length = (length * HEX_RADIX) + digitValue;
  }
  return length;
}

/**
 * Returns the flush-pkt sentinel (`"0000"`).
 *
 * @returns 4 bytes of ASCII `"0000"`
 *
 * @example
 * ```ts
 * flushPkt(); // <Uint8Array 30 30 30 30>
 * ```
 */
export function flushPkt(): Uint8Array {
  return new Uint8Array(FLUSH_PKT,);
}

/**
 * Returns the delim-pkt sentinel (`"0001"`).
 *
 * @returns 4 bytes of ASCII `"0001"`
 *
 * @example
 * ```ts
 * delimPkt();
 * ```
 */
export function delimPkt(): Uint8Array {
  return new Uint8Array(DELIM_PKT,);
}

/**
 * Encodes one pkt-line: prepends the hex length prefix to `payload`.
 *
 * @param payload - line content (bytes or UTF-8 string); at most 65516 bytes
 *
 * @returns full pkt-line: 4-byte length prefix concatenated with payload
 *
 * @throws Error when the payload needs more bytes than one frame can state
 *
 * @example
 * ```ts
 * encodePkt('hello\n');
 * ```
 */
export function encodePkt(payload: Uint8Array | string,): Uint8Array {
  /**
   * Payload bytes; strings are UTF-8 encoded here so callers may pass either.
   */
  const bytes = ((typeof payload) === 'string')
    ? new TextEncoder().encode(payload,)
    : payload;
  if (bytes.byteLength > MAX_PKT_LEN_PAYLOAD) {
    throw new Error(
      `pkt-line: payload of ${String(bytes.byteLength,)} bytes exceeds the ${
        String(MAX_PKT_LEN_PAYLOAD,)
      }-byte maximum; split it across several pkt-lines`,
    );
  }
  /**
   * Total frame length includes the length prefix itself per the spec.
   */
  const total = bytes.byteLength
    + PKT_LEN_BYTES;
  /**
   * Four-digit hex length prefix, zero-padded.
   */
  const lengthHex = total.toString(HEX_RADIX,)
    .padStart(
    PKT_LEN_BYTES,
    '0',
  );
  /**
   * ASCII bytes of the length prefix written at offset 0 of the frame.
   */
  const prefix = new TextEncoder().encode(lengthHex,);
  /**
   * Destination buffer sized to fit prefix plus payload.
   */
  const out = new Uint8Array(total,);
  out.set(
    prefix,
    0,
  );
  out.set(
    bytes,
    PKT_LEN_BYTES,
  );
  return out;
}

/**
 * Decoded pkt-line. `null` represents a flush-pkt; `'delim'` represents a delim-pkt;
 * a `Uint8Array` is a regular data line (without the length prefix).
 */
export type PktLine = Uint8Array | null | 'delim';

/**
 * Decodes the `data` byte stream into a list of pkt-lines.
 *
 * @param data - concatenated pkt-line stream
 *
 * @returns parsed pkt-line list
 *
 * @example
 * ```ts
 * decodePktLines(buffer);
 * ```
 */
export function decodePktLines(data: Uint8Array,): PktLine[] {
  /**
   * Reused decoder for the ASCII length prefixes.
   */
  const decoder = new TextDecoder();
  /**
   * Output accumulator visited in stream order.
   */
  const out: PktLine[] = [];
  /**
   * Cursor advancing through `data` as each frame is consumed.
   */
  let offset = 0;
  while (offset < data
    .byteLength) {
    if ((data.byteLength
      - offset) < PKT_LEN_BYTES) {
      throw new Error(
        `pkt-line decode: trailing ${
          String(data.byteLength
            - offset,)
        } bytes too short for length prefix`,
      );
    }
    /**
     * Length prefix bytes for the upcoming frame.
     */
    const lengthSlice = data.subarray(
      offset,
      offset + PKT_LEN_BYTES,
    );
    /**
     * Decoded ASCII length string parsed below as hex.
     */
    const lengthText = decoder.decode(lengthSlice,);
    /**
     * Frame length including the prefix; flush/delim are special low values.
     */
    const length = parsePktLenPrefix(lengthText,);
    if (length === FLUSH_LEN) {
      out.push(null,);
      offset += PKT_LEN_BYTES;
      continue;
    }
    if (length === DELIM_LEN) {
      out.push('delim',);
      offset += PKT_LEN_BYTES;
      continue;
    }
    if (length < PKT_LEN_BYTES)
      throw new Error(`pkt-line decode: length ${String(length,)} below header size`,);
    if ((offset + length) > data
      .byteLength) {
      throw new Error(
        `pkt-line decode: line claims ${String(length,)} bytes but only ${
          String(data.byteLength
            - offset,)
        } remain`,
      );
    }
    /**
     * Frame payload sliced from `data` after the length prefix.
     */
    const payload = data.subarray(
      offset + PKT_LEN_BYTES,
      offset + length,
    );
    out.push(new Uint8Array(payload,),);
    offset += length;
  }
  return out;
}
