/**
 Source-dependent readiness for a transport adapter that resolves original auth later. @module
 */
import type {
  ApiKeyAuth,
  AuthResult,
  Model,
  Api,
  ProviderAuth,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Readiness without adapter credentials

/**
 Module logger records readiness without credentials or provider payloads.
 */
const moduleLogger = tagged({ tag: 'pi-plugin-openai-fast.keyless-auth', },);

/**
 Leave request authentication entirely at the original registry dispatch.

 @returns empty request authentication with explicit routing provenance
 */
function resolve(): Promise<AuthResult> {
  return Promise.resolve({
    auth: {},
    source: 'routes-to-original-provider',
  },);
}

/**
 Gate selectable companions on original-provider availability without inheriting its credentials.

 @param isConfigured - fresh native source-only availability capability

 @returns keyless auth descriptor without a separate login or token resolution

 @mutates isConfigured - readiness checks invoke native availability handling

 @example
 ```ts
 const auth = createKeylessAuth(checkNativeAvailability);
 ```
 */
export function createKeylessAuth(
  isConfigured: (signal: ForeignBorrowed<AbortSignal>) => Promise<boolean>,
): ProviderAuth {
  /**
   Factory logger preserves the readiness capability boundary.
   */
  const l = tagged({
    tag: createKeylessAuth.name,
    l: moduleLogger,
  },);
  l.debug('creating source-dependent priority readiness',);

  /**
   Recheck the original provider instead of caching a previous login state.

   @param signal - cancellation authority supplied by native availability handling

   @returns adapter readiness only when original chat models are available
   */
  async function check({ signal, }: { readonly signal: ForeignBorrowed<AbortSignal>; },): ReturnType<NonNullable<ApiKeyAuth['check']>> {
    /**
     Readiness logger records only the configuration decision.
     */
    const inner = tagged({
      tag: check.name,
      l,
    },);
    signal.throwIfAborted();
    if (!await isConfigured(signal,)) {
      inner.debug('original provider is unavailable; hiding priority companions',);
      return undefined;
    }
    inner.debug('original provider is available; exposing priority companions',);
    return {
      type: 'api_key',
      source: 'routes-to-original-provider',
    };
  }

  return Object.freeze({
    apiKey: {
      name: 'Routes to existing OpenAI authentication',
      check,
      resolve,
    },
  },);
}

/**
 Exclude local physical routing targets from normal available-model lists.

 @returns empty physical-target availability list

 @example
 ```ts
 const targets = noAvailableTargets();
 ```
 */
export function noAvailableTargets(): readonly Model<Api>[] {
  return [];
}

//endregion
