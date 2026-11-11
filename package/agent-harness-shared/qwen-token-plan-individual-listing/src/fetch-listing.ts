/**
 Qwen Token Plan individual listing transport: the one bounded fetch that
 turns the bare `GET {baseUrl}/models` listing into the entries the mapping
 reads. Every failure throws `QwenTokenPlanListingError`, so callers keep their
 checked-in snapshot instead of losing the route. @module
 */

import { QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS, } from './constants.ts';
import {
  isRecord,
  type QwenTokenPlanListingEntry,
} from './listing.ts';

//region Types

/**
 Response shape the listing read needs from one HTTP call; `Response`
 satisfies it.
 */
export type QwenTokenPlanListingResponse = {
  readonly ok: boolean;
  readonly status: number;
  json: () => Promise<unknown>;
};

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors the pinned export surface (`doc/planning/qwen-token-plan-individual-provider.md` section "Shared core export surface"), which spells `apiKey: string | undefined` so a harness passes its credential's explicit absence without fighting `exactOptionalPropertyTypes` */
/**
 HTTP call shape the listing read makes: one GET of `{baseUrl}/models`
 carrying the bearer key and an abort signal. Injectable so tests exercise the
 read without the network; `globalThis.fetch` backs the default.
 */
export type QwenTokenPlanListingFetcher = (options: {
  readonly baseUrl: string;
  readonly apiKey: string | undefined;
  readonly signal: AbortSignal;
}) => Promise<QwenTokenPlanListingResponse>;
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/**
 Listing failure naming the endpoint and the failing step, with the caught
 transport failure preserved as `cause`.
 */
export class QwenTokenPlanListingError extends Error {
  /**
   Build one listing failure.

   @param message - failure text naming endpoint and failing step

   @param options - caught transport failure preserved as `cause`
   */
  constructor(
    message: string,
    options?: { readonly cause?: unknown; },
  ) {
    super(
      message,
      options,
    );
    this.name = 'QwenTokenPlanListingError';
  }
}

//endregion Types

//region Transport

/**
 Listing URL one base URL earns: the harnesses pass their
 `/compatible-mode/v1` prefix without a trailing slash, and the route appends
 the listing path the planning doc pins (`GET {baseUrl}/models`).

  @param baseUrl - chat-completions base URL one harness configured

  @returns the listing endpoint to GET
  */
function listingUrlOf({
  baseUrl,
}: {
  readonly baseUrl: string;
}): string {
  return `${baseUrl}/models`;
}

/**
 Default transport: the runtime fetch against the listing endpoint, carrying
 the bearer key the endpoint requires and the bounded abort signal.

  @param options - base URL, credential, and abort signal for the one GET

  @returns the runtime's own response promise
  */
function defaultListingFetch(
  options: Parameters<QwenTokenPlanListingFetcher>[0],
): Promise<QwenTokenPlanListingResponse> {
  /**
   Base URL, credential, and abort signal the one GET carries.
   */
  const {
    baseUrl,
    apiKey,
    signal,
  } = options;
  return globalThis.fetch(
    listingUrlOf({ baseUrl, }),
    apiKey === undefined
      ? { signal, }
      : {
        signal,
        headers: {
          Authorization: `Bearer ${apiKey}`,
        },
      },
  );
}

//endregion Transport

//region Public API

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors the pinned export surface (`doc/planning/qwen-token-plan-individual-provider.md` section "Shared core export surface"), which spells `apiKey: string | undefined` so a harness passes its credential's explicit absence without fighting `exactOptionalPropertyTypes`; the body below carries no nullish union */
/**
 Fetch the model listing the service publishes: the live membership read at
 mount or refresh. The abort bound keeps a stalled endpoint from holding up
 provider startup, a caller signal cancels the read, and every failure wraps
 into `QwenTokenPlanListingError` so callers keep one failure shape and their
 snapshot.

  @param baseUrl - chat-completions base URL one harness configured

  @param apiKey - bearer key the endpoint requires, absent to send none

  @param fetcher - transport step, defaulted to the runtime fetch

  @param signal - caller abort, composed with the timeout bound

  @returns listing entries in endpoint order, narrowed to id and `created`

  @throws QwenTokenPlanListingError when the endpoint is unreachable, non-OK, or listless

  @example
  ```ts
  const listing = await fetchQwenTokenPlanIndividualListing({
    baseUrl: 'https://token-plan.maas.qwencloudapi.com/compatible-mode/v1',
    apiKey,
  });
  ```
  */
export async function fetchQwenTokenPlanIndividualListing({
  baseUrl,
  apiKey,
  fetcher = defaultListingFetch,
  signal,
}: {
  readonly baseUrl: string;
  readonly apiKey: string | undefined;
  readonly fetcher?: QwenTokenPlanListingFetcher;
  readonly signal?: AbortSignal;
}): Promise<readonly QwenTokenPlanListingEntry[]> {
  /**
   Listing endpoint this base URL serves membership from.
   */
  const listingUrl = listingUrlOf({ baseUrl, });
  /**
   Abort bound protecting provider startup, composed with the caller's own
   signal so either one cancels the read.
   */
  const boundedSignal = AbortSignal.any(signal === undefined
    ? [
      AbortSignal.timeout(QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS),
    ]
    : [
      AbortSignal.timeout(QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS),
      signal,
    ]);
  try {
    /**
     Endpoint response the status gate inspects before any body read.
     */
    const response = await fetcher({
      baseUrl,
      apiKey,
      signal: boundedSignal,
    });
    if (!response.ok) {
      throw new QwenTokenPlanListingError(
        `Qwen Token Plan individual listing GET ${listingUrl} failed: HTTP ${String(response.status)}`,
      );
    }
    /**
     Decoded body narrowed before any member read.
     */
    const body: unknown = await response.json();
    if ((!isRecord(body)) || (!Array.isArray(body.data))) {
      throw new QwenTokenPlanListingError(
        `Qwen Token Plan individual listing GET ${listingUrl} failed: response carries no model list`,
      );
    }
    /**
     Listing entries collected in endpoint order, narrowed to the fields the
     mapping reads: a member without a usable id drops out instead of serving
     a nameless model.
     */
    const listing: QwenTokenPlanListingEntry[] = [];
    for (const item of body.data) {
      if (!isRecord(item)) {
        continue;
      }
      /**
       Entry id, usable only as a non-empty string.
       */
      const { id, } = item;
      if (((typeof id) !== 'string') || (id.length === 0)) {
        continue;
      }
      /**
       Entry `created` stamp, kept only as an integer second count.
       */
      const { created, } = item;
      listing.push(((typeof created) === 'number') && Number.isInteger(created)
        ? {
          id,
          created,
        }
        : { id, });
    }
    return listing;
  } catch (error) {
    throw error instanceof QwenTokenPlanListingError
      ? error
      : new QwenTokenPlanListingError(
        `Qwen Token Plan individual listing GET ${listingUrl} failed at the transport step`,
        { cause: error, },
      );
  }
}
/* oxlint-enable no-restricted-syntax/no-nullish-union */

//endregion Public API
