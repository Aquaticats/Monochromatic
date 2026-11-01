/**
 Decryption identity resolution that mirrors mise's sops/age configuration.

 The order is `MISE_SOPS_AGE_KEY`, an existing `MISE_SOPS_AGE_KEY_FILE`, an
 existing `SOPS_AGE_KEY_FILE`, `SOPS_AGE_KEY`, then the mise configuration
 directory's `age.txt` (honoring `MISE_CONFIG_DIR`, then `XDG_CONFIG_HOME`,
 then `$HOME/.config`). Missing identities name every candidate consulted.

 @module
 */

import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  CURATED_CHILD_ENV_NAMES,
  CURATED_EDITOR_ENV_NAMES,
  MISE_AGE_IDENTITY_FILE_NAME,
} from './constants.ts';
import { isNonBlank, } from './environment.ts';
import { EnvLocalJsonMissingIdentityError, } from './errors.ts';
import type { EnvLocalJsonEnvironment, } from './types.ts';

//region Constants

/**
 Logger for identity candidate decisions; names variables and paths only.
 */
const identityLogger = tagged({ tag: 'envLocalJsonIdentity', },);

/**
 Directory name under `$HOME/.config` that holds mise configuration.
 */
const MISE_CONFIG_DIRECTORY_NAME = 'mise';

//endregion Constants

//region Types

/**
 Environment name or configuration root that supplied the identity.
 */
export type SopsIdentitySource =
  | 'MISE_SOPS_AGE_KEY'
    | 'MISE_SOPS_AGE_KEY_FILE'
    | 'SOPS_AGE_KEY_FILE'
    | 'SOPS_AGE_KEY'
    | 'mise-config-dir'
    | 'xdg-config-home'
    | 'home-config';

/**
 Resolved decryption identity.

 `value` holds the inline age key for `inline` identities and the identity
 file path for `file` identities. Neither errors nor logs ever include it.
 */
export type SopsIdentity = {
  /**
   Whether `value` is inline key material or an identity file path.
   */
  readonly kind: 'inline' | 'file';

  /**
   Candidate that supplied this identity.
   */
  readonly source: SopsIdentitySource;

  /**
   Inline key material or identity file path.
   */
  readonly value: string;
};

/**
 Options for {@link resolveSopsIdentity}.
 */
export type ResolveSopsIdentityOptions = {
  /**
   Environment whose identity names are consulted.
   */
  readonly env: EnvLocalJsonEnvironment;

  /**
   Home directory used for the default mise configuration path.
   */
  readonly home: string;
};

/**
 Options for {@link buildSopsChildEnv}.
 */
export type BuildSopsChildEnvOptions = {
  /**
   Parent environment to copy curated names from.
   */
  readonly env: EnvLocalJsonEnvironment;

  /**
   Home directory forwarded when the parent environment has no `HOME`.
   */
  readonly home: string;

  /**
   Identity to expose to the sops child.
   */
  readonly identity: SopsIdentity;

  /**
   Whether editor selection variables are needed for interactive editing.
   */
  readonly interactive?: boolean;
};

//endregion Types

//region Predicates

/**
 Whether the named file exists as a regular file.

 @param path - candidate identity path

 @returns `true` when the path stats as a regular file

 @example
 ```ts
 await isExistingFile({ path: identityPath });
 ```
 */
async function isExistingFile({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    /**
     Metadata of the candidate path.
     */
    const metadata = await stat(path,);
    return metadata.isFile();
  }
  catch (error) {
    identityLogger.debug(
      `identity candidate ${path} is not usable: ${caughtValueText(error,)}`,
    );
    return false;
  }
}

/**
 Describes whether a candidate environment variable is set, without echoing
 its value.

 @param value - raw environment value

 @returns short state description for the missing-identity report

 @example
 ```ts
 describeEnvironmentState(undefined); // 'unset'
 ```
 */
function describeEnvironmentState(value: unknown,): string {
  if (value === undefined)
    return 'unset';
  if (!isNonBlank(value,))
    return 'set but blank';
  return 'set';
}

//endregion Predicates

//region Configuration directory

/**
 Resolves the mise configuration directory and which source named it.

 @param env - environment holding configuration-directory overrides

 @param home - fallback home directory

 @returns absolute configuration directory and source label

 @example
 ```ts
 miseConfigurationBase({ env, home }); // { dir: '/home/user/.config/mise', source: 'home-config' }
 ```
 */
function miseConfigurationBase({
  env,
  home,
}: {
  readonly env: EnvLocalJsonEnvironment;
  readonly home: string;
},): {
  readonly dir: string;
  readonly source: SopsIdentitySource;
} {
  /**
   Explicit mise configuration directory, when set.
   */
  const configured = env.MISE_CONFIG_DIR;
  if (isNonBlank(configured,))
    return {
      dir: configured,
      source: 'mise-config-dir',
    };
  /**
   XDG configuration root, when set.
   */
  const xdg = env.XDG_CONFIG_HOME;
  if (isNonBlank(xdg,)) {
    return {
      dir: join(
        xdg,
        MISE_CONFIG_DIRECTORY_NAME,
      ),
      source: 'xdg-config-home',
    };
  }
  return {
    dir: join(
      home,
      '.config',
      MISE_CONFIG_DIRECTORY_NAME,
    ),
    source: 'home-config',
  };
}

//endregion Configuration directory

//region Identity resolution

/**
 Resolves the decryption identity by the same precedence mise uses.

 @param env - environment holding identity variables

 @param home - home directory for the default mise identity path

 @returns first usable inline or file identity

 @throws EnvLocalJsonMissingIdentityError when every candidate is unusable;
 the error lists all candidates and their state, never key material

 @example
 ```ts
 const identity = await resolveSopsIdentity({ env: process.env, home: os.homedir() });
 ```
 */
export async function resolveSopsIdentity({
  env,
  home,
}: ResolveSopsIdentityOptions,): Promise<SopsIdentity> {
  /**
   Ordered descriptions of every candidate consulted, for the error report.
   */
  const candidates: string[] = [];

  /**
   Inline key from mise's own variable.
   */
  const miseInlineKey = env.MISE_SOPS_AGE_KEY;
  if (isNonBlank(miseInlineKey,)) {
    identityLogger.debug('using MISE_SOPS_AGE_KEY as inline identity',);
    return {
      kind: 'inline',
      source: 'MISE_SOPS_AGE_KEY',
      value: miseInlineKey,
    };
  }
  candidates.push(`MISE_SOPS_AGE_KEY (${describeEnvironmentState(miseInlineKey,)})`,);

  /**
   Identity file named by mise's file variable.
   */
  const miseKeyFile = env.MISE_SOPS_AGE_KEY_FILE;
  if (isNonBlank(miseKeyFile,)) {
    if (await isExistingFile({ path: miseKeyFile, },)) {
      identityLogger.debug(`using MISE_SOPS_AGE_KEY_FILE ${miseKeyFile}`,);
      return {
        kind: 'file',
        source: 'MISE_SOPS_AGE_KEY_FILE',
        value: miseKeyFile,
      };
    }
    candidates.push(`MISE_SOPS_AGE_KEY_FILE (${miseKeyFile}, missing)`,);
  }
  else {
    candidates.push(
      `MISE_SOPS_AGE_KEY_FILE (${describeEnvironmentState(miseKeyFile,)})`,
    );
  }

  /**
   Identity file named by sops's own file variable.
   */
  const sopsKeyFile = env.SOPS_AGE_KEY_FILE;
  if (isNonBlank(sopsKeyFile,)) {
    if (await isExistingFile({ path: sopsKeyFile, },)) {
      identityLogger.debug(`using SOPS_AGE_KEY_FILE ${sopsKeyFile}`,);
      return {
        kind: 'file',
        source: 'SOPS_AGE_KEY_FILE',
        value: sopsKeyFile,
      };
    }
    candidates.push(`SOPS_AGE_KEY_FILE (${sopsKeyFile}, missing)`,);
  }
  else {
    candidates.push(`SOPS_AGE_KEY_FILE (${describeEnvironmentState(sopsKeyFile,)})`,);
  }

  /**
   Inline key from sops's own variable.
   */
  const sopsInlineKey = env.SOPS_AGE_KEY;
  if (isNonBlank(sopsInlineKey,)) {
    identityLogger.debug('using SOPS_AGE_KEY as inline identity',);
    return {
      kind: 'inline',
      source: 'SOPS_AGE_KEY',
      value: sopsInlineKey,
    };
  }
  candidates.push(`SOPS_AGE_KEY (${describeEnvironmentState(sopsInlineKey,)})`,);

  /**
   Configuration directory holding the default mise age identity.
   */
  const base = miseConfigurationBase({
    env,
    home,
  },);
  /**
   Default identity path under the resolved mise configuration directory.
   */
  const defaultPath = join(
    base.dir,
    MISE_AGE_IDENTITY_FILE_NAME,
  );
  if (await isExistingFile({ path: defaultPath, },)) {
    identityLogger.debug(`using mise identity ${defaultPath}`,);
    return {
      kind: 'file',
      source: base.source,
      value: defaultPath,
    };
  }
  candidates.push(`${defaultPath} (mise identity default, missing)`,);

  throw new EnvLocalJsonMissingIdentityError({ candidates, },);
}

//endregion Identity resolution

//region Child environment

/**
 Builds the curated environment passed to every sops child process.

 Only the named allow-list, the identity variable, and (for interactive
 editing) the editor selection variables reach sops; unrelated credentials
 in the parent environment never do.

 @param env - parent environment

 @param home - home directory forwarded when the parent lacks `HOME`

 @param identity - identity chosen by {@link resolveSopsIdentity}

 @param interactive - whether editor selection variables are forwarded

 @returns read-only child environment

 @example
 ```ts
 const childEnv = buildSopsChildEnv({ env: process.env, home: os.homedir(), identity });
 ```
 */
export function buildSopsChildEnv({
  env,
  home,
  identity,
  interactive = false,
}: BuildSopsChildEnvOptions,): Readonly<Record<string, string>> {
  /**
   Curated child environment being assembled.
   */
  const childEnv: Record<string, string> = {};

  for (const name of CURATED_CHILD_ENV_NAMES) {
    if (name === 'HOME')
      continue;
    /**
     Parent value for this allow-listed name.
     */
    const value = env[name];
    if (isNonBlank(value,))
      childEnv[name] = value;
  }

  /**
   Parent home if set, otherwise the caller-provided home.
   */
  const parentHome = env.HOME;
  childEnv.HOME = isNonBlank(parentHome,)
    ? parentHome
    : home;

  if (childEnv.PATH === undefined) {
    /**
     Windows-cased PATH fallback from the parent environment.
     */
    const windowsPath = env.Path;
    if (isNonBlank(windowsPath,))
      childEnv.PATH = windowsPath;
  }

  if (interactive) {
    for (const name of CURATED_EDITOR_ENV_NAMES) {
      /**
       Parent value for this editor variable.
       */
      const value = env[name];
      if (isNonBlank(value,))
        childEnv[name] = value;
    }
  }

  if (identity.kind === 'inline')
    childEnv.SOPS_AGE_KEY = identity.value;
  else
    childEnv.SOPS_AGE_KEY_FILE = identity.value;

  return childEnv;
}

//endregion Child environment
