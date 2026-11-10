/**
 * Smart-HTTP wire protocol parsers vendored from the gaps in
 * `isomorphic-git`'s server-side surface.
 *
 * isomorphic-git ships `parseUploadPackRequest` and `writeRefsAdResponse`
 * but does **not** ship server-side `parseReceivePackRequest`,
 * `writeReceivePackResponse`, `writeUploadPackResponse`, or sideband
 * multiplex (`mux` is commented out in `src/models/GitSideBand.js`).
 *
 * This file holds the parsers and shared types. Writers and the sideband
 * multiplexer live in `pack-protocol-writers.ts`.
 *
 * Spec references:
 *
 * - Documentation/gitprotocol-pack.txt in the git source tree
 * - https://git-scm.com/docs/protocol-v0
 * - isomorphic-git's `src/wire/parseUploadPackRequest.js` and
 *   `src/wire/writeRefsAdResponse.js` for the analogous client/server shapes
 */

import {
  decodePktLines,
  parsePktLenPrefix,
} from './pkt-line.ts';

export {
  multiplexSideband,
  type RefUpdateResult,
  SidebandChannels,
  writeReceivePackResponse,
  writeUploadPackResponse,
} from './pack-protocol-writers.ts';

/**
 * Pkt-line header is 4 hex digits (length-as-hex).
 */
const PKT_HEADER_BYTES = 4;

/**
 * Minimum number of whitespace-separated tokens in a receive-pack triplet (`old new ref`).
 */
const TRIPLET_MIN_TOKENS = 3;

/**
 * ASCII line-feed used as optional trailing terminator on pkt-line text payloads.
 */
const ASCII_LF = 0x0A;

/**
 * One ref-update triplet sent by the client (`oldOid newOid refName`).
 */
export type RefUpdateTriplet = {
  readonly oldOid: string;
  readonly newOid: string;
  readonly refName: string;
};

/**
 * Parsed `git-receive-pack` request body. The packfile bytes are
 * everything after the trailing flush-pkt; they may be empty if all
 * triplets are deletions.
 */
export type ReceivePackRequest = {
  /**
   * Capabilities advertised by the client (parsed from the first triplet).
   */
  readonly capabilities: readonly string[];
  /**
   * Ordered ref-update triplets `(old, new, refname)`.
   */
  readonly triplets: readonly RefUpdateTriplet[];
  /**
   * Raw pack data following the flush-pkt; empty when the request is delete-only.
   */
  readonly packfile: Uint8Array;
};

/**
 * Parsed `git-upload-pack` request body.
 *
 * @example
 * ```ts
 * const req = parseUploadPackBody(body);
 * for (const oid of req.wants) await sendObject(oid);
 * ```
 */
export type UploadPackRequest = {
  /**
   * Capabilities the client advertised in the first `want` line.
   */
  readonly capabilities: readonly string[];
  /**
   * OIDs the client wants to receive.
   */
  readonly wants: readonly string[];
  /**
   * OIDs the client already has (used for thin-pack negotiation).
   */
  readonly haves: readonly string[];
  /**
   * OIDs the client wants to mark as shallow.
   */
  readonly shallows: readonly string[];
  /**
   * Whether the client signalled the negotiation is complete.
   */
  readonly done: boolean;
};

/**
 * Decodes a `git-receive-pack` request body.
 *
 * Wire shape: ordered pkt-lines `(old new ref [\0caps])`, terminated by a
 * flush-pkt, optionally followed by raw pack bytes.
 *
 * @param body - concatenated request body bytes
 *
 * @returns parsed request
 *
 * @throws when the body is malformed (missing flush-pkt, malformed triplet)
 *
 * @example
 * ```ts
 * const { capabilities, triplets, packfile } = parseReceivePackBody(body);
 * ```
 */
export function parseReceivePackBody(body: Uint8Array,): ReceivePackRequest {
  /**
   * Reused decoder for the pkt-line length prefixes and text payloads.
   */
  const decoder = new TextDecoder();
  /**
   * Triplets collected in stream order; populated in the scan loop below.
   */
  const triplets: RefUpdateTriplet[] = [];
  /**
   * Streaming pkt-line scan result; isolated in an IIFE so the loop's mutable cursor never leaks to the surrounding function body.
   */
  const scanResult = (function scanReceivePackBody(): {
    capabilities: readonly string[];
    offset: number;
    sawFlush: boolean;
  } {
    /**
     * Capability list parsed from the first triplet's NUL-separated suffix.
     */
    let capabilities: readonly string[] = [];
    /**
     * Cursor advancing through `body` as each pkt-line is consumed.
     */
    let offset = 0;
    /**
     * Sentinel flipped when the protocol's flush-pkt is observed.
     */
    let sawFlush = false;
    while (offset < body
      .byteLength) {
      if ((body.byteLength
        - offset) < PKT_HEADER_BYTES)
        throw new Error('receive-pack: truncated pkt-len header',);
      /**
       * Length prefix string for the current pkt-line.
       */
      const lengthHex = decoder.decode(body.subarray(
        offset,
        offset + PKT_HEADER_BYTES,
      ),);
      /**
       * Decoded total pkt-line length including header.
       */
      const length = parsePktLenPrefix(lengthHex,);
      if (length === 0) {
        sawFlush = true;
        offset += PKT_HEADER_BYTES;
        break;
      }
      if (length < PKT_HEADER_BYTES)
        throw new Error(`receive-pack: invalid pkt-len ${String(length,)}`,);
      if ((offset + length) > body
        .byteLength)
        throw new Error('receive-pack: pkt-line overruns body',);
      /**
       * Payload bytes of the current pkt-line, length prefix excluded.
       */
      const payload = body.subarray(
        offset + PKT_HEADER_BYTES,
        offset + length,
      );
      offset += length;
      /**
       * Payload without the optional trailing line-feed git emits on text lines.
       */
      const trimmed = payload[payload.byteLength
        - 1]
        === ASCII_LF
        ? payload.subarray(
          0,
          payload.byteLength
            - 1,
        )
        : payload;
      /**
       * UTF-8 decoded line text.
       */
      const text = decoder.decode(trimmed,);
      /**
       * NUL-separated split: index 0 is the triplet, index 1 the capability suffix.
       */
      const nullSplit = text.split('\0',);
      /**
       * Triplet portion of the line ("old new ref").
       */
      const tripletText = nullSplit[0]
        ?? '';
      if ((triplets.length
        === 0) && (nullSplit.length
          >= 2)) {
        /**
         * Raw capability string from the first triplet line.
         */
        const capsText = nullSplit[1]
          ?? '';
        capabilities = capsText.length
          === 0
          ? []
          : capsText.split(' ',)
            .filter(function nonEmpty(s,) {
            return s.length
              > 0;
          },);
      }
      /**
       * Whitespace-separated triplet tokens; ref names may contain spaces.
       */
      const parts = tripletText.split(' ',);
      if (parts.length
        < TRIPLET_MIN_TOKENS)
        throw new Error(`receive-pack: malformed triplet "${tripletText}"`,);
      /**
       * Destructured triplet; ref names may contain embedded spaces.
       */
      const [oldOid, newOid, ...refTokens] = parts;
      triplets.push({
        oldOid: oldOid ?? '',
        newOid: newOid ?? '',
        refName: refTokens.join(' ',),
      },);
    }
    return {
      capabilities,
      offset,
      sawFlush,
    };
  })();
  if (!scanResult.sawFlush)
    throw new Error('receive-pack: missing flush-pkt before pack data',);
  /**
   * Pack bytes following the flush-pkt; empty for delete-only requests.
   */
  const packfile = new Uint8Array(body.subarray(scanResult.offset,),);
  return {
    capabilities: scanResult.capabilities,
    triplets,
    packfile,
  };
}

/**
 * Decodes a `git-upload-pack` request body.
 *
 * Wire shape: ordered pkt-lines `("want " <oid> [<caps>])`,
 * `("have " <oid>)`, optional `shallow` / `deepen-*`, terminated either
 * by a flush-pkt or by `"done"`.
 *
 * @param body - concatenated request body bytes
 *
 * @returns parsed request
 *
 * @example
 * ```ts
 * const req = parseUploadPackBody(body);
 * ```
 */
export function parseUploadPackBody(body: Uint8Array,): UploadPackRequest {
  /**
   * Reused decoder for the pkt-line text payloads.
   */
  const decoder = new TextDecoder();
  /**
   * OIDs the client requested; populated by `want` lines in the scan loop.
   */
  const wants: string[] = [];
  /**
   * OIDs the client already has; populated by `have` lines in the scan loop.
   */
  const haves: string[] = [];
  /**
   * OIDs flagged shallow; populated by `shallow` lines in the scan loop.
   */
  const shallows: string[] = [];
  /**
   * Streaming pkt-line scan result; isolated in an IIFE so the mutable accumulators stay scoped to the loop.
   */
  const scanResult = (function scanUploadPackBody(): {
    capabilities: readonly string[];
    done: boolean;
  } {
    /**
     * Capabilities advertised on the first `want` line.
     */
    let capabilities: readonly string[] = [];
    /**
     * Set when the client signalled negotiation completion via `done`.
     */
    let done = false;
    for (const line of decodePktLines(body,)) {
      if ((line === null) || (line === 'delim'))
        continue;
      /**
       * Payload without the optional trailing line-feed git emits on text lines.
       */
      const trimmed = line[line.byteLength
        - 1]
        === ASCII_LF
        ? line.subarray(
          0,
          line.byteLength
            - 1,
        )
        : line;
      /**
       * UTF-8 decoded line text.
       */
      const text = decoder.decode(trimmed,)
        .trim();
      if (text === 'done') {
        done = true;
        continue;
      }
      /**
       * Space-separated tokens; key/value/rest decide the line's meaning.
       */
      const tokens = text.split(' ',);
      /**
       * Destructured tokens: `key` selects the request kind, `rest` holds caps.
       */
      const [key, value, ...rest] = tokens;
      if ((capabilities.length
        === 0) && (rest.length
          > 0)) {
        capabilities = rest.filter(function nonEmpty(s,) {
          return s.length
            > 0;
        },);
      }
      if ((value === undefined) || (value === ''))
        continue;
      if (key === 'want')
        wants.push(value,);
      else if (key === 'have')
        haves.push(value,);
      else if (key === 'shallow')
        shallows.push(value,);
    }
    return {
      capabilities,
      done,
    };
  })();
  return {
    capabilities: scanResult.capabilities,
    wants,
    haves,
    shallows,
    done: scanResult.done,
  };
}
