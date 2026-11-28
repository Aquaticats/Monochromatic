/**
 Unit tests for the ovh backend's configuration: which name a missing key is
 reported by, how the API host is chosen, which flavor and region overrides win,
 how the OpenSSH commands are read, where the managed keypair lives, and which VM
 names are accepted.

 No body here stands for a response: nothing in this file talks to OVHcloud. The
 five names are made-up strings from the stand-in fixture, and the assertions
 about a missing name check the name only, never a value.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_ENDPOINT,
  envRegions,
  flavorOverride,
  isEndpointName,
  ovhDataDir,
  parseRegions,
  requireCredentials,
  resolveEndpointRoot,
  scpCommand,
  sshCommand,
  validateOvhName,
} from '../../../dist/final/node/index.mjs';
import {
  installEnv,
  STAND_IN_APPLICATION_KEY,
  STAND_IN_CONSUMER_KEY,
  STAND_IN_PROJECT,
  throwMessage,
  withEnv,
} from './fixture.ovh-api-stand-in.ts';

/**
 Names whose final `mvm-<name>` is not a valid RFC 1123 hostname.
 */
const INVALID_NAMES = [
  '',
  'dev_01',
  '-dev',
  'dev-',
  'has.dot',
  'a'.repeat(64,),
];

/**
 Every variable {@link requireCredentials} reads, in the order it requires them.
 */
const REQUIRED_IN_ORDER = [
  'OVH_APPLICATION_KEY',
  'OVH_APPLICATION_SECRET',
  'OVH_CONSUMER_KEY',
  'MVM_OVH_PROJECT_ID',
];

await describe({
  name: 'ovh config',
  concurrency: 1,
  children: [
    //region Missing names

    it({
      name: 'requireCredentials reports the first missing name, in the order the guide issues them',
      fn: async () => {
        await using _env = await installEnv();
        using _noKey = withEnv({ name: 'OVH_APPLICATION_KEY', },);
        expect(throwMessage({ run: () => requireCredentials() },),).toContain('OVH_APPLICATION_KEY',);
        {
          using _key = withEnv({
            name: 'OVH_APPLICATION_KEY',
            value: 'standin-supplied-application-key',
          },);
          using _noSecret = withEnv({ name: 'OVH_APPLICATION_SECRET', },);
          const message = throwMessage({ run: () => requireCredentials() },);
          expect(message,).toContain('OVH_APPLICATION_SECRET',);
          expect(message,).not.toContain('standin-supplied-application-key',);
          {
            using _secret = withEnv({
              name: 'OVH_APPLICATION_SECRET',
              value: 'standin-supplied-application-secret',
            },);
            using _noConsumer = withEnv({ name: 'OVH_CONSUMER_KEY', },);
            expect(throwMessage({ run: () => requireCredentials() },),).toContain('OVH_CONSUMER_KEY',);
            {
              using _consumer = withEnv({
                name: 'OVH_CONSUMER_KEY',
                value: 'standin-supplied-consumer-key',
              },);
              using _noProject = withEnv({ name: 'MVM_OVH_PROJECT_ID', },);
              const message = throwMessage({ run: () => requireCredentials() },);
              expect(message,).toContain('MVM_OVH_PROJECT_ID',);
              expect(message,).not.toContain('standin-supplied-consumer-key',);
            }
          }
        }
      },
    },),
    it({
      name: 'requireCredentials returns every value when all of them are set',
      fn: async () => {
        await using _env = await installEnv();
        const credentials = requireCredentials();
        expect(credentials.applicationKey,).toBe(STAND_IN_APPLICATION_KEY,);
        expect(credentials.consumerKey,).toBe(STAND_IN_CONSUMER_KEY,);
        expect(credentials.projectId,).toBe(STAND_IN_PROJECT,);
        expect(credentials.endpointRoot,).toBe('https://ca.api.ovh.com',);
      },
    },),

    //endregion Missing names

    //region Endpoint

    it({
      name: 'resolveEndpointRoot defaults to the Canadian host and takes the names OVHcloud clients accept',
      fn: async () => {
        expect(DEFAULT_ENDPOINT,).toBe('ovh-ca',);
        expect(resolveEndpointRoot(),).toBe('https://ca.api.ovh.com',);
        expect(resolveEndpointRoot('',),).toBe('https://ca.api.ovh.com',);
        expect(resolveEndpointRoot('ovh-eu',),).toBe('https://eu.api.ovh.com',);
        expect(resolveEndpointRoot('ovh-us',),).toBe('https://api.us.ovhcloud.com',);
      },
    },),
    it({
      name: 'resolveEndpointRoot takes a complete https root and drops one trailing slash',
      fn: async () => {
        expect(resolveEndpointRoot('https://ca.api.ovh.com/',),).toBe('https://ca.api.ovh.com',);
      },
    },),
    it({
      name: 'resolveEndpointRoot refuses anything else and names the accepted forms',
      fn: async () => {
        const message = throwMessage({ run: () => resolveEndpointRoot('ovh-mars',) },);
        expect(message,).toContain('ovh-ca',);
        expect(message,).toContain('ovh-eu',);
        expect(message,).toContain('ovh-us',);
        expect(message,).toContain('ovh-mars',);
      },
    },),
    it({
      name: 'isEndpointName knows the three names and nothing else',
      fn: async () => {
        expect(isEndpointName('ovh-ca',),).toBe(true,);
        expect(isEndpointName('https://ca.api.ovh.com',),).toBe(false,);
      },
    },),

    //endregion Endpoint

    //region Overrides

    it({
      name: 'flavorOverride lets the invocation win over the environment and reports "smallest" when neither is set',
      fn: async () => {
        using _unset = withEnv({ name: 'MVM_OVH_FLAVOR', },);
        expect(flavorOverride('b3-8',),).toBe('b3-8',);
        expect(flavorOverride(),).toBe('',);
      },
    },),
    it({
      name: 'flavorOverride reads the environment when the invocation names nothing',
      fn: async () => {
        using _set = withEnv({
          name: 'MVM_OVH_FLAVOR',
          value: 'b3-16',
        },);
        expect(flavorOverride(),).toBe('b3-16',);
      },
    },),
    it({
      name: 'parseRegions splits a comma series, trims it, and drops empty parts',
      fn: async () => {
        expect(parseRegions('bhs5, gra9',),).toEqual(['bhs5', 'gra9',],);
        expect(parseRegions(' , bhs5 ,',),).toEqual(['bhs5',],);
        expect(parseRegions('',),).toEqual([],);
        expect(parseRegions(),).toEqual([],);
      },
    },),
    it({
      name: 'envRegions reads MVM_OVH_REGIONS and is empty when it is unset',
      fn: async () => {
        using _unset = withEnv({ name: 'MVM_OVH_REGIONS', },);
        expect(envRegions(),).toEqual([],);
        {
          using _set = withEnv({
            name: 'MVM_OVH_REGIONS',
            value: 'bhs5',
          },);
          expect(envRegions(),).toEqual(['bhs5',],);
        }
      },
    },),

    //endregion Overrides

    //region OpenSSH commands

    it({
      name: 'sshCommand and scpCommand default to the bare names',
      fn: async () => {
        using _ssh = withEnv({ name: 'MVM_OVH_SSH_COMMAND', },);
        using _scp = withEnv({ name: 'MVM_OVH_SCP_COMMAND', },);
        expect(sshCommand(),).toEqual(['ssh',],);
        expect(scpCommand(),).toEqual(['scp',],);
      },
    },),
    it({
      name: 'sshCommand reads a JSON array naming the executable and its leading arguments',
      fn: async () => {
        using _set = withEnv({
          name: 'MVM_OVH_SSH_COMMAND',
          value: JSON.stringify(['/usr/bin/env', 'ssh', '-F', '/dev/null',],),
        },);
        expect(sshCommand(),).toEqual(['/usr/bin/env', 'ssh', '-F', '/dev/null',],);
      },
    },),
    it({
      name: 'sshCommand refuses a value that is not a JSON array and names the variable',
      fn: async () => {
        using _set = withEnv({
          name: 'MVM_OVH_SSH_COMMAND',
          value: 'ssh -F /dev/null',
        },);
        const message = throwMessage({ run: () => sshCommand() },);
        expect(message,).toContain('MVM_OVH_SSH_COMMAND',);
        expect(message,).toContain('JSON array',);
      },
    },),

    //endregion OpenSSH commands

    //region Data directory

    it({
      name: 'ovhDataDir sits under the tool data directory unless the environment moves it',
      fn: async () => {
        using _unset = withEnv({ name: 'MVM_OVH_DATA_DIR', },);
        expect(ovhDataDir(),).toBe(join(
          homedir(),
          '.local',
          'share',
          'mvm',
          'ovh',
        ),);
        using _set = withEnv({
          name: 'MVM_OVH_DATA_DIR',
          value: '/tmp/elsewhere',
        },);
        expect(ovhDataDir(),).toBe('/tmp/elsewhere',);
      },
    },),

    //endregion Data directory

    //region Names

    it({
      name: 'validateOvhName accepts names that make an RFC 1123 hostname',
      fn: async () => {
        expect(() => validateOvhName('dev-01',),).not.toThrow();
        expect(() => validateOvhName('build2',),).not.toThrow();
      },
    },),
    it({
      name: 'validateOvhName rejects underscores, empty names, hyphen edges, dots, and over-length names',
      fn: async () => {
        for (const name of INVALID_NAMES) {
          expect(() => validateOvhName(name,),).toThrow();
        }
      },
    },),

    //endregion Names
  ],
},);
