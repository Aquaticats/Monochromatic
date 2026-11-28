/**
 Unit tests for the size a caller asks a machine to have: how the two numbers are
 read, how they are described back in a refusal, and the refusal a backend raises
 when nothing it may use fits them.

 The numbers here are request values a person types, not a provider's catalogue,
 so nothing in this file claims what any provider offers.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeSizeRequest,
  isEmptySizeRequest,
  memoryGiBArgument,
  NoSizeFitsError,
  refuseSizeAndOverride,
  sizeRequest,
  vcpusArgument,
} from '../../dist/final/node/index.mjs';
// The one test helper this file borrows; it reads a thrown message and names its input.
import { throwMessage, } from './ovh/fixture.ovh-api-stand-in.ts';

//region Fixtures

/**
 Smallest number of virtual CPUs a request may carry.
 */
const TWO_VCPUS = 2;

/**
 Memory a request in these tests asks for, in GiB.
 */
const FOUR_GIB = 4;

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    //region Reading the two numbers

    describe({
      name: sizeRequest.name,
      children: [
        it({
          name: 'carries each number that was given and omits the one that was not',
          fn: async () => {
            expect(sizeRequest({
              memoryGiB: FOUR_GIB,
              vcpus: undefined,
            },),).toEqual({ memoryGiB: FOUR_GIB, },);
            expect(sizeRequest({
              memoryGiB: undefined,
              vcpus: TWO_VCPUS,
            },),).toEqual({ vcpus: TWO_VCPUS, },);
            expect(sizeRequest({
              memoryGiB: FOUR_GIB,
              vcpus: TWO_VCPUS,
            },),).toEqual({
              memoryGiB: FOUR_GIB,
              vcpus: TWO_VCPUS,
            },);
          },
        },),
      ],
    },),

    describe({
      name: isEmptySizeRequest.name,
      children: [
        it({
          name: 'is empty when neither number was given',
          fn: async () => {
            expect(isEmptySizeRequest({ request: {}, },),).toBe(true,);
          },
        },),
        it({
          name: 'is not empty when either number was given',
          fn: async () => {
            expect(isEmptySizeRequest({
              request: { vcpus: TWO_VCPUS, },
            },),).toBe(false,);
            expect(isEmptySizeRequest({
              request: { memoryGiB: FOUR_GIB, },
            },),).toBe(false,);
            expect(isEmptySizeRequest({
              request: {
                memoryGiB: FOUR_GIB,
                vcpus: TWO_VCPUS,
              },
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: describeSizeRequest.name,
      children: [
        it({
          name: 'names each number as a floor and joins both with "and"',
          fn: async () => {
            expect(describeSizeRequest({ request: { vcpus: TWO_VCPUS, }, },),)
              .toBe('at least 2 vCPUs',);
            expect(describeSizeRequest({ request: { memoryGiB: FOUR_GIB, }, },),)
              .toBe('at least 4 GiB of memory',);
            expect(
              describeSizeRequest({
                request: {
                  memoryGiB: FOUR_GIB,
                  vcpus: TWO_VCPUS,
                },
              },),
            ).toBe('at least 2 vCPUs and at least 4 GiB of memory',);
          },
        },),
        it({
          name: 'describes an empty request as nothing asked',
          fn: async () => {
            expect(describeSizeRequest({ request: {}, },),).toBe('nothing',);
          },
        },),
      ],
    },),

    describe({
      name: vcpusArgument.name,
      children: [
        it({
          name: 'keeps absent and reads a positive whole number',
          fn: async () => {
            expect(vcpusArgument(undefined,),).toBeUndefined();
            expect(vcpusArgument(TWO_VCPUS,),).toBe(TWO_VCPUS,);
          },
        },),
        it({
          name: 'refuses zero, a negative number, a fraction and a non-number, naming the input',
          fn: async () => {
            for (const bad of [
              0,
              -1,
              1.5,
              '2',
            ]) {
              const message = throwMessage({
                run: () => vcpusArgument(bad,),
              },);
              expect(message,).toContain('vcpus',);
              expect(message,).toContain(String(bad,),);
            }
          },
        },),
      ],
    },),

    describe({
      name: memoryGiBArgument.name,
      children: [
        it({
          name: 'keeps absent and reads a positive whole number',
          fn: async () => {
            expect(memoryGiBArgument(undefined,),).toBeUndefined();
            expect(memoryGiBArgument(FOUR_GIB,),).toBe(FOUR_GIB,);
          },
        },),
        it({
          name: 'refuses zero, a negative number, a fraction and a non-number, naming the input',
          fn: async () => {
            for (const bad of [
              0,
              -2,
              0.5,
              '4',
            ]) {
              const message = throwMessage({
                run: () => memoryGiBArgument(bad,),
              },);
              expect(message,).toContain('memory',);
              expect(message,).toContain(String(bad,),);
            }
          },
        },),
      ],
    },),

    //endregion Reading the two numbers

    //region Refusing a named size beside a number

    describe({
      name: refuseSizeAndOverride.name,
      children: [
        it({
          name: 'passes when only one of the two ways to size a machine was used',
          fn: async () => {
            refuseSizeAndOverride({
              override: '',
              overrideOptions: '--server-type or MVM_OVH_FLAVOR',
              request: { vcpus: TWO_VCPUS, },
            },);
            refuseSizeAndOverride({
              override: 'b3-8',
              overrideOptions: '--server-type or MVM_OVH_FLAVOR',
              request: {},
            },);
          },
        },),
        it({
          name: 'refuses a named size beside a number, naming both and the way out',
          fn: async () => {
            const message = throwMessage({
              run: () => refuseSizeAndOverride({
                override: 'b3-8',
                overrideOptions: '--server-type or MVM_OVH_FLAVOR',
                request: { vcpus: TWO_VCPUS, },
              },),
            },);
            expect(message,).toContain('b3-8',);
            expect(message,).toContain('at least 2 vCPUs',);
            expect(message,).toContain('--server-type or MVM_OVH_FLAVOR',);
          },
        },),
      ],
    },),

    //endregion Refusing a named size beside a number

    //region The refusal when nothing fits

    describe({
      name: NoSizeFitsError.name,
      children: [
        it({
          name: 'names what was asked, the largest size it could have used, and the way to name one',
          fn: async () => {
            const error = new NoSizeFitsError({
              asked: 'at least 2 vCPUs and at least 4 GiB of memory',
              largest: 'b3-256 (64 vCPUs, 256 GiB of memory, 400 GB of disk)',
              overrideOptions: '--server-type or MVM_OVH_FLAVOR',
              place: 'region BHS5 on OVHcloud',
            },);
            expect(error,).toBeInstanceOf(Error,);
            expect(error.message,).toContain('region BHS5 on OVHcloud',);
            expect(error.message,).toContain('at least 2 vCPUs and at least 4 GiB of memory',);
            expect(error.message,).toContain('b3-256',);
            expect(error.message,).toContain('--server-type or MVM_OVH_FLAVOR',);
          },
        },),
        it({
          name: 'says no size could have been used at all when the place offers none to a create',
          fn: async () => {
            const error = new NoSizeFitsError({
              asked: 'at least 2 vCPUs',
              largest: '',
              overrideOptions: '--server-type or MVM_HCLOUD_SERVER_TYPE',
              place: 'the requested locations on Hetzner',
            },);
            expect(error.message,).toContain('no size it may use',);
            expect(error.message,).toContain('--server-type or MVM_HCLOUD_SERVER_TYPE',);
          },
        },),
        it({
          name: 'says how many sizes the family rule left out when it left any',
          fn: async () => {
            const error = new NoSizeFitsError({
              asked: 'at least 2 vCPUs',
              largest: '',
              leftOut: 1,
              overrideOptions: '--server-type or MVM_OVH_FLAVOR',
              place: 'region BHS5 on OVHcloud',
            },);
            expect(error.message,).toContain('1 size in stock was left out',);
          },
        },),
      ],
    },),

    //endregion The refusal when nothing fits
  ],
},);
