import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runSubcommand, } from '../dist/final/node/run-subcommand.mjs';

/**
 Parsed config as the subcommand takes it.
 */
type WireguardConfig = Parameters<typeof runSubcommand>[0]['config'];

/**
 Builds a parsed config with the facts a start notice depends on.

 @param exemptMark - `ExemptMark`,
 absent for none.

 @param table - `Table` value,
 absent for automatic.

 @param dns - `DNS =` servers.

 @returns The config.

 @example
 ```ts
 configWith({ dns: ['10.0.0.1'] });
 ```
 */
function configWith(
  {
    exemptMark,
    table,
    dns,
  }: {
    readonly exemptMark?: number;
    readonly table?: string;
    readonly dns: readonly string[];
  },
): WireguardConfig {
  return {
    interfaceName: 'wgtest',
    addresses: ['10.0.0.2/32',],
    dns,
    dnsSearch: [],
    ...(exemptMark === undefined ? {} : { exemptMark, }),
    ...(table === undefined ? {} : { table, }),
    endpointPorts: [51_820,],
    allowedFromFiles: [],
    preUp: [],
    postUp: [],
    preDown: [],
    postDown: [],
    wgConfig: '',
  };
}

/**
 Runs a subcommand over recording steps.

 @param subcommand - `up` or `down`.

 @param config - Parsed config.

 @returns Everything the steps were asked to do,
 in order.

 @example
 ```ts
 await record({ subcommand: 'up', config: configWith({ dns: [] }) });
 ```
 */
async function record(
  {
    subcommand,
    config,
  }: {
    readonly subcommand: 'up' | 'down';
    readonly config: WireguardConfig;
  },
): Promise<readonly string[]> {
  /**
   Events in the order they happened.
   */
  const events: string[] = [];
  await runSubcommand({
    subcommand,
    target: 'wgtest',
    config,
    steps: {
      up: (call,) => {
        events.push(`up ${call.config.interfaceName}`,);
        return Promise.resolve();
      },
      down: (call,) => {
        events.push(`down ${call.config.interfaceName}`,);
        return Promise.resolve();
      },
      printNotice: (notice,) => {
        events.push(`notice ${notice}`,);
      },
    },
  },);
  return events;
}

await describe({
  name: runSubcommand.name,
  children: [
    it({
      name: 'warns about a config without ExemptMark naming both policies as not in force, and then still brings the interface up',
      fn: async () => {
        const events = await record({ subcommand: 'up', config: configWith({ dns: ['10.0.0.1',], }), },);
        expect(events,).toHaveLength(2,);
        expect(events[0],).toContain('notice Config for wgtest has no ExemptMark',);
        expect(events[0],).toContain('POLICY NOT IN FORCE',);
        expect(events[0],).toContain('Add `ExemptMark = ',);
        expect(events[1],).toBe('up wgtest',);
      },
    },),
    it({
      name: 'reports Table = off as the reason the policies are not in force on every start, and then still brings the interface up',
      fn: async () => {
        const events = await record({ subcommand: 'up', config: configWith({ exemptMark: 100, table: 'off', dns: ['10.0.0.1',], }), },);
        expect(events,).toHaveLength(2,);
        expect(events[0],).toContain('notice ',);
        expect(events[0],).toContain('POLICY NOT IN FORCE',);
        expect(events[0],).toContain('Table = off',);
        expect(events[1],).toBe('up wgtest',);
        /**
         A second start of the same config prints the same report again.
         */
        const again = await record({ subcommand: 'up', config: configWith({ exemptMark: 100, table: 'off', dns: ['10.0.0.1',], }), },);
        expect(again,).toEqual(events,);
      },
    },),
    it({
      name: 'prints the policy notice before bringing up a config with a mark, and brings it up once',
      fn: async () => {
        const events = await record({ subcommand: 'up', config: configWith({ exemptMark: 100, dns: ['10.0.0.1',], }), },);
        expect(events,).toHaveLength(2,);
        expect(events[0],).toContain('notice ',);
        expect(events[0],).toContain('OUTSIDE the WireGuard tunnel',);
        expect(events.filter((event,) => event.startsWith('up ',),),).toEqual(['up wgtest',],);
      },
    },),
    it({
      name: 'prints no start notice for down and takes the interface down',
      fn: async () => {
        expect(await record({ subcommand: 'down', config: configWith({ dns: [], }), },),).toEqual(['down wgtest',],);
        expect(await record({ subcommand: 'down', config: configWith({ exemptMark: 100, table: 'off', dns: [], }), },),).toEqual(['down wgtest',],);
      },
    },),
  ],
},);
