import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  deleteLinkVerified,
  teardownKeepingTunnelOnFailure,
  type TeardownSteps,
} from '../dist/final/node/tunnel-teardown.mjs';
import {
  trafficBlockScript,
  trafficBlockTable,
} from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Configuration the sequence is given.
 */
type Config = Parameters<typeof teardownKeepingTunnelOnFailure>[0]['config'];

/**
 Builds a configuration for the sequence.

 @param table - `Table` value,
 absent for automatic.

 @param interfaceName - Interface name.

 @returns The configuration.

 @example
 ```ts
 configWith({ table: 'off' });
 ```
 */
function configWith({ table, interfaceName = 'wgtest', }: { readonly table?: string; readonly interfaceName?: string; } = {},): Config {
  return {
    interfaceName,
    addresses: ['10.0.0.2/32',],
    dns: ['10.0.0.1',],
    dnsSearch: [],
    ...(table === undefined ? {} : { table, }),
    exemptMark: 100,
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
 Runs the sequence with recording steps,
 where the named steps fail.

 @param failing - Steps that throw,
 by name.

 @param table - `Table` value.

 @param exemptionOutcome - What the application watcher stop answers.

 @returns What the steps did,
 and what the sequence threw.

 @example
 ```ts
 await run({ failing: ['stopRouteWatcher'] });
 ```
 */
async function run(
  {
    failing = [],
    table,
    exemptionOutcome = { ok: true, },
  }: {
    readonly failing?: readonly string[];
    readonly table?: string;
    readonly exemptionOutcome?: Awaited<ReturnType<TeardownSteps['stopApplicationExemptions']>>;
  } = {},
): Promise<{ readonly events: readonly string[]; readonly thrown: unknown; }> {
  /**
   Events in the order they happened.
   */
  const events: string[] = [];
  /**
   Builds a step that records its name and throws when asked to.

   @param name - Step name.

   @returns The step.
   */
  function step(name: string,): () => Promise<void> {
    return () => {
      events.push(name,);
      return failing.includes(name,) ? Promise.reject(new Error(`${name} failed`,),) : Promise.resolve();
    };
  }
  /**
   Steps that record.
   */
  const steps: TeardownSteps = {
    stopRouteWatcher: step('stopRouteWatcher',),
    restartRouteWatcher: step('restartRouteWatcher',),
    stopApplicationExemptions: () => {
      events.push('stopApplicationExemptions',);
      return failing.includes('stopApplicationExemptions',)
        ? Promise.reject(new Error('stopApplicationExemptions failed',),)
        : Promise.resolve(exemptionOutcome,);
    },
    removePolicyRules: step('removePolicyRules',),
    removeBypassState: step('removeBypassState',),
    removeKillSwitch: step('removeKillSwitch',),
    deleteLink: step('deleteLink',),
    revertDns: step('revertDns',),
    removeOpenSnitchAllowance: step('removeOpenSnitchAllowance',),
    blockTraffic: step('blockTraffic',),
    removeTrafficBlock: step('removeTrafficBlock',),
  };
  /**
   What the sequence threw,
   absent when it did not.
   */
  const thrown = await teardownKeepingTunnelOnFailure({ config: configWith(table === undefined ? {} : { table, },), steps, },).then(
    (): undefined => undefined,
    (error: unknown) => error,
  );
  return { events, thrown, };
}

/**
 Text of what was thrown.

 @param thrown - Thrown value.

 @returns Its message.

 @example
 ```ts
 messageOf({ thrown: new Error('x') }); // 'Error: x'
 ```
 */
function messageOf({ thrown, }: { readonly thrown: unknown; },): string {
  return String(thrown,);
}

/**
 The steps that can fail without a routing change,
 in the order they must run.
 */
const CAN_FAIL_FIRST = ['stopRouteWatcher', 'stopApplicationExemptions',] as const;

await describe({
  name: teardownKeepingTunnelOnFailure.name,
  children: [
    it({
      name: 'runs the steps that can fail before any routing is removed, the policy rules next, and the leftovers after the interface is gone',
      fn: async () => {
        const { events, thrown, } = await run();
        expect(thrown,).toBe(undefined,);
        expect(events,).toEqual([
          ...CAN_FAIL_FIRST,
          'removePolicyRules',
          'removeBypassState',
          'removeKillSwitch',
          'deleteLink',
          'revertDns',
          'removeOpenSnitchAllowance',
          'removeTrafficBlock',
        ],);
      },
    },),
    it({
      name: 'skips the policy rules when Table is off, and removes everything else',
      fn: async () => {
        const { events, thrown, } = await run({ table: 'off', },);
        expect(thrown,).toBe(undefined,);
        expect(events.includes('removePolicyRules',),).toBe(false,);
        expect(events.includes('deleteLink',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps the tunnel fully up when the route watcher cannot be stopped: nothing else runs, nothing is blocked, and the error says what to do',
      fn: async () => {
        const { events, thrown, } = await run({ failing: ['stopRouteWatcher',], },);
        expect(events,).toEqual(['stopRouteWatcher',],);
        expect((thrown as Error).name,).toBe('TeardownRefusedError',);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('could not stop the route watcher (Error: stopRouteWatcher failed)',);
        expect(message,).toContain('the tunnel wgtest is still fully up; nothing was removed',);
        expect(message,).toContain('then run `wg-quicker down wgtest` again',);
        expect(message,).toContain('with the build that started the tunnel',);
      },
    },),
    it({
      name: 'starts the route watcher again when the application watcher cannot be stopped, removes no routing, and says what is uncertain',
      fn: async () => {
        const { events, thrown, } = await run({ failing: ['stopApplicationExemptions',], },);
        expect(events,).toEqual([...CAN_FAIL_FIRST, 'restartRouteWatcher',],);
        expect((thrown as Error).name,).toBe('TeardownRefusedError',);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('could not stop the application watcher',);
        expect(message,).toContain('the route watcher was started again',);
        expect(message,).toContain('sudo wg-quicker-exempt watch-stop wgtest',);
      },
    },),
    it({
      name: 'says so loudly when the route watcher that was stopped cannot be started again either',
      fn: async () => {
        const { events, thrown, } = await run({ failing: ['stopApplicationExemptions', 'restartRouteWatcher',], },);
        expect(events.includes('blockTraffic',),).toBe(false,);
        expect(messageOf({ thrown, },),).toContain('already stopped and could not be started again',);
        expect(messageOf({ thrown, },),).toContain('the bypass table no longer follows the default route',);
      },
    },),
    it({
      name: 'goes on when the application watcher program is missing, as before, because that case is reported by the stop itself',
      fn: async () => {
        const { events, thrown, } = await run({ exemptionOutcome: { ok: false, report: 'wg-quicker-exempt was not found', }, },);
        expect(thrown,).toBe(undefined,);
        expect(events.includes('deleteLink',),).toBe(true,);
      },
    },),
    it({
      name: 'blocks traffic and says how to finish when a step after the routing removal began fails',
      fn: async () => {
        for (
          const [failing, what,] of [
            ['removePolicyRules', 'remove the policy routing rules',],
            ['removeBypassState', 'remove the application-bypass routes and rules',],
            ['removeKillSwitch', 'remove the firewall kill switch',],
            ['deleteLink', 'delete the interface',],
          ] as const
        ) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own recording.
          const { events, thrown, } = await run({ failing: [failing,], },);
          expect(events.at(-2,),).toBe(failing,);
          expect(events.at(-1,),).toBe('blockTraffic',);
          expect((thrown as Error).name,).toBe('TeardownIncompleteError',);
          const message = messageOf({ thrown, },);
          expect(message,).toContain(`could not ${what} (Error: ${failing} failed)`,);
          expect(message,).toContain('traffic is BLOCKED (nft table inet wg-quicker-wgtest-blocked',);
          expect(message,).toContain('run `wg-quicker down wgtest` again, with this build or with the build that started the tunnel',);
          expect(message,).toContain('sudo nft delete table inet wg-quicker-wgtest-blocked',);
        }
      },
    },),
    it({
      name: 'says that traffic leaves outside the tunnel when the block cannot be installed either',
      fn: async () => {
        const { thrown, } = await run({ failing: ['deleteLink', 'blockTraffic',], },);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('traffic could NOT be blocked (Error: blockTraffic failed)',);
        expect(message,).toContain('it leaves OUTSIDE the tunnel now',);
        expect(message,).not.toContain('traffic is BLOCKED',);
      },
    },),
    it({
      name: 'reports a failure after the interface is gone without blocking anything, and still runs the other leftovers',
      fn: async () => {
        const { events, thrown, } = await run({ failing: ['revertDns',], },);
        expect(events.includes('blockTraffic',),).toBe(false,);
        expect(events.at(-1,),).toBe('removeTrafficBlock',);
        expect(events.includes('removeOpenSnitchAllowance',),).toBe(true,);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('removed the tunnel but could not revert the DNS settings',);
        expect(message,).toContain('nothing is blocked',);
      },
    },),
  ],
},);

await describe({
  name: trafficBlockScript.name,
  children: [
    it({
      name: 'drops what leaves through any interface but loopback and what is forwarded, in a table of its own',
      fn: async () => {
        expect(trafficBlockTable({ interfaceName: 'wgtest', },),).toBe('wg-quicker-wgtest-blocked',);
        /**
         The script for the interface.
         */
        const script = trafficBlockScript({ interfaceName: 'wgtest', },);
        expect(script,).toContain('add table inet wg-quicker-wgtest-blocked\n',);
        expect(script,).toContain('hook output priority -400; policy accept; }',);
        expect(script,).toContain('output oifname != "lo" counter drop\n',);
        expect(script,).toContain('hook forward priority -400; policy accept; }',);
        expect(script,).toContain('forward counter drop\n',);
        expect(script.endsWith('\n',),).toBe(true,);
        expect(trafficBlockTable({ interfaceName: 'wgtest', },),).not.toBe('wg-quicker-wgtest',);
      },
    },),
  ],
},);

await describe({
  name: deleteLinkVerified.name,
  children: [
    it({
      name: 'resolves for an interface that does not exist and rejects, naming the interface, for one that is still there after the delete: the loopback cannot be deleted',
      fn: async () => {
        await deleteLinkVerified({ config: configWith({ interfaceName: 'wgtest-no-such-link', },), },);
        /**
         What the delete of the loopback led to.
         */
        const refusal = await deleteLinkVerified({ config: configWith({ interfaceName: 'lo', },), },).then(
          (): string => 'resolved',
          String,
        );
        expect(refusal,).toContain('the interface lo still exists after `ip link delete`',);
      },
    },),
  ],
},);
