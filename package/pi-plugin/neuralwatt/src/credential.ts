/**
 Credential seam for the NeuralWatt pi extension:
 one resolution chain over pi's credential,
 the sops/age-encrypted `.env.local.json` store,
 and the environment.

 The chain order is the owner-accepted one from
 `doc/planning/neuralwatt-pi-plugin.md` (`cred-home`, `cold-start`):
 pi credential first,
 then the encrypted store,
 then the environment,
 and finally absence,
 which still keeps the provider registered.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { readEnvLocalJson, } from '@monochromatic-dev/env-local-json-core/ts';
import { NEURALWATT_API_KEY_ENV, } from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type {
  ApiKeyAuth,
  ApiKeyCredential,
  AuthContext,
  AuthResult,
} from '@earendil-works/pi-ai';
import { dirname, join, resolve as resolvePath, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import {
  ENV_LOCAL_JSON_PATH_VARIABLE,
  NEURALWATT_CREDENTIAL_GUIDANCE,
  NEURALWATT_CREDENTIAL_NAME,
} from './constants.ts';

//region Types

/**
 Which accepted home answered the credential question.
 */
export type NeuralWattCredentialSource =
  | 'pi-credential'
  | 'encrypted-store'
  | 'environment'
  | 'none';

/**
 Outcome of one credential resolution:
 the key when any home answered,
 and which home it came from.
 */
export type NeuralWattCredentialResolution = {
  /**
   Resolved key, `undefined` when no home carries one.
   */
  readonly key: string | undefined;
  /**
   Home that answered, `none` when none did.
   */
  readonly source: NeuralWattCredentialSource;
  /**
   Store path the encrypted store loaded from, when that home answered.
   */
  readonly storePath?: string;
};

/**
 One store read's outcome:
 the stored value under the credential name and the store path when loaded.
 */
export type NeuralWattStoreRead = {
  /**
   Stored value, `undefined` when the store carried none.
   */
  readonly value: string | undefined;
  /**
   Store path, `undefined` when no store was read.
   */
  readonly path: string | undefined;
};

/**
 Inputs one resolution reads.
 Injected so tests drive every home without touching real secrets.
 */
export type NeuralWattCredentialSources = {
  /**
   Value pi resolved from `--api-key` or its `auth.json`.
   */
  readonly credentialKey: string | undefined;
  /**
   Value the runtime environment carries under the credential name.
   */
  readonly environmentKey: string | undefined;
  /**
   Reads the encrypted store lazily, per resolve, with no caching.
   */
  readonly readStoreValue: () => Promise<NeuralWattStoreRead>;
};

//endregion Types

//region Resolution chain

/**
 First non-blank candidate, so whitespace never counts as a value.

 @param candidate - raw value read from one home

 @returns the trimmed value when one is present
 */
function firstNonBlank(
  candidate: string | undefined,
): string | undefined {
  if (candidate === undefined)
    return undefined;
  const trimmed = candidate.trim();
  return trimmed.length === 0 ? undefined : trimmed;
}

/**
 Resolve the credential across the accepted homes in owner-accepted order.
 A blank value is absence,
 so whitespace never masquerades as a key.

 @param sources - home values and the lazy store read

 @returns the first non-blank key and the home it came from
 */
export async function resolveNeuralWattCredential({
  sources,
}: {
  readonly sources: NeuralWattCredentialSources;
},): Promise<NeuralWattCredentialResolution> {
  /**
   pi credential already resolved at the request boundary.
   */
  const credentialKey = firstNonBlank(sources.credentialKey,);
  if (credentialKey !== undefined)
    return {
      key: credentialKey,
      source: 'pi-credential',
    };
  /**
   Encrypted store read, attempted fresh on every resolution.
   */
  const stored = await sources.readStoreValue();
  const storedKey = firstNonBlank(stored.value,);
  if (storedKey !== undefined)
    return {
      key: storedKey,
      source: 'encrypted-store',
      ...(stored.path === undefined ? {} : { storePath: stored.path, },),
    };
  /**
   Environment value, the launch-time fallback.
   */
  const environmentKey = firstNonBlank(sources.environmentKey,);
  if (environmentKey !== undefined)
    return {
      key: environmentKey,
      source: 'environment',
    };
  return {
    key: undefined,
    source: 'none',
  };
}

//endregion Resolution chain

//region Store path derivation

/**
 Derive the repository's `.env.local.json` from this package's own location,
 so a pi launch without the shell environment still finds the store.
 The walk finds the owning `package.json` first
 (`package/pi-plugin/neuralwatt` under any build layout),
 then steps to the repository root.

 @param moduleUrl - `import.meta.url` of the calling module

 @param fileExists - existence probe, injectable for tests

 @returns the store path when it validates as a file,
 `undefined` otherwise, never a hard error
 */
export async function deriveRepoEnvLocalJsonPath({
  moduleUrl,
  fileExists,
}: {
  readonly moduleUrl: string;
  readonly fileExists: (path: string) => Promise<boolean>;
},): Promise<string | undefined> {
  /**
   Directory holding the calling module, walked upward for the package root.
   */
  let directory = dirname(fileURLToPath(moduleUrl,),);
  for (;;) {
    if (await fileExists(join(directory, 'package.json',),))
      break;
    /**
     Parent directory, compared against the current one to detect the
     filesystem root.
     */
    const parent = dirname(directory,);
    if (parent === directory)
      return undefined;
    directory = parent;
  }
  /**
   Repository store candidate: `package/pi-plugin/neuralwatt` sits three
   levels below the repository root.
   */
  const candidate = resolvePath(
    directory,
    '..',
    '..',
    '..',
    '.env.local.json',
  );
  return await fileExists(candidate,) ? candidate : undefined;
}

//endregion Store path derivation

//region Store reader

/**
 Read the credential from the sops/age-encrypted store under the accepted
 order.
 The reader's own environment view hides the credential name,
 because `readEnvLocalJson` never returns values already present in the
 environment it is handed and the accepted order puts the store first.

 @param environment - launch environment the reader inspects

 @param moduleUrl - `import.meta.url` of the calling module

 @param fileExists - existence probe for the derived store candidate

 @param signal - abort signal forwarded to the store read

 @returns the stored value and the store path when a store loaded

 @throws whatever `readEnvLocalJson` throws when an explicit path misbehaves;
 callers log and fall through
 */
export async function readNeuralWattStoreValue({
  environment,
  moduleUrl,
  fileExists,
  signal,
}: {
  readonly environment: Readonly<Record<string, string | undefined>>;
  readonly moduleUrl: string;
  readonly fileExists: (path: string) => Promise<boolean>;
  readonly signal?: AbortSignal;
},): Promise<NeuralWattStoreRead> {
  /**
   Reader environment with the credential name hidden,
   so the store wins the accepted order over a stale environment value.
   */
  const readerEnvironment = { ...environment, };
  delete readerEnvironment[NEURALWATT_API_KEY_ENV];
  /**
   Non-blank store path override the library would honor on its own.
   */
  const override = firstNonBlank(environment[ENV_LOCAL_JSON_PATH_VARIABLE],);
  /**
   Derived repository store, offered only when no override exists and the
   candidate validates as a file.
   */
  const derived = override === undefined
    ? await deriveRepoEnvLocalJsonPath({ moduleUrl, fileExists, },)
    : undefined;
  /**
   Store read, discovery-only when neither an override nor a derived path
   applies.
   */
  const result = await readEnvLocalJson({
    keys: [NEURALWATT_API_KEY_ENV],
    env: { ...readerEnvironment, },
    ...(derived === undefined ? {} : { path: derived, },),
    ...(signal === undefined ? {} : { signal, },),
  },);
  return {
    value: result.values[NEURALWATT_API_KEY_ENV],
    path: result.loaded ? result.path : undefined,
  };
}

//endregion Store reader

//region Pi auth object

/**
 Existence probe for the derived store candidate,
 defaulting to the runtime filesystem.

 @param path - absolute path to test

 @returns whether the path is an existing file
 */
async function defaultFileExists(
  path: string,
): Promise<boolean> {
  const { stat, } = await import('node:fs/promises',);
  try {
    return (await stat(path,)).isFile();
  } catch {
    return false;
  }
}

/**
 pi status label naming the winning home without exposing the key.

 @param resolution - resolution whose home names the label

 @returns the credential name for pi's credential,
 the store path for the store,
 and the environment name for the environment
 */
function sourceLabel(
  resolution: NeuralWattCredentialResolution,
): string {
  switch (resolution.source) {
    case 'pi-credential':
      return NEURALWATT_CREDENTIAL_NAME;
    case 'encrypted-store':
      return resolution.storePath ?? 'encrypted env store';
    case 'environment':
      return NEURALWATT_API_KEY_ENV;
    case 'none':
      return 'no credential';
  }
}

/**
 Build the structural api-key auth pi's `Provider.auth` carries.
 `resolve` walks the accepted chain and reports the winning home as pi's
 `source` label;
 the key itself is never logged.

 @param logger - package logger, re-tagged for this seam

 @param environment - launch environment the store reader inspects

 @param moduleUrl - `import.meta.url` used for the derived store path

 @param fileExists - existence probe for the derived store candidate

 @returns the api-key auth object the provider serves

 @example
 ```typescript
 const auth = { apiKey: createNeuralWattApiKeyAuth({ logger }) };
 ```
 */
export function createNeuralWattApiKeyAuth({
  logger,
  environment = process.env,
  moduleUrl = import.meta.url,
  fileExists = defaultFileExists,
}: {
  readonly logger: Logger;
  readonly environment?: Readonly<Record<string, string | undefined>>;
  readonly moduleUrl?: string;
  readonly fileExists?: (path: string) => Promise<boolean>;
},): NonNullable<ApiKeyAuth> {
  /**
   Logger tagged for this seam, so records name the credential resolution.
   */
  const l = tagged({
    tag: createNeuralWattApiKeyAuth.name,
    l: logger,
  },);
  return {
    name: NEURALWATT_CREDENTIAL_NAME,
    resolve: async function resolveNeuralWattApiKey(input: {
      readonly ctx: AuthContext;
      readonly credential?: ApiKeyCredential;
      readonly signal: AbortSignal;
    },): Promise<AuthResult | undefined> {
      /**
       Logger tagged for this resolve, so records name the entry point.
       */
      const rl = tagged({
        tag: resolveNeuralWattApiKey.name,
        l,
      },);
      /**
       Environment value under the credential name, through pi's own env seam.
       */
      const environmentKey = await input.ctx.env(NEURALWATT_API_KEY_ENV,);
      /**
       Store read that logs and falls through instead of failing the request.
       */
      const readStoreValue = async function readStoreValue(): Promise<NeuralWattStoreRead> {
        try {
          return await readNeuralWattStoreValue({
            environment,
            moduleUrl,
            fileExists,
            signal: input.signal,
          },);
        } catch (error) {
          rl.warn(
            `encrypted store read failed, falling through to the environment: ${caughtValueText(error,)}`,
          );
          return {
            value: undefined,
            path: undefined,
          };
        }
      };
      /**
       Resolved key and the home it came from.
       */
      const resolution = await resolveNeuralWattCredential({
        sources: {
          credentialKey: input.credential?.key,
          environmentKey,
          readStoreValue,
        },
      },);
      if (resolution.key === undefined) {
        rl.info(NEURALWATT_CREDENTIAL_GUIDANCE,);
        return undefined;
      }
      return {
        auth: { apiKey: resolution.key, },
        source: sourceLabel(resolution,),
      };
    },
  };
}

//endregion Pi auth object
