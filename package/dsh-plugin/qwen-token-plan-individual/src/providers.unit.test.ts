import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isTransientQwenTokenPlanIndividualSettingsWriteFailure,
  mapQwenTokenPlanIndividualProviders,
  qwenTokenPlanIndividualListingSnapshot,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS,
  QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,
  refreshQwenTokenPlanIndividualRoute,
  type QwenTokenPlanIndividualCredentialsLookupResult,
  type QwenTokenPlanIndividualCredentialsTarget,
  type QwenTokenPlanIndividualSettingsTarget,
  type QwenTokenPlanListingEntry,
  writeQwenTokenPlanIndividualRoute,
} from '../dist/final/node/index.mjs';
import {
  MEASURED_MEDIA_IDS,
  MEASURED_SERVICEABLE_IDS,
} from './measured-listing-fixtures.ts';

//region Test fixtures

/**
 Exact contention text `@deepseek-ai/dsh-atomic-write` `withFileLock` throws
 when its `DEFAULT_LOCK_WAIT_MS` wait on the profile writer lock expires
 (`dsh-atomic-write` `lib/index.js`), path suffix included. Quoted here rather
 than derived from the plugin's markers so marker drift fails this test.
 */
const UPSTREAM_WRITER_LOCK_TIMEOUT_MESSAGE = 'atomic-write: timed out waiting for the writer lock at /fixture/profile/package.json.lock';

/**
 Exact text `@deepseek-ai/dsh-config-editor` `ConfigEditor.edit` throws when
 the entry registry recomposed under the edit (`dsh-config-editor`
 `lib/index.js`).
 */
const UPSTREAM_ENTRY_CHANGED_MESSAGE = 'Configuration entry changed during reload';

/**
 Exact text `@deepseek-ai/dsh-config-editor` `ConfigEditor.edit` throws when
 the entry vanished under the edit (`dsh-config-editor` `lib/index.js`).
 */
const UPSTREAM_ENTRY_UNAVAILABLE_MESSAGE = 'Configuration entry is no longer available';

/**
 Exact text `@deepseek-ai/dsh-config-editor` `ConfigEditor.edit` throws when
 the plugin fiber stopped under the edit (`dsh-config-editor` `lib/index.js`).
 */
const UPSTREAM_PLUGIN_INACTIVE_MESSAGE = 'Configuration plugin is no longer active';

/**
 Exact refusal `@deepseek-ai/dsh-settings` throws when the profile mounts no
 such entry: a permanent shape the retry must not burn budget on.
 */
const UPSTREAM_MISSING_ENTRY_MESSAGE = 'No configurable plugin entry "llm-pi-ai"';

/**
 One recorded settings merge write.
 */
type RecordedWrite = {
  ns: string;
  patch: Record<string, unknown>;
};

/**
 Recording fake of the settings seam: serves a fixed form value through
 `describe`, records every accepted merge write, and can refuse writes at the
 given zero-based call indices to reach the failure branches. Refusals throw
 `refuseWith`, so a test decides between a permanent refusal and the exact
 contention text an upstream tool throws.

 @param options - stored form value, refusing call indices, and the failure
 those refusals throw

 @returns fake target plus the writes it accepted and every call it saw

 @example
 ```ts
 const fake = fakeSettingsOf({ stored: { providers: {} } });
 ```
 */
function fakeSettingsOf({
  stored,
  failWrites = [],
  refuseWith = new Error('settings entry unavailable'),
}: {
  readonly stored?: unknown;
  readonly failWrites?: readonly number[];
  readonly refuseWith?: Error;
} = {}): {
  readonly target: QwenTokenPlanIndividualSettingsTarget;
  readonly writes: RecordedWrite[];
  readonly attempts: RecordedWrite[];
} {
  /**
   Writes the fake accepted.
   */
  const writes: RecordedWrite[] = [];
  /**
   Every update call the fake received, including the refused ones: refusal
   indexing counts attempts, so a refused call still advances the counter.
   */
  const attempts: RecordedWrite[] = [];
  return {
    target: {
      describe: () => (stored === undefined
        ? []
        : [{ ns: QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE, value: stored }]),
      update: async (ns, patch) => {
        /**
         Position of this call among every attempt, refused or accepted.
         */
        const attempt = attempts.length;
        attempts.push({ ns, patch });
        if (failWrites.includes(attempt)) throw refuseWith;
        writes.push({ ns, patch });
      },
    },
    writes,
    attempts,
  };
}

/**
 Delay primitive stand-in recording every requested delay, so a test reaches
 the retry branches without waiting out the mount-race budget in real time.

 @returns recorded sleep plus the delays it was asked for

 @example
 ```ts
 const sleeper = recordingSleepOf();
 await writeQwenTokenPlanIndividualRoute({ settings, listing, sleep: sleeper.sleep });
 ```
 */
function recordingSleepOf(): {
  readonly sleep: (ms: number) => Promise<undefined>;
  readonly delays: number[];
} {
  /**
   Delays the sleep was asked to wait.
   */
  const delays: number[] = [];
  return {
    sleep: async function sleep(ms: number): Promise<undefined> {
      delays.push(ms);
      return undefined;
    },
    delays,
  };
}

/**
 The one accepted write a test's premise promises, or a loud failure when no
 write landed.

 @param writes - writes the fake accepted

 @returns the first accepted write

 @example
 ```ts
 firstWriteOf(fake.writes);
 ```
 */
function firstWriteOf(writes: readonly RecordedWrite[]): RecordedWrite {
  /**
   First accepted write, before the premise check.
   */
  const [first] = writes;
  if (first === undefined) {
    throw new Error('expected one accepted settings write');
  }
  return first;
}

/**
 Provider dict one write's patch carried under its `providers` key: the shape
 the loader merges per provider key.

 @param write - recorded write to read

 @returns provider dict the patch carried

 @example
 ```ts
 writtenProvidersOf(firstWriteOf(fake.writes));
 ```
 */
function writtenProvidersOf(write: RecordedWrite): Record<string, {
  models?: readonly { id: string }[];
}> {
  return (write.patch.providers ?? {}) as Record<string, {
    models?: readonly { id: string }[];
  }>;
}

/**
 Credentials fake resolving one fixed value, or the configured failure, so
 every credential branch of the refresh is reachable.

 @param options - resolved value, or the failure to reject with

 @returns fake credentials target plus the references it was asked about

 @example
 ```ts
 const credentials = fakeCredentialsOf({ value: 'test-key' });
 ```
 */
function fakeCredentialsOf({
  value,
  rejectWith,
}: {
  readonly value?: string;
  readonly rejectWith?: Error;
} = {}): {
  readonly target: QwenTokenPlanIndividualCredentialsTarget;
  readonly askedAbout: string[];
} {
  /**
   Reference names the fake was asked to resolve.
   */
  const askedAbout: string[] = [];
  return {
    target: {
      resolve: async (ref) => {
        askedAbout.push(ref);
        if (rejectWith !== undefined) throw rejectWith;
        return value === undefined
          ? undefined
          : { value, source: 'test', };
      },
    },
    askedAbout,
  };
}

/**
 Route mapping the checked-in snapshot earns: the exact patch content a
 snapshot write must carry.

 @returns provider dict for the snapshot listing

 @example
 ```ts
 expectedSnapshotProviders();
 ```
 */
function expectedSnapshotProviders(): Record<string, unknown> {
  return mapQwenTokenPlanIndividualProviders({
    listing: qwenTokenPlanIndividualListingSnapshot.entries,
  },);
}

//endregion Test fixtures

await describe({
  name: '',
  children: [
    //region Route write

    describe({
      name: writeQwenTokenPlanIndividualRoute.name,
      children: [
        it({
          name: 'writes the mapped route into the llm-pi-ai settings entry',
          fn: async () => {
            const fake = fakeSettingsOf();
            await writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.ns,)
              .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapQwenTokenPlanIndividualProviders({
                  listing: [{ id: 'seed', },],
                },),
              },);
          },
        },),
        it({
          name: 'skips the write when the stored route already holds the same content regardless of key order',
          fn: async () => {
            /**
             Route profile whose object keys are reversed, standing in for the
             key order a YAML round-trip can produce.
             */
            const reorderedProfile = Object.fromEntries(Object.entries(mapQwenTokenPlanIndividualProviders({
              listing: [{ id: 'seed', },],
            },)[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE] ?? {},)
              .toReversed(),);
            const fake = fakeSettingsOf({
              stored: {
                providers: { [QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]: reorderedProfile, },
              },
            },);
            await writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
            },);
            expect(fake.writes.length,)
              .toBe(0,);
          },
        },),
        it({
          name: 'writes when the stored route holds different content',
          fn: async () => {
            const fake = fakeSettingsOf({
              stored: {
                providers: mapQwenTokenPlanIndividualProviders({ listing: [{ id: 'older', },], },),
              },
            },);
            await writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'writes when the entry carries no stored form value',
          fn: async () => {
            const fake = fakeSettingsOf();
            await writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'writes only the served route key beside hand-declared routes, leaving them to the loader merge',
          fn: async () => {
            /**
             A hand-declared sibling route, as the profile document carries
             one beside the block this plugin replaces.
             */
            const HAND_DECLARED_ROUTE_KEY = 'opencode-go';
            const fake = fakeSettingsOf({
              stored: {
                providers: {
                  [HAND_DECLARED_ROUTE_KEY]: {
                    api: 'openai-completions',
                    baseURL: 'https://opencode.example/v1',
                  },
                  [QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]: {
                    models: [{ id: 'older', }],
                  },
                },
              },
            },);
            await writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            /**
             Provider keys the accepted patch carries; the loader merges per
             key, so carrying only the served key leaves every sibling standing.
             */
            const acceptedWrite = firstWriteOf(fake.writes,);
            const writtenProviderKeys = Object.keys(writtenProvidersOf(acceptedWrite,));
            expect(writtenProviderKeys).toEqual([QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,]);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapQwenTokenPlanIndividualProviders({
                  listing: [{ id: 'seed', },],
                },),
              },);
          },
        },),
        it({
          name: 'retries a writer-lock timeout and lands the write',
          fn: async () => {
            /**
             Sleep the retry loop used, recorded instead of waited out.
             */
            const sleeper = recordingSleepOf();
            const fake = fakeSettingsOf({
              failWrites: [0,],
              refuseWith: new Error(UPSTREAM_WRITER_LOCK_TIMEOUT_MESSAGE),
            },);
            await writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
              sleep: sleeper.sleep,
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.attempts.length,)
              .toBe(2,);
            expect(sleeper.delays)
              .toEqual([QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS,],);
          },
        },),
        it({
          name: 'gives up after the mount-race attempt budget when the writer lock never frees',
          fn: async () => {
            const sleeper = recordingSleepOf();
            const fake = fakeSettingsOf({
              failWrites: Array.from({
                length: QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS,
              }, function byAttempt(_value, index) {
                return index;
              },),
              refuseWith: new Error(UPSTREAM_WRITER_LOCK_TIMEOUT_MESSAGE),
            },);
            await expect(writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
              sleep: sleeper.sleep,
            },),)
              .rejects
              .toHaveProperty('message', UPSTREAM_WRITER_LOCK_TIMEOUT_MESSAGE,);
            expect(fake.attempts.length,)
              .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS,);
            expect(sleeper.delays.length,)
              .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS - 1,);
            expect(fake.writes.length,)
              .toBe(0,);
          },
        },),
        it({
          name: 'does not retry a permanent settings refusal',
          fn: async () => {
            const sleeper = recordingSleepOf();
            const fake = fakeSettingsOf({
              failWrites: [0,],
              refuseWith: new Error(UPSTREAM_MISSING_ENTRY_MESSAGE),
            },);
            await expect(writeQwenTokenPlanIndividualRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
              sleep: sleeper.sleep,
            },),)
              .rejects
              .toHaveProperty('message', UPSTREAM_MISSING_ENTRY_MESSAGE,);
            expect(fake.attempts.length,)
              .toBe(1,);
            expect(sleeper.delays)
              .toEqual([]);
          },
        },),
        it({
          name: 'classifies every upstream contention message as transient and the missing-entry refusal as permanent',
          fn: async () => {
            for (
              const message of [
                UPSTREAM_WRITER_LOCK_TIMEOUT_MESSAGE,
                UPSTREAM_ENTRY_CHANGED_MESSAGE,
                UPSTREAM_ENTRY_UNAVAILABLE_MESSAGE,
                UPSTREAM_PLUGIN_INACTIVE_MESSAGE,
              ]
            ) {
              expect(isTransientQwenTokenPlanIndividualSettingsWriteFailure({
                error: new Error(message),
              },))
                .toBe(true,);
            }
            expect(isTransientQwenTokenPlanIndividualSettingsWriteFailure({
              error: new Error(UPSTREAM_MISSING_ENTRY_MESSAGE),
            },))
              .toBe(false,);
          },
        },),
      ],
    },),

    //endregion Route write

    //region Mount refresh

    describe({
      name: refreshQwenTokenPlanIndividualRoute.name,
      children: [
        it({
          name: 'writes the checked-in snapshot first and the live listing second',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => [{ id: 'live-a', }, { id: 'live-b', },],
            },);
            expect(fake.writes.length,)
              .toBe(2,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
            expect(fake.writes[1]
              ?.patch,)
              .toEqual({
                providers: mapQwenTokenPlanIndividualProviders({
                  listing: [{ id: 'live-a', }, { id: 'live-b', },],
                },),
              },);
          },
        },),
        it({
          name: 'forwards the resolved credential value and the DSH endpoint base to the listing fetch',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'live-key', });
            /**
             Wire facts the injected fetch received; each field stays absent
             until the fetch receives it.
             */
            const seen: { baseUrl?: string; apiKey?: string; } = {};
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async (options) => {
                seen.baseUrl = options.baseUrl;
                if (options.apiKey !== undefined) {
                  seen.apiKey = options.apiKey;
                }
                return [{ id: 'live-a', },];
              },
            },);
            expect(seen.baseUrl).toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL);
            expect(seen.apiKey).toBe('live-key',);
          },
        },),
        it({
          name: 'keeps the served snapshot and returns when the fetch fails',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => {
                throw new Error('offline',);
              },
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
          },
        },),
        it({
          name: 'keeps the served snapshot when the live listing maps to no serviceable model',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => [
                { id: 'wan3.0-video', },
                { id: 'probe-image', },
              ],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
          },
        },),
        it({
          name: 'keeps the served snapshot when the live listing is empty',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => [],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
          },
        },),
        it({
          name: 'keeps the served snapshot when no credentials lookup is provided',
          fn: async () => {
            const fake = fakeSettingsOf();
            /**
             Listing calls the injected fetch received; the no-credential
             degradation must leave it untouched.
             */
            let fetchCalls = 0;
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              fetchListing: async () => {
                fetchCalls += 1;
                return [{ id: 'live-a', },];
              },
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
            expect(fetchCalls).toBe(0,);
          },
        },),
        it({
          name: 'keeps the served snapshot when no credentials service appears within the mount-race budget',
          fn: async () => {
            const sleeper = recordingSleepOf();
            const fake = fakeSettingsOf();
            /**
             Lookup calls the refresh made; the budget must bound the wait for
             a service the loader registers late.
             */
            let lookupCalls = 0;
            let fetchCalls = 0;
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: function getCredentials(): QwenTokenPlanIndividualCredentialsLookupResult {
                lookupCalls += 1;
                return {};
              },
              fetchListing: async () => {
                fetchCalls += 1;
                return [{ id: 'live-a', },];
              },
              sleep: sleeper.sleep,
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
            expect(fetchCalls).toBe(0,);
            expect(lookupCalls).toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS,);
            expect(sleeper.delays.length)
              .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS - 1,);
          },
        },),
        it({
          name: 'lands the live listing when the credentials service mounts after the refresh starts',
          fn: async () => {
            const sleeper = recordingSleepOf();
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'late-key', });
            /**
             Mount moment the lookup saw nothing, then a mounted service; the
             loader can register the service after this plugin's `apply`.
             */
            const ABSENT_LOOKUPS = 2;
            let lookupCalls = 0;
            /**
             API keys the fetch received; the delayed lookup must deliver the
             resolved one.
             */
            const seenApiKeys: string[] = [];
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: function getCredentials(): QwenTokenPlanIndividualCredentialsLookupResult {
                lookupCalls += 1;
                return lookupCalls <= ABSENT_LOOKUPS
                  ? {}
                  : { credentials: credentials.target, };
              },
              fetchListing: async (options) => {
                if (options.apiKey !== undefined) {
                  seenApiKeys.push(options.apiKey);
                }
                return [{ id: 'live-a', },];
              },
              sleep: sleeper.sleep,
            },);
            expect(fake.writes.length,)
              .toBe(2,);
            expect(fake.writes[1]
              ?.patch,)
              .toEqual({
                providers: mapQwenTokenPlanIndividualProviders({
                  listing: [{ id: 'live-a', },],
                },),
              },);
            expect(seenApiKeys).toEqual(['late-key',],);
            expect(lookupCalls).toBe(ABSENT_LOOKUPS + 1,);
            expect(sleeper.delays.length)
              .toBe(ABSENT_LOOKUPS,);
          },
        },),
        it({
          name: 'keeps the served snapshot when the credential reference resolves no value',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf();
            let fetchCalls = 0;
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => {
                fetchCalls += 1;
                return [{ id: 'live-a', },];
              },
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
            expect(fetchCalls).toBe(0,);
          },
        },),
        it({
          name: 'keeps the served snapshot when credential resolution fails',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({
              rejectWith: new Error('credential store unreadable',),
            },);
            let fetchCalls = 0;
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => {
                fetchCalls += 1;
                return [{ id: 'live-a', },];
              },
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
            expect(fetchCalls).toBe(0,);
          },
        },),
        it({
          name: 'resolves the route credential reference through the credentials service',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => [{ id: 'live-a', },],
            },);
            expect(credentials.askedAbout)
              .toEqual(['QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY',]);
          },
        },),
        it({
          name: 'still lands the live listing when the snapshot write is refused',
          fn: async () => {
            const fake = fakeSettingsOf({ failWrites: [0,], },);
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => [{ id: 'live-a', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapQwenTokenPlanIndividualProviders({
                  listing: [{ id: 'live-a', },],
                },),
              },);
          },
        },),
        it({
          name: 'keeps the served snapshot when the live write is refused',
          fn: async () => {
            const fake = fakeSettingsOf({ failWrites: [1,], },);
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async () => [{ id: 'live-a', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({ providers: expectedSnapshotProviders(), },);
          },
        },),
        it({
          name: 'writes the full measured serviceable membership with the media ids absent',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              getCredentials: () => ({ credentials: credentials.target, }),
              fetchListing: async (): Promise<readonly QwenTokenPlanListingEntry[]> => {
                throw new Error('offline',);
              },
            },);
            /**
             Served model ids the snapshot write carried.
             */
            const writtenModelIds = writtenProvidersOf(firstWriteOf(fake.writes,))[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]
              ?.models
              ?.map(function byId(model) {
                return model.id;
              },) ?? [];
            expect([...writtenModelIds].toSorted())
              .toEqual([...MEASURED_SERVICEABLE_IDS].toSorted(),);
            /**
             Written ids the measured listing classified as media; the media
             skip rule must empty this.
             */
            const writtenMediaIds = writtenModelIds.filter(function isMeasuredMedia(id) {
              return (MEASURED_MEDIA_IDS as readonly string[])
                .includes(id);
            },);
            expect(writtenMediaIds).toEqual([]);
          },
        },),
      ],
    },),

    //endregion Mount refresh
  ],
},);
