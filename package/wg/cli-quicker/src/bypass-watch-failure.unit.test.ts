import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createWatcherDelivery,
  monitorFailureReport,
  watcherStoppedReport,
} from '../dist/final/node/bypass-watch-failure.mjs';
import {
  hostSyncFailureText,
  MAX_LISTED_FAILURES,
  refreshOrReport,
  runHostRefreshLoop,
  type GuardedRefresh,
} from '../dist/final/node/bypass-host-refresh.mjs';
import type { DeliveryCommand, } from '../dist/final/node/runtime-failure-report.mjs';
import type { BypassState, } from '../dist/final/node/tunnel-bypass-hosts.mjs';
import {
  POLICY_BYPASS_TABLE,
  POLICY_CLEARNET_HOSTS,
} from '../dist/final/node/update-policy-notice.mjs';

/**
 State claiming table 52000 and preference 50.
 */
const STATE: BypassState = {
  version: 2,
  interfaceName: 'wgtest0',
  mark: 100,
  table: 52_000,
  preference: 50,
  ownerId: 'owner',
  routes: [],
};

await describe({
  name: 'watcher failure reports',
  children: [
    it({
      name: 'name the bypass-table policy for a failed monitor, and both policies for a watcher that stopped',
      fn: async () => {
        /**
         Report of a failed monitor.
         */
        const monitor = monitorFailureReport({ error: new Error('ip exited 1',), },);
        expect(monitor,).toContain('POLICY NOT IN FORCE',);
        expect(monitor,).toContain(POLICY_BYPASS_TABLE,);
        expect(monitor,).toContain('ip exited 1',);
        expect(monitor,).toContain('will be restarted',);
        /**
         Report of a watcher that stopped.
         */
        const stopped = watcherStoppedReport({ error: new Error('state vanished',), },);
        expect(stopped,).toContain(POLICY_BYPASS_TABLE,);
        expect(stopped,).toContain(POLICY_CLEARNET_HOSTS,);
        expect(stopped,).toContain('state vanished',);
        expect(stopped,).toContain('run `wg-quicker up` again',);
      },
    },),
  ],
},);

await describe({
  name: createWatcherDelivery.name,
  children: [
    it({
      name: 'notifies the sudo caller by uid and gid, and journals even when no user is named',
      fn: async () => {
        /**
         Commands run for a watcher started through sudo.
         */
        const withUser: DeliveryCommand[] = [];
        await createWatcherDelivery({
          environment: { SUDO_UID: '1000', SUDO_GID: '1001', },
          run: (call,) => {
            withUser.push(call,);
            return Promise.resolve('',);
          },
        },)({ kind: 'k', line: 'POLICY NOT IN FORCE: x', },);
        expect(withUser.map((call,) => call.command,).includes('systemd-cat',),).toBe(true,);
        expect(withUser.length,).toBeGreaterThanOrEqual(2,);
        /**
         Commands run for a watcher without a sudo caller.
         */
        const withoutUser: DeliveryCommand[] = [];
        await createWatcherDelivery({
          environment: {},
          run: (call,) => {
            withoutUser.push(call,);
            return Promise.resolve('',);
          },
        },)({ kind: 'k', line: 'POLICY NOT IN FORCE: x', },);
        expect(withoutUser.map((call,) => call.command,),).toEqual(['systemd-cat',],);
      },
    },),
  ],
},);

await describe({
  name: 'the update-host refresh delivers loud reports',
  children: [
    it({
      name: 'delivers the loud text of a failed refresh and of a refresh with problems, and nothing for a clean one',
      fn: async () => {
        /**
         Loud lines delivered.
         */
        const delivered: string[] = [];
        /**
         Kinds delivered.
         */
        const kinds: string[] = [];
        /**
         Delivery that records the kind and the line.
         */
        const deliver = (call: { readonly kind: string; readonly line: string; },) => {
          delivered.push(call.line,);
          kinds.push(call.kind,);
          return Promise.resolve();
        };
        await refreshOrReport({
          state: STATE,
          absentCounts: new Map(),
          hosts: ['crates.io',],
          lookup: () => Promise.resolve({ addresses: [], },),
          tools: { list: () => Promise.resolve('[]',), add: () => Promise.resolve(), remove: () => Promise.resolve(), },
          readDependencies: () => Promise.reject(new Error('resolvectl is unavailable',),),
          deliver,
        },);
        expect(delivered,).toHaveLength(1,);
        expect(delivered[0],).toContain('POLICY NOT IN FORCE: clearnet hosts',);
        expect(delivered[0],).toContain('resolvectl is unavailable',);
        expect(kinds[0],).toBe('update-host refresh failed',);
        await refreshOrReport({
          state: STATE,
          absentCounts: new Map(),
          hosts: ['crates.io',],
          lookup: () => Promise.resolve({ addresses: [], },),
          tools: { list: () => Promise.resolve('[]',), add: () => Promise.resolve(), remove: () => Promise.resolve(), },
          readDependencies: () => Promise.resolve({ dnsServers: [], endpoints: [], },),
          deliver,
        },);
        expect(delivered,).toHaveLength(2,);
        expect(delivered[1],).toContain('sets no DNS server',);
        expect(kinds[1],).toBe('update-host problems',);
        expect(hostSyncFailureText({ report: { added: 0, removed: 0, failures: [], nextRefreshSeconds: 30, }, },),).toBe('',);
        /**
         Text for a refresh with far more problems than the cap names.
         */
        const long = hostSyncFailureText({
          report: {
            added: 0,
            removed: 0,
            failures: Array.from({ length: 500, }, (_unused, index,) => `problem number ${String(index,)}`,),
            nextRefreshSeconds: 30,
          },
        },);
        expect(long,).toContain('500 problem(s)',);
        expect(long,).toContain(`problem number ${String(MAX_LISTED_FAILURES - 1,)}`,);
        expect(long,).not.toContain(`problem number ${String(MAX_LISTED_FAILURES,)};`,);
        expect(long,).toContain(`and ${String(500 - MAX_LISTED_FAILURES,)} more`,);
        expect(long.length,).toBeLessThan(1_000,);
        await refreshOrReport({
          state: STATE,
          absentCounts: new Map(),
          hosts: ['crates.io',],
          lookup: () => Promise.resolve({ addresses: [{ family: '-4', address: '185.199.108.133', ttlSeconds: 60, },], },),
          tools: { list: () => Promise.resolve('[]',), add: () => Promise.resolve(), remove: () => Promise.resolve(), },
          readDependencies: () => Promise.resolve({ dnsServers: ['9.9.9.9',], endpoints: [], },),
          deliver,
        },);
        expect(delivered,).toHaveLength(2,);
      },
    },),
    it({
      name: 'passes the delivery and the same counters to every round of the loop',
      fn: async () => {
        /**
         Stops the loop after the second round.
         */
        const controller = new AbortController();
        /**
         Arguments of each round.
         */
        const rounds: { readonly deliver: unknown; readonly absentCounts: unknown; readonly state: unknown; }[] = [];
        /**
         Delivery handed to the loop.
         */
        const deliver = () => Promise.resolve();
        /**
         Refresh that records its arguments.
         */
        const refresh: GuardedRefresh = (call,) => {
          rounds.push({ deliver: call.deliver, absentCounts: call.absentCounts, state: call.state, },);
          if (rounds.length >= 2)
            controller.abort();
          return Promise.resolve(0,);
        };
        await runHostRefreshLoop({
          state: STATE,
          signal: controller.signal,
          deliver,
          refresh,
          readHosts: () => Promise.resolve({ kind: 'read', hosts: ['crates.io',], },),
        },);
        expect(rounds,).toHaveLength(2,);
        expect(rounds[0]?.deliver,).toBe(deliver,);
        expect(rounds[1]?.deliver,).toBe(deliver,);
        expect(rounds[0]?.absentCounts,).toBe(rounds[1]?.absentCounts,);
        expect(rounds[0]?.state,).toBe(STATE,);
        /**
         Rounds of a loop that has no delivery of its own.
         */
        const undelivered: unknown[] = [];
        /**
         Stops that loop after its first round.
         */
        const withoutDelivery = new AbortController();
        await runHostRefreshLoop({
          state: STATE,
          signal: withoutDelivery.signal,
          refresh: (call,) => {
            undelivered.push(call.deliver,);
            withoutDelivery.abort();
            return Promise.resolve(0,);
          },
          readHosts: () => Promise.resolve({ kind: 'read', hosts: ['crates.io',], },),
        },);
        expect(undelivered,).toEqual([undefined,],);
      },
    },),
  ],
},);
