import { readFile, } from 'node:fs/promises';
import { homedir, } from 'node:os';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { askUserQuestionConfigPath, } from './config-path.ts';
import { validateAskUserQuestionConfig, } from './config-schema.ts';
import {
  type AskUserQuestionConfig,
  AskUserQuestionConfigError,
  type AskUserQuestionConfigReadResult,
  type LoadAskUserQuestionConfigOptions,
} from './config-types.ts';
import {
  editorEnvironmentFromProcess,
  resolveEditorCommand,
} from './editor-command.ts';

//region Constants

/**
 Missing-file system error code.
 */
const FILE_NOT_FOUND_CODE = 'ENOENT';

//endregion Constants

//region Logger

/**
 Tagged logger for user config loading.
 */
const l = tagged({ tag: 'ask-user-question:config', },);

//endregion Logger

//region File reading

/**
 Narrows missing-file system errors without unsafe assertion.
 
 @param error - caught filesystem value
 
 @returns whether error carries ENOENT code
 */
function isFileNotFoundError(error: unknown,): boolean {
  if (!Error.isError(error,))
    return false;
  if (!('code' in error))
    return false;
  return error.code === FILE_NOT_FOUND_CODE;
}

/**
 Reads optional JSON config and validates its shape.
 
 @param configPath - absolute user config path
 
 @returns validated file and presence marker
 
 @throws {@link AskUserQuestionConfigError} for read,
 parse,
 or shape failures
 */
async function readConfigFile(
  { configPath, }: { readonly configPath: string; },
): Promise<AskUserQuestionConfigReadResult> {
  try {
    /**
     Decoded JSON candidate.
     */
    const value: unknown = JSON.parse(await readFile(
      configPath,
      'utf8',
    ),);
    l.debug(`loaded user config from ${configPath}`,);
    return {
      loaded: true,
      value: validateAskUserQuestionConfig({
        value,
        configPath,
      },),
    };
  }
  catch (error: unknown) {
    if (isFileNotFoundError(error,)) {
      l.debug(`user config absent at ${configPath}`,);
      return {
        loaded: false,
        value: {},
      };
    }
    if (error instanceof AskUserQuestionConfigError)
      throw error;
    throw new AskUserQuestionConfigError(
      `Ask-user-question config could not be read at ${configPath}: ${String(error,)}`,
      error,
    );
  }
}

//endregion File reading

//region Public loading

/**
 Loads user editor override and resolves effective editor command.
 
 User config takes precedence over VISUAL,
 then EDITOR,
 then platform fallback.
 
 @param options - optional home,
 environment,
 and platform overrides
 
 @returns effective editor command and source metadata
 
 @throws {@link AskUserQuestionConfigError} when config cannot be read or validated
 
 @example
 ```ts
 await loadAskUserQuestionConfig();
 ```
 */
export async function loadAskUserQuestionConfig(
  options: LoadAskUserQuestionConfigOptions = {},
): Promise<AskUserQuestionConfig> {
  /**
   Effective home without hardcoded account path.
   */
  const home = options.home
    ?? process.env
      .HOME
    ?? homedir();
  /**
   User config path.
   */
  const configPath = askUserQuestionConfigPath({ home, },);
  /**
   Optional parsed config file.
   */
  const file = await readConfigFile({ configPath, },);
  /**
   Effective editor environment.
   */
  const env = options.env
    ?? editorEnvironmentFromProcess(process.env,);
  try {
    return {
      editorCommand: resolveEditorCommand({
        ...(file.value
          .editor
          === undefined
          ? {}
          : { configuredEditor: file.value
            .editor, }),
        env,
        platform: options.platform
          ?? process.platform,
      },),
      source: {
        path: configPath,
        loaded: file.loaded,
      },
    };
  }
  catch (error: unknown) {
    throw new AskUserQuestionConfigError(
      `Ask-user-question editor config is invalid at ${configPath}: ${String(error,)}`,
      error,
    );
  }
}

//endregion Public loading

export { askUserQuestionConfigPath, } from './config-path.ts';
export {
  type AskUserQuestionConfig,
  AskUserQuestionConfigError,
  type LoadAskUserQuestionConfigOptions,
} from './config-types.ts';
