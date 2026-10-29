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
  type PostinstallLink,
  type PostinstallRun,
  type PostinstallStep,
  type ToolEnv,
  type ToolOptions,
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
function isNameList(value: unknown,): value is readonly string[] {
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
function isStringRecord(value: unknown,): value is Readonly<Record<string, string>> {
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
function isAbsent(value: unknown,): boolean {
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
    return { pinned: String(text,), };
  if (key === 'channel')
    return { channel: String(text,), };
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

//region Options

/**
 The postinstall command step.

 @param value - raw `options.postinstall.run`

 @param fail - plan error reporter

 @returns typed command step

 @throws Error (through `fail`) when the shape is not `{ sourceBin, args }` with a non-empty `sourceBin` and a list of non-empty `args`

 @example
 ```ts
 parsePostinstallRun({ value: { sourceBin: 'a', args: ['link', 'dir',], }, fail, });
 ```
 */
function parsePostinstallRun(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): PostinstallRun {
  if ((!isRecord(value,)) || ((typeof value.sourceBin) !== 'string') || (value.sourceBin === '')
    || (!isNameList(value.args,)))
    return fail('options.postinstall.run must be { sourceBin, args: [names] }',);
  return {
    when: 'always',
    run: {
      sourceBin: value.sourceBin,
      args: value.args,
    },
  };
}

/**
 The postinstall step: a link or a command.

 @param value - raw `options.postinstall`

 @param fail - plan error reporter

 @returns typed postinstall step

 @throws Error (through `fail`) when the shape is neither `{ when: 'always', link: { sourceBin, linkName, linkDir? } }` nor `{ when: 'always', run: { sourceBin, args } }`

 @example
 ```ts
 parsePostinstall({ value: { when: 'always', link: { sourceBin: 'a', linkName: 'b', }, }, fail, });
 ```
 */
function parsePostinstall(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): PostinstallStep {
  if (isRecord(value,) && (value.when === 'always') && (value.run !== undefined) && (value.link === undefined))
    return parsePostinstallRun({
      value: value.run,
      fail,
    },);
  return parsePostinstallLink({
    value,
    fail,
  },);
}

/**
 The postinstall link step.

 @param value - raw `options.postinstall`

 @param fail - plan error reporter

 @returns typed link step

 @throws Error (through `fail`) when the shape is not `{ when: 'always', link: { sourceBin, linkName, linkDir? } }`

 @example
 ```ts
 parsePostinstallLink({ value: { when: 'always', link: { sourceBin: 'a', linkName: 'b', }, }, fail, });
 ```
 */
function parsePostinstallLink(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): PostinstallLink {
  /**
   The link record, absent when the shape is wrong.
   */
  const link = isRecord(value,) ? value.link : undefined;
  if ((!isRecord(value,)) || (value.when !== 'always')
    || (!isRecord(link,))
    || ((typeof link.sourceBin) !== 'string')
    || ((typeof link.linkName) !== 'string')
    || ((link.linkDir !== undefined) && ((typeof link.linkDir) !== 'string')))
    return fail('options.postinstall must be { when: always, link: { sourceBin, linkName, linkDir? } } or { when: always, run: { sourceBin, args } }',);
  return {
    when: 'always',
    link: {
      sourceBin: String(link.sourceBin,),
      linkName: String(link.linkName,),
      ...(link.linkDir === undefined ? {} : { linkDir: String(link.linkDir,), }),
    },
  };
}

/**
 The bootstrap metadata.

 @param value - raw `options.bootstrap`

 @param fail - plan error reporter

 @returns typed bootstrap record

 @throws Error (through `fail`) when the distribution is unknown or `requiresTool` is not a string

 @example
 ```ts
 parseBootstrap({ value: { distribution: 'wheel', }, fail, });
 ```
 */
function parseBootstrap(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): NonNullable<ToolOptions['bootstrap']> {
  if ((!isRecord(value,)) || ((value.distribution !== 'wheel') && (value.distribution !== 'standalone-binary'))
    || ((!isAbsent(value.requiresTool,)) && ((typeof value.requiresTool) !== 'string')))
    return fail('options.bootstrap must be { distribution: wheel or standalone-binary, requiresTool? }',);
  return {
    distribution: value.distribution,
    ...(isAbsent(value.requiresTool,) ? {} : { requiresTool: String(value.requiresTool,), }),
  };
}

/**
 One optional name-list option.

 @param key - option name, for the error

 @param value - raw option value

 @param fail - plan error reporter

 @returns zero lists when absent, else the one list

 @throws Error (through `fail`) when present but not a list of names

 @example
 ```ts
 nameListOption({ key: 'rustTargets', value: ['wasm32-unknown-unknown',], fail, });
 ```
 */
function nameListOption(
  {
    key,
    value,
    fail,
  }: {
    readonly key: string;
    readonly value: unknown;
    readonly fail: Fail;
  },
): readonly (readonly string[])[] {
  if (value === undefined)
    return [];
  if (!isNameList(value,))
    return fail(`options.${key} must be a list of names`,);
  return [value,];
}

/**
 Every option key the plan accepts.
 */
const KNOWN_OPTIONS: ReadonlySet<string> = new Set([
  'rustComponents',
  'rustTargets',
  'cargoFeatures',
  'allowBuilds',
  'androidPackages',
  'cargoGit',
  'assetPattern',
  'postinstall',
  'binAliases',
  'bootstrap',
],);

/**
 The provider options, validated key by key; unknown keys are rejected.

 @param value - raw `options` field

 @param fail - plan error reporter

 @returns typed options, empty when absent

 @throws Error (through `fail`) when the field is not an object, names an unknown option, or an option is malformed

 @example
 ```ts
 parseOptions({ value: { binAliases: { bunx: 'bun', }, }, fail, });
 ```
 */
export function parseOptions(
  {
    value,
    fail,
  }: {
    readonly value: unknown;
    readonly fail: Fail;
  },
): ToolOptions {
  if (isAbsent(value,))
    return {};
  if (!isRecord(value,))
    return fail('options must be an object',);
  /**
   First option key the plan does not know.
   */
  const unknownKey = Object.keys(value,)
    .find(function isUnknown(key: string,): boolean {
      return !KNOWN_OPTIONS.has(key,);
    },);
  if (unknownKey !== undefined)
    return fail(`options.${unknownKey} is not a known option`,);
  /**
   Name-list options, each zero or one list.
   */
  const [rustComponents,] = nameListOption({
    key: 'rustComponents',
    value: value.rustComponents,
    fail,
  },);
  const [rustTargets,] = nameListOption({
    key: 'rustTargets',
    value: value.rustTargets,
    fail,
  },);
  const [cargoFeatures,] = nameListOption({
    key: 'cargoFeatures',
    value: value.cargoFeatures,
    fail,
  },);
  const [allowBuilds,] = nameListOption({
    key: 'allowBuilds',
    value: value.allowBuilds,
    fail,
  },);
  const [androidPackages,] = nameListOption({
    key: 'androidPackages',
    value: value.androidPackages,
    fail,
  },);
  /**
   Remaining raw options, each checked below.
   */
  const {
    cargoGit,
    assetPattern,
    binAliases,
    postinstall,
    bootstrap,
  } = value;
  if ((cargoGit !== undefined) && ((!isRecord(cargoGit,)) || ((typeof cargoGit.url) !== 'string')))
    return fail('options.cargoGit must be { url }',);
  if ((assetPattern !== undefined) && ((typeof assetPattern) !== 'string'))
    return fail('options.assetPattern must be a string',);
  if ((binAliases !== undefined) && (!isStringRecord(binAliases,)))
    return fail('options.binAliases must map names to names',);
  return {
    ...(rustComponents === undefined ? {} : { rustComponents, }),
    ...(rustTargets === undefined ? {} : { rustTargets, }),
    ...(cargoFeatures === undefined ? {} : { cargoFeatures, }),
    ...(allowBuilds === undefined ? {} : { allowBuilds, }),
    ...(androidPackages === undefined ? {} : { androidPackages, }),
    ...(isRecord(cargoGit,) ? { cargoGit: { url: String(cargoGit.url,), }, } : {}),
    ...((typeof assetPattern) === 'string' ? { assetPattern: String(assetPattern,), } : {}),
    ...(isStringRecord(binAliases,) ? { binAliases, } : {}),
    ...(postinstall === undefined ? {} : { postinstall: parsePostinstall({
      value: postinstall,
      fail,
    },), }),
    ...(bootstrap === undefined ? {} : { bootstrap: parseBootstrap({
      value: bootstrap,
      fail,
    },), }),
  };
}

//endregion Options

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
                url: String(entry.url,),
                sha256: String(entry.sha256,),
              },
            ];
          },),),
      ];
    },),);
}

//endregion Artifacts
