/**
 Unit tests for the hetzner backend's one-shot run against a stand-in HTTP layer:
 a run that asks for two cores and 4 GB creates the smallest server type that
 meets both, prints that type before it creates, and destroys the server its own
 create made; a run that asks for a floor beside a source machine to clone is
 refused before anything is read.

 Nothing here is a recording or a capture: every body is written by hand from
 Hetzner's own resource shapes, as the other Hetzner tests in this package do.
 No request leaves this process, because `globalThis.fetch` answers every one of
 them, and no `ssh` runs, because the created server carries no public IPv4 and
 the run stops before it probes one.

 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  DATA_DIR_ENV,
  hetznerRun,
} from '../../../dist/final/node/index.mjs';

//region Constants

/**
 How many cores the run under test asks its machine to have.
 */
const ASKED_CORES = 2;

/**
 How much memory the run under test asks its machine to have, in gigabytes.
 */
const ASKED_MEMORY_GB = 4;

/**
 Type the floors select: the smallest offered that has both.
 */
const FITTING_TYPE = 'cx23';

/**
 Cheapest offered type, which is what a run gets when the floors never reach
 the create, and which has neither enough cores nor enough memory.
 */
const CHEAPEST_TYPE = 'cx11';

/**
 Command words the run hands its guest, as one string the guest's shell reads.
 */
const GUEST_COMMAND = 'uname -a';

/**
 Public key line this stand-in's project holds, so the managed keypair matches a
 key already uploaded and the run uploads nothing.
 */
const STAND_IN_PUBLIC_KEY =
  'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMVMhetznerfixturehetznerfixturefixture mvm';

/**
 Server type bodies the stand-in catalogue holds: the cheapest type first in
 price but too small for the floors, and the type that meets them.
 */
const SERVER_TYPES = [
  {
    name: CHEAPEST_TYPE,
    architecture: 'x86',
    cores: 1,
    cpu_type: 'shared',
    deprecation: null,
    disk: 25,
    memory: 2,
    prices: [{ location: 'fsn1', price_hourly: { gross: '0.0049', }, },],
  },
  {
    name: FITTING_TYPE,
    architecture: 'x86',
    cores: ASKED_CORES,
    cpu_type: 'shared',
    deprecation: null,
    disk: 40,
    memory: ASKED_MEMORY_GB,
    prices: [{ location: 'fsn1', price_hourly: { gross: '0.0080', }, },],
  },
] as const;

/**
 The image body `GET /images?type=system` answers with, shaped as Hetzner's
 `image` resource carries the fields this backend reads.
 */
const SYSTEM_IMAGES = [
  {
    id: 3,
    name: 'ubuntu-24.04',
    type: 'system',
    os_flavor: 'ubuntu',
    deprecated: null,
    created: '2026-01-01T00:00:00+00:00',
    description: 'Ubuntu 24.04',
  },
] as const;

/**
 Identifier of the server the stand-in's create makes.
 */
const SERVER_ID = 4_242;

/**
 Identifier of the boot action the stand-in reports, already finished so the
 wait returns at once.
 */
const ACTION_ID = 9_191;

//endregion Constants

//region Stand-in layers

/**
 Sets a variable for a scope, restoring what it held before.
 
 @param name - variable to set
 
 @param value - value to set
 
 @returns a dispose that restores the variable

 @example
 ```ts
 using _token = withEnv({ name: 'HCLOUD_TOKEN', value: 'tok-test' });
 ```
 */
function withEnv({
  name,
  value,
}: {
  readonly name: string;
  readonly value: string;
},): Disposable {
  /**
   Value the variable held before this scope.
   */
  const prior = process.env[name];
  process.env[name] = value;
  return {
    [Symbol.dispose]() {
      if (prior === undefined) {
        delete process.env[name];
      }
      else {
        process.env[name] = prior;
      }
    },
  };
}

/**
 One request the stand-in answered.
 */
type Call = {
  /**
   Complete URL requested.
   */
  readonly url: string;
  /**
   HTTP method, `GET` when the request carried none.
   */
  readonly method: string;
  /**
   Decoded JSON body, when the request carried one.
   */
  readonly body: unknown;
};

/**
 One page of a list answer, carrying the pagination record the API's reader
 requires: a page with no `next_page` is the last one.

 @param key - resource key the page lists under

 @param items - resources on this page

 @returns the answer body

 @example
 ```ts
 page('servers', [server]);
 ```
 */
function page(key: string, items: readonly unknown[],): Record<string, unknown> {
  return {
    [key]: items,
    meta: { pagination: { next_page: null, }, },
  };
}

/**
 Replaces fetch with an answer per resource path, recording every request, and
 holds the server its create made so the lookup by name finds it.
 
 The server a run makes carries a random name, so a fixed table of answers
 cannot answer the lookup before the delete; this stand-in reads the name out of
 the create's own body.
 
 @returns the recorded calls and a dispose that restores fetch

 @example
 ```ts
 using standIn = installStandIn();
 ```
 */
function installStandIn(): Disposable & { readonly calls: Call[]; } {
  /**
   Requests answered so far, in order.
   */
  const calls: Call[] = [];
  /**
   The server this run created, while it exists.
   */
  let created: Record<string, unknown> | undefined;
  /**
   Whether the delete of {@link created} was accepted.
   */
  let deleted = false;
  const original = globalThis.fetch;
  globalThis.fetch = (async function standIn(input: unknown, init?: RequestInit,) {
    /**
     Raw request body, when the request carried one.
     */
    const rawBody = init?.body;
    /**
     This request, as the test reads it.
     */
    const call: Call = {
      url: String(input,),
      method: init?.method ?? 'GET',
      body: ((typeof rawBody) === 'string') ? JSON.parse(rawBody,) : undefined,
    };
    calls.push(call,);
    /**
     Path and query of this request, below the API's `/v1` root.
     */
    const path = `/${call.url.split('/v1/',)[1] ?? ''}`;
    if ((call.method === 'GET') && path.startsWith('/ssh_keys',)) {
      return Response.json(page('ssh_keys', [{
        id: 11,
        name: 'mvm-standin',
        fingerprint: 'aa:bb',
        public_key: STAND_IN_PUBLIC_KEY,
        labels: {},
        created: '2026-01-01T00:00:00+00:00',
      },],),);
    }
    if ((call.method === 'GET') && path.startsWith('/images',)) {
      return Response.json(page('images', SYSTEM_IMAGES,),);
    }
    if ((call.method === 'GET') && path.startsWith('/server_types',)) {
      return Response.json(page('server_types', SERVER_TYPES,),);
    }
    if ((call.method === 'POST') && (path === '/servers')) {
      /**
       Body of the create, which names the machine and the type taken.
       */
      const body = call.body as {
        readonly location: string;
        readonly name: string;
        readonly server_type: string;
      };
      created = {
        id: SERVER_ID,
        name: body.name,
        status: 'running',
        labels: {},
        public_net: { ipv4: null, },
        server_type: SERVER_TYPES.find(function sameName(type,) {
          return type.name === body.server_type;
        },),
      };
      return Response.json({
        server: created,
        action: { id: ACTION_ID, status: 'running', },
      },);
    }
    if ((call.method === 'GET') && path.startsWith(`/actions/${String(ACTION_ID,)}`,)) {
      return Response.json({ action: { id: ACTION_ID, status: 'success', }, },);
    }
    if ((call.method === 'GET') && path.startsWith('/servers',)) {
      return Response.json(
        page('servers', ((created === undefined) || deleted) ? [] : [created,],),
      );
    }
    if ((call.method === 'DELETE') && path.startsWith('/servers/',)) {
      deleted = true;
      return new Response(null, { status: 204, },);
    }
    return Response.json(
      { error: { code: 'unplanned', message: `the stand-in planned no answer for ${call.method} ${path}`, }, },
      { status: 500, },
    );
  }) as unknown as typeof fetch;
  return {
    calls,
    [Symbol.dispose]() {
      globalThis.fetch = original;
    },
  };
}

/**
 Creates a directory holding a managed keypair, so no `ssh-keygen` runs, and
 points {@link DATA_DIR_ENV} at it for the life of the returned record.
 
 @returns the directory and a dispose that restores the variable

 @example
 ```ts
 await using dataDir = await installDataDir();
 ```
 */
async function installDataDir(): Promise<AsyncDisposable & { readonly dir: string; }> {
  /**
   Directory this test owns; the keypair the backend signs with lands here.
   */
  const dir = await mkdtemp(join(
    scratchRoot(),
    'hetzner-run-',
  ),);
  await writeFile(
    join(dir, 'id_ed25519',),
    'fixture private key, never used: no ssh runs in this test\n',
  );
  await writeFile(
    join(dir, 'id_ed25519.pub',),
    `${STAND_IN_PUBLIC_KEY}\n`,
  );
  /**
   Variable restore for {@link DATA_DIR_ENV}.
   */
  const restore = withEnv({
    name: DATA_DIR_ENV,
    value: dir,
  },);
  return {
    dir,
    async [Symbol.asyncDispose]() {
      restore[Symbol.dispose]();
      await rm(dir, {
        force: true,
        recursive: true,
      },);
    },
  };
}

/**
 Everything a console or stream write carried while a run was in flight, in the
 order it was written.
 
 @param stubs - the recording stubs a case installed
 
 @returns one text per write, joined with newlines

 @example
 ```ts
 const printed = printedText(stubs);
 ```
 */
function printedText(
  stubs: readonly { readonly args: readonly unknown[][]; }[],
): string {
  return stubs
    .flatMap(function argLists(stub,) {
      return stub.args;
    },)
    .map(function asText(args,) {
      return args.map(String,)
        .join(' ',);
    },)
    .join('\n',);
}

//endregion Stand-in layers

await describe({
  name: 'hetzner one-shot run',
  concurrency: 1,
  children: [
    it({
      name: `a run asking for ${String(ASKED_CORES,)} cores and ${String(ASKED_MEMORY_GB,)} GB creates ${FITTING_TYPE} and prints that type before it creates`,
      fn: async ctx => {
        await using dataDir = await installDataDir();
        using _token = withEnv({
          name: 'HCLOUD_TOKEN',
          value: 'tok-test',
        },);
        using standIn = installStandIn();
        /**
         Everything written to a console method or to standard error while the run
         is in flight, which is where a size line reaches a person.
         */
        const info = ctx.sinon.stub(console, 'info',);
        const log = ctx.sinon.stub(console, 'log',);
        const warn = ctx.sinon.stub(console, 'warn',);
        const error = ctx.sinon.stub(console, 'error',);
        const stderr = ctx.sinon.stub(process.stderr, 'write',);
        stderr.returns(true,);
        /**
         The failure the run ended with, which is the stand-in's server having no
         public address to reach.
         */
        let caught: unknown;
        try {
          await hetznerRun({
            command: GUEST_COMMAND,
            memoryGiB: ASKED_MEMORY_GB,
            vcpus: ASKED_CORES,
          },);
        }
        catch (thrown: unknown) {
          caught = thrown;
        }
        // The run stops before it probes an address, so no ssh process starts.
        expect(caughtValueText(caught,),).toContain('no public IPv4',);

        /**
         The create request the run made, which carries the type it took.
         */
        const created = standIn.calls.find(function createServer(call,) {
          return (call.method === 'POST') && call.url.endsWith('/servers',);
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        // The floors reached the create: the type taken meets both.
        expect((created.body as { readonly server_type: string; }).server_type,).toBe(FITTING_TYPE,);

        /**
         Everything the run printed.
         */
        const printed = printedText([
          info,
          log,
          warn,
          error,
          stderr,
        ],);
        /**
         Index of the line naming the type the walk settled on.
         */
        const sizeLine = printed.indexOf(`fits ${FITTING_TYPE} (`);
        /**
         Index of the line naming the create of the machine.
         */
        const createLine = printed.indexOf('creating mvm-ephemeral-',);
        expect(sizeLine,).toBeGreaterThan(-1,);
        expect(createLine,).toBeGreaterThan(-1,);
        // The size reaches a person before the create it justifies does.
        expect(sizeLine,).toBeLessThan(createLine,);

        // The machine its own create made was destroyed, so the run leaves nothing billed.
        expect(
          standIn.calls.filter(function deleteServer(call,) {
            return (call.method === 'DELETE') && call.url.endsWith(`/servers/${String(SERVER_ID,)}`,);
          },).length,
        ).toBe(1,);
      },
    },),
    it({
      name: 'a run naming a source machine and a floor is refused before anything is read',
      fn: async () => {
        await using _dataDir = await installDataDir();
        using _token = withEnv({
          name: 'HCLOUD_TOKEN',
          value: 'tok-test',
        },);
        using standIn = installStandIn();
        /**
         The refusal this call threw, when it threw one.
         */
        let caught: unknown;
        try {
          await hetznerRun({
            command: GUEST_COMMAND,
            from: 'dev-01',
            vcpus: ASKED_CORES,
          },);
        }
        catch (thrown: unknown) {
          caught = thrown;
        }
        /**
         What the refusal says: the source machine it cannot resize and the floor it
         cannot honour.
         */
        const message = caughtValueText(caught,);
        expect(message,).toContain('at least 2 vCPUs',);
        expect(message,).toContain('keeps');
        expect(standIn.calls,).toHaveLength(0,);
      },
    },),
  ],
},);
