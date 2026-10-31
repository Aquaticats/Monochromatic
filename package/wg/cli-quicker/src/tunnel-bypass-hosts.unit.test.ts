import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hostSyncFailureText,
  noDnsFailure,
  refreshOrReport,
  refreshUpdateHostRules,
  runHostRefreshLoop,
  sleepUnlessAborted,
} from '../dist/final/node/bypass-host-refresh.mjs';
import { MAX_CONCURRENT_IP_COMMANDS, } from '../dist/final/node/host-rule-limits.data.mjs';
import {
  ABSENT_REFRESHES_BEFORE_REMOVAL,
  hostRuleArguments,
  isPublicAddress,
  normalizeAddress,
  parseOwnedHostRules,
  planHostRules,
  refreshSeconds,
  type BypassState,
  type HostLookupResult,
  type HostRuleTools,
  type OwnedHostRule,
} from '../dist/final/node/tunnel-bypass-hosts.mjs';
import { UPDATE_HOSTS, } from '../dist/final/node/update-hosts.data.mjs';
import {
  removeUpdateHostRules,
  syncUpdateHostRules,
} from '../dist/final/node/tunnel-bypass-hosts-sync.mjs';


/**
 State claiming table 52000 and preference 50.
 */
const STATE: BypassState = {
  version: 2,
  interfaceName: 'wg0',
  mark: 100,
  table: 52_000,
  preference: 50,
  ownerId: 'owner',
  routes: [],
};

/**
 Kernel listing for IPv4 holding the fwmark rule,
 two owned `to` rules,
 and look-alikes that are not owned.
 */
const LISTING_V4 = JSON.stringify([
  { priority: 50, src: 'all', fwmark: '0x64', table: '52000', protocol: '201', },
  { priority: 50, src: 'all', dst: '140.82.112.3', dstlen: 32, table: '52000', protocol: '201', },
  { priority: 50, src: 'all', dst: '151.101.0.223', dstlen: 32, table: '52000', protocol: '201', },
  { priority: 50, src: 'all', dst: '10.0.0.1', dstlen: 32, table: '60000', protocol: '201', },
  { priority: 51, src: 'all', dst: '10.0.0.2', dstlen: 32, table: '52000', protocol: '201', },
  { priority: 50, src: 'all', dst: '10.0.0.3', dstlen: 32, table: '52000', },
],);

/**
 Kernel listing for IPv6 with one owned rule in an expanded spelling.
 */
const LISTING_V6 = JSON.stringify([
  { priority: 50, src: 'all', dst: '2606:4700:0:0:0:0:6810:84e5', dstlen: 128, table: '52000', protocol: '201', },
],);

/**
 Builds fake rule tools over listings and records mutations.

 @param failAdd - Addresses whose add fails.

 @returns Tools and the recorded operations.

 @example
 ```ts
 fakeTools({ failAdd: [] });
 ```
 */
function fakeTools(
  { failAdd = [], }: { readonly failAdd?: readonly string[]; } = {},
): { readonly tools: HostRuleTools; readonly operations: string[]; } {
  /**
   Recorded operations in order.
   */
  const operations: string[] = [];
  return {
    operations,
    tools: {
      list: ({ family, },) => Promise.resolve(family === '-4' ? LISTING_V4 : LISTING_V6,),
      add: ({ rule, },) => {
        if (failAdd.includes(rule.address,))
          return Promise.reject(new Error(`add refused for ${rule.address}`,),);
        operations.push(`add ${rule.family} ${rule.address}`,);
        return Promise.resolve();
      },
      remove: ({ rule, },) => {
        operations.push(`remove ${rule.family} ${rule.address}`,);
        return Promise.resolve();
      },
    },
  };
}

/**
 Builds a resolver from a table.

 @param table - Answers by host.

 @returns Resolver.

 @example
 ```ts
 tableLookup({ table: { 'crates.io': { addresses: [] } } });
 ```
 */
function tableLookup(
  { table, }: { readonly table: Readonly<Record<string, HostLookupResult>>; },
): (call: { readonly host: string; },) => Promise<HostLookupResult> {
  return ({ host, },) => Promise.resolve(table[host] ?? { addresses: [], failure: `no address for ${host}`, },);
}

await describe({
  name: normalizeAddress.name,
  children: [
    it({
      name: 'compares IPv6 spellings equal and rejects malformed or mapped literals',
      fn: async () => {
        expect(normalizeAddress({ family: '-6', text: '2606:4700::6810:84e5', },),).toBe(
          normalizeAddress({ family: '-6', text: '2606:4700:0:0:0:0:6810:84E5', },),
        );
        expect(normalizeAddress({ family: '-6', text: '::1', },),).toBe('0:0:0:0:0:0:0:1',);
        expect(normalizeAddress({ family: '-6', text: '::ffff:1.2.3.4', },),).toBe('',);
        expect(normalizeAddress({ family: '-6', text: '1::2::3', },),).toBe('',);
        expect(normalizeAddress({ family: '-6', text: 'not-an-address', },),).toBe('',);
        expect(normalizeAddress({ family: '-4', text: '203.0.113.7', },),).toBe('203.0.113.7',);
        expect(normalizeAddress({ family: '-4', text: '2606:4700::1', },),).toBe('',);
        expect(normalizeAddress({ family: '-4', text: '203.0.113.256', },),).toBe('',);
      },
    },),
  ],
},);

await describe({
  name: isPublicAddress.name,
  children: [
    it({
      name: 'accepts public unicast addresses and refuses loopback, private, link-local, shared and multicast ones',
      fn: async () => {
        for (const address of ['140.82.112.3', '172.32.0.1', '100.128.0.1', '8.8.8.8',])
          expect(isPublicAddress({ family: '-4', address, },),).toBe(true,);
        for (
          const address of [
            '0.0.0.0',
            '10.1.1.1',
            '127.0.0.1',
            '169.254.1.1',
            '172.16.0.1',
            '172.31.255.255',
            '192.168.1.1',
            '100.64.0.1',
            '224.0.0.1',
            '255.255.255.255',
          ]
        )
          expect(isPublicAddress({ family: '-4', address, },),).toBe(false,);
        expect(isPublicAddress({ family: '-6', address: normalizeAddress({ family: '-6', text: '2606:4700::6810:84e5', },), },),).toBe(true,);
        for (const text of ['::', '::1', 'fe80::1', 'fc00::1', 'fd12::1', 'ff02::1',])
          expect(isPublicAddress({ family: '-6', address: normalizeAddress({ family: '-6', text, },), },),).toBe(false,);
        for (const address of ['192.0.0.8', '192.0.2.1', '198.51.100.4', '203.0.113.9', '198.18.0.1', '198.19.255.254', '192.88.99.1',])
          expect(isPublicAddress({ family: '-4', address, },),).toBe(false,);
        for (const address of ['198.17.255.255', '198.20.0.0', '203.0.114.1', '192.0.1.1',])
          expect(isPublicAddress({ family: '-4', address, },),).toBe(true,);
        for (const text of ['::ffff:7f00:1', '::ffff:0:0', '64:ff9b::808:808', '2002:c000:204::1', '2001:db8::1', '100::1',])
          expect(isPublicAddress({ family: '-6', address: normalizeAddress({ family: '-6', text, },), },),).toBe(false,);
        for (const text of ['2003::1', '2001:db9::1', '64:ff9c::1', '100:0:0:1::1',])
          expect(isPublicAddress({ family: '-6', address: normalizeAddress({ family: '-6', text, },), },),).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: refreshSeconds.name,
  children: [
    it({
      name: 'clamps the smallest TTL into the refresh window',
      fn: async () => {
        expect(refreshSeconds({ ttlSeconds: 0, },),).toBe(30,);
        expect(refreshSeconds({ ttlSeconds: 5, },),).toBe(30,);
        expect(refreshSeconds({ ttlSeconds: 120, },),).toBe(120,);
        expect(refreshSeconds({ ttlSeconds: 86_400, },),).toBe(900,);
      },
    },),
  ],
},);

await describe({
  name: parseOwnedHostRules.name,
  children: [
    it({
      name: 'reads only to-rules with the private protocol, the claimed table and the claimed preference',
      fn: async () => {
        expect(parseOwnedHostRules({ text: LISTING_V4, family: '-4', state: STATE, },),).toEqual([
          { family: '-4', address: '140.82.112.3', },
          { family: '-4', address: '151.101.0.223', },
        ],);
        expect(parseOwnedHostRules({ text: LISTING_V6, family: '-6', state: STATE, },),).toEqual([
          { family: '-6', address: '2606:4700:0:0:0:0:6810:84e5', },
        ],);
        expect(parseOwnedHostRules({ text: '', family: '-4', state: STATE, },),).toEqual([],);
        expect(parseOwnedHostRules({ text: '{}', family: '-4', state: STATE, },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: planHostRules.name,
  children: [
    it({
      name: 'adds missing rules at once and removes a vanished rule only after consecutive absent refreshes',
      fn: async () => {
        /**
         Rule that stays wanted.
         */
        const kept: OwnedHostRule = { family: '-4', address: '140.82.112.3', };
        /**
         Rule that DNS stopped returning.
         */
        const gone: OwnedHostRule = { family: '-4', address: '151.101.0.223', };
        /**
         Rule that DNS started returning.
         */
        const fresh: OwnedHostRule = { family: '-4', address: '185.199.108.133', };
        /**
         Counters shared between refreshes.
         */
        const absentCounts = new Map<string, number>();
        /**
         First refresh after the answer changed.
         */
        const first = planHostRules({ desired: [kept, fresh,], owned: [kept, gone,], absentCounts, },);
        expect(first.add,).toEqual([fresh,],);
        expect(first.remove,).toEqual([],);
        expect(ABSENT_REFRESHES_BEFORE_REMOVAL,).toBe(2,);
        /**
         Second consecutive absence.
         */
        const second = planHostRules({ desired: [kept, fresh,], owned: [kept, gone, fresh,], absentCounts, },);
        expect(second.remove,).toEqual([gone,],);
        /**
         An address that returns resets its counter.
         */
        const reset = new Map<string, number>();
        planHostRules({ desired: [kept,], owned: [kept, gone,], absentCounts: reset, },);
        planHostRules({ desired: [kept, gone,], owned: [kept, gone,], absentCounts: reset, },);
        expect(planHostRules({ desired: [kept,], owned: [kept, gone,], absentCounts: reset, },).remove,).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: hostRuleArguments.name,
  children: [
    it({
      name: 'builds a destination rule in the claimed table, preference and protocol',
      fn: async () => {
        expect(hostRuleArguments({ verb: 'add', rule: { family: '-4', address: '140.82.112.3', }, state: STATE, },),).toEqual([
          '-4',
          'rule',
          'add',
          'to',
          '140.82.112.3/32',
          'table',
          '52000',
          'pref',
          '50',
          'protocol',
          '201',
        ],);
        expect(hostRuleArguments({ verb: 'delete', rule: { family: '-6', address: '2606:4700:0:0:0:0:6810:84e5', }, state: STATE, },),)
          .toContain('2606:4700:0:0:0:0:6810:84e5/128',);
      },
    },),
  ],
},);

await describe({
  name: syncUpdateHostRules.name,
  children: [
    it({
      name: 'adds public answers, refuses non-public ones, and reports them instead of throwing',
      fn: async () => {
        const { tools, operations, } = fakeTools();
        /**
         Outcome of a reconciliation whose answers include a loopback address.
         */
        const report = await syncUpdateHostRules({
          state: STATE,
          hosts: ['github.com', 'crates.io', 'pypi.org',],
          lookup: tableLookup({
            table: {
              'github.com': { addresses: [{ family: '-4', address: '140.82.112.3', ttlSeconds: 60, }, { family: '-4', address: '185.199.108.133', ttlSeconds: 45, },], },
              'crates.io': { addresses: [{ family: '-4', address: '127.0.0.1', ttlSeconds: 10, },], },
              'pypi.org': { addresses: [], failure: 'no address for pypi.org: A ENOTFOUND', },
            },
          },),
          tools,
          absentCounts: new Map(),
        },);
        expect(operations,).toEqual(['add -4 185.199.108.133',],);
        expect(report.added,).toBe(1,);
        expect(report.failures,).toHaveLength(2,);
        expect(report.failures.join('|',),).toContain('refused non-public answer 127.0.0.1',);
        expect(report.failures.join('|',),).toContain('pypi.org',);
        expect(report.nextRefreshSeconds,).toBe(45,);
        expect(hostSyncFailureText({ report, },),).toContain('POLICY NOT IN FORCE',);
        expect(hostSyncFailureText({ report, },),).toContain('update hosts',);
      },
    },),
    it({
      name: 'reports a failing rule command and a throwing resolver without changing other rules',
      fn: async () => {
        const { tools, operations, } = fakeTools({ failAdd: ['185.199.108.133',], });
        /**
         Outcome when one add fails and one lookup throws.
         */
        const report = await syncUpdateHostRules({
          state: STATE,
          hosts: ['github.com', 'nodejs.org',],
          lookup: ({ host, },) => host === 'nodejs.org'
            ? Promise.reject(new Error('resolver down',))
            : Promise.resolve({ addresses: [{ family: '-4', address: '185.199.108.133', ttlSeconds: 60, },
              { family: '-4', address: '185.199.109.133', ttlSeconds: 60, },], },),
          tools,
          absentCounts: new Map(),
        },);
        expect(operations,).toEqual(['add -4 185.199.109.133',],);
        expect(report.failures.join('|',),).toContain('cannot add rule for 185.199.108.133',);
        expect(report.failures.join('|',),).toContain('resolver down',);
      },
    },),
    it({
      name: 'keeps existing rules through a DNS outage for one refresh and says nothing is wrong when answers are clean',
      fn: async () => {
        const { tools, operations, } = fakeTools();
        /**
         Counters carried between the two refreshes.
         */
        const absentCounts = new Map<string, number>();
        /**
         Resolver that has stopped answering.
         */
        const lookup = tableLookup({ table: {}, },);
        await syncUpdateHostRules({ state: STATE, hosts: ['github.com',], lookup, tools, absentCounts, },);
        expect(operations.filter((operation,) => operation.startsWith('remove',)),).toEqual([],);
        await syncUpdateHostRules({ state: STATE, hosts: ['github.com',], lookup, tools, absentCounts, },);
        expect(operations.filter((operation,) => operation.startsWith('remove',)),).toHaveLength(3,);
        /**
         A clean refresh reports no failure text.
         */
        const clean = await syncUpdateHostRules({
          state: STATE,
          hosts: ['github.com',],
          lookup: tableLookup({ table: { 'github.com': { addresses: [{ family: '-4', address: '140.82.112.3', ttlSeconds: 300, },], }, }, },),
          tools: fakeTools().tools,
          absentCounts: new Map(),
        },);
        expect(hostSyncFailureText({ report: clean, },),).toBe('',);
      },
    },),
  ],
},);

await describe({
  name: removeUpdateHostRules.name,
  children: [
    it({
      name: 'removes only the rules the lifecycle owns',
      fn: async () => {
        const { tools, operations, } = fakeTools();
        await removeUpdateHostRules({ state: STATE, tools, },);
        expect(operations.toSorted(),).toEqual([
          'remove -4 140.82.112.3',
          'remove -4 151.101.0.223',
          'remove -6 2606:4700:0:0:0:0:6810:84e5',
        ],);
      },
    },),
  ],
},);

await describe({
  name: 'update-host reconciliation limits',
  children: [
    it({
      name: 'never runs more than the allowed number of rule commands at once, however many rules are added',
      fn: async () => {
        /**
         Commands running now.
         */
        let running = 0;
        /**
         Most commands seen running together.
         */
        let peak = 0;
        /**
         Rule tools whose listings are empty and whose adds take a moment.
         */
        const slow: HostRuleTools = {
          list: () => Promise.resolve('[]',),
          add: async () => {
            running += 1;
            peak = Math.max(peak, running,);
            await new Promise((resolve,) => {
              setTimeout(resolve, 5,);
            },);
            running -= 1;
          },
          remove: () => Promise.resolve(),
        };
        /**
         Reconciliation of forty-five new addresses.
         */
        const report = await syncUpdateHostRules({
          state: STATE,
          hosts: ['big.example',],
          lookup: () => Promise.resolve({
            addresses: Array.from({ length: 45, }, (_unused, index,) => ({
              family: '-4' as const,
              address: `140.82.112.${String(index,)}`,
              ttlSeconds: 60,
            }),),
          },),
          tools: slow,
          absentCounts: new Map(),
        },);
        expect(report.added,).toBe(45,);
        expect(peak,).toBe(MAX_CONCURRENT_IP_COMMANDS,);
      },
    },),
    it({
      name: 'adds no rule for a protected address and reports the refusal',
      fn: async () => {
        const { tools, operations, } = fakeTools();
        /**
         Reconciliation where one answer is the tunnel's own DNS server.
         */
        const report = await syncUpdateHostRules({
          state: STATE,
          hosts: ['github.com',],
          lookup: tableLookup({
            table: {
              'github.com': { addresses: [{ family: '-4', address: '1.1.1.1', ttlSeconds: 60, }, { family: '-4', address: '185.199.108.133', ttlSeconds: 60, },], },
            },
          },),
          tools,
          absentCounts: new Map(),
          protectedKeys: new Set(['-4|1.1.1.1',],),
        },);
        expect(operations,).toEqual(['add -4 185.199.108.133',],);
        expect(report.failures.join('|',),).toContain('refused 1.1.1.1 for github.com',);
      },
    },),
    it({
      name: 'puts the preflight failures first in the report',
      fn: async () => {
        const { tools, } = fakeTools();
        /**
         Reconciliation with a failure found before any lookup.
         */
        const report = await syncUpdateHostRules({
          state: STATE,
          hosts: [],
          lookup: tableLookup({ table: {}, },),
          tools,
          absentCounts: new Map(),
          preflightFailures: ['found early',],
        },);
        expect(report.failures[0],).toBe('found early',);
      },
    },),
  ],
},);

await describe({
  name: refreshUpdateHostRules.name,
  children: [
    it({
      name: 'installs no host rule, asks no resolver and retries soon when the tunnel sets no DNS server',
      fn: async () => {
        const { tools, operations, } = fakeTools();
        /**
         Hosts the resolver was asked about.
         */
        const asked: string[] = [];
        /**
         Seconds until the next refresh.
         */
        const seconds = await refreshUpdateHostRules({
          state: STATE,
          absentCounts: new Map(),
          lookup: ({ host, },) => {
            asked.push(host,);
            return Promise.resolve({ addresses: [{ family: '-4', address: '185.199.108.133', ttlSeconds: 60, },], },);
          },
          tools,
          readDependencies: () => Promise.resolve({ dnsServers: [], endpoints: [], },),
        },);
        expect(asked,).toEqual([],);
        expect(operations.filter((operation,) => operation.startsWith('add',),),).toEqual([],);
        expect(seconds,).toBe(30,);
      },
    },),
    it({
      name: 'names the missing DNS server in the loud report of every refresh',
      fn: async () => {
        const { tools, } = fakeTools();
        /**
         Reports of two consecutive refreshes without DNS.
         */
        const reports = await Promise.all([1, 2,].map(async () => await syncUpdateHostRules({
          state: STATE,
          hosts: [],
          lookup: tableLookup({ table: {}, },),
          tools,
          absentCounts: new Map(),
          preflightFailures: [noDnsFailure({ interfaceName: 'wg0', },),],
        },),),);
        for (const report of reports) {
          expect(hostSyncFailureText({ report, },),).toContain('POLICY NOT IN FORCE: update hosts',);
          expect(hostSyncFailureText({ report, },),).toContain('the tunnel wg0 sets no DNS server',);
          expect(hostSyncFailureText({ report, },),).toContain('the process watch and the toolchain paths still run',);
        }
      },
    },),
    it({
      name: 'asks the whole update-host list and refuses the tunnel\'s own addresses when DNS is set',
      fn: async () => {
        const { tools, operations, } = fakeTools();
        /**
         Hosts the resolver was asked about.
         */
        const asked: string[] = [];
        /**
         Seconds until the next refresh.
         */
        const seconds = await refreshUpdateHostRules({
          state: STATE,
          absentCounts: new Map(),
          lookup: ({ host, },) => {
            asked.push(host,);
            return Promise.resolve({
              addresses: [
                { family: '-4', address: '9.9.9.9', ttlSeconds: 120, },
                { family: '-4', address: '185.199.108.133', ttlSeconds: 90, },
              ],
            },);
          },
          tools,
          readDependencies: () => Promise.resolve({ dnsServers: ['9.9.9.9',], endpoints: [], },),
        },);
        expect(asked.toSorted(),).toEqual(UPDATE_HOSTS.toSorted(),);
        expect(operations.filter((operation,) => operation.startsWith('add',),),).toEqual(['add -4 185.199.108.133',],);
        expect(seconds,).toBe(90,);
      },
    },),
    it({
      name: 'changes nothing and retries soon when the tunnel\'s dependencies cannot be read',
      fn: async () => {
        const { tools, operations, } = fakeTools();
        expect(await refreshOrReport({
          state: STATE,
          absentCounts: new Map(),
          lookup: tableLookup({ table: {}, },),
          tools,
          readDependencies: () => Promise.reject(new Error('resolvectl is unavailable',),),
        },),).toBe(30,);
        expect(operations,).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: sleepUnlessAborted.name,
  children: [
    it({
      name: 'ends at once on an aborted signal and early when the signal aborts during the wait',
      fn: async () => {
        /**
         Moment the first wait began.
         */
        const first = performance.now();
        await sleepUnlessAborted({ seconds: 60, signal: AbortSignal.abort(), },);
        expect(performance.now() - first,).toBeLessThan(1_000,);
        /**
         Controller aborted shortly after the second wait starts.
         */
        const controller = new AbortController();
        setTimeout(() => {
          controller.abort();
        }, 20,);
        /**
         Moment the second wait began.
         */
        const second = performance.now();
        await sleepUnlessAborted({ seconds: 60, signal: controller.signal, },);
        expect(performance.now() - second,).toBeLessThan(2_000,);
      },
    },),
    it({
      name: 'lets the refresh loop end without a refresh when it is already stopped',
      fn: async () => {
        await runHostRefreshLoop({ state: STATE, signal: AbortSignal.abort(), },);
      },
    },),
  ],
},);

await describe({
  name: refreshOrReport.name,
  children: [
    it({
      name: 'turns a failing rule listing into a retry delay instead of throwing',
      fn: async () => {
        /**
         Rule tools whose listing fails.
         */
        const broken: HostRuleTools = {
          list: () => Promise.reject(new Error('ip is unavailable',)),
          add: () => Promise.resolve(),
          remove: () => Promise.resolve(),
        };
        expect(await refreshOrReport({
          state: STATE,
          absentCounts: new Map(),
          lookup: tableLookup({ table: {}, },),
          tools: broken,
        },),).toBe(30,);
      },
    },),
  ],
},);
