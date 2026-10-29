/**
 Tests for the PipeWire proof: the safety stop for a stream linked anywhere but the private sink, with allowed and
 rejected graphs.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SafetyStopError, } from './errors.ts';
import {
  findSinkNodeId,
  NO_NODE,
  parsePwDump,
  type PwObject,
  routingOf,
} from './pipewire.ts';

/**
 Builds the graph objects of one app process: a client, an output stream, and the links the caller lists.
 
 @param pid - process id of the app, as text
 
 @param streamId - node id of the output stream
 
 @param targets - node ids the stream links to
 
 @returns the objects
 */
function appGraph(
  pid: string,
  streamId: number,
  targets: readonly number[],
  clientId = 40,
): readonly PwObject[] {
  return [
    { id: clientId, type: 'PipeWire:Interface:Client', info: { props: { 'application.process.id': Number(pid,), }, }, },
    {
      id: streamId,
      type: 'PipeWire:Interface:Node',
      info: { props: { 'client.id': clientId, 'media.class': 'Stream/Output/Audio', }, },
    },
    ...targets.map((target, index,): PwObject => ({
      id: 500 + index,
      type: 'PipeWire:Interface:Link',
      info: { 'output-node-id': streamId, 'input-node-id': target, },
    }),),
  ];
}

/**
 The private sink node.
 */
const SINK: PwObject = {
  id: 108,
  type: 'PipeWire:Interface:Node',
  info: { props: { 'node.name': 'music_player_e2e_5', 'media.class': 'Audio/Sink', }, },
};

await describe({
  name: parsePwDump.name,
  children: [
    it({
      name: 'decodes a dump and keeps only entries with a numeric id',
      fn: async ({ expect, }) => {
        expect(parsePwDump('[{"id":3},{"id":"x"},5,null,{"name":"no id"}]',),).toEqual([{ id: 3, },],);
      },
    },),
    it({
      name: 'gives no objects for empty text, malformed text and JSON that is not a list',
      fn: async ({ expect, }) => {
        expect(parsePwDump('',),).toEqual([],);
        expect(parsePwDump('not json',),).toEqual([],);
        expect(parsePwDump('{"id":1}',),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: findSinkNodeId.name,
  children: [
    it({
      name: 'finds the node that has the sink name',
      fn: async ({ expect, }) => {
        expect(findSinkNodeId({ objects: [SINK,], sinkName: 'music_player_e2e_5', },),).toBe(108,);
      },
    },),
    it({
      name: 'gives NO_NODE for another name, or a client with that name',
      fn: async ({ expect, }) => {
        expect(findSinkNodeId({ objects: [SINK,], sinkName: 'other', },),).toBe(NO_NODE,);
        const client: PwObject = {
          id: 9,
          type: 'PipeWire:Interface:Client',
          info: { props: { 'node.name': 'music_player_e2e_5', }, },
        };
        expect(findSinkNodeId({ objects: [client,], sinkName: 'music_player_e2e_5', },),).toBe(NO_NODE,);
      },
    },),
  ],
},);

await describe({
  name: routingOf.name,
  children: [
    it({
      name: 'accepts a stream linked to the private sink only',
      fn: async ({ expect, }) => {
        const routing = routingOf({ objects: [SINK, ...appGraph('4242', 60, [108,],),], appPid: '4242', sinkNodeId: 108, },);
        expect(routing,).toEqual({ linked: 1, detail: '1 output streams, 1 links, all to node 108', },);
      },
    },),
    it({
      name: 'reports no links while the stream is not linked yet',
      fn: async ({ expect, }) => {
        const routing = routingOf({ objects: [SINK, ...appGraph('4242', 60, [],),], appPid: '4242', sinkNodeId: 108, },);
        expect(routing.linked,).toBe(0,);
      },
    },),
    it({
      name: 'stops the run when the stream also links to another node',
      fn: async ({ expect, }) => {
        let caught: unknown;
        try {
          routingOf({ objects: [SINK, ...appGraph('4242', 60, [108, 77,],),], appPid: '4242', sinkNodeId: 108, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(SafetyStopError,);
        expect((caught as Error).message,).toBe('SAFETY STOP: the app stream 60 links to node 77, not to the private sink 108',);
      },
    },),
    it({
      name: 'stops the run when the stream links only to the default device',
      fn: async ({ expect, }) => {
        let caught: unknown;
        try {
          routingOf({ objects: [SINK, ...appGraph('4242', 60, [77,],),], appPid: '4242', sinkNodeId: 108, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(SafetyStopError,);
      },
    },),
    it({
      name: 'stops the run for a link without a target node',
      fn: async ({ expect, }) => {
        const objects: readonly PwObject[] = [
          ...appGraph('4242', 60, [],),
          { id: 501, type: 'PipeWire:Interface:Link', info: { 'output-node-id': 60, }, },
        ];
        let caught: unknown;
        try {
          routingOf({ objects, appPid: '4242', sinkNodeId: 108, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(SafetyStopError,);
      },
    },),
    it({
      name: 'ignores streams of other processes, such as the user own player',
      fn: async ({ expect, }) => {
        const other = appGraph('999', 61, [77,], 41,);
        const routing = routingOf({
          objects: [SINK, ...appGraph('4242', 60, [108,],), ...other,],
          appPid: '4242',
          sinkNodeId: 108,
        },);
        expect(routing.linked,).toBe(1,);
      },
    },),
    it({
      name: 'ignores nodes of the app that are not output audio streams',
      fn: async ({ expect, }) => {
        const objects: readonly PwObject[] = [
          { id: 40, type: 'PipeWire:Interface:Client', info: { props: { 'application.process.id': 4_242, }, }, },
          {
            id: 62,
            type: 'PipeWire:Interface:Node',
            info: { props: { 'client.id': 40, 'media.class': 'Stream/Input/Audio', }, },
          },
          { id: 600, type: 'PipeWire:Interface:Link', info: { 'output-node-id': 62, 'input-node-id': 77, }, },
        ];
        expect(routingOf({ objects, appPid: '4242', sinkNodeId: 108, },).linked,).toBe(0,);
      },
    },),
  ],
},);
