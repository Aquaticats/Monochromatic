/**
 Verifies built pi-statusline extension registration and footer warning behavior.
 
 @module
 */

import type { ExtensionFactory, } from '@earendil-works/pi-coding-agent';
import {
  createAfterProviderResponseEvent,
  createExtensionContext,
  fakePiApi,
  getAfterProviderResponseHandler,
} from './pi-api-fixture.ts';

//region Constants

/**
 Built extension path consumed by Pi.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Expected provider response registration from the extension entry point.
 */
const EXPECTED_REGISTRATION = 'event:after_provider_response';

/**
 Status key owned by built extension.
 */
const EXPECTED_STATUS_KEY = 'pi-statusline.usage';

/**
 Verification clock used for deterministic reset formatting.
 */
const NOW_MS = Date.parse('2026-06-01T12:00:00Z',);

/**
 Milliseconds in forty seconds for projected-overflow fixture.
 */
const FORTY_SECONDS_MS = 40_000;

//endregion Constants

//region Types

/**
 Built pi-statusline extension module shape.
 */
type StatuslineExtensionModule = {
  /**
   Pi extension factory.
   */
  readonly default: ExtensionFactory;
};

/**
 Date.now restore handle.
 */
type DateNowRestore = {
  /**
   Restores original Date.now implementation.
   */
  readonly [Symbol.dispose]: () => void;
};

//endregion Types

//region Helpers

/**
 Detects built pi-statusline extension module shape.
 
 @param value - imported module namespace
 
 @returns whether module exports extension factory
 
 @example
 ```ts
 isStatuslineExtensionModule(await import('../dist/final/node/index.mjs'));
 ```
 */
function isStatuslineExtensionModule(
  value: unknown,
): value is StatuslineExtensionModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value) && ((typeof value.default) === 'function');
}

/**
 Freezes Date.now for deterministic built-extension verification.
 
 @param nowMs - timestamp returned by Date.now
 
 @returns restore handle used by `using`
 
 @example
 ```ts
 using frozen = freezeDateNow(Date.now());
 ```
 */
function freezeDateNow(nowMs: number,): DateNowRestore {
  /**
   Original Date.now implementation restored after verification.
   */
  const originalDateNow = Date.now;
  Date.now = function now(): number {
    return nowMs;
  };

  return {
    [Symbol.dispose]: function restoreDateNow(): void {
      Date.now = originalDateNow;
    },
  };
}

/**
 Creates Anthropic token rate-limit headers for verification.
 
 @returns deterministic projected-overflow token header group
 
 @example
 ```ts
 const headers = verificationHeaders();
 ```
 */
function verificationHeaders(): Record<string, string> {
  return {
    'anthropic-ratelimit-tokens-limit': '100',
    'anthropic-ratelimit-tokens-remaining': '60',
    'anthropic-ratelimit-tokens-reset': new Date(NOW_MS + FORTY_SECONDS_MS,).toISOString(),
  };
}

//endregion Helpers

//region Verification

/**
 Verifies built {@link StatuslineExtensionModule} registers and renders usage warning status.
 
 @returns verification result text
 
 @throws when built extension import, registration, or handler output fails
 
 @example
 ```ts
 console.log(await verifyBuiltExtension());
 ```
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   Date.now restore handle scoped to this verification run.
   */
  using frozenDateNow = freezeDateNow(NOW_MS,);
  void frozenDateNow;

  /**
   Built extension module imported through package output.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isStatuslineExtensionModule(mod,)) {
    throw new Error(
      'built pi-statusline extension does not export a default extension factory',
    );
  }

  /**
   Fake Pi API harness for registration verification.
   */
  const harness = fakePiApi();
  await mod.default(harness.api,);

  if (!harness.registrations
    .includes(EXPECTED_REGISTRATION,)) {
    throw new Error(
      `missing pi-statusline registration: ${EXPECTED_REGISTRATION}`,
    );
  }

  /**
   Captured provider response handler from built extension.
   */
  const handler = getAfterProviderResponseHandler(
    harness.afterProviderResponseHandlers,
  );
  /**
   Fake context capturing footer status writes.
   */
  const {
    ctx,
    statuses,
  } = createExtensionContext();

  await handler(
    createAfterProviderResponseEvent(verificationHeaders(),),
    ctx,
  );

  /**
   Captured footer status text.
   */
  const status = statuses.get(EXPECTED_STATUS_KEY,);
  if (status !== 'anthropic tokens <error>60% left →120%</error> (40s)') {
    throw new Error(
      `unexpected pi-statusline status: ${String(status,)}`,
    );
  }

  return 'pi-statusline extension verified: event:after_provider_response, anthropic tokens 60% left →120% (40s)';
}

//endregion Verification

console.log(await verifyBuiltExtension(),);
