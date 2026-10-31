import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { filterHostAnswers, } from '../dist/final/node/host-answer-filter.mjs';
import {
  MAX_ADDRESSES_PER_HOST,
  MAX_ADDRESSES_TOTAL,
} from '../dist/final/node/host-rule-limits.data.mjs';

/**
 Builds an answer of IPv4 addresses `base.0` to `base.<count - 1>`.

 @param host - Host name.

 @param base - First three octets,
 such as `140.82.112`.

 @param count - Number of addresses.

 @returns The host's answer.

 @example
 ```ts
 numbered({ host: 'a.example', base: '140.82.112', count: 3 });
 ```
 */
function numbered(
  {
    host,
    base,
    count,
  }: {
    readonly host: string;
    readonly base: string;
    readonly count: number;
  },
) {
  return {
    host,
    result: {
      addresses: Array.from({ length: count, }, (_unused, index,) => ({
        family: '-4' as const,
        address: `${base}.${String(index,)}`,
        ttlSeconds: 60,
      }),),
    },
  };
}

await describe({
  name: filterHostAnswers.name,
  children: [
    it({
      name: 'keeps public addresses, merges the hosts that share one, and keeps the shortest TTL',
      fn: async () => {
        /**
         Two hosts answering one shared address.
         */
        const filtered = filterHostAnswers({
          answers: [
            { host: 'pypi.org', result: { addresses: [{ family: '-4', address: '151.101.0.223', ttlSeconds: 300, },], }, },
            { host: 'files.pythonhosted.org', result: { addresses: [{ family: '-4', address: '151.101.0.223', ttlSeconds: 45, },], }, },
          ],
          protectedKeys: new Set(),
        },);
        expect(filtered.failures,).toEqual([],);
        expect(filtered.wanted,).toEqual([{
          rule: { family: '-4', address: '151.101.0.223', },
          ttlSeconds: 45,
          hosts: ['pypi.org', 'files.pythonhosted.org',],
        },],);
      },
    },),
    it({
      name: 'refuses every reserved range that is not public unicast, naming the host',
      fn: async () => {
        for (
          const address of [
            '192.0.0.1',
            '192.0.2.1',
            '198.51.100.4',
            '203.0.113.9',
            '198.18.0.1',
            '198.19.255.255',
            '192.88.99.1',
          ]
        ) {
          /**
           Outcome for one reserved IPv4 address.
           */
          const filtered = filterHostAnswers({
            answers: [{ host: 'a.example', result: { addresses: [{ family: '-4', address, ttlSeconds: 60, },], }, },],
            protectedKeys: new Set(),
          },);
          expect(filtered.wanted,).toEqual([],);
          expect(filtered.failures,).toEqual([`refused non-public answer ${address} for a.example`,],);
        }
        for (
          const address of [
            '::ffff:7f00:1',
            '64:ff9b::808:808',
            '2002:c000:204::1',
            '2001:db8::1',
            '100::1',
          ]
        ) {
          /**
           Outcome for one reserved IPv6 address.
           */
          const filtered = filterHostAnswers({
            answers: [{ host: 'a.example', result: { addresses: [{ family: '-6', address, ttlSeconds: 60, },], }, },],
            protectedKeys: new Set(),
          },);
          expect(filtered.wanted,).toEqual([],);
          expect(filtered.failures,).toHaveLength(1,);
          expect(filtered.failures[0],).toContain('refused non-public answer',);
        }
      },
    },),
    it({
      name: 'still accepts real public addresses next to the reserved ones',
      fn: async () => {
        /**
         Real public addresses of both families.
         */
        const filtered = filterHostAnswers({
          answers: [{
            host: 'a.example',
            result: {
              addresses: [
                { family: '-4', address: '140.82.112.3', ttlSeconds: 60, },
                { family: '-4', address: '185.199.108.133', ttlSeconds: 60, },
                { family: '-6', address: '2606:4700::6810:84e5', ttlSeconds: 60, },
              ],
            },
          },],
          protectedKeys: new Set(),
        },);
        expect(filtered.failures,).toEqual([],);
        expect(filtered.wanted,).toHaveLength(3,);
      },
    },),
    it({
      name: 'refuses the tunnel\'s DNS servers and peer endpoints, however they are spelled, and says why',
      fn: async () => {
        /**
         Answer naming the tunnel's own DNS server, its endpoint and an ordinary address.
         */
        const filtered = filterHostAnswers({
          answers: [{
            host: 'a.example',
            result: {
              addresses: [
                { family: '-4', address: '1.1.1.1', ttlSeconds: 60, },
                { family: '-6', address: '2606:4700:4700::1111', ttlSeconds: 60, },
                { family: '-4', address: '140.82.112.3', ttlSeconds: 60, },
              ],
            },
          },],
          protectedKeys: new Set(['-4|1.1.1.1', '-6|2606:4700:4700:0:0:0:0:1111',],),
        },);
        expect(filtered.wanted.map((item,) => item.rule.address,),).toEqual(['140.82.112.3',],);
        expect(filtered.failures,).toHaveLength(2,);
        for (const failure of filtered.failures)
          expect(failure,).toContain('the tunnel depends on that address as a DNS server or a peer endpoint',);
      },
    },),
    it({
      name: 'cuts one host at the per-host cap and says how many it answered',
      fn: async () => {
        /**
         A host answering more addresses than the cap.
         */
        const filtered = filterHostAnswers({
          answers: [numbered({ host: 'big.example', base: '140.82.112', count: MAX_ADDRESSES_PER_HOST + 6, },),],
          protectedKeys: new Set(),
        },);
        expect(filtered.wanted,).toHaveLength(MAX_ADDRESSES_PER_HOST,);
        expect(filtered.failures,).toEqual([
          `big.example answered ${String(MAX_ADDRESSES_PER_HOST + 6,)} addresses; only the first ${String(MAX_ADDRESSES_PER_HOST,)} get rules`,
        ],);
      },
    },),
    it({
      name: 'keeps the first addresses at the total cap and names how many use the tunnel instead',
      fn: async () => {
        /**
         Seven hosts of sixty-four distinct addresses each,
         more than the total cap.
         */
        const answers = Array.from({ length: 7, }, (_unused, index,) => numbered({
          host: `host${String(index,)}.example`,
          base: `151.101.${String(index,)}`,
          count: MAX_ADDRESSES_PER_HOST,
        },),);
        /**
         Rules after the cap.
         */
        const filtered = filterHostAnswers({ answers, protectedKeys: new Set(), },);
        expect(filtered.wanted,).toHaveLength(MAX_ADDRESSES_TOTAL,);
        expect(filtered.failures,).toEqual([
          `the update hosts answered ${String(7 * MAX_ADDRESSES_PER_HOST,)} distinct addresses; only the first ${String(MAX_ADDRESSES_TOTAL,)} get rules `
            + `and the other ${String((7 * MAX_ADDRESSES_PER_HOST) - MAX_ADDRESSES_TOTAL,)} use the tunnel`,
        ],);
        expect(filtered.wanted[0]?.rule.address,).toBe('151.101.0.0',);
      },
    },),
    it({
      name: 'carries resolver failures and malformed answers into the failures',
      fn: async () => {
        /**
         A failed host and a malformed address.
         */
        const filtered = filterHostAnswers({
          answers: [
            { host: 'down.example', result: { addresses: [], failure: 'no address for down.example', }, },
            { host: 'odd.example', result: { addresses: [{ family: '-4', address: '999.1.1.1', ttlSeconds: 60, },], }, },
          ],
          protectedKeys: new Set(),
        },);
        expect(filtered.wanted,).toEqual([],);
        expect(filtered.failures,).toEqual([
          'no address for down.example',
          'ignored malformed -4 answer 999.1.1.1 for odd.example',
        ],);
      },
    },),
  ],
},);
