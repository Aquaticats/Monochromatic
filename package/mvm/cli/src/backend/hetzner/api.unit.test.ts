/**
 Unit tests for the Hetzner API client against a stubbed `fetch`: request
 construction (bearer auth, label selector, body fields, DELETE path),
 label-scoped exact lookup, and the action-poll state machine. No real calls.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fetchAllPages,
  waitForAction,
} from '@monochromatic-dev/mvm-cli/ts/backend/hetzner/api.ts';
import {
  createServer,
  deleteServer,
  getMvmServerByName,
} from '@monochromatic-dev/mvm-cli/ts/backend/hetzner/api-resources.ts';

/**
 One recorded fetch invocation.
 */
type Call = {
  readonly url: string;
  readonly method: string;
  readonly body: unknown;
  readonly headers: unknown;
};

/**
 Fetch stub recorder: replaceable global fetch plus the recorded calls.
 */
type FetchMock = Disposable & { readonly calls: readonly Call[] };

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
 Builds a JSON Response for the stub.
 */
function jsonResponse(body: unknown, status = 200,): Response {
  return Response.json(body, { status, },);
}

/**
 Replaces global fetch with a handler, recording calls; restores on dispose.
 */
function installFetch(handler: (call: Call,) => Response,): FetchMock {
  const calls: Call[] = [];
  const original = globalThis.fetch;
  globalThis.fetch = (async function stubFetch(input: unknown, init?: RequestInit,) {
    const rawBody = init?.body;
    const call: Call = {
      url: String(input,),
      method: init?.method ?? 'GET',
      body: ((typeof rawBody) === 'string') ? JSON.parse(rawBody,) : undefined,
      headers: init?.headers,
    };
    calls.push(call,);
    return handler(call,);
  }) as unknown as typeof fetch;
  return {
    calls,
    [Symbol.dispose]() {
      globalThis.fetch = original;
    },
  };
}

/**
 Returns the first recorded call, throwing when none was made.
 */
function firstCall(mock: FetchMock,): Call {
  const [call] = mock.calls;
  if (call === undefined) {
    throw new Error('expected at least one fetch call',);
  }
  return call;
}

/**
 Representative server payload.
 */
const SERVER = {
  id: 1,
  name: 'mvm-dev',
  status: 'running',
  public_net: { ipv4: { ip: '203.0.113.7', }, },
  labels: { mvm: 'true', },
};

await describe({
  name: 'hetzner api client',
  concurrency: 1,
  children: [
    it({
      name: 'fetchAllPages follows next-page metadata and preserves item order',
      fn: async () => {
        using _t = withToken();
        using mock = installFetch(function paginatedResponse(call,) {
          const page = new URL(call.url,)
            .searchParams
            .get('page',);
          return page === '1'
            ? jsonResponse({
              servers: [SERVER,],
              meta: { pagination: { next_page: 2, }, },
            },)
            : jsonResponse({
              servers: [{ ...SERVER, id: 2, },],
              meta: { pagination: { next_page: null, }, },
            },);
        },);
        const servers = await fetchAllPages<typeof SERVER>({
          key: 'servers',
          path: '/servers',
        },);
        expect(servers.map(function serverId(server,) {
          return server.id;
        },),).toEqual([1, 2,],);
        expect(mock.calls.length,).toBe(2,);
      },
    },),
    it({
      name: 'getMvmServerByName sends bearer auth and label-scoped exact-name query',
      fn: async () => {
        using _t = withToken();
        using mock = installFetch(() =>
          jsonResponse({ servers: [SERVER,], meta: { pagination: { next_page: null, }, }, },)
        );
        const server = await getMvmServerByName({ name: 'dev', },);
        expect(server.id,).toBe(1,);
        const call = firstCall(mock,);
        expect(call.url,).toContain('label_selector=',);
        expect(call.url,).toContain('name=mvm-dev',);
        expect((call.headers as Record<string, string>).Authorization,).toBe('Bearer tok-test',);
      },
    },),
    it({
      name: 'getMvmServerByName throws when no server matches',
      fn: async () => {
        using _t = withToken();
        using _mock = installFetch(() =>
          jsonResponse({ servers: [], meta: { pagination: { next_page: null, }, }, },)
        );
        await expect(getMvmServerByName({ name: 'ghost', },),).rejects.toThrow('no mvm-managed',);
      },
    },),
    it({
      name: 'getMvmServerByName throws when multiple servers match',
      fn: async () => {
        using _t = withToken();
        using _mock = installFetch(() =>
          jsonResponse({
            servers: [SERVER, { ...SERVER, id: 2, },],
            meta: { pagination: { next_page: null, }, },
          },)
        );
        await expect(getMvmServerByName({ name: 'dev', },),).rejects.toThrow('ambiguous',);
      },
    },),
    it({
      name: 'createServer posts the expected body',
      fn: async () => {
        using _t = withToken();
        using mock = installFetch(() =>
          jsonResponse({ server: SERVER, action: { id: 9, status: 'running', }, },)
        );
        const result = await createServer({
          image: 'ubuntu-24.04',
          labels: { mvm: 'true', },
          location: 'fsn1',
          name: 'mvm-dev',
          serverType: 'cx22',
          sshKeyId: 42,
        },);
        expect(result.server.id,).toBe(1,);
        const call = firstCall(mock,);
        expect(call.method,).toBe('POST',);
        expect(call.url,).toContain('/servers',);
        const body = call.body as {
          server_type: string;
          ssh_keys: readonly number[];
          start_after_create: boolean;
          image: string;
        };
        expect(body.server_type,).toBe('cx22',);
        expect(body.ssh_keys,).toEqual([42,],);
        expect(body.start_after_create,).toBe(true,);
        expect(body.image,).toBe('ubuntu-24.04',);
      },
    },),
    it({
      name: 'waitForAction polls the generic action endpoint and resolves on success',
      fn: async () => {
        using _t = withToken();
        using mock = installFetch(() => jsonResponse({ action: { id: 9, status: 'success', }, },));
        await waitForAction({ id: 9, },);
        expect(firstCall(mock,).url,).toContain('/actions/9',);
      },
    },),
    it({
      name: 'waitForAction rejects with the action error message',
      fn: async () => {
        using _t = withToken();
        using _mock = installFetch(() =>
          jsonResponse({ action: { id: 9, status: 'error', error: { code: 'x', message: 'boom', }, }, },)
        );
        await expect(waitForAction({ id: 9, },),).rejects.toThrow('boom',);
      },
    },),
    it({
      name: 'deleteServer issues a DELETE and tolerates a 204 body',
      fn: async () => {
        using _t = withToken();
        using mock = installFetch(() => new Response(null, { status: 204, },));
        await deleteServer({ id: 7, },);
        const call = firstCall(mock,);
        expect(call.method,).toBe('DELETE',);
        expect(call.url,).toContain('/servers/7',);
      },
    },),
  ],
},);
