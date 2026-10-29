/**
 UUID generation for the Figma-to-Penpot converter.
 
 Produces real v4 UUIDs via `crypto.randomUUID` and a deterministic
 counter-based fallback, plus stable UUIDs derived from a Figma GUID so the
 same node always maps to the same Penpot id across passes.
 
 @module figma-to-penpot-uuid
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import type { Uuid, } from './types.ts';

/**
 Mutable counter cell feeding the synthetic-UUID fallback; a one-key object
 keeps the mutation off module root (no module-root `let`).
 */
const syntheticCounter: { value: number; } = { value: 0, };

/**
 Advance and read the synthetic-UUID fallback counter.
 
 @returns next monotonic counter value
 
 @example
 ```ts
 const n = nextSyntheticCounter();
 ```
 */
function nextSyntheticCounter(): number {
  syntheticCounter.value += 1;
  return syntheticCounter.value;
}

/* oxlint-disable eslint/no-magic-numbers, unicorn/prefer-math-trunc -- UUID v4 bit-layout: the masks, shifts, segment widths, hex radix, and the `>>> 0` uint32 coercion below are the literal RFC 4122 field geometry; Math.trunc would drop the unsigned wrap, and naming each constant would obscure the byte layout */

/**
 Format `value` as zero-padded lowercase hex of `width` digits.
 
 @param value - non-negative integer to encode
 
 @param width - minimum digit count, left-padded with `0`
 
 @returns hex string of at least `width` characters
 
 @example
 ```ts
 toHexPadded({ value: 255, width: 4, }); // "00ff"
 ```
 */
export function toHexPadded(
  {
    value,
    width,
  }: Readonly<{
    value: number;
    width: number;
  }>,
): string {
  return (value >>> 0).toString(16,)
    .padStart(
    width,
    '0',
    );
}

/**
 Generate a unique UUID v4.
 
 Uses `crypto.randomUUID` when available and falls back to a counter-derived
 synthetic UUID on platforms without it.
 
 @returns fresh UUID v4 string
 
 @example
 ```ts
 const id = nextUuid();
 ```
 */
export function nextUuid(): Uuid {
  /**
   Counter snapshot taken every call so the synthetic fallback stays monotonic even when `crypto.randomUUID` is the active path.
   */
  const c = nextSyntheticCounter();
  try {
    return crypto.randomUUID();
  }
  catch (error) {
    console.warn(`[figma-penpot] crypto.randomUUID failed, using deterministic fallback: ${caughtValueText(error,)}`,);
    /**
     First 8-hex segment (low 32 bits of the counter).
     */
    const a = toHexPadded({
      value: c & 0xFF_FF_FF_FF,
      width: 8,
    },);
    /**
     Second 4-hex segment (bits 32-47 of the counter).
     */
    const b = toHexPadded({
      value: (c >> 32) & 0xFF_FF,
      width: 4,
    },);
    /**
     3-hex tail shared by the version and variant segments.
     */
    const tail = toHexPadded({
      value: c & 0xF_FF,
      width: 3,
    },)
      .slice(-3,);
    /**
     Final 12-hex node segment composed from the remaining counter bits.
     */
    const node = `${
      toHexPadded({
        value: c & 0xFF_FF,
        width: 4,
      },)
    }${
      toHexPadded({
        value: (c >> 4) & 0xFF_FF,
        width: 4,
      },)
    }${
      toHexPadded({
        value: (c >> 8) & 0xFF_FF,
        width: 4,
      },)
    }`;
    return `${a}-${b}-4${tail}-8${tail}-${node}`.slice(
      0,
      36,
    );
  }
}

/**
 Generate a stable UUID from a Figma GUID (sessionID + localID).
 
 Deterministic so parents and children resolve to the same id across passes.
 
 @param sessionId - Figma session id component of the GUID
 
 @param localId - Figma local id component of the GUID
 
 @returns stable UUID v4 string encoding both id components
 
 @example
 ```ts
 const id = guidToUuid({ sessionId: 0, localId: 12, });
 ```
 */
export function guidToUuid(
  {
    sessionId,
    localId,
  }: Readonly<{
    sessionId: number;
    localId: number;
  }>,
): Uuid {
  /**
   First 8-hex segment: full session id so nodes from one session cluster.
   */
  const a = toHexPadded({
    value: sessionId,
    width: 8,
  },);
  /**
   Second 4-hex segment: high bits of the local id.
   */
  const b = toHexPadded({
    value: (localId >>> 16) & 0xFF_FF,
    width: 4,
  },);
  /**
   Version segment: literal `'4'` marker plus 3 hex digits of the local id.
   */
  const version = `4${
    toHexPadded({
      value: (localId >>> 4) & 0xF_FF,
      width: 3,
    },)
  }`;
  /**
   Variant nibble forced into the 8-B range for a well-formed v4 UUID.
   */
  const variantNibble = ((localId & 0xF) | 0x8).toString(16,);
  /**
   Variant segment: variant nibble plus 3 hex digits of the session id.
   */
  const variant = `${variantNibble}${
    toHexPadded({
      value: (sessionId >>> 16) & 0xF_FF,
      width: 3,
    },)
  }`;
  /**
   Final 12-hex node segment encoding the remaining bits of both ids losslessly.
   */
  const node = `${
    toHexPadded({
      value: sessionId & 0xFF_FF,
      width: 4,
    },)
  }${
    toHexPadded({
      value: (localId >>> 8) & 0xFF,
      width: 2,
    },)
  }${
    toHexPadded({
      value: localId & 0xFF,
      width: 2,
    },)
  }${
    toHexPadded({
      value: (sessionId >>> 8) & 0xFF,
      width: 2,
    },)
  }${
    toHexPadded({
      value: (sessionId >>> 24) & 0xFF,
      width: 2,
    },)
  }`;
  return `${a}-${b}-${version}-${variant}-${node}`.slice(
    0,
    36,
  );
}

/* oxlint-enable eslint/no-magic-numbers, unicorn/prefer-math-trunc */
