/**
 The run's private PipeWire null sink, and the proof that the app plays into it and nowhere else.
 
 The null sink is created by this run under a name that carries the run's process id. Every audio check asks PipeWire
 for the app's output streams and the links that leave them; a link to any node other than the null sink ends the run
 at once with a safety stop.
 
 @example
 ```ts
 import { createNullSink, } from './pipewire.ts';
 
 await using sink = await createNullSink({ name: 'music_player_e2e_1', },);
 ```
 
 @packageDocumentation
 */

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { runCommand, } from './command.ts';
import {
  CheckFailureError,
  expect,
  SafetyStopError,
} from './errors.ts';
import { pollUntil, } from './poll.ts';

/**
 Node id reported while no node matches.
 */
export const NO_NODE = -1;

/**
 How many times the sink node is looked for after the module is loaded.
 */
const SINK_LOOKUPS = 40;

/**
 Pause between two lookups of the sink node.
 */
const SINK_LOOKUP_PAUSE_MS = 25;

/**
 Object properties of one PipeWire node or client, as `pw-dump` prints them.
 */
export type PwProps = Readonly<Record<string, boolean | number | string>>;

/**
 The part of a `pw-dump` object this run reads.
 */
export type PwObject = {
  readonly id: number;
  readonly info?: {
    readonly 'input-node-id'?: number;
    readonly 'output-node-id'?: number;
    readonly props?: PwProps;
  };
  readonly type?: string;
};

/**
 Streams and links found for the app, and a sentence that says so.
 */
export type Routing = {
  readonly detail: string;
  readonly linked: number;
};

/**
 The private null sink: its name, its PipeWire node and the module that made it.
 */
export type NullSink = {
  readonly [Symbol.asyncDispose]: () => Promise<void>;
  readonly moduleId: string;
  readonly name: string;
  readonly nodeId: number;
};

/**
 Narrows a decoded `pw-dump` entry to an object with a numeric id. The other fields are read defensively by the
 functions below, which compare them to expected strings and numbers.
 
 @param value - one entry of the decoded dump
 
 @returns whether the entry has the shape of a PipeWire object
 
 @example
 ```ts
 isPwObject({ id: 3, },); // true
 ```
 */
function isPwObject(value: unknown,): value is PwObject {
  return ((typeof value) === 'object') && (value !== null)
    && ('id' in value)
    && ((typeof value.id) === 'number');
}

/**
 Decodes the JSON that `pw-dump` printed.
 
 @param stdout - the text `pw-dump` printed; empty when it failed
 
 @returns the objects of the dump; none for empty or malformed text
 
 @example
 ```ts
 parsePwDump('[{"id":3}]',); // [{ id: 3 }]
 ```
 */
export function parsePwDump(stdout: string,): readonly PwObject[] {
  /**
   Tagged logger naming the dump reader.
   */
  const rl = tagged({
    tag: parsePwDump.name,
    l: logger,
  },);
  try {
    /**
     Decoded JSON of the dump.
     */
    const decoded: unknown = JSON.parse(stdout === '' ? '[]' : stdout,);
    return Array.isArray(decoded,) ? decoded.filter(isPwObject,) : [];
  }
  catch (error: unknown) {
    rl.debug(`the dump is not JSON: ${String(error,)}`,);
    return [];
  }
}

/**
 Asks PipeWire for every object it knows.
 
 @returns the objects of the current dump; none when `pw-dump` fails
 
 @example
 ```ts
 const objects = await dumpPipeWire();
 ```
 */
export async function dumpPipeWire(): Promise<readonly PwObject[]> {
  /**
   Output of `pw-dump`.
   */
  const { stdout, } = await runCommand({
    command: 'pw-dump',
    args: [],
  },);
  return parsePwDump(stdout,);
}

/**
 Whether a PipeWire object is of the given kind.
 
 @param object - object of the dump
 
 @param kind - part of the object's type name, such as `Node`
 
 @returns whether the type name contains the kind
 
 @example
 ```ts
 isKind({ object: { id: 1, type: 'PipeWire:Interface:Node', }, kind: 'Node', },); // true
 ```
 */
function isKind(
  {
    object,
    kind,
  }: {
    readonly kind: string;
    readonly object: PwObject;
  },
): boolean {
  return (object.type ?? '').includes(kind,);
}

/**
 Finds the node of the null sink by its name.
 
 @param objects - objects of the dump
 
 @param sinkName - the `node.name` the sink was created with
 
 @returns the node id, or `NO_NODE`
 
 @example
 ```ts
 findSinkNodeId({ objects, sinkName: 'music_player_e2e_1', },);
 ```
 */
export function findSinkNodeId(
  {
    objects,
    sinkName,
  }: {
    readonly objects: readonly PwObject[];
    readonly sinkName: string;
  },
): number {
  /**
   The node whose name is the sink name.
   */
  const node = objects.find(function isSinkNode(object,): boolean {
    return isKind({
      object,
      kind: 'Node',
    },) && (object.info
      ?.props?.['node.name']
      === sinkName);
  },);
  return node?.id ?? NO_NODE;
}

/**
 Process ids of the clients that belong to one app process, as text.
 
 @param objects - objects of the dump
 
 @param appPid - process id of the app, as text
 
 @returns ids of the PipeWire clients that report this process id
 
 @example
 ```ts
 clientIdsOf({ objects, appPid: '4242', },);
 ```
 */
function clientIdsOf(
  {
    objects,
    appPid,
  }: {
    readonly appPid: string;
    readonly objects: readonly PwObject[];
  },
): readonly number[] {
  return objects
    .filter(function belongsToApp(object,): boolean {
      return isKind({
        object,
        kind: 'Client',
      },)
        && (String(object.info
          ?.props?.['application.process.id']
          ?? '',) === appPid);
    },)
    .map(function idOf(object,): number {
      return object.id;
    },);
}

/**
 Proves that every output stream of the app process links to the null sink only.
 
 @param objects - objects of the dump
 
 @param appPid - process id of the app, as text
 
 @param sinkNodeId - node id of the private null sink
 
 @returns how many output streams and links the app has, and a sentence that says so
 
 @throws SafetyStopError when a stream of the app links to any node other than the null sink
 
 @example
 ```ts
 const routing = routingOf({ objects, appPid, sinkNodeId, },);
 ```
 */
export function routingOf(
  {
    objects,
    appPid,
    sinkNodeId,
  }: {
    readonly appPid: string;
    readonly objects: readonly PwObject[];
    readonly sinkNodeId: number;
  },
): Routing {
  /**
   PipeWire clients of the app process.
   */
  const clientIds = clientIdsOf({
    objects,
    appPid,
  },);

  /**
   Output streams that belong to those clients.
   */
  const streams = objects.filter(function isAppStream(object,): boolean {
    return isKind({
      object,
      kind: 'Node',
    },)
      && clientIds.includes(Number(object.info
        ?.props?.['client.id'],),)
      && (object.info
        ?.props?.['media.class']
        === 'Stream/Output/Audio');
  },);

  /**
   Links that leave each stream, with the node they enter.
   */
  const targets = streams.flatMap(function targetsOf(stream,): readonly {
    readonly stream: number;
    readonly target: number
  }[] {
    return objects
      .filter(function leavesStream(object,): boolean {
        return isKind({
          object,
          kind: 'Link',
        },) && (object.info?.['output-node-id'] === stream.id);
      },)
      .map(function targetOf(link,): {
        readonly stream: number;
        readonly target: number
      } {
        return {
          stream: stream.id,
          target: link.info?.['input-node-id'] ?? NO_NODE,
        };
      },);
  },);

  /**
   A link that enters another node than the null sink.
   */
  const stray = targets.find(function isStray(link,): boolean {
    return link.target !== sinkNodeId;
  },);
  if (stray !== undefined)
    throw new SafetyStopError(
      `SAFETY STOP: the app stream ${stray.stream} links to node ${stray.target}, not to the private sink ${sinkNodeId}`,
    );
  return {
    linked: targets.length,
    detail: `${streams.length} output streams, ${targets.length} links, all to node ${sinkNodeId}`,
  };
}

/**
 Creates the private null sink and waits for its node to appear.
 
 @param name - sink name, unique to this run
 
 @returns the sink; disposing it unloads the module
 
 @throws CheckFailureError when the module cannot be loaded or its node never appears
 
 @example
 ```ts
 await using sink = await createNullSink({ name: `music_player_e2e_${process.pid}`, },);
 ```
 */
export async function createNullSink(
  {
    name,
  }: {
    readonly name: string;
  },
): Promise<NullSink> {
  /**
   Tagged logger naming the sink creation.
   */
  const rl = tagged({
    tag: createNullSink.name,
    l: logger,
  },);

  /**
   Result of loading the null-sink module.
   */
  const loaded = await runCommand({
    command: 'pactl',
    args: [
      'load-module',
      'module-null-sink',
      `sink_name=${name}`,
      `sink_properties=device.description=${name}`,
    ],
  },);
  expect({
    condition: loaded.status === 0,
    message: `could not create the null sink: ${loaded.stderr}`,
  },);

  /**
   Module id that unloading needs.
   */
  const moduleId = loaded.stdout
    .trim();
  rl.info(`loaded module ${moduleId} for sink ${name}`,);

  /**
   Unloads the module that made the sink.
   */
  async function unload(): Promise<void> {
    await runCommand({
      command: 'pactl',
      args: [
        'unload-module',
        moduleId,
      ],
    },);
    rl.info(`unloaded module ${moduleId}`,);
  }

  /**
   Where the lookup leaves the node id it found.
   */
  const found = { nodeId: NO_NODE, };

  /**
   One lookup of the sink node.

   @returns whether the node is there
   */
  async function lookUpSinkNode(): Promise<boolean> {
    found.nodeId = findSinkNodeId({
      objects: await dumpPipeWire(),
      sinkName: name,
    },);
    return found.nodeId !== NO_NODE;
  }

  /**
   Whether the node appeared within the lookups.
   */
  const appeared = await pollUntil({
    attempts: SINK_LOOKUPS,
    intervalMs: SINK_LOOKUP_PAUSE_MS,
    order: 'check-then-wait',
    check: lookUpSinkNode,
  },);
  if (appeared)
    return {
      moduleId,
      name,
      nodeId: found.nodeId,
      [Symbol.asyncDispose]: unload,
    };
  await unload();
  throw new CheckFailureError('the null sink node never appeared',);
}

/**
 Reads the current routing of the app against the private sink.
 
 @param appPid - process id of the app, as text
 
 @param sink - the private null sink
 
 @returns the app's streams and links
 
 @throws SafetyStopError when a stream of the app links to any node other than the null sink
 
 @example
 ```ts
 const routing = await currentRouting({ appPid, sink, },);
 ```
 */
export async function currentRouting(
  {
    appPid,
    sink,
  }: {
    readonly appPid: string;
    readonly sink: NullSink;
  },
): Promise<Routing> {
  return routingOf({
    objects: await dumpPipeWire(),
    appPid,
    sinkNodeId: sink.nodeId,
  },);
}
