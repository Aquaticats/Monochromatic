/**
 Pure parsers for adb command output. No side effects, no spawning: every
 function takes captured stdout and returns structured data, so the brittle
 "what does this Android version print" logic is isolated and unit-testable.
 
 @module
 */

import { AdbCommandError, } from './errors.ts';

//region Package-name validation

/**
 Characters permitted in an Android application id: ASCII letters, digits,
 underscore, and the dot segment separator. Used to validate names before
 they are forwarded as tokens to the on-device shell.
 */
const ALLOWED_PACKAGE_CHARS: ReadonlySet<string> = new Set(
  'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.',
);

/**
 Report whether `name` is a syntactically valid Android application id.
 
 Validation matters because each name is later interpolated into an
 `adb shell cmd appops ...` command string the device shell parses; rejecting
 anything outside {@link ALLOWED_PACKAGE_CHARS} keeps shell metacharacters out.
 Scans by index (not spread or `for...of`) to avoid splitting on grapheme
 boundaries; ASCII validation needs raw UTF-16 units anyway.
 
 @param name - Candidate application id to validate.
 
 @returns `true` when `name` is non-empty and every character is allowed.
 
 @example
 ```ts
 isValidPackageName({ name: 'com.example.app', },); // true
 isValidPackageName({ name: 'rm -rf', },); // false
 ```
 */
export function isValidPackageName({ name, }: { readonly name: string; },): boolean {
  if (name.length === 0) {
    return false;
  }
  for (let index = 0; index < name.length; index += 1) {
    if (!ALLOWED_PACKAGE_CHARS.has(name.charAt(index,),)) {
      return false;
    }
  }
  return true;
}

//endregion

//region Device line parsing

/**
 One entry from `adb devices`: a serial plus its connection state. State is a
 free-form string because adb's vocabulary (`device`, `offline`,
 `unauthorized`, ...) varies by version.
 */
export type Device = {
  readonly serial: string;
  readonly state: string;
};

/**
 Parse `adb devices` output into structured {@link Device} entries.
 
 Skips the `List of devices attached` header and `* daemon ...` chatter, and
 tolerates either tab or space separation between serial and state.
 
 @param stdout - Captured stdout from `adb devices`.
 
 @returns One Device per recognized line, in listed order.
 
 @example
 ```ts
 parseDevices({ stdout: 'List of devices attached\nABC123\tdevice\n', },);
 // [{ serial: 'ABC123', state: 'device' }]
 ```
 */
export function parseDevices({ stdout, }: { readonly stdout: string; },): readonly Device[] {
  return stdout.split('\n',)
    .flatMap(function lineToDevices(rawLine,): readonly Device[] {
    /**
     Trimmed line, with the device serial and state when present.
     */
      const line = rawLine.trim();
      if (
        (line.length === 0)
          || line.startsWith('List of devices attached',)
          || line.startsWith('*',)
      ) {
        return [];
      }
    /**
     Whitespace-split fields: `[serial, state, ...]` (tab or space separated).
     */
      const fields = line
        .split('\t',)
        .flatMap(function bySpace(part,): readonly string[] {
          return part.split(' ',);
        },)
        .filter(function isNonEmpty(field,): boolean {
          return field.length > 0;
        },);
    /**
     First field is the serial, second is the state.
     */
      const [serial, state,] = fields;
      if ((serial === undefined) || (state === undefined)) {
        return [];
      }
      return [{
        serial,
        state,
      },];
    },);
}

//endregion

//region Package list parsing

/**
 Parse `pm list packages -3` output into application ids.
 
 Keeps only `package:`-prefixed lines, strips the prefix, and validates each
 id with {@link isValidPackageName}. An invalid id signals output this
 parser does not understand, so it throws rather than silently dropping
 entries.
 
 @param stdout - Captured stdout from `adb shell pm list packages -3`.
 
 @returns Validated application ids, in listed order.
 
 @throws {@link AdbCommandError} when a stripped name fails validation.
 
 @example
 ```ts
 parsePackageList({ stdout: 'package:com.example.app\n', },); // ['com.example.app']
 ```
 */
export function parsePackageList({ stdout, }: { readonly stdout: string; },): readonly string[] {
  /**
   Line prefix `pm list packages` puts before each application id.
   */
  const prefix = 'package:';
  /**
   Stripped application ids, before validation.
   */
  const names = stdout
    .split('\n',)
    .map(function trimLine(line,): string {
      return line.trim();
    },)
    .filter(function isPackageLine(line,): boolean {
      return line.startsWith(prefix,);
    },)
    .map(function stripPrefix(line,): string {
      return line.slice(prefix.length,);
    },);
  /**
   First name failing validation, or `undefined` when all pass.
   */
  const invalid = names.find(function isInvalid(name,): boolean {
    return !isValidPackageName({ name, },);
  },);
  if (invalid !== undefined) {
    throw new AdbCommandError(`pm list packages emitted an unparseable package name: ${JSON.stringify(invalid,)}`,);
  }
  return names;
}

//endregion

//region Exemption query parsing

/**
 Parse `cmd appops query-op AUTO_REVOKE_PERMISSIONS_IF_UNUSED ignore` output
 into the application ids currently exempted.
 
 Tolerant by design: keeps only lines that {@link isValidPackageName} reports
 as a valid application id, so headers, uid lines, and `No operations.`
 chatter are ignored. The exact output format must be confirmed on a real
 device; the per-app `get` fallback covers Android versions where this
 query is unavailable.
 
 @param stdout - Captured stdout from the `query-op` invocation.
 
 @returns Application ids that appear as bare valid names, in listed order.
 
 @example
 ```ts
 parseExemptedQuery({ stdout: 'com.a\ncom.b\nNo operations.\n', },);
 // ['com.a', 'com.b']
 ```
 */
export function parseExemptedQuery({ stdout, }: { readonly stdout: string; },): readonly string[] {
  return stdout
    .split('\n',)
    .map(function trimLine(line,): string {
      return line.trim();
    },)
    .filter(function isPackageName(line,): boolean {
      return isValidPackageName({ name: line, },);
    },);
}

//endregion
