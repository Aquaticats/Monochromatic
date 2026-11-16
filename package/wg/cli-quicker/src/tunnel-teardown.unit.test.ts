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
  BLOCK_EXCEPTIONS,
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
 The lines the manual-steps step answers.
 */
const MANUAL_STEPS: readonly string[] = ['to look at what could not be read:', '  sudo cat /run/wg-quicker/state.json',];

/**
 Reads what a call throws.

 @param call - Call expected to throw.

 @returns The thrown value,
 or nothing when it did not throw.

 @example
 ```ts
 await thrownBy({ call: () => Promise.reject(new Error('x')) });
 ```
 */
async function thrownBy({ call, }: { readonly call: () => unknown; },): Promise<unknown> {
  try {
    await call();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

/**
 Runs the sequence with recording steps,
 where the named steps fail.

 @param failing - Steps that throw,
 by name.

 @param table - `Table` value.

 @param exemptionOutcome - What the application watcher stop answers.

 @param earlierBlock - Whether the traffic block of an earlier `down` is in place.

 @param blockError - What the traffic block step throws when it is among the failing steps.

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
    earlierBlock = false,
    blockError = new Error('blockTraffic failed',),
  }: {
    readonly failing?: readonly string[];
    readonly table?: string;
    readonly exemptionOutcome?: Awaited<ReturnType<TeardownSteps['stopApplicationExemptions']>>;
    readonly earlierBlock?: boolean;
    readonly blockError?: Error;
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
    blockTraffic: () => {
      events.push('blockTraffic',);
      return failing.includes('blockTraffic',) ? Promise.reject(blockError,) : Promise.resolve();
    },
    removeTrafficBlock: step('removeTrafficBlock',),
    trafficBlockPresent: () => Promise.resolve(earlierBlock,),
    manualSteps: () => {
      events.push('manualSteps',);
      return Promise.resolve(MANUAL_STEPS,);
    },
  };
  /**
   What the sequence threw,
   absent when it did not.
   */
  const thrown = await thrownBy({
    call: () => teardownKeepingTunnelOnFailure({
      config: configWith(table === undefined ? {} : { table, },),
      steps,
    },),
  },);
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
      name: 'runs the steps that can fail before any routing is removed, then the block, the policy rules, the lift of the block once the interface is gone, and the leftovers',
      fn: async () => {
        const { events, thrown, } = await run();
        expect(thrown,).toBe(undefined,);
        expect(events,).toEqual([
          ...CAN_FAIL_FIRST,
          'blockTraffic',
          'removePolicyRules',
          'removeBypassState',
          'removeKillSwitch',
          'deleteLink',
          'removeTrafficBlock',
          'revertDns',
          'removeOpenSnitchAllowance',
        ],);
      },
    },),
    it({
      name: 'skips the policy rules when Table is off, and still blocks before the first removal',
      fn: async () => {
        const { events, thrown, } = await run({ table: 'off', },);
        expect(thrown,).toBe(undefined,);
        expect(events.includes('removePolicyRules',),).toBe(false,);
        expect(events.includes('deleteLink',),).toBe(true,);
        expect(events.indexOf('blockTraffic',),).toBeLessThan(events.indexOf('removeBypassState',),);
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
      name: 'does not say nothing was removed when an earlier down removed routing and left its block: the retried down says the tunnel is not fully up and traffic is blocked',
      fn: async () => {
        const { thrown, } = await run({ failing: ['stopRouteWatcher',], earlierBlock: true, },);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('an earlier down already removed part of the routing of wgtest, so the tunnel is NOT fully up, and traffic is BLOCKED',);
        expect(message,).toContain('nft table inet wg-quicker-wgtest-blocked',);
        expect(message,).toContain(BLOCK_EXCEPTIONS,);
        expect(message,).toContain('sudo nft list table inet wg-quicker-wgtest-blocked',);
        expect(message,).toContain('this attempt: nothing was removed',);
        expect(message,).not.toContain('is still fully up',);
        expect(message,).toContain('then run `wg-quicker down wgtest` again',);
      },
    },),
    it({
      name: 'starts the route watcher again when the application watcher cannot be stopped, removes no routing, installs no block, and says what is uncertain',
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
      name: 'stops with the tunnel fully up, and a loud error naming the program and what to do, when the application watcher program is missing',
      fn: async () => {
        const { events, thrown, } = await run({ exemptionOutcome: { ok: false, report: 'wg-quicker-exempt was not found in /usr/local/bin and /usr/bin', }, },);
        expect(events,).toEqual([...CAN_FAIL_FIRST, 'restartRouteWatcher',],);
        expect((thrown as Error).name,).toBe('TeardownRefusedError',);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('could not stop the application watcher (Error: wg-quicker-exempt was not found in /usr/local/bin and /usr/bin)',);
        expect(message,).toContain('the tunnel wgtest is still fully up; no routing was removed; the route watcher was started again',);
        expect(message,).toContain('put it back where wg-quicker looks for it',);
        expect(message,).toContain('then run `wg-quicker down wgtest` again',);
      },
    },),
    it({
      name: 'stops with the tunnel fully up when the traffic block that protects the removal cannot be installed, and removes no routing',
      fn: async () => {
        const { events, thrown, } = await run({ failing: ['blockTraffic',], },);
        expect(events,).toEqual([...CAN_FAIL_FIRST, 'blockTraffic', 'restartRouteWatcher',],);
        expect((thrown as Error).name,).toBe('TeardownRefusedError',);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('could not install the traffic block that protects the removal of the routing (Error: blockTraffic failed)',);
        expect(message,).toContain('the tunnel wgtest is still fully up; no routing was removed',);
        expect(message,).toContain('the application watcher stays stopped',);
        expect(message,).toContain('sudo nft list ruleset',);
        // The by-hand lines belong to the failure to read what to let through, not to a failed nft.
        expect(events.includes('manualSteps',),).toBe(false,);
        for (const line of MANUAL_STEPS)
          expect(message,).not.toContain(line,);
      },
    },),
    it({
      name: 'stops with the tunnel fully up and the exact by-hand lines under the error when the block cannot read what the routing sends around the tunnel',
      fn: async () => {
        /**
         The error the block step raises when the state or the rules cannot be read.
         */
        const unreadable = Object.assign(new Error('could not read what the routing sends around the tunnel (state file damaged)',), { clearnetUnreadable: true, },);
        const { events, thrown, } = await run({ failing: ['blockTraffic',], blockError: unreadable, },);
        expect(events,).toEqual([...CAN_FAIL_FIRST, 'blockTraffic', 'restartRouteWatcher', 'manualSteps',],);
        expect((thrown as Error).name,).toBe('TeardownRefusedError',);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('the tunnel wgtest is still fully up; no routing was removed',);
        expect(message,).toContain('fix what the error names',);
        // Every by-hand line follows the fix line, each on a line of its own, and nothing is left out.
        /**
         The lines of the message after the fix line.
         */
        const lines = message.split('\n',);
        const fixIndex = lines.findIndex((line,) => line.startsWith('fix: ',),);
        expect(fixIndex,).toBeGreaterThan(-1,);
        expect(lines.slice(fixIndex + 1,),).toEqual([...MANUAL_STEPS,],);
        expect(lines[fixIndex],).toContain('then run `wg-quicker down wgtest` again',);
      },
    },),
    it({
      name: 'has blocked traffic before the first routing removal, so a down killed anywhere in the removal leaves traffic blocked and not outside the tunnel',
      fn: async () => {
        for (const failing of ['removePolicyRules', 'removeBypassState', 'removeKillSwitch', 'deleteLink',] as const) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own recording.
          const { events, } = await run({ failing: [failing,], },);
          expect(events.filter((event,) => event === 'blockTraffic',).length,).toBe(1,);
          // The policy rules are the first removal, and are recorded even when they are the step that fails.
          expect(events.indexOf('blockTraffic',),).toBeLessThan(events.indexOf('removePolicyRules',),);
          expect(events.includes(failing,),).toBe(true,);
          expect(events.includes('removeTrafficBlock',),).toBe(false,);
        }
      },
    },),
    it({
      name: 'says traffic is blocked, and how to finish, when a step of the routing removal fails, without a second block',
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
          expect(events.at(-1,),).toBe(failing,);
          expect((thrown as Error).name,).toBe('TeardownIncompleteError',);
          const message = messageOf({ thrown, },);
          expect(message,).toContain(`could not ${what} (Error: ${failing} failed)`,);
          expect(message,).toContain('traffic is BLOCKED (nft table inet wg-quicker-wgtest-blocked',);
          expect(message,).toContain('installed before any routing was removed',);
          expect(message,).toContain(BLOCK_EXCEPTIONS,);
          expect(message,).toContain('except what the routing sends around it by design',);
          expect(message,).toContain('run `wg-quicker down wgtest` again, with this build or with the build that started the tunnel',);
          expect(message,).toContain('sudo nft delete table inet wg-quicker-wgtest-blocked',);
        }
      },
    },),
    it({
      name: 'lifts the block once the interface is verified gone, before the other leftovers',
      fn: async () => {
        const { events, } = await run();
        expect(events.indexOf('removeTrafficBlock',),).toBe(events.indexOf('deleteLink',) + 1,);
      },
    },),
    it({
      name: 'says traffic is still blocked, and how to lift it, when the block cannot be lifted, and still runs the other leftovers',
      fn: async () => {
        const { events, thrown, } = await run({ failing: ['removeTrafficBlock',], },);
        expect(events.includes('revertDns',),).toBe(true,);
        expect(events.includes('removeOpenSnitchAllowance',),).toBe(true,);
        expect((thrown as Error).name,).toBe('TeardownIncompleteError',);
        const message = messageOf({ thrown, },);
        expect(message,).toContain('removed the tunnel but could not lift the traffic block (Error: removeTrafficBlock failed)',);
        expect(message,).toContain('traffic is still BLOCKED (nft table inet wg-quicker-wgtest-blocked',);
        expect(message,).toContain(BLOCK_EXCEPTIONS,);
        expect(message,).toContain('sudo nft delete table inet wg-quicker-wgtest-blocked',);
        expect(message,).not.toContain('nothing is blocked',);
      },
    },),
    it({
      name: 'reports a failure after the interface is gone without blocking anything, and still runs the other leftovers',
      fn: async () => {
        const { events, thrown, } = await run({ failing: ['revertDns',], },);
        expect(events.filter((event,) => event === 'blockTraffic',).length,).toBe(1,);
        expect(events.at(-1,),).toBe('removeOpenSnitchAllowance',);
        expect(events.includes('removeTrafficBlock',),).toBe(true,);
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
        expect(script,).toContain('add table inet wg-quicker-wgtest-blocked\nflush table inet wg-quicker-wgtest-blocked\n',);
        expect(script,).not.toContain('delete',);
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
        const refusal = await thrownBy({
          call: () => deleteLinkVerified({ config: configWith({ interfaceName: 'lo', },), },),
        },);
        expect(String(refusal,),).toContain('the interface lo still exists after `ip link delete`',);
      },
    },),
  ],
},);
