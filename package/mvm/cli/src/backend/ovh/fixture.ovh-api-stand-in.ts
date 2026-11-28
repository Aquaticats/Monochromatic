/**
 Stand-in for OVHcloud's cloud API and for this backend's environment, used by
 the ovh backend's unit tests.

 Nothing here is a recording or a capture. Every body is written by hand from
 OVHcloud's schema of that API (`https://ca.api.ovh.com/v1/cloud.json`, read on
 2026-11-28): the model named beside each builder is the schema model whose
 fields it carries, and only the fields this backend reads appear. No claim about
 what OVHcloud really answers rests on these shapes.

 `installStandIn` replaces `globalThis.fetch`, so a test that installs one cannot
 reach the network at all: it answers the clock path itself and hands every other
 request to the test's handler. Every key value here is a made-up string, and the
 project identifier is made up too.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  access,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  isAbsentPathError,
  type OvhFlavor,
  type OvhImage,
  type OvhInstance,
  type OvhInstanceState,
  type OvhQuotas,
  type OvhRegion,
  type OvhRegionService,
  type OvhRegionStatus,
  type OvhSshKey,
} from '../../../dist/final/node/index.mjs';

//region Constants

/**
 Made-up project identifier every stand-in path carries.
 */
export const STAND_IN_PROJECT = 'mvm-stand-in-project';

/**
 Made-up application key.
 */
export const STAND_IN_APPLICATION_KEY = 'standin-ak-0000-not-a-real-key';

/**
 Made-up application secret.
 */
export const STAND_IN_APPLICATION_SECRET = 'standin-as-1111-not-a-real-secret';

/**
 Made-up consumer key.
 */
export const STAND_IN_CONSUMER_KEY = 'standin-ck-2222-not-a-real-key';

/**
 Unix seconds the stand-in says the API host's clock reads.
 */
export const STAND_IN_SERVER_TIME = 1_800_000_000;

/**
 Public key line the environment fixture writes, so no test has to run
 `ssh-keygen`: this host confines that program to `~/.ssh`, and the key's content
 is irrelevant to everything below the SSH layer.
 */
export const STAND_IN_PUBLIC_KEY = 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMVMstandinkey mvm-stand-in';

/**
 File the fake `ssh` appends its arguments to, one JSON array per line.
 */
export const SSH_CALLS_FILE = 'ssh-calls.jsonl';

/**
 Source of the stand-in `ssh`.
 */
export const FAKE_SSH_PATH: string = join(
  import.meta.dirname,
  'fixture.fake-ovh-ssh.ts',
);

/**
 Path of the clock endpoint the stand-in answers itself.
 */
export const AUTH_TIME_PATH = '/1.0/auth/time';

//endregion Constants

//region Shapes

/**
 One request the stand-in answered.
 */
export type RecordedCall = {
  /**
   Complete URL requested.
   */
  readonly url: string;
  /**
   Path and query, with the API root removed.
   */
  readonly path: string;
  /**
   HTTP method.
   */
  readonly method: string;
  /**
   Decoded JSON body, when the request carried one.
   */
  readonly body: unknown;
  /**
   Headers the request carried.
   */
  readonly headers: Readonly<Record<string, string>>;
};

/**
 Answers one request.
 */
export type Handler = (call: RecordedCall,) => Response;

/**
 Stand-in fetch plus everything it was asked.
 */
export type StandIn = Disposable & {
  readonly calls: readonly RecordedCall[];
};

//endregion Shapes

//region Requests

/**
 The headers of one request, whatever form the caller gave them in.
 
 @param init - the request's init, when it had one
 
 @returns the headers as a record
 
 @example
 ```ts
 headersOf({ headers: { 'X-Ovh-Application': 'ak' } });
 ```
 */
function headersOf(init?: RequestInit,): Readonly<Record<string, string>> {
  /**
   Headers in whatever form fetch accepts.
   */
  const raw = init?.headers;
  if (raw === undefined) {
    return {};
  }
  if (raw instanceof Headers) {
    return Object.fromEntries(raw.entries(),);
  }
  if (Array.isArray(raw,)) {
    return Object.fromEntries(raw,);
  }
  return raw;
}

/**
 Builds a project-scoped path, the form the backend must send.
 
 @param suffix - resource path below the project, beginning with `/`
 
 @returns path from the API root
 
 @example
 ```ts
 cloudPath('/instance'); // '/v1/cloud/project/mvm-stand-in-project/instance'
 ```
 */
export function cloudPath(suffix: string,): string {
  return `/v1/cloud/project/${STAND_IN_PROJECT}${suffix}`;
}

/**
 Builds a JSON answer with status 200.
 
 @param body - answer body
 
 @returns a response carrying `body`
 
 @example
 ```ts
 jsonAnswer({ id: 'instance-1' });
 ```
 */
export function jsonAnswer(body: unknown,): Response {
  return Response.json(body,);
}

/**
 Builds OVHcloud's failure envelope, the shape its own client reads
 (`github.com/ovh/python-ovh`, `ovh/client.py` lines 516-545 of the clone read on
 2026-11-28: a decoded body with `message` and `errorCode`).
 
 @param message - OVHcloud's own message
 
 @param status - HTTP status
 
 @param errorCode - machine-readable code
 
 @returns a response carrying the failure envelope
 
 @example
 ```ts
 failureAnswer({ message: 'This flavor is not available', status: 400 });
 ```
 */
export function failureAnswer({
  errorCode = 'INVALID_PARAMETER',
  message,
  status,
}: {
  readonly errorCode?: string;
  readonly message: string;
  readonly status: number;
},): Response {
  return Response.json(
    {
      errorCode,
      message,
    },
    { status, },
  );
}

/**
 Replaces `globalThis.fetch` with a handler, answering the clock path itself so
 no test has to, and recording every call.
 
 @param handler - answers every request except the clock read
 
 @returns the recorded calls, and a dispose that restores fetch
 
 @example
 ```ts
 using standIn = installStandIn(() => jsonAnswer([]));
 ```
 */
export function installStandIn(handler: Handler,): StandIn {
  return install({
    answerClock: true,
    handler,
  },);
}

/**
 Replaces `globalThis.fetch` with a handler that answers everything itself,
 including the clock path, and records every call. For the tests of what happens
 when the clock cannot be read.
 
 @param handler - answers every request, the clock read included
 
 @returns the recorded calls, and a dispose that restores fetch
 
 @example
 ```ts
 using standIn = installRawStandIn(() => new Response('no clock', { status: 503 }));
 ```
 */
export function installRawStandIn(handler: Handler,): StandIn {
  return install({
    answerClock: false,
    handler,
  },);
}

/**
 The shared body of the two installers.
 
 @param answerClock - whether the clock path is answered here
 
 @param handler - answers the requests the clock answer does not cover
 
 @returns the recorded calls, and a dispose that restores fetch
 
 @example
 ```ts
 using standIn = install({ answerClock: true, handler });
 ```
 */
function install({
  answerClock,
  handler,
}: {
  readonly answerClock: boolean;
  readonly handler: Handler;
},): StandIn {
  /**
   Every request answered, in order.
   */
  const calls: RecordedCall[] = [];
  /**
   The fetch that was there before, restored on dispose.
   */
  const original = globalThis.fetch;
  globalThis.fetch = function standInFetch(
    input: URL | RequestInfo,
    init?: RequestInit,
  ): Promise<Response> {
    /**
     Requested URL, as the backend built it.
     */
    const url = (function urlOf(): string {
      if ((typeof input) === 'string') {
        return input;
      }
      return (input instanceof URL) ? input.href : input.url;
    })();
    /**
     Path and query, with the API root removed.
     */
    const path = (function pathOf(): string {
      /**
       Index where the path begins after the scheme and host.
       */
      const start = url.indexOf(
        '/',
        url.indexOf('//',) + 2,
      );
      return (start === (-1)) ? url : url.slice(start,);
    })();
    /**
     Raw request body, decoded below when it is JSON text.
     */
    const rawBody = init?.body;
    /**
     The recording this request becomes.
     */
    const call: RecordedCall = {
      url,
      path,
      method: init?.method ?? 'GET',
      body: ((typeof rawBody) === 'string') ? JSON.parse(rawBody,) : undefined,
      headers: headersOf(init,),
    };
    calls.push(call,);
    if (answerClock && (path === AUTH_TIME_PATH)) {
      return Promise.resolve(jsonAnswer(STAND_IN_SERVER_TIME,),);
    }
    return Promise.resolve(handler(call,),);
  };
  return {
    calls,
    [Symbol.dispose]() {
      globalThis.fetch = original;
    },
  };
}

/**
 Replaces fetch with a scripted series of answers, in order, so a test reads the
 requests the backend made and the answers it was given.
 
 @param answers - responses to hand out in order
 
 @returns the recorded calls, and a dispose that restores fetch
 
 @example
 ```ts
 using standIn = scriptedStandIn([jsonAnswer([]), jsonAnswer([])]);
 ```
 */
export function scriptedStandIn(answers: readonly Response[],): StandIn {
  /**
   Answers not yet handed out.
   */
  const queue = [...answers,];
  return installStandIn(function next() {
    return queue.shift() ?? failureAnswer({
      message: 'the stand-in was asked for one answer more than it holds',
      status: 500,
    },);
  },);
}

/**
 The requests recorded so far, filtered to one method and path.
 
 @param calls - recorded calls
 
 @param method - HTTP method to keep
 
 @param path - path to keep
 
 @returns matching calls in order
 
 @example
 ```ts
 matching({ calls, method: 'DELETE', path: cloudPath('/instance/instance-1') });
 ```
 */
export function matching({
  calls,
  method,
  path,
}: {
  readonly calls: readonly RecordedCall[];
  readonly method: string;
  readonly path: string;
},): readonly RecordedCall[] {
  return calls.filter(function same(call,) {
    return (call.method === method) && (call.path === path);
  },);
}

//endregion Requests

//region Environment

/**
 Sets one variable for a scope, restoring what it held before.
 
 @param name - variable to set
 
 @param value - value to set, or omitted to unset it
 
 @returns a dispose that restores the variable
 
 @example
 ```ts
 using _unset = withEnv({ name: 'OVH_APPLICATION_KEY' });
 ```
 */
export function withEnv({
  name,
  value,
}: {
  readonly name: string;
  readonly value?: string;
},): Disposable {
  /**
   Value the variable held before this scope.
   */
  const previous = process.env[name];
  if (value === undefined) {
    Reflect.deleteProperty(
      process.env,
      name,
    );
  }
  else {
    process.env[name] = value;
  }
  return {
    [Symbol.dispose]() {
      if (previous === undefined) {
        Reflect.deleteProperty(
          process.env,
          name,
        );
      }
      else {
        process.env[name] = previous;
      }
    },
  };
}

/**
 A test's private environment: the five names, a disposable data directory, and
 the stand-in `ssh`.
 */
export type OvhTestEnv = AsyncDisposable & {
  /**
   Directory holding the managed keypair and the fake ssh's recorded calls.
   */
  readonly dir: string;
};

/**
 Sets the backend's environment for a test scope, restoring every variable and
 removing the directory afterwards.
 
 @param sshCommand - command `ssh` runs through, the stand-in by default
 
 @returns the private directory, and a dispose that restores the environment
 
 @example
 ```ts
 await using env = await installEnv();
 ```
 */
export async function installEnv({
  sshCommand,
}: {
  readonly sshCommand?: readonly string[];
} = {},): Promise<OvhTestEnv> {
  /**
   Directory this test owns; the keypair and the call log land here.
   */
  const dir = await mkdtemp(join(
    scratchRoot(),
    'ovh-test-',
  ),);
  /**
   Variables that held a value before this scope, restored on dispose.
   */
  const restore = new Map<string, string>();
  /**
   Variables that were unset before this scope, unset again on dispose.
   */
  const remove = new Set<string>();
  /**
   Variables this fixture owns, in a fixed order.
   */
  const values: Readonly<Record<string, string>> = {
    OVH_ENDPOINT: 'ovh-ca',
    OVH_APPLICATION_KEY: STAND_IN_APPLICATION_KEY,
    OVH_APPLICATION_SECRET: STAND_IN_APPLICATION_SECRET,
    OVH_CONSUMER_KEY: STAND_IN_CONSUMER_KEY,
    MVM_OVH_PROJECT_ID: STAND_IN_PROJECT,
    MVM_OVH_DATA_DIR: dir,
    MVM_OVH_SSH_COMMAND: JSON.stringify(sshCommand ?? [
      process.execPath,
      FAKE_SSH_PATH,
    ],),
    MVM_FAKE_OVH_SSH_CALLS: join(
      dir,
      SSH_CALLS_FILE,
    ),
    MVM_FAKE_OVH_SSH_EXIT: '0',
  };
  for (const [name, value,] of Object.entries(values,)) {
    /**
     Value the variable held before this scope, when it held one.
     */
    const before = process.env[name];
    if (before === undefined) {
      remove.add(name,);
    }
    else {
      restore.set(
        name,
        before,
      );
    }
    process.env[name] = value;
  }
  for (const name of [
    'MVM_OVH_REGIONS',
    'MVM_OVH_FLAVOR',
    'MVM_OVH_SCP_COMMAND',
  ]) {
    remove.add(name,);
    Reflect.deleteProperty(
      process.env,
      name,
    );
  }
  await writeFile(
    join(
      dir,
      'id_ed25519',
    ),
    'stand-in private key; no ssh ever reads it\n',
    { mode: 0o600, },
  );
  await writeFile(
    join(
      dir,
      'id_ed25519.pub',
    ),
    `${STAND_IN_PUBLIC_KEY}\n`,
  );
  return {
    dir,
    async [Symbol.asyncDispose](): Promise<void> {
      for (const [name, value,] of restore) {
        process.env[name] = value;
      }
      for (const name of remove) {
        Reflect.deleteProperty(
          process.env,
          name,
        );
      }
      await rm(
        dir,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 The arguments the stand-in `ssh` was run with, one array per call.
 
 @param dir - directory the environment fixture made
 
 @returns each call's arguments, oldest first
 
 @example
 ```ts
 const calls = await sshCalls({ dir: env.dir });
 ```
 */
export async function sshCalls({
  dir,
}: {
  readonly dir: string;
},): Promise<readonly (readonly string[])[]> {
  /**
   Path of the log the stand-in `ssh` writes.
   */
  const logPath = join(
    dir,
    SSH_CALLS_FILE,
  );
  try {
    await access(logPath,);
  }
  catch (error: unknown) {
    if (isAbsentPathError(error,)) {
      return [];
    }
    throw error;
  }
  /**
   The log, one JSON array per line.
   */
  const text = await readFile(
    logPath,
    'utf8',
  );
  return text.trim()
    .split('\n',)
    .filter(function notEmpty(line,) {
      return line !== '';
    },)
    .map(function decode(line,) {
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- lines this fixture's own fake ssh wrote
      return JSON.parse(line,) as readonly string[];
    },);
}

//endregion Environment

//region Failures

/**
 Runs something expected to throw and returns the text of what it threw.
 
 @param run - the call expected to fail
 
 @returns the thrown value's message, `''` when the call did not throw
 
 @example
 ```ts
 const message = throwMessage({ run: () => sshCommand() });
 ```
 */
export function throwMessage({
  run,
}: {
  readonly run: () => unknown;
},): string {
  try {
    run();
  }
  catch (error: unknown) {
    return caughtValueText(error,);
  }
  return '';
}

/**
 Runs something expected to throw and returns what it threw.
 
 @param run - the call expected to fail
 
 @returns the thrown value
 
 @throws Error when the call resolved instead of throwing
 
 @example
 ```ts
 const error = await errorFrom({ run: () => ovhList() });
 ```
 */
export async function errorFrom({
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

//endregion Failures

//region Bodies

/**
 An instance body, from the schema's `cloud.instance.Instance`.
 
 @param flavorId - identifier of the flavor the instance runs
 
 @param id - instance identifier
 
 @param imageUser - login account the carried image names
 
 @param ip - public IPv4 the instance holds, `''` for none
 
 @param name - instance name
 
 @param region - region the instance runs in
 
 @param status - lifecycle status
 
 @param withImage - whether the body carries its image, as the API sometimes omits it
 
 @returns the body, carrying only fields this backend reads
 
 @example
 ```ts
 instanceAnswer({ name: 'mvm-dev', status: 'ACTIVE' });
 ```
 */
export function instanceAnswer({
  flavorId = 'flavor-small',
  id = 'instance-1',
  imageUser = 'ubuntu',
  ip = '192.0.2.10',
  name = 'mvm-dev',
  region = 'bhs5',
  status = 'ACTIVE',
  withImage = true,
}: {
  readonly flavorId?: string;
  readonly id?: string;
  readonly imageUser?: string;
  readonly ip?: string;
  readonly name?: string;
  readonly region?: string;
  readonly status?: OvhInstanceState;
  readonly withImage?: boolean;
} = {},): OvhInstance {
  return {
    id,
    name,
    region,
    status,
    flavorId,
    imageId: 'image-ubuntu',
    ...(withImage ? { image: imageAnswer({ user: imageUser, },), } : {}),
    ipAddresses: (ip === '')
      ? []
      : [
        {
          ip,
          version: 4,
          type: 'public',
        },
      ],
  };
}

/**
 An image body, from the schema's `cloud.image.Image`.
 
 @param creationDate - creation timestamp
 
 @param id - image identifier
 
 @param name - catalogue name
 
 @param status - image status
 
 @param user - login account the image names
 
 @param visibility - who can see the image
 
 @returns the body, carrying only fields this backend reads
 
 @example
 ```ts
 imageAnswer({ name: 'Ubuntu 24.04' });
 ```
 */
export function imageAnswer({
  creationDate = '2026-01-01T00:00:00Z',
  flavorType,
  id = 'image-ubuntu',
  minDisk = 10,
  minRam = 1_024,
  name = 'Ubuntu 24.04',
  status = 'active',
  tags,
  user = 'ubuntu',
  visibility = 'public',
}: {
  readonly creationDate?: string;
  readonly flavorType?: string;
  readonly id?: string;
  readonly minDisk?: number;
  readonly minRam?: number;
  readonly name?: string;
  readonly status?: string;
  readonly tags?: readonly string[];
  readonly user?: string;
  readonly visibility?: string;
} = {},): OvhImage {
  return {
    id,
    name,
    region: 'bhs5',
    type: 'image',
    status,
    visibility,
    user,
    creationDate,
    size: 2.5,
    minDisk,
    minRam,
    ...((flavorType === undefined) ? {} : { flavorType, }),
    ...((tags === undefined) ? {} : { tags, }),
  };
}

/**
 A flavor body, from the schema's `cloud.flavor.Flavor`.
 
 @param available - whether the flavor is in stock
 
 @param id - flavor identifier
 
 @param osType - operating system family
 
 @param ram - memory in GiB
 
 @param region - region the flavor is offered in
 
 @param vcpus - virtual CPUs
 
 @returns the body, carrying only fields this backend reads
 
 @example
 ```ts
 flavorAnswer({ id: 'b3-8', vcpus: 2, ram: 8 });
 ```
 */
export function flavorAnswer({
  available = true,
  disk = 20,
  id = 'flavor-small',
  name = id,
  osType = 'linux',
  ram = 2,
  region = 'bhs5',
  vcpus = 1,
}: {
  readonly available?: boolean;
  readonly disk?: number;
  readonly id?: string;
  readonly name?: string;
  readonly osType?: string;
  readonly ram?: number;
  readonly region?: string;
  readonly vcpus?: number;
} = {},): OvhFlavor {
  return {
    id,
    name,
    region,
    vcpus,
    ram,
    disk,
    osType,
    type: 'general',
    available,
    quota: 10,
  };
}

/**
 A region body, from the schema's `cloud.Region`.
 
 @param countryCode - ISO 3166-1 alpha-2 country code, lowercase
 
 @param name - region name
 
 @param services - services the region offers; the default is one that creates instances
 
 @param status - region status
 
 @returns the body, carrying only fields this backend reads
 
 @example
 ```ts
 regionAnswer({ name: 'bhs5', countryCode: 'ca' });
 ```
 */
export function regionAnswer({
  countryCode = 'ca',
  name = 'bhs5',
  services = [
    {
      endpoint: 'https://compute.stand-in.invalid/',
      name: 'instance',
      status: 'UP',
    },
  ],
  status = 'UP',
}: {
  readonly countryCode?: string;
  readonly name?: string;
  readonly services?: readonly OvhRegionService[];
  readonly status?: OvhRegionStatus;
} = {},): OvhRegion {
  return {
    name,
    countryCode,
    continentCode: 'NA',
    status,
    type: 'region',
    datacenterLocation: 'Beauharnois',
    availabilityZones: ['bhs5-a',],
    services,
  };
}

/**
 An SSH key body, from the schema's `cloud.sshkey.SshKey`.
 
 @param id - key identifier
 
 @param name - key name
 
 @param publicKey - the whole public key line
 
 @returns the body, carrying only fields this backend reads
 
 @example
 ```ts
 sshKeyAnswer({ id: 'key-1', publicKey: 'ssh-ed25519 AAAA mvm' });
 ```
 */
export function sshKeyAnswer({
  id = 'key-1',
  name = 'mvm-test',
  publicKey = 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITESTKEYmvm test',
}: {
  readonly id?: string;
  readonly name?: string;
  readonly publicKey?: string;
} = {},): OvhSshKey {
  return {
    id,
    name,
    publicKey,
    regions: ['bhs5',],
  };
}

/**
 A quota body, from the schema's `cloud.quota.Quotas`.
 
 @param region - region the quota applies to
 
 @param usedInstances - instances the project holds there
 
 @param maxInstances - instances the project may hold there
 
 @returns the body, carrying only fields this backend reads
 
 @example
 ```ts
 quotaAnswer({ region: 'bhs5' });
 ```
 */
export function quotaAnswer({
  maxInstances = 10,
  region = 'bhs5',
  usedInstances = 1,
}: {
  readonly maxInstances?: number;
  readonly region?: string;
  readonly usedInstances?: number;
} = {},): OvhQuotas {
  return {
    region,
    instance: {
      maxInstances,
      usedInstances,
      maxCores: 40,
      usedCores: 2,
      maxRam: 102_400,
      usedRAM: 4_096,
    },
  };
}

//endregion Bodies
