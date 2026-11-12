/**
 Served route profile for the NeuralWatt provider: the settings seam the route
 is written through, the skip that protects an unchanged user document, and the
 refresh that lands the live listing. @module
 */

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  NEURALWATT_ROUTE,
  NEURALWATT_SETTINGS_NAMESPACE,
} from './constants.ts';
import {
  fetchNeuralWattListing,
} from './fetch-listing.ts';
import {
  mapNeuralWattProviders,
  type NeuralWattListingEntry,
} from './listing.ts';
import {
  neuralWattLogger,
} from './logging.ts';
import {
  neuralWattListingSnapshot,
} from './snapshot.ts';

//region Types

/**
 Settings surface the route is written through: the merge write into one entry
 and the descriptor read that guards an unchanged user document. Structurally
 typed, so the real settings service and test fakes both satisfy it.
 */
export type NeuralWattSettingsTarget = {
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
 storedRouteOf({ providers: { neuralwatt: { models: [] } } });
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
    NEURALWATT_ROUTE
  );
  return route;
}

//endregion Helpers

//region Public API

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
export const NEURALWATT_SETTINGS_WRITE_DEFERRAL_MARKERS: readonly string[] = [
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
 isNeuralWattSettingsWriteDeferral({
   error: new Error('HMR transactions cannot be nested'),
 });
 ```
 */
export function isNeuralWattSettingsWriteDeferral({
  error,
}: {
  readonly error: unknown;
}): boolean {
  /**
   Rendered failure text the markers are matched against.
   */
  const text = caughtValueText(error);
  return NEURALWATT_SETTINGS_WRITE_DEFERRAL_MARKERS
    .some(function byMarker(marker) {
      return text.includes(marker);
    });
}

/**
 Write the mapping of one listing into the served settings entry, skipping the
 write when the stored route already holds the same content so an unchanged
 user document is never rewritten.

 @param settings - settings surface the route is written through

 @param listing - listing entries the written mapping is built from

 @throws When the settings entry cannot take the write, for example a profile
 that does not activate the `llm-pi-ai` entry or a home patch or command-line
 overlay that overrides its config.

 @example
 ```ts
 await writeNeuralWattRoute({ settings, listing: neuralWattListingSnapshot });
 ```
 */
export async function writeNeuralWattRoute({
  settings,
  listing,
}: {
  readonly settings: NeuralWattSettingsTarget;
  readonly listing: readonly NeuralWattListingEntry[];
}): Promise<void> {
  /**
   Logger tagged with the function name.
   */
  const logger = tagged({
    tag: writeNeuralWattRoute.name,
    l: neuralWattLogger,
  });
  /**
   Route mapping this listing earns.
   */
  const mapped = mapNeuralWattProviders({ listing });
  /**
   Form value the served entry currently holds, read back only to guard the
   user document against a content-identical rewrite.
   */
  const stored = settings
    .describe()
    .find(function byNamespace(entry) {
      return entry.ns === NEURALWATT_SETTINGS_NAMESPACE;
    })
    ?.value;
  if (sameJsonShape({
    left: storedRouteOf(stored),
    right: mapped[NEURALWATT_ROUTE],
  },)) {
    logger.debug('neuralwatt route write skipped: the stored profile already matches');
    return;
  }
  await settings.update(
    NEURALWATT_SETTINGS_NAMESPACE,
    { providers: mapped }
  );
  logger.debug('neuralwatt route written to the llm-pi-ai settings entry');
}

/**
 Land the served route at mount: write the checked-in snapshot first so the
 route stands even while the endpoint is slow, then land the live listing. A
 fetch failure or a listing without serviceable models keeps the served
 snapshot and logs why, so provider startup never fails on the network.

 @param settings - settings surface the route is written through

 @param fetchListing - listing fetch, defaulted to the real transport

 @param onWriteDeferral - deferral sink for a settings write the host refused
 while its reload transaction was still open: handed a retry that re-runs this
 whole refresh, which the caller must invoke from a context outside that
 transaction (the refusing chain keeps the transaction's context for good, so
 an in-chain retry can never land). Omitting it treats such a refusal like any
 other write failure and keeps the served snapshot.

 @example
 ```ts
 await refreshNeuralWattRoute({ settings });
 ```
 */
export async function refreshNeuralWattRoute({
  settings,
  fetchListing = fetchNeuralWattListing,
  onWriteDeferral,
}: {
  readonly settings: NeuralWattSettingsTarget;
  readonly fetchListing?: typeof fetchNeuralWattListing;
  readonly onWriteDeferral?: (retry: () => Promise<void>) => void;
}): Promise<void> {
  /**
   Logger tagged with the function name.
   */
  const logger = tagged({
    tag: refreshNeuralWattRoute.name,
    l: neuralWattLogger,
  });
  /**
   Whole-mount retry handed to the deferral sink: one refresh re-runs the
   snapshot write and the live write in their original order, so a deferred run
   can never leave the older snapshot content landing after the newer live
   listing. Each absent optional is re-passed as absent rather than as
   `undefined`, so the re-run sees the same composition shape.
   */
  function retryRefresh(): Promise<void> {
    return refreshNeuralWattRoute({
      settings,
      fetchListing,
      ...(onWriteDeferral === undefined
        ? {}
        : { onWriteDeferral, }),
    },);
  }
  try {
    await writeNeuralWattRoute({
      settings,
      listing: neuralWattListingSnapshot,
    });
  } catch (error) {
    if ((onWriteDeferral !== undefined)
      && isNeuralWattSettingsWriteDeferral({ error, },)) {
      logger
        .debug(`neuralwatt snapshot route write deferred past the host's open reload transaction (${caughtValueText(error)}); the caller re-runs the mount refresh outside it`);
      onWriteDeferral(retryRefresh);
      return;
    }
    logger
      .warn(`neuralwatt snapshot route write could not complete (${caughtValueText(error)}); the live listing write still runs. The step reads the stored profile back and then merges, so either one failing lands here: a profile that does not activate the ${NEURALWATT_SETTINGS_NAMESPACE} entry, a home patch or command-line overlay that overrides its config, or a host shutdown during mount.`);
  }
  try {
    /**
     Listing entries the live endpoint served at mount.
     */
    const listing = await fetchListing();
    try {
      /**
       Route mapping the live listing earns, built before the gate that protects
       the served snapshot from an empty catalog.
       */
      const mapped = mapNeuralWattProviders({ listing });
      /**
       Serviceable model count the live mapping produced; zero keeps the snapshot.
       */
      const modelCount = mapped[NEURALWATT_ROUTE]
        ?.models
        ?.length
        ?? 0;
      if (modelCount === 0) {
        logger
          .warn('neuralwatt listing GET mapped to no serviceable model; keeping the served snapshot');
        return;
      }
      await writeNeuralWattRoute({
        settings,
        listing
      });
    } catch (error) {
      if ((onWriteDeferral !== undefined)
        && isNeuralWattSettingsWriteDeferral({ error, },)) {
        logger
          .debug(`neuralwatt live route write deferred past the host's open reload transaction (${caughtValueText(error)}); the caller re-runs the mount refresh outside it`);
        onWriteDeferral(retryRefresh);
        return;
      }
      logger
        .warn(`neuralwatt live route write could not complete (${caughtValueText(error)}); keeping the served snapshot. The step reads the stored profile back and then merges, so either one failing lands here: a profile that does not activate the ${NEURALWATT_SETTINGS_NAMESPACE} entry, a home patch or command-line overlay that overrides its config, or a host shutdown before the mount fetch resolved.`);
    }
  } catch (error) {
    logger
      .warn(`neuralwatt listing GET failed (${caughtValueText(error)}); keeping the served snapshot`);
  }
}

//endregion Public API
