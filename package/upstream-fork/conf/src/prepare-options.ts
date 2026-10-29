/**
 Options normalization for the config store.
 
 Fills upstream `conf` 15.1.0's defaults,
 validates the encryption algorithm,
 resolves the config directory,
 and strips leading dots from the file extension,
 so the rest of the store reads one fully-resolved options shape.
 
 @module
 */

import {
  InvalidEncryptionAlgorithmError,
  MissingProjectNameError,
} from './errors.ts';
import {
  DEFAULT_ENCRYPTION_ALGORITHM,
  SUPPORTED_ENCRYPTION_ALGORITHMS,
  isSupportedEncryptionAlgorithm,
} from './encryption.ts';
import { configDirectory, } from './config-dir.ts';
import type { Options, } from './options.ts';

//region Types

/**
 Options after defaults,
 validation,
 and path resolution,
 where every behavioral knob has a concrete value.
 
 @example
 ```ts
 const prepared: PreparedOptions<Record<string, unknown>> = prepareOptions({ cwd: '/tmp/app', });
 prepared.configName; // 'config'
 ```
 */
export type PreparedOptions<T extends Record<string, unknown>> = Readonly<Required<Pick<Options<T>, | 'cwd'
  | 'configName'
  | 'fileExtension'
  | 'projectSuffix'
  | 'clearInvalidConfig'
  | 'accessPropertiesByDotNotation'
  | 'configFileMode'
  | 'encryptionAlgorithm'>>> & Options<T>;

//endregion Types

//region Constants

/**
 Default config file name without extension.
 
 @example
 ```ts
 DEFAULT_CONFIG_NAME; // 'config'
 ```
 */
const DEFAULT_CONFIG_NAME = 'config';

/**
 Default config file extension without a leading dot.
 
 @example
 ```ts
 DEFAULT_FILE_EXTENSION; // 'json'
 ```
 */
const DEFAULT_FILE_EXTENSION = 'json';

/**
 Default suffix appended to `projectName` to avoid clashing with native
 apps.
 
 @example
 ```ts
 DEFAULT_PROJECT_SUFFIX; // 'nodejs'
 ```
 */
const DEFAULT_PROJECT_SUFFIX = 'nodejs';

/**
 Default config file mode before the process umask reduces it.
 
 @example
 ```ts
 DEFAULT_CONFIG_FILE_MODE; // 0o666
 ```
 */
const DEFAULT_CONFIG_FILE_MODE = 0o666;

//endregion Constants

//region Helpers

/**
 Strips leading dots from a caller-supplied file extension.
 
 @param fileExtension - Extension as the caller wrote it,
 possibly `.json`-style.
 
 @returns Extension without leading dots.
 
 @example
 ```ts
 stripLeadingDots('..json'); // 'json'
 ```
 */
function stripLeadingDots(fileExtension: string,): string {
  /**
   Cursor walking forward over the leading-dot run.
   */
  const cursor = {
    index: 0,
  };
  while ((cursor.index < fileExtension.length) && (fileExtension[cursor.index] === '.'))
    cursor.index += 1;
  return fileExtension.slice(cursor.index,);
}

//endregion Helpers

//region Preparation

/**
 Normalizes caller options into the concrete shape the store runs on.
 
 @param partialOptions - Caller options; every field is optional.
 
 @returns Options with defaults filled,
 the algorithm validated,
 `cwd` resolved,
 and the file extension normalized.
 
 @throws InvalidEncryptionAlgorithmError When `encryptionAlgorithm` names
 an unsupported algorithm.
 
 @throws MissingProjectNameError When neither `cwd` nor `projectName`
 resolves a config directory.
 
 @example
 ```ts
 const prepared = prepareOptions({
   projectName: 'foo',
   fileExtension: '.json',
 });
 prepared.fileExtension; // 'json'
 ```
 */
export function prepareOptions<T extends Record<string, unknown>>(partialOptions: Options<T>,): PreparedOptions<T> {
  /**
   Caller options with every defaulted knob filled in.
   */
  const options = {
    configName: DEFAULT_CONFIG_NAME,
    fileExtension: DEFAULT_FILE_EXTENSION,
    projectSuffix: DEFAULT_PROJECT_SUFFIX,
    clearInvalidConfig: false,
    accessPropertiesByDotNotation: true,
    configFileMode: DEFAULT_CONFIG_FILE_MODE,
    ...partialOptions,
    encryptionAlgorithm: partialOptions.encryptionAlgorithm ?? DEFAULT_ENCRYPTION_ALGORITHM,
  };
  if (!isSupportedEncryptionAlgorithm(options.encryptionAlgorithm,))
    throw new InvalidEncryptionAlgorithmError({
      supported: [...SUPPORTED_ENCRYPTION_ALGORITHMS,],
    },);
  /**
   File extension after leading dots are stripped;
   non-string extensions pass through untouched.
   */
  const normalizedFileExtension = (typeof options.fileExtension) === 'string'
    ? stripLeadingDots(options.fileExtension,)
    : options.fileExtension;
  if ((options.cwd === undefined) || (options.cwd === '')) {
    if ((options.projectName === undefined) || (options.projectName === ''))
      throw new MissingProjectNameError();
    return {
      ...options,
      cwd: configDirectory({
        projectName: options.projectName,
        projectSuffix: options.projectSuffix,
      },),
      fileExtension: normalizedFileExtension,
    };
  }
  return {
    ...options,
    cwd: options.cwd,
    fileExtension: normalizedFileExtension,
  };
}

//endregion Preparation
