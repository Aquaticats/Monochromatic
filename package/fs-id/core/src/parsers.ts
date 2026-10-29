/**
 Linear parsers and identity-shape validation for platform command output.
 
 @module
 */

import type {
  FsId,
  FsIdSource,
} from './types.ts';

/**
 Maximum accepted platform payload length.
 
 @example
 ```ts
 const bounded = value.length <= MAX_PAYLOAD_LENGTH;
 ```
 */
const MAX_PAYLOAD_LENGTH = 512;

/**
 ASCII characters accepted in normalized platform payloads.
 
 @example
 ```ts
 SAFE_PAYLOAD_CHARACTERS.includes('a');
 ```
 */
const SAFE_PAYLOAD_CHARACTERS = 'abcdefghijklmnopqrstuvwxyz0123456789-.';

/**
 ASCII characters accepted in macOS device-node paths.
 
 @example
 ```ts
 SAFE_DEVICE_CHARACTERS.includes('/');
 ```
 */
const SAFE_DEVICE_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/_-.';

/**
 ASCII whitespace that terminates a Windows serial token.
 
 @example
 ```ts
 SERIAL_TERMINATORS.has('\n');
 ```
 */
const SERIAL_TERMINATORS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
  '\f',
  '\v',
],);

/**
 Case-insensitive Windows `vol` serial label.
 
 @example
 ```ts
 output.toLowerCase().includes(SERIAL_LABEL);
 ```
 */
const SERIAL_LABEL = 'serial number is';

/**
 ASCII drive letters accepted for Windows volume queries.
 
 @example
 ```ts
 WINDOWS_DRIVE_LETTERS.includes('c');
 ```
 */
const WINDOWS_DRIVE_LETTERS = 'abcdefghijklmnopqrstuvwxyz';

/**
 Stable prefixes used by generated identifiers.
 
 @example
 ```ts
 SOURCE_PREFIXES['fs-uuid'];
 ```
 */
const SOURCE_PREFIXES: Readonly<Record<FsIdSource, string>> = {
  'fs-uuid': 'fs-uuid_',
  'volume-uuid': 'volume-uuid_',
  'volume-serial': 'volume-serial_',
  'f-fsid': 'f-fsid_',
  'device-number': 'device-number_',
};

/**
 Checks canonical payload grammar without throwing.
 
 @param value - Canonical payload candidate
 
 @returns Whether payload is lowercase safe ASCII
 
 @example
 ```ts
 isNormalizedIdentityPayload('1a2b-3c4d'); // true
 ```
 */
function isNormalizedIdentityPayload(value: string,): boolean {
  if ((value.length === 0)
    || (value.length > MAX_PAYLOAD_LENGTH)
    || (value === '-')
    || (value !== value.trim())
    || (value !== value.toLowerCase())) {
    return false;
  }
  for (const character of value) {
    if (!SAFE_PAYLOAD_CHARACTERS.includes(character,))
      return false;
  }
  return true;
}

/**
 Normalizes and validates one command-supplied identity payload.
 
 @param value - Untrusted command output token
 
 @returns Lowercase safe payload
 
 @throws when payload is empty or unsafe
 
 @example
 ```ts
 normalizeIdentityPayload(' 1A2B-3C4D '); // '1a2b-3c4d'
 ```
 */
export function normalizeIdentityPayload(value: string,): string {
  /**
   Trimmed lowercase representation used by trust keys.
   */
  const normalized = value.trim()
    .toLowerCase();
  if (!isNormalizedIdentityPayload(normalized,))
    throw new TypeError('invalid filesystem identity payload',);
  return normalized;
}

/**
 Checks whether string follows generated filesystem-ID grammar.
 
 @param value - Candidate identifier
 
 @returns Whether value has known source prefix and safe payload
 
 @example
 ```ts
 isFsId('volume-serial_1a2b-3c4d'); // true
 ```
 */
export function isFsId(value: string,): value is FsId {
  return Object.values(SOURCE_PREFIXES,)
    .some(function prefixMatches(prefix,): boolean {
      if (!value.startsWith(prefix,))
        return false;
      return isNormalizedIdentityPayload(value.slice(prefix.length,),);
    },);
}

/**
 Asserts generated filesystem-ID grammar.
 
 @param value - Candidate identifier
 
 @returns Nothing after narrowing value to generated identity
 
 @throws when value does not have generated shape
 
 @example
 ```ts
 assertFsId('fs-uuid_abcd');
 ```
 */
export function assertFsId(value: string,): asserts value is FsId {
  if (!isFsId(value,))
    throw new TypeError('invalid generated filesystem identity',);
}

/**
 Creates a source-qualified colon-free identity.
 
 @param source - Mechanism that produced payload
 
 @param payload - Validated platform value
 
 @returns Branded identity
 
 @throws when payload is unsafe or empty
 
 @example
 ```ts
 createFsId({ source: 'fs-uuid', payload: 'ABCD' });
 ```
 */
export function createFsId({
  source,
  payload,
}: {
  readonly source: FsIdSource;
  readonly payload: string;
},): FsId {
  /**
   Source-qualified canonical string before assertion narrowing.
   */
  const candidate = `${SOURCE_PREFIXES[source]}${normalizeIdentityPayload(payload,)}`;
  assertFsId(candidate,);
  return candidate;
}

/**
 Parses UUID output emitted by `findmnt`.
 
 @param output - Captured standard output
 
 @returns Safe token
 
 @throws when output has no safe UUID
 
 @example
 ```ts
 parseFindmntUuid(' 1234-ABCD\n'); // '1234-abcd'
 ```
 */
export function parseFindmntUuid(output: string,): string {
  return normalizeIdentityPayload(output,);
}

/**
 Checks device path against ASCII command-argument grammar.
 
 @param device - Device path candidate
 
 @returns Whether every code unit is allowed ASCII
 
 @example
 ```ts
 isSafeDevicePath('/dev/disk3s1'); // true
 ```
 */
function isSafeDevicePath(device: string,): boolean {
  for (let index = 0; index < device.length; index += 1) {
    if (!SAFE_DEVICE_CHARACTERS.includes(device.charAt(index,),))
      return false;
  }
  return true;
}

/**
 Parses mounted device node from POSIX `df -P` output.
 
 @param output - Captured portable-format filesystem report
 
 @returns Device node safe for `diskutil info`
 
 @throws when output has no safe `/dev/` row
 
 @example
 ```ts
 parseDfDevice('Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/disk3s1 1 1 0 100% /');
 ```
 */
export function parseDfDevice(output: string,): string {
  /**
   Nonempty rows in source order.
   */
  const lines = output.split('\n',)
    .filter(function lineHasContent(line,): boolean {
      return line.trim()
        .length
        > 0;
    },);
  for (const line of lines.toReversed()) {
    /**
     Trimmed data row.
     */
    const trimmed = line.trim();
    /**
     First field endpoint.
     */
    const whitespaceIndex = (function findWhitespace(): number {
      for (let index = 0; index < trimmed.length; index += 1) {
        if ((trimmed.charAt(index,) === ' ') || (trimmed.charAt(index,) === '\t'))
          return index;
      }
      return trimmed.length;
    })();
    /**
     Filesystem device field.
     */
    const device = trimmed.slice(
      0,
      whitespaceIndex,
    );
    if (!device.startsWith('/dev/',))
      continue;
    if (isSafeDevicePath(device,))
      return device;
  }
  throw new TypeError('df returned no safe mounted device node',);
}

/**
 Parses `VolumeUUID` from structured `diskutil info -plist` output.
 
 @param output - Captured XML property list
 
 @returns Safe token
 
 @throws when property list has no safe Volume UUID
 
 @example
 ```ts
 parseDiskutilVolumeUuid('<key>VolumeUUID</key><string>ABCD-1234</string>');
 ```
 */
export function parseDiskutilVolumeUuid(output: string,): string {
  /**
   Invariant plist key emitted independent of display locale.
   */
  const key = '<key>VolumeUUID</key>';
  /**
   String value opening tag.
   */
  const stringOpen = '<string>';
  /**
   String value closing tag.
   */
  const stringClose = '</string>';
  /**
   Key position or absent sentinel.
   */
  const keyIndex = output.indexOf(key,);
  if (keyIndex === (-1))
    throw new TypeError('diskutil plist has no VolumeUUID key',);
  /**
   Value opening position after exact key.
   */
  const openIndex = output.indexOf(
    stringOpen,
    keyIndex + key.length,
  );
  if (openIndex === (-1))
    throw new TypeError('diskutil plist VolumeUUID has no string value',);
  /**
   Value start after opening tag.
   */
  const valueStart = openIndex + stringOpen.length;
  /**
   Value end or absent sentinel.
   */
  const closeIndex = output.indexOf(
    stringClose,
    valueStart,
  );
  if (closeIndex === (-1))
    throw new TypeError('diskutil plist VolumeUUID string is unterminated',);
  return normalizeIdentityPayload(output.slice(
    valueStart,
    closeIndex,
  ),);
}

/**
 Extracts safe Windows drive root from canonical path.
 
 @param path - Canonical Windows path
 
 @returns Uppercase drive root
 
 @throws for non-drive path
 
 @example
 ```ts
 windowsDriveRoot('c:\\repo'); // 'C:\\'
 ```
 */
export function windowsDriveRoot(path: string,): string {
  /**
   Lowercase drive letter candidate.
   */
  const letter = path.charAt(0,)
    .toLowerCase();
  /**
   Root separator candidate.
   */
  const separator = path.charAt(2,);
  if ((!WINDOWS_DRIVE_LETTERS.includes(letter,))
    || (path.charAt(1,) !== ':')
    || ((separator !== '\\') && (separator !== '/'))) {
    throw new TypeError(`Windows path has no drive root: ${path}`,);
  }
  return `${letter.toUpperCase()}:\\`;
}

/**
 Extracts serial token from Windows `vol` output.
 
 @param output - Captured command text
 
 @returns Serial token or empty string when absent
 
 @example
 ```ts
 parseVolumeSerial('Volume Serial Number is 1A2B-3C4D');
 ```
 */
export function parseVolumeSerial(output: string,): string {
  /**
   Lowercase copy whose offsets match original output.
   */
  const lower = output.toLowerCase();
  /**
   Label start or `-1` when localized/unexpected output omits it.
   */
  const labelIndex = lower.indexOf(SERIAL_LABEL,);
  if (labelIndex === (-1))
    return '';

  return (function scanSerialToken(): string {
    /**
     Forward-only cursor after label.
     */
    let cursor = labelIndex + SERIAL_LABEL.length;
    while ((cursor < output.length)
      && ((output.charAt(cursor,) === ' ') || (output.charAt(cursor,) === '\t'))) {
      cursor += 1;
    }
    /**
     Serial characters collected once to avoid repeated string rebuilding.
     */
    const characters: string[] = [];
    while (cursor < output.length) {
      /**
       Current character whose whitespace membership ends token.
       */
      const character = output.charAt(cursor,);
      if (SERIAL_TERMINATORS.has(character,))
        break;
      characters.push(character,);
      cursor += 1;
    }
    return characters.join('',);
  })();
}
