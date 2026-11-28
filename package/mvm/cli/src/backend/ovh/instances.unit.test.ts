/**
 Unit tests for reading the project's instances: only names carrying the tool's
 prefix are mvm's, a name is matched exactly, the public IPv4 is picked out of the
 address list, and an instance that reaches a failed status stops the wait.

 Instance bodies here are stand-ins written by hand from OVHcloud's schema
 (`cloud.instance.Instance`, `cloud.instance.IpAddress`), never a recording, so
 nothing in this file claims what OVHcloud really answers.

 @module
 */

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
  waitUntilRunning,
} from '../../../dist/final/node/index.mjs';
import type { OvhInstance, } from '../../../dist/final/node/index.mjs';
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
        expect(Error.isError(error,) ? error.message : '',).toContain('no mvm-managed',);
      },
    },),
    it({
      name: 'findMvmInstance reports a missing instance by name',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(() => jsonAnswer(PROJECT_INSTANCES,),);
        const error = await errorFrom({ run: () => findMvmInstance({ name: 'ghost', },) },);
        expect(Error.isError(error,),).toBe(true,);
        expect(Error.isError(error,) ? error.message : '',).toContain('ghost',);
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
        expect(Error.isError(error,) ? error.message : '',).toContain('ambiguous',);
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
        expect(Error.isError(error,) ? error.message : '',).toContain('ERROR',);
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
  ],
},);
