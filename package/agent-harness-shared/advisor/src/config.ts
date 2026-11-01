/**
 Advisor extension config loading and merging.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import * as v from 'valibot';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  type AdvisorConfigFile,
  AdvisorConfigFileSchema,
} from './config-schemas.ts';
import {
  CONFIG_FILE_NAME,
  DEFAULT_COLLECTION_GRACE_MS,
  DEFAULT_MAX_ADVISOR_OUTPUT_TOKENS,
  DEFAULT_TIMEOUT_MS,
} from './constants.ts';
import type { AdvisorConfig, } from './types.ts';

/**
 Sentinel returned by {@link loadConfigFile} when a config scope's file is absent.
 A `unique symbol`; callers narrow with `=== NO_CONFIG_FILE`.
 */
const NO_CONFIG_FILE: unique symbol = Symbol('advisor/no-config-file',);

//region Defaults

/**
 Default runtime config before user files are merged.
 */
export const DEFAULT_CONFIG: Omit<AdvisorConfig, 'source'> = {
  enabled: true,
  timeoutMs: DEFAULT_TIMEOUT_MS,
  hedgingEnabled: false,
  collectionGraceMs: DEFAULT_COLLECTION_GRACE_MS,
  maxAdvisorOutputTokens: DEFAULT_MAX_ADVISOR_OUTPUT_TOKENS,
  includePriorAdvisorResults: true,
};

//endregion Defaults

//region Public API

/**
 Options for loading Advisor config.
 */
export type LoadConfigOptions = {
  /**
   Current working directory used for project config lookup.
   */
  readonly cwd: string;
  /**
   Home directory override for tests.
   */
  readonly home?: string;
};

/**
 Load and merge global and project Advisor config files.
 
 @param options - lookup directories for global and project config
 
 @returns merged runtime configuration
 
 @throws when a present config file is invalid JSON or fails schema validation
 
 @example
 ```typescript
 const config = await loadMergedConfig({ cwd: process.cwd() });
 ```
 */
export async function loadMergedConfig(
  options: LoadConfigOptions,
): Promise<AdvisorConfig> {
  /**
   Path metadata for both config scopes.
   */
  const paths = getConfigPaths(options,);
  /**
   Global and project config file contents, when present.
   */
  const [global, project,] = await Promise.all([
    loadConfigFile({
      path: paths.globalPath,
      label: 'global',
    },),
    loadConfigFile({
      path: paths.projectPath,
      label: 'project',
    },),
  ],);
  /**
   Config values merged with project scalar overrides.
   */
  const merged = mergeConfigFiles({
    defaults: DEFAULT_CONFIG,
    configs: [
      global,
      project,
    ],
  },);

  if (merged.hedgingEnabled && (merged.hedgeDelayMs === undefined))
    throw new Error('advisor: hedgingEnabled requires an explicit positive hedgeDelayMs; overlapping requests can both be billed',);

  return {
    ...merged,
    source: {
      globalPath: paths.globalPath,
      projectPath: paths.projectPath,
      globalLoaded: global !== NO_CONFIG_FILE,
      projectLoaded: project !== NO_CONFIG_FILE,
    },
  };
}

/**
 Build absolute global and project config paths.
 
 @param options - lookup directories
 
 @returns config paths
 
 @example
 ```typescript
 getConfigPaths({ cwd: '/repo', home: '/home/me' });
 ```
 */
export function getConfigPaths(
  options: LoadConfigOptions,
): {
  readonly globalPath: string;
  readonly projectPath: string;
} {
  /**
   Home directory used by pi for global agent config.
   */
  const home = options.home
    ?? process
      .env
      .HOME
    ?? '~';
  return {
    globalPath: join(
      home,
      '.pi',
      'agent',
      'extensions',
      CONFIG_FILE_NAME,
    ),
    projectPath: join(
      options.cwd,
      '.pi',
      'extensions',
      CONFIG_FILE_NAME,
    ),
  };
}

//endregion Public API

//region Internal loading

/**
 Merge config file overrides without letting explicit undefined erase defaults.
 
 @param defaults - default runtime config
 
 @param configs - config files in merge order, with {@link NO_CONFIG_FILE} entries skipped
 
 @returns merged runtime config without source metadata
 */
function mergeConfigFiles(
  {
    defaults,
    configs,
  }: ForeignBorrowed<Readonly<{
    defaults: Omit<AdvisorConfig, 'source'>;
    configs: readonly (AdvisorConfigFile | typeof NO_CONFIG_FILE)[];
  }>>,
): Omit<AdvisorConfig, 'source'> {
  /**
   Current merged config, replaced for each present override file.
   */
  let merged = defaults;
  for (const config of configs) {
    if (config === NO_CONFIG_FILE)
      continue;
    /**
     Merged context cap, omitted when neither scope configures one.
     */
    const maxContextChars = config.maxContextChars
      ?? merged
        .maxContextChars;
    /**
     Delay inherits across config scopes; explicit disablement remains independent.
     */
    const hedgeDelayMs = config.hedgeDelayMs ?? merged.hedgeDelayMs;
    /**
     Unrelated project overrides must not discard global reviewer instructions.
     */
    const systemPrompt = config.systemPrompt ?? merged.systemPrompt;
    /**
     Configured initial reviewer overrides scalar-wise; an explicit empty string clears it.
     */
    const defaultReviewerModel = config.defaultReviewerModel
      ?? merged
        .defaultReviewerModel;
    merged = {
      hedgingEnabled: config.hedgingEnabled ?? merged.hedgingEnabled,
      collectionGraceMs: config.collectionGraceMs ?? merged.collectionGraceMs,
      ...(hedgeDelayMs === undefined ? {} : { hedgeDelayMs, }),
      enabled: config.enabled
        ?? merged
          .enabled,
      timeoutMs: config.timeoutMs
        ?? merged
          .timeoutMs,
      maxAdvisorOutputTokens: config.maxAdvisorOutputTokens
        ?? merged
          .maxAdvisorOutputTokens,
      includePriorAdvisorResults: config.includePriorAdvisorResults
        ?? merged
          .includePriorAdvisorResults,
      ...(maxContextChars === undefined ? {} : { maxContextChars, }),
      ...(systemPrompt === undefined ? {} : { systemPrompt, }),
      ...(defaultReviewerModel === undefined ? {} : { defaultReviewerModel, }),
    };
  }
  return merged;
}

/**
 Load one optional config file.
 
 @param path - config file path
 
 @param label - config scope label
 
 @returns parsed config file, or {@link NO_CONFIG_FILE} when absent
 */
async function loadConfigFile(
  {
    path,
    label,
  }: {
    readonly path: string;
    readonly label: string;
  },
): Promise<AdvisorConfigFile | typeof NO_CONFIG_FILE> {
  /**
   Raw JSON data, or `undefined` when file is absent.
   */
  const raw = await readJsonFile({
    path,
    label,
  },);
  if (raw === undefined)
    return NO_CONFIG_FILE;
  /**
   Validation result from valibot for locally parsed JSON value.
   */
  const result = v.safeParse(
    AdvisorConfigFileSchema,
    raw,
  );
  if (result.success)
    return result.output;
  throw new Error(
    `advisor: invalid ${label} config at ${path}: ${JSON.stringify(result.issues,)}`,
  );
}

/**
 Read and parse an optional JSON file.
 
 @param path - JSON file path
 
 @param label - config scope label
 
 @returns parsed JSON data, or `undefined` when absent
 */
async function readJsonFile(
  {
    path,
    label,
  }: {
    readonly path: string;
    readonly label: string;
  },
): Promise<unknown> {
  try {
    /**
     UTF-8 JSON file contents.
     */
    const text = await readFile(
      path,
      'utf8',
    );
    return JSON.parse(text,);
  }
  catch (error) {
    if (((typeof error) === 'object')
      && (error !== null)
      && isFileMissingError(error,))
      return undefined;
    throw new Error(
      `advisor: failed to read ${label} config at ${path}: ${
        caughtValueText(error,)
      }`,
      { cause: error, },
    );
  }
}

/**
 Detect Node ENOENT missing-file errors without unsafe assertion.
 
 @param error - caught error value
 
 @returns whether error reports a missing config file
 
 @mutates error - `Error.isError` can inspect runtime-owned error capability
 */
function isFileMissingError(
  error: ForeignHostCapability<object>,
): boolean {
  if ((!(Error.isError(error,))) || (!('code' in error)))
    return false;
  return error.code === 'ENOENT';
}

//endregion Internal loading
