/**
 Validators that turn one raw plan record's fields into their typed shapes.

 Each validator takes the raw value and a `fail` callback that throws a plan
 error naming the field, so `plan.load.ts` builds a `ToolPlanEntry` without
 type assertions. A raw `null` reads as absent for the optional fields, so
 data modules written before absent keys were omitted still load.

 Node builtins only, like the loader.
 */

import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import {
  TOOL_ROUTES,
  type ToolEnv,
  type ToolPlanEntry,
  type ToolRequest,
  type ToolRoute,
} from './types.ts';

//region Shared

/**
 Throws a plan error describing one defect of the record being validated.
 */
export type Fail = (problem: string,) => never;

/**
 OS families a plan entry may name.
 */
type OsFamily = NonNullable<ToolPlanEntry['os']>[number];

/**
 Every OS family, for membership checks.
 */
const OS_FAMILIES: readonly OsFamily[] = [
  'linux',
  'macos',
  'windows',
];

/**
 Whether a raw value is a non-null, non-array object.

 @param value - raw value

 @returns true for a plain record

 @example
 ```ts
 isRecord({ a: 1, },); // true
 ```
 */
export function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Whether a raw value is a list of non-empty strings.

 @param value - raw value

 @returns true when every element is a non-empty string

 @example
 ```ts
 isNameList(['a',],); // true
 ```
 */
export function isNameList(value: unknown,): value is readonly string[] {
  return Array.isArray(value,) && value.every(function isName(item: unknown,): boolean {
    return ((typeof item) === 'string') && (item !== '');
  },);
}

/**
 Whether a raw value is a record of string values.

 @param value - raw value

 @returns true when every value is a string

 @example
 ```ts
 isStringRecord({ A: 'b', },); // true
 ```
 */
export function isStringRecord(value: unknown,): value is Readonly<Record<string, string>> {
  return isRecord(value,)
    && Object.values(value,)
      .every(function isString(item: unknown,): boolean {
        return (typeof item) === 'string';
      },);
}

/**
 Whether a raw value is absent: missing or a legacy `null`.

 @param value - raw value

 @returns true for `undefined` or `null`

 @example
 ```ts
 isAbsent(null,); // true
 ```
 */
export function isAbsent(value: unknown,): boolean {
  return (value === undefined) || (value === null);
}

//endregion Shared

//region Entry fields

/**
 The request intent: exactly one of `track: 'latest'`, `pinned`, or `channel`.

 @param value - raw `request` field

 @param fail - plan error reporter

 @returns typed request

 @throws Error (through `fail`) when the intent is missing, ambiguous, or not a string

 @example
 ```ts
 parseRequest({ value: { pinned: '1.0.0', }, fail, });
 ```
 */
export function parseRequest(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): ToolRequest {
  if (!isRecord(value,))
    return fail('request is missing',);
  /**
   Intent keys the request sets.
   */
  const present = [
    'track',
    'pinned',
    'channel',
  ].filter(function isSet(key: string,): boolean {
    return value[key] !== undefined;
  },);
  /**
   The one intent's value.
   */
  const [key = '',] = present;
  /**
   Text of that intent.
   */
  const text = value[key];
  if ((present.length !== 1) || ((typeof text) !== 'string'))
    return fail('request needs exactly one string intent (track, pinned or channel)',);
  if (key === 'pinned')
    return { pinned: text, };
  if (key === 'channel')
    return { channel: text, };
  if (text !== 'latest')
    return fail('request.track must be latest',);
  return { track: 'latest', };
}

/**
 The install route.

 @param value - raw `route` field

 @param fail - plan error reporter

 @returns typed route

 @throws Error (through `fail`) when the value names no route

 @example
 ```ts
 parseRoute({ value: 'archive', fail, }); // 'archive'
 ```
 */
export function parseRoute(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): ToolRoute {
  /**
   The matching route, absent for any other value.
   */
  const route = TOOL_ROUTES.find(function matches(candidate: ToolRoute,): boolean {
    return candidate === value;
  },);
  return route ?? fail(`route ${JSON.stringify(value,)} is not one of ${TOOL_ROUTES.join(', ',)}`,);
}

/**
 The declared binary names.

 @param value - raw `bins` field

 @param fail - plan error reporter

 @returns non-empty list of names

 @throws Error (through `fail`) when the list is empty or holds a non-name

 @example
 ```ts
 parseBins({ value: ['rg',], fail, });
 ```
 */
export function parseBins(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): readonly string[] {
  if ((!isNameList(value,)) || (value.length === 0))
    return fail('bins must be a non-empty list of names',);
  return value;
}

/**
 Whether a raw value is one of the OS families (a three-element check).

 @param item - raw list element

 @returns true for linux, macos, or windows

 @example
 ```ts
 isOsFamily('linux',); // true
 ```
 */
function isOsFamily(item: unknown,): item is OsFamily {
  return OS_FAMILIES.some(function same(family: OsFamily,): boolean {
    return family === item;
  },);
}

/**
 The OS filter; absent means every OS.

 @param value - raw `os` field

 @param fail - plan error reporter

 @returns the families in declared order, as zero lists (absent) or one

 @throws Error (through `fail`) when a family is not linux, macos, or windows

 @example
 ```ts
 parseOs({ value: ['linux',], fail, }); // [['linux']]
 ```
 */
export function parseOs(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): readonly (readonly OsFamily[])[] {
  if (isAbsent(value,))
    return [];
  if ((!Array.isArray(value,)) || (!value.every(isOsFamily,)))
    return fail('os must be absent or a list of linux, macos and windows',);
  return [value.filter(isOsFamily,),];
}

/**
 An optional string field such as `manager` or `packageId`.

 @param key - field name, for the error

 @param value - raw field value

 @param fail - plan error reporter

 @returns zero values when absent, else the one string

 @throws Error (through `fail`) when present but not a non-empty string

 @example
 ```ts
 optionalName({ key: 'manager', value: 'cargo', fail, }); // ['cargo']
 ```
 */
export function optionalName(
  {
    key,
    value,
    fail,
  }: {
    readonly key: string;
    readonly value: unknown;
    readonly fail: Fail;
  },
): readonly string[] {
  if (isAbsent(value,))
    return [];
  if (((typeof value) !== 'string') || (value === ''))
    return fail(`${key} must be absent or a non-empty string`,);
  return [value,];
}

/**
 The tool's environment exports.

 @param value - raw `env` field

 @param fail - plan error reporter

 @returns string map, empty when absent

 @throws Error (through `fail`) when a value is not a string

 @example
 ```ts
 parseEnv({ value: { RUSTUP_TOOLCHAIN: 'nightly', }, fail, });
 ```
 */
export function parseEnv(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): ToolEnv {
  if (isAbsent(value,))
    return {};
  if (!isStringRecord(value,))
    return fail('env must be an object of strings',);
  return value;
}

//endregion Entry fields

//region Artifacts

/**
 The artifact table of a `plan.artifacts.data.ts` module.

 @param value - raw `ARTIFACTS` export

 @param fail - module error reporter

 @returns artifact rows keyed by tool id, then platform

 @throws Error (through `fail`) when a row is not `{ url, sha256 }`

 @example
 ```ts
 parseArtifacts({ value: { age: { 'linux-x64': { url: 'https://x', sha256: 'ab', }, }, }, fail, });
 ```
 */
export function parseArtifacts(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): Readonly<Record<string, ToolArtifacts>> {
  if (!isRecord(value,))
    return fail('expected an ARTIFACTS object export',);
  return Object.fromEntries(Object.entries(value,)
    .map(function toolRows([id, table,]: readonly [
      string,
      unknown,
    ],): readonly [
      string,
      ToolArtifacts,
    ] {
      if (!isRecord(table,))
        return fail(`artifacts for ${JSON.stringify(id,)} must be an object`,);
      return [
        id,
        Object.fromEntries(Object.entries(table,)
          .map(function row([platform, entry,]: readonly [
            string,
            unknown,
          ],): readonly [
            string,
            {
              readonly url: string;
              readonly sha256: string
            },
          ] {
            if ((!isRecord(entry,)) || ((typeof entry.url) !== 'string')
              || ((typeof entry.sha256) !== 'string'))
              return fail(`artifact ${id} ${platform} must be { url, sha256 }`,);
            return [
              platform,
              {
                url: entry.url,
                sha256: entry.sha256,
              },
            ];
          },),),
      ];
    },),);
}

//endregion Artifacts
