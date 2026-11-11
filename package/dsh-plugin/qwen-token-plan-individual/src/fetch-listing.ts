/**
 Qwen Token Plan Individual listing transport: the shared bounded fetch that
 turns the model endpoint into listing entries, plus the credential seam that
 resolves the route's API key exactly the way the installed llm-pi-ai adapter
 resolves an `apiKeyEnv` name (through the credentials service, degrading to
 "no key" when nothing resolves). @module
 */

import {
  credentialRef,
  type CredentialRef,
  type ResolvedCredential,
} from '@deepseek-ai/dsh-credentials';
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

/**
 Credentials service shape the key resolution reads: the one `resolve` call the
 installed llm-pi-ai adapter makes per operation. Structurally typed, so the
 real `CredentialProvider` and test fakes both satisfy it.
 */
export type QwenTokenPlanIndividualCredentialsTarget = {
  resolve: (ref: CredentialRef) => Promise<ResolvedCredential | undefined>;
};

//endregion Types

//region Public API

/**
 Resolve the route's API key from the credentials service the way the
 installed llm-pi-ai adapter resolves an `apiKeyEnv` name: one `resolve` per
 operation (never cached, so a rotated credential reaches the next listing
 read), with the failure half answered as absence so the caller serves the
 checked-in snapshot instead of failing.

 @param credentials - credentials service to resolve the reference through

 @returns the resolved secret, or undefined when no service or no value answers

 @example
 ```ts
 const apiKey = await resolveQwenTokenPlanIndividualApiKey({ credentials });
 ```
 */
export async function resolveQwenTokenPlanIndividualApiKey({
  credentials,
}: {
  readonly credentials: QwenTokenPlanIndividualCredentialsTarget | undefined;
}): Promise<string | undefined> {
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
    return undefined;
  }
  try {
    /**
     Resolved value for the route's credential reference, or undefined while
     the reference is unconfigured.
     */
    const hit = await credentials
      .resolve(credentialRef(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME));
    if (hit === undefined) {
      logger
        .debug(`the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference resolves no value`);
      return undefined;
    }
    logger
      .debug(`the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference resolved from source ${hit.source}`);
    return hit.value;
  } catch (error) {
    logger
      .warn(`the ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} reference could not resolve (${caughtValueText(error)})`);
    return undefined;
  }
}

//endregion Public API
