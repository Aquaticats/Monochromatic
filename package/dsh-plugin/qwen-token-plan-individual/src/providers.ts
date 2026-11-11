/**
 Served route profile for the Qwen Token Plan Individual provider: the
 settings seam the route is written through, the skip that protects an
 unchanged user document, and the refresh that lands the live listing after
 the checked-in snapshot. @module
 */

import {
  wait,
} from '@monochromatic-dev/async-time-core/ts';
import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS,
  QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS,
  QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,
} from './constants.ts';
import {
  fetchQwenTokenPlanIndividualListing,
  QwenTokenPlanIndividualCredentialError,
  resolveQwenTokenPlanIndividualApiKey,
  type QwenTokenPlanIndividualCredentialsLookupResult,
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
 Message fragments marking one settings write as a transient failure worth
 retrying, each quoted from the tool that throws it:

 - `atomic-write: timed out waiting for the writer lock at `: `@deepseek-ai/dsh-atomic-write` `withFileLock` gave up its `DEFAULT_LOCK_WAIT_MS` wait on the profile writer lock, which a boot-time or plugin-manager writer holds across a loader reload.
 - `Configuration entry changed during reload`, `Configuration entry is no longer available`, `Configuration plugin is no longer active`: `@deepseek-ai/dsh-config-editor` `ConfigEditor.edit` refused mid-recompose; the next attempt re-reads the entry.

 Every other refusal (no `llm-pi-ai` entry, a config overridden by a home
 patch, a schema rejection) is permanent and surfaces immediately without
 burning the retry budget.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_WRITE_TRANSIENT_FAILURE_MARKERS: readonly string[] = [
  'atomic-write: timed out waiting for the writer lock at ',
  'Configuration entry changed during reload',
  'Configuration entry is no longer available',
  'Configuration plugin is no longer active',
];

/**
 Whether one failed settings write is a transient contention shape, keyed on
 the message text of the throwing tool because `@deepseek-ai/dsh-atomic-write`
 and `@deepseek-ai/dsh-config-editor` throw plain `Error`s with no class to
 discriminate on.

 @param error - caught failure from `settings.update`

 @returns whether the write earns another attempt

 @example
 ```ts
 isTransientQwenTokenPlanIndividualSettingsWriteFailure({
   error: new Error('atomic-write: timed out waiting for the writer lock at /x/package.json.lock'),
 });
 ```
 */
export function isTransientQwenTokenPlanIndividualSettingsWriteFailure({
  error,
}: {
  readonly error: unknown;
}): boolean {
  /**
   Rendered failure text the markers are matched against.
   */
  const text = caughtValueText(error);
  return QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_WRITE_TRANSIENT_FAILURE_MARKERS
    .some(function byMarker(marker) {
      return text.includes(marker);
    });
}

/**
 Message fragments marking one settings write as deferred past the host's open
 reload transaction rather than failed, each quoted from the tool that throws
 it:

 - `HMR transactions cannot be nested`: `@deepseek-ai/dsh-hmr` `runExclusive`
   refuses the `@deepseek-ai/dsh-config-editor` `ConfigEditor.edit` write while
   the calling chain already runs inside a reload transaction
   (`dsh-hmr` `lib/index.js:749`). The refusing chain keeps that transaction's
   async context for the rest of its life (2026-11-11: both mount writes of one
   refresh refused together at 2026-11-11T15:54:00Z on the live web host), so an
   in-chain retry can never land and the write must be re-rooted outside.

 The plugin-manager composes this plugin inside exactly that transaction on
 every `set_plugin`, `install_bundle`, and `set_bundle`
 (`dsh-plugin-manager` `configure` → `runExclusive`), so this is the shape a
 live re-trigger takes.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_WRITE_DEFERRAL_MARKERS: readonly string[] = [
  'HMR transactions cannot be nested',
];

/**
 Whether one failed settings write is a deferral past the host's open reload
 transaction, keyed on the message text of the throwing tool because
 `@deepseek-ai/dsh-hmr` throws a plain `Error` with no class to discriminate on.

 @param error - caught failure from `settings.update`

 @returns whether the write is owed a retry from outside that transaction

 @example
 ```ts
 isQwenTokenPlanIndividualSettingsWriteDeferral({
   error: new Error('HMR transactions cannot be nested'),
 });
 ```
 */
export function isQwenTokenPlanIndividualSettingsWriteDeferral({
  error,
}: {
  readonly error: unknown;
}): boolean {
  /**
   Rendered failure text the markers are matched against.
   */
  const text = caughtValueText(error);
  return QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_WRITE_DEFERRAL_MARKERS
    .some(function byMarker(marker) {
      return text.includes(marker);
    });
}

/**
 Look the route's credentials service up through the caller's lookup,
 re-trying within the mount-race budget. The loader can register the service
 after this plugin's `apply` runs (`@deepseek-ai/dsh-llm-pi-ai` resolves the
 same service per request instead of at mount), so an absent lookup is not
 final until the budget runs out.

 @param getCredentials - caller's lookup of the credentials service, returning
 the mounted service or an answer without it

 @param sleep - delay primitive between attempts

 @returns the lookup's final answer: the mounted service when one appeared
 within the budget, an answer without one otherwise

 @example
 ```ts
 const answer = await lookupQwenTokenPlanIndividualCredentials({
   getCredentials: function getCredentials() {
     const credentials = ctx.get('credentials');
     return credentials === undefined ? {} : { credentials, };
   },
   sleep: wait,
 });
 ```
 */
async function lookupQwenTokenPlanIndividualCredentials({
  getCredentials,
  sleep,
}: {
  readonly getCredentials: () => QwenTokenPlanIndividualCredentialsLookupResult;
  readonly sleep: (ms: number) => Promise<unknown>;
}): Promise<QwenTokenPlanIndividualCredentialsLookupResult> {
  for (
    let attempt = 1;
    attempt <= QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS;
    attempt += 1
  ) {
    /**
     Credentials answer this attempt finds; a mounted service ends the wait
     early.
     */
    const answer = getCredentials();
    if (answer.credentials !== undefined) return answer;
    if (attempt < QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- one lookup at a time is the design: an answer only exists after the previous attempt saw none, so the wait is what lets a late registration land instead of every attempt repeating an unchanged lookup at once.
      await sleep(QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS);
    }
  }
  return {};
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
 beside it keep standing untouched. A transient contention failure earns
 retries within the mount-race budget, because the plugin-manager holds the
 same `package.json` writer lock across a long reload at boot.

 @param settings - settings surface the route is written through

 @param listing - listing entries the written mapping is built from

 @param sleep - delay between retry attempts, injected so tests reach the
 retry branch without waiting out the budget

 @throws When the settings entry cannot take the write after the mount-race
 budget: a permanent refusal, for example a profile that does not activate the
 `llm-pi-ai` entry or a home patch or command-line overlay that overrides its
 config, throws without retrying.

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
  sleep = wait,
}: {
  readonly settings: QwenTokenPlanIndividualSettingsTarget;
  readonly listing: readonly QwenTokenPlanListingEntry[];
  readonly sleep?: (ms: number) => Promise<unknown>;
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
  for (
    let attempt = 1;
    attempt <= QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS;
    attempt += 1
  ) {
    try {
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
      // oxlint-disable-next-line eslint/no-await-in-loop -- one write at a time is the design: the retry exists because a competing writer holds the same `package.json` lock across the attempt, so issuing the next write before this one settles would only queue behind the same hold instead of landing after it.
      await settings.update(
        QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,
        { providers: mapped }
      );
      logger
        .debug('qwen-token-plan-individual route written to the llm-pi-ai settings entry');
      return;
    } catch (error) {
      /**
       Whether this attempt still has retry budget behind it.
       */
      const hasRetryBudget = attempt < QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS;
      if (!hasRetryBudget) {
        throw error;
      }
      if (!isTransientQwenTokenPlanIndividualSettingsWriteFailure({ error },)) {
        throw error;
      }
      logger
        .debug(`qwen-token-plan-individual route write attempt ${attempt} of ${QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS} failed transiently (${caughtValueText(error)}); retrying in ${QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS} ms`);
      // oxlint-disable-next-line eslint/no-await-in-loop -- the pause between attempts is the design: the next attempt is worth making only after the previous one failed and the hold it measured has had time to release, so firing the attempts together would measure nothing.
      await sleep(QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS);
    }
  }
  /* v8 ignore next 2 -- @preserve: every iteration returns or throws, so the budget ending is unreachable */
  throw new Error('qwen-token-plan-individual route write ended without a write or a throw');
}

/**
 Land the served route at mount: write the checked-in snapshot first so the
 route stands even while the endpoint is slow, then resolve the route's
 credential and land the live listing. No resolving credential, a fetch
 failure, or a listing without serviceable models keeps the served snapshot
 and logs why, so provider startup never fails on the network or on
 credentials.

 @param settings - settings surface the route is written through

 @param getCredentials - lookup of the credentials service, consulted when
 the listing fetch runs instead of at plugin apply: the loader can register the
 service after this plugin's `apply`, and the shipped adapter resolves the
 same service per request the same lazy way. Omitting the lookup stands for a
 composition that mounts no credentials service and keeps the snapshot.

 @param fetchListing - listing fetch, defaulted to the real transport

 @param sleep - delay between retry attempts, forwarded to the route writes
 and the credentials lookup

 @param onWriteDeferral - deferral sink for a settings write the host refused
 while its reload transaction was still open: handed a retry that re-runs this
 whole refresh, which the caller must invoke from a context outside that
 transaction (the refusing chain keeps the transaction's context for good, so
 an in-chain retry can never land). Omitting it treats such a refusal like any
 other write failure and keeps the served snapshot.

 @example
 ```ts
 await refreshQwenTokenPlanIndividualRoute({
   settings,
   getCredentials: function getCredentials() {
     const credentials = ctx.get('credentials');
     return credentials === undefined ? {} : { credentials, };
   },
 });
 ```
 */
export async function refreshQwenTokenPlanIndividualRoute({
  settings,
  getCredentials,
  fetchListing = fetchQwenTokenPlanIndividualListing,
  sleep = wait,
  onWriteDeferral,
}: {
  readonly settings: QwenTokenPlanIndividualSettingsTarget;
  readonly getCredentials?: () => QwenTokenPlanIndividualCredentialsLookupResult;
  readonly fetchListing?: typeof fetchQwenTokenPlanIndividualListing;
  readonly sleep?: (ms: number) => Promise<unknown>;
  readonly onWriteDeferral?: (retry: () => Promise<void>) => void;
}): Promise<void> {
  /**
   Logger tagged with the function name.
   */
  const logger = tagged({
    tag: refreshQwenTokenPlanIndividualRoute.name,
    l: qwenTokenPlanIndividualLogger,
  });
  /**
   Whole-mount retry handed to the deferral sink: one refresh re-runs the
   snapshot write, the listing fetch, and the live write in their original
   order, so a deferred run can never leave the older snapshot content landing
   after the newer live listing. Each absent optional is re-passed as absent
   rather than as `undefined`, so the re-run sees the same composition shape.
   */
  function retryRefresh(): Promise<void> {
    return refreshQwenTokenPlanIndividualRoute({
      settings,
      fetchListing,
      sleep,
      ...(getCredentials === undefined
        ? {}
        : { getCredentials, }),
      ...(onWriteDeferral === undefined
        ? {}
        : { onWriteDeferral, }),
    },);
  }
  try {
    await writeQwenTokenPlanIndividualRoute({
      settings,
      listing: qwenTokenPlanIndividualListingSnapshot.entries,
      sleep,
    });
  } catch (error) {
    if ((onWriteDeferral !== undefined) && isQwenTokenPlanIndividualSettingsWriteDeferral({ error },)) {
      logger
        .debug(`qwen-token-plan-individual snapshot route write deferred past the host's open reload transaction (${caughtValueText(error)}); the caller re-runs the mount refresh outside it`);
      onWriteDeferral(retryRefresh);
      return;
    }
    logger
      .warn(`qwen-token-plan-individual snapshot route write could not complete (${caughtValueText(error)}); the live listing write still runs. The step reads the stored profile back and then merges, so either one failing lands here: a profile that does not activate the ${QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE} entry, a home patch or command-line overlay that overrides its config, or a host shutdown during mount.`);
  }
  try {
    /**
     Credentials presence the listing fetch resolves its key from: looked up
     now rather than at apply so a service the loader registers late is still
     seen, with an absent `credentials` property degrading to the served
     snapshot through the resolution's own typed failure.
     */
    const presence = getCredentials === undefined
      ? {}
      : await lookupQwenTokenPlanIndividualCredentials({
        getCredentials,
        sleep,
      });
    /**
     API key the route's credential reference resolves to; the typed failure
     its absence throws lands in the outer catch below.
     */
    const apiKey = await resolveQwenTokenPlanIndividualApiKey(presence);
    try {
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
          listing,
          sleep,
        });
      } catch (error) {
        if ((onWriteDeferral !== undefined) && isQwenTokenPlanIndividualSettingsWriteDeferral({ error },)) {
          logger
            .debug(`qwen-token-plan-individual live route write deferred past the host's open reload transaction (${caughtValueText(error)}); the caller re-runs the mount refresh outside it`);
          onWriteDeferral(retryRefresh);
          return;
        }
        logger
          .warn(`qwen-token-plan-individual live route write could not complete (${caughtValueText(error)}); keeping the served snapshot. The step reads the stored profile back and then merges, so either one failing lands here: a profile that does not activate the ${QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE} entry, a home patch or command-line overlay that overrides its config, or a host shutdown before the mount fetch resolved.`);
      }
    } catch (error) {
      logger
        .warn(`qwen-token-plan-individual listing GET failed (${caughtValueText(error)}); keeping the served snapshot`);
    }
  } catch (error) {
    if (error instanceof QwenTokenPlanIndividualCredentialError) {
      logger
        .warn(`${caughtValueText(error)}; keeping the served snapshot`);
      return;
    }
    /* v8 ignore next 2 -- @preserve: the resolution seam wraps every failure into QwenTokenPlanIndividualCredentialError, so an unexpected value here is a defect the caller must see */
    throw error;
  }
}

//endregion Public API
