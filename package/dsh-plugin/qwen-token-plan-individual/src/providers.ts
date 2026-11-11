/**
 Served route profile for the Qwen Token Plan Individual provider: the
 settings seam the route is written through, the skip that protects an
 unchanged user document, and the refresh that lands the live listing after
 the checked-in snapshot. @module
 */

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
  QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,
} from './constants.ts';
import {
  fetchQwenTokenPlanIndividualListing,
  resolveQwenTokenPlanIndividualApiKey,
  type QwenTokenPlanIndividualCredentialsTarget,
} from './fetch-listing.ts';
import {
  mapQwenTokenPlanIndividualProviders,
  type QwenTokenPlanListingEntry,
} from './listing.ts';
import {
  qwenTokenPlanIndividualLogger,
} from './logging.ts';
import {
  qwenTokenPlanIndividualListingSnapshot,
} from './snapshot.ts';

//region Types

/**
 Settings surface the route is written through: the merge write into one entry
 and the descriptor read that guards an unchanged user document. Structurally
 typed, so the real settings service and test fakes both satisfy it.
 */
export type QwenTokenPlanIndividualSettingsTarget = {
  update: (
    ns: string,
    patch: Record<string, unknown>
  ) => Promise<void>;
  describe: () => readonly {
    readonly ns: string;
    readonly value: unknown
  }[];
};

//endregion Types

//region Helpers

/**
 Structural equality over JSON-shaped values. The route write is skipped only
 on a true content match, and YAML round-trips may reorder keys, so the walk
 compares key sets instead of serialized text.

 @param left - stored-side value the skip decision treats as already present

 @param right - freshly mapped value the skip decision would otherwise write

 @returns whether both candidates hold the same JSON-shaped content

 @example
 ```ts
 sameJsonShape({ left: { id: 'a', models: [] }, right: { models: [], id: 'a' } });
 ```
 */
function sameJsonShape({
  left,
  right,
}: {
  readonly left: unknown;
  readonly right: unknown;
}): boolean {
  if (Object.is(
    left,
    right
  )) return true;
  if (Array.isArray(left) || Array.isArray(right)) {
    /**
     Right candidate narrowed to an array shape for element-wise comparison,
     captured before the callback because narrowing does not reach closures.
     */
    const rightItems: readonly unknown[] = Array.isArray(right) ? right : [];
    return Array.isArray(left)
      && Array.isArray(right)
      && (left.length === rightItems.length)
      && left.every(function byElement(
        item,
        index
      ) {
        return sameJsonShape({
          left: item,
          right: rightItems[index]
        });
      });
  }
  if (
    ((typeof left) !== 'object')
      || (left === null)
      || ((typeof right) !== 'object')
      || (right === null)
  ) return false;
  /**
   Left candidate narrowed to an object shape for field-wise comparison,
   captured before the callback because narrowing does not reach closures.
   */
  const leftObject: object = left;
  /**
   Right candidate narrowed to an object shape for field-wise comparison,
   captured before the callback because narrowing does not reach closures.
   */
  const rightObject: object = right;
  /**
   Own keys of the left candidate; the right candidate must carry exactly these.
   */
  const leftKeys = Object.keys(leftObject);
  return (leftKeys.length
    === Object.keys(rightObject)
      .length)
    && leftKeys.every(function byKey(key) {
      /**
       Stored field under one shared own key, compared field-wise.
       */
      const leftField: unknown = Reflect.get(
        leftObject,
        key
      );
      /**
       Freshly mapped field under the same shared own key.
       */
      const rightField: unknown = Reflect.get(
        rightObject,
        key
      );
      return Object.hasOwn(
        rightObject,
        key
      )
        && sameJsonShape({
          left: leftField,
          right: rightField
        });
    });
}

/**
 Stored route profile inside a settings form value. Any shape other than the
 expected `providers` mapping counts as absent, so an unreadable stored value
 can only cause the route write to run again.

 @param value - settings form value read back from `describe`

 @returns stored route profile, or undefined when the form carries none

 @example
 ```ts
 storedRouteOf({ providers: { 'qwen-token-plan-individual': { models: [] } } });
 ```
 */
function storedRouteOf(value: unknown): unknown {
  if (((typeof value) !== 'object') || (value === null)) return undefined;
  /**
   Provider dict the stored form carries, before the per-route lookup.
   */
  const providers: unknown = Reflect.get(
    value,
    'providers'
  );
  if (((typeof providers) !== 'object') || (providers === null)) return undefined;
  /**
   Stored route profile under the served route key.
   */
  const route: unknown = Reflect.get(
    providers,
    QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE
  );
  return route;
}

//endregion Helpers

//region Public API

/**
 Write the mapping of one listing into the served settings entry, skipping the
 write when the stored route already holds the same content so an unchanged
 user document is never rewritten. The patch carries the served route key
 alone: the loader merges `providers` per provider key, so hand-declared routes
 beside it keep standing untouched.

 @param settings - settings surface the route is written through

 @param listing - listing entries the written mapping is built from

 @throws When the settings entry cannot take the write, for example a profile
 that does not activate the `llm-pi-ai` entry or a home patch or command-line
 overlay that overrides its config.

 @example
 ```ts
 await writeQwenTokenPlanIndividualRoute({
   settings,
   listing: qwenTokenPlanIndividualListingSnapshot.entries,
 });
 ```
 */
export async function writeQwenTokenPlanIndividualRoute({
  settings,
  listing,
}: {
  readonly settings: QwenTokenPlanIndividualSettingsTarget;
  readonly listing: readonly QwenTokenPlanListingEntry[];
}): Promise<void> {
  /**
   Logger tagged with the function name.
   */
  const logger = tagged({
    tag: writeQwenTokenPlanIndividualRoute.name,
    l: qwenTokenPlanIndividualLogger,
  });
  /**
   Route mapping this listing earns.
   */
  const mapped = mapQwenTokenPlanIndividualProviders({
    listing,
  });
  /**
   Form value the served entry currently holds, read back only to guard the
   user document against a content-identical rewrite.
   */
  const stored = settings
    .describe()
    .find(function byNamespace(entry) {
      return entry.ns === QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE;
    })
    ?.value;
  if (sameJsonShape({
    left: storedRouteOf(stored),
    right: mapped[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE],
  },)) {
    logger
      .debug('qwen-token-plan-individual route write skipped: the stored profile already matches');
    return;
  }
  await settings.update(
    QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,
    { providers: mapped }
  );
  logger
    .debug('qwen-token-plan-individual route written to the llm-pi-ai settings entry');
}

/**
 Land the served route at mount: write the checked-in snapshot first so the
 route stands even while the endpoint is slow, then resolve the route's
 credential and land the live listing. No resolving credential, a fetch
 failure, or a listing without serviceable models keeps the served snapshot
 and logs why, so provider startup never fails on the network or on
 credentials.

 @param settings - settings surface the route is written through

 @param credentials - credentials service the route's API key resolves through,
 named explicitly because a composition without one keeps the snapshot

 @param fetchListing - listing fetch, defaulted to the real transport

 @example
 ```ts
 await refreshQwenTokenPlanIndividualRoute({ settings, credentials });
 ```
 */
export async function refreshQwenTokenPlanIndividualRoute({
  settings,
  credentials,
  fetchListing = fetchQwenTokenPlanIndividualListing,
}: {
  readonly settings: QwenTokenPlanIndividualSettingsTarget;
  readonly credentials: QwenTokenPlanIndividualCredentialsTarget | undefined;
  readonly fetchListing?: typeof fetchQwenTokenPlanIndividualListing;
}): Promise<void> {
  /**
   Logger tagged with the function name.
   */
  const logger = tagged({
    tag: refreshQwenTokenPlanIndividualRoute.name,
    l: qwenTokenPlanIndividualLogger,
  });
  try {
    await writeQwenTokenPlanIndividualRoute({
      settings,
      listing: qwenTokenPlanIndividualListingSnapshot.entries,
    });
  } catch (error) {
    logger
      .warn(`qwen-token-plan-individual snapshot route write could not complete (${caughtValueText(error)}); the live listing write still runs. The step reads the stored profile back and then merges, so either one failing lands here: a profile that does not activate the ${QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE} entry, a home patch or command-line overlay that overrides its config, or a host shutdown during mount.`);
  }
  try {
    /**
     API key the route's credential reference resolves to, or undefined when
     no credential answers; that absence keeps the served snapshot.
     */
    const apiKey = await resolveQwenTokenPlanIndividualApiKey({
      credentials,
    });
    if (apiKey === undefined) {
      logger
        .warn(`no ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME} credential resolves; keeping the served snapshot`);
      return;
    }
    /**
     Listing entries the live endpoint served at mount.
     */
    const listing = await fetchListing({
      baseUrl: QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
      apiKey,
      signal: AbortSignal.timeout(QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS),
    });
    try {
      /**
       Route mapping the live listing earns, built before the gate that protects
       the served snapshot from an empty catalog.
       */
      const mapped = mapQwenTokenPlanIndividualProviders({
        listing,
      });
      /**
       Serviceable model count the live mapping produced; zero keeps the snapshot.
       */
      const modelCount = mapped[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]
        ?.models
        ?.length
        ?? 0;
      if (modelCount === 0) {
        logger
          .warn('qwen-token-plan-individual listing GET mapped to no serviceable model; keeping the served snapshot');
        return;
      }
      await writeQwenTokenPlanIndividualRoute({
        settings,
        listing
      });
    } catch (error) {
      logger
        .warn(`qwen-token-plan-individual live route write could not complete (${caughtValueText(error)}); keeping the served snapshot. The step reads the stored profile back and then merges, so either one failing lands here: a profile that does not activate the ${QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE} entry, a home patch or command-line overlay that overrides its config, or a host shutdown before the mount fetch resolved.`);
    }
  } catch (error) {
    logger
      .warn(`qwen-token-plan-individual listing GET failed (${caughtValueText(error)}); keeping the served snapshot`);
  }
}

//endregion Public API
