/**
 * `git-upload-pack` and `git-receive-pack` response writers, plus the
 * sideband-64k multiplexer they share.
 *
 * Vendored from `isomorphic-git` (the package only ships server-side
 * `parseUploadPackRequest` and `writeRefsAdResponse`; everything here
 * is missing). Split out of `pack-protocol.ts` for the max-lines budget.
 */

import {
  encodePkt,
  flushPkt,
  MAX_PKT_LEN_PAYLOAD,
  PKT_LEN_BYTES,
} from './pkt-line.ts';

/**
 * Sideband channel for pack data.
 */
const SIDEBAND_CHANNEL_PACK = 0x01;

/**
 * Sideband channel for progress messages (printed to stderr by the client).
 */
const SIDEBAND_CHANNEL_PROGRESS = 0x02;

/**
 * Sideband channel for fatal errors.
 */
const SIDEBAND_CHANNEL_ERROR = 0x03;

/**
 * Largest total pkt-line length a client that negotiated plain side-band
 * accepts: gitprotocol-pack allows "up to 999 data bytes plus 1 control code,
 * for a total of up to 1000 bytes in a pkt-line".
 */
const SIDEBAND_PLAIN_PKT_LEN_LIMIT = 1_000;

/**
 * Subtracted from the payload ceilings above to leave room for the 1-byte channel marker.
 */
const SIDEBAND_CHANNEL_MARKER_BYTES = 1;

/**
 * Maximum bytes of pack data (excluding the 1-byte channel marker) per
 * side-band-64k pkt-line: the pkt-line payload ceiling minus the channel byte.
 */
const MAX_SIDEBAND_64K_PAYLOAD = MAX_PKT_LEN_PAYLOAD - SIDEBAND_CHANNEL_MARKER_BYTES;

/**
 * Maximum bytes of pack data per plain (non-64k) side-band pkt-line.
 */
const MAX_SIDEBAND_PAYLOAD = SIDEBAND_PLAIN_PKT_LEN_LIMIT
  - PKT_LEN_BYTES
  - SIDEBAND_CHANNEL_MARKER_BYTES;

/**
 * Per-ref result from a `git-receive-pack` apply.
 */
export type RefUpdateResult = {
  readonly refName: string;
  readonly ok: boolean;
  readonly error?: string;
};

/**
 * Splits `payload` into sideband-multiplexed pkt-lines on the given channel.
 *
 * @param row - inputs
 *
 * @returns ordered pkt-lines, ready to be concatenated and written to the response
 *
 * @example
 * ```ts
 * for (const chunk of multiplexSideband({ payload, channel: 1, useSideBand64k: true })) {
 *   await response.write(chunk);
 * }
 * ```
 */
export function multiplexSideband(row: {
  readonly payload: Uint8Array;
  readonly channel: number;
  readonly useSideBand64k: boolean;
},): Uint8Array[] {
  /**
   * Per-frame payload ceiling depends on which sideband flavour was negotiated.
   */
  const max = row.useSideBand64k ? MAX_SIDEBAND_64K_PAYLOAD : MAX_SIDEBAND_PAYLOAD;
  /**
   * Output frames produced as the payload is sliced.
   */
  const out: Uint8Array[] = [];
  /**
   * Read position advancing through `row.payload`.
   */
  let cursor = 0;
  while (cursor
    < row
    .payload
    .byteLength) {
    /**
     * Next chunk of payload bytes for one sideband frame.
     */
    const slice = row.payload
      .subarray(
      cursor,
      cursor + max,
    );
    cursor += slice.byteLength;
    /**
     * Frame buffer with the channel marker prepended to the slice.
     */
    const wrapped = new Uint8Array(slice.byteLength
      + 1,);
    wrapped[0] = row.channel;
    wrapped.set(
      slice,
      1,
    );
    out.push(encodePkt(wrapped,),);
  }
  return out;
}

/**
 * Builds a complete `git-upload-pack` response stream.
 *
 * Layout:
 *
 * 1. `PKT-LINE("NAK\n")`: standard "no negotiated common bases"
 *    answer for the simplest case (no haves).
 * 2. If sideband requested, the packfile is multiplexed onto channel 1;
 *    progress lines (if any) onto channel 2. Without sideband, the
 *    packfile is emitted raw after the NAK.
 *
 * @param row - inputs
 *
 * @returns ordered byte chunks; concatenate to send
 *
 * @example
 * ```ts
 * const chunks = writeUploadPackResponse({ packfile, useSideBand: true, useSideBand64k: true });
 * for (const chunk of chunks) await response.write(chunk);
 * ```
 */
export function writeUploadPackResponse(row: {
  readonly packfile: Uint8Array;
  readonly useSideBand: boolean;
  readonly useSideBand64k: boolean;
  readonly progress?: string;
},): Uint8Array[] {
  /**
   * Response chunks; starts with the mandatory NAK acknowledging no common bases.
   */
  const chunks: Uint8Array[] = [encodePkt('NAK\n',),];
  if (row.useSideBand) {
    if ((row.progress
      !== undefined) && (row.progress
        .length
        > 0)) {
      /**
       * Fresh encoder reused only for the progress payload.
       */
      const encoder = new TextEncoder();
      chunks.push(...multiplexSideband({
        payload: encoder.encode(row.progress,),
        channel: SIDEBAND_CHANNEL_PROGRESS,
        useSideBand64k: row.useSideBand64k,
      },),);
    }
    chunks.push(...multiplexSideband({
      payload: row.packfile,
      channel: SIDEBAND_CHANNEL_PACK,
      useSideBand64k: row.useSideBand64k,
    },),);
    chunks.push(flushPkt(),);
    return chunks;
  }
  chunks.push(row.packfile,);
  return chunks;
}

/**
 * Builds a `git-receive-pack` status report.
 *
 * Layout (no sideband):
 *
 * 1. `PKT-LINE("unpack ok\n")` or `PKT-LINE("unpack <error>\n")`
 * 2. Per ref: `PKT-LINE("ok <ref>\n")` or `PKT-LINE("ng <ref> <reason>\n")`
 * 3. flush-pkt
 *
 * Layout (sideband / sideband-64k requested by client):
 *
 * 1. The bytes of the no-sideband layout above are concatenated into
 *    one buffer
 * 2. {@link multiplexSideband} emits them as one or more pkt-lines on
 *    channel 1 (data)
 * 3. An outer flush-pkt terminates the sideband stream
 *
 * The system git CLI requires the wrapped form whenever it sees
 * `side-band` or `side-band-64k` in the negotiated capabilities; an
 * unwrapped report is parsed as raw sideband bytes and rejected with
 * `protocol error: bad band #<n>`.
 *
 * @param row - inputs
 *
 * @returns ordered byte chunks; concatenate to send
 *
 * @example
 * ```ts
 * const chunks = writeReceivePackResponse({
 *   unpackOk: true,
 *   refResults: [{ refName: 'refs/heads/main', ok: true }],
 *   useSideBand: true,
 *   useSideBand64k: true,
 * });
 * ```
 */
export function writeReceivePackResponse(row: {
  readonly unpackOk: boolean;
  readonly unpackError?: string;
  readonly refResults: readonly RefUpdateResult[];
  /**
   * Whether the client negotiated `side-band` or `side-band-64k`.
   */
  readonly useSideBand?: boolean;
  /**
   * Whether the client negotiated `side-band-64k` specifically.
   */
  readonly useSideBand64k?: boolean;
},): Uint8Array[] {
  /**
   * Status report chunks; sideband wrapping is applied later when negotiated.
   */
  const report: Uint8Array[] = [
    row.unpackOk
      ? encodePkt('unpack ok\n',)
      : encodePkt(`unpack ${row.unpackError
        ?? 'failed'}\n`,),
  ];
  for (const result of row.refResults) {
    if (result.ok)
      report.push(encodePkt(`ok ${result.refName}\n`,),);
    else
      report.push(encodePkt(`ng ${result.refName} ${result.error
        ?? 'failed'}\n`,),);
  }
  report.push(flushPkt(),);
  if (row.useSideBand
    !== true)
    return report;
  /**
   * Running sum of report chunk lengths to size the flattened buffer.
   */
  const total = report.reduce(
    function sumByteLength(
      sum: number,
      chunk: Uint8Array,
    ): number {
      return sum + chunk
        .byteLength;
    },
    0,
  );
  /**
   * Flattened report bytes; sideband multiplexing operates on a single buffer.
   */
  const flat = new Uint8Array(total,);
  report.reduce(
    function writeChunkAt(
      cursor: number,
      chunk: Uint8Array,
    ): number {
      flat.set(
        chunk,
        cursor,
      );
      return cursor + chunk
        .byteLength;
    },
    0,
  );
  return [
    ...multiplexSideband({
      payload: flat,
      channel: SIDEBAND_CHANNEL_PACK,
      useSideBand64k: row.useSideBand64k
        === true,
    },),
    flushPkt(),
  ];
}

/**
 * Sideband channel constants exported for tests and `iso-server.ts`.
 */
export const SidebandChannels: {
  readonly PACK: number;
  readonly PROGRESS: number;
  readonly ERROR: number;
} = {
  PACK: SIDEBAND_CHANNEL_PACK,
  PROGRESS: SIDEBAND_CHANNEL_PROGRESS,
  ERROR: SIDEBAND_CHANNEL_ERROR,
} as const;
