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
  orderFlavors,
  resolveFlavor,
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
  ],
},);
