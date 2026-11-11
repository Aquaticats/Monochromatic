/**
 Credential seam for the Qwen Token Plan Individual pi extension:
 one resolution chain over pi's stored credential and two environment names.

 The chain order is the owner-decided one from
 `doc/planning/qwen-token-plan-individual-provider.md`
 ("Install and credential seams"):
 the `qwen-token-plan-individual` entry pi already resolves
 (its credential store keeps working because the override keeps the provider
 id),
 then `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY`,
 then the builtin's own `QWEN_TOKEN_PLAN_API_KEY`,
 and finally absence,
 which still keeps the provider registered and the catalog served.

 @module
 */

import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type {
  ApiKeyAuth,
  ApiKeyCredential,
  Credential,
} from '@earendil-works/pi-ai';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL,
} from './constants.ts';

//region Types

/**
 Which accepted home answered the credential question.
 */
export type QwenTokenPlanIndividualCredentialSource =
  | 'pi-credential'
    | 'individual-environment'
    | 'builtin-environment'
    | 'none';

/**
 Outcome of one credential resolution:
 the key when any home answered,
 and which home it came from.
 */
export type QwenTokenPlanIndividualCredentialResolution = {
  /**
   Resolved key, absent when no home carries one.
   */
  readonly key?: string;
  /**
   Home that answered, `none` when none did.
   */
  readonly source: QwenTokenPlanIndividualCredentialSource;
};

/**
 Inputs one resolution reads.
 Injected so tests drive every home without touching real secrets.
 */
export type QwenTokenPlanIndividualCredentialSources = {
  /**
   Value pi resolved from `--api-key` or its `auth.json`.
   */
  readonly credentialKey?: string;
  /**
   Value the launch environment carries under the credential name the shared
   core pins.
   */
  readonly individualEnvironmentKey?: string;
  /**
   Value the launch environment carries under the builtin's own environment
   name, the recorded fallback.
   */
  readonly builtinEnvironmentKey?: string;
};

//endregion Types

//region Resolution chain

/**
 Resolve the credential across the accepted homes in owner-decided order.
 Trimming happens at each home,
 so whitespace never masquerades as a key.

 @param sources - home values

 @returns the first real key and the home it came from

 @example
 ```typescript
 const resolution = resolveQwenTokenPlanIndividualCredential({ sources });
 ```
 */
export function resolveQwenTokenPlanIndividualCredential({
  sources,
}: {
  readonly sources: QwenTokenPlanIndividualCredentialSources;
},): QwenTokenPlanIndividualCredentialResolution {
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
   Environment value under the credential name the shared core pins.
   */
  const individualEnvironmentKey = sources.individualEnvironmentKey
    ?.trim();
  if ((individualEnvironmentKey !== undefined)
    && (individualEnvironmentKey.length > 0))
    return {
      key: individualEnvironmentKey,
      source: 'individual-environment',
    };
  /**
   Environment value under the builtin's own name, the recorded fallback.
   */
  const builtinEnvironmentKey = sources.builtinEnvironmentKey
    ?.trim();
  if ((builtinEnvironmentKey !== undefined)
    && (builtinEnvironmentKey.length > 0))
    return {
      key: builtinEnvironmentKey,
      source: 'builtin-environment',
    };
  return {
    source: 'none',
  };
}

//endregion Resolution chain

//region Stored credential narrowing

/**
 Whether a stored credential carries an api key field to read.
 pi's `Credential` union also holds OAuth records,
 whose index signature would answer `key` with a non-string.

 @param credential - credential pi resolved for this route

 @returns `true` only for the api-key record

 @example
 ```typescript
 if (isApiKeyCredential(credential)) {
   // credential.key is the stored key here
 }
 ```
 */
export function isApiKeyCredential(
  credential: ForeignBorrowed<Credential>,
): credential is ApiKeyCredential {
  return credential.type === 'api_key';
}

/**
 Stored api-key value pi resolved for this route,
 expressed as the credential sources fragment it fills:
 absence is the fragment carrying no `credentialKey`,
 so no nullish union crosses the seam.

 @param credential - credential pi resolved, absent when it resolved none

 @returns the sources fragment carrying the stored key when one landed

 @example
 ```typescript
 const sources = { ...storedCredentialSources({ credential: context.credential }) };
 ```
 */
export function storedCredentialSources({
  credential,
}: {
  readonly credential?: ForeignBorrowed<Credential>;
},): QwenTokenPlanIndividualCredentialSources {
  if (credential === undefined)
    return {};
  if (!isApiKeyCredential(credential))
    return {};
  if (credential.key === undefined)
    return {};
  return { credentialKey: credential.key, };
}

//endregion Stored credential narrowing

//region Pi auth object

/**
 pi status labels for the home that answered, keyed by source.
 */
const SOURCE_LABELS: Record<QwenTokenPlanIndividualCredentialSource, string> = {
  'pi-credential': QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL,
  'individual-environment': QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
  'builtin-environment': QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
  'none': 'no credential',
};

/**
 Build the structural api-key auth pi's `Provider.auth` carries.
 `resolve` walks the accepted chain and reports the winning home as pi's
 `source` label;
 the key itself is never logged.

 @param logger - package logger, re-tagged for this seam

 @returns the api-key auth object the provider serves

 @example
 ```typescript
 const auth = createQwenTokenPlanIndividualApiKeyAuth({ logger });
 ```
 */
export function createQwenTokenPlanIndividualApiKeyAuth({
  logger,
}: {
  readonly logger: Logger;
},): NonNullable<ApiKeyAuth> {
  /**
   Logger tagged for this seam, so records name the credential resolution.
   */
  const l = tagged({
    tag: createQwenTokenPlanIndividualApiKeyAuth.name,
    l: logger,
  },);
  return {
    name: QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL,
    /**
     Resolve the credential through the accepted chain at request time.

     @param input - pi's auth context, stored credential, and abort signal

     @returns the resolved key with its home label,
     or `undefined` when no home answers
     */
    resolve: async function resolveQwenTokenPlanIndividualApiKey(input) {
      /**
       Logger tagged for this resolve, so records name the entry point.
       */
      const rl = tagged({
        tag: resolveQwenTokenPlanIndividualApiKey.name,
        l,
      },);
      /**
       Environment values under both accepted names, through pi's own env seam.
       */
      const [individualEnvironmentKey, builtinEnvironmentKey,] = await Promise.all([
        input.ctx
          .env(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,),
        input.ctx
          .env(QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,),
      ],);
      /**
       Sources fragment the stored credential fills, when pi resolved one.
       */
      const storedSources = storedCredentialSources(input.credential === undefined
        ? {}
        : { credential: input.credential, },);
      /**
       Resolved key and the home it came from.
       */
      const resolution = resolveQwenTokenPlanIndividualCredential({
        sources: {
          ...storedSources,
          ...(individualEnvironmentKey === undefined
            ? {}
            : { individualEnvironmentKey, }),
          ...(builtinEnvironmentKey === undefined
            ? {}
            : { builtinEnvironmentKey, }),
        },
      },);
      if (resolution.key === undefined) {
        rl.info(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,);
        return undefined;
      }
      return {
        auth: { apiKey: resolution.key, },
        source: SOURCE_LABELS[resolution.source],
      };
    },
  };
}

//endregion Pi auth object
