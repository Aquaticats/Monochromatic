/**
 Unit tests for reading the project's instances: only names carrying the tool's
 prefix are mvm's, an instance the project lists as deleted is nobody's while
 one it lists as deleting stays mvm's, a name is matched exactly, the public
 IPv4 is picked out of the address list, an instance that reaches a failed
 status stops the wait, and a wait for accepted deletes ends on a status of
 deleted or on absence, says each status once, and names what is left at its
 bound.

 Instance bodies here are stand-ins written by hand from OVHcloud's schema
 (`cloud.instance.Instance`, `cloud.instance.IpAddress`), never a recording, so
 nothing in this file claims what OVHcloud really answers.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findMvmInstance,
  listMvmInstances,
  mvmInstances,
  publicIpv4,
  runningTimeoutMessage,
  waitUntilDeleted,
  waitUntilRunning,
  type OvhInstance,
} from '../../../dist/final/node/index.mjs';
import {
  cloudPath,
  errorFrom,
  installEnv,
  installStandIn,
  instanceAnswer,
  jsonAnswer,
  scriptedStandIn,
  throwMessage,
} from './fixture.ovh-api-stand-in.ts';

//region Fixtures

/**
 What the project holds: two of mvm's machines and one that is somebody else's,
 whose name merely starts with the same letters.
 */
const PROJECT_INSTANCES = [
  instanceAnswer({ id: 'instance-dev', name: 'mvm-dev', },),
  instanceAnswer({ id: 'instance-build', name: 'mvm-build', },),
  instanceAnswer({ id: 'instance-foreign', name: 'mvmx-other', },),
  instanceAnswer({ id: 'instance-web', name: 'web-1', },),
] as const satisfies readonly OvhInstance[];

//endregion Fixtures

//region Recording logger

/**
 A logger that keeps every message it is given, so a test can read what a wait said.
 
 @param said - list the messages are appended to, in order
 
 @returns a {@link Logger} whose level methods append and whose flush resolves
 
 @example
 ```ts
 const said: string[] = [];
 waitUntilDeleted({ instances, rl: recordingLogger(said) });
 ```
 */
function recordingLogger(said: string[],): Logger {
  return {
    debug: function record(message: string,): void {
      said.push(message,);
    },
    error: function record(message: string,): void {
      said.push(message,);
    },
    fatal: function record(message: string,): void {
      said.push(message,);
    },
    flush: function flushed(): Promise<void> {
      return Promise.resolve();
    },
    info: function record(message: string,): void {
      said.push(message,);
    },
    trace: function record(message: string,): void {
      said.push(message,);
    },
    warn: function record(message: string,): void {
      said.push(message,);
    },
  };
}

//endregion Recording logger

await describe({
  name: 'ovh instances',
  concurrency: 1,
  children: [
    //region Ownership

    it({
      name: 'mvmInstances keeps only names carrying the tool prefix, dropped from any other name',
      fn: async () => {
        expect(
          mvmInstances({ instances: PROJECT_INSTANCES, },)
            .map(function instanceName(instance,) {
              return instance.name;
            },),
        ).toEqual(['mvm-dev', 'mvm-build',],);
      },
    },),
    it({
      name: 'listMvmInstances asks the project and drops every instance it does not own',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer(PROJECT_INSTANCES,),);
        expect(
          (await listMvmInstances()).map(function instanceName(instance,) {
            return instance.name;
          },),
        ).toEqual(['mvm-dev', 'mvm-build',],);
      },
    },),
    it({
      name: 'mvmInstances drops an owned instance the project lists as deleted',
      fn: async () => {
        expect(
          mvmInstances({
            instances: [
              ...PROJECT_INSTANCES,
              instanceAnswer({ id: 'instance-gone', name: 'mvm-gone', status: 'DELETED', },),
            ],
          },)
            .map(function instanceName(instance,) {
              return instance.name;
            },),
        ).toEqual(['mvm-dev', 'mvm-build',],);
      },
    },),
    it({
      name: 'findMvmInstance resolves the live instance when a deleted one still carries its name',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() =>
          jsonAnswer([
            instanceAnswer({ id: 'instance-dev-gone', name: 'mvm-dev', status: 'DELETED', },),
            ...PROJECT_INSTANCES,
          ],)
        );
        expect((await findMvmInstance({ name: 'dev', },)).id,).toBe('instance-dev',);
      },
    },),

    //endregion Ownership

    //region Lookup

    it({
      name: 'findMvmInstance resolves one owned instance by its unprefixed name',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer(PROJECT_INSTANCES,),);
        expect((await findMvmInstance({ name: 'dev', },)).id,).toBe('instance-dev',);
      },
    },),
    it({
      name: 'findMvmInstance refuses a name only an instance mvm does not own matches',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer(PROJECT_INSTANCES,),);
        const error = await errorFrom({ run: () => findMvmInstance({ name: 'x-other', },) },);
        expect(caughtValueText(error,),).toContain('no mvm-managed',);
      },
    },),
    it({
      name: 'findMvmInstance reports a missing instance by name',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer(PROJECT_INSTANCES,),);
        const error = await errorFrom({ run: () => findMvmInstance({ name: 'ghost', },) },);
        expect(Error.isError(error,),).toBe(true,);
        expect(caughtValueText(error,),).toContain('ghost',);
      },
    },),
    it({
      name: 'findMvmInstance reports an ambiguous name instead of picking one',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer([
          instanceAnswer({ id: 'instance-1', name: 'mvm-dev', },),
          instanceAnswer({ id: 'instance-2', name: 'mvm-dev', },),
        ],),);
        const error = await errorFrom({ run: () => findMvmInstance({ name: 'dev', },) },);
        expect(caughtValueText(error,),).toContain('ambiguous',);
      },
    },),

    //endregion Lookup

    //region Addresses

    it({
      name: 'publicIpv4 takes the public IPv4 and skips private and IPv6 addresses',
      fn: async () => {
        /**
         Instance whose address list holds a private IPv4 first, then an IPv6,
         then the public IPv4 to connect to.
         */
        const instance: OvhInstance = {
          ...instanceAnswer({},),
          ipAddresses: [
            {
              ip: '10.0.0.4',
              type: 'private',
              version: 4,
            },
            {
              ip: '2001:db8::1',
              type: 'public',
              version: 6,
            },
            {
              ip: '192.0.2.11',
              type: 'public',
              version: 4,
            },
          ],
        };
        expect(publicIpv4({ instance, },),).toBe('192.0.2.11',);
      },
    },),
    it({
      name: 'publicIpv4 names the addresses it did find when there is no public IPv4',
      fn: async () => {
        const message = throwMessage({
          run: () => publicIpv4({ instance: instanceAnswer({ ip: '', },), },),
        },);
        expect(message,).toContain('no public IPv4',);
      },
    },),

    //endregion Addresses

    //region Waiting

    it({
      name: 'runningTimeoutMessage names the instance and the last status it showed',
      fn: async () => {
        const message = runningTimeoutMessage({
          lastStatus: 'BUILD',
          name: 'dev-01',
        },);
        expect(message,).toContain('dev-01',);
        expect(message,).toContain('its last status was BUILD',);
      },
    },),
    it({
      name: 'runningTimeoutMessage says so when no status was read',
      fn: async () => {
        expect(
          runningTimeoutMessage({
            lastStatus: '',
            name: 'dev-01',
          },),
        ).toContain('no status was read',);
      },
    },),
    it({
      name: 'waitUntilRunning polls until the instance reports ACTIVE',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = scriptedStandIn([
          jsonAnswer(instanceAnswer({ status: 'BUILD', },),),
          jsonAnswer(instanceAnswer({ status: 'ACTIVE', },),),
        ],);
        const instance = await waitUntilRunning({
          instanceId: 'instance-1',
          name: 'dev',
        },);
        expect(instance.status,).toBe('ACTIVE',);
        expect(
          standIn.calls.filter(function readOne(call,) {
            return call.path === cloudPath('/instance/instance-1',);
          },).length,
        ).toBe(2,);
      },
    },),
    it({
      name: 'waitUntilRunning stops at once when the instance reaches a failed status',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer(instanceAnswer({ status: 'ERROR', },),),);
        const error = await errorFrom({
          run: async function wait() {
            await waitUntilRunning({
              instanceId: 'instance-1',
              name: 'dev',
            },);
          },
        },);
        expect(caughtValueText(error,),).toContain('ERROR',);
      },
    },),
    it({
      name: 'waitUntilRunning reads the instance path of the project',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(instanceAnswer({},),),);
        await waitUntilRunning({
          instanceId: 'instance-1',
          name: 'dev',
        },);
        expect(
          standIn.calls.some(function readOne(call,) {
            return call.path === cloudPath('/instance/instance-1',);
          },),
        ).toBe(true,);
      },
    },),

    //endregion Waiting

    //region Waiting for a delete

    it({
      name: 'waitUntilDeleted says each status once and stops when the project lists the machine as deleted',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = scriptedStandIn([
          jsonAnswer([instanceAnswer({
            id: 'instance-1',
            name: 'mvm-dev',
            status: 'DELETING',
          },),],),
          jsonAnswer([instanceAnswer({
            id: 'instance-1',
            name: 'mvm-dev',
            status: 'DELETING',
          },),],),
          jsonAnswer([instanceAnswer({
            id: 'instance-1',
            name: 'mvm-dev',
            status: 'DELETED',
          },),],),
        ],);
        /**
         What the wait said, in order, through the logger it was given.
         */
        const said: string[] = [];
        await waitUntilDeleted({
          instances: [{ id: 'instance-1', name: 'dev', },],
          pollIntervalMs: 1,
          rl: recordingLogger(said,),
        },);
        expect(standIn.calls,).toHaveLength(3,);
        expect(said,).toEqual(['instance dev is DELETING',],);
      },
    },),
    it({
      name: 'waitUntilDeleted stops when the project no longer lists the machine',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = scriptedStandIn([
          jsonAnswer([instanceAnswer({
            id: 'instance-1',
            name: 'mvm-dev',
            status: 'DELETING',
          },),],),
          jsonAnswer([],),
        ],);
        /**
         What the wait said, in order.
         */
        const said: string[] = [];
        await waitUntilDeleted({
          instances: [{ id: 'instance-1', name: 'dev', },],
          pollIntervalMs: 1,
          rl: recordingLogger(said,),
        },);
        expect(standIn.calls,).toHaveLength(2,);
        expect(said,).toEqual(['instance dev is DELETING',],);
      },
    },),
    it({
      name: 'waitUntilDeleted returns at once when the project has already stopped listing the machine',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer([],),);
        /**
         What the wait said, in order.
         */
        const said: string[] = [];
        await waitUntilDeleted({
          instances: [{ id: 'instance-1', name: 'dev', },],
          pollIntervalMs: 1,
          rl: recordingLogger(said,),
        },);
        expect(standIn.calls,).toHaveLength(1,);
        expect(said,).toEqual([],);
      },
    },),
    it({
      name: 'waitUntilDeleted throws at its bound, naming the machine, its last status, the bill and both commands',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer([instanceAnswer({
          id: 'instance-1',
          name: 'mvm-dev',
          status: 'DELETING',
        },),],),);
        /**
         What the wait said before its bound passed, which must hold the status once.
         */
        const said: string[] = [];
        const error = await errorFrom({
          run: () => waitUntilDeleted({
            instances: [{ id: 'instance-1', name: 'dev', },],
            pollIntervalMs: 1,
            rl: recordingLogger(said,),
            timeoutMs: 20,
          },),
        },);
        /**
         The message the bound's passing throws.
         */
        const message = caughtValueText(error,);
        expect(message,).toContain('dev',);
        expect(message,).toContain('DELETING',);
        expect(message,).toContain('bills an instance until it is deleted',);
        expect(message,).toContain('mvm --backend ovh list',);
        expect(message,).toContain('mvm --backend ovh destroy dev',);
        expect(said,).toEqual(['instance dev is DELETING',],);
      },
    },),

    //endregion Waiting for a delete
  ],
},);
