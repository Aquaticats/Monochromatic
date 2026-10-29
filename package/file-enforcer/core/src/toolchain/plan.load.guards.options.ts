/**
 Validators for a raw plan record's `options` table: postinstall steps,
 bootstrap, and the name-list and map options.

 Split from `plan.load.guards.ts`, which keeps the shared validators and the
 entry-field and artifact validators. Node builtins only, like the loader.
 */

import {
  type Fail,
  isAbsent,
  isNameList,
  isRecord,
  isStringRecord,
} from './plan.load.guards.ts';
import type {
  PostinstallLink,
  PostinstallRun,
  PostinstallStep,
  ToolOptions,
} from './types.ts';

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
  if ((!isRecord(value,))
    || ((typeof value.sourceBin) !== 'string')
    || (value.sourceBin === '')
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
  if (isRecord(value,)
    && (value.when === 'always')
    && (value.run !== undefined)
    && (value.link === undefined))
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
      sourceBin: link.sourceBin,
      linkName: link.linkName,
      ...(link.linkDir === undefined ? {} : { linkDir: link.linkDir, }),
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
   Rustup components to add, absent when the option is unset.
   */
  const [rustComponents,] = nameListOption({
    key: 'rustComponents',
    value: value.rustComponents,
    fail,
  },);
  /**
   Rustup targets to add, absent when the option is unset.
   */
  const [rustTargets,] = nameListOption({
    key: 'rustTargets',
    value: value.rustTargets,
    fail,
  },);
  /**
   Cargo features to enable, absent when the option is unset.
   */
  const [cargoFeatures,] = nameListOption({
    key: 'cargoFeatures',
    value: value.cargoFeatures,
    fail,
  },);
  /**
   Packages allowed to run build scripts, absent when the option is unset.
   */
  const [allowBuilds,] = nameListOption({
    key: 'allowBuilds',
    value: value.allowBuilds,
    fail,
  },);
  /**
   Android SDK packages to install, absent when the option is unset.
   */
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
    ...((typeof assetPattern) === 'string' ? { assetPattern, } : {}),
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
