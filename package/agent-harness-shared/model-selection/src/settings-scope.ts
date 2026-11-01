/**
 Reads pi settings needed to reconstruct effective model scope.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import * as v from 'valibot';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 Sentinel returned by internal {@link loadSettingsFile} when a settings file is
 absent. A `unique symbol`; narrowed with `=== NO_SETTINGS_FILE`.
 */
const NO_SETTINGS_FILE: unique symbol = Symbol('model-selection/no-settings-file',);

//region Types and schemas

/**
 Pi settings subset needed for model-scope reconstruction.
 */
export type PiSettingsFile = {
  /**
   Pi model-cycle patterns.
   */
  readonly enabledModels?: readonly string[];
};

/**
 Options for reading pi settings files.
 */
export type LoadSettingsScopeOptions = {
  /**
   Current working directory for project settings lookup.
   */
  readonly cwd: string;
  /**
   Home directory override for tests.
   */
  readonly home?: string;
  /**
   Error prefix used by the consuming extension.
   */
  readonly errorPrefix?: string;
};

/**
 Reconstructed enabled-model patterns from pi settings.
 */
export type SettingsScopePatterns = {
  /**
   Effective enabled-model patterns, omitted when unrestricted.
   */
  readonly patterns?: readonly string[];
  /**
   Settings file path that supplied effective patterns.
   */
  readonly sourcePath?: string;
};

/**
 Pi settings subset schema.
 */
export const PiSettingsFileSchema: v.GenericSchema<PiSettingsFile> = v.object({
  enabledModels: v.exactOptional(
    v.array(v.string(),),
  ),
},);

//endregion Types and schemas

//region Public API

/**
 Load effective `enabledModels` from global and project pi settings.
 
 Project settings override global settings when `enabledModels` is present.
 
 @param options - lookup directories and optional error prefix
 
 @returns effective patterns and source path
 
 @example
 ```typescript
 await loadSettingsScopePatterns({ cwd: process.cwd() });
 ```
 */
export async function loadSettingsScopePatterns(
  options: LoadSettingsScopeOptions,
): Promise<SettingsScopePatterns> {
  /**
   Paths checked for pi settings.
   */
  const paths = getSettingsPaths(options,);
  /**
   Error prefix for thrown messages.
   */
  const errorPrefix = options.errorPrefix
    ?? 'model selection';
  /**
   Global and project settings subsets read concurrently.
   */
  const [global, project,] = await Promise.all([
    loadSettingsFile({
      path: paths.globalPath,
      label: 'global',
      errorPrefix,
    },),
    loadSettingsFile({
      path: paths.projectPath,
      label: 'project',
      errorPrefix,
    },),
  ],);

  if ((project !== NO_SETTINGS_FILE) && (project.enabledModels
    !== undefined)) {
    return {
      patterns: cleanPatterns(project.enabledModels,),
      sourcePath: paths.projectPath,
    };
  }

  if ((global !== NO_SETTINGS_FILE) && (global.enabledModels
    !== undefined)) {
    return {
      patterns: cleanPatterns(global.enabledModels,),
      sourcePath: paths.globalPath,
    };
  }

  return {};
}

/**
 Build absolute global and project pi settings paths.
 
 @param options - lookup directories
 
 @returns settings paths
 
 @example
 ```typescript
 getSettingsPaths({ cwd: '/repo', home: '/home/me' });
 ```
 */
export function getSettingsPaths(
  options: LoadSettingsScopeOptions,
): {
  readonly globalPath: string;
  readonly projectPath: string;
} {
  /**
   Process environment variables.
   */
  const { env, } = process;
  /**
   HOME environment variable value, when present.
   */
  const { HOME: envHome, } = env;
  /**
   Home directory used by pi for global settings.
   */
  const home = options.home
    ?? envHome
    ?? '~';
  return {
    globalPath: join(
      home,
      '.pi',
      'agent',
      'settings.json',
    ),
    projectPath: join(
      options.cwd,
      '.pi',
      'settings.json',
    ),
  };
}

//endregion Public API

//region Internal helpers

/**
 Load one optional pi settings file.
 
 @param path - settings file path
 
 @param label - settings scope label
 
 @param errorPrefix - message prefix
 
 @returns parsed settings subset, or {@link NO_SETTINGS_FILE} when absent
 */
async function loadSettingsFile(
  {
    path,
    label,
    errorPrefix,
  }: {
    readonly path: string;
    readonly label: string;
    readonly errorPrefix: string;
  },
): Promise<PiSettingsFile | typeof NO_SETTINGS_FILE> {
  /**
   Raw parsed JSON object, or `undefined` when absent.
   */
  const raw = await readJsonFile({
    path,
    label,
    errorPrefix,
  },);
  if (raw === undefined)
    return NO_SETTINGS_FILE;

  /**
   Valibot validation result for settings subset.
   */
  const result = v.safeParse(
    PiSettingsFileSchema,
    raw,
  );
  if (result.success)
    return result.output;
  throw new Error(
    `${errorPrefix}: invalid ${label} settings at ${path}: ${JSON.stringify(result.issues,)}`,
  );
}

/**
 Read one JSON settings file.
 
 @param path - settings file path
 
 @param label - settings scope label
 
 @param errorPrefix - message prefix
 
 @returns parsed JSON data, or `undefined` when absent
 */
async function readJsonFile(
  {
    path,
    label,
    errorPrefix,
  }: {
    readonly path: string;
    readonly label: string;
    readonly errorPrefix: string;
  },
): Promise<unknown> {
  try {
    return JSON.parse(await readFile(
      path,
      'utf8',
    ),);
  }
  catch (error) {
    if (isFileMissingError(error,))
      return undefined;
    throw new Error(
      `${errorPrefix}: failed to read ${label} settings at ${path}: ${
        caughtValueText(error,)
      }`,
      { cause: error, },
    );
  }
}

/**
 Remove empty pattern strings.
 
 @param patterns - raw model patterns
 
 @returns cleaned model patterns
 */
function cleanPatterns(
  patterns: readonly string[],
): string[] {
  return patterns
    .map(function trimPattern(pattern,) {
      return pattern.trim();
    },)
    .filter(function keepPattern(pattern,) {
      return pattern !== '';
    },);
}

/**
 Detect Node ENOENT missing-file errors without unsafe assertion.
 
 @param error - caught error value
 
 @returns whether error reports a missing settings file
 */
function isFileMissingError(
  error: unknown,
): boolean {
  if ((!(Error.isError(error,))) || (!('code' in error)))
    return false;
  return error.code === 'ENOENT';
}

//endregion Internal helpers
