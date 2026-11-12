/**
 Qwen Token Plan Individual listing transport: the shared bounded fetch that
 turns the model endpoint into listing entries, plus the credential seam that
 resolves the route's API key exactly the way the installed llm-pi-ai adapter
 resolves an `apiKeyEnv` name (through the credentials service), answering the
 failure half as one typed error so the caller serves the checked-in snapshot
 instead of failing. @module
 */

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
} from './constants.ts';
import {
  qwenTokenPlanIndividualLogger,
} from './logging.ts';

export {
  fetchQwenTokenPlanIndividualListing,
  type QwenTokenPlanListingFetcher,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';

//region Types

/* oxlint-disable typescript/method-signature-style -- the mirrored `CredentialProvider.resolve` is a method, and method bivariance is what lets the real service satisfy this structural mirror; the property form fails assignability under strictFunctionTypes (measured: TS2379 on `index.ts` with `resolve: (ref: string) => ...`), `lint-boundaries.md` records the boundary */
/**
 Credentials service shape the key resolution reads: the one `resolve` call the
 installed llm-pi-ai adapter makes per operation. Structurally typed around the
 reference-name string, so the real `CredentialProvider` and test fakes both
 satisfy it without this package importing the branding module: `credentialRef`
 validates the reference name (throwing `TypeError` on a malformed one) and
 applies a compile-time brand that `brandString` implements as identity, so for
 the compile-time constant this seam passes, the branded call and the raw string
 resolve identically.
 */
export type QwenTokenPlanIndividualCredentialsTarget = {
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors `@deepseek-ai/dsh-credentials` `CredentialProvider.resolve(ref): Promise<ResolvedCredential | undefined>` (`lib/types/index.d.ts`), where `undefined` is the seam's own "reference unconfigured" answer and must reach the caller unchanged; `lint-boundaries.md` records why the config cannot express this mirror */
  resolve(ref: string): Promise<{
    readonly value: string;
    readonly source: string;
  } | undefined>;
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
};
/* oxlint-enable typescript/method-signature-style */

/**
 One credentials-service lookup answer: the `credentials` property carries the
 mounted service, and its absence says the composition mounts none yet. The
 optional property is this repo's absence model, so a late registration (the
 loader can start the service after this plugin's `apply`) reads the same as a
 composition that never mounts one, without a nullish union on the seam.

 @example
 ```ts
 const found: QwenTokenPlanIndividualCredentialsLookupResult = { credentials };
 ```
 */
export type QwenTokenPlanIndividualCredentialsLookupResult = {
  readonly credentials?: QwenTokenPlanIndividualCredentialsTarget;
};

/**
 Failure naming that the route's credential reference resolved no value: no
 credentials service is mounted, the reference is unconfigured, or the store
 could not answer. The mount refresh reads this as "serve the checked-in
 snapshot".
 */
export class QwenTokenPlanIndividualCredentialError extends Error {
  /**
   Build one credential-resolution failure.

   @param message - failure text naming the reference and the failing step

   @param options - caught store failure preserved as `cause`
   */
  constructor(
    message: string,
    options?: { readonly cause?: unknown }
  ) {
    super(
      message,
      options
    );
    this.name = 'QwenTokenPlanIndividualCredentialError';
  }
}

//endregion Types

//region Public API

/**
 Resolve the route's API key from the credentials service the way the
 installed llm-pi-ai adapter resolves an `apiKeyEnv` name: one `resolve` per
 operation (never cached, so a rotated credential reaches the next listing
 read), with the failure half thrown as one typed error so the caller serves
 the checked-in snapshot instead of failing.

 @param credentials - credentials service to resolve the reference through,
 named explicitly because a composition without one is one failure shape

 @returns resolved secret

 @throws QwenTokenPlanIndividualCredentialError when no service is mounted, the
 reference resolves no value, or the store cannot answer

 @example
 ```ts
 const apiKey = await resolveQwenTokenPlanIndividualApiKey({ credentials });
 ```
 */
export async function resolveQwenTokenPlanIndividualApiKey({
  credentials,
}: {
  readonly credentials?: QwenTokenPlanIndividualCredentialsTarget;
}): Promise<string> {
  /**
   Logger tagged with the function name.
   */
  const logger = tagged({
    tag: resolveQwenTokenPlanIndividualApiKey.name,
    l: qwenTokenPlanIndividualLogger,
  });
  if (credentials === undefined) {
    logger
      .debug(`no credentials service is mounted, so the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference cannot resolve`);
    throw new QwenTokenPlanIndividualCredentialError(
      `no credentials service is mounted, so the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference cannot resolve`,
    );
  }
  try {
    /**
     Resolved value for the route's credential reference; absence falls
     through the try to the typed failure below.
     */
    const hit = await credentials
      .resolve(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME);
    if (hit !== undefined) {
      logger
        .debug(`the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference resolved from source ${hit.source}`);
      return hit.value;
    }
    logger
      .debug(`the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference resolves no value`);
  } catch (error) {
    logger
      .warn(`the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference could not resolve (${caughtValueText(error)})`);
    throw new QwenTokenPlanIndividualCredentialError(
      `the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference could not resolve (${caughtValueText(error)})`,
      { cause: error },
    );
  }
  throw new QwenTokenPlanIndividualCredentialError(
    `the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference resolves no value`,
  );
}

//endregion Public API
