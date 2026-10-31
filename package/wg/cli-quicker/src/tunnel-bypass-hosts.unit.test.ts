import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hostSyncFailureText,
  refreshOrReport,
} from '../dist/final/node/bypass-host-refresh.mjs';
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
        const fresh: OwnedHostRule = { family: '-4', address: '203.0.113.9', };
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
              'github.com': { addresses: [{ family: '-4', address: '140.82.112.3', ttlSeconds: 60, }, { family: '-4', address: '203.0.113.9', ttlSeconds: 45, },], },
              'crates.io': { addresses: [{ family: '-4', address: '127.0.0.1', ttlSeconds: 10, },], },
              'pypi.org': { addresses: [], failure: 'no address for pypi.org: A ENOTFOUND', },
            },
          },),
          tools,
          absentCounts: new Map(),
        },);
        expect(operations,).toEqual(['add -4 203.0.113.9',],);
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
        const { tools, operations, } = fakeTools({ failAdd: ['203.0.113.9',], });
        /**
         Outcome when one add fails and one lookup throws.
         */
        const report = await syncUpdateHostRules({
          state: STATE,
          hosts: ['github.com', 'nodejs.org',],
          lookup: ({ host, },) => host === 'nodejs.org'
            ? Promise.reject(new Error('resolver down',))
            : Promise.resolve({ addresses: [{ family: '-4', address: '203.0.113.9', ttlSeconds: 60, },
              { family: '-4', address: '198.51.100.4', ttlSeconds: 60, },], },),
          tools,
          absentCounts: new Map(),
        },);
        expect(operations,).toEqual(['add -4 198.51.100.4',],);
        expect(report.failures.join('|',),).toContain('cannot add rule for 203.0.113.9',);
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
