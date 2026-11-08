/**
 NeuralWatt listing transport: the one bounded fetch that turns the public
 model endpoint into listing entries. @module
 */

import {
  NEURALWATT_LISTING_TIMEOUT_MS,
  NEURALWATT_LISTING_URL,
} from './constants.ts';
import {
  isRecord,
  type NeuralWattListingEntry,
} from './listing.ts';

//region Types

/**
 Response shape the listing read needs from one HTTP call; `Response`
 satisfies it.
 */
export type NeuralWattListingResponse = {
  readonly ok: boolean;
  readonly status: number;
  json: () => Promise<unknown>;
};

/**
 HTTP call shape the listing read makes: one GET carrying an abort signal.
 */
export type NeuralWattListingFetcher = (options: {
  readonly url: string;
  readonly signal: AbortSignal;
}) => Promise<NeuralWattListingResponse>;

/**
 Listing failure naming the endpoint and the failing step, with the caught
 transport failure preserved as `cause`.
 */
export class NeuralWattListingError extends Error {
  /**
   Build one listing failure.

   @param message - failure text naming endpoint and failing step

   @param options - caught transport failure preserved as `cause`
   */
  constructor(
    message: string,
    options?: { readonly cause?: unknown }
  ) {
    super(
      message,
      options
    );
    this.name = 'NeuralWattListingError';
  }
}

//endregion Types

//region Transport

/**
 Default transport: the runtime fetch against the listing endpoint, which
 serves the catalog publicly and needs no credential.

 @param options - endpoint and abort signal for the one GET

 @returns the runtime's own response promise
 */
function defaultListingFetch(options: {
  readonly url: string;
  readonly signal: AbortSignal;
}): Promise<NeuralWattListingResponse> {
  return globalThis.fetch(
    options.url,
    { signal: options.signal }
  );
}

//endregion Transport

//region Public API

/**
 Fetch the model listing the service publishes: the live catalog read at
 mount. The abort bound keeps a stalled endpoint from holding up provider
 startup, and every failure wraps into `NeuralWattListingError` so callers
 keep one failure shape.

 @param fetcher - transport step, defaulted to the runtime fetch

 @param timeoutMs - abort bound protecting provider startup

 @returns listing entries in endpoint order, non-record members dropped

 @throws NeuralWattListingError when the endpoint is unreachable, non-OK, or listless

 @example
 ```ts
 const listing = await fetchNeuralWattListing();
 ```
 */
export async function fetchNeuralWattListing({
  fetcher = defaultListingFetch,
  timeoutMs = NEURALWATT_LISTING_TIMEOUT_MS,
}: {
  readonly fetcher?: NeuralWattListingFetcher;
  readonly timeoutMs?: number;
} = {}): Promise<readonly NeuralWattListingEntry[]> {
  try {
    /**
     Endpoint response the status gate inspects before any body read.
     */
    const response = await fetcher({
      url: NEURALWATT_LISTING_URL,
      signal: AbortSignal.timeout(timeoutMs),
    });
    if (!response.ok) {
      throw new NeuralWattListingError(
        `NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed: HTTP ${String(response.status)}`,
      );
    }
    /**
     Decoded body narrowed before any member read.
     */
    const body: unknown = await response.json();
    if ((!isRecord(body)) || (!Array.isArray(body.data))) {
      throw new NeuralWattListingError(
        `NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed: response carries no model list`,
      );
    }
    /**
     Listing entries collected in endpoint order.
     */
    const listing: NeuralWattListingEntry[] = [];
    for (const item of body.data) {
      if (isRecord(item)) {
        listing.push(item);
      }
    }
    return listing;
  } catch (error) {
    throw error instanceof NeuralWattListingError
      ? error
      : new NeuralWattListingError(
        `NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed at the transport step`,
        { cause: error },
      );
  }
}

//endregion Public API
