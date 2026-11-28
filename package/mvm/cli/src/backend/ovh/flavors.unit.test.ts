/**
 Unit tests for flavor selection: the smallest flavor in stock wins, a flavor
 that is out of stock is never chosen, a Linux flavor is preferred, an explicit
 `--server-type` or `MVM_OVH_FLAVOR` wins by identifier or by name, and a region
 with nothing in stock says so with the error a create walks past.

 Flavor bodies here are stand-ins written by hand from OVHcloud's schema
 (`cloud.flavor.Flavor`), never a recording, so nothing in this file claims what
 OVHcloud really offers in a region.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NoFlavorInStockError,
  NoSizeFitsError,
  orderFlavors,
  resolveFlavor,
  resolveFlavorsThatFit,
  selectFlavor,
  type OvhFlavor,
} from '../../../dist/final/node/index.mjs';
import {
  cloudPath,
  errorFrom,
  flavorAnswer,
  installEnv,
  installStandIn,
  jsonAnswer,
  matching,
  throwMessage,
  withEnv,
} from './fixture.ovh-api-stand-in.ts';

//region Fixtures

/**
 Flavors of one region: a large one, the smallest one, one out of stock that is
 smaller still, the smallest Windows one, and the smallest arm one.
 */
const FLAVORS = [
  flavorAnswer({
    id: 'b3-16',
    ram: 16,
    vcpus: 4,
  },),
  flavorAnswer({
    id: 'b3-8',
    ram: 8,
    vcpus: 2,
  },),
  flavorAnswer({
    available: false,
    id: 'out-of-stock',
    ram: 1,
    vcpus: 1,
  },),
  flavorAnswer({
    id: 'windows-small',
    osType: 'windows',
    ram: 4,
    vcpus: 1,
  },),
  flavorAnswer({
    id: 'arm-small',
    ram: 2,
    vcpus: 1,
  },),
] as const satisfies readonly OvhFlavor[];

//endregion Fixtures

//region Size fixtures

/**
 Virtual CPUs the size cases ask for.
 */
const ASKED_VCPUS = 2;

/**
 Memory the size cases ask for, in gibibytes.
 */
const ASKED_MEMORY_GIB = 4;

/**
 A catalogue shaped like the real BHS5 one on 2026-11-28: the smallest sizes the
 general-purpose families offered, one flex twin beside a plain size, and the
 GPU, storage shaped and bare metal families the region also listed.
 */
const REAL_SHAPED = [
  flavorAnswer({ disk: 25, id: 'd2-2', ram: 2, vcpus: 1 }),
  flavorAnswer({ disk: 50, id: 'c3-4', ram: 4, vcpus: 2 }),
  flavorAnswer({ disk: 50, id: 'c3-4-flex', ram: 4, vcpus: 2 }),
  flavorAnswer({ disk: 50, id: 'b2-7', ram: 7, vcpus: 2 }),
  flavorAnswer({ disk: 50, id: 'b2-7-flex', ram: 7, vcpus: 2 }),
  flavorAnswer({ disk: 50, id: 'b3-8', ram: 8, vcpus: 2 }),
  flavorAnswer({ disk: 25, id: 't1-45', ram: 45, vcpus: 8 }),
  flavorAnswer({ disk: 25, id: 'i1-45', ram: 45, vcpus: 8 }),
  flavorAnswer({ disk: 50, id: 'bm-l1', osType: 'baremetal-linux', ram: 96, vcpus: 16 }),
] as const satisfies readonly OvhFlavor[];

//endregion Size fixtures

await describe({
  name: 'ovh flavors',
  concurrency: 1,
  children: [
    //region Ordering

    it({
      name: 'orderFlavors sorts by virtual CPUs, then memory, then identifier',
      fn: async () => {
        expect(
          orderFlavors({ flavors: FLAVORS, },)
            .map(function flavorId(flavor,) {
              return flavor.id;
            },),
        ).toEqual(['out-of-stock', 'arm-small', 'windows-small', 'b3-8', 'b3-16',],);
      },
    },),

    //endregion Ordering

    //region Selection

    it({
      name: 'selectFlavor picks the smallest flavor in stock and never an unavailable one',
      fn: async () => {
        const chosen = selectFlavor({
          flavors: FLAVORS,
          override: '',
          region: 'bhs5',
        },);
        expect(chosen.id,).toBe('arm-small',);
      },
    },),
    it({
      name: 'selectFlavor prefers a Linux flavor over a smaller Windows one',
      fn: async () => {
        const chosen = selectFlavor({
          flavors: [
            flavorAnswer({
              id: 'windows-tiny',
              osType: 'windows',
              ram: 1,
              vcpus: 1,
            },),
            flavorAnswer({
              id: 'linux-big',
              ram: 8,
              vcpus: 2,
            },),
          ],
          override: '',
          region: 'bhs5',
        },);
        expect(chosen.id,).toBe('linux-big',);
      },
    },),
    it({
      name: 'selectFlavor falls back to the smallest flavor of any kind when no Linux flavor is in stock',
      fn: async () => {
        const chosen = selectFlavor({
          flavors: [flavorAnswer({
            id: 'windows-only',
            osType: 'windows',
          },),],
          override: '',
          region: 'bhs5',
        },);
        expect(chosen.id,).toBe('windows-only',);
      },
    },),
    it({
      name: 'selectFlavor takes an override by identifier and by name',
      fn: async () => {
        expect(
          selectFlavor({
            flavors: FLAVORS,
            override: 'b3-16',
            region: 'bhs5',
          },).id,
        ).toBe('b3-16',);
        expect(
          selectFlavor({
            flavors: FLAVORS,
            override: 'b3-8',
            region: 'bhs5',
          },).id,
        ).toBe('b3-8',);
      },
    },),
    it({
      name: 'selectFlavor names what is in stock when the override names nothing in stock there',
      fn: async () => {
        const message = throwMessage({
          run: () => selectFlavor({
            flavors: FLAVORS,
            override: 'out-of-stock',
            region: 'bhs5',
          },),
        },);
        expect(message,).toContain('out-of-stock',);
        expect(message,).toContain('bhs5',);
        expect(message,).toContain('b3-8',);
      },
    },),
    it({
      name: 'selectFlavor raises the error a create walks past when the region has nothing in stock',
      fn: async () => {
        const error = await errorFrom({
          run: async function choose() {
            selectFlavor({
              flavors: [flavorAnswer({ available: false, },),],
              override: '',
              region: 'bhs5',
            },);
          },
        },);
        expect(error,).toBeInstanceOf(NoFlavorInStockError,);
      },
    },),

    //endregion Selection

    //region Reading

    it({
      name: 'resolveFlavor reads the region own flavors and honours the override from the environment',
      fn: async () => {
        await using _env = await installEnv();
        using _set = withEnv({
          name: 'MVM_OVH_FLAVOR',
          value: 'b3-16',
        },);
        using standIn = installStandIn(() => jsonAnswer(FLAVORS,),);
        const chosen = await resolveFlavor({ region: 'bhs5', },);
        expect(chosen.id,).toBe('b3-16',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'GET',
            path: `${cloudPath('/flavor',)}?region=bhs5`,
          },).length,
        ).toBe(1,);
      },
    },),

    //endregion Reading

    //region Ordering by disk and by the flex twin

    it({
      name: 'orderFlavors breaks a tie on virtual CPUs and memory with the smaller disk',
      fn: async () => {
        expect(
          orderFlavors({
            flavors: [
              flavorAnswer({ disk: 100, id: 'b3-8-big-disk', ram: 8, vcpus: 2 }),
              flavorAnswer({ disk: 50, id: 'b3-8', ram: 8, vcpus: 2 }),
            ],
          })
            .map(function flavorId(flavor) {
              return flavor.id;
            }),
        ).toEqual(['b3-8', 'b3-8-big-disk']);
      },
    }),
    it({
      name: 'orderFlavors prefers the plain size over its flex twin, which has the same numbers',
      fn: async () => {
        expect(
          orderFlavors({
            flavors: [
              flavorAnswer({ disk: 50, id: 'c3-4-flex', ram: 4, vcpus: 2 }),
              flavorAnswer({ disk: 50, id: 'c3-4', ram: 4, vcpus: 2 }),
            ],
          })
            .map(function flavorId(flavor) {
              return flavor.id;
            }),
        ).toEqual(['c3-4', 'c3-4-flex']);
      },
    }),

    //endregion Ordering by disk and by the flex twin

    //region Sizes a request may use

    it({
      name: 'resolveFlavorsThatFit keeps only the general-purpose families and the floors the request set',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(REAL_SHAPED));
        const fit = await resolveFlavorsThatFit({
          region: 'bhs5',
          request: { memoryGiB: ASKED_MEMORY_GIB, vcpus: ASKED_VCPUS },
        });
        expect(
          fit.flavors.map(function flavorId(flavor) {
            return flavor.id;
          }),
        ).toEqual(['c3-4', 'c3-4-flex', 'b2-7', 'b2-7-flex', 'b3-8']);
        expect(fit.leftOut).toBe(3);
        expect(
          matching({
            calls: standIn.calls,
            method: 'GET',
            path: `${cloudPath('/flavor')}?region=bhs5`,
          }).length,
        ).toBe(1);
      },
    }),
    it({
      name: 'resolveFlavorsThatFit takes the smallest size that fits, not the largest in stock',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(REAL_SHAPED));
        const fit = await resolveFlavorsThatFit({
          region: 'bhs5',
          request: { vcpus: ASKED_VCPUS },
        });
        expect(fit.flavors[0]?.id).toBe('c3-4');
        expect(standIn.calls.length).toBeGreaterThan(0);
      },
    }),
    it({
      name: 'refuses and says so when the only size that fits is a GPU machine',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(
          () => jsonAnswer([flavorAnswer({ disk: 25, id: 't1-45', ram: 45, vcpus: 8 })]),
        );
        const error = await errorFrom({
          run: async function choose() {
            await resolveFlavorsThatFit({ region: 'bhs5', request: { vcpus: 1 } });
          },
        });
        expect(error).toBeInstanceOf(NoSizeFitsError);
        expect((error as Error).message).toContain(
          '1 size in stock was left out as a family mvm does not pick without being told',
        );
        expect((error as Error).message).toContain('--server-type');
        expect(standIn.calls.length).toBeGreaterThan(0);
      },
    }),
    it({
      name: 'names the largest size it could have used when the floors are above every size in stock',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(REAL_SHAPED));
        const error = await errorFrom({
          run: async function choose() {
            await resolveFlavorsThatFit({
              region: 'bhs5',
              request: { memoryGiB: 512, vcpus: 128 },
            });
          },
        });
        expect(error).toBeInstanceOf(NoSizeFitsError);
        expect((error as Error).message).toContain('at least 128 vCPUs');
        expect((error as Error).message).toContain('at least 512 GiB of memory');
        expect((error as Error).message).toContain('b3-8');
        expect((error as Error).message).toContain('bhs5');
        expect(standIn.calls.length).toBeGreaterThan(0);
      },
    }),
    it({
      name: 'refuses a flavor named outright beside a floor, without reading the catalogue',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(REAL_SHAPED));
        const error = await errorFrom({
          run: async function choose() {
            await resolveFlavorsThatFit({
              override: 'b3-8',
              region: 'bhs5',
              request: { vcpus: ASKED_VCPUS },
            });
          },
        });
        expect((error as Error).message).toContain('b3-8');
        expect((error as Error).message).toContain('--server-type');
        expect(standIn.calls.length).toBe(0);
      },
    }),

    //endregion Sizes a request may use
  ],
},);
