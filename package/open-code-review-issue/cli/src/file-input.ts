/**
 Named-file transport for strict OCR structured input.
 
 @module
 */

import { readFile, } from 'node:fs/promises';

import { parseStructuredInput, } from './ingest.ts';
import { InputValidationError, } from './input-validation-error.ts';
import type { NormalizedInput, } from './model.ts';

/**
 Byte count needed to inspect longest forbidden BOM.
 */
const LONGEST_BOM_BYTES = 3;

/**
 Hexadecimal prefixes for UTF-8 and UTF-16 byte-order marks.
 */
const FORBIDDEN_BOM_PREFIXES: readonly string[] = [
  'efbbbf',
  'fffe',
  'feff',
];

/**
 Rejects byte-order marks before strict decoding.
 
 @param bytes - Exact named-file bytes.
 
 @param path - Input path used in diagnostic evidence.
 
 @throws {@link InputValidationError} when bytes start with known BOM.
 
 @example
 ```ts
 rejectByteOrderMark({ bytes: Buffer.from('[]'), path: 'review.json' });
 ```
 */
function rejectByteOrderMark({
  bytes,
  path,
}: {
  readonly bytes: Uint8Array;
  readonly path: string;
},): void {
  /**
   Leading bytes encoded for prefix comparison.
   */
  const leadingHex = Buffer.from(bytes.subarray(
    0,
    LONGEST_BOM_BYTES,
  ),)
    .toString('hex',);
  if (FORBIDDEN_BOM_PREFIXES.some(function startsWithBom(prefix,): boolean {
    return leadingHex.startsWith(prefix,);
  },)) {
    throw new InputValidationError(`input file ${path} starts with a forbidden byte-order mark`,);
  }
}

/**
 Decodes exact bytes as strict UTF-8 without replacement characters.
 
 @param bytes - Named-file bytes after BOM validation.
 
 @param path - Input path used in diagnostic evidence.
 
 @returns Decoded structured text.
 
 @throws {@link InputValidationError} when bytes are malformed UTF-8.
 
 @example
 ```ts
 decodeUtf8({ bytes: Buffer.from('[]'), path: 'review.json' }); // '[]'
 ```
 */
function decodeUtf8({
  bytes,
  path,
}: {
  readonly bytes: Uint8Array;
  readonly path: string;
},): string {
  try {
    return new TextDecoder(
      'utf-8',
      {
        fatal: true,
        ignoreBOM: true,
      },
    ).decode(bytes,);
  }
  catch (error: unknown) {
    throw new InputValidationError(`input file ${path} is not strict UTF-8: ${String(error,)}`,);
  }
}

/**
 Reads and atomically validates one named OCR input file.
 
 @param path - Named file path supplied positionally by caller.
 
 @returns Normalized structured findings.
 
 @throws {@link InputValidationError} when encoding or structured input is invalid.
 
 @example
 ```ts
 const input = await readStructuredInputFile({ path: 'review.json' });
 ```
 */
export async function readStructuredInputFile({
  path,
}: {
  readonly path: string;
},): Promise<NormalizedInput> {
  /**
   Exact bytes read without implicit text decoding.
   */
  const bytes = await readFile(path,);
  rejectByteOrderMark({
    bytes,
    path,
  });
  return parseStructuredInput({ text: decodeUtf8({
    bytes,
    path,
  }), },);
}
