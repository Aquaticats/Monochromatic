/**
 Unit tests for the server type a size request may use, against a stubbed
 `fetch`: the smallest type that meets both floors wins, a deprecated type, a
 type the locations do not offer and a plan sold with dedicated cores are all
 left alone, and a request nothing meets refuses and names what it declined.

 The stand-in catalogue is shaped like Hetzner's documented server type object
 (`cores`, `memory`, `disk`, `cpu_type`, `architecture`, `deprecation`, `prices`
 with a `location` each), copied from `hcloud-python`'s `ServerType` model and
 the Terraform `hcloud_server_type` data source read on 2026-11-28. The plan
 names are Hetzner's own; the numbers are chosen so each rule is exercised,
 because this round makes no Hetzner call and therefore has no capture to copy
 sizes from.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeServerType,
  NoSizeFitsError,
  orderServerTypes,
  resolveServerType,
} from '../../../dist/final/node/index.mjs';

//region Fixtures

/**
 Cores the size cases ask for.
 */
const ASKED_VCPUS = 2;

/**
 Memory the size cases ask for, in gigabytes.
 */
const ASKED_MEMORY_GB = 4;

/**
 A catalogue with one type per rule: the general-purpose smallest, a bigger
 general-purpose one, a deprecated one, one the location does not offer, one
 whose CPU is dedicated, and an arm one.
 */
const TYPES = [
  {
    name: 'cx23',
    architecture: 'x86',
    cores: 2,
    cpu_type: 'shared',
    deprecation: null,
    disk: 40,
    memory: 4,
    prices: [{ location: 'fsn1', price_hourly: { gross: '0.0080', }, },],
  },
  {
    name: 'cx33',
    architecture: 'x86',
    cores: 4,
    cpu_type: 'shared',
    deprecation: null,
    disk: 80,
    memory: 8,
    prices: [{ location: 'fsn1', price_hourly: { gross: '0.0150', }, },],
  },
  {
    name: 'cx22',
    architecture: 'x86',
    cores: 2,
    cpu_type: 'shared',
    deprecation: { unavailable_after: '2026-01-01T00:00:00Z', },
    disk: 40,
    memory: 4,
    prices: [{ location: 'fsn1', price_hourly: { gross: '0.0050', }, },],
  },
  {
    name: 'cx-only-in-ash',
    architecture: 'x86',
    cores: 2,
    cpu_type: 'shared',
    deprecation: null,
    disk: 40,
    memory: 4,
    prices: [{ location: 'ash', price_hourly: { gross: '0.0040', }, },],
  },
  {
    name: 'ccx13',
    architecture: 'x86',
    cores: 2,
    cpu_type: 'dedicated',
    deprecation: null,
    disk: 80,
    memory: 8,
    prices: [{ location: 'fsn1', price_hourly: { gross: '0.0300', }, },],
  },
  {
    name: 'cax11',
    architecture: 'arm',
    cores: 2,
    cpu_type: 'shared',
    deprecation: null,
    disk: 40,
    memory: 4,
    prices: [{ location: 'fsn1', price_hourly: { gross: '0.0088', }, },],
  },
] as const;

/**
 Locations the size cases ask about.
 */
const LOCATIONS = ['fsn1',] as const;

//endregion Fixtures

//region Helpers

/**
 Runs something expected to reject and returns what it threw.
 
 @param run - the call expected to fail
 
 @returns the thrown value
 
 @throws Error when the call resolved instead of throwing
 
 @example
 ```ts
 const error = await rejected({ run: () => resolveServerType({ locations, request }) });
 ```
 */
async function rejected({
  run,
}: {
  readonly run: () => Promise<unknown>;
},): Promise<unknown> {
  try {
    await run();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('the call was expected to fail and resolved instead',);
}

//endregion Helpers

await describe({
  name: '',
  concurrency: 1,
  children: [
    //region Ordering and description

    describe({
      name: orderServerTypes.name,
      children: [
        it({
          name: 'orders by cores, then memory, then disk, then name',
          fn: async () => {
            expect(
              orderServerTypes({ types: TYPES, },)
                .map(function typeName(type,) {
                  return type.name;
                },),
            ).toEqual([
              'cax11',
              'cx-only-in-ash',
              'cx22',
              'cx23',
              'ccx13',
              'cx33',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: describeServerType.name,
      children: [
        it({
          name: 'names the type with its cores, memory and disk',
          fn: async () => {
            expect(describeServerType(TYPES[0],),)
              .toBe('cx23 (2 cores, 4 GB of memory, 40 GB of disk)',);
          },
        },),
      ],
    },),

    //endregion Ordering and description

    //region Choosing a type for a request

    describe({
      name: resolveServerType.name,
      children: [
        it({
          name: 'takes the smallest type that meets both floors and leaves the rest alone',
          fn: async () => {
            const chosen = await resolveServerType({
              locations: LOCATIONS,
              request: {
                memoryGiB: ASKED_MEMORY_GB,
                vcpus: ASKED_VCPUS,
              },
              types: TYPES,
            },);
            expect(chosen.name,).toBe('cax11',);
          },
        },),
        it({
          name: 'moves to a larger shared-core type when the smallest one misses one floor',
          fn: async () => {
            const chosen = await resolveServerType({
              locations: LOCATIONS,
              request: { memoryGiB: 8, },
              types: TYPES,
            },);
            expect(chosen.name,).toBe('cx33',);
          },
        },),
        it({
          name: 'leaves a dedicated-core plan out even when it is the only type that meets the floors',
          fn: async () => {
            const error = await rejected({
              run: () => resolveServerType({
                locations: LOCATIONS,
                request: { memoryGiB: 8, vcpus: 2, },
                types: [TYPES[4],],
              },),
            },);
            expect(error,).toBeInstanceOf(NoSizeFitsError,);
            expect((error as Error).message,).toContain('at least 2 vCPUs',);
            expect((error as Error).message,).toContain('MVM_HCLOUD_SERVER_TYPE',);
            expect((error as Error).message,).toContain('1 size in stock was left out',);
          },
        }),
        it({
          name: 'refuses when the locations offer nothing, naming what was asked and the way to name a type',
          fn: async () => {
            const error = await rejected({
              run: () => resolveServerType({
                locations: ['sin',],
                request: { vcpus: ASKED_VCPUS, },
                types: TYPES,
              },),
            },);
            expect((error as Error).message,).toContain('at least 2 vCPUs',);
            expect((error as Error).message,).toContain('sin',);
            expect((error as Error).message,).toContain('--server-type',);
            expect((error as Error).message,).toContain('no size it may use',);
          },
        }),
        it({
          name: 'names the largest type the locations had when the floors are above every one of them',
          fn: async () => {
            const caught = await rejected({
              run: () => resolveServerType({
                locations: LOCATIONS,
                request: { vcpus: 64, },
                types: TYPES,
              },),
            },);
            expect(caught,).toBeInstanceOf(NoSizeFitsError,);
            expect((caught as Error).message,).toContain('cx33 (4 cores',);
          },
        }),
      ],
    },),

    //endregion Choosing a type for a request
  ],
},);
