/**
 Unit tests for the cheapest-server-type resolver against a stubbed `fetch`:
 picks the cheapest non-deprecated type offered in the target locations,
 honours an architecture filter, and throws when nothing matches. No real
 calls.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { resolveCheapestServerType, } from '@monochromatic-dev/mvm-cli/ts/backend/hetzner/server-types.ts';

/**
 Sets HCLOUD_TOKEN for a `using` scope so requireToken passes, restoring after.
 */
function withToken(): Disposable {
  const prior = process.env.HCLOUD_TOKEN;
  process.env.HCLOUD_TOKEN = 'tok-test';
  return {
    [Symbol.dispose]() {
      if (prior === undefined) {
        Reflect.deleteProperty(process.env, 'HCLOUD_TOKEN',);
      }
      else {
        process.env.HCLOUD_TOKEN = prior;
      }
    },
  };
}

/**
 Replaces global fetch with a fixed server_types body; restores on dispose.
 */
function installServerTypes(serverTypes: readonly unknown[],): Disposable {
  const original = globalThis.fetch;
  globalThis.fetch = (async function stubFetch() {
    return Response.json(
      { server_types: serverTypes, meta: { pagination: { next_page: null, }, }, },
      { status: 200, },
    );
  }) as unknown as typeof fetch;
  return {
    [Symbol.dispose]() {
      globalThis.fetch = original;
    },
  };
}

/**
 Mixed server types: a deprecated cheap one, a US-only cheap one, and the
 cheapest EU x86 and arm options.
 */
const SERVER_TYPES = [
  { name: 'cx23', architecture: 'x86', deprecation: null, prices: [{ location: 'fsn1', price_hourly: { gross: '0.0080', }, }, { location: 'nbg1', price_hourly: { gross: '0.0080', }, },], },
  { name: 'cax11', architecture: 'arm', deprecation: null, prices: [{ location: 'fsn1', price_hourly: { gross: '0.0088', }, },], },
  { name: 'cx22', architecture: 'x86', deprecation: { unavailable_after: '2026-01-01T00:00:00Z', }, prices: [{ location: 'fsn1', price_hourly: { gross: '0.0050', }, },], },
  { name: 'ccx-us', architecture: 'x86', deprecation: null, prices: [{ location: 'ash', price_hourly: { gross: '0.0001', }, },], },
];

await describe({
  name: 'hetzner cheapest server type',
  concurrency: 1,
  children: [
    it({
      name: 'picks the cheapest non-deprecated type offered in the locations',
      fn: async () => {
        using _t = withToken();
        using _mock = installServerTypes(SERVER_TYPES,);
        // cx22 is cheaper but deprecated; ccx-us is cheapest but not in fsn1/nbg1.
        expect(await resolveCheapestServerType({ locations: ['fsn1', 'nbg1',], },),).toBe('cx23',);
      },
    },),
    it({
      name: 'honours an architecture filter',
      fn: async () => {
        using _t = withToken();
        using _mock = installServerTypes(SERVER_TYPES,);
        expect(
          await resolveCheapestServerType({ architecture: 'arm', locations: ['fsn1',], },),
        ).toBe('cax11',);
      },
    },),
    it({
      name: 'throws when no type is offered in the requested locations',
      fn: async () => {
        using _t = withToken();
        using _mock = installServerTypes(SERVER_TYPES,);
        await expect(
          resolveCheapestServerType({ locations: ['sin',], },),
        ).rejects.toThrow('no non-deprecated Hetzner server type',);
      },
    },),
  ],
},);
