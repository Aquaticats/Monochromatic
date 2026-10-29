/**
 Unit tests for the location-fallback creation path against a stubbed
 `fetch`: a 412 `resource_unavailable` advances to the next location, and an
 all-locations-out-of-stock run throws a clear error. No real provisioning.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { createWithFallback, } from '@monochromatic-dev/mvm-cli/ts/backend/hetzner/lifecycle.ts';

/**
 One recorded fetch invocation.
 */
type Call = {
  readonly url: string;
  readonly body: unknown;
};

/**
 Sets HCLOUD_TOKEN for a `using` scope so requireToken passes, restoring after.
 */
function withToken(): Disposable {
  const prior = process.env.HCLOUD_TOKEN;
  process.env.HCLOUD_TOKEN = 'tok-test';
  return {
    [Symbol.dispose]() {
      if (prior === undefined) {
        delete process.env.HCLOUD_TOKEN;
      }
      else {
        process.env.HCLOUD_TOKEN = prior;
      }
    },
  };
}

/**
 Builds a JSON Response for the stub.
 */
function jsonResponse(body: unknown, status = 200,): Response {
  return Response.json(body, { status, },);
}

/**
 Serves queued responses in order, recording each call; restores on dispose.
 */
function installQueue(responses: Response[],): Disposable & { readonly calls: Call[] } {
  const calls: Call[] = [];
  const original = globalThis.fetch;
  globalThis.fetch = (async function stubFetch(input: unknown, init?: RequestInit,) {
    const rawBody = init?.body;
    calls.push({
      url: String(input,),
      body: ((typeof rawBody) === 'string') ? JSON.parse(rawBody,) : undefined,
    },);
    return responses.shift() ?? jsonResponse({ error: { code: 'x', message: 'unexpected call', }, }, 500,);
  }) as unknown as typeof fetch;
  return {
    calls,
    [Symbol.dispose]() {
      globalThis.fetch = original;
    },
  };
}

/**
 No-op logger satisfying the helper's logger surface.
 */
const RL = {
  debug() {},
  info() {},
};

/**
 412 out-of-stock response.
 */
function outOfStock(): Response {
  return jsonResponse({ error: { code: 'resource_unavailable', message: 'no stock', }, }, 412,);
}

await describe({
  name: 'hetzner createWithFallback',
  concurrency: 1,
  children: [
    it({
      name: 'advances to the next location on a 412 and succeeds there',
      fn: async () => {
        using _t = withToken();
        using mock = installQueue([
          outOfStock(),
          jsonResponse({
            server: { id: 1, name: 'mvm-x', status: 'running', public_net: { ipv4: { ip: '203.0.113.7', }, }, labels: {}, },
            action: { id: 9, status: 'running', },
          },),
        ],);
        const result = await createWithFallback({
          image: 'ubuntu-24.04',
          fullName: 'mvm-x',
          locations: ['fsn1', 'nbg1',],
          rl: RL,
          serverType: 'cx22',
          sshKeyId: 1,
        },);
        expect(result.server.id,).toBe(1,);
        expect(mock.calls.length,).toBe(2,);
        const [, second] = mock.calls;
        if (second === undefined) {
          throw new Error('expected a second create call',);
        }
        expect((second.body as { location: string, }).location,).toBe('nbg1',);
      },
    },),
    it({
      name: 'throws a clear error when every location is out of stock',
      fn: async () => {
        using _t = withToken();
        using _mock = installQueue([
          outOfStock(),
          outOfStock(),
        ],);
        await expect(createWithFallback({
          image: 'ubuntu-24.04',
          fullName: 'mvm-x',
          locations: ['fsn1', 'nbg1',],
          rl: RL,
          serverType: 'cx22',
          sshKeyId: 1,
        },),).rejects.toThrow('no capacity',);
      },
    },),
  ],
},);
