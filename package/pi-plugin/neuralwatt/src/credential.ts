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

import {
  NEURALWATT_API_KEY_ENV,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  readEnvLocalJson,
  type EnvLocalJsonEnvironment,
} from '@monochromatic-dev/env-local-json-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type {
  ApiKeyAuth,
  ApiKeyCredential,
  AuthContext,
} from '@earendil-works/pi-ai';
import {
  dirname,
  join,
  resolve as resolvePath,
} from 'node:path';
import {
  fileURLToPath,
} from 'node:url';
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
   Resolved key, absent when no home carries one.
   */
  readonly key?: string;
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
   Stored value, absent when the store carried none.
   */
  readonly value?: string;
  /**
   Store path, absent when no store was read.
   */
  readonly path?: string;
};

/**
 Inputs one resolution reads.
 Injected so tests drive every home without touching real secrets.
 */
export type NeuralWattCredentialSources = {
  /**
   Value pi resolved from `--api-key` or its `auth.json`.
   */
  readonly credentialKey?: string;
  /**
   Value the runtime environment carries under the credential name.
   */
  readonly environmentKey?: string;
  /**
   Reads the encrypted store lazily, per resolve, with no caching.
   */
  readonly readStoreValue: () => Promise<NeuralWattStoreRead>;
};

//endregion Types

//region Resolution chain

/**
 Resolve the credential across the accepted homes in owner-accepted order.
 Trimming happens at each home,
 so whitespace never masquerades as a key.

 @param sources - home values and the lazy store read

 @returns the first real key and the home it came from

 @example
 ```typescript
 const resolution = await resolveNeuralWattCredential({ sources });
 ```
 */
export async function resolveNeuralWattCredential({
  sources,
}: {
  readonly sources: NeuralWattCredentialSources;
},): Promise<NeuralWattCredentialResolution> {
  /**
   pi credential already resolved at the request boundary.
   */
  const credentialKey = sources.credentialKey
    ?.trim();
  if ((credentialKey !== undefined)
    && (credentialKey.length > 0))
    return {
      key: credentialKey,
      source: 'pi-credential',
    };
  /**
   Encrypted store read, attempted fresh on every resolution.
   */
  const stored = await sources.readStoreValue();
  /**
   Stored key, trimmed like every other home.
   */
  const storedKey = stored.value
    ?.trim();
  if ((storedKey !== undefined)
    && (storedKey.length > 0))
    return {
      key: storedKey,
      source: 'encrypted-store',
      ...(stored.path === undefined ? {} : { storePath: stored.path, }),
    };
  /**
   Environment value, the launch-time fallback.
   */
  const environmentKey = sources.environmentKey
    ?.trim();
  if ((environmentKey !== undefined)
    && (environmentKey.length > 0))
    return {
      key: environmentKey,
      source: 'environment',
    };
  return {
    source: 'none',
  };
}

//endregion Resolution chain

//region Store path derivation

/**
 Collect the ancestor directories of one starting directory, nearest first,
 ending at the filesystem root.

 @param start - absolute directory the walk starts in

 @returns every ancestor, the starting directory included
 */
function collectAncestors({
  start,
}: {
  readonly start: string;
},): string[] {
  /**
   Ancestors collected nearest first.
   */
  const directories: string[] = [];
  for (
    let current = start;
    ;
    current = dirname(current,)
  ) {
    directories.push(current,);
    if (dirname(current,) === current)
      return directories;
  }
}

/**
 Derive the repository's `.env.local.json` from this package's own location,
 so a pi launch without the shell environment still finds the store.
 The walk finds the owning `package.json` first
 (`package/pi-plugin/neuralwatt` under any build layout),
 then steps to the repository root.

 @param moduleUrl - `import.meta.url` of the calling module

 @param fileExists - existence probe, injectable for tests

 @returns the store path when it validates as a file,
 otherwise a result with the path absent, never a hard error

 @example
 ```typescript
 const { path } = await deriveRepoEnvLocalJsonPath({ moduleUrl, fileExists });
 ```
 */
export async function deriveRepoEnvLocalJsonPath({
  moduleUrl,
  fileExists,
}: {
  readonly moduleUrl: string;
  readonly fileExists: (path: string) => Promise<boolean>;
},): Promise<{ path?: string }> {
  /**
   Ancestors of the calling module, probed for the package manifest below.
   */
  const ancestors = collectAncestors({
    start: dirname(fileURLToPath(moduleUrl,),),
  },);
  /**
   Manifest probes, one per ancestor, resolved together so the walk never
   awaits inside a loop.
   */
  const manifests = await Promise.all(ancestors.map(function probe(directory,) {
    return fileExists(join(
      directory,
      'package.json',
    ),);
  },),);
  /**
   Index of the nearest ancestor carrying a package manifest.
   */
  const packageRootIndex = manifests.indexOf(true,);
  /**
   Nearest manifest-bearing ancestor, absent when the walk found none.
   */
  const packageRoot = ancestors[packageRootIndex];
  if ((packageRootIndex === (-1))
    || (packageRoot === undefined))
    return {};
  /**
   Repository store candidate: `package/pi-plugin/neuralwatt` sits three
   levels below the repository root.
   */
  const candidate = resolvePath(
    packageRoot,
    '..',
    '..',
    '..',
    '.env.local.json',
  );
  return await fileExists(candidate,) ? { path: candidate, } : {};
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

 @param readStore - store reader seam, injected in tests so no run decrypts

 @param signal - abort signal forwarded to the store read

 @returns the stored value and the store path when a store loaded

 @throws whatever `readEnvLocalJson` throws when an explicit path misbehaves;
 callers log and fall through

 @example
 ```typescript
 const stored = await readNeuralWattStoreValue({ environment, moduleUrl, fileExists });
 ```
 */
export async function readNeuralWattStoreValue({
  environment,
  moduleUrl,
  fileExists,
  readStore = readEnvLocalJson,
  signal,
}: {
  readonly environment: EnvLocalJsonEnvironment;
  readonly moduleUrl: string;
  readonly fileExists: (path: string) => Promise<boolean>;
  readonly readStore?: typeof readEnvLocalJson;
  readonly signal?: AbortSignal;
},): Promise<NeuralWattStoreRead> {
  /**
   Reader environment with the credential name hidden,
   so the store wins the accepted order over a stale environment value.
   Names are filtered before any pair is built,
   so no callback parameter carries a mutable tuple.
   */
  const readerEnvironment = Object.fromEntries(Object.keys(environment,)
    .filter(
    function keepsNonCredential(name: string,) {
      return name !== NEURALWATT_API_KEY_ENV;
    },
    )
    .map(
    function toEnvironmentEntry(name: string,) {
      return [
        name,
        environment[name],
      ] as const;
    },
    ),);
  /**
   Non-blank store path override the library would honor on its own.
   */
  const override = environment[ENV_LOCAL_JSON_PATH_VARIABLE]
    ?.trim();
  /**
   Derived repository store, sought only when no override exists.
   */
  const derived = ((override !== undefined) && (override.length > 0))
    ? {}
    : await deriveRepoEnvLocalJsonPath({
      moduleUrl,
      fileExists,
    },);
  /**
   Store read, discovery-only when neither an override nor a derived path
   applies.
   */
  const result = await readStore({
    keys: [NEURALWATT_API_KEY_ENV],
    env: readerEnvironment,
    ...(derived.path === undefined ? {} : { path: derived.path, }),
    ...(signal === undefined ? {} : { signal, }),
  },);
  /**
   Stored value under the credential name, absent when the store carried
   none.
   */
  const value = result.values[NEURALWATT_API_KEY_ENV];
  return {
    ...(value === undefined ? {} : { value, }),
    ...(result.path === undefined ? {} : { path: result.path, }),
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
  /**
   Runtime filesystem probe, imported lazily so browser-shaped consumers
   never load it.
   */
  const { stat, } = await import('node:fs/promises',);
  return stat(path,)
    .then(
    function isRegularFile(result,) {
      return result.isFile();
    },
    function statRefused() {
      return false;
    },
    );
}

/**
 pi status labels for the home that answered, keyed by source.
 */
const SOURCE_LABELS: Record<NeuralWattCredentialSource, string> = {
  'pi-credential': NEURALWATT_CREDENTIAL_NAME,
  'encrypted-store': 'encrypted env store',
  'environment': NEURALWATT_API_KEY_ENV,
  'none': 'no credential',
};

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
  if (resolution.source === 'encrypted-store')
    return resolution.storePath ?? SOURCE_LABELS['encrypted-store'];
  return SOURCE_LABELS[resolution.source];
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
  readStore = readEnvLocalJson,
}: {
  readonly logger: Logger;
  readonly environment?: EnvLocalJsonEnvironment;
  readonly moduleUrl?: string;
  readonly fileExists?: (path: string) => Promise<boolean>;
  readonly readStore?: typeof readEnvLocalJson;
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
    /**
     Resolve the credential through the accepted chain at request time.

     @param input - pi's auth context, stored credential, and abort signal

     @returns the resolved key with its home label,
     or `undefined` when no home answers
     */
    resolve: async function resolveNeuralWattApiKey(input) {
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
      const environmentKey = await input.ctx
        .env(NEURALWATT_API_KEY_ENV,);
      /**
       Store read that logs and falls through instead of failing the request.

       @returns the stored value and path when a store answered,
       otherwise an empty result
       */
      async function readStoreValue(): Promise<NeuralWattStoreRead> {
        try {
          return await readNeuralWattStoreValue({
            environment,
            moduleUrl,
            fileExists,
            readStore,
            signal: input.signal,
          },);
        } catch (error) {
          rl.warn(
            `encrypted store read failed, falling through to the environment: ${caughtValueText(error,)}`,
          );
          return {};
        }
      }
      /**
       pi credential key, narrowed before the sources object is built so no
       optional slot receives an explicit undefined.
       */
      const credentialKey = input.credential
        ?.key;
      /**
       Resolved key and the home it came from.
       */
      const resolution = await resolveNeuralWattCredential({
        sources: {
          readStoreValue,
          ...(credentialKey === undefined ? {} : { credentialKey, }),
          ...(environmentKey === undefined ? {} : { environmentKey, }),
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
