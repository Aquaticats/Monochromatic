import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mapQwenTokenPlanIndividualProviders,
  qwenTokenPlanIndividualListingSnapshot,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,
  refreshQwenTokenPlanIndividualRoute,
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
 One recorded settings merge write.
 */
type RecordedWrite = {
  ns: string;
  patch: Record<string, unknown>;
};

/**
 Wire options the injected listing fetch received, captured so a test can
 assert the refresh forwards the resolved credential.
 */
type CapturedFetchOptions = {
  baseUrl: string;
  apiKey: string | undefined;
};

/**
 Recording fake of the settings seam: serves a fixed form value through
 `describe`, records every accepted merge write, and can refuse writes at the
 given zero-based call indices to reach the failure branches.

 @param options - stored form value and refusing call indices

 @returns fake target plus the writes it accepted

 @example
 ```ts
 const fake = fakeSettingsOf({ stored: { providers: {} } });
 ```
 */
function fakeSettingsOf({
  stored,
  failWrites = [],
}: {
  readonly stored?: unknown;
  readonly failWrites?: readonly number[];
} = {}): {
  readonly target: QwenTokenPlanIndividualSettingsTarget;
  readonly writes: RecordedWrite[];
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
        if (failWrites.includes(attempt)) throw new Error('settings entry unavailable');
        writes.push({ ns, patch });
      },
    },
    writes,
  };
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
  readonly rejectWith?: unknown;
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
            const writtenProviderKeys = Object.keys(
              (fake.writes[0]?.patch['providers'] ?? {}) as Record<string, unknown>,
            );
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
              credentials: credentials.target,
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
             Wire options the injected fetch received.
             */
            let captured: CapturedFetchOptions | undefined;
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              credentials: credentials.target,
              fetchListing: async (options) => {
                captured = {
                  baseUrl: options.baseUrl,
                  apiKey: options.apiKey,
                };
                return [{ id: 'live-a', },];
              },
            },);
            expect(captured).toEqual({
              baseUrl: QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
              apiKey: 'live-key',
            },);
          },
        },),
        it({
          name: 'keeps the served snapshot and returns when the fetch fails',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf({ value: 'test-key', });
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              credentials: credentials.target,
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
              credentials: credentials.target,
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
              credentials: credentials.target,
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
          name: 'keeps the served snapshot when no credentials service is mounted',
          fn: async () => {
            const fake = fakeSettingsOf();
            /**
             Listing calls the injected fetch received; the no-credential
             degradation must leave it untouched.
             */
            let fetchCalls = 0;
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              credentials: undefined,
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
          name: 'keeps the served snapshot when the credential reference resolves no value',
          fn: async () => {
            const fake = fakeSettingsOf();
            const credentials = fakeCredentialsOf();
            let fetchCalls = 0;
            await refreshQwenTokenPlanIndividualRoute({
              settings: fake.target,
              credentials: credentials.target,
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
              credentials: credentials.target,
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
              credentials: credentials.target,
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
              credentials: credentials.target,
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
              credentials: credentials.target,
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
              credentials: credentials.target,
              fetchListing: async (): Promise<readonly QwenTokenPlanListingEntry[]> => {
                throw new Error('offline',);
              },
            },);
            /**
             Served model ids the snapshot write carried.
             */
            const writtenModelIds = (
              ((fake.writes[0]?.patch['providers'] as Record<string, unknown> | undefined)
                ?.[QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE] as { models?: readonly { id: string }[] })
                ?.models ?? []
            )
              .map(function byId(model) {
                return model.id;
              },);
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
